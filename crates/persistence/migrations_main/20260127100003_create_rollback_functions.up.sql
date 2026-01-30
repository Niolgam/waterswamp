-- Migration: Funções de Rollback para Requisições
-- ============================================================================
-- Permite reverter requisições para estados anteriores de forma controlada
-- ============================================================================

-- ============================================================================
-- 1. FUNÇÃO PARA LISTAR PONTOS DE ROLLBACK DISPONÍVEIS
-- ============================================================================

CREATE OR REPLACE FUNCTION fn_list_rollback_points(
    p_requisition_id UUID,
    p_limit INTEGER DEFAULT 20
) RETURNS TABLE (
    history_id UUID,
    operation TEXT,
    status_after TEXT,
    performed_at TIMESTAMPTZ,
    performed_by_name TEXT,
    changed_fields TEXT[],
    can_rollback BOOLEAN,
    rollback_blocked_reason TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        rh.id,
        rh.operation::TEXT,
        rh.status_after,
        rh.performed_at,
        rh.performed_by_name,
        rh.changed_fields,
        -- Verificar se pode fazer rollback para este ponto
        CASE 
            WHEN rh.is_rollback_point = FALSE THEN FALSE
            WHEN rh.operation IN ('DELETE', 'SOFT_DELETE') THEN FALSE
            -- Não pode rollback se já houve movimentações de estoque
            WHEN EXISTS (
                SELECT 1 FROM stock_movements sm 
                WHERE sm.requisition_id = p_requisition_id
                  AND sm.movement_date > rh.performed_at
            ) THEN FALSE
            ELSE TRUE
        END,
        -- Razão do bloqueio
        CASE 
            WHEN rh.is_rollback_point = FALSE THEN 'Estado marcado como não-reversível'
            WHEN rh.operation IN ('DELETE', 'SOFT_DELETE') THEN 'Não é possível reverter para estado de exclusão'
            WHEN EXISTS (
                SELECT 1 FROM stock_movements sm 
                WHERE sm.requisition_id = p_requisition_id
                  AND sm.movement_date > rh.performed_at
            ) THEN 'Existem movimentações de estoque posteriores a este ponto'
            ELSE NULL
        END
    FROM requisition_history rh
    WHERE rh.requisition_id = p_requisition_id
      AND rh.data_after IS NOT NULL -- Precisa ter estado para restaurar
    ORDER BY rh.performed_at DESC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================================
-- 2. FUNÇÃO PRINCIPAL DE ROLLBACK
-- ============================================================================

CREATE OR REPLACE FUNCTION fn_rollback_requisition(
    p_requisition_id UUID,
    p_to_history_id UUID,
    p_reason TEXT,
    p_user_id UUID
) RETURNS JSONB AS $$
DECLARE
    v_target_state JSONB;
    v_current_state JSONB;
    v_current_status TEXT;
    v_target_status TEXT;
    v_history_performed_at TIMESTAMPTZ;
    v_has_stock_movements BOOLEAN;
    v_result JSONB;
    v_new_history_id UUID;
BEGIN
    -- ========================================================================
    -- VALIDAÇÕES
    -- ========================================================================
    
    -- 1. Verificar se reason foi fornecido
    IF p_reason IS NULL OR TRIM(p_reason) = '' THEN
        RAISE EXCEPTION 'Justificativa é obrigatória para rollback';
    END IF;
    
    -- 2. Buscar estado alvo do histórico
    SELECT data_after, status_after, performed_at, is_rollback_point
    INTO v_target_state, v_target_status, v_history_performed_at
    FROM requisition_history
    WHERE id = p_to_history_id 
      AND requisition_id = p_requisition_id
      AND is_rollback_point = TRUE;
    
    IF v_target_state IS NULL THEN
        RAISE EXCEPTION 'Ponto de histórico não encontrado ou não é válido para rollback: %', p_to_history_id;
    END IF;
    
    -- 3. Capturar estado atual
    SELECT to_jsonb(r.*), r.status::TEXT
    INTO v_current_state, v_current_status
    FROM requisitions r 
    WHERE id = p_requisition_id;
    
    IF v_current_state IS NULL THEN
        RAISE EXCEPTION 'Requisição não encontrada: %', p_requisition_id;
    END IF;
    
    -- 4. Verificar se requisição está em estado que permite rollback
    IF v_current_status IN ('FULFILLED', 'PARTIALLY_FULFILLED') THEN
        RAISE EXCEPTION 'Não é possível fazer rollback de requisição já atendida (status: %)', v_current_status;
    END IF;
    
    -- 5. Verificar se há movimentações de estoque posteriores ao ponto de rollback
    SELECT EXISTS (
        SELECT 1 FROM stock_movements 
        WHERE requisition_id = p_requisition_id
          AND movement_date > v_history_performed_at
    ) INTO v_has_stock_movements;
    
    IF v_has_stock_movements THEN
        RAISE EXCEPTION 'Existem movimentações de estoque posteriores ao ponto de rollback. Rollback não permitido.';
    END IF;
    
    -- ========================================================================
    -- EXECUÇÃO DO ROLLBACK
    -- ========================================================================
    
    -- Definir contexto de auditoria
    PERFORM fn_set_audit_context(p_user_id);
    PERFORM set_config('audit.delete_reason', p_reason, TRUE);
    
    -- Desabilitar trigger temporariamente para controle manual
    ALTER TABLE requisitions DISABLE TRIGGER trg_requisition_audit;
    
    -- Aplicar rollback dos campos principais
    UPDATE requisitions SET
        status = (v_target_state->>'status')::requisition_status_enum,
        priority = (v_target_state->>'priority')::requisition_priority_enum,
        total_value = (v_target_state->>'total_value')::DECIMAL,
        needed_by = (v_target_state->>'needed_by')::DATE,
        approved_by = (v_target_state->>'approved_by')::UUID,
        approved_at = (v_target_state->>'approved_at')::TIMESTAMPTZ,
        fulfilled_by = (v_target_state->>'fulfilled_by')::UUID,
        fulfilled_at = (v_target_state->>'fulfilled_at')::TIMESTAMPTZ,
        rejection_reason = v_target_state->>'rejection_reason',
        cancellation_reason = v_target_state->>'cancellation_reason',
        notes = v_target_state->>'notes',
        internal_notes = v_target_state->>'internal_notes',
        updated_at = NOW()
    WHERE id = p_requisition_id;
    
    -- Reabilitar trigger
    ALTER TABLE requisitions ENABLE TRIGGER trg_requisition_audit;
    
    -- Registrar rollback manualmente no histórico
    INSERT INTO requisition_history (
        requisition_id,
        requisition_number,
        operation,
        data_before,
        data_after,
        changed_fields,
        changes_diff,
        status_before,
        status_after,
        performed_by,
        ip_address,
        reason,
        is_rollback,
        rollback_to_history_id,
        is_rollback_point
    ) 
    SELECT
        p_requisition_id,
        v_current_state->>'requisition_number',
        'ROLLBACK'::audit_operation_enum,
        v_current_state,
        v_target_state,
        fn_get_changed_fields(v_current_state, v_target_state),
        fn_generate_diff(v_current_state, v_target_state),
        v_current_status,
        v_target_status,
        p_user_id,
        fn_get_audit_ip(),
        p_reason,
        TRUE,
        p_to_history_id,
        TRUE -- Rollback também pode ser ponto de rollback futuro
    RETURNING id INTO v_new_history_id;
    
    -- ========================================================================
    -- RESULTADO
    -- ========================================================================
    
    v_result := jsonb_build_object(
        'success', TRUE,
        'requisition_id', p_requisition_id,
        'rollback_history_id', v_new_history_id,
        'rolled_back_to', p_to_history_id,
        'previous_status', v_current_status,
        'restored_status', v_target_status,
        'changed_fields', fn_get_changed_fields(v_current_state, v_target_state),
        'performed_at', NOW(),
        'performed_by', p_user_id
    );
    
    RETURN v_result;
    
EXCEPTION WHEN OTHERS THEN
    -- Garantir que trigger seja reabilitado em caso de erro
    ALTER TABLE requisitions ENABLE TRIGGER trg_requisition_audit;
    RAISE;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 3. FUNÇÃO PARA ROLLBACK DE ITENS
-- ============================================================================

CREATE OR REPLACE FUNCTION fn_rollback_requisition_items(
    p_requisition_id UUID,
    p_to_history_id UUID,
    p_user_id UUID
) RETURNS JSONB AS $$
DECLARE
    v_history_performed_at TIMESTAMPTZ;
    v_items_before JSONB;
    v_items_after JSONB;
    v_restored_count INTEGER := 0;
BEGIN
    -- Buscar timestamp do ponto de rollback
    SELECT performed_at INTO v_history_performed_at
    FROM requisition_history
    WHERE id = p_to_history_id;
    
    IF v_history_performed_at IS NULL THEN
        RAISE EXCEPTION 'Ponto de histórico não encontrado: %', p_to_history_id;
    END IF;
    
    -- Capturar estado atual dos itens
    SELECT jsonb_agg(to_jsonb(ri.*)) INTO v_items_after
    FROM requisition_items ri
    WHERE ri.requisition_id = p_requisition_id;
    
    -- Para cada item que foi alterado após o ponto de rollback,
    -- restaurar para o estado mais próximo antes do ponto
    
    -- Restaurar itens soft-deleted após o ponto
    UPDATE requisition_items ri SET
        deleted_at = NULL,
        deleted_by = NULL,
        deletion_reason = NULL
    FROM requisition_item_history rih
    WHERE ri.id = rih.requisition_item_id
      AND ri.requisition_id = p_requisition_id
      AND rih.operation = 'SOFT_DELETE'
      AND rih.performed_at > v_history_performed_at
      AND ri.deleted_at IS NOT NULL;
    
    GET DIAGNOSTICS v_restored_count = ROW_COUNT;
    
    -- TODO: Implementar restauração completa de valores alterados
    -- Isso requer lógica mais complexa para itens individuais
    
    RETURN jsonb_build_object(
        'success', TRUE,
        'restored_items_count', v_restored_count,
        'items_before', v_items_after,
        'note', 'Itens soft-deleted foram restaurados. Alterações em valores individuais requerem rollback manual.'
    );
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 4. FUNÇÃO PARA CANCELAR REQUISIÇÃO COM AUDITORIA COMPLETA
-- ============================================================================

CREATE OR REPLACE FUNCTION fn_cancel_requisition(
    p_requisition_id UUID,
    p_reason TEXT,
    p_user_id UUID
) RETURNS JSONB AS $$
DECLARE
    v_current_status TEXT;
    v_has_stock_movements BOOLEAN;
BEGIN
    -- Validar reason
    IF p_reason IS NULL OR TRIM(p_reason) = '' THEN
        RAISE EXCEPTION 'Justificativa é obrigatória para cancelamento';
    END IF;
    
    -- Verificar status atual
    SELECT status::TEXT INTO v_current_status
    FROM requisitions WHERE id = p_requisition_id;
    
    IF v_current_status IS NULL THEN
        RAISE EXCEPTION 'Requisição não encontrada: %', p_requisition_id;
    END IF;
    
    IF v_current_status IN ('CANCELLED', 'FULFILLED') THEN
        RAISE EXCEPTION 'Requisição não pode ser cancelada (status atual: %)', v_current_status;
    END IF;
    
    -- Verificar movimentações de estoque
    SELECT EXISTS (
        SELECT 1 FROM stock_movements WHERE requisition_id = p_requisition_id
    ) INTO v_has_stock_movements;
    
    IF v_has_stock_movements THEN
        RAISE EXCEPTION 'Requisição já gerou movimentações de estoque. Use estorno ao invés de cancelamento.';
    END IF;
    
    -- Definir contexto
    PERFORM fn_set_audit_context(p_user_id);
    
    -- Cancelar
    UPDATE requisitions SET
        status = 'CANCELLED',
        cancellation_reason = p_reason,
        updated_at = NOW()
    WHERE id = p_requisition_id;
    
    -- Liberar reservas (se existirem)
    UPDATE warehouse_stocks ws SET
        reserved_quantity = reserved_quantity - COALESCE((
            SELECT SUM(ri.approved_quantity)
            FROM requisition_items ri
            WHERE ri.requisition_id = p_requisition_id
              AND ri.catalog_item_id = ws.catalog_item_id
              AND ri.deleted_at IS NULL
        ), 0),
        updated_at = NOW()
    WHERE ws.warehouse_id = (SELECT warehouse_id FROM requisitions WHERE id = p_requisition_id)
      AND ws.catalog_item_id IN (
          SELECT catalog_item_id FROM requisition_items 
          WHERE requisition_id = p_requisition_id AND deleted_at IS NULL
      );
    
    RETURN jsonb_build_object(
        'success', TRUE,
        'requisition_id', p_requisition_id,
        'previous_status', v_current_status,
        'new_status', 'CANCELLED',
        'reason', p_reason
    );
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 5. VIEW PARA AUDITORIA CONSOLIDADA
-- ============================================================================

CREATE OR REPLACE VIEW vw_requisition_audit_trail AS
SELECT 
    rh.id AS history_id,
    rh.requisition_id,
    rh.requisition_number,
    rh.operation::TEXT AS operation,
    rh.status_before,
    rh.status_after,
    rh.changed_fields,
    rh.performed_at,
    rh.performed_by,
    rh.performed_by_name,
    rh.reason,
    rh.is_rollback,
    rh.rollback_to_history_id,
    -- Contagem de itens alterados na mesma transação
    (
        SELECT COUNT(*) 
        FROM requisition_item_history rih 
        WHERE rih.transaction_id = rh.transaction_id
    ) AS items_changed_count,
    -- Resumo das alterações
    CASE 
        WHEN rh.operation = 'INSERT' THEN 'Requisição criada'
        WHEN rh.operation = 'APPROVAL' THEN 'Requisição aprovada'
        WHEN rh.operation = 'REJECTION' THEN 'Requisição rejeitada: ' || COALESCE(rh.reason, '-')
        WHEN rh.operation = 'CANCELLATION' THEN 'Requisição cancelada: ' || COALESCE(rh.reason, '-')
        WHEN rh.operation = 'ROLLBACK' THEN 'Rollback realizado: ' || COALESCE(rh.reason, '-')
        WHEN rh.operation = 'STATUS_CHANGE' THEN 'Status alterado de ' || rh.status_before || ' para ' || rh.status_after
        WHEN rh.operation = 'UPDATE' THEN 'Atualização em ' || array_length(rh.changed_fields, 1)::TEXT || ' campo(s)'
        ELSE rh.operation::TEXT
    END AS summary
FROM requisition_history rh
ORDER BY rh.requisition_id, rh.performed_at DESC;

-- ============================================================================
-- 6. COMENTÁRIOS
-- ============================================================================

COMMENT ON FUNCTION fn_list_rollback_points IS 
    'Lista pontos de histórico disponíveis para rollback com validação de viabilidade';

COMMENT ON FUNCTION fn_rollback_requisition IS 
    'Reverte uma requisição para um estado anterior do histórico com validações de negócio';

COMMENT ON FUNCTION fn_cancel_requisition IS 
    'Cancela uma requisição com validações e liberação de reservas';

COMMENT ON VIEW vw_requisition_audit_trail IS 
    'Visão consolidada do histórico de auditoria de requisições';

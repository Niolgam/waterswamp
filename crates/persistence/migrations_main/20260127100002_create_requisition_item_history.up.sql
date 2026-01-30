-- Migration: Histórico de Itens de Requisição + Soft Delete
-- ============================================================================
-- Auditoria de itens e suporte a soft delete para permitir recuperação
-- ============================================================================

-- ============================================================================
-- 1. ADICIONAR SOFT DELETE À TABELA DE ITENS
-- ============================================================================

ALTER TABLE requisition_items 
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_by UUID,
    ADD COLUMN IF NOT EXISTS deletion_reason TEXT;

-- Índice para filtrar itens ativos
CREATE INDEX IF NOT EXISTS idx_requisition_items_active 
    ON requisition_items(requisition_id) 
    WHERE deleted_at IS NULL;

-- Índice para itens deletados (auditoria)
CREATE INDEX IF NOT EXISTS idx_requisition_items_deleted 
    ON requisition_items(deleted_at DESC) 
    WHERE deleted_at IS NOT NULL;

-- ============================================================================
-- 2. TABELA DE HISTÓRICO DE ITENS
-- ============================================================================

CREATE TABLE requisition_item_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Referências (sem FK para permitir histórico de deletados)
    requisition_item_id UUID NOT NULL,
    requisition_id UUID NOT NULL,
    catalog_item_id UUID,
    catalog_item_name VARCHAR(200), -- Cache para referência
    
    -- Operação
    operation audit_operation_enum NOT NULL,
    
    -- Snapshots
    data_before JSONB,
    data_after JSONB,
    changed_fields TEXT[],
    changes_diff JSONB,
    
    -- Campos críticos extraídos (para consultas rápidas)
    quantity_before DECIMAL(15, 3),
    quantity_after DECIMAL(15, 3),
    
    -- Contexto
    performed_by UUID NOT NULL,
    performed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address INET,
    reason TEXT,
    
    -- Agrupamento com operação na requisição pai
    transaction_id UUID,
    parent_history_id UUID REFERENCES requisition_history(id),
    
    -- Controle de rollback
    is_rollback BOOLEAN NOT NULL DEFAULT FALSE,
    rollback_to_history_id UUID REFERENCES requisition_item_history(id)
);

-- Índices
CREATE INDEX idx_req_item_history_item 
    ON requisition_item_history(requisition_item_id, performed_at DESC);

CREATE INDEX idx_req_item_history_requisition 
    ON requisition_item_history(requisition_id, performed_at DESC);

CREATE INDEX idx_req_item_history_catalog 
    ON requisition_item_history(catalog_item_id) 
    WHERE catalog_item_id IS NOT NULL;

CREATE INDEX idx_req_item_history_operation 
    ON requisition_item_history(operation);

CREATE INDEX idx_req_item_history_date 
    ON requisition_item_history(performed_at DESC);

CREATE INDEX idx_req_item_history_transaction 
    ON requisition_item_history(transaction_id) 
    WHERE transaction_id IS NOT NULL;

CREATE INDEX idx_req_item_history_parent 
    ON requisition_item_history(parent_history_id) 
    WHERE parent_history_id IS NOT NULL;

-- ============================================================================
-- 3. TRIGGER DE AUDITORIA PARA ITENS
-- ============================================================================

CREATE OR REPLACE FUNCTION fn_requisition_item_audit()
RETURNS TRIGGER AS $$
DECLARE
    v_user_id UUID;
    v_operation audit_operation_enum;
    v_data_before JSONB;
    v_data_after JSONB;
    v_catalog_name VARCHAR(200);
    v_quantity_before DECIMAL(15, 3);
    v_quantity_after DECIMAL(15, 3);
    v_reason TEXT;
BEGIN
    v_user_id := fn_get_audit_user_id();
    
    -- Buscar nome do item do catálogo
    SELECT name INTO v_catalog_name 
    FROM catalog_items 
    WHERE id = COALESCE(NEW.catalog_item_id, OLD.catalog_item_id);
    
    IF TG_OP = 'INSERT' THEN
        v_operation := 'INSERT';
        v_data_after := to_jsonb(NEW);
        v_quantity_after := NEW.requested_quantity;
        
    ELSIF TG_OP = 'UPDATE' THEN
        v_data_before := to_jsonb(OLD);
        v_data_after := to_jsonb(NEW);
        v_quantity_before := OLD.requested_quantity;
        v_quantity_after := NEW.requested_quantity;
        
        -- Detectar soft delete
        IF OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL THEN
            v_operation := 'SOFT_DELETE';
            v_reason := NEW.deletion_reason;
        -- Detectar restore
        ELSIF OLD.deleted_at IS NOT NULL AND NEW.deleted_at IS NULL THEN
            v_operation := 'RESTORE';
        ELSE
            v_operation := 'UPDATE';
        END IF;
        
    ELSIF TG_OP = 'DELETE' THEN
        v_operation := 'DELETE';
        v_data_before := to_jsonb(OLD);
        v_quantity_before := OLD.requested_quantity;
        v_reason := current_setting('audit.delete_reason', TRUE);
    END IF;
    
    INSERT INTO requisition_item_history (
        requisition_item_id,
        requisition_id,
        catalog_item_id,
        catalog_item_name,
        operation,
        data_before,
        data_after,
        changed_fields,
        changes_diff,
        quantity_before,
        quantity_after,
        performed_by,
        ip_address,
        reason,
        transaction_id
    ) VALUES (
        COALESCE(NEW.id, OLD.id),
        COALESCE(NEW.requisition_id, OLD.requisition_id),
        COALESCE(NEW.catalog_item_id, OLD.catalog_item_id),
        v_catalog_name,
        v_operation,
        v_data_before,
        v_data_after,
        CASE WHEN v_data_before IS NOT NULL AND v_data_after IS NOT NULL 
             THEN fn_get_changed_fields(v_data_before, v_data_after) 
             ELSE NULL END,
        CASE WHEN v_data_before IS NOT NULL AND v_data_after IS NOT NULL 
             THEN fn_generate_diff(v_data_before, v_data_after) 
             ELSE NULL END,
        v_quantity_before,
        v_quantity_after,
        COALESCE(v_user_id, '00000000-0000-0000-0000-000000000000'::UUID),
        fn_get_audit_ip(),
        v_reason,
        current_setting('audit.transaction_id', TRUE)::UUID
    );
    
    RETURN COALESCE(NEW, OLD);
EXCEPTION 
    WHEN invalid_text_representation THEN
        -- transaction_id não definido, inserir sem ele
        INSERT INTO requisition_item_history (
            requisition_item_id,
            requisition_id,
            catalog_item_id,
            catalog_item_name,
            operation,
            data_before,
            data_after,
            changed_fields,
            changes_diff,
            quantity_before,
            quantity_after,
            performed_by,
            ip_address,
            reason
        ) VALUES (
            COALESCE(NEW.id, OLD.id),
            COALESCE(NEW.requisition_id, OLD.requisition_id),
            COALESCE(NEW.catalog_item_id, OLD.catalog_item_id),
            v_catalog_name,
            v_operation,
            v_data_before,
            v_data_after,
            CASE WHEN v_data_before IS NOT NULL AND v_data_after IS NOT NULL 
                 THEN fn_get_changed_fields(v_data_before, v_data_after) 
                 ELSE NULL END,
            CASE WHEN v_data_before IS NOT NULL AND v_data_after IS NOT NULL 
                 THEN fn_generate_diff(v_data_before, v_data_after) 
                 ELSE NULL END,
            v_quantity_before,
            v_quantity_after,
            COALESCE(v_user_id, '00000000-0000-0000-0000-000000000000'::UUID),
            fn_get_audit_ip(),
            v_reason
        );
        RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_requisition_item_audit
AFTER INSERT OR UPDATE OR DELETE ON requisition_items
FOR EACH ROW EXECUTE FUNCTION fn_requisition_item_audit();

-- ============================================================================
-- 4. FUNÇÃO DE SOFT DELETE PARA ITENS
-- ============================================================================

CREATE OR REPLACE FUNCTION fn_soft_delete_requisition_item(
    p_item_id UUID,
    p_user_id UUID,
    p_reason TEXT
) RETURNS BOOLEAN AS $$
DECLARE
    v_requisition_id UUID;
    v_requisition_status VARCHAR(30);
BEGIN
    -- Verificar se item existe e não está deletado
    SELECT ri.requisition_id, r.status::TEXT
    INTO v_requisition_id, v_requisition_status
    FROM requisition_items ri
    JOIN requisitions r ON r.id = ri.requisition_id
    WHERE ri.id = p_item_id AND ri.deleted_at IS NULL;
    
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Item não encontrado ou já deletado: %', p_item_id;
    END IF;
    
    -- Verificar se requisição permite alteração
    IF v_requisition_status NOT IN ('DRAFT', 'PENDING') THEN
        RAISE EXCEPTION 'Não é possível remover itens de requisição com status %', v_requisition_status;
    END IF;
    
    -- Definir contexto
    PERFORM fn_set_audit_context(p_user_id);
    
    -- Executar soft delete
    UPDATE requisition_items SET
        deleted_at = NOW(),
        deleted_by = p_user_id,
        deletion_reason = p_reason
    WHERE id = p_item_id;
    
    -- Recalcular total da requisição
    UPDATE requisitions SET
        total_value = (
            SELECT COALESCE(SUM(total_value), 0)
            FROM requisition_items
            WHERE requisition_id = v_requisition_id
              AND deleted_at IS NULL
        ),
        updated_at = NOW()
    WHERE id = v_requisition_id;
    
    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 5. FUNÇÃO DE RESTORE PARA ITENS
-- ============================================================================

CREATE OR REPLACE FUNCTION fn_restore_requisition_item(
    p_item_id UUID,
    p_user_id UUID
) RETURNS BOOLEAN AS $$
DECLARE
    v_requisition_id UUID;
    v_requisition_status VARCHAR(30);
BEGIN
    -- Verificar se item existe e está deletado
    SELECT ri.requisition_id, r.status::TEXT
    INTO v_requisition_id, v_requisition_status
    FROM requisition_items ri
    JOIN requisitions r ON r.id = ri.requisition_id
    WHERE ri.id = p_item_id AND ri.deleted_at IS NOT NULL;
    
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Item não encontrado ou não está deletado: %', p_item_id;
    END IF;
    
    -- Verificar se requisição permite alteração
    IF v_requisition_status NOT IN ('DRAFT', 'PENDING') THEN
        RAISE EXCEPTION 'Não é possível restaurar itens em requisição com status %', v_requisition_status;
    END IF;
    
    -- Definir contexto
    PERFORM fn_set_audit_context(p_user_id);
    
    -- Restaurar item
    UPDATE requisition_items SET
        deleted_at = NULL,
        deleted_by = NULL,
        deletion_reason = NULL
    WHERE id = p_item_id;
    
    -- Recalcular total da requisição
    UPDATE requisitions SET
        total_value = (
            SELECT COALESCE(SUM(total_value), 0)
            FROM requisition_items
            WHERE requisition_id = v_requisition_id
              AND deleted_at IS NULL
        ),
        updated_at = NOW()
    WHERE id = v_requisition_id;
    
    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 6. COMENTÁRIOS
-- ============================================================================

COMMENT ON TABLE requisition_item_history IS 
    'Histórico de alterações em itens de requisição';

COMMENT ON COLUMN requisition_items.deleted_at IS 
    'Timestamp de soft delete - NULL significa item ativo';

COMMENT ON COLUMN requisition_items.deletion_reason IS 
    'Justificativa obrigatória para exclusão do item';

COMMENT ON FUNCTION fn_soft_delete_requisition_item IS 
    'Executa soft delete de um item com validações de negócio';

COMMENT ON FUNCTION fn_restore_requisition_item IS 
    'Restaura um item soft-deleted com validações de negócio';

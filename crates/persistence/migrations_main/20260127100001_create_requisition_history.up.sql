-- Migration: Histórico de Requisições (Auditoria Detalhada)
-- ============================================================================
-- Tabela dedicada para auditoria de requisições com suporte a rollback
-- ============================================================================

-- ============================================================================
-- 1. TABELA DE HISTÓRICO DE REQUISIÇÕES
-- ============================================================================

CREATE TABLE requisition_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- ========================================================================
    -- REFERÊNCIA À REQUISIÇÃO
    -- ========================================================================
    -- Sem FK para permitir histórico mesmo após exclusão
    requisition_id UUID NOT NULL,
    requisition_number VARCHAR(20), -- Cache para referência rápida
    
    -- ========================================================================
    -- TIPO DE OPERAÇÃO
    -- ========================================================================
    operation audit_operation_enum NOT NULL,
    
    -- ========================================================================
    -- SNAPSHOTS DE ESTADO
    -- ========================================================================
    -- Estado COMPLETO antes da operação (NULL para INSERT)
    data_before JSONB,
    
    -- Estado COMPLETO após a operação (NULL para DELETE)
    data_after JSONB,
    
    -- Lista de campos que mudaram (para facilitar consultas)
    changed_fields TEXT[],
    
    -- Diff detalhado: { "campo": { "old": X, "new": Y } }
    changes_diff JSONB,
    
    -- ========================================================================
    -- MUDANÇAS DE STATUS (campos específicos para consultas rápidas)
    -- ========================================================================
    status_before VARCHAR(30),
    status_after VARCHAR(30),
    
    -- ========================================================================
    -- CONTEXTO DA OPERAÇÃO
    -- ========================================================================
    performed_by UUID NOT NULL,
    performed_by_name VARCHAR(200), -- Cache do nome do usuário
    performed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address INET,
    user_agent TEXT,
    
    -- ========================================================================
    -- JUSTIFICATIVA
    -- ========================================================================
    -- Obrigatória para: ROLLBACK, CANCELLATION, REJECTION, DELETE
    reason TEXT,
    
    -- ========================================================================
    -- CONTROLE DE ROLLBACK
    -- ========================================================================
    is_rollback BOOLEAN NOT NULL DEFAULT FALSE,
    -- Aponta para qual registro do histórico foi restaurado
    rollback_to_history_id UUID REFERENCES requisition_history(id),
    -- Indica se este estado pode ser usado como alvo de rollback
    is_rollback_point BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- ========================================================================
    -- METADADOS
    -- ========================================================================
    -- Agrupa operações da mesma transação (ex: UPDATE + UPDATE_ITEMS)
    transaction_id UUID DEFAULT gen_random_uuid(),
    
    -- Dados extras específicos do contexto
    metadata JSONB,
    
    -- ========================================================================
    -- CONSTRAINTS
    -- ========================================================================
    CONSTRAINT ck_requisition_history_reason CHECK (
        -- Reason obrigatório para operações destrutivas
        (operation IN ('DELETE', 'SOFT_DELETE', 'ROLLBACK', 'CANCELLATION', 'REJECTION') 
         AND reason IS NOT NULL AND reason <> '')
        OR
        (operation NOT IN ('DELETE', 'SOFT_DELETE', 'ROLLBACK', 'CANCELLATION', 'REJECTION'))
    ),
    
    CONSTRAINT ck_requisition_history_rollback CHECK (
        -- Se é rollback, deve ter referência ao estado alvo
        (is_rollback = TRUE AND rollback_to_history_id IS NOT NULL)
        OR
        (is_rollback = FALSE)
    ),
    
    CONSTRAINT ck_requisition_history_data CHECK (
        -- INSERT deve ter data_after, DELETE deve ter data_before
        (operation = 'INSERT' AND data_after IS NOT NULL)
        OR
        (operation = 'DELETE' AND data_before IS NOT NULL)
        OR
        (operation NOT IN ('INSERT', 'DELETE'))
    )
);

-- ============================================================================
-- 2. ÍNDICES
-- ============================================================================

-- Consulta principal: histórico de uma requisição
CREATE INDEX idx_req_history_requisition 
    ON requisition_history(requisition_id, performed_at DESC);

-- Busca por número da requisição
CREATE INDEX idx_req_history_number 
    ON requisition_history(requisition_number) 
    WHERE requisition_number IS NOT NULL;

-- Filtragem por tipo de operação
CREATE INDEX idx_req_history_operation 
    ON requisition_history(operation);

-- Filtragem por mudanças de status
CREATE INDEX idx_req_history_status_change 
    ON requisition_history(status_before, status_after) 
    WHERE status_before IS DISTINCT FROM status_after;

-- Auditoria por usuário
CREATE INDEX idx_req_history_user 
    ON requisition_history(performed_by, performed_at DESC);

-- Filtragem por período
CREATE INDEX idx_req_history_date 
    ON requisition_history(performed_at DESC);

-- Rollbacks realizados
CREATE INDEX idx_req_history_rollbacks 
    ON requisition_history(requisition_id, performed_at DESC) 
    WHERE is_rollback = TRUE;

-- Pontos de rollback disponíveis
CREATE INDEX idx_req_history_rollback_points 
    ON requisition_history(requisition_id, performed_at DESC) 
    WHERE is_rollback_point = TRUE;

-- Busca em campos alterados
CREATE INDEX idx_req_history_changed_fields 
    ON requisition_history USING GIN(changed_fields);

-- Busca em metadata
CREATE INDEX idx_req_history_metadata 
    ON requisition_history USING GIN(metadata) 
    WHERE metadata IS NOT NULL;

-- Agrupamento por transação
CREATE INDEX idx_req_history_transaction 
    ON requisition_history(transaction_id);

-- ============================================================================
-- 3. TRIGGER AUTOMÁTICO DE AUDITORIA
-- ============================================================================

CREATE OR REPLACE FUNCTION fn_requisition_audit()
RETURNS TRIGGER AS $$
DECLARE
    v_user_id UUID;
    v_operation audit_operation_enum;
    v_data_before JSONB;
    v_data_after JSONB;
    v_changed_fields TEXT[];
    v_changes_diff JSONB;
    v_status_before VARCHAR(30);
    v_status_after VARCHAR(30);
    v_reason TEXT;
BEGIN
    -- Obter usuário do contexto da sessão
    v_user_id := fn_get_audit_user_id();
    
    -- Determinar operação
    IF TG_OP = 'INSERT' THEN
        v_operation := 'INSERT';
        v_data_after := to_jsonb(NEW);
        v_status_after := NEW.status::TEXT;
        v_user_id := COALESCE(v_user_id, NEW.requester_id);
        
    ELSIF TG_OP = 'UPDATE' THEN
        v_data_before := to_jsonb(OLD);
        v_data_after := to_jsonb(NEW);
        v_status_before := OLD.status::TEXT;
        v_status_after := NEW.status::TEXT;
        v_changed_fields := fn_get_changed_fields(v_data_before, v_data_after);
        v_changes_diff := fn_generate_diff(v_data_before, v_data_after);
        
        -- Determinar tipo específico de operação
        IF OLD.status::TEXT <> NEW.status::TEXT THEN
            CASE NEW.status::TEXT
                WHEN 'APPROVED' THEN v_operation := 'APPROVAL';
                WHEN 'REJECTED' THEN 
                    v_operation := 'REJECTION';
                    v_reason := NEW.rejection_reason;
                WHEN 'CANCELLED' THEN 
                    v_operation := 'CANCELLATION';
                    v_reason := NEW.cancellation_reason;
                ELSE v_operation := 'STATUS_CHANGE';
            END CASE;
        ELSE
            v_operation := 'UPDATE';
        END IF;
        
        v_user_id := COALESCE(v_user_id, NEW.approved_by, NEW.fulfilled_by, NEW.requester_id);
        
    ELSIF TG_OP = 'DELETE' THEN
        v_operation := 'DELETE';
        v_data_before := to_jsonb(OLD);
        v_status_before := OLD.status::TEXT;
        v_reason := current_setting('audit.delete_reason', TRUE);
        
        -- DELETE sem reason definido no contexto
        IF v_reason IS NULL OR v_reason = '' THEN
            v_reason := 'Exclusão direta (sem justificativa registrada)';
        END IF;
    END IF;
    
    -- Inserir registro de histórico
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
        is_rollback_point
    ) VALUES (
        COALESCE(NEW.id, OLD.id),
        COALESCE(NEW.requisition_number, OLD.requisition_number),
        v_operation,
        v_data_before,
        v_data_after,
        v_changed_fields,
        v_changes_diff,
        v_status_before,
        v_status_after,
        COALESCE(v_user_id, '00000000-0000-0000-0000-000000000000'::UUID),
        fn_get_audit_ip(),
        v_reason,
        -- Não marca como ponto de rollback se for DELETE ou já é um rollback
        v_operation NOT IN ('DELETE', 'SOFT_DELETE')
    );
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Criar trigger
CREATE TRIGGER trg_requisition_audit
AFTER INSERT OR UPDATE OR DELETE ON requisitions
FOR EACH ROW EXECUTE FUNCTION fn_requisition_audit();

-- ============================================================================
-- 4. COMENTÁRIOS
-- ============================================================================

COMMENT ON TABLE requisition_history IS 
    'Histórico completo de alterações em requisições para auditoria e rollback';

COMMENT ON COLUMN requisition_history.data_before IS 
    'Snapshot JSONB completo do registro antes da alteração';

COMMENT ON COLUMN requisition_history.data_after IS 
    'Snapshot JSONB completo do registro após a alteração';

COMMENT ON COLUMN requisition_history.changed_fields IS 
    'Array com nomes dos campos alterados para consultas rápidas';

COMMENT ON COLUMN requisition_history.changes_diff IS 
    'Diff detalhado no formato { "campo": { "old": valor, "new": valor } }';

COMMENT ON COLUMN requisition_history.is_rollback_point IS 
    'Indica se este estado pode ser usado como destino de rollback';

COMMENT ON FUNCTION fn_requisition_audit IS 
    'Trigger function que registra automaticamente todas as alterações em requisições';

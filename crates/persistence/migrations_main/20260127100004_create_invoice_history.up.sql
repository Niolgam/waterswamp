-- Migration: Histórico de Notas Fiscais (Invoices)
-- ============================================================================
-- Auditoria detalhada para notas fiscais com suporte a estornos
-- ============================================================================

-- ============================================================================
-- 1. TABELA DE HISTÓRICO DE NOTAS FISCAIS
-- ============================================================================

CREATE TABLE invoice_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Referência
    invoice_id UUID NOT NULL,
    invoice_number VARCHAR(50), -- Cache para referência
    
    -- Operação
    operation audit_operation_enum NOT NULL,
    
    -- Snapshots
    data_before JSONB,
    data_after JSONB,
    changed_fields TEXT[],
    changes_diff JSONB,
    
    -- Campos críticos extraídos
    status_before VARCHAR(30),
    status_after VARCHAR(30),
    total_value_before DECIMAL(15, 2),
    total_value_after DECIMAL(15, 2),
    
    -- Contexto
    performed_by UUID NOT NULL,
    performed_by_name VARCHAR(200),
    performed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address INET,
    reason TEXT,
    
    -- Controle
    transaction_id UUID DEFAULT gen_random_uuid(),
    is_rollback BOOLEAN NOT NULL DEFAULT FALSE,
    rollback_to_history_id UUID REFERENCES invoice_history(id),
    is_rollback_point BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Metadados
    metadata JSONB,
    
    -- Constraints
    CONSTRAINT ck_invoice_history_reason CHECK (
        (operation IN ('DELETE', 'ROLLBACK', 'CANCELLATION') 
         AND reason IS NOT NULL AND reason <> '')
        OR
        (operation NOT IN ('DELETE', 'ROLLBACK', 'CANCELLATION'))
    )
);

-- Índices
CREATE INDEX idx_inv_history_invoice 
    ON invoice_history(invoice_id, performed_at DESC);
CREATE INDEX idx_inv_history_number 
    ON invoice_history(invoice_number);
CREATE INDEX idx_inv_history_operation 
    ON invoice_history(operation);
CREATE INDEX idx_inv_history_status 
    ON invoice_history(status_before, status_after) 
    WHERE status_before IS DISTINCT FROM status_after;
CREATE INDEX idx_inv_history_user 
    ON invoice_history(performed_by);
CREATE INDEX idx_inv_history_date 
    ON invoice_history(performed_at DESC);
CREATE INDEX idx_inv_history_rollbacks 
    ON invoice_history(invoice_id) WHERE is_rollback = TRUE;

-- ============================================================================
-- 2. TRIGGER DE AUDITORIA PARA INVOICES
-- ============================================================================

CREATE OR REPLACE FUNCTION fn_invoice_audit()
RETURNS TRIGGER AS $$
DECLARE
    v_user_id UUID;
    v_operation audit_operation_enum;
    v_data_before JSONB;
    v_data_after JSONB;
    v_status_before VARCHAR(30);
    v_status_after VARCHAR(30);
    v_reason TEXT;
BEGIN
    v_user_id := fn_get_audit_user_id();
    
    IF TG_OP = 'INSERT' THEN
        v_operation := 'INSERT';
        v_data_after := to_jsonb(NEW);
        v_status_after := NEW.status::TEXT;
        v_user_id := COALESCE(v_user_id, NEW.received_by);
        
    ELSIF TG_OP = 'UPDATE' THEN
        v_data_before := to_jsonb(OLD);
        v_data_after := to_jsonb(NEW);
        v_status_before := OLD.status::TEXT;
        v_status_after := NEW.status::TEXT;
        
        -- Determinar tipo de operação
        IF OLD.status::TEXT <> NEW.status::TEXT THEN
            CASE NEW.status::TEXT
                WHEN 'CANCELLED' THEN 
                    v_operation := 'CANCELLATION';
                    v_reason := NEW.rejection_reason;
                WHEN 'POSTED' THEN 
                    v_operation := 'APPROVAL'; -- Posted é como "aprovação" da NF
                ELSE 
                    v_operation := 'STATUS_CHANGE';
            END CASE;
        ELSE
            v_operation := 'UPDATE';
        END IF;
        
        v_user_id := COALESCE(v_user_id, NEW.posted_by, NEW.checked_by, NEW.received_by);
        
    ELSIF TG_OP = 'DELETE' THEN
        v_operation := 'DELETE';
        v_data_before := to_jsonb(OLD);
        v_status_before := OLD.status::TEXT;
        v_reason := current_setting('audit.delete_reason', TRUE);
    END IF;
    
    INSERT INTO invoice_history (
        invoice_id,
        invoice_number,
        operation,
        data_before,
        data_after,
        changed_fields,
        changes_diff,
        status_before,
        status_after,
        total_value_before,
        total_value_after,
        performed_by,
        ip_address,
        reason,
        is_rollback_point
    ) VALUES (
        COALESCE(NEW.id, OLD.id),
        COALESCE(NEW.invoice_number, OLD.invoice_number),
        v_operation,
        v_data_before,
        v_data_after,
        CASE WHEN v_data_before IS NOT NULL AND v_data_after IS NOT NULL 
             THEN fn_get_changed_fields(v_data_before, v_data_after) 
             ELSE NULL END,
        CASE WHEN v_data_before IS NOT NULL AND v_data_after IS NOT NULL 
             THEN fn_generate_diff(v_data_before, v_data_after) 
             ELSE NULL END,
        v_status_before,
        v_status_after,
        (v_data_before->>'total_value')::DECIMAL,
        (v_data_after->>'total_value')::DECIMAL,
        COALESCE(v_user_id, '00000000-0000-0000-0000-000000000000'::UUID),
        fn_get_audit_ip(),
        v_reason,
        v_operation NOT IN ('DELETE')
    );
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_invoice_audit
AFTER INSERT OR UPDATE OR DELETE ON invoices
FOR EACH ROW EXECUTE FUNCTION fn_invoice_audit();

-- ============================================================================
-- 3. TABELA DE HISTÓRICO DE ITENS DE NF
-- ============================================================================

CREATE TABLE invoice_item_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Referências
    invoice_item_id UUID NOT NULL,
    invoice_id UUID NOT NULL,
    catalog_item_id UUID,
    catalog_item_name VARCHAR(200),
    
    -- Operação
    operation audit_operation_enum NOT NULL,
    
    -- Snapshots
    data_before JSONB,
    data_after JSONB,
    changed_fields TEXT[],
    
    -- Campos críticos
    quantity_before DECIMAL(15, 4),
    quantity_after DECIMAL(15, 4),
    value_before DECIMAL(15, 4),
    value_after DECIMAL(15, 4),
    
    -- Contexto
    performed_by UUID NOT NULL,
    performed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address INET,
    reason TEXT,
    
    -- Agrupamento
    transaction_id UUID,
    parent_history_id UUID REFERENCES invoice_history(id)
);

-- Índices
CREATE INDEX idx_inv_item_history_item 
    ON invoice_item_history(invoice_item_id, performed_at DESC);
CREATE INDEX idx_inv_item_history_invoice 
    ON invoice_item_history(invoice_id, performed_at DESC);
CREATE INDEX idx_inv_item_history_date 
    ON invoice_item_history(performed_at DESC);

-- ============================================================================
-- 4. TRIGGER DE AUDITORIA PARA ITENS DE NF
-- ============================================================================

CREATE OR REPLACE FUNCTION fn_invoice_item_audit()
RETURNS TRIGGER AS $$
DECLARE
    v_user_id UUID;
    v_operation audit_operation_enum;
    v_catalog_name VARCHAR(200);
BEGIN
    v_user_id := fn_get_audit_user_id();
    
    SELECT name INTO v_catalog_name 
    FROM catalog_items 
    WHERE id = COALESCE(NEW.catalog_item_id, OLD.catalog_item_id);
    
    IF TG_OP = 'INSERT' THEN
        v_operation := 'INSERT';
    ELSIF TG_OP = 'UPDATE' THEN
        v_operation := 'UPDATE';
    ELSIF TG_OP = 'DELETE' THEN
        v_operation := 'DELETE';
    END IF;
    
    INSERT INTO invoice_item_history (
        invoice_item_id,
        invoice_id,
        catalog_item_id,
        catalog_item_name,
        operation,
        data_before,
        data_after,
        changed_fields,
        quantity_before,
        quantity_after,
        value_before,
        value_after,
        performed_by,
        ip_address
    ) VALUES (
        COALESCE(NEW.id, OLD.id),
        COALESCE(NEW.invoice_id, OLD.invoice_id),
        COALESCE(NEW.catalog_item_id, OLD.catalog_item_id),
        v_catalog_name,
        v_operation,
        CASE WHEN OLD IS NOT NULL THEN to_jsonb(OLD) ELSE NULL END,
        CASE WHEN NEW IS NOT NULL THEN to_jsonb(NEW) ELSE NULL END,
        CASE WHEN OLD IS NOT NULL AND NEW IS NOT NULL 
             THEN fn_get_changed_fields(to_jsonb(OLD), to_jsonb(NEW)) 
             ELSE NULL END,
        OLD.quantity_raw,
        NEW.quantity_raw,
        OLD.unit_value_raw,
        NEW.unit_value_raw,
        COALESCE(v_user_id, '00000000-0000-0000-0000-000000000000'::UUID),
        fn_get_audit_ip()
    );
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_invoice_item_audit
AFTER INSERT OR UPDATE OR DELETE ON invoice_items
FOR EACH ROW EXECUTE FUNCTION fn_invoice_item_audit();

-- ============================================================================
-- 5. VIEW DE AUDITORIA DE NF
-- ============================================================================

CREATE OR REPLACE VIEW vw_invoice_audit_trail AS
SELECT 
    ih.id AS history_id,
    ih.invoice_id,
    ih.invoice_number,
    ih.operation::TEXT AS operation,
    ih.status_before,
    ih.status_after,
    ih.total_value_before,
    ih.total_value_after,
    ih.changed_fields,
    ih.performed_at,
    ih.performed_by,
    ih.performed_by_name,
    ih.reason,
    ih.is_rollback,
    (
        SELECT COUNT(*) 
        FROM invoice_item_history iih 
        WHERE iih.transaction_id = ih.transaction_id
    ) AS items_changed_count,
    CASE 
        WHEN ih.operation = 'INSERT' THEN 'NF registrada'
        WHEN ih.operation = 'APPROVAL' THEN 'NF lançada no estoque'
        WHEN ih.operation = 'CANCELLATION' THEN 'NF cancelada: ' || COALESCE(ih.reason, '-')
        WHEN ih.operation = 'ROLLBACK' THEN 'Estorno realizado: ' || COALESCE(ih.reason, '-')
        WHEN ih.operation = 'STATUS_CHANGE' THEN 'Status alterado de ' || ih.status_before || ' para ' || ih.status_after
        WHEN ih.operation = 'UPDATE' THEN 'Atualização em ' || array_length(ih.changed_fields, 1)::TEXT || ' campo(s)'
        ELSE ih.operation::TEXT
    END AS summary
FROM invoice_history ih
ORDER BY ih.invoice_id, ih.performed_at DESC;

-- ============================================================================
-- 6. COMENTÁRIOS
-- ============================================================================

COMMENT ON TABLE invoice_history IS 
    'Histórico de alterações em notas fiscais para auditoria';

COMMENT ON TABLE invoice_item_history IS 
    'Histórico de alterações em itens de notas fiscais';

COMMENT ON VIEW vw_invoice_audit_trail IS 
    'Visão consolidada do histórico de auditoria de notas fiscais';

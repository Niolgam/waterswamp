-- FIX: Analisar como vai ser, porque não pode ficar somente com status. Quando mudar para POSTED ai sim ir para o estoque
CREATE TYPE invoice_status_enum AS ENUM (
    'PENDING',      -- Aguardando conferência
    'CHECKING',     -- Em conferência
    'CHECKED',      -- Conferida, aguardando lançamento
    'POSTED',       -- Lançada no estoque
    'REJECTED',     -- Rejeitada (divergências graves)
    'CANCELLED'     -- Cancelada
);

CREATE TABLE invoices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    invoice_number VARCHAR(50) NOT NULL,
    series VARCHAR(10),
    access_key VARCHAR(44), -- Chave de 44 dígitos da NFe
    issue_date TIMESTAMPTZ NOT NULL, 
    
    -- Relacionamentos Base
    supplier_id UUID NOT NULL REFERENCES suppliers(id) ON DELETE RESTRICT,
    warehouse_id UUID NOT NULL REFERENCES warehouses(id) ON DELETE RESTRICT,
    
    -- Valores
    total_products DECIMAL(15, 2) NOT NULL DEFAULT 0,
    total_freight DECIMAL(15, 2) NOT NULL DEFAULT 0,
    total_discount DECIMAL(15, 2) NOT NULL DEFAULT 0,
    total_value DECIMAL(15, 2) NOT NULL DEFAULT 0,
    
    -- Status e workflow
    status invoice_status_enum NOT NULL DEFAULT 'PENDING',
    
    -- Recebimento físico
    received_at TIMESTAMPTZ,
    received_by UUID REFERENCES users(id) ON DELETE RESTRICT, 
    
    -- Conferência
    checked_at TIMESTAMPTZ,
    checked_by UUID REFERENCES users(id) ON DELETE RESTRICT, 
    
    -- Lançamento no estoque
    posted_at TIMESTAMPTZ,
    posted_by UUID REFERENCES users(id) ON DELETE RESTRICT, 
    
    -- Integração com Compras (Empenho/Contrato)
    commitment_number VARCHAR(100), -- Ex: 2024NE000123
    purchase_order_number VARCHAR(100), 
    contract_number VARCHAR(100), 
    
    -- Metadados
    notes TEXT,
    rejection_reason TEXT,
    pdf_url TEXT,
    xml_url TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints de Integridade
    CONSTRAINT uq_invoices_access_key UNIQUE (access_key),
    CONSTRAINT ck_invoices_posted_fields CHECK (
        (status = 'POSTED' AND posted_at IS NOT NULL AND posted_by IS NOT NULL) OR
        (status <> 'POSTED')
    ),
    CONSTRAINT ck_invoices_rejection_reason CHECK (
        (status = 'REJECTED' AND rejection_reason IS NOT NULL) OR
        (status <> 'REJECTED')
    )
);

CREATE INDEX idx_invoices_access_key ON invoices(access_key) WHERE access_key IS NOT NULL;
CREATE INDEX idx_invoices_status ON invoices(status);
CREATE INDEX idx_invoices_received_by ON invoices(received_by) WHERE received_by IS NOT NULL;
CREATE INDEX idx_invoices_checked_by ON invoices(checked_by) WHERE checked_by IS NOT NULL;
CREATE INDEX idx_invoices_posted_by ON invoices(posted_by) WHERE posted_by IS NOT NULL;
CREATE INDEX idx_invoices_issue_date ON invoices(issue_date DESC);

CREATE TRIGGER set_timestamp_invoices
BEFORE UPDATE ON invoices
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();


-- ============================================================================
-- Trigger para lançamento automático no estoque quando NF é confirmada
-- E estorno automático quando NF é cancelada/revertida
-- ============================================================================

CREATE OR REPLACE FUNCTION fn_auto_post_invoice()
RETURNS TRIGGER AS $$
DECLARE
    v_item RECORD;
BEGIN
    -- ========================================================================
    -- CENÁRIO A: NF confirmada (status -> POSTED)
    -- Cria movimentações de ENTRADA para cada item
    -- ========================================================================
    IF NEW.status = 'POSTED' AND (OLD.status IS NULL OR OLD.status <> 'POSTED') THEN
        
        -- Validação: precisa ter usuário responsável
        IF NEW.posted_by IS NULL THEN
            RAISE EXCEPTION 'É obrigatório informar o usuário responsável pelo lançamento (posted_by)';
        END IF;
        
        -- Processa cada item da nota
        FOR v_item IN 
            SELECT 
                ii.id,
                ii.catalog_item_id,
                ii.unit_raw_id,
                ii.unit_conversion_id,
                ii.quantity_raw,
                ii.conversion_factor,
                ii.quantity_base,
                ii.unit_value_base,
                ii.total_value,
                ii.batch_number,
                ii.expiration_date
            FROM invoice_items ii
            WHERE ii.invoice_id = NEW.id
        LOOP
            INSERT INTO stock_movements (
                warehouse_id,
                catalog_item_id,
                movement_type,
                unit_raw_id,
                unit_conversion_id,
                quantity_raw,
                conversion_factor,
                quantity_base,
                unit_price_base,
                total_value,
                invoice_id,
                invoice_item_id,
                document_number,
                user_id,
                batch_number,
                expiration_date
            ) VALUES (
                NEW.warehouse_id,
                v_item.catalog_item_id,
                'ENTRY',
                v_item.unit_raw_id,
                v_item.unit_conversion_id,
                v_item.quantity_raw,
                v_item.conversion_factor,
                v_item.quantity_base,
                v_item.unit_value_base,
                v_item.total_value,
                NEW.id,
                v_item.id,
                NEW.invoice_number,
                NEW.posted_by,
                v_item.batch_number,
                v_item.expiration_date
            );
        END LOOP;
    
    -- ========================================================================
    -- CENÁRIO B: NF estornada (POSTED -> outro status)
    -- Cria movimentações de ADJUSTMENT_SUB para reverter as entradas
    -- ========================================================================
    ELSIF OLD.status = 'POSTED' AND NEW.status <> 'POSTED' THEN
        
        -- Cria movimentações de estorno baseadas nas entradas originais
        INSERT INTO stock_movements (
            warehouse_id,
            catalog_item_id,
            movement_type,
            unit_raw_id,
            unit_conversion_id,
            quantity_raw,
            conversion_factor,
            quantity_base,
            unit_price_base,
            total_value,
            invoice_id,
            invoice_item_id,
            document_number,
            user_id,
            batch_number,
            expiration_date,
            notes
        )
        SELECT 
            sm.warehouse_id,
            sm.catalog_item_id,
            'ADJUSTMENT_SUB',
            sm.unit_raw_id,
            sm.unit_conversion_id,
            sm.quantity_raw,
            sm.conversion_factor,
            sm.quantity_base,
            sm.unit_price_base,
            sm.total_value,
            sm.invoice_id,
            sm.invoice_item_id,
            'ESTORNO NF ' || NEW.invoice_number,
            COALESCE(NEW.posted_by, OLD.posted_by),
            sm.batch_number,
            sm.expiration_date,
            'Estorno automático - NF revertida de POSTED para ' || NEW.status::TEXT
        FROM stock_movements sm
        WHERE sm.invoice_id = NEW.id
          AND sm.movement_type = 'ENTRY';
        
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_auto_post_invoice
AFTER UPDATE OF status ON invoices
FOR EACH ROW
WHEN (NEW.status = 'POSTED' OR OLD.status = 'POSTED')
EXECUTE FUNCTION fn_auto_post_invoice();

CREATE TABLE invoice_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Relacionamentos
    invoice_id UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    catalog_item_id UUID NOT NULL REFERENCES catalog_items(id) ON DELETE RESTRICT,
    
    unit_conversion_id UUID REFERENCES unit_conversions(id) ON DELETE SET NULL,

    -- Unidade conforme a Nota (Ex: CX, FD, GL)
    unit_raw_id UUID NOT NULL REFERENCES units_of_measure(id) ON DELETE RESTRICT,

    -- Quantidade e Valores Brutos (conforme o papel da NF)
    quantity_raw DECIMAL(15, 4) NOT NULL CHECK (quantity_raw > 0),
    unit_value_raw DECIMAL(15, 4) NOT NULL CHECK (unit_value_raw >= 0),
    total_value DECIMAL(15, 2) NOT NULL, -- quantity_raw * unit_value_raw

    -- Fator de conversão aplicado no momento da entrada (Snapshot)
    -- Ex: 10.0000 (guardamos aqui para garantir que o cálculo nunca mude no futuro)
    conversion_factor DECIMAL(15, 4) NOT NULL DEFAULT 1.0 CHECK (conversion_factor > 0),
    
    -- Valores convertidos para unidade base
    quantity_base DECIMAL(15, 3) NOT NULL GENERATED ALWAYS AS (quantity_raw * conversion_factor) STORED,
    unit_value_base DECIMAL(15, 4) NOT NULL GENERATED ALWAYS AS (
        CASE WHEN conversion_factor > 0 THEN unit_value_raw / conversion_factor ELSE unit_value_raw END
    ) STORED,

    -- Dados Fiscais
    ncm VARCHAR(10), -- Nomenclatura Comum do Mercosul
    cfop VARCHAR(4), -- Código Fiscal de Operações
    cest VARCHAR(7), -- Código Especificador da Substituição Tributária
    
    -- Lote e Validade (para materiais com controle de lote)
    -- TODO: Como será no estoque o controle?
    batch_number VARCHAR(50),
    manufacturing_date DATE,
    expiration_date DATE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT ck_invoice_items_expiration CHECK (
        expiration_date IS NULL OR manufacturing_date IS NULL OR 
        expiration_date > manufacturing_date
    )
);

CREATE INDEX idx_invoice_items_invoice ON invoice_items(invoice_id);
CREATE INDEX idx_invoice_items_catalog_item ON invoice_items(catalog_item_id);
CREATE INDEX idx_invoice_items_unit_conversion ON invoice_items(unit_conversion_id) 
    WHERE unit_conversion_id IS NOT NULL;
CREATE INDEX idx_invoice_items_batch ON invoice_items(batch_number) 
    WHERE batch_number IS NOT NULL;
CREATE INDEX idx_invoice_items_expiration ON invoice_items(expiration_date) 
    WHERE expiration_date IS NOT NULL;

CREATE OR REPLACE FUNCTION fn_update_invoice_totals()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        UPDATE invoices 
        SET total_products = COALESCE((
            SELECT SUM(total_value) FROM invoice_items WHERE invoice_id = OLD.invoice_id
        ), 0),
        total_value = COALESCE((
            SELECT SUM(total_value) FROM invoice_items WHERE invoice_id = OLD.invoice_id
        ), 0) + total_freight - total_discount,
        updated_at = NOW()
        WHERE id = OLD.invoice_id;
        RETURN OLD;
    ELSE
        UPDATE invoices 
        SET total_products = COALESCE((
            SELECT SUM(total_value) FROM invoice_items WHERE invoice_id = NEW.invoice_id
        ), 0),
        total_value = COALESCE((
            SELECT SUM(total_value) FROM invoice_items WHERE invoice_id = NEW.invoice_id
        ), 0) + total_freight - total_discount,
        updated_at = NOW()
        WHERE id = NEW.invoice_id;
        RETURN NEW;
    END IF;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_update_invoice_totals
AFTER INSERT OR UPDATE OR DELETE ON invoice_items
FOR EACH ROW
EXECUTE FUNCTION fn_update_invoice_totals();

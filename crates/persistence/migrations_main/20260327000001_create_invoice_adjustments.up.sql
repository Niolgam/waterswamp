-- ============================================================================
-- Migration: Tabelas de Ajuste de Liquidação (Glosas)
-- Permite estornos parciais ou recusas de itens de NF preservando a NF original
-- ============================================================================

CREATE TABLE invoice_adjustments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    invoice_id UUID NOT NULL REFERENCES invoices(id) ON DELETE RESTRICT,
    reason TEXT NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_invoice_adjustments_invoice ON invoice_adjustments(invoice_id);
CREATE INDEX idx_invoice_adjustments_created_by ON invoice_adjustments(created_by);

CREATE TABLE invoice_adjustment_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    adjustment_id UUID NOT NULL REFERENCES invoice_adjustments(id) ON DELETE CASCADE,
    invoice_item_id UUID NOT NULL REFERENCES invoice_items(id) ON DELETE RESTRICT,
    adjusted_quantity DECIMAL(15, 4) NOT NULL DEFAULT 0 CHECK (adjusted_quantity >= 0),
    adjusted_value DECIMAL(15, 2) NOT NULL DEFAULT 0 CHECK (adjusted_value >= 0),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_invoice_adjustment_items_adjustment ON invoice_adjustment_items(adjustment_id);
CREATE INDEX idx_invoice_adjustment_items_invoice_item ON invoice_adjustment_items(invoice_item_id);

COMMENT ON TABLE invoice_adjustments IS 'Documentos de ajuste de liquidação (glosas) vinculados a notas fiscais já lançadas';
COMMENT ON TABLE invoice_adjustment_items IS 'Itens de ajuste por item de NF: quantidade devolvida e desconto financeiro';
COMMENT ON COLUMN invoice_adjustment_items.adjusted_quantity IS 'Quantidade física devolvida/recusada';
COMMENT ON COLUMN invoice_adjustment_items.adjusted_value IS 'Desconto financeiro aplicado (glosa)';

-- Rastreamento de estoque por lote (RF-021 — Motor FEFO)
-- Cada linha representa a quantidade disponível de um lote em um almoxarifado.
-- Atualizado sincronamente com stock_movements que têm batch_number preenchido.

CREATE TABLE warehouse_batch_stocks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    warehouse_id UUID NOT NULL REFERENCES warehouses(id) ON DELETE CASCADE,
    catalog_item_id UUID NOT NULL REFERENCES catmat_items(id) ON DELETE CASCADE,
    batch_number VARCHAR(50) NOT NULL,

    -- Data de validade (NULL = item sem validade, vai para o final da fila FEFO)
    expiration_date DATE,

    -- Saldo físico do lote neste almoxarifado
    quantity DECIMAL(15, 3) NOT NULL DEFAULT 0 CHECK (quantity >= 0),

    -- Custo médio ponderado deste lote (snapshot da entrada)
    unit_cost DECIMAL(15, 4) NOT NULL DEFAULT 0,

    -- Status de qualidade
    is_quarantined BOOLEAN NOT NULL DEFAULT FALSE,
    quarantine_reason TEXT,
    quarantined_at TIMESTAMPTZ,
    quarantined_by UUID REFERENCES users(id) ON DELETE SET NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_batch_stocks UNIQUE (warehouse_id, catalog_item_id, batch_number)
);

CREATE INDEX idx_batch_stocks_warehouse_item ON warehouse_batch_stocks (warehouse_id, catalog_item_id);
-- Índice FEFO: menor expiration_date primeiro (NULLS LAST = sem validade vai por último)
CREATE INDEX idx_batch_stocks_fefo ON warehouse_batch_stocks
    (warehouse_id, catalog_item_id, expiration_date ASC NULLS LAST)
    WHERE quantity > 0 AND is_quarantined = FALSE;
CREATE INDEX idx_batch_stocks_expiring ON warehouse_batch_stocks (expiration_date)
    WHERE expiration_date IS NOT NULL AND quantity > 0;

CREATE TRIGGER set_batch_stocks_updated_at
    BEFORE UPDATE ON warehouse_batch_stocks
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE warehouse_batch_stocks IS 'Saldo por lote por almoxarifado. Alimentado por stock_movements com batch_number. Base para o motor FEFO (RF-021).';

-- Seed: configurações FEFO
INSERT INTO system_settings (key, value, value_type, category, description)
VALUES
    ('fefo.enabled', 'true', 'boolean', 'fefo',
     'Habilita o motor FEFO (First Expired, First Out) para saídas de estoque (RF-021)'),
    ('fefo.expiry_alert_days', '30', 'number', 'fefo',
     'Dias de antecedência para alertar sobre lotes próximos do vencimento'),
    ('fefo.allow_expired_exit', 'false', 'boolean', 'fefo',
     'Se false, bloqueia saída de lotes com expiration_date < CURRENT_DATE')
ON CONFLICT (key) DO NOTHING;

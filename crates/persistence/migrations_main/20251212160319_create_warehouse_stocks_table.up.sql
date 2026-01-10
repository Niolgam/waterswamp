CREATE TABLE warehouse_stocks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Relacionamentos
    warehouse_id UUID NOT NULL REFERENCES warehouses(id) ON DELETE CASCADE,
    catalog_item_id UUID NOT NULL REFERENCES catalog_items(id) ON DELETE RESTRICT,

    -- Quantidades (Estado Atual)
    quantity DECIMAL(15, 3) NOT NULL DEFAULT 0 CHECK (quantity >= 0),
    reserved_quantity DECIMAL(15, 3) NOT NULL DEFAULT 0 CHECK (reserved_quantity >= 0),
    
    -- Valor médio ponderado
    average_unit_value DECIMAL(15, 4) NOT NULL DEFAULT 0 CHECK (average_unit_value >= 0),

    -- Parâmetros de Controle e Reposição
    min_stock DECIMAL(15, 3) CHECK (min_stock IS NULL OR min_stock >= 0),
    max_stock DECIMAL(15, 3) CHECK (max_stock IS NULL OR max_stock >= 0),
    reorder_point DECIMAL(15, 3), -- Ponto de pedido
    resupply_days INTEGER DEFAULT 0, -- Lead time do fornecedor (em dias)

    -- Localização física
    location VARCHAR(100), -- Ex: Corredor A, Prateleira 2
    secondary_location VARCHAR(100), -- Localização alternativa
    
    -- Bloqueio administrativo
    is_blocked BOOLEAN NOT NULL DEFAULT FALSE,
    block_reason TEXT,
    blocked_at TIMESTAMPTZ,
    blocked_by UUID REFERENCES users(id) ON DELETE RESTRICT,

    -- Última movimentação (cache para relatórios)
    last_entry_at TIMESTAMPTZ,
    last_exit_at TIMESTAMPTZ,
    last_inventory_at TIMESTAMPTZ, -- Último inventário físico

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_warehouse_stocks_item UNIQUE (warehouse_id, catalog_item_id),
    CONSTRAINT ck_warehouse_stocks_min_max CHECK (
        min_stock IS NULL OR max_stock IS NULL OR min_stock <= max_stock
    ),
    CONSTRAINT ck_warehouse_stocks_reserved CHECK (reserved_quantity <= quantity),
    CONSTRAINT ck_warehouse_stocks_block_reason CHECK (
        (is_blocked = TRUE AND block_reason IS NOT NULL) OR
        (is_blocked = FALSE)
    )
);

CREATE INDEX idx_warehouse_stocks_warehouse ON warehouse_stocks(warehouse_id);
CREATE INDEX idx_warehouse_stocks_catalog_item ON warehouse_stocks(catalog_item_id);
CREATE INDEX idx_warehouse_stocks_location ON warehouse_stocks(warehouse_id, location) 
    WHERE location IS NOT NULL;
CREATE INDEX idx_warehouse_stocks_available ON warehouse_stocks(warehouse_id, catalog_item_id) 
    WHERE (quantity - reserved_quantity) > 0;
CREATE INDEX idx_warehouse_stocks_blocked ON warehouse_stocks(warehouse_id) 
    WHERE is_blocked = TRUE;
CREATE INDEX idx_warehouse_stocks_low ON warehouse_stocks(warehouse_id)
    WHERE min_stock IS NOT NULL AND quantity <= min_stock;
CREATE INDEX idx_warehouse_stocks_updated ON warehouse_stocks(updated_at DESC);

CREATE TRIGGER set_timestamp_warehouse_stocks
BEFORE UPDATE ON warehouse_stocks
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE OR REPLACE FUNCTION fn_get_available_quantity(
    p_warehouse_id UUID,
    p_catalog_item_id UUID
) RETURNS DECIMAL(15, 3) AS $$
DECLARE
    v_available DECIMAL(15, 3);
BEGIN
    SELECT COALESCE(quantity - reserved_quantity, 0)
    INTO v_available
    FROM warehouse_stocks
    WHERE warehouse_id = p_warehouse_id 
      AND catalog_item_id = p_catalog_item_id
      AND is_blocked = FALSE;
    
    RETURN COALESCE(v_available, 0);
END;
$$ LANGUAGE plpgsql STABLE;

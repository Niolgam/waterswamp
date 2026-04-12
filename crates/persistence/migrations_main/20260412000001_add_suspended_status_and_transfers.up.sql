-- ============================================================================
-- Migration: DRS Features - SUSPENDED status + stock_transfers table
-- ============================================================================

-- 1. Add SUSPENDED status to requisition_status_enum
ALTER TYPE requisition_status_enum ADD VALUE IF NOT EXISTS 'SUSPENDED';

-- 2. Create stock_transfer_status_enum
DO $$ BEGIN
    CREATE TYPE stock_transfer_status_enum AS ENUM (
        'PENDING',      -- Iniciada pela origem, aguardando confirmação do destino
        'CONFIRMED',    -- Destino confirmou recebimento (efetivada)
        'REJECTED',     -- Destino rejeitou o recebimento (origem restaurada)
        'CANCELLED',    -- Cancelada pela origem antes da confirmação
        'EXPIRED'       -- Timeout de confirmação expirou (automático)
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- 3. Create stock_transfers table
CREATE TABLE IF NOT EXISTS stock_transfers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    transfer_number VARCHAR(50) NOT NULL UNIQUE, -- gerado automaticamente

    source_warehouse_id UUID NOT NULL REFERENCES warehouses(id) ON DELETE RESTRICT,
    destination_warehouse_id UUID NOT NULL REFERENCES warehouses(id) ON DELETE RESTRICT,

    status stock_transfer_status_enum NOT NULL DEFAULT 'PENDING',

    -- Documentação
    notes TEXT,
    rejection_reason TEXT,
    cancellation_reason TEXT,

    -- Responsáveis
    initiated_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    confirmed_by UUID REFERENCES users(id) ON DELETE RESTRICT,
    rejected_by UUID REFERENCES users(id) ON DELETE RESTRICT,
    cancelled_by UUID REFERENCES users(id) ON DELETE RESTRICT,

    -- Timestamps
    initiated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    confirmed_at TIMESTAMPTZ,
    rejected_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ, -- prazo para confirmação (nullable = sem prazo)

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT ck_transfer_different_warehouses
        CHECK (source_warehouse_id != destination_warehouse_id)
);

-- 4. Create stock_transfer_items table
CREATE TABLE IF NOT EXISTS stock_transfer_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    transfer_id UUID NOT NULL REFERENCES stock_transfers(id) ON DELETE CASCADE,
    catalog_item_id UUID NOT NULL REFERENCES catmat_items(id) ON DELETE RESTRICT,

    -- Quantidades
    quantity_requested DECIMAL(15, 3) NOT NULL CHECK (quantity_requested > 0),
    quantity_confirmed DECIMAL(15, 3), -- pode ser menor (recebimento parcial)

    -- Movimentações geradas
    source_movement_id UUID REFERENCES stock_movements(id) ON DELETE SET NULL,
    destination_movement_id UUID REFERENCES stock_movements(id) ON DELETE SET NULL,

    -- Unidade
    unit_raw_id UUID NOT NULL REFERENCES units_of_measure(id) ON DELETE RESTRICT,
    unit_conversion_id UUID REFERENCES unit_conversions(id) ON DELETE SET NULL,
    conversion_factor DECIMAL(15, 4) NOT NULL DEFAULT 1.0,

    -- Lote
    batch_number VARCHAR(50),
    expiration_date DATE,

    notes TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (transfer_id, catalog_item_id)
);

-- 5. Trigger: atualizar updated_at em stock_transfers
CREATE TRIGGER set_timestamp_stock_transfers
BEFORE UPDATE ON stock_transfers
FOR EACH ROW EXECUTE FUNCTION trigger_set_timestamp();

-- 6. Trigger: gerar transfer_number automaticamente
CREATE OR REPLACE FUNCTION fn_generate_transfer_number()
RETURNS TRIGGER AS $$
DECLARE
    v_year TEXT;
    v_seq INT;
BEGIN
    v_year := TO_CHAR(NOW(), 'YYYY');
    SELECT COALESCE(MAX(CAST(SPLIT_PART(transfer_number, '-', 3) AS INT)), 0) + 1
    INTO v_seq
    FROM stock_transfers
    WHERE transfer_number LIKE 'TRF-' || v_year || '-%';

    NEW.transfer_number := 'TRF-' || v_year || '-' || LPAD(v_seq::TEXT, 6, '0');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_generate_transfer_number
BEFORE INSERT ON stock_transfers
FOR EACH ROW
WHEN (NEW.transfer_number IS NULL OR NEW.transfer_number = '')
EXECUTE FUNCTION fn_generate_transfer_number();

-- 7. Indexes
CREATE INDEX IF NOT EXISTS idx_stock_transfers_source ON stock_transfers(source_warehouse_id);
CREATE INDEX IF NOT EXISTS idx_stock_transfers_destination ON stock_transfers(destination_warehouse_id);
CREATE INDEX IF NOT EXISTS idx_stock_transfers_status ON stock_transfers(status);
CREATE INDEX IF NOT EXISTS idx_stock_transfers_initiated_by ON stock_transfers(initiated_by);
CREATE INDEX IF NOT EXISTS idx_stock_transfer_items_transfer ON stock_transfer_items(transfer_id);
CREATE INDEX IF NOT EXISTS idx_stock_transfer_items_catalog ON stock_transfer_items(catalog_item_id);

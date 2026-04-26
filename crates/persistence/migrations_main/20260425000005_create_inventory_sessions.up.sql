-- Ticket 1.3 (RF-019) + Ticket 1.2 (RF-049): Motor de Inventário com tolerância
-- parametrizável e assinatura Gov.br no documento de conciliação.

CREATE TYPE inventory_session_status_enum AS ENUM (
    'OPEN',
    'COUNTING',
    'RECONCILING',
    'COMPLETED',
    'CANCELLED'
);

CREATE TABLE inventory_sessions (
    id                           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    warehouse_id                 UUID NOT NULL REFERENCES warehouses(id) ON DELETE RESTRICT,
    status                       inventory_session_status_enum NOT NULL DEFAULT 'OPEN',
    -- Percentual de tolerância de divergência (ex: 0.02 = 2%).
    -- Divergências acima deste limiar bloqueiam a conciliação até o SEI ser informado (RN-012).
    tolerance_percentage         DECIMAL(5,4) NOT NULL DEFAULT 0.02,
    -- Obrigatório quando algum item tem divergência > tolerance_percentage (RN-012)
    sei_process_number           VARCHAR(20),
    notes                        TEXT,
    created_by                   UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    counting_started_at          TIMESTAMPTZ,
    reconciliation_started_at    TIMESTAMPTZ,
    completed_at                 TIMESTAMPTZ,
    cancelled_at                 TIMESTAMPTZ,
    -- Assinatura Gov.br do documento de conciliação (RF-049)
    govbr_signed_at              TIMESTAMPTZ,
    govbr_signed_by              UUID REFERENCES users(id) ON DELETE RESTRICT,
    created_at                   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE inventory_session_items (
    id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id       UUID NOT NULL REFERENCES inventory_sessions(id) ON DELETE CASCADE,
    catalog_item_id  UUID NOT NULL REFERENCES catmat_items(id) ON DELETE RESTRICT,
    unit_raw_id      UUID NOT NULL REFERENCES units_of_measure(id) ON DELETE RESTRICT,
    -- Quantidade do sistema no momento em que start-counting foi chamado
    system_quantity  DECIMAL(15,4) NOT NULL DEFAULT 0,
    -- Preenchido pelo contador físico
    counted_quantity DECIMAL(15,4),
    -- Preenchido após reconciliação (referência ao movimento ADJUSTMENT gerado)
    movement_id      UUID REFERENCES stock_movements(id) ON DELETE SET NULL,
    notes            TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (session_id, catalog_item_id)
);

CREATE INDEX idx_inventory_sessions_warehouse ON inventory_sessions(warehouse_id);
CREATE INDEX idx_inventory_sessions_status    ON inventory_sessions(status);
CREATE INDEX idx_inventory_session_items_session ON inventory_session_items(session_id);

-- Semente de configuração: tolerância padrão de inventário (2%)
INSERT INTO system_settings (key, value, value_type, description, category)
VALUES (
    'inventory.tolerance_percentage',
    '0.02',
    'number',
    'Percentual máximo de divergência tolerado sem exigir número SEI (ex: 0.02 = 2%)',
    'inventory'
) ON CONFLICT (key) DO NOTHING;

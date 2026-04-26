-- Ticket 1.1 (RN-005): Desfazimento com bloqueio transitório aguardando assinatura Gov.br.
-- O estoque só é deduzido após confirm-signature, não no momento da solicitação.

CREATE TYPE disposal_request_status_enum AS ENUM (
    'AWAITING_SIGNATURE',
    'SIGNED',
    'CANCELLED'
);

CREATE TABLE disposal_requests (
    id                    UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    warehouse_id          UUID NOT NULL REFERENCES warehouses(id) ON DELETE RESTRICT,
    sei_process_number    VARCHAR(20) NOT NULL,
    justification         TEXT NOT NULL,
    technical_opinion_url TEXT NOT NULL,
    status                disposal_request_status_enum NOT NULL DEFAULT 'AWAITING_SIGNATURE',
    notes                 TEXT,
    requested_by          UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    requested_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    signed_by             UUID REFERENCES users(id) ON DELETE RESTRICT,
    signed_at             TIMESTAMPTZ,
    cancelled_by          UUID REFERENCES users(id) ON DELETE RESTRICT,
    cancelled_at          TIMESTAMPTZ,
    cancellation_reason   TEXT,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE disposal_request_items (
    id                   UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    disposal_request_id  UUID NOT NULL REFERENCES disposal_requests(id) ON DELETE CASCADE,
    catalog_item_id      UUID NOT NULL REFERENCES catmat_items(id) ON DELETE RESTRICT,
    unit_raw_id          UUID NOT NULL REFERENCES units_of_measure(id) ON DELETE RESTRICT,
    unit_conversion_id   UUID REFERENCES unit_conversions(id) ON DELETE RESTRICT,
    quantity_raw         DECIMAL(15,4) NOT NULL CHECK (quantity_raw > 0),
    conversion_factor    DECIMAL(15,6) NOT NULL DEFAULT 1,
    batch_number         VARCHAR(100),
    notes                TEXT,
    -- Preenchido após assinatura quando o movimento de estoque é criado
    movement_id          UUID REFERENCES stock_movements(id) ON DELETE SET NULL,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_disposal_requests_warehouse ON disposal_requests(warehouse_id);
CREATE INDEX idx_disposal_requests_status    ON disposal_requests(status);
CREATE INDEX idx_disposal_requests_requested_by ON disposal_requests(requested_by);
CREATE INDEX idx_disposal_request_items_request ON disposal_request_items(disposal_request_id);

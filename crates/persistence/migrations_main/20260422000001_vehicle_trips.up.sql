-- ==========================================================================
-- Épico 2 — RF-USO: Programação e Uso de Veículos
--
-- RF-USO-01: Solicitação de viagem (PENDENTE → APROVADA | REJEITADA).
-- RF-USO-02: Checkin (APROVADA → CHECKIN): registra km saída, associa condutor,
--             vehicle.allocation_status → EM_USO.
-- RF-USO-03: Checkout (CHECKIN → CONCLUIDA): registra km retorno,
--             vehicle.allocation_status → LIVRE.
-- RF-USO-04: Cancelamento em qualquer estado pré-CHECKIN.
--
-- FSM: PENDENTE → APROVADA → CHECKIN → CONCLUIDA
--                → REJEITADA (terminal)
--      PENDENTE | APROVADA → CANCELADA (terminal)
-- ==========================================================================

CREATE TYPE trip_status_enum AS ENUM (
    'PENDENTE',
    'APROVADA',
    'REJEITADA',
    'CHECKIN',
    'CONCLUIDA',
    'CANCELADA'
);

CREATE TABLE vehicle_trips (
    id                      UUID            PRIMARY KEY DEFAULT uuid_generate_v4(),
    vehicle_id              UUID            NOT NULL REFERENCES vehicles(id) ON DELETE RESTRICT,
    driver_id               UUID            REFERENCES drivers(id) ON DELETE SET NULL,
    requester_id            UUID,
    -- Dados da solicitação
    destino                 TEXT            NOT NULL,
    finalidade              TEXT            NOT NULL,
    passageiros             INTEGER         NOT NULL DEFAULT 0,
    data_saida_prevista     TIMESTAMPTZ     NOT NULL,
    data_retorno_prevista   TIMESTAMPTZ,
    notes                   TEXT,
    -- FSM
    status                  trip_status_enum NOT NULL DEFAULT 'PENDENTE',
    -- Aprovação
    aprovado_por            UUID,
    aprovado_em             TIMESTAMPTZ,
    motivo_rejeicao         TEXT,
    -- Checkin
    checkin_em              TIMESTAMPTZ,
    checkin_por             UUID,
    checkin_km              BIGINT,
    checkin_odometer_id     UUID,           -- FK lógica para leituras_hodometro(id)
    -- Checkout
    checkout_em             TIMESTAMPTZ,
    checkout_por            UUID,
    checkout_km             BIGINT,
    checkout_odometer_id    UUID,
    km_percorridos          BIGINT GENERATED ALWAYS AS (
                                CASE
                                    WHEN checkout_km IS NOT NULL AND checkin_km IS NOT NULL
                                    THEN checkout_km - checkin_km
                                    ELSE NULL
                                END
                            ) STORED,
    -- Cancelamento
    cancelado_por           UUID,
    cancelado_em            TIMESTAMPTZ,
    motivo_cancelamento     TEXT,
    -- OCC
    version                 INTEGER         NOT NULL DEFAULT 1,
    created_by              UUID,
    updated_by              UUID,
    created_at              TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vtrips_vehicle_id   ON vehicle_trips(vehicle_id);
CREATE INDEX idx_vtrips_driver_id    ON vehicle_trips(driver_id) WHERE driver_id IS NOT NULL;
CREATE INDEX idx_vtrips_status       ON vehicle_trips(status) WHERE status NOT IN ('CONCLUIDA', 'REJEITADA', 'CANCELADA');
CREATE INDEX idx_vtrips_saida        ON vehicle_trips(data_saida_prevista);
CREATE INDEX idx_vtrips_requester    ON vehicle_trips(requester_id) WHERE requester_id IS NOT NULL;

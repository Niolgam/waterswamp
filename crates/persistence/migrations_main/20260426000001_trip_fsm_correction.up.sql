-- ==========================================================================
-- Trip FSM Correction (GAP 1-3)
--
-- GAP 1: Replace trip_status_enum with DRS-correct 9-state FSM.
--   Old: PENDENTE, APROVADA, REJEITADA, CHECKIN, CONCLUIDA, CANCELADA
--   New: SOLICITADA, APROVADA, ALOCADA, EM_CURSO, AGUARDANDO_PC,
--        CONCLUIDA, CANCELADA, REJEITADA, CONFLITO_MANUAL
--
-- GAP 2: Add columns for new states (ALOCADA, AGUARDANDO_PC, CONFLITO_MANUAL).
--
-- GAP 3: Swap column semantics — DRS defines:
--   Check-out = saída (departure); Check-in = retorno (return).
--   Old schema had it backwards: checkin_* held departure data.
-- ==========================================================================

-- ── Step 1: Recreate trip_status_enum ────────────────────────────────────

CREATE TYPE trip_status_enum_v2 AS ENUM (
    'SOLICITADA',
    'APROVADA',
    'ALOCADA',
    'EM_CURSO',
    'AGUARDANDO_PC',
    'CONCLUIDA',
    'CANCELADA',
    'REJEITADA',
    'CONFLITO_MANUAL'
);

ALTER TABLE vehicle_trips ALTER COLUMN status DROP DEFAULT;
ALTER TABLE vehicle_trips ALTER COLUMN status TYPE TEXT;

DROP TYPE trip_status_enum;
ALTER TYPE trip_status_enum_v2 RENAME TO trip_status_enum;

-- Migrate existing rows
UPDATE vehicle_trips SET status = 'SOLICITADA' WHERE status = 'PENDENTE';
UPDATE vehicle_trips SET status = 'EM_CURSO'   WHERE status = 'CHECKIN';

ALTER TABLE vehicle_trips
    ALTER COLUMN status TYPE trip_status_enum USING status::trip_status_enum;
ALTER TABLE vehicle_trips
    ALTER COLUMN status SET DEFAULT 'SOLICITADA';

-- ── Step 2: Swap checkin/checkout column semantics ────────────────────────
-- Drop generated column before renaming its source columns.
ALTER TABLE vehicle_trips DROP COLUMN km_percorridos;

-- Move old checkin_* (departure) to temp names
ALTER TABLE vehicle_trips RENAME COLUMN checkin_em           TO _tmp_dep_em;
ALTER TABLE vehicle_trips RENAME COLUMN checkin_por          TO _tmp_dep_por;
ALTER TABLE vehicle_trips RENAME COLUMN checkin_km           TO _tmp_dep_km;
ALTER TABLE vehicle_trips RENAME COLUMN checkin_odometer_id  TO _tmp_dep_odometer_id;

-- Move old checkout_* (return) to the new checkin_* (return) names
ALTER TABLE vehicle_trips RENAME COLUMN checkout_em          TO checkin_em;
ALTER TABLE vehicle_trips RENAME COLUMN checkout_por         TO checkin_por;
ALTER TABLE vehicle_trips RENAME COLUMN checkout_km          TO checkin_km;
ALTER TABLE vehicle_trips RENAME COLUMN checkout_odometer_id TO checkin_odometer_id;

-- Move temp (departure) to the new checkout_* (departure) names
ALTER TABLE vehicle_trips RENAME COLUMN _tmp_dep_em          TO checkout_em;
ALTER TABLE vehicle_trips RENAME COLUMN _tmp_dep_por         TO checkout_por;
ALTER TABLE vehicle_trips RENAME COLUMN _tmp_dep_km          TO checkout_km;
ALTER TABLE vehicle_trips RENAME COLUMN _tmp_dep_odometer_id TO checkout_odometer_id;

-- Re-add km_percorridos: return_km − departure_km
ALTER TABLE vehicle_trips
    ADD COLUMN km_percorridos BIGINT GENERATED ALWAYS AS (
        CASE
            WHEN checkin_km IS NOT NULL AND checkout_km IS NOT NULL
            THEN checkin_km - checkout_km
            ELSE NULL
        END
    ) STORED;

-- ── Step 3: Add columns for new FSM states ───────────────────────────────

-- APROVADA → ALOCADA
ALTER TABLE vehicle_trips ADD COLUMN allocated_at    TIMESTAMPTZ;
ALTER TABLE vehicle_trips ADD COLUMN allocated_by    UUID;

-- EM_CURSO → AGUARDANDO_PC
ALTER TABLE vehicle_trips ADD COLUMN waiting_pc_at   TIMESTAMPTZ;

-- → CONFLITO_MANUAL
ALTER TABLE vehicle_trips ADD COLUMN conflict_reason TEXT;
ALTER TABLE vehicle_trips ADD COLUMN conflict_at     TIMESTAMPTZ;
ALTER TABLE vehicle_trips ADD COLUMN conflict_by     UUID;

-- Update partial index to reflect new terminal states
DROP INDEX IF EXISTS idx_vtrips_status;
CREATE INDEX idx_vtrips_status ON vehicle_trips(status)
    WHERE status NOT IN ('CONCLUIDA', 'REJEITADA', 'CANCELADA', 'CONFLITO_MANUAL');

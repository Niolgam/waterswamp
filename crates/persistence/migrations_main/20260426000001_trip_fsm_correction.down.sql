-- Revert Trip FSM Correction
-- NOTE: data migration (SOLICITADA→PENDENTE, EM_CURSO→CHECKIN) is not reversible if rows exist.

ALTER TABLE vehicle_trips DROP COLUMN IF EXISTS conflict_by;
ALTER TABLE vehicle_trips DROP COLUMN IF EXISTS conflict_at;
ALTER TABLE vehicle_trips DROP COLUMN IF EXISTS conflict_reason;
ALTER TABLE vehicle_trips DROP COLUMN IF EXISTS waiting_pc_at;
ALTER TABLE vehicle_trips DROP COLUMN IF EXISTS allocated_by;
ALTER TABLE vehicle_trips DROP COLUMN IF EXISTS allocated_at;

ALTER TABLE vehicle_trips DROP COLUMN km_percorridos;

-- Restore original column names (reverse the swap)
ALTER TABLE vehicle_trips RENAME COLUMN checkout_em          TO _tmp_dep_em;
ALTER TABLE vehicle_trips RENAME COLUMN checkout_por         TO _tmp_dep_por;
ALTER TABLE vehicle_trips RENAME COLUMN checkout_km          TO _tmp_dep_km;
ALTER TABLE vehicle_trips RENAME COLUMN checkout_odometer_id TO _tmp_dep_odometer_id;

ALTER TABLE vehicle_trips RENAME COLUMN checkin_em           TO checkout_em;
ALTER TABLE vehicle_trips RENAME COLUMN checkin_por          TO checkout_por;
ALTER TABLE vehicle_trips RENAME COLUMN checkin_km           TO checkout_km;
ALTER TABLE vehicle_trips RENAME COLUMN checkin_odometer_id  TO checkout_odometer_id;

ALTER TABLE vehicle_trips RENAME COLUMN _tmp_dep_em          TO checkin_em;
ALTER TABLE vehicle_trips RENAME COLUMN _tmp_dep_por         TO checkin_por;
ALTER TABLE vehicle_trips RENAME COLUMN _tmp_dep_km          TO checkin_km;
ALTER TABLE vehicle_trips RENAME COLUMN _tmp_dep_odometer_id TO checkin_odometer_id;

ALTER TABLE vehicle_trips
    ADD COLUMN km_percorridos BIGINT GENERATED ALWAYS AS (
        CASE
            WHEN checkout_km IS NOT NULL AND checkin_km IS NOT NULL
            THEN checkout_km - checkin_km ELSE NULL
        END
    ) STORED;

CREATE TYPE trip_status_enum_old AS ENUM (
    'PENDENTE', 'APROVADA', 'REJEITADA', 'CHECKIN', 'CONCLUIDA', 'CANCELADA'
);

ALTER TABLE vehicle_trips ALTER COLUMN status DROP DEFAULT;
ALTER TABLE vehicle_trips ALTER COLUMN status TYPE TEXT;

DROP TYPE trip_status_enum;
ALTER TYPE trip_status_enum_old RENAME TO trip_status_enum;

UPDATE vehicle_trips SET status = 'PENDENTE' WHERE status = 'SOLICITADA';
UPDATE vehicle_trips SET status = 'CHECKIN'  WHERE status = 'EM_CURSO';
UPDATE vehicle_trips SET status = 'CONCLUIDA' WHERE status IN ('AGUARDANDO_PC', 'CONFLITO_MANUAL');
DELETE FROM vehicle_trips WHERE status = 'ALOCADA';

ALTER TABLE vehicle_trips
    ALTER COLUMN status TYPE trip_status_enum USING status::trip_status_enum;
ALTER TABLE vehicle_trips ALTER COLUMN status SET DEFAULT 'PENDENTE';

DROP INDEX IF EXISTS idx_vtrips_status;
CREATE INDEX idx_vtrips_status ON vehicle_trips(status)
    WHERE status NOT IN ('CONCLUIDA', 'REJEITADA', 'CANCELADA');

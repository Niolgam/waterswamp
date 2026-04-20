-- Reverte a migration vehicle_fsm_dual_axis

DROP INDEX IF EXISTS idx_drivers_status_cnh;
DROP INDEX IF EXISTS idx_drivers_credenciamento_status;

ALTER TABLE drivers
    DROP COLUMN IF EXISTS credenciamento_status;

DROP TYPE IF EXISTS credenciamento_status_enum;

ALTER TABLE vehicle_status_history
    DROP COLUMN IF EXISTS new_allocation_status,
    DROP COLUMN IF EXISTS old_allocation_status,
    DROP COLUMN IF EXISTS new_operational_status,
    DROP COLUMN IF EXISTS old_operational_status;

DROP INDEX IF EXISTS idx_vehicles_status_combo;
DROP INDEX IF EXISTS idx_vehicles_allocation_status;
DROP INDEX IF EXISTS idx_vehicles_operational_status;

ALTER TABLE vehicles
    DROP COLUMN IF EXISTS allocation_status,
    DROP COLUMN IF EXISTS operational_status;

DROP TYPE IF EXISTS allocation_status_enum;
DROP TYPE IF EXISTS operational_status_enum;

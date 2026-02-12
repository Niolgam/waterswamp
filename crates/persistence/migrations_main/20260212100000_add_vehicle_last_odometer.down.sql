DROP TRIGGER IF EXISTS trg_update_vehicle_last_odometer ON fuelings;
DROP FUNCTION IF EXISTS update_vehicle_last_odometer();
DROP INDEX IF EXISTS idx_fuelings_vehicle_date_km;
ALTER TABLE vehicles
    DROP COLUMN IF EXISTS last_fueling_id,
    DROP COLUMN IF EXISTS last_odometer_date,
    DROP COLUMN IF EXISTS last_odometer_km;

DROP INDEX IF EXISTS idx_fuelings_vehicle_date;
DROP INDEX IF EXISTS idx_fuelings_trip;
DROP INDEX IF EXISTS idx_fuelings_fuel_catalog;

ALTER TABLE fuelings
    DROP COLUMN IF EXISTS consumo_litros_100km,
    DROP COLUMN IF EXISTS km_anterior,
    DROP COLUMN IF EXISTS trip_id,
    DROP COLUMN IF EXISTS fuel_catalog_id;

DROP INDEX IF EXISTS idx_vtrips_requester;
DROP INDEX IF EXISTS idx_vtrips_saida;
DROP INDEX IF EXISTS idx_vtrips_status;
DROP INDEX IF EXISTS idx_vtrips_driver_id;
DROP INDEX IF EXISTS idx_vtrips_vehicle_id;
DROP TABLE IF EXISTS vehicle_trips;
DROP TYPE IF EXISTS trip_status_enum;

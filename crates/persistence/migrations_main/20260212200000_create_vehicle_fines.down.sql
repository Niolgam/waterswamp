DROP TRIGGER IF EXISTS set_vehicle_fines_updated_at ON vehicle_fines;
DROP TABLE IF EXISTS vehicle_fines;

DROP TRIGGER IF EXISTS set_vehicle_fine_types_updated_at ON vehicle_fine_types;
DROP TABLE IF EXISTS vehicle_fine_types;

DROP TYPE IF EXISTS fine_payment_status_enum;
DROP TYPE IF EXISTS fine_severity_enum;

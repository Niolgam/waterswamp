-- Reverte a migration vehicle_department_transfers

DROP INDEX IF EXISTS idx_vdt_data_efetiva;
DROP INDEX IF EXISTS idx_vdt_vehicle_id;
DROP TABLE IF EXISTS vehicle_department_transfers;

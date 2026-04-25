DROP INDEX IF EXISTS idx_vds_disposal_id;
DROP INDEX IF EXISTS idx_vdp_status;
DROP INDEX IF EXISTS idx_vdp_vehicle_id;
DROP TABLE IF EXISTS vehicle_disposal_steps;
DROP TABLE IF EXISTS vehicle_disposal_processes;
DROP TYPE IF EXISTS disposal_destination_enum;
DROP TYPE IF EXISTS disposal_status_enum;

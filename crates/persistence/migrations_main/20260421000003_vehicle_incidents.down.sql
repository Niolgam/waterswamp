-- Reverte a migration vehicle_incidents

DROP INDEX IF EXISTS idx_vehicle_incidents_status;
DROP INDEX IF EXISTS idx_vehicle_incidents_vehicle_id;
DROP TABLE IF EXISTS vehicle_incidents;
DROP TYPE IF EXISTS vehicle_incident_status_enum;
DROP TYPE IF EXISTS vehicle_incident_type_enum;

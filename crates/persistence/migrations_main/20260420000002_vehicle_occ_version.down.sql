-- Reverte a migration vehicle_occ_version

ALTER TABLE fuelings DROP COLUMN IF EXISTS version;
ALTER TABLE drivers  DROP COLUMN IF EXISTS version;
ALTER TABLE vehicles DROP COLUMN IF EXISTS version;

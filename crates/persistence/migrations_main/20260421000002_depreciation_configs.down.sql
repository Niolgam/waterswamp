-- Reverte a migration depreciation_configs

DROP INDEX IF EXISTS idx_depreciation_configs_category;
DROP TABLE IF EXISTS depreciation_configs;

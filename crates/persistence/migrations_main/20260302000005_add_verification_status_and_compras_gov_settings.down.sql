-- Remove verification_status from all catalog tables
ALTER TABLE catmat_groups DROP COLUMN IF EXISTS verification_status;
ALTER TABLE catmat_classes DROP COLUMN IF EXISTS verification_status;
ALTER TABLE catmat_pdms DROP COLUMN IF EXISTS verification_status;
ALTER TABLE catmat_items DROP COLUMN IF EXISTS verification_status;
ALTER TABLE catser_sections DROP COLUMN IF EXISTS verification_status;
ALTER TABLE catser_divisions DROP COLUMN IF EXISTS verification_status;
ALTER TABLE catser_groups DROP COLUMN IF EXISTS verification_status;
ALTER TABLE catser_classes DROP COLUMN IF EXISTS verification_status;
ALTER TABLE catser_items DROP COLUMN IF EXISTS verification_status;

-- Remove ComprasGov system settings
DELETE FROM system_settings WHERE key IN (
    'compras_gov.validation_enabled',
    'compras_gov.catmat_api_base_url',
    'compras_gov.catser_api_base_url'
);

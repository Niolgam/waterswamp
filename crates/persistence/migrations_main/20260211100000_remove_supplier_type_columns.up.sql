-- Remove supplier_type and is_international_neighborhood columns
DROP INDEX IF EXISTS idx_suppliers_supplier_type;
ALTER TABLE suppliers DROP COLUMN IF EXISTS supplier_type;
ALTER TABLE suppliers DROP COLUMN IF EXISTS is_international_neighborhood;
DROP TYPE IF EXISTS supplier_type_enum;

-- Restore supplier_type_enum, supplier_type column, is_international_neighborhood column, and index
CREATE TYPE supplier_type_enum AS ENUM ('INDIVIDUAL', 'LEGAL_ENTITY', 'GOVERNMENT_UNIT');
ALTER TABLE suppliers ADD COLUMN supplier_type supplier_type_enum NOT NULL DEFAULT 'INDIVIDUAL';
ALTER TABLE suppliers ADD COLUMN is_international_neighborhood BOOLEAN NOT NULL DEFAULT false;
CREATE INDEX idx_suppliers_supplier_type ON suppliers (supplier_type);

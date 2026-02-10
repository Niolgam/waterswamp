-- Supplier type enum
CREATE TYPE supplier_type_enum AS ENUM ('INDIVIDUAL', 'LEGAL_ENTITY', 'GOVERNMENT_UNIT');

CREATE TABLE suppliers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Type
    supplier_type supplier_type_enum NOT NULL,
    -- Names
    legal_name VARCHAR(200) NOT NULL,
    trade_name VARCHAR(200),
    -- Document: CPF (11 digits) for Individual, CNPJ (14 digits) for Legal Entity, UG code for Government Unit
    document_number VARCHAR(20) NOT NULL,
    -- Representative
    representative_name VARCHAR(200),
    -- Address
    address VARCHAR(300),
    neighborhood VARCHAR(100),
    is_international_neighborhood BOOLEAN NOT NULL DEFAULT false,
    city_id UUID REFERENCES cities(id),
    zip_code VARCHAR(10),
    -- Contact
    email VARCHAR(200),
    phone VARCHAR(30),
    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_by UUID,
    -- Constraints
    CONSTRAINT uq_suppliers_document_number UNIQUE (document_number)
);

CREATE INDEX idx_suppliers_supplier_type ON suppliers (supplier_type);
CREATE INDEX idx_suppliers_city ON suppliers (city_id);
CREATE INDEX idx_suppliers_legal_name_trgm ON suppliers USING gin (legal_name gin_trgm_ops);
CREATE INDEX idx_suppliers_document_number_trgm ON suppliers USING gin (document_number gin_trgm_ops);
CREATE INDEX idx_suppliers_is_active ON suppliers (is_active);

CREATE TRIGGER set_suppliers_updated_at
    BEFORE UPDATE ON suppliers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

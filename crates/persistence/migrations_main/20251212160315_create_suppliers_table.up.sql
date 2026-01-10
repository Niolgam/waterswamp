CREATE TYPE supplier_type_enum AS ENUM ('PJ', 'PF', 'MEI');

CREATE TABLE suppliers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL, -- Razão Social
    trade_name VARCHAR(255), -- Nome Fantasia
    tax_id VARCHAR(20) NOT NULL, -- CNPJ ou CPF (sem formatação)
    supplier_type supplier_type_enum NOT NULL DEFAULT 'PJ',
    state_registration VARCHAR(20), -- Inscrição Estadual
    municipal_registration VARCHAR(20), -- Inscrição Municipal
    
    city_id UUID REFERENCES cities(id) ON DELETE SET NULL,
    address TEXT,
    zip_code VARCHAR(8),
    
    email VARCHAR(255),
    phone VARCHAR(20),
    contact_name VARCHAR(100), -- Nome do contato principal

    bank_name VARCHAR(100),
    bank_agency VARCHAR(10),
    bank_account VARCHAR(20),
    pix_key VARCHAR(100),

    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT uq_suppliers_tax_id UNIQUE (tax_id),
    CONSTRAINT ck_suppliers_tax_id_length CHECK (
        (supplier_type = 'PJ' AND length(tax_id) = 14) OR
        (supplier_type IN ('PF', 'MEI') AND length(tax_id) = 11)
    )
);

CREATE INDEX idx_suppliers_tax_id ON suppliers(tax_id);
CREATE INDEX idx_suppliers_name ON suppliers(name);
CREATE INDEX idx_suppliers_trade_name ON suppliers(trade_name) WHERE trade_name IS NOT NULL;
CREATE INDEX idx_suppliers_city ON suppliers(city_id) WHERE city_id IS NOT NULL;
CREATE INDEX idx_suppliers_active ON suppliers(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_suppliers_search ON suppliers USING gin ((name || ' ' || COALESCE(trade_name, '')) gin_trgm_ops);

CREATE TRIGGER set_timestamp_suppliers
BEFORE UPDATE ON suppliers
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

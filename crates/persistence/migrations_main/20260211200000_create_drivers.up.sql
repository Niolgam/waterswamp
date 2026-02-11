-- Driver type enum
CREATE TYPE driver_type_enum AS ENUM ('OUTSOURCED', 'SERVER');

CREATE TABLE drivers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Type: terceirizado or servidor
    driver_type driver_type_enum NOT NULL,
    -- Personal info
    full_name VARCHAR(200) NOT NULL,
    cpf VARCHAR(11) NOT NULL,
    -- Driver's license (CNH)
    cnh_number VARCHAR(20) NOT NULL,
    cnh_category VARCHAR(5) NOT NULL,
    cnh_expiration DATE NOT NULL,
    -- Contact
    phone VARCHAR(30),
    email VARCHAR(200),
    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_by UUID,
    -- Constraints
    CONSTRAINT uq_drivers_cpf UNIQUE (cpf),
    CONSTRAINT uq_drivers_cnh_number UNIQUE (cnh_number)
);

CREATE INDEX idx_drivers_driver_type ON drivers (driver_type);
CREATE INDEX idx_drivers_full_name_trgm ON drivers USING gin (full_name gin_trgm_ops);
CREATE INDEX idx_drivers_cpf_trgm ON drivers USING gin (cpf gin_trgm_ops);
CREATE INDEX idx_drivers_is_active ON drivers (is_active);

CREATE TRIGGER set_drivers_updated_at
    BEFORE UPDATE ON drivers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

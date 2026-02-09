-- ============================
-- Vehicle Fleet Management Schema
-- ============================

-- Enum types
CREATE TYPE vehicle_status_enum AS ENUM (
    'ACTIVE',
    'IN_MAINTENANCE',
    'RESERVED',
    'INACTIVE',
    'DECOMMISSIONING'
);

CREATE TYPE acquisition_type_enum AS ENUM (
    'PURCHASE',
    'DONATION',
    'CESSION',
    'TRANSFER'
);

CREATE TYPE decommission_reason_enum AS ENUM (
    'TOTAL_LOSS',
    'END_OF_LIFE',
    'UNECONOMICAL',
    'OTHER'
);

CREATE TYPE decommission_destination_enum AS ENUM (
    'AUCTION',
    'SCRAP',
    'DONATION',
    'OTHER'
);

CREATE TYPE document_type_enum AS ENUM (
    'CRLV',
    'INVOICE',
    'DONATION_TERM',
    'INSURANCE_POLICY',
    'TECHNICAL_REPORT',
    'PHOTO',
    'OTHER'
);

-- ============================
-- Vehicle Categories
-- ============================
CREATE TABLE vehicle_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_vehicle_categories_name UNIQUE (name)
);

CREATE TRIGGER set_vehicle_categories_updated_at
    BEFORE UPDATE ON vehicle_categories
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================
-- Vehicle Makes (Marcas)
-- ============================
CREATE TABLE vehicle_makes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_vehicle_makes_name UNIQUE (name)
);

CREATE TRIGGER set_vehicle_makes_updated_at
    BEFORE UPDATE ON vehicle_makes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================
-- Vehicle Models (Modelos)
-- ============================
CREATE TABLE vehicle_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    make_id UUID NOT NULL REFERENCES vehicle_makes(id),
    name VARCHAR(100) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_vehicle_models_name_make UNIQUE (make_id, name)
);

CREATE TRIGGER set_vehicle_models_updated_at
    BEFORE UPDATE ON vehicle_models
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================
-- Colors
-- ============================
CREATE TABLE vehicle_colors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL,
    hex_code VARCHAR(7),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_vehicle_colors_name UNIQUE (name)
);

CREATE TRIGGER set_vehicle_colors_updated_at
    BEFORE UPDATE ON vehicle_colors
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================
-- Fuel Types
-- ============================
CREATE TABLE vehicle_fuel_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_vehicle_fuel_types_name UNIQUE (name)
);

CREATE TRIGGER set_vehicle_fuel_types_updated_at
    BEFORE UPDATE ON vehicle_fuel_types
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================
-- Transmission Types
-- ============================
CREATE TABLE vehicle_transmission_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_vehicle_transmission_types_name UNIQUE (name)
);

CREATE TRIGGER set_vehicle_transmission_types_updated_at
    BEFORE UPDATE ON vehicle_transmission_types
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================
-- Vehicles (Main table)
-- ============================
CREATE TABLE vehicles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Identification
    license_plate VARCHAR(10) NOT NULL,
    chassis_number VARCHAR(17) NOT NULL,
    renavam VARCHAR(11) NOT NULL,
    engine_number VARCHAR(30),
    -- Classification
    category_id UUID NOT NULL REFERENCES vehicle_categories(id),
    make_id UUID NOT NULL REFERENCES vehicle_makes(id),
    model_id UUID NOT NULL REFERENCES vehicle_models(id),
    color_id UUID NOT NULL REFERENCES vehicle_colors(id),
    fuel_type_id UUID NOT NULL REFERENCES vehicle_fuel_types(id),
    transmission_type_id UUID REFERENCES vehicle_transmission_types(id),
    -- Year
    manufacture_year INT NOT NULL,
    model_year INT NOT NULL,
    -- Technical specs
    passenger_capacity INT,
    load_capacity_kg NUMERIC(10,2),
    engine_displacement INT,       -- cilindradas (cc)
    horsepower INT,
    -- Acquisition
    acquisition_type acquisition_type_enum NOT NULL DEFAULT 'PURCHASE',
    acquisition_date DATE,
    purchase_value NUMERIC(14,2),
    -- Institutional
    patrimony_number VARCHAR(50),
    department_id UUID,            -- FK to organizational_units if needed
    -- Status
    status vehicle_status_enum NOT NULL DEFAULT 'ACTIVE',
    -- Soft delete
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    deleted_at TIMESTAMPTZ,
    deleted_by UUID,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_by UUID,
    -- Constraints
    CONSTRAINT uq_vehicles_license_plate UNIQUE (license_plate),
    CONSTRAINT uq_vehicles_chassis_number UNIQUE (chassis_number),
    CONSTRAINT uq_vehicles_renavam UNIQUE (renavam),
    CONSTRAINT chk_vehicles_manufacture_year CHECK (manufacture_year >= 1900 AND manufacture_year <= 2100),
    CONSTRAINT chk_vehicles_model_year CHECK (model_year >= 1900 AND model_year <= 2100),
    CONSTRAINT chk_vehicles_passenger_capacity CHECK (passenger_capacity IS NULL OR passenger_capacity > 0),
    CONSTRAINT chk_vehicles_load_capacity CHECK (load_capacity_kg IS NULL OR load_capacity_kg > 0),
    CONSTRAINT chk_vehicles_purchase_value CHECK (purchase_value IS NULL OR purchase_value >= 0)
);

CREATE INDEX idx_vehicles_status ON vehicles (status) WHERE is_deleted = false;
CREATE INDEX idx_vehicles_category ON vehicles (category_id) WHERE is_deleted = false;
CREATE INDEX idx_vehicles_make ON vehicles (make_id) WHERE is_deleted = false;
CREATE INDEX idx_vehicles_department ON vehicles (department_id) WHERE is_deleted = false;
CREATE INDEX idx_vehicles_license_plate_trgm ON vehicles USING gin (license_plate gin_trgm_ops);
CREATE INDEX idx_vehicles_chassis_trgm ON vehicles USING gin (chassis_number gin_trgm_ops);
CREATE INDEX idx_vehicles_is_deleted ON vehicles (is_deleted);

CREATE TRIGGER set_vehicles_updated_at
    BEFORE UPDATE ON vehicles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================
-- Vehicle Documents
-- ============================
CREATE TABLE vehicle_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    document_type document_type_enum NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    description TEXT,
    uploaded_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vehicle_documents_vehicle ON vehicle_documents (vehicle_id);
CREATE INDEX idx_vehicle_documents_type ON vehicle_documents (document_type);

CREATE TRIGGER set_vehicle_documents_updated_at
    BEFORE UPDATE ON vehicle_documents
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================
-- Vehicle Status History (Audit trail)
-- ============================
CREATE TABLE vehicle_status_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    old_status vehicle_status_enum,
    new_status vehicle_status_enum NOT NULL,
    reason TEXT,
    changed_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vehicle_status_history_vehicle ON vehicle_status_history (vehicle_id);
CREATE INDEX idx_vehicle_status_history_date ON vehicle_status_history (created_at);

-- ============================
-- Seed data for lookup tables
-- ============================
INSERT INTO vehicle_categories (name, description) VALUES
    ('Passeio', 'Veículo de passeio'),
    ('Utilitário', 'Veículo utilitário'),
    ('Caminhão', 'Caminhão de carga'),
    ('Ônibus', 'Ônibus de transporte'),
    ('Van', 'Van de transporte'),
    ('Motocicleta', 'Motocicleta'),
    ('Ambulância', 'Veículo de emergência médica');

INSERT INTO vehicle_fuel_types (name) VALUES
    ('Gasolina'),
    ('Etanol'),
    ('Diesel'),
    ('Flex'),
    ('GNV'),
    ('Elétrico'),
    ('Híbrido');

INSERT INTO vehicle_transmission_types (name) VALUES
    ('Manual'),
    ('Automático'),
    ('CVT'),
    ('Automatizado');

INSERT INTO vehicle_colors (name) VALUES
    ('Branco'),
    ('Preto'),
    ('Prata'),
    ('Cinza'),
    ('Vermelho'),
    ('Azul'),
    ('Verde'),
    ('Amarelo');

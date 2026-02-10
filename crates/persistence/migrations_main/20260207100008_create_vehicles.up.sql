CREATE TABLE vehicles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Identificação
    license_plate VARCHAR(10) NOT NULL,
    chassis_number VARCHAR(17) NOT NULL,
    renavam VARCHAR(11) NOT NULL,
    engine_number VARCHAR(25),
    -- Componentes mecânicos (nº série)
    bomba_injetora VARCHAR(25),
    caixa_cambio VARCHAR(25),
    diferencial VARCHAR(25),
    -- Classificação (model carrega make e category)
    model_id UUID NOT NULL REFERENCES vehicle_models(id),
    color_id UUID NOT NULL REFERENCES vehicle_colors(id),
    fuel_type_id UUID NOT NULL REFERENCES vehicle_fuel_types(id),
    transmission_type_id UUID REFERENCES vehicle_transmission_types(id),
    -- Ano
    manufacture_year INT NOT NULL,
    model_year INT NOT NULL,
    -- Operacional
    frota VARCHAR(30),
    rateio BOOLEAN NOT NULL DEFAULT false,
    km_inicial NUMERIC(12,2),
    cap_tanque_comb NUMERIC(10,2),
    -- Aquisição
    acquisition_type acquisition_type_enum NOT NULL DEFAULT 'PURCHASE',
    acquisition_date DATE,
    purchase_value NUMERIC(14,2),
    -- Institucional
    patrimony_number VARCHAR(50),
    department_id UUID,
    -- Status
    status vehicle_status_enum NOT NULL DEFAULT 'ACTIVE',
    -- Observações
    observacoes TEXT,
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
    CONSTRAINT chk_vehicles_km_inicial CHECK (km_inicial IS NULL OR km_inicial >= 0),
    CONSTRAINT chk_vehicles_cap_tanque CHECK (cap_tanque_comb IS NULL OR cap_tanque_comb > 0),
    CONSTRAINT chk_vehicles_purchase_value CHECK (purchase_value IS NULL OR purchase_value >= 0)
);

CREATE INDEX idx_vehicles_status ON vehicles (status) WHERE is_deleted = false;
CREATE INDEX idx_vehicles_model ON vehicles (model_id) WHERE is_deleted = false;
CREATE INDEX idx_vehicles_department ON vehicles (department_id) WHERE is_deleted = false;
CREATE INDEX idx_vehicles_frota ON vehicles (frota) WHERE is_deleted = false AND frota IS NOT NULL;
CREATE INDEX idx_vehicles_license_plate_trgm ON vehicles USING gin (license_plate gin_trgm_ops);
CREATE INDEX idx_vehicles_chassis_trgm ON vehicles USING gin (chassis_number gin_trgm_ops);
CREATE INDEX idx_vehicles_is_deleted ON vehicles (is_deleted);

CREATE TRIGGER set_vehicles_updated_at
    BEFORE UPDATE ON vehicles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

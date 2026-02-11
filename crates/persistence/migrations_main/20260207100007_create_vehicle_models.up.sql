CREATE TABLE vehicle_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    make_id UUID NOT NULL REFERENCES vehicle_makes(id),
    category_id UUID REFERENCES vehicle_categories(id),
    name VARCHAR(100) NOT NULL,
    -- Technical specifications
    passenger_capacity INT,
    engine_displacement INT,              -- cc
    horsepower INT,
    load_capacity NUMERIC(10,2),          -- kg
    -- Fuel consumption averages (km/l)
    avg_consumption_min NUMERIC(10,2),
    avg_consumption_max NUMERIC(10,2),
    avg_consumption_target NUMERIC(10,2),
    --
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_vehicle_models_name_make UNIQUE (make_id, name),
    CONSTRAINT chk_vehicle_models_passenger_capacity CHECK (passenger_capacity IS NULL OR passenger_capacity > 0),
    CONSTRAINT chk_vehicle_models_load_capacity CHECK (load_capacity IS NULL OR load_capacity > 0),
    CONSTRAINT chk_vehicle_models_avg_consumption_min CHECK (avg_consumption_min IS NULL OR avg_consumption_min > 0),
    CONSTRAINT chk_vehicle_models_avg_consumption_max CHECK (avg_consumption_max IS NULL OR avg_consumption_max > 0),
    CONSTRAINT chk_vehicle_models_avg_consumption_target CHECK (avg_consumption_target IS NULL OR avg_consumption_target > 0)
);

CREATE INDEX idx_vehicle_models_make ON vehicle_models (make_id);
CREATE INDEX idx_vehicle_models_category ON vehicle_models (category_id);

CREATE TRIGGER set_vehicle_models_updated_at
    BEFORE UPDATE ON vehicle_models
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

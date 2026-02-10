CREATE TABLE vehicle_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    make_id UUID NOT NULL REFERENCES vehicle_makes(id),
    category_id UUID REFERENCES vehicle_categories(id),
    name VARCHAR(100) NOT NULL,
    -- Especificações técnicas do modelo
    passenger_capacity INT,
    engine_displacement INT,              -- cilindradas (cc)
    horsepower INT,
    capacidade_carga NUMERIC(10,2),       -- capacidade de carga (kg)
    -- Médias de consumo (km/l)
    media_min NUMERIC(10,2),
    media_max NUMERIC(10,2),
    media_desejada NUMERIC(10,2),
    --
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_vehicle_models_name_make UNIQUE (make_id, name),
    CONSTRAINT chk_vehicle_models_passenger_capacity CHECK (passenger_capacity IS NULL OR passenger_capacity > 0),
    CONSTRAINT chk_vehicle_models_capacidade_carga CHECK (capacidade_carga IS NULL OR capacidade_carga > 0),
    CONSTRAINT chk_vehicle_models_media_min CHECK (media_min IS NULL OR media_min > 0),
    CONSTRAINT chk_vehicle_models_media_max CHECK (media_max IS NULL OR media_max > 0),
    CONSTRAINT chk_vehicle_models_media_desejada CHECK (media_desejada IS NULL OR media_desejada > 0)
);

CREATE INDEX idx_vehicle_models_make ON vehicle_models (make_id);
CREATE INDEX idx_vehicle_models_category ON vehicle_models (category_id);

CREATE TRIGGER set_vehicle_models_updated_at
    BEFORE UPDATE ON vehicle_models
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

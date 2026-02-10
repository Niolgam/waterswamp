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

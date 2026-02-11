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

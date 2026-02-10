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

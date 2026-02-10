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

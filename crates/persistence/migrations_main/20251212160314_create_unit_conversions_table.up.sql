CREATE TABLE unit_conversions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    catalog_item_id UUID NOT NULL REFERENCES catalog_items(id) ON DELETE CASCADE,
    from_unit_id UUID NOT NULL REFERENCES units_of_measure(id) ON DELETE RESTRICT,
    to_unit_id UUID NOT NULL REFERENCES units_of_measure(id) ON DELETE RESTRICT,
    conversion_factor DECIMAL(12, 4) NOT NULL CHECK (conversion_factor > 0),
    description VARCHAR(100), -- Descrição amigável (Ex: "Caixa com 12 unidades")

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_unit_conversion_item_units UNIQUE (catalog_item_id, from_unit_id, to_unit_id),
    CONSTRAINT ck_unit_conversion_different_units CHECK (from_unit_id <> to_unit_id)
);

CREATE INDEX idx_unit_conversions_catalog_item ON unit_conversions(catalog_item_id);
CREATE INDEX idx_unit_conversions_from_unit ON unit_conversions(from_unit_id);
CREATE INDEX idx_unit_conversions_to_unit ON unit_conversions(to_unit_id);
CREATE INDEX idx_unit_conversions_lookup ON unit_conversions(catalog_item_id, from_unit_id);
CREATE INDEX idx_unit_conversions_description ON unit_conversions 
USING gin (description gin_trgm_ops) WHERE description IS NOT NULL;

CREATE TRIGGER set_timestamp_unit_conversions
BEFORE UPDATE ON unit_conversions
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

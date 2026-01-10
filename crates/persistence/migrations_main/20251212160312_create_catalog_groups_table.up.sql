CREATE TYPE item_type_enum AS ENUM ('MATERIAL', 'SERVICE');

CREATE TABLE catalog_groups (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(10) NOT NULL,
    name VARCHAR(200) NOT NULL,
    item_type item_type_enum NOT NULL DEFAULT 'MATERIAL',
    budget_classification_id UUID NOT NULL REFERENCES budget_classifications(id) ON DELETE RESTRICT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_catalog_groups_code UNIQUE (code),
    CONSTRAINT uq_catalog_groups_name_type UNIQUE (name, item_type)
);

-- Índices
CREATE INDEX idx_catalog_groups_code ON catalog_groups(code);
CREATE INDEX idx_catalog_groups_name ON catalog_groups(name);
CREATE INDEX idx_catalog_groups_item_type ON catalog_groups(item_type);
CREATE INDEX idx_catalog_groups_budget ON catalog_groups(budget_classification_id);
CREATE INDEX idx_catalog_groups_active ON catalog_groups(is_active) WHERE is_active = TRUE;

-- Trigger para updated_at
CREATE TRIGGER set_timestamp_catalog_groups
BEFORE UPDATE ON catalog_groups
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

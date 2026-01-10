CREATE TYPE warehouse_type_enum AS ENUM ('CENTRAL', 'SECTOR');

CREATE TABLE warehouses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(200) NOT NULL,
    code VARCHAR(50) NOT NULL,

    warehouse_type warehouse_type_enum NOT NULL DEFAULT 'SECTOR',
    city_id UUID NOT NULL REFERENCES cities(id) ON DELETE RESTRICT,

    -- Responsável técnico (Usuário do sistema)
    -- TODO: Adicionar FK quando tabela users existir
    responsible_user_id UUID,

    -- Unidade administrativa/Setor responsável (ex: Secretaria de Saúde)
    -- TODO: Adicionar FK quando tabela organizational_units existir
    responsible_unit_id UUID,

    -- Permissões Logísticas
    allows_transfers BOOLEAN NOT NULL DEFAULT TRUE, -- Permite enviar/receber de outros estoques
    is_budgetary BOOLEAN NOT NULL DEFAULT FALSE, -- Indica se as entradas geram impacto orçamentário

    address VARCHAR(500),
    phone VARCHAR(20),
    email VARCHAR(255),

    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_warehouses_code UNIQUE (code)
);

CREATE INDEX idx_warehouses_code ON warehouses(code);
CREATE INDEX idx_warehouses_type ON warehouses(warehouse_type);
CREATE INDEX idx_warehouses_city ON warehouses(city_id);
CREATE INDEX idx_warehouses_responsible_unit ON warehouses(responsible_unit_id) 
    WHERE responsible_unit_id IS NOT NULL;
CREATE INDEX idx_warehouses_active ON warehouses(is_active) WHERE is_active = TRUE;

CREATE TRIGGER set_timestamp_warehouses
BEFORE UPDATE ON warehouses
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

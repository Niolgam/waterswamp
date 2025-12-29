-- Add up migration script here
-- Cria a tabela de grupos de materiais (material_groups)
CREATE TABLE material_groups (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Código numérico do grupo de material (ex: 125, 3022)
    code VARCHAR(10) UNIQUE NOT NULL,

    -- Denominação do grupo (ex: "Material de Expediente")
    name VARCHAR(200) NOT NULL,

    -- Descrição opcional do grupo
    description TEXT,

    -- Elemento de Despesa (ex: "Ajuda de Custo")
    expense_element VARCHAR(200),

    -- Indica se é exclusivo para cadastro de pessoal
    is_personnel_exclusive BOOLEAN NOT NULL DEFAULT FALSE,

    -- Status ativo/inativo
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Índices para otimizar buscas
CREATE INDEX idx_material_groups_code ON material_groups(code);
CREATE INDEX idx_material_groups_name ON material_groups(name);
CREATE INDEX idx_material_groups_is_active ON material_groups(is_active);

-- Trigger para atualizar updated_at automaticamente
CREATE TRIGGER set_timestamp_material_groups
BEFORE UPDATE ON material_groups
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();

-- Cria a tabela de materiais/serviços (materials)
CREATE TABLE materials (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Relacionamento com grupo de material
    material_group_id UUID NOT NULL REFERENCES material_groups(id) ON DELETE RESTRICT,

    -- Denominação do material/serviço (ex: "Água Sanitária")
    name VARCHAR(200) NOT NULL,

    -- Valor estimado do material
    estimated_value DECIMAL(15, 2) NOT NULL CHECK (estimated_value >= 0),

    -- Unidade de medida (ex: "Litro", "Unidade", "Kg")
    unit_of_measure VARCHAR(50) NOT NULL,

    -- Especificação detalhada do material
    specification TEXT NOT NULL,

    -- Links de busca (opcional)
    search_links TEXT,

    -- Código CATMAT (sistema do governo, opcional)
    catmat_code VARCHAR(20),

    -- URL da foto do material (opcional)
    photo_url TEXT,

    -- Status ativo/inativo
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraint: nome único dentro de um mesmo grupo
    CONSTRAINT unique_material_name_per_group UNIQUE (material_group_id, name)
);

-- Índices para otimizar buscas
CREATE INDEX idx_materials_material_group_id ON materials(material_group_id);
CREATE INDEX idx_materials_name ON materials(name);
CREATE INDEX idx_materials_is_active ON materials(is_active);
CREATE INDEX idx_materials_catmat_code ON materials(catmat_code) WHERE catmat_code IS NOT NULL;

-- Trigger para atualizar updated_at automaticamente
CREATE TRIGGER set_timestamp_materials
BEFORE UPDATE ON materials
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();

-- Inserir alguns grupos de materiais de exemplo
INSERT INTO material_groups (code, name, description, expense_element, is_personnel_exclusive) VALUES
('125', 'Material de Expediente', 'Materiais de escritório e papelaria', 'Ajuda de Custo', FALSE),
('3022', 'Material de Limpeza e Produtos de Higienização', 'Produtos para limpeza e higienização', NULL, FALSE),
('3001', 'Material de Consumo Geral', 'Materiais de uso geral e consumo', NULL, FALSE);

-- Inserir alguns materiais de exemplo
INSERT INTO materials (material_group_id, name, estimated_value, unit_of_measure, specification, catmat_code) VALUES
(
    (SELECT id FROM material_groups WHERE code = '3022'),
    'Água Sanitária',
    7.00,
    'Litro',
    'Água sanitária para limpeza e desinfecção. Concentração mínima de 2,0% a 2,5% de cloro ativo.',
    NULL
),
(
    (SELECT id FROM material_groups WHERE code = '125'),
    'Papel A4',
    25.00,
    'Resma',
    'Papel sulfite branco A4 75g/m². Pacote com 500 folhas.',
    NULL
);

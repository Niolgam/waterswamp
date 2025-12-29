-- Add up migration script here
-- Cria ENUMs para o sistema de almoxarifado

CREATE TYPE movement_type AS ENUM (
    'ENTRADA',
    'SAIDA',
    'AJUSTE',
    'TRANSFERENCIA_SAIDA',
    'TRANSFERENCIA_ENTRADA',
    'PERDA',
    'DEVOLUCAO'
);

CREATE TYPE requisition_status AS ENUM (
    'PENDENTE',
    'APROVADA',
    'REJEITADA',
    'EM_ATENDIMENTO',
    'ATENDIDA',
    'ATENDIDA_PARCIALMENTE',
    'CANCELADA'
);

-- Cria a tabela de Almoxarifados (Warehouses)
CREATE TABLE warehouses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Informações básicas
    name VARCHAR(200) NOT NULL,
    code VARCHAR(50) UNIQUE NOT NULL,

    -- Localização
    city_id UUID NOT NULL REFERENCES cities(id) ON DELETE RESTRICT,

    -- Responsável
    responsible_user_id UUID REFERENCES users(id) ON DELETE SET NULL,

    -- Informações de contato
    address VARCHAR(500),
    phone VARCHAR(20),
    email VARCHAR(255),

    -- Status
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT unique_warehouse_code UNIQUE (code)
);

-- Índices para warehouses
CREATE INDEX idx_warehouses_city_id ON warehouses(city_id);
CREATE INDEX idx_warehouses_code ON warehouses(code);
CREATE INDEX idx_warehouses_is_active ON warehouses(is_active);
CREATE INDEX idx_warehouses_responsible_user_id ON warehouses(responsible_user_id) WHERE responsible_user_id IS NOT NULL;

-- Trigger para atualizar updated_at
CREATE TRIGGER set_timestamp_warehouses
BEFORE UPDATE ON warehouses
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();

-- Cria a tabela de Estoque por Almoxarifado (Warehouse Stocks)
CREATE TABLE warehouse_stocks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Relacionamentos
    warehouse_id UUID NOT NULL REFERENCES warehouses(id) ON DELETE CASCADE,
    material_id UUID NOT NULL REFERENCES materials(id) ON DELETE RESTRICT,

    -- Quantidades e valores
    quantity DECIMAL(15, 3) NOT NULL DEFAULT 0 CHECK (quantity >= 0),
    average_unit_value DECIMAL(15, 2) NOT NULL DEFAULT 0 CHECK (average_unit_value >= 0),

    -- Controle de estoque
    min_stock DECIMAL(15, 3) CHECK (min_stock >= 0),
    max_stock DECIMAL(15, 3) CHECK (max_stock >= 0),

    -- Localização física no almoxarifado
    location VARCHAR(100),

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT unique_warehouse_material UNIQUE (warehouse_id, material_id),
    CONSTRAINT check_min_max_stock CHECK (min_stock IS NULL OR max_stock IS NULL OR min_stock <= max_stock)
);

-- Índices para warehouse_stocks
CREATE INDEX idx_warehouse_stocks_warehouse_id ON warehouse_stocks(warehouse_id);
CREATE INDEX idx_warehouse_stocks_material_id ON warehouse_stocks(material_id);
CREATE INDEX idx_warehouse_stocks_quantity ON warehouse_stocks(quantity);
CREATE INDEX idx_warehouse_stocks_low_stock ON warehouse_stocks(warehouse_id, material_id)
    WHERE min_stock IS NOT NULL AND quantity <= min_stock;

-- Trigger para atualizar updated_at
CREATE TRIGGER set_timestamp_warehouse_stocks
BEFORE UPDATE ON warehouse_stocks
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();

-- Cria a tabela de Movimentações de Estoque (Stock Movements)
CREATE TABLE stock_movements (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Relacionamento
    warehouse_stock_id UUID NOT NULL REFERENCES warehouse_stocks(id) ON DELETE CASCADE,

    -- Tipo de movimentação
    movement_type movement_type NOT NULL,

    -- Quantidades e valores
    quantity DECIMAL(15, 3) NOT NULL,
    unit_value DECIMAL(15, 2) NOT NULL CHECK (unit_value >= 0),
    total_value DECIMAL(15, 2) NOT NULL,

    -- Saldos antes e depois (para auditoria)
    balance_before DECIMAL(15, 3) NOT NULL,
    balance_after DECIMAL(15, 3) NOT NULL,
    average_before DECIMAL(15, 2) NOT NULL,
    average_after DECIMAL(15, 2) NOT NULL,

    -- Data da movimentação
    movement_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Documentação
    document_number VARCHAR(100),
    requisition_id UUID REFERENCES requisitions(id) ON DELETE SET NULL,

    -- Usuário que fez a movimentação
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,

    -- Observações
    notes TEXT,

    -- Timestamp de criação (imutável)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Índices para stock_movements
CREATE INDEX idx_stock_movements_warehouse_stock_id ON stock_movements(warehouse_stock_id);
CREATE INDEX idx_stock_movements_movement_type ON stock_movements(movement_type);
CREATE INDEX idx_stock_movements_movement_date ON stock_movements(movement_date DESC);
CREATE INDEX idx_stock_movements_user_id ON stock_movements(user_id);
CREATE INDEX idx_stock_movements_requisition_id ON stock_movements(requisition_id) WHERE requisition_id IS NOT NULL;
CREATE INDEX idx_stock_movements_document_number ON stock_movements(document_number) WHERE document_number IS NOT NULL;

-- Cria a tabela de Requisições (Requisitions)
CREATE TABLE requisitions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Almoxarifado
    warehouse_id UUID NOT NULL REFERENCES warehouses(id) ON DELETE RESTRICT,

    -- Solicitante
    requester_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,

    -- Status
    status requisition_status NOT NULL DEFAULT 'PENDENTE',

    -- Valor total
    total_value DECIMAL(15, 2) NOT NULL DEFAULT 0,

    -- Datas
    request_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Aprovação
    approved_by UUID REFERENCES users(id) ON DELETE SET NULL,
    approved_at TIMESTAMPTZ,

    -- Atendimento
    fulfilled_by UUID REFERENCES users(id) ON DELETE SET NULL,
    fulfilled_at TIMESTAMPTZ,

    -- Rejeição
    rejection_reason TEXT,

    -- Observações
    notes TEXT,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT check_approved_fields CHECK (
        (status = 'APROVADA' AND approved_by IS NOT NULL AND approved_at IS NOT NULL) OR
        (status != 'APROVADA' AND (approved_by IS NULL OR approved_at IS NULL) OR approved_by IS NOT NULL)
    ),
    CONSTRAINT check_fulfilled_fields CHECK (
        (status IN ('ATENDIDA', 'ATENDIDA_PARCIALMENTE') AND fulfilled_by IS NOT NULL AND fulfilled_at IS NOT NULL) OR
        (status NOT IN ('ATENDIDA', 'ATENDIDA_PARCIALMENTE'))
    ),
    CONSTRAINT check_rejected_reason CHECK (
        (status = 'REJEITADA' AND rejection_reason IS NOT NULL) OR
        (status != 'REJEITADA')
    )
);

-- Índices para requisitions
CREATE INDEX idx_requisitions_warehouse_id ON requisitions(warehouse_id);
CREATE INDEX idx_requisitions_requester_id ON requisitions(requester_id);
CREATE INDEX idx_requisitions_status ON requisitions(status);
CREATE INDEX idx_requisitions_request_date ON requisitions(request_date DESC);
CREATE INDEX idx_requisitions_approved_by ON requisitions(approved_by) WHERE approved_by IS NOT NULL;
CREATE INDEX idx_requisitions_fulfilled_by ON requisitions(fulfilled_by) WHERE fulfilled_by IS NOT NULL;

-- Trigger para atualizar updated_at
CREATE TRIGGER set_timestamp_requisitions
BEFORE UPDATE ON requisitions
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();

-- Cria a tabela de Itens da Requisição (Requisition Items)
CREATE TABLE requisition_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Relacionamentos
    requisition_id UUID NOT NULL REFERENCES requisitions(id) ON DELETE CASCADE,
    material_id UUID NOT NULL REFERENCES materials(id) ON DELETE RESTRICT,

    -- Quantidades
    requested_quantity DECIMAL(15, 3) NOT NULL CHECK (requested_quantity > 0),
    fulfilled_quantity DECIMAL(15, 3) CHECK (fulfilled_quantity >= 0),

    -- Valores (capturados no momento da requisição)
    unit_value DECIMAL(15, 2) NOT NULL,
    total_value DECIMAL(15, 2) NOT NULL,

    -- Timestamp de criação (imutável)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT check_fulfilled_quantity CHECK (
        fulfilled_quantity IS NULL OR fulfilled_quantity <= requested_quantity
    ),
    CONSTRAINT unique_requisition_material UNIQUE (requisition_id, material_id)
);

-- Índices para requisition_items
CREATE INDEX idx_requisition_items_requisition_id ON requisition_items(requisition_id);
CREATE INDEX idx_requisition_items_material_id ON requisition_items(material_id);

-- Adiciona constraint na tabela stock_movements para referenciar requisitions
-- (já está definida acima, mas garantir que existe)
-- ALTER TABLE stock_movements ADD CONSTRAINT fk_stock_movements_requisition
--     FOREIGN KEY (requisition_id) REFERENCES requisitions(id) ON DELETE SET NULL;

-- Inserir dados de exemplo para almoxarifados
INSERT INTO warehouses (name, code, city_id, address, is_active) VALUES
(
    'Almoxarifado Central - Campus Cuiabá',
    'ALM-CBA-01',
    (SELECT id FROM cities WHERE name = 'Cuiabá' LIMIT 1),
    'Av. Fernando Corrêa da Costa, 2367 - Boa Esperança',
    TRUE
);

-- Inserir estoque inicial nos almoxarifados (usando os materiais já cadastrados)
INSERT INTO warehouse_stocks (warehouse_id, material_id, quantity, average_unit_value, min_stock, max_stock, location)
SELECT
    w.id,
    m.id,
    100.000,
    m.estimated_value,
    50.000,
    500.000,
    'Prateleira A-01'
FROM warehouses w
CROSS JOIN materials m
WHERE w.code = 'ALM-CBA-01'
LIMIT 2;  -- Apenas os 2 materiais de exemplo

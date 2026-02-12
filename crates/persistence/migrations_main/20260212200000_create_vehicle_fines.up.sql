-- =============================================================================
-- Enums para multas de veículos
-- =============================================================================

-- Gravidade da infração (CTB Art. 258)
CREATE TYPE fine_severity_enum AS ENUM (
    'LIGHT',         -- Leve
    'MEDIUM',        -- Média
    'SERIOUS',       -- Grave
    'VERY_SERIOUS'   -- Gravíssima
);

-- Status do pagamento da multa
CREATE TYPE fine_payment_status_enum AS ENUM (
    'PENDING',       -- Pendente
    'PAID',          -- Pago
    'OVERDUE',       -- Vencido
    'CANCELLED',     -- Cancelado
    'UNDER_APPEAL'   -- Em recurso
);

-- =============================================================================
-- Tabela de tipos de multas (infrações)
-- =============================================================================
CREATE TABLE vehicle_fine_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(10) NOT NULL UNIQUE,
    description VARCHAR(500) NOT NULL,
    severity fine_severity_enum NOT NULL,
    points INTEGER NOT NULL DEFAULT 0,
    fine_amount NUMERIC(12, 2) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_by UUID,

    CONSTRAINT ck_vehicle_fine_types_points_positive CHECK (points >= 0),
    CONSTRAINT ck_vehicle_fine_types_amount_positive CHECK (fine_amount >= 0)
);

CREATE INDEX idx_vehicle_fine_types_code ON vehicle_fine_types (code);
CREATE INDEX idx_vehicle_fine_types_severity ON vehicle_fine_types (severity) WHERE is_active = true;
CREATE INDEX idx_vehicle_fine_types_is_active ON vehicle_fine_types (is_active);
CREATE INDEX idx_vehicle_fine_types_description_trgm ON vehicle_fine_types USING gin (description gin_trgm_ops);

CREATE TRIGGER set_vehicle_fine_types_updated_at
    BEFORE UPDATE ON vehicle_fine_types
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- =============================================================================
-- Tabela de multas de veículos
-- =============================================================================
CREATE TABLE vehicle_fines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Relacionamentos
    vehicle_id UUID NOT NULL REFERENCES vehicles(id) ON DELETE RESTRICT,
    fine_type_id UUID NOT NULL REFERENCES vehicle_fine_types(id) ON DELETE RESTRICT,
    supplier_id UUID NOT NULL REFERENCES suppliers(id) ON DELETE RESTRICT,
    driver_id UUID REFERENCES drivers(id) ON DELETE SET NULL,

    -- Dados da multa
    auto_number VARCHAR(50),
    fine_date TIMESTAMPTZ NOT NULL,
    notification_date TIMESTAMPTZ,
    due_date TIMESTAMPTZ NOT NULL,
    location VARCHAR(500),

    -- Processo SEI
    sei_process_number VARCHAR(50),

    -- Valores
    fine_amount NUMERIC(12, 2) NOT NULL,
    discount_amount NUMERIC(12, 2),
    paid_amount NUMERIC(12, 2),
    payment_date TIMESTAMPTZ,

    -- Status
    payment_status fine_payment_status_enum NOT NULL DEFAULT 'PENDING',

    -- Observações
    notes TEXT,

    -- Soft delete
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    deleted_at TIMESTAMPTZ,
    deleted_by UUID,

    -- Auditoria
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_by UUID,

    -- Constraints
    CONSTRAINT ck_vehicle_fines_amount_positive CHECK (fine_amount >= 0),
    CONSTRAINT ck_vehicle_fines_discount_positive CHECK (discount_amount IS NULL OR discount_amount >= 0),
    CONSTRAINT ck_vehicle_fines_paid_positive CHECK (paid_amount IS NULL OR paid_amount >= 0)
);

-- Índices para performance
CREATE INDEX idx_vehicle_fines_vehicle ON vehicle_fines (vehicle_id) WHERE is_deleted = false;
CREATE INDEX idx_vehicle_fines_fine_type ON vehicle_fines (fine_type_id) WHERE is_deleted = false;
CREATE INDEX idx_vehicle_fines_supplier ON vehicle_fines (supplier_id) WHERE is_deleted = false;
CREATE INDEX idx_vehicle_fines_driver ON vehicle_fines (driver_id) WHERE is_deleted = false;
CREATE INDEX idx_vehicle_fines_payment_status ON vehicle_fines (payment_status) WHERE is_deleted = false;
CREATE INDEX idx_vehicle_fines_due_date ON vehicle_fines (due_date) WHERE is_deleted = false;
CREATE INDEX idx_vehicle_fines_fine_date ON vehicle_fines (fine_date) WHERE is_deleted = false;
CREATE INDEX idx_vehicle_fines_sei_process ON vehicle_fines (sei_process_number) WHERE sei_process_number IS NOT NULL AND is_deleted = false;
CREATE INDEX idx_vehicle_fines_auto_number_trgm ON vehicle_fines USING gin (auto_number gin_trgm_ops);

CREATE TRIGGER set_vehicle_fines_updated_at
    BEFORE UPDATE ON vehicle_fines
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

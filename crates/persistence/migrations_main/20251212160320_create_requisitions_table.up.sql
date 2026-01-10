CREATE TYPE requisition_status_enum AS ENUM (
    'DRAFT',              -- Rascunho (pode ser editada)
    'PENDING',            -- Aguardando aprovação
    'APPROVED',           -- Aprovada, aguardando atendimento
    'REJECTED',           -- Rejeitada
    'PROCESSING',         -- Em separação/atendimento
    'FULFILLED',          -- Totalmente atendida
    'PARTIALLY_FULFILLED', -- Parcialmente atendida
    'CANCELLED'           -- Cancelada
);

CREATE TYPE requisition_priority_enum AS ENUM (
    'LOW',
    'NORMAL',
    'HIGH',
    'URGENT'
);

CREATE TABLE requisitions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    requisition_number VARCHAR(20) NOT NULL,

    warehouse_id UUID NOT NULL REFERENCES warehouses(id) ON DELETE RESTRICT,
    
    -- Unidade de destino (quem está solicitando)
    -- TODO: FK para organizational_units quando existir
    destination_unit_id UUID NOT NULL,
    destination_unit_name VARCHAR(200), -- Cache do nome para histórico
    
    requester_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT, 
    requester_name VARCHAR(200), -- Cache do nome para histórico

    status requisition_status_enum NOT NULL DEFAULT 'DRAFT',
    priority requisition_priority_enum NOT NULL DEFAULT 'NORMAL',

    total_value DECIMAL(15, 2) NOT NULL DEFAULT 0,

    request_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    needed_by DATE, -- Data limite para atendimento
    
    approved_by UUID REFERENCES users(id) ON DELETE RESTRICT, 
    approved_at TIMESTAMPTZ,
    
    fulfilled_by UUID REFERENCES users(id) ON DELETE RESTRICT,
    fulfilled_at TIMESTAMPTZ,

    rejection_reason TEXT,
    cancellation_reason TEXT,
    notes TEXT,
    internal_notes TEXT, -- Notas visíveis apenas para o almoxarifado

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_requisitions_number UNIQUE (requisition_number),
    CONSTRAINT ck_requisitions_approved CHECK (
        (status IN ('APPROVED', 'PROCESSING', 'FULFILLED', 'PARTIALLY_FULFILLED') 
         AND approved_by IS NOT NULL AND approved_at IS NOT NULL) OR
        (status NOT IN ('APPROVED', 'PROCESSING', 'FULFILLED', 'PARTIALLY_FULFILLED'))
    ),
    CONSTRAINT ck_requisitions_fulfilled CHECK (
        (status IN ('FULFILLED', 'PARTIALLY_FULFILLED') 
         AND fulfilled_by IS NOT NULL AND fulfilled_at IS NOT NULL) OR
        (status NOT IN ('FULFILLED', 'PARTIALLY_FULFILLED'))
    ),
    CONSTRAINT ck_requisitions_rejected CHECK (
        (status = 'REJECTED' AND rejection_reason IS NOT NULL) OR
        (status <> 'REJECTED')
    ),
    CONSTRAINT ck_requisitions_cancelled CHECK (
        (status = 'CANCELLED' AND cancellation_reason IS NOT NULL) OR
        (status <> 'CANCELLED')
    )
);

CREATE INDEX idx_requisitions_number ON requisitions(requisition_number);
CREATE INDEX idx_requisitions_warehouse ON requisitions(warehouse_id);
CREATE INDEX idx_requisitions_destination ON requisitions(destination_unit_id);
CREATE INDEX idx_requisitions_requester ON requisitions(requester_id);
CREATE INDEX idx_requisitions_status ON requisitions(status);
CREATE INDEX idx_requisitions_priority ON requisitions(priority);
CREATE INDEX idx_requisitions_date ON requisitions(request_date DESC);
CREATE INDEX idx_requisitions_needed_by ON requisitions(needed_by) WHERE needed_by IS NOT NULL;
CREATE INDEX idx_requisitions_pending ON requisitions(warehouse_id, request_date)
    WHERE status = 'PENDING';
CREATE INDEX idx_requisitions_to_fulfill ON requisitions(warehouse_id, priority DESC, request_date)
    WHERE status IN ('APPROVED', 'PROCESSING');

CREATE TRIGGER set_timestamp_requisitions
BEFORE UPDATE ON requisitions
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE OR REPLACE FUNCTION fn_generate_requisition_number()
RETURNS TRIGGER AS $$
DECLARE
    v_year TEXT;
    v_sequence INTEGER;
BEGIN
    IF NEW.requisition_number IS NULL OR NEW.requisition_number = '' THEN
        v_year := to_char(NOW(), 'YYYY');
        
        SELECT COALESCE(MAX(
            CAST(SUBSTRING(requisition_number FROM '\d+$') AS INTEGER)
        ), 0) + 1
        INTO v_sequence
        FROM requisitions
        WHERE requisition_number LIKE 'REQ' || v_year || '%';
        
        NEW.requisition_number := 'REQ' || v_year || LPAD(v_sequence::TEXT, 6, '0');
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_generate_requisition_number
BEFORE INSERT ON requisitions
FOR EACH ROW
EXECUTE FUNCTION fn_generate_requisition_number();

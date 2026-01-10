CREATE TABLE requisition_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Relacionamentos
    requisition_id UUID NOT NULL REFERENCES requisitions(id) ON DELETE CASCADE,
    catalog_item_id UUID NOT NULL REFERENCES catalog_items(id) ON DELETE RESTRICT,

    -- Quantidades (usando unidade base do item)
    requested_quantity DECIMAL(15, 3) NOT NULL CHECK (requested_quantity > 0),
    approved_quantity DECIMAL(15, 3) CHECK (approved_quantity >= 0), 
    fulfilled_quantity DECIMAL(15, 3) NOT NULL DEFAULT 0 CHECK (fulfilled_quantity >= 0),

    -- Valores de referência (Snapshot do average_unit_value no momento da criação)
    unit_value DECIMAL(15, 4) NOT NULL,
    total_value DECIMAL(15, 2) NOT NULL,

    -- Justificativa do item (opcional)
    justification TEXT,
    
    -- Motivo de corte (se approved_quantity < requested_quantity)
    cut_reason TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT ck_requisition_items_approved CHECK (
        approved_quantity IS NULL OR approved_quantity <= requested_quantity
    ),
    CONSTRAINT ck_requisition_items_fulfilled CHECK (
        fulfilled_quantity <= COALESCE(approved_quantity, requested_quantity)
    ),
    CONSTRAINT uq_requisition_items_item UNIQUE (requisition_id, catalog_item_id)
);

CREATE INDEX idx_requisition_items_requisition ON requisition_items(requisition_id);
CREATE INDEX idx_requisition_items_catalog_item ON requisition_items(catalog_item_id);
CREATE INDEX idx_requisition_items_pending ON requisition_items(requisition_id)
    WHERE fulfilled_quantity < COALESCE(approved_quantity, requested_quantity);

CREATE OR REPLACE FUNCTION fn_update_requisition_total()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        UPDATE requisitions 
        SET total_value = COALESCE((
            SELECT SUM(total_value) FROM requisition_items WHERE requisition_id = OLD.requisition_id
        ), 0),
        updated_at = NOW()
        WHERE id = OLD.requisition_id;
        RETURN OLD;
    ELSE
        UPDATE requisitions 
        SET total_value = COALESCE((
            SELECT SUM(total_value) FROM requisition_items WHERE requisition_id = NEW.requisition_id
        ), 0),
        updated_at = NOW()
        WHERE id = NEW.requisition_id;
        RETURN NEW;
    END IF;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_update_requisition_total
AFTER INSERT OR UPDATE OR DELETE ON requisition_items
FOR EACH ROW
EXECUTE FUNCTION fn_update_requisition_total();

CREATE OR REPLACE FUNCTION fn_capture_requisition_item_value()
RETURNS TRIGGER AS $$
DECLARE
    v_avg_value DECIMAL(15, 4);
BEGIN
    SELECT COALESCE(ws.average_unit_value, ci.estimated_value, 0)
    INTO v_avg_value
    FROM catalog_items ci
    LEFT JOIN warehouse_stocks ws ON ws.catalog_item_id = ci.id 
        AND ws.warehouse_id = (SELECT warehouse_id FROM requisitions WHERE id = NEW.requisition_id)
    WHERE ci.id = NEW.catalog_item_id;
    
    NEW.unit_value := v_avg_value;
    NEW.total_value := NEW.requested_quantity * v_avg_value;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_capture_requisition_item_value
BEFORE INSERT ON requisition_items
FOR EACH ROW
EXECUTE FUNCTION fn_capture_requisition_item_value();

CREATE TABLE stock_reservations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    requisition_id UUID NOT NULL REFERENCES requisitions(id) ON DELETE CASCADE,
    requisition_item_id UUID NOT NULL REFERENCES requisition_items(id) ON DELETE CASCADE,
    catalog_item_id UUID NOT NULL REFERENCES catalog_items(id) ON DELETE RESTRICT,
    warehouse_id UUID NOT NULL REFERENCES warehouses(id) ON DELETE RESTRICT,
    quantity DECIMAL(15, 3) NOT NULL CHECK (quantity > 0),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    
    consumed_at TIMESTAMPTZ,
    released_at TIMESTAMPTZ,
    release_reason TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT ck_stock_reservations_status CHECK (
        (is_active = TRUE AND consumed_at IS NULL AND released_at IS NULL) OR
        (is_active = FALSE AND (consumed_at IS NOT NULL OR released_at IS NOT NULL))
    )
);

CREATE INDEX idx_stock_reservations_requisition ON stock_reservations(requisition_id);
CREATE INDEX idx_stock_reservations_requisition_item ON stock_reservations(requisition_item_id);
CREATE INDEX idx_stock_reservations_catalog_item ON stock_reservations(catalog_item_id);
CREATE INDEX idx_stock_reservations_warehouse ON stock_reservations(warehouse_id);
CREATE INDEX idx_stock_reservations_active ON stock_reservations(warehouse_id, catalog_item_id)
    WHERE is_active = TRUE;

CREATE OR REPLACE FUNCTION fn_manage_stock_reservation()
RETURNS TRIGGER AS $$
BEGIN
    -- CENÁRIO A: Requisição aprovada -> Criar Reservas
    IF (NEW.status = 'APPROVED' AND OLD.status = 'PENDING') THEN
        -- Criar reservas para cada item
        INSERT INTO stock_reservations (
            requisition_id, 
            requisition_item_id,
            catalog_item_id, 
            warehouse_id, 
            quantity
        )
        SELECT 
            NEW.id,
            ri.id,
            ri.catalog_item_id, 
            NEW.warehouse_id, 
            COALESCE(ri.approved_quantity, ri.requested_quantity)
        FROM requisition_items ri
        WHERE ri.requisition_id = NEW.id;

        -- Atualiza o total reservado no warehouse_stocks
        UPDATE warehouse_stocks ws
        SET reserved_quantity = reserved_quantity + COALESCE(ri.approved_quantity, ri.requested_quantity),
            updated_at = NOW()
        FROM requisition_items ri
        WHERE ri.requisition_id = NEW.id 
          AND ws.warehouse_id = NEW.warehouse_id 
          AND ws.catalog_item_id = ri.catalog_item_id;

    -- CENÁRIO B: Requisição Finalizada, Rejeitada ou Cancelada -> Liberar Reservas
    ELSIF (NEW.status IN ('FULFILLED', 'PARTIALLY_FULFILLED', 'REJECTED', 'CANCELLED') 
           AND OLD.status IN ('APPROVED', 'PROCESSING')) THEN
        
        -- Diminui o reserved_quantity na warehouse_stocks
        UPDATE warehouse_stocks ws
        SET reserved_quantity = GREATEST(0, reserved_quantity - sr.quantity),
            updated_at = NOW()
        FROM stock_reservations sr
        WHERE sr.requisition_id = NEW.id 
          AND sr.is_active = TRUE
          AND ws.warehouse_id = NEW.warehouse_id 
          AND ws.catalog_item_id = sr.catalog_item_id;

        -- Desativa as reservas
        UPDATE stock_reservations 
        SET is_active = FALSE,
            released_at = CASE WHEN NEW.status IN ('REJECTED', 'CANCELLED') THEN NOW() ELSE NULL END,
            consumed_at = CASE WHEN NEW.status IN ('FULFILLED', 'PARTIALLY_FULFILLED') THEN NOW() ELSE NULL END,
            release_reason = CASE 
                WHEN NEW.status = 'REJECTED' THEN NEW.rejection_reason
                WHEN NEW.status = 'CANCELLED' THEN NEW.cancellation_reason
                ELSE NULL
            END
        WHERE requisition_id = NEW.id AND is_active = TRUE;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_handle_requisition_reservation
AFTER UPDATE OF status ON requisitions
FOR EACH ROW
EXECUTE FUNCTION fn_manage_stock_reservation();

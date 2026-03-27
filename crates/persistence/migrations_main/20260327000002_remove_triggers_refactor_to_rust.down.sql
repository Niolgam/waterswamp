-- Reverter remoção de triggers — restaurar lógica no banco de dados

-- 6. Recálculo de totais da requisição
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

-- 5. Recálculo de totais da NF
CREATE OR REPLACE FUNCTION fn_update_invoice_totals()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        UPDATE invoices
        SET total_products = COALESCE((
            SELECT SUM(total_value) FROM invoice_items WHERE invoice_id = OLD.invoice_id
        ), 0),
        total_value = COALESCE((
            SELECT SUM(total_value) FROM invoice_items WHERE invoice_id = OLD.invoice_id
        ), 0) + total_freight - total_discount,
        updated_at = NOW()
        WHERE id = OLD.invoice_id;
        RETURN OLD;
    ELSE
        UPDATE invoices
        SET total_products = COALESCE((
            SELECT SUM(total_value) FROM invoice_items WHERE invoice_id = NEW.invoice_id
        ), 0),
        total_value = COALESCE((
            SELECT SUM(total_value) FROM invoice_items WHERE invoice_id = NEW.invoice_id
        ), 0) + total_freight - total_discount,
        updated_at = NOW()
        WHERE id = NEW.invoice_id;
        RETURN NEW;
    END IF;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_update_invoice_totals
AFTER INSERT OR UPDATE OR DELETE ON invoice_items
FOR EACH ROW
EXECUTE FUNCTION fn_update_invoice_totals();

-- 4. Captura de preço no momento da requisição
CREATE OR REPLACE FUNCTION fn_capture_requisition_item_value()
RETURNS TRIGGER AS $$
DECLARE
    v_avg_value DECIMAL(15, 4);
BEGIN
    SELECT COALESCE(ws.average_unit_value, ci.estimated_value, 0)
    INTO v_avg_value
    FROM catmat_items ci
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

-- 3. Gestão de reservas de requisição
CREATE OR REPLACE FUNCTION fn_manage_stock_reservation()
RETURNS TRIGGER AS $$
BEGIN
    IF (NEW.status = 'APPROVED' AND OLD.status = 'PENDING') THEN
        INSERT INTO stock_reservations (requisition_id, requisition_item_id, catalog_item_id, warehouse_id, quantity)
        SELECT NEW.id, ri.id, ri.catalog_item_id, NEW.warehouse_id,
               COALESCE(ri.approved_quantity, ri.requested_quantity)
        FROM requisition_items ri
        WHERE ri.requisition_id = NEW.id;

        UPDATE warehouse_stocks ws
        SET reserved_quantity = reserved_quantity + COALESCE(ri.approved_quantity, ri.requested_quantity),
            updated_at = NOW()
        FROM requisition_items ri
        WHERE ri.requisition_id = NEW.id
          AND ws.warehouse_id = NEW.warehouse_id
          AND ws.catalog_item_id = ri.catalog_item_id;
    ELSIF (NEW.status IN ('FULFILLED', 'PARTIALLY_FULFILLED', 'REJECTED', 'CANCELLED')
           AND OLD.status IN ('APPROVED', 'PROCESSING')) THEN
        UPDATE warehouse_stocks ws
        SET reserved_quantity = GREATEST(0, reserved_quantity - sr.quantity),
            updated_at = NOW()
        FROM stock_reservations sr
        WHERE sr.requisition_id = NEW.id
          AND sr.is_active = TRUE
          AND ws.warehouse_id = NEW.warehouse_id
          AND ws.catalog_item_id = sr.catalog_item_id;

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

-- 2. Lançamento automático de NF no estoque (fn_auto_post_invoice omitido por complexidade — ver migration original)

-- 1. Motor de estoque (fn_process_stock_movement omitido por complexidade — ver migration original)

-- Remove trigger de updated_at adicionado na UP
DROP TRIGGER IF EXISTS set_timestamp_requisition_items ON requisition_items;
ALTER TABLE requisition_items DROP COLUMN IF EXISTS updated_at;

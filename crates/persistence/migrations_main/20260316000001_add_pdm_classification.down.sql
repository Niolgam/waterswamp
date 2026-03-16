-- Reverter: remover classificação do PDM e restaurar trigger original

DROP INDEX IF EXISTS idx_catmat_pdms_stockable;
DROP INDEX IF EXISTS idx_catmat_pdms_permanent;

ALTER TABLE catmat_pdms
    DROP CONSTRAINT IF EXISTS ck_pdm_not_both_stockable_permanent;

ALTER TABLE catmat_pdms
    DROP COLUMN IF EXISTS is_stockable,
    DROP COLUMN IF EXISTS is_permanent;

-- Restaurar fn_auto_post_invoice ao comportamento original (sem verificação de PDM)
CREATE OR REPLACE FUNCTION fn_auto_post_invoice()
RETURNS TRIGGER AS $$
DECLARE
    v_item RECORD;
BEGIN
    IF NEW.status = 'POSTED' AND (OLD.status IS NULL OR OLD.status <> 'POSTED') THEN

        IF NEW.posted_by IS NULL THEN
            RAISE EXCEPTION 'É obrigatório informar o usuário responsável pelo lançamento (posted_by)';
        END IF;

        FOR v_item IN
            SELECT
                ii.id,
                ii.catalog_item_id,
                ii.unit_raw_id,
                ii.unit_conversion_id,
                ii.quantity_raw,
                ii.conversion_factor,
                ii.quantity_base,
                ii.unit_value_base,
                ii.total_value,
                ii.batch_number,
                ii.expiration_date
            FROM invoice_items ii
            WHERE ii.invoice_id = NEW.id
        LOOP
            INSERT INTO stock_movements (
                warehouse_id, catalog_item_id, movement_type,
                unit_raw_id, unit_conversion_id,
                quantity_raw, conversion_factor, quantity_base,
                unit_price_base, total_value,
                invoice_id, invoice_item_id, document_number, user_id,
                batch_number, expiration_date
            ) VALUES (
                NEW.warehouse_id, v_item.catalog_item_id, 'ENTRY',
                v_item.unit_raw_id, v_item.unit_conversion_id,
                v_item.quantity_raw, v_item.conversion_factor, v_item.quantity_base,
                v_item.unit_value_base, v_item.total_value,
                NEW.id, v_item.id, NEW.invoice_number, NEW.posted_by,
                v_item.batch_number, v_item.expiration_date
            );
        END LOOP;

    ELSIF OLD.status = 'POSTED' AND NEW.status <> 'POSTED' THEN

        INSERT INTO stock_movements (
            warehouse_id, catalog_item_id, movement_type,
            unit_raw_id, unit_conversion_id,
            quantity_raw, conversion_factor, quantity_base,
            unit_price_base, total_value,
            invoice_id, invoice_item_id,
            document_number, user_id,
            batch_number, expiration_date, notes
        )
        SELECT
            sm.warehouse_id, sm.catalog_item_id, 'ADJUSTMENT_SUB',
            sm.unit_raw_id, sm.unit_conversion_id,
            sm.quantity_raw, sm.conversion_factor, sm.quantity_base,
            sm.unit_price_base, sm.total_value,
            sm.invoice_id, sm.invoice_item_id,
            'ESTORNO NF ' || NEW.invoice_number,
            COALESCE(NEW.posted_by, OLD.posted_by),
            sm.batch_number, sm.expiration_date,
            'Estorno automático - NF revertida de POSTED para ' || NEW.status::TEXT
        FROM stock_movements sm
        WHERE sm.invoice_id = NEW.id
          AND sm.movement_type = 'ENTRY';

    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

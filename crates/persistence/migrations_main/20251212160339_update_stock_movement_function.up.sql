-- ============================================================================
-- Migration: Update Stock Movement Function to Use System Settings
-- Description: Updates fn_process_stock_movement to fetch configuration from system_settings
-- ============================================================================

CREATE OR REPLACE FUNCTION fn_process_stock_movement()
RETURNS TRIGGER AS $$
DECLARE
    v_curr_qty DECIMAL(15, 3) := 0;
    v_curr_avg DECIMAL(15, 4) := 0;
    v_new_qty DECIMAL(15, 3);
    v_new_avg DECIMAL(15, 4);
    v_is_stockable BOOLEAN;
    v_is_blocked BOOLEAN;
    v_price_diff_percent DECIMAL(15, 4);
    v_threshold DECIMAL;
BEGIN
    -- 1. VERIFICATION: Is item stockable?
    SELECT is_stockable INTO v_is_stockable
    FROM catalog_items
    WHERE id = NEW.catalog_item_id;

    -- If not stockable (Service), doesn't affect physical balance
    IF v_is_stockable = FALSE THEN
        NEW.balance_before := 0;
        NEW.balance_after := 0;
        NEW.average_before := 0;
        NEW.average_after := 0;
        RETURN NEW;
    END IF;

    -- 2. CAPTURE CURRENT STATE (with lock to avoid race condition)
    SELECT quantity, average_unit_value, is_blocked
    INTO v_curr_qty, v_curr_avg, v_is_blocked
    FROM warehouse_stocks
    WHERE warehouse_id = NEW.warehouse_id AND catalog_item_id = NEW.catalog_item_id
    FOR UPDATE;

    -- If it's the first record of item in warehouse
    IF NOT FOUND THEN
        v_curr_qty := 0;
        v_curr_avg := 0;
        v_is_blocked := FALSE;
    END IF;

    -- 3. BLOCK VERIFICATION
    -- Entries are allowed even with blocked item (for return/correction)
    IF v_is_blocked = TRUE
       AND NEW.movement_type NOT IN ('ENTRY', 'RETURN', 'TRANSFER_IN', 'ADJUSTMENT_ADD', 'DONATION_IN') THEN
        RAISE EXCEPTION 'Operação negada: O item está BLOQUEADO neste almoxarifado. Motivo: %',
            (SELECT block_reason FROM warehouse_stocks
             WHERE warehouse_id = NEW.warehouse_id AND catalog_item_id = NEW.catalog_item_id);
    END IF;

    -- 4. SNAPSHOT FOR AUDIT (BEFORE)
    NEW.balance_before := v_curr_qty;
    NEW.average_before := v_curr_avg;

    -- 5. PRICE VALIDATION (for ad-hoc entries with dynamic limit)
    IF NEW.movement_type IN ('ADJUSTMENT_ADD', 'DONATION_IN') AND v_curr_avg > 0 AND NEW.unit_price_base > 0 THEN

        -- Fetch global price divergence configuration FROM SYSTEM_SETTINGS
        SELECT (value::TEXT)::DECIMAL INTO v_threshold
        FROM system_settings
        WHERE key = 'inventory.price_divergence_threshold';

        -- Fallback if key doesn't exist (default 20%)
        IF v_threshold IS NULL THEN
            v_threshold := 0.20;
        END IF;

        v_price_diff_percent := ABS((NEW.unit_price_base - v_curr_avg) / v_curr_avg);

        IF v_price_diff_percent > v_threshold THEN
            NEW.requires_review := TRUE;
            IF NEW.divergence_justification IS NULL OR NEW.divergence_justification = '' THEN
                RAISE EXCEPTION 'Variação de preço > %%% em relação ao custo médio. Informe uma justificativa.', (v_threshold * 100);
            END IF;
        END IF;
    END IF;

    -- 6. NEW BALANCE CALCULATION

    -- Group A: ENTRY movements (increment quantity)
    IF NEW.movement_type IN ('ENTRY', 'RETURN', 'TRANSFER_IN', 'ADJUSTMENT_ADD', 'DONATION_IN') THEN
        v_new_qty := v_curr_qty + NEW.quantity_base;

        -- Weighted average cost calculation
        IF v_new_qty > 0 AND NEW.unit_price_base > 0 THEN
            v_new_avg := ((v_curr_qty * v_curr_avg) + (NEW.quantity_base * NEW.unit_price_base)) / v_new_qty;
        ELSIF v_new_qty > 0 THEN
            v_new_avg := v_curr_avg; -- Keep average if entry without value
        ELSE
            v_new_avg := NEW.unit_price_base;
        END IF;

    -- Group B: EXIT movements (decrement quantity)
    ELSE
        -- Sufficient balance validation
        IF NEW.quantity_base > v_curr_qty THEN
            RAISE EXCEPTION 'Saldo insuficiente. Disponível: %, Solicitado: %', v_curr_qty, NEW.quantity_base;
        END IF;

        v_new_qty := v_curr_qty - NEW.quantity_base;
        v_new_avg := v_curr_avg; -- Exits don't change average cost

        -- Force exit price as current average price (accounting)
        NEW.unit_price_base := v_curr_avg;
        NEW.total_value := NEW.quantity_base * v_curr_avg;
    END IF;

    -- 7. SNAPSHOT FOR AUDIT (AFTER)
    NEW.balance_after := v_new_qty;
    NEW.average_after := v_new_avg;

    -- 8. UPSERT IN BALANCE TABLE (warehouse_stocks)
    INSERT INTO warehouse_stocks (
        warehouse_id,
        catalog_item_id,
        quantity,
        average_unit_value,
        last_entry_at,
        last_exit_at,
        updated_at
    )
    VALUES (
        NEW.warehouse_id,
        NEW.catalog_item_id,
        v_new_qty,
        v_new_avg,
        CASE WHEN NEW.movement_type IN ('ENTRY', 'RETURN', 'TRANSFER_IN', 'ADJUSTMENT_ADD', 'DONATION_IN')
             THEN NOW() ELSE NULL END,
        CASE WHEN NEW.movement_type IN ('EXIT', 'LOSS', 'TRANSFER_OUT', 'ADJUSTMENT_SUB', 'DONATION_OUT')
             THEN NOW() ELSE NULL END,
        NOW()
    )
    ON CONFLICT (warehouse_id, catalog_item_id)
    DO UPDATE SET
        quantity = EXCLUDED.quantity,
        average_unit_value = EXCLUDED.average_unit_value,
        last_entry_at = COALESCE(EXCLUDED.last_entry_at, warehouse_stocks.last_entry_at),
        last_exit_at = COALESCE(EXCLUDED.last_exit_at, warehouse_stocks.last_exit_at),
        updated_at = NOW();

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION fn_process_stock_movement() IS 'Processa movimentações de estoque com integração ao system_settings para configurações dinâmicas';

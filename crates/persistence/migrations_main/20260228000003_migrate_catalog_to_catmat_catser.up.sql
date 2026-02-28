-- ============================================================================
-- Migration: Migrar tabelas dependentes de catalog_items para CATMAT/CATSER
-- e remover catalog_items + catalog_groups
-- ============================================================================

-- ============================================================================
-- 1. WAREHOUSE_STOCKS: catalog_item_id → catmat_item_id (só materiais)
-- ============================================================================

ALTER TABLE warehouse_stocks
    ADD COLUMN catmat_item_id UUID REFERENCES catmat_items(id) ON DELETE RESTRICT;

-- Migrar dados existentes (se houver)
UPDATE warehouse_stocks ws
SET catmat_item_id = (
    SELECT ci2.id FROM catmat_items ci2
    WHERE ci2.code = ci.catmat_code
)
FROM catalog_items ci
WHERE ws.catalog_item_id = ci.id
  AND ci.catmat_code IS NOT NULL;

-- Remover coluna antiga e constraints
ALTER TABLE warehouse_stocks DROP CONSTRAINT IF EXISTS uq_warehouse_stocks_item;

-- Dropar índices antigos de warehouse_stocks
DROP INDEX IF EXISTS idx_warehouse_stocks_catalog_item;
DROP INDEX IF EXISTS idx_warehouse_stocks_available;

ALTER TABLE warehouse_stocks DROP COLUMN catalog_item_id;

-- Adicionar constraint NOT NULL (após migração de dados)
-- ALTER TABLE warehouse_stocks ALTER COLUMN catmat_item_id SET NOT NULL;

-- Recriar constraints e índices
ALTER TABLE warehouse_stocks ADD CONSTRAINT uq_warehouse_stocks_item UNIQUE (warehouse_id, catmat_item_id);
CREATE INDEX idx_warehouse_stocks_catmat_item ON warehouse_stocks(catmat_item_id);
CREATE INDEX idx_warehouse_stocks_available ON warehouse_stocks(warehouse_id, catmat_item_id)
    WHERE (quantity - reserved_quantity) > 0;

-- ============================================================================
-- 2. STOCK_MOVEMENTS: catalog_item_id → catmat_item_id (só materiais)
-- ============================================================================

ALTER TABLE stock_movements
    ADD COLUMN catmat_item_id UUID REFERENCES catmat_items(id) ON DELETE RESTRICT;

UPDATE stock_movements sm
SET catmat_item_id = (
    SELECT ci2.id FROM catmat_items ci2
    WHERE ci2.code = ci.catmat_code
)
FROM catalog_items ci
WHERE sm.catalog_item_id = ci.id
  AND ci.catmat_code IS NOT NULL;

DROP INDEX IF EXISTS idx_stock_movements_catalog_item;
DROP INDEX IF EXISTS idx_stock_movements_audit;
DROP INDEX IF EXISTS idx_stock_movements_item_flow;

ALTER TABLE stock_movements DROP COLUMN catalog_item_id;

CREATE INDEX idx_stock_movements_catmat_item ON stock_movements(catmat_item_id);
CREATE INDEX idx_stock_movements_audit ON stock_movements(warehouse_id, catmat_item_id, movement_date DESC);
CREATE INDEX idx_stock_movements_item_flow ON stock_movements(catmat_item_id, movement_date DESC);

-- ============================================================================
-- 3. STOCK_RESERVATIONS: catalog_item_id → catmat_item_id (só materiais)
-- ============================================================================

ALTER TABLE stock_reservations
    ADD COLUMN catmat_item_id UUID REFERENCES catmat_items(id) ON DELETE RESTRICT;

UPDATE stock_reservations sr
SET catmat_item_id = (
    SELECT ci2.id FROM catmat_items ci2
    WHERE ci2.code = ci.catmat_code
)
FROM catalog_items ci
WHERE sr.catalog_item_id = ci.id
  AND ci.catmat_code IS NOT NULL;

DROP INDEX IF EXISTS idx_stock_reservations_catalog_item;
DROP INDEX IF EXISTS idx_stock_reservations_active;

ALTER TABLE stock_reservations DROP COLUMN catalog_item_id;

CREATE INDEX idx_stock_reservations_catmat_item ON stock_reservations(catmat_item_id);
CREATE INDEX idx_stock_reservations_active ON stock_reservations(warehouse_id, catmat_item_id)
    WHERE is_active = TRUE;

-- ============================================================================
-- 4. INVOICE_ITEMS: catalog_item_id → catmat_item_id / catser_item_id
-- ============================================================================

ALTER TABLE invoice_items
    ADD COLUMN catmat_item_id UUID REFERENCES catmat_items(id) ON DELETE RESTRICT,
    ADD COLUMN catser_item_id UUID REFERENCES catser_items(id) ON DELETE RESTRICT;

-- Migrar dados existentes
UPDATE invoice_items ii
SET catmat_item_id = (
    SELECT ci2.id FROM catmat_items ci2
    WHERE ci2.code = ci.catmat_code
)
FROM catalog_items ci
WHERE ii.catalog_item_id = ci.id
  AND ci.catmat_code IS NOT NULL
  AND ci.is_stockable = TRUE;

UPDATE invoice_items ii
SET catser_item_id = (
    SELECT cs.id FROM catser_items cs
    WHERE cs.code = ci.catmat_code
)
FROM catalog_items ci
WHERE ii.catalog_item_id = ci.id
  AND ci.catmat_code IS NOT NULL
  AND ci.is_stockable = FALSE;

DROP INDEX IF EXISTS idx_invoice_items_catalog_item;

ALTER TABLE invoice_items DROP COLUMN catalog_item_id;

-- Constraint: exatamente um dos dois FKs preenchido
ALTER TABLE invoice_items ADD CONSTRAINT ck_invoice_items_catalog_ref CHECK (
    (catmat_item_id IS NOT NULL AND catser_item_id IS NULL) OR
    (catmat_item_id IS NULL AND catser_item_id IS NOT NULL)
);

CREATE INDEX idx_invoice_items_catmat ON invoice_items(catmat_item_id) WHERE catmat_item_id IS NOT NULL;
CREATE INDEX idx_invoice_items_catser ON invoice_items(catser_item_id) WHERE catser_item_id IS NOT NULL;

-- ============================================================================
-- 5. REQUISITION_ITEMS: catalog_item_id → catmat_item_id / catser_item_id
-- ============================================================================

ALTER TABLE requisition_items
    ADD COLUMN catmat_item_id UUID REFERENCES catmat_items(id) ON DELETE RESTRICT,
    ADD COLUMN catser_item_id UUID REFERENCES catser_items(id) ON DELETE RESTRICT;

UPDATE requisition_items ri
SET catmat_item_id = (
    SELECT ci2.id FROM catmat_items ci2
    WHERE ci2.code = ci.catmat_code
)
FROM catalog_items ci
WHERE ri.catalog_item_id = ci.id
  AND ci.catmat_code IS NOT NULL
  AND ci.is_stockable = TRUE;

UPDATE requisition_items ri
SET catser_item_id = (
    SELECT cs.id FROM catser_items cs
    WHERE cs.code = ci.catmat_code
)
FROM catalog_items ci
WHERE ri.catalog_item_id = ci.id
  AND ci.catmat_code IS NOT NULL
  AND ci.is_stockable = FALSE;

DROP INDEX IF EXISTS idx_requisition_items_catalog_item;

ALTER TABLE requisition_items DROP CONSTRAINT IF EXISTS uq_requisition_items_item;
ALTER TABLE requisition_items DROP COLUMN catalog_item_id;

ALTER TABLE requisition_items ADD CONSTRAINT ck_requisition_items_catalog_ref CHECK (
    (catmat_item_id IS NOT NULL AND catser_item_id IS NULL) OR
    (catmat_item_id IS NULL AND catser_item_id IS NOT NULL)
);

CREATE INDEX idx_requisition_items_catmat ON requisition_items(catmat_item_id) WHERE catmat_item_id IS NOT NULL;
CREATE INDEX idx_requisition_items_catser ON requisition_items(catser_item_id) WHERE catser_item_id IS NOT NULL;

-- ============================================================================
-- 6. ATUALIZAR FUNÇÕES SQL QUE REFERENCIAM catalog_items
-- ============================================================================

-- 6a. fn_process_stock_movement: consulta catmat_items em vez de catalog_items
CREATE OR REPLACE FUNCTION fn_process_stock_movement()
RETURNS TRIGGER AS $$
DECLARE
    v_curr_qty DECIMAL(15, 3) := 0;
    v_curr_avg DECIMAL(15, 4) := 0;
    v_new_qty DECIMAL(15, 3);
    v_new_avg DECIMAL(15, 4);
    v_is_blocked BOOLEAN;
    v_price_diff_percent DECIMAL(15, 4);
    v_threshold DECIMAL;
BEGIN
    -- Materiais sempre são estocáveis, não precisa verificar is_stockable

    -- 2. CAPTURA DO ESTADO ATUAL (com lock)
    SELECT quantity, average_unit_value, is_blocked
    INTO v_curr_qty, v_curr_avg, v_is_blocked
    FROM warehouse_stocks
    WHERE warehouse_id = NEW.warehouse_id AND catmat_item_id = NEW.catmat_item_id
    FOR UPDATE;

    IF NOT FOUND THEN
        v_curr_qty := 0;
        v_curr_avg := 0;
        v_is_blocked := FALSE;
    END IF;

    -- 3. VERIFICAÇÃO DE BLOQUEIO
    IF v_is_blocked = TRUE
       AND NEW.movement_type NOT IN ('ENTRY', 'RETURN', 'TRANSFER_IN', 'ADJUSTMENT_ADD', 'DONATION_IN') THEN
        RAISE EXCEPTION 'Operação negada: O item está BLOQUEADO neste almoxarifado. Motivo: %',
            (SELECT block_reason FROM warehouse_stocks
             WHERE warehouse_id = NEW.warehouse_id AND catmat_item_id = NEW.catmat_item_id);
    END IF;

    -- 4. SNAPSHOT (ANTES)
    NEW.balance_before := v_curr_qty;
    NEW.average_before := v_curr_avg;

    -- 5. VALIDAÇÃO DE PREÇO
    IF NEW.movement_type IN ('ADJUSTMENT_ADD', 'DONATION_IN') AND v_curr_avg > 0 AND NEW.unit_price_base > 0 THEN
        SELECT (value::TEXT)::DECIMAL INTO v_threshold
        FROM system_settings
        WHERE key = 'inventory.price_divergence_threshold';

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

    -- 6. CÁLCULO DE SALDOS
    IF NEW.movement_type IN ('ENTRY', 'RETURN', 'TRANSFER_IN', 'ADJUSTMENT_ADD', 'DONATION_IN') THEN
        v_new_qty := v_curr_qty + NEW.quantity_base;

        IF v_new_qty > 0 AND NEW.unit_price_base > 0 THEN
            v_new_avg := ((v_curr_qty * v_curr_avg) + (NEW.quantity_base * NEW.unit_price_base)) / v_new_qty;
        ELSIF v_new_qty > 0 THEN
            v_new_avg := v_curr_avg;
        ELSE
            v_new_avg := NEW.unit_price_base;
        END IF;
    ELSE
        IF NEW.quantity_base > v_curr_qty THEN
            RAISE EXCEPTION 'Saldo insuficiente. Disponível: %, Solicitado: %', v_curr_qty, NEW.quantity_base;
        END IF;

        v_new_qty := v_curr_qty - NEW.quantity_base;
        v_new_avg := v_curr_avg;

        NEW.unit_price_base := v_curr_avg;
        NEW.total_value := NEW.quantity_base * v_curr_avg;
    END IF;

    -- 7. SNAPSHOT (DEPOIS)
    NEW.balance_after := v_new_qty;
    NEW.average_after := v_new_avg;

    -- 8. UPSERT NA warehouse_stocks
    INSERT INTO warehouse_stocks (
        warehouse_id,
        catmat_item_id,
        quantity,
        average_unit_value,
        last_entry_at,
        last_exit_at,
        updated_at
    )
    VALUES (
        NEW.warehouse_id,
        NEW.catmat_item_id,
        v_new_qty,
        v_new_avg,
        CASE WHEN NEW.movement_type IN ('ENTRY', 'RETURN', 'TRANSFER_IN', 'ADJUSTMENT_ADD', 'DONATION_IN')
             THEN NOW() ELSE NULL END,
        CASE WHEN NEW.movement_type IN ('EXIT', 'LOSS', 'TRANSFER_OUT', 'ADJUSTMENT_SUB', 'DONATION_OUT')
             THEN NOW() ELSE NULL END,
        NOW()
    )
    ON CONFLICT (warehouse_id, catmat_item_id)
    DO UPDATE SET
        quantity = EXCLUDED.quantity,
        average_unit_value = EXCLUDED.average_unit_value,
        last_entry_at = COALESCE(EXCLUDED.last_entry_at, warehouse_stocks.last_entry_at),
        last_exit_at = COALESCE(EXCLUDED.last_exit_at, warehouse_stocks.last_exit_at),
        updated_at = NOW();

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 6b. fn_get_available_quantity: atualizar para catmat_item_id
CREATE OR REPLACE FUNCTION fn_get_available_quantity(
    p_warehouse_id UUID,
    p_catmat_item_id UUID
) RETURNS DECIMAL(15, 3) AS $$
DECLARE
    v_available DECIMAL(15, 3);
BEGIN
    SELECT COALESCE(quantity - reserved_quantity, 0)
    INTO v_available
    FROM warehouse_stocks
    WHERE warehouse_id = p_warehouse_id
      AND catmat_item_id = p_catmat_item_id
      AND is_blocked = FALSE;

    RETURN COALESCE(v_available, 0);
END;
$$ LANGUAGE plpgsql STABLE;

-- 6c. fn_manage_stock_reservation: atualizar para catmat_item_id
CREATE OR REPLACE FUNCTION fn_manage_stock_reservation()
RETURNS TRIGGER AS $$
BEGIN
    IF (NEW.status = 'APPROVED' AND OLD.status = 'PENDING') THEN
        INSERT INTO stock_reservations (
            requisition_id,
            requisition_item_id,
            catmat_item_id,
            warehouse_id,
            quantity
        )
        SELECT
            NEW.id,
            ri.id,
            ri.catmat_item_id,
            NEW.warehouse_id,
            COALESCE(ri.approved_quantity, ri.requested_quantity)
        FROM requisition_items ri
        WHERE ri.requisition_id = NEW.id
          AND ri.catmat_item_id IS NOT NULL;

        UPDATE warehouse_stocks ws
        SET reserved_quantity = reserved_quantity + COALESCE(ri.approved_quantity, ri.requested_quantity),
            updated_at = NOW()
        FROM requisition_items ri
        WHERE ri.requisition_id = NEW.id
          AND ri.catmat_item_id IS NOT NULL
          AND ws.warehouse_id = NEW.warehouse_id
          AND ws.catmat_item_id = ri.catmat_item_id;

    ELSIF (NEW.status IN ('FULFILLED', 'PARTIALLY_FULFILLED', 'REJECTED', 'CANCELLED')
           AND OLD.status IN ('APPROVED', 'PROCESSING')) THEN

        UPDATE warehouse_stocks ws
        SET reserved_quantity = GREATEST(0, reserved_quantity - sr.quantity),
            updated_at = NOW()
        FROM stock_reservations sr
        WHERE sr.requisition_id = NEW.id
          AND sr.is_active = TRUE
          AND ws.warehouse_id = NEW.warehouse_id
          AND ws.catmat_item_id = sr.catmat_item_id;

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

-- 6d. fn_capture_requisition_item_value: atualizar para catmat_item_id
CREATE OR REPLACE FUNCTION fn_capture_requisition_item_value()
RETURNS TRIGGER AS $$
DECLARE
    v_avg_value DECIMAL(15, 4);
BEGIN
    -- Tentar buscar valor do estoque (só para materiais)
    IF NEW.catmat_item_id IS NOT NULL THEN
        SELECT COALESCE(ws.average_unit_value, ci.estimated_value, 0)
        INTO v_avg_value
        FROM catmat_items ci
        LEFT JOIN warehouse_stocks ws ON ws.catmat_item_id = ci.id
            AND ws.warehouse_id = (SELECT warehouse_id FROM requisitions WHERE id = NEW.requisition_id)
        WHERE ci.id = NEW.catmat_item_id;
    ELSE
        -- Para serviços, buscar estimated_value do catser_items
        SELECT COALESCE(cs.estimated_value, 0)
        INTO v_avg_value
        FROM catser_items cs
        WHERE cs.id = NEW.catser_item_id;
    END IF;

    NEW.unit_value := COALESCE(v_avg_value, 0);
    NEW.total_value := NEW.requested_quantity * COALESCE(v_avg_value, 0);

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 6e. fn_requisition_item_audit: atualizar referência de catalog_items
CREATE OR REPLACE FUNCTION fn_requisition_item_audit()
RETURNS TRIGGER AS $$
DECLARE
    v_user_id UUID;
    v_operation audit_operation_enum;
    v_data_before JSONB;
    v_data_after JSONB;
    v_catalog_name VARCHAR(200);
    v_quantity_before DECIMAL(15, 3);
    v_quantity_after DECIMAL(15, 3);
    v_reason TEXT;
    v_item_id UUID;
BEGIN
    v_user_id := fn_get_audit_user_id();

    -- Buscar nome do item (CATMAT ou CATSER)
    v_item_id := COALESCE(NEW.catmat_item_id, OLD.catmat_item_id);
    IF v_item_id IS NOT NULL THEN
        SELECT description INTO v_catalog_name FROM catmat_items WHERE id = v_item_id;
    ELSE
        v_item_id := COALESCE(NEW.catser_item_id, OLD.catser_item_id);
        IF v_item_id IS NOT NULL THEN
            SELECT description INTO v_catalog_name FROM catser_items WHERE id = v_item_id;
        END IF;
    END IF;

    IF TG_OP = 'INSERT' THEN
        v_operation := 'INSERT';
        v_data_after := to_jsonb(NEW);
        v_quantity_after := NEW.requested_quantity;

    ELSIF TG_OP = 'UPDATE' THEN
        v_data_before := to_jsonb(OLD);
        v_data_after := to_jsonb(NEW);
        v_quantity_before := OLD.requested_quantity;
        v_quantity_after := NEW.requested_quantity;

        IF OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL THEN
            v_operation := 'SOFT_DELETE';
            v_reason := NEW.deletion_reason;
        ELSIF OLD.deleted_at IS NOT NULL AND NEW.deleted_at IS NULL THEN
            v_operation := 'RESTORE';
        ELSE
            v_operation := 'UPDATE';
        END IF;

    ELSIF TG_OP = 'DELETE' THEN
        v_operation := 'DELETE';
        v_data_before := to_jsonb(OLD);
        v_quantity_before := OLD.requested_quantity;
        v_reason := current_setting('audit.delete_reason', TRUE);
    END IF;

    INSERT INTO requisition_item_history (
        requisition_item_id,
        requisition_id,
        catalog_item_id,
        catalog_item_name,
        operation,
        data_before,
        data_after,
        changed_fields,
        changes_diff,
        quantity_before,
        quantity_after,
        performed_by,
        ip_address,
        reason,
        transaction_id
    ) VALUES (
        COALESCE(NEW.id, OLD.id),
        COALESCE(NEW.requisition_id, OLD.requisition_id),
        v_item_id,
        v_catalog_name,
        v_operation,
        v_data_before,
        v_data_after,
        CASE WHEN v_data_before IS NOT NULL AND v_data_after IS NOT NULL
             THEN fn_get_changed_fields(v_data_before, v_data_after)
             ELSE NULL END,
        CASE WHEN v_data_before IS NOT NULL AND v_data_after IS NOT NULL
             THEN fn_generate_diff(v_data_before, v_data_after)
             ELSE NULL END,
        v_quantity_before,
        v_quantity_after,
        COALESCE(v_user_id, '00000000-0000-0000-0000-000000000000'::UUID),
        fn_get_audit_ip(),
        v_reason,
        current_setting('audit.transaction_id', TRUE)::UUID
    );

    RETURN COALESCE(NEW, OLD);
EXCEPTION
    WHEN invalid_text_representation THEN
        INSERT INTO requisition_item_history (
            requisition_item_id,
            requisition_id,
            catalog_item_id,
            catalog_item_name,
            operation,
            data_before,
            data_after,
            changed_fields,
            changes_diff,
            quantity_before,
            quantity_after,
            performed_by,
            ip_address,
            reason
        ) VALUES (
            COALESCE(NEW.id, OLD.id),
            COALESCE(NEW.requisition_id, OLD.requisition_id),
            v_item_id,
            v_catalog_name,
            v_operation,
            v_data_before,
            v_data_after,
            CASE WHEN v_data_before IS NOT NULL AND v_data_after IS NOT NULL
                 THEN fn_get_changed_fields(v_data_before, v_data_after)
                 ELSE NULL END,
            CASE WHEN v_data_before IS NOT NULL AND v_data_after IS NOT NULL
                 THEN fn_generate_diff(v_data_before, v_data_after)
                 ELSE NULL END,
            v_quantity_before,
            v_quantity_after,
            COALESCE(v_user_id, '00000000-0000-0000-0000-000000000000'::UUID),
            fn_get_audit_ip(),
            v_reason
        );
        RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- 6f. fn_invoice_item_audit: atualizar referência
CREATE OR REPLACE FUNCTION fn_invoice_item_audit()
RETURNS TRIGGER AS $$
DECLARE
    v_user_id UUID;
    v_operation audit_operation_enum;
    v_catalog_name VARCHAR(200);
    v_item_id UUID;
BEGIN
    v_user_id := fn_get_audit_user_id();

    -- Buscar nome do item (CATMAT ou CATSER)
    v_item_id := COALESCE(NEW.catmat_item_id, OLD.catmat_item_id);
    IF v_item_id IS NOT NULL THEN
        SELECT description INTO v_catalog_name FROM catmat_items WHERE id = v_item_id;
    ELSE
        v_item_id := COALESCE(NEW.catser_item_id, OLD.catser_item_id);
        IF v_item_id IS NOT NULL THEN
            SELECT description INTO v_catalog_name FROM catser_items WHERE id = v_item_id;
        END IF;
    END IF;

    IF TG_OP = 'INSERT' THEN
        v_operation := 'INSERT';
    ELSIF TG_OP = 'UPDATE' THEN
        v_operation := 'UPDATE';
    ELSIF TG_OP = 'DELETE' THEN
        v_operation := 'DELETE';
    END IF;

    INSERT INTO invoice_item_history (
        invoice_item_id,
        invoice_id,
        catalog_item_id,
        catalog_item_name,
        operation,
        data_before,
        data_after,
        changed_fields,
        quantity_before,
        quantity_after,
        value_before,
        value_after,
        performed_by,
        ip_address
    ) VALUES (
        COALESCE(NEW.id, OLD.id),
        COALESCE(NEW.invoice_id, OLD.invoice_id),
        v_item_id,
        v_catalog_name,
        v_operation,
        CASE WHEN OLD IS NOT NULL THEN to_jsonb(OLD) ELSE NULL END,
        CASE WHEN NEW IS NOT NULL THEN to_jsonb(NEW) ELSE NULL END,
        CASE WHEN OLD IS NOT NULL AND NEW IS NOT NULL
             THEN fn_get_changed_fields(to_jsonb(OLD), to_jsonb(NEW))
             ELSE NULL END,
        OLD.quantity_raw,
        NEW.quantity_raw,
        OLD.unit_value_raw,
        NEW.unit_value_raw,
        COALESCE(v_user_id, '00000000-0000-0000-0000-000000000000'::UUID),
        fn_get_audit_ip()
    );

    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 7. REMOVER TABELAS ANTIGAS
-- ============================================================================

-- Dropar triggers que referenciam catalog_items/catalog_groups
DROP TRIGGER IF EXISTS trg_catalog_item_leaf_only ON catalog_items;
DROP TRIGGER IF EXISTS set_timestamp_catalog_items ON catalog_items;
DROP TRIGGER IF EXISTS trg_catalog_group_hierarchy_consistency ON catalog_groups;
DROP TRIGGER IF EXISTS trg_catalog_group_leaf_safety ON catalog_groups;
DROP TRIGGER IF EXISTS set_timestamp_catalog_groups ON catalog_groups;

-- Dropar funções antigas
DROP FUNCTION IF EXISTS fn_prevent_item_in_synthetic_group();
DROP FUNCTION IF EXISTS fn_validate_catalog_group_hierarchy();
DROP FUNCTION IF EXISTS fn_prevent_subgroup_if_items_exist();

-- Dropar tabelas na ordem correta (itens antes de grupos)
DROP TABLE IF EXISTS catalog_items CASCADE;
DROP TABLE IF EXISTS catalog_groups CASCADE;

-- Dropar enum que não é mais necessário
DROP TYPE IF EXISTS item_type_enum;

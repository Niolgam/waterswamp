CREATE TYPE stock_movement_type_enum AS ENUM (
    'ENTRY',            -- Entrada por NF
    'EXIT',             -- Saída por requisição
    'LOSS',             -- Perda (quebra, validade, extravio)
    'RETURN',           -- Devolução de setor para o almoxarifado
    'TRANSFER_IN',      -- Entrada via transferência entre almoxarifados
    'TRANSFER_OUT',     -- Saída via transferência entre almoxarifados
    'ADJUSTMENT_ADD',   -- Ajuste de inventário (entrada/sobra)
    'ADJUSTMENT_SUB',   -- Ajuste de inventário (saída/falta)
    'DONATION_IN',      -- Entrada por doação
    'DONATION_OUT'      -- Saída por doação
);

CREATE TABLE stock_movements (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    warehouse_id UUID NOT NULL REFERENCES warehouses(id) ON DELETE RESTRICT,
    catalog_item_id UUID NOT NULL REFERENCES catalog_items(id) ON DELETE RESTRICT,
    movement_type stock_movement_type_enum NOT NULL,
    movement_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Unidade original (como veio no documento)
    unit_raw_id UUID NOT NULL REFERENCES units_of_measure(id) ON DELETE RESTRICT,
    unit_conversion_id UUID REFERENCES unit_conversions(id) ON DELETE SET NULL,
    
    quantity_raw DECIMAL(15, 4) NOT NULL, -- Quantidade no documento original
    conversion_factor DECIMAL(15, 4) NOT NULL DEFAULT 1.0 CHECK (conversion_factor > 0),
    quantity_base DECIMAL(15, 3) NOT NULL, -- Quantidade convertida para unidade base
    unit_price_base DECIMAL(15, 4) NOT NULL DEFAULT 0.00 CHECK (unit_price_base >= 0),
    total_value DECIMAL(15, 2) NOT NULL DEFAULT 0.00,

    -- Saldos antes e depois (Trilha de Auditoria completa)
    balance_before DECIMAL(15, 3) NOT NULL,
    balance_after DECIMAL(15, 3) NOT NULL,
    average_before DECIMAL(15, 4) NOT NULL,
    average_after DECIMAL(15, 4) NOT NULL,

    -- Documentos de origem
    invoice_id UUID REFERENCES invoices(id) ON DELETE SET NULL,
    invoice_item_id UUID REFERENCES invoice_items(id) ON DELETE SET NULL,
    requisition_id UUID REFERENCES requisitions(id) ON DELETE SET NULL,
    requisition_item_id UUID REFERENCES requisition_items(id) ON DELETE SET NULL,
    
    -- Para transferências: almoxarifado de origem/destino
    related_warehouse_id UUID REFERENCES warehouses(id) ON DELETE SET NULL,
    related_movement_id UUID REFERENCES stock_movements(id) ON DELETE SET NULL,
    
    -- Documentação adicional
    document_number VARCHAR(100), -- Número de documento externo (OS, termo, etc)
    notes TEXT,
    attachment_url TEXT,

    -- Controle de qualidade
    divergence_justification TEXT, -- Se preço divergiu muito do médio
    requires_review BOOLEAN NOT NULL DEFAULT FALSE, -- Flag para auditoria
    reviewed_at TIMESTAMPTZ,
    reviewed_by UUID REFERENCES users(id) ON DELETE RESTRICT,

    -- Responsável pela movimentação
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT, 
    
    -- Lote (para itens com controle)
    batch_number VARCHAR(50),
    expiration_date DATE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT ck_stock_movements_quantity_sign CHECK (
        (movement_type IN ('ENTRY', 'RETURN', 'TRANSFER_IN', 'ADJUSTMENT_ADD', 'DONATION_IN') 
         AND quantity_base > 0) OR
        (movement_type IN ('EXIT', 'LOSS', 'TRANSFER_OUT', 'ADJUSTMENT_SUB', 'DONATION_OUT') 
         AND quantity_base > 0)
    ),
    CONSTRAINT ck_stock_movements_transfer CHECK (
        (movement_type IN ('TRANSFER_IN', 'TRANSFER_OUT') AND related_warehouse_id IS NOT NULL) OR
        (movement_type NOT IN ('TRANSFER_IN', 'TRANSFER_OUT'))
    )
);

CREATE INDEX idx_stock_movements_warehouse ON stock_movements(warehouse_id);
CREATE INDEX idx_stock_movements_catalog_item ON stock_movements(catalog_item_id);
CREATE INDEX idx_stock_movements_type ON stock_movements(movement_type);
CREATE INDEX idx_stock_movements_date ON stock_movements(movement_date DESC);
CREATE INDEX idx_stock_movements_audit ON stock_movements(warehouse_id, catalog_item_id, movement_date DESC);
CREATE INDEX idx_stock_movements_document ON stock_movements(document_number) 
    WHERE document_number IS NOT NULL;
CREATE INDEX idx_stock_movements_invoice ON stock_movements(invoice_id) 
    WHERE invoice_id IS NOT NULL;
CREATE INDEX idx_stock_movements_requisition ON stock_movements(requisition_id) 
    WHERE requisition_id IS NOT NULL;
CREATE INDEX idx_stock_movements_review ON stock_movements(warehouse_id, created_at)
    WHERE requires_review = TRUE AND reviewed_at IS NULL;
CREATE INDEX idx_stock_movements_item_flow ON stock_movements(catalog_item_id, movement_date DESC);
CREATE INDEX idx_stock_movements_batch ON stock_movements(batch_number)
    WHERE batch_number IS NOT NULL;

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
BEGIN
    -- 1. VERIFICAÇÃO: Item é estocável?
    SELECT is_stockable INTO v_is_stockable 
    FROM catalog_items 
    WHERE id = NEW.catalog_item_id;

    -- Se não for estocável (Serviço), não afeta saldo físico
    IF v_is_stockable = FALSE THEN
        NEW.balance_before := 0;
        NEW.balance_after := 0;
        NEW.average_before := 0;
        NEW.average_after := 0;
        RETURN NEW;
    END IF;

    -- 2. CAPTURA DO ESTADO ATUAL (com lock para evitar race condition)
    SELECT quantity, average_unit_value, is_blocked
    INTO v_curr_qty, v_curr_avg, v_is_blocked
    FROM warehouse_stocks
    WHERE warehouse_id = NEW.warehouse_id AND catalog_item_id = NEW.catalog_item_id
    FOR UPDATE;

    -- Se for o primeiro registro do item no almoxarifado
    IF NOT FOUND THEN
        v_curr_qty := 0;
        v_curr_avg := 0;
        v_is_blocked := FALSE;
    END IF;

    -- 3. VERIFICAÇÃO DE BLOQUEIO
    -- Entradas são permitidas mesmo com item bloqueado (para devolução/correção)
    IF v_is_blocked = TRUE 
       AND NEW.movement_type NOT IN ('ENTRY', 'RETURN', 'TRANSFER_IN', 'ADJUSTMENT_ADD', 'DONATION_IN') THEN
        RAISE EXCEPTION 'Operação negada: O item está BLOQUEADO neste almoxarifado. Motivo: %', 
            (SELECT block_reason FROM warehouse_stocks 
             WHERE warehouse_id = NEW.warehouse_id AND catalog_item_id = NEW.catalog_item_id);
    END IF;

    -- 4. SNAPSHOT PARA AUDITORIA (ANTES)
    NEW.balance_before := v_curr_qty;
    NEW.average_before := v_curr_avg;

    -- 5. VALIDAÇÃO DE PREÇO (para entradas avulsas)
    IF NEW.movement_type IN ('ADJUSTMENT_ADD', 'DONATION_IN') AND v_curr_avg > 0 AND NEW.unit_price_base > 0 THEN
        v_price_diff_percent := ABS((NEW.unit_price_base - v_curr_avg) / v_curr_avg);
        IF v_price_diff_percent > 0.20 THEN
            NEW.requires_review := TRUE;
            IF NEW.divergence_justification IS NULL OR NEW.divergence_justification = '' THEN
                RAISE EXCEPTION 'Variação de preço > 20%% em relação ao custo médio. Informe uma justificativa.';
            END IF;
        END IF;
    END IF;

    -- 6. CÁLCULO DE NOVOS SALDOS
    
    -- Grupo A: Movimentações de ENTRADA (incrementam quantidade)
    IF NEW.movement_type IN ('ENTRY', 'RETURN', 'TRANSFER_IN', 'ADJUSTMENT_ADD', 'DONATION_IN') THEN
        v_new_qty := v_curr_qty + NEW.quantity_base;
        
        -- Cálculo do custo médio ponderado
        IF v_new_qty > 0 AND NEW.unit_price_base > 0 THEN
            v_new_avg := ((v_curr_qty * v_curr_avg) + (NEW.quantity_base * NEW.unit_price_base)) / v_new_qty;
        ELSIF v_new_qty > 0 THEN
            v_new_avg := v_curr_avg; -- Mantém o médio se entrada sem valor
        ELSE
            v_new_avg := NEW.unit_price_base;
        END IF;

    -- Grupo B: Movimentações de SAÍDA (decrementam quantidade)
    ELSE
        -- Validação de saldo suficiente
        IF NEW.quantity_base > v_curr_qty THEN
            RAISE EXCEPTION 'Saldo insuficiente. Disponível: %, Solicitado: %', v_curr_qty, NEW.quantity_base;
        END IF;
        
        v_new_qty := v_curr_qty - NEW.quantity_base;
        v_new_avg := v_curr_avg; -- Saídas não alteram o custo médio
        
        -- Força o preço da saída como o preço médio atual (contabilidade)
        NEW.unit_price_base := v_curr_avg;
        NEW.total_value := NEW.quantity_base * v_curr_avg;
    END IF;

    -- 7. SNAPSHOT PARA AUDITORIA (DEPOIS)
    NEW.balance_after := v_new_qty;
    NEW.average_after := v_new_avg;

    -- 8. UPSERT NA TABELA DE SALDOS
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

CREATE TRIGGER trg_stock_movement_process
BEFORE INSERT ON stock_movements
FOR EACH ROW
EXECUTE FUNCTION fn_process_stock_movement();

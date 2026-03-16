-- =============================================================================
-- Adiciona classificação de material ao PDM (Padrão Descritivo de Material)
--
-- is_stockable: TRUE  → item entra no estoque ao ser lançado em NF
-- is_permanent: TRUE  → item é bem permanente (patrimônio) — não gera estoque
-- Ambos FALSE         → consumo/uso direto — sem impacto em estoque
--
-- Regra: is_stockable e is_permanent são mutuamente exclusivos.
-- =============================================================================

ALTER TABLE catmat_pdms
    ADD COLUMN is_stockable BOOLEAN NOT NULL DEFAULT TRUE,
    ADD COLUMN is_permanent BOOLEAN NOT NULL DEFAULT FALSE;

COMMENT ON COLUMN catmat_pdms.is_stockable IS
    'Se TRUE, itens deste PDM geram movimentação de ENTRY no estoque ao serem lançados em NF';

COMMENT ON COLUMN catmat_pdms.is_permanent IS
    'Se TRUE, itens são bens permanentes (patrimônio) — não entram no estoque; serão tratados pelo módulo de patrimônio';

ALTER TABLE catmat_pdms
    ADD CONSTRAINT ck_pdm_not_both_stockable_permanent
    CHECK (NOT (is_stockable = TRUE AND is_permanent = TRUE));

-- Índices para filtros comuns no módulo de catálogo
CREATE INDEX idx_catmat_pdms_stockable ON catmat_pdms(is_stockable) WHERE is_stockable = TRUE;
CREATE INDEX idx_catmat_pdms_permanent ON catmat_pdms(is_permanent) WHERE is_permanent = TRUE;

-- =============================================================================
-- Atualiza fn_auto_post_invoice para respeitar a classificação do PDM
--
-- ANTES: criava ENTRY para TODOS os itens da NF
-- DEPOIS: cria ENTRY apenas para itens cujo PDM tem is_stockable = TRUE
--         Itens permanentes (is_permanent) são ignorados pelo estoque (módulo futuro)
--         Itens de uso direto (ambos FALSE) também são ignorados
-- =============================================================================

CREATE OR REPLACE FUNCTION fn_auto_post_invoice()
RETURNS TRIGGER AS $$
DECLARE
    v_item RECORD;
BEGIN
    -- ========================================================================
    -- CENÁRIO A: NF confirmada (status -> POSTED)
    -- Cria movimentações de ENTRADA apenas para itens estocáveis
    -- ========================================================================
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
                ii.expiration_date,
                pdm.is_stockable,
                pdm.is_permanent
            FROM invoice_items ii
            JOIN catmat_items ci ON ci.id = ii.catalog_item_id
            JOIN catmat_pdms pdm ON pdm.id = ci.pdm_id
            WHERE ii.invoice_id = NEW.id
        LOOP
            IF v_item.is_stockable THEN
                -- Item estocável: gera movimentação de ENTRY no warehouse
                INSERT INTO stock_movements (
                    warehouse_id,
                    catalog_item_id,
                    movement_type,
                    unit_raw_id,
                    unit_conversion_id,
                    quantity_raw,
                    conversion_factor,
                    quantity_base,
                    unit_price_base,
                    total_value,
                    invoice_id,
                    invoice_item_id,
                    document_number,
                    user_id,
                    batch_number,
                    expiration_date
                ) VALUES (
                    NEW.warehouse_id,
                    v_item.catalog_item_id,
                    'ENTRY',
                    v_item.unit_raw_id,
                    v_item.unit_conversion_id,
                    v_item.quantity_raw,
                    v_item.conversion_factor,
                    v_item.quantity_base,
                    v_item.unit_value_base,
                    v_item.total_value,
                    NEW.id,
                    v_item.id,
                    NEW.invoice_number,
                    NEW.posted_by,
                    v_item.batch_number,
                    v_item.expiration_date
                );

            -- ELSIF v_item.is_permanent THEN
            --   Bem permanente: será tratado pelo módulo de patrimônio (implementação futura)
            --   INSERT INTO patrimonio_entries (...);

            -- ELSE: consumo/uso direto — nenhuma movimentação necessária
            END IF;
        END LOOP;

    -- ========================================================================
    -- CENÁRIO B: NF estornada (POSTED -> outro status)
    -- Cria ADJUSTMENT_SUB para reverter somente as ENTRYs existentes
    -- (apenas itens estocáveis tiveram ENTRY criado — os demais não têm nada a reverter)
    -- ========================================================================
    ELSIF OLD.status = 'POSTED' AND NEW.status <> 'POSTED' THEN

        INSERT INTO stock_movements (
            warehouse_id,
            catalog_item_id,
            movement_type,
            unit_raw_id,
            unit_conversion_id,
            quantity_raw,
            conversion_factor,
            quantity_base,
            unit_price_base,
            total_value,
            invoice_id,
            invoice_item_id,
            document_number,
            user_id,
            batch_number,
            expiration_date,
            notes
        )
        SELECT
            sm.warehouse_id,
            sm.catalog_item_id,
            'ADJUSTMENT_SUB',
            sm.unit_raw_id,
            sm.unit_conversion_id,
            sm.quantity_raw,
            sm.conversion_factor,
            sm.quantity_base,
            sm.unit_price_base,
            sm.total_value,
            sm.invoice_id,
            sm.invoice_item_id,
            'ESTORNO NF ' || NEW.invoice_number,
            COALESCE(NEW.posted_by, OLD.posted_by),
            sm.batch_number,
            sm.expiration_date,
            'Estorno automático - NF revertida de POSTED para ' || NEW.status::TEXT
        FROM stock_movements sm
        WHERE sm.invoice_id = NEW.id
          AND sm.movement_type = 'ENTRY';

    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

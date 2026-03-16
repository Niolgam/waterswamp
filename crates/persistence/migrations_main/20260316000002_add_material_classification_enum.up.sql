-- =============================================================================
-- Substitui os dois booleanos mutuamente exclusivos (is_stockable, is_permanent)
-- por um único enum material_classification_enum com 3 estados:
--
--   STOCKABLE   → item entra no estoque ao ser lançado em NF
--   PERMANENT   → bem permanente (patrimônio) — módulo de patrimônio futuro
--   DIRECT_USE  → consumo/uso direto — sem impacto em estoque
--
-- Vantagem: elimina o estado inválido (is_stockable=TRUE AND is_permanent=TRUE),
-- torna a interface mais clara e dispensa a constraint de exclusão mútua.
-- =============================================================================

-- 1. Criar o tipo enum
CREATE TYPE material_classification_enum AS ENUM ('STOCKABLE', 'PERMANENT', 'DIRECT_USE');

-- 2. Adicionar nova coluna derivando o valor dos booleanos existentes
ALTER TABLE catmat_pdms
    ADD COLUMN material_classification material_classification_enum NOT NULL
        GENERATED ALWAYS AS (
            CASE
                WHEN is_stockable = TRUE  THEN 'STOCKABLE'::material_classification_enum
                WHEN is_permanent = TRUE  THEN 'PERMANENT'::material_classification_enum
                ELSE 'DIRECT_USE'::material_classification_enum
            END
        ) STORED;

-- 3. Converter para coluna normal (remover GENERATED, manter valor migrado)
ALTER TABLE catmat_pdms ALTER COLUMN material_classification DROP EXPRESSION;

-- 4. Remover os booleanos e a constraint de exclusão mútua
ALTER TABLE catmat_pdms
    DROP CONSTRAINT IF EXISTS ck_pdm_not_both_stockable_permanent,
    DROP COLUMN IF EXISTS is_stockable,
    DROP COLUMN IF EXISTS is_permanent;

-- 5. Remover índices dos booleanos
DROP INDEX IF EXISTS idx_catmat_pdms_stockable;
DROP INDEX IF EXISTS idx_catmat_pdms_permanent;

-- 6. Criar índice no enum para filtros no módulo de catálogo
CREATE INDEX idx_catmat_pdms_classification ON catmat_pdms(material_classification);

-- =============================================================================
-- Atualiza fn_auto_post_invoice para usar o novo enum
-- =============================================================================

CREATE OR REPLACE FUNCTION fn_auto_post_invoice()
RETURNS TRIGGER AS $$
DECLARE
    v_item RECORD;
BEGIN
    -- ========================================================================
    -- CENÁRIO A: NF confirmada (status -> POSTED)
    -- Cria movimentações de ENTRADA apenas para itens STOCKABLE
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
                pdm.material_classification
            FROM invoice_items ii
            JOIN catmat_items ci ON ci.id = ii.catalog_item_id
            JOIN catmat_pdms pdm ON pdm.id = ci.pdm_id
            WHERE ii.invoice_id = NEW.id
        LOOP
            IF v_item.material_classification = 'STOCKABLE' THEN
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

            -- ELSIF v_item.material_classification = 'PERMANENT' THEN
            --   Bem permanente: será tratado pelo módulo de patrimônio (implementação futura)
            --   INSERT INTO patrimonio_entries (...);

            -- ELSE: DIRECT_USE — consumo/uso direto, nenhuma movimentação necessária
            END IF;
        END LOOP;

    -- ========================================================================
    -- CENÁRIO B: NF estornada (POSTED -> outro status)
    -- Cria ADJUSTMENT_SUB para reverter somente as ENTRYs existentes
    -- (apenas STOCKABLE geraram ENTRY — PERMANENT e DIRECT_USE não têm nada a reverter)
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

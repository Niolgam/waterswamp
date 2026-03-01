-- ============================================================================
-- Rollback: Reverter hierarquia CATMAT de 4 níveis para 3 níveis
-- ============================================================================

-- Restaurar class_id em catmat_items a partir do PDM
ALTER TABLE catmat_items ADD COLUMN class_id UUID;

UPDATE catmat_items ci
SET class_id = p.class_id
FROM catmat_pdms p
WHERE p.id = ci.pdm_id;

ALTER TABLE catmat_items ALTER COLUMN class_id SET NOT NULL;
ALTER TABLE catmat_items ADD CONSTRAINT catmat_items_class_id_fkey
    FOREIGN KEY (class_id) REFERENCES catmat_classes(id) ON DELETE RESTRICT;

-- Restaurar colunas operacionais removidas
ALTER TABLE catmat_items ADD COLUMN supplementary_description TEXT;
ALTER TABLE catmat_items ADD COLUMN specification TEXT;
ALTER TABLE catmat_items ADD COLUMN estimated_value DECIMAL(15, 2) NOT NULL DEFAULT 0;
ALTER TABLE catmat_items ADD COLUMN search_links TEXT;
ALTER TABLE catmat_items ADD COLUMN photo_url TEXT;
ALTER TABLE catmat_items ADD COLUMN is_permanent BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE catmat_items ADD COLUMN shelf_life_days INTEGER;
ALTER TABLE catmat_items ADD COLUMN requires_batch_control BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE catmat_items ADD CONSTRAINT ck_catmat_items_shelf_life
    CHECK (shelf_life_days IS NULL OR shelf_life_days > 0);

-- Remover colunas novas
ALTER TABLE catmat_items DROP COLUMN ncm_code;
ALTER TABLE catmat_items DROP COLUMN pdm_id;

-- Remover índices novos
DROP INDEX IF EXISTS idx_catmat_items_pdm;
DROP INDEX IF EXISTS idx_catmat_items_ncm;
DROP INDEX IF EXISTS idx_catmat_items_description_search;

-- Restaurar índices antigos
CREATE INDEX idx_catmat_items_class ON catmat_items(class_id);
CREATE INDEX idx_catmat_items_permanent ON catmat_items(is_permanent) WHERE is_permanent = TRUE;
CREATE INDEX idx_catmat_items_search ON catmat_items
    USING gin ((description || ' ' || COALESCE(specification, '')) gin_trgm_ops);

-- Remover tabela catmat_pdms
DROP TABLE IF EXISTS catmat_pdms;

-- ============================================================================
-- Migration: Ajustar hierarquia CATMAT para 4 níveis (Grupo → Classe → PDM → Item)
-- Adicionar tabela catmat_pdms entre catmat_classes e catmat_items
-- Adicionar coluna ncm_code em catmat_items
-- Remover campos operacionais extras de catmat_items
-- ============================================================================

-- ============================================================================
-- 1. Criar tabela CATMAT_PDMS (Padrão Descritivo de Material)
-- ============================================================================

CREATE TABLE catmat_pdms (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    class_id UUID NOT NULL REFERENCES catmat_classes(id) ON DELETE RESTRICT,
    code VARCHAR(20) NOT NULL UNIQUE,
    description VARCHAR(500) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_catmat_pdms_class ON catmat_pdms(class_id);
CREATE INDEX idx_catmat_pdms_code ON catmat_pdms(code);
CREATE INDEX idx_catmat_pdms_active ON catmat_pdms(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_catmat_pdms_search ON catmat_pdms USING gin (description gin_trgm_ops);

CREATE TRIGGER set_timestamp_catmat_pdms
BEFORE UPDATE ON catmat_pdms
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 2. Migrar dados existentes de catmat_items → catmat_pdms
--    Para cada item existente, criar um PDM na mesma classe
-- ============================================================================

-- Criar PDMs a partir dos itens existentes (usando o código do item como base)
INSERT INTO catmat_pdms (id, class_id, code, description, is_active, created_at, updated_at)
SELECT
    uuid_generate_v4(),
    ci.class_id,
    'PDM-' || ci.code,
    ci.description,
    ci.is_active,
    ci.created_at,
    ci.updated_at
FROM catmat_items ci
ON CONFLICT (code) DO NOTHING;

-- ============================================================================
-- 3. Alterar catmat_items: adicionar pdm_id, ncm_code; remover campos extras
-- ============================================================================

-- Adicionar colunas novas
ALTER TABLE catmat_items ADD COLUMN pdm_id UUID REFERENCES catmat_pdms(id) ON DELETE RESTRICT;
ALTER TABLE catmat_items ADD COLUMN ncm_code VARCHAR(20);

-- Atualizar pdm_id com os PDMs criados
UPDATE catmat_items ci
SET pdm_id = p.id
FROM catmat_pdms p
WHERE p.code = 'PDM-' || ci.code;

-- Tornar pdm_id NOT NULL
ALTER TABLE catmat_items ALTER COLUMN pdm_id SET NOT NULL;

-- Remover FK para class_id (agora o item referencia PDM, não classe)
ALTER TABLE catmat_items DROP CONSTRAINT IF EXISTS catmat_items_class_id_fkey;
ALTER TABLE catmat_items DROP COLUMN class_id;

-- Remover campos operacionais extras
ALTER TABLE catmat_items DROP COLUMN IF EXISTS supplementary_description;
ALTER TABLE catmat_items DROP COLUMN IF EXISTS specification;
ALTER TABLE catmat_items DROP COLUMN IF EXISTS estimated_value;
ALTER TABLE catmat_items DROP COLUMN IF EXISTS search_links;
ALTER TABLE catmat_items DROP COLUMN IF EXISTS photo_url;
ALTER TABLE catmat_items DROP COLUMN IF EXISTS is_permanent;
ALTER TABLE catmat_items DROP COLUMN IF EXISTS shelf_life_days;
ALTER TABLE catmat_items DROP COLUMN IF EXISTS requires_batch_control;

-- Remover constraint que referenciava shelf_life_days
ALTER TABLE catmat_items DROP CONSTRAINT IF EXISTS ck_catmat_items_shelf_life;

-- Remover índices antigos que referenciavam colunas removidas
DROP INDEX IF EXISTS idx_catmat_items_class;
DROP INDEX IF EXISTS idx_catmat_items_permanent;
DROP INDEX IF EXISTS idx_catmat_items_search;

-- Criar novos índices
CREATE INDEX idx_catmat_items_pdm ON catmat_items(pdm_id);
CREATE INDEX idx_catmat_items_ncm ON catmat_items(ncm_code) WHERE ncm_code IS NOT NULL;
CREATE INDEX idx_catmat_items_description_search ON catmat_items USING gin (description gin_trgm_ops);

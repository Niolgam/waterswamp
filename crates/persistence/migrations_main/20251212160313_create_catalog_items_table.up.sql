CREATE TABLE catalog_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Relacionamentos
    group_id UUID NOT NULL REFERENCES catalog_groups(id) ON DELETE RESTRICT,
    unit_of_measure_id UUID NOT NULL REFERENCES units_of_measure(id) ON DELETE RESTRICT,

    -- Dados Identificadores
    name VARCHAR(200) NOT NULL,
    catmat_code VARCHAR(20), -- Código CATMAT/CATSER (Federal)
    
    -- Detalhamento Técnico e Financeiro
    specification TEXT NOT NULL, -- Descrição detalhada para Editais/TRs
    estimated_value DECIMAL(15, 2) NOT NULL DEFAULT 0 CHECK (estimated_value >= 0),
    search_links TEXT, -- Links para pesquisa de preço
    photo_url TEXT,

    -- Comportamento no Sistema
    is_stockable BOOLEAN NOT NULL DEFAULT TRUE, -- FALSE para Serviços
    is_permanent BOOLEAN NOT NULL DEFAULT FALSE, -- TRUE para Bens Patrimoniais
    
    -- Parâmetros de validade (para materiais perecíveis)
    shelf_life_days INTEGER, -- Validade em dias após fabricação
    requires_batch_control BOOLEAN NOT NULL DEFAULT FALSE,
    
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_catalog_items_name_group UNIQUE (group_id, name),
    CONSTRAINT uq_catalog_items_catmat UNIQUE (catmat_code),
    CONSTRAINT ck_catalog_items_shelf_life CHECK (shelf_life_days IS NULL OR shelf_life_days > 0)
);

CREATE INDEX idx_catalog_items_group ON catalog_items(group_id);
CREATE INDEX idx_catalog_items_unit ON catalog_items(unit_of_measure_id);
CREATE INDEX idx_catalog_items_name ON catalog_items(name);
CREATE INDEX idx_catalog_items_catmat ON catalog_items(catmat_code) WHERE catmat_code IS NOT NULL;
CREATE INDEX idx_catalog_items_active ON catalog_items(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_catalog_items_stockable ON catalog_items(is_stockable) WHERE is_stockable = TRUE;
CREATE INDEX idx_catalog_items_search ON catalog_items 
USING gin ((name || ' ' || specification) gin_trgm_ops);

CREATE TRIGGER set_timestamp_catalog_items
BEFORE UPDATE ON catalog_items
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- Impacto na Tabela catalog_items
-- Previne que itens sejam vinculados a grupos que possuem filhos (grupos sintéticos)
CREATE OR REPLACE FUNCTION fn_prevent_item_in_synthetic_group()
RETURNS TRIGGER AS $$
BEGIN
    IF EXISTS (SELECT 1 FROM catalog_groups WHERE parent_id = NEW.group_id) THEN
        RAISE EXCEPTION 'Itens só podem ser vinculados a grupos folha (sem subgrupos).';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_catalog_item_leaf_only
BEFORE INSERT OR UPDATE ON catalog_items
FOR EACH ROW EXECUTE PROCEDURE fn_prevent_item_in_synthetic_group();

-- Create enum for item type
CREATE TYPE item_type_enum AS ENUM ('MATERIAL', 'SERVICE');

-- Create catalog_groups table with hierarchical support
CREATE TABLE catalog_groups (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    parent_id UUID REFERENCES catalog_groups(id) ON DELETE RESTRICT,
    
    -- Identificação
    name VARCHAR(100) NOT NULL,
    code VARCHAR(20) NOT NULL, -- Ex: 'INFORMATICA', 'PAPELARIA'
    
    -- Governança Orçamentária
    item_type item_type_enum NOT NULL, -- MATERIAL ou SERVICE
    budget_classification_id UUID NOT NULL REFERENCES budget_classifications(id),
    
    -- Status e Auditoria
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Garante que não existam códigos duplicados sob o mesmo pai
    CONSTRAINT unique_group_code_per_level UNIQUE (parent_id, code)
);

-- Índices
CREATE INDEX idx_catalog_groups_parent ON catalog_groups(parent_id);
CREATE INDEX idx_catalog_groups_code ON catalog_groups(code);
CREATE INDEX idx_catalog_groups_name ON catalog_groups(name);
CREATE INDEX idx_catalog_groups_item_type ON catalog_groups(item_type);
CREATE INDEX idx_catalog_groups_budget ON catalog_groups(budget_classification_id);
CREATE INDEX idx_catalog_groups_active ON catalog_groups(is_active) WHERE is_active = TRUE;

-- Trigger para updated_at
CREATE TRIGGER set_timestamp_catalog_groups
BEFORE UPDATE ON catalog_groups
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- A. Validação de Herança Orçamentária
-- Garante que um subgrupo herde a natureza do pai
CREATE OR REPLACE FUNCTION fn_validate_catalog_group_hierarchy()
RETURNS TRIGGER AS $$
DECLARE
    v_parent_type item_type_enum;
    v_parent_full_code VARCHAR(30);
    v_new_full_code VARCHAR(30);
BEGIN
    -- Se tiver pai, validamos a linhagem
    IF NEW.parent_id IS NOT NULL THEN
        SELECT g.item_type, b.full_code 
        INTO v_parent_type, v_parent_full_code
        FROM catalog_groups g
        JOIN budget_classifications b ON g.budget_classification_id = b.id
        WHERE g.id = NEW.parent_id;

        SELECT full_code INTO v_new_full_code 
        FROM budget_classifications WHERE id = NEW.budget_classification_id;

        -- 1. Trava de Tipo (Material vs Serviço)
        IF NEW.item_type != v_parent_type THEN
            RAISE EXCEPTION 'Conflito: Grupo pai é % mas subgrupo tenta ser %.', v_parent_type, NEW.item_type;
        END IF;

        -- 2. Trava de Elemento (c.g.mm.ee - Ex: 3.3.90.30)
        -- Impede que um subgrupo mude de 'Consumo' para 'Permanente' ou 'Serviços'
        IF LEFT(v_new_full_code, 10) != LEFT(v_parent_full_code, 10) THEN
            RAISE EXCEPTION 'Conflito Orçamentário: O subgrupo deve pertencer ao mesmo Elemento de Despesa do pai (%).', LEFT(v_parent_full_code, 10);
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_catalog_group_hierarchy_consistency
BEFORE INSERT OR UPDATE OF parent_id, budget_classification_id, item_type ON catalog_groups
FOR EACH ROW EXECUTE PROCEDURE fn_validate_catalog_group_hierarchy();

-- B. Bloqueio de Grupos Sintéticos (Regra do Nó Folha)
-- Impede a criação de subgrupos se o pai já possuir itens diretamente vinculados
CREATE OR REPLACE FUNCTION fn_prevent_subgroup_if_items_exist()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.parent_id IS NOT NULL AND EXISTS (SELECT 1 FROM catalog_items WHERE group_id = NEW.parent_id) THEN
        RAISE EXCEPTION 'Não é possível criar subgrupos: o grupo pai já possui itens vinculados. Mova os itens primeiro.';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_catalog_group_leaf_safety
BEFORE INSERT ON catalog_groups
FOR EACH ROW EXECUTE PROCEDURE fn_prevent_subgroup_if_items_exist();

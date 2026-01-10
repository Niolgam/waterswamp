-- ============================================================================
-- SIGALM - Sistema de Gestão de Almoxarifado
-- Migration: Classificações Orçamentárias (Hierarquia de 5 níveis)
-- ============================================================================

CREATE TABLE budget_classifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- O ID do nível superior (Ex: O 'Elemento' aponta para a 'Modalidade')
    parent_id UUID REFERENCES budget_classifications(id) ON DELETE RESTRICT,
    
    -- O código apenas deste nível (Ex: '30')
    code_part VARCHAR(10) NOT NULL,
    
    -- O código completo calculado/armazenado para busca rápida (Ex: '3.3.90.30')
    full_code VARCHAR(30) UNIQUE NOT NULL,
    
    name VARCHAR(255) NOT NULL,
    
    -- Nível da classificação (1 a 5)
    -- 1: Categoria Econômica, 2: Grupo de Despesa, 3: Modalidade, 4: Elemento, 5: Subelemento
    level INTEGER NOT NULL CHECK (level BETWEEN 1 AND 5),

    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Índices para navegação na árvore
CREATE INDEX idx_budget_classifications_parent ON budget_classifications(parent_id);
CREATE INDEX idx_budget_classifications_level ON budget_classifications(level);
CREATE INDEX idx_budget_classifications_full_code ON budget_classifications(full_code);
CREATE INDEX idx_budget_classifications_active ON budget_classifications(is_active) WHERE is_active = TRUE;

-- Trigger para updated_at
CREATE TRIGGER set_timestamp_budget_classifications
BEFORE UPDATE ON budget_classifications
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- Função para calcular hierarquia automaticamente
CREATE OR REPLACE FUNCTION fn_calculate_budget_hierarchy()
RETURNS TRIGGER AS $$
DECLARE
    v_parent_full_code VARCHAR(30);
    v_parent_level INTEGER;
BEGIN
    -- Caso 1: É um item de nível 1 (Categoria Econômica)
    IF NEW.parent_id IS NULL THEN
        NEW.level := 1;
        NEW.full_code := NEW.code_part;
    
    -- Caso 2: É um item dependente (níveis 2 a 5)
    ELSE
        -- Busca o código completo e o nível do pai
        SELECT full_code, level 
        INTO v_parent_full_code, v_parent_level
        FROM budget_classifications 
        WHERE id = NEW.parent_id;

        -- Validação de segurança: Não permitir mais que 5 níveis
        IF v_parent_level >= 5 THEN
            RAISE EXCEPTION 'A Classificação Orçamentária não pode exceder 5 níveis.';
        END IF;

        NEW.level := v_parent_level + 1;
        NEW.full_code := v_parent_full_code || '.' || NEW.code_part;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_budget_hierarchy_auto
BEFORE INSERT OR UPDATE OF parent_id, code_part ON budget_classifications
FOR EACH ROW
EXECUTE FUNCTION fn_calculate_budget_hierarchy();


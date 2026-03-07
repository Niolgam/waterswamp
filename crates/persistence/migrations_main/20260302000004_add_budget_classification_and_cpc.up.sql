-- ============================================================================
-- Migration: Adicionar budget_classification_id em catmat_items e catser_items,
--            e code_cpc em catser_items
-- ============================================================================

-- catmat_items: adicionar budget_classification_id
ALTER TABLE catmat_items
    ADD COLUMN budget_classification_id UUID REFERENCES budget_classifications(id) ON DELETE SET NULL;

CREATE INDEX idx_catmat_items_budget_class ON catmat_items(budget_classification_id) WHERE budget_classification_id IS NOT NULL;

-- catser_items: adicionar budget_classification_id e code_cpc
ALTER TABLE catser_items
    ADD COLUMN budget_classification_id UUID REFERENCES budget_classifications(id) ON DELETE SET NULL,
    ADD COLUMN code_cpc VARCHAR(20);

CREATE INDEX idx_catser_items_budget_class ON catser_items(budget_classification_id) WHERE budget_classification_id IS NOT NULL;
CREATE INDEX idx_catser_items_cpc ON catser_items(code_cpc) WHERE code_cpc IS NOT NULL;

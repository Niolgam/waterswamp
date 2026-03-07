DROP INDEX IF EXISTS idx_catser_items_cpc;
DROP INDEX IF EXISTS idx_catser_items_budget_class;
ALTER TABLE catser_items DROP COLUMN IF EXISTS code_cpc;
ALTER TABLE catser_items DROP COLUMN IF EXISTS budget_classification_id;

DROP INDEX IF EXISTS idx_catmat_items_budget_class;
ALTER TABLE catmat_items DROP COLUMN IF EXISTS budget_classification_id;

-- ==========================================================================
-- Ticket 1.1 — RF-AST-11: Configuração de Depreciação por Categoria
--
-- Uma linha por categoria de veículo. O cálculo mensal é linear:
--   depreciation_monthly = (purchase_value - residual_value_min) / useful_life_years / 12
--   accumulated = min(depreciation_monthly * months_elapsed, purchase_value - residual_value_min)
--   current_value = max(purchase_value - accumulated, residual_value_min)
--
-- `is_active = false` suspende o cálculo (sinistros, baixas — RNs RF-AST-09, RF-AST-12).
-- ==========================================================================

CREATE TABLE depreciation_configs (
    id                  UUID            PRIMARY KEY DEFAULT uuid_generate_v4(),
    vehicle_category_id UUID            NOT NULL UNIQUE REFERENCES vehicle_categories(id) ON DELETE RESTRICT,
    useful_life_years   NUMERIC(5, 2)   NOT NULL CHECK (useful_life_years > 0),
    residual_value_min  NUMERIC(15, 2)  NOT NULL DEFAULT 0 CHECK (residual_value_min >= 0),
    -- Apenas método linear suportado nesta versão (DRS RN20)
    method              TEXT            NOT NULL DEFAULT 'LINEAR' CHECK (method = 'LINEAR'),
    is_active           BOOLEAN         NOT NULL DEFAULT true,
    notes               TEXT,
    created_by          UUID,
    updated_by          UUID,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_depreciation_configs_category ON depreciation_configs(vehicle_category_id);

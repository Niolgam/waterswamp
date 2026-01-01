-- Add fields for stock management features
-- Feature 1: Material blocking
-- Feature 2: Stock maintenance (resupply days already have min_stock, max_stock, location)

ALTER TABLE warehouse_stocks
ADD COLUMN IF NOT EXISTS resupply_days INTEGER,
ADD COLUMN IF NOT EXISTS is_blocked BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS block_reason TEXT,
ADD COLUMN IF NOT EXISTS blocked_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS blocked_by UUID REFERENCES users(id);

-- Create index for blocked materials queries
CREATE INDEX IF NOT EXISTS idx_warehouse_stocks_blocked ON warehouse_stocks(is_blocked) WHERE is_blocked = TRUE;

-- Add comment
COMMENT ON COLUMN warehouse_stocks.resupply_days IS 'Prazo de ressuprimento em dias';
COMMENT ON COLUMN warehouse_stocks.is_blocked IS 'Material bloqueado para requisições';
COMMENT ON COLUMN warehouse_stocks.block_reason IS 'Justificativa do bloqueio';
COMMENT ON COLUMN warehouse_stocks.blocked_at IS 'Data/hora do bloqueio';
COMMENT ON COLUMN warehouse_stocks.blocked_by IS 'Usuário que bloqueou';

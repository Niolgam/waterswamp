-- Rollback stock management fields

DROP INDEX IF EXISTS idx_warehouse_stocks_blocked;

ALTER TABLE warehouse_stocks
DROP COLUMN IF EXISTS blocked_by,
DROP COLUMN IF EXISTS blocked_at,
DROP COLUMN IF EXISTS block_reason,
DROP COLUMN IF EXISTS is_blocked,
DROP COLUMN IF EXISTS resupply_days;

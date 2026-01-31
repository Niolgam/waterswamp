-- ============================================================================
-- Rollback: Remove Audit Views Optimization Indexes
-- ============================================================================

-- Requisition history indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_req_history_audit_trail_cover;
DROP INDEX CONCURRENTLY IF EXISTS idx_req_history_date_cover;
DROP INDEX CONCURRENTLY IF EXISTS idx_req_history_rollback_user;
DROP INDEX CONCURRENTLY IF EXISTS idx_req_history_critical_ops;
DROP INDEX CONCURRENTLY IF EXISTS idx_req_history_ip;
DROP INDEX CONCURRENTLY IF EXISTS idx_req_history_rollback_points_cover;

-- Requisition item history indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_req_item_history_transaction_count;

-- Invoice history indexes
DROP INDEX CONCURRENTLY IF EXISTS idx_inv_history_audit_trail_cover;
DROP INDEX CONCURRENTLY IF EXISTS idx_inv_item_history_transaction_count;

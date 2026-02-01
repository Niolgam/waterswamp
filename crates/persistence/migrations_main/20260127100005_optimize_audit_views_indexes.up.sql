-- ============================================================================
-- Migration: Optimize Audit Views with Covering Indexes
-- Description: Adds covering indexes to improve performance of audit trail views
-- Note: Using regular CREATE INDEX (not CONCURRENTLY) because SQLx runs migrations in transactions
-- ============================================================================

-- ============================================================================
-- 1. ÍNDICES PARA vw_requisition_audit_trail
-- ============================================================================

-- Covering index para a view principal
-- Inclui todas as colunas selecionadas pela view para evitar table lookups
CREATE INDEX IF NOT EXISTS idx_req_history_audit_trail_cover
    ON requisition_history(requisition_id, performed_at DESC)
    INCLUDE (
        id, requisition_number, operation, status_before, status_after,
        changed_fields, performed_by, performed_by_name, reason,
        is_rollback, rollback_to_history_id, transaction_id
    );

-- Índice para a subquery que conta itens por transaction_id
-- Otimiza: (SELECT COUNT(*) FROM requisition_item_history WHERE transaction_id = ...)
CREATE INDEX IF NOT EXISTS idx_req_item_history_transaction_count
    ON requisition_item_history(transaction_id)
    INCLUDE (id);

-- Índice para busca por período com campos da view
CREATE INDEX IF NOT EXISTS idx_req_history_date_cover
    ON requisition_history(performed_at DESC)
    INCLUDE (requisition_id, operation, status_after, performed_by_name);

-- ============================================================================
-- 2. ÍNDICES PARA vw_invoice_audit_trail
-- ============================================================================

-- Covering index para a view de auditoria de NFs
CREATE INDEX IF NOT EXISTS idx_inv_history_audit_trail_cover
    ON invoice_history(invoice_id, performed_at DESC)
    INCLUDE (
        id, invoice_number, operation, status_before, status_after,
        changed_fields, performed_by, performed_by_name, reason,
        is_rollback, rollback_to_history_id, transaction_id
    );

-- Índice para subquery de itens de NF
CREATE INDEX IF NOT EXISTS idx_inv_item_history_transaction_count
    ON invoice_item_history(transaction_id)
    INCLUDE (id);

-- ============================================================================
-- 3. ÍNDICES PARA CONSULTAS FREQUENTES DE AUDITORIA
-- ============================================================================

-- Busca de rollbacks por usuário (análise de quem faz mais rollbacks)
CREATE INDEX IF NOT EXISTS idx_req_history_rollback_user
    ON requisition_history(performed_by, performed_at DESC)
    WHERE is_rollback = TRUE;

-- Busca de operações críticas (aprovações, rejeições, cancelamentos)
CREATE INDEX IF NOT EXISTS idx_req_history_critical_ops
    ON requisition_history(operation, performed_at DESC)
    WHERE operation IN ('APPROVAL', 'REJECTION', 'CANCELLATION');

-- Busca por IP (análise de segurança)
CREATE INDEX IF NOT EXISTS idx_req_history_ip
    ON requisition_history(ip_address, performed_at DESC)
    WHERE ip_address IS NOT NULL;

-- ============================================================================
-- 4. ÍNDICES PARA fn_list_rollback_points
-- ============================================================================

-- Otimiza a função que lista pontos de rollback disponíveis
CREATE INDEX IF NOT EXISTS idx_req_history_rollback_points_cover
    ON requisition_history(requisition_id, performed_at DESC)
    INCLUDE (id, operation, status_after, performed_by_name, changed_fields, is_rollback)
    WHERE is_rollback_point = TRUE;

-- ============================================================================
-- 5. ESTATÍSTICAS PARA O QUERY PLANNER
-- ============================================================================

-- Atualiza estatísticas das tabelas de histórico para o planner
ANALYZE requisition_history;
ANALYZE requisition_item_history;
ANALYZE invoice_history;
ANALYZE invoice_item_history;

-- ============================================================================
-- 6. COMENTÁRIOS
-- ============================================================================

COMMENT ON INDEX idx_req_history_audit_trail_cover IS
    'Covering index para vw_requisition_audit_trail - evita table lookups';

COMMENT ON INDEX idx_req_item_history_transaction_count IS
    'Otimiza contagem de itens por transação na view de auditoria';

COMMENT ON INDEX idx_req_history_critical_ops IS
    'Índice parcial para operações críticas (aprovação, rejeição, cancelamento)';

COMMENT ON INDEX idx_req_history_ip IS
    'Índice para análise de segurança por IP de origem';

COMMENT ON INDEX idx_req_history_rollback_points_cover IS
    'Covering index para fn_list_rollback_points';

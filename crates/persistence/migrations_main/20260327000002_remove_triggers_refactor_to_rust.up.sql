-- ============================================================================
-- Migration: Remoção de Triggers — Regras de Negócio Migradas para Rust
--
-- Triggers removidas e suas responsabilidades em Rust:
--   trg_stock_movement_process   → StockMovementService::process_movement()
--   trg_auto_post_invoice        → InvoiceService::post_invoice() / cancel_invoice()
--   trg_handle_requisition_reservation → RequisitionService::approve/cancel
--   trg_capture_requisition_item_value → RequisitionService::add_item()
--   trg_update_invoice_totals    → InvoiceService::recalculate_totals_in_tx()
--   trg_update_requisition_total → RequisitionService::recalculate_totals_in_tx()
--
-- Triggers MANTIDAS:
--   trg_generate_requisition_number  (sequencial anti-race)
--   set_timestamp_*                  (updated_at automático)
--   trg_requisition_audit            (auditoria)
--   trg_invoice_audit                (auditoria)
--   fn_org_unit_integrity            (hierarquia org)
--   fn_update_descendant_paths       (caminhos org)
-- ============================================================================

-- 1. Motor de estoque (movimentações)
DROP TRIGGER IF EXISTS trg_stock_movement_process ON stock_movements;
DROP FUNCTION IF EXISTS fn_process_stock_movement() CASCADE;

-- 2. Lançamento automático de NF no estoque
DROP TRIGGER IF EXISTS trg_auto_post_invoice ON invoices;
DROP FUNCTION IF EXISTS fn_auto_post_invoice() CASCADE;

-- 3. Gestão de reservas de requisição
DROP TRIGGER IF EXISTS trg_handle_requisition_reservation ON requisitions;
DROP FUNCTION IF EXISTS fn_manage_stock_reservation() CASCADE;

-- 4. Captura de preço no momento da requisição
DROP TRIGGER IF EXISTS trg_capture_requisition_item_value ON requisition_items;
DROP FUNCTION IF EXISTS fn_capture_requisition_item_value() CASCADE;

-- 5. Recálculo de totais da NF
DROP TRIGGER IF EXISTS trg_update_invoice_totals ON invoice_items;
DROP FUNCTION IF EXISTS fn_update_invoice_totals() CASCADE;

-- 6. Recálculo de totais da requisição
DROP TRIGGER IF EXISTS trg_update_requisition_total ON requisition_items;
DROP FUNCTION IF EXISTS fn_update_requisition_total() CASCADE;

-- 7. Adicionar updated_at em requisition_items (necessário para operações Rust)
ALTER TABLE requisition_items
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

CREATE TRIGGER set_timestamp_requisition_items
BEFORE UPDATE ON requisition_items
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- 8. Acrescentar catmat_item_id como alias de catalog_item_id para compatibilidade com DTO Rust
--    Feito via VIEW para não quebrar triggers existentes que usam catalog_item_id diretamente
--    (sem renomear a coluna original)

COMMENT ON TABLE stock_movements IS
    'Movimentações de estoque — processadas pelo StockMovementService em Rust (triggers removidas em 2026-03-27)';

COMMENT ON TABLE invoice_adjustments IS
    'Ajustes de liquidação (glosas) processados pelo InvoiceAdjustmentService';

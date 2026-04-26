-- Materialized view: aggregate stock position per warehouse
CREATE MATERIALIZED VIEW mv_warehouse_stock_summary AS
SELECT
    ws.warehouse_id,
    w.name                                                              AS warehouse_name,
    w.code                                                              AS warehouse_code,
    COUNT(*)                                                            AS total_items,
    SUM(ws.quantity * ws.average_unit_value)                            AS total_stock_value,
    COUNT(*) FILTER (WHERE ws.min_stock IS NOT NULL
                       AND ws.quantity <= ws.min_stock)                 AS low_stock_count,
    COUNT(*) FILTER (WHERE ws.is_blocked)                               AS blocked_count
FROM warehouse_stocks ws
JOIN warehouses w ON w.id = ws.warehouse_id
GROUP BY ws.warehouse_id, w.name, w.code;

CREATE UNIQUE INDEX ON mv_warehouse_stock_summary(warehouse_id);

-- Materialized view: daily movement aggregates (last 90 days by default, full refresh)
CREATE MATERIALIZED VIEW mv_daily_movements AS
SELECT
    date_trunc('day', sm.movement_date)::DATE                          AS movement_date,
    sm.warehouse_id,
    sm.movement_type::TEXT                                              AS movement_type,
    COUNT(*)                                                            AS movement_count,
    SUM(sm.quantity_base)                                               AS total_quantity,
    SUM(sm.total_value)                                                 AS total_value
FROM stock_movements sm
GROUP BY date_trunc('day', sm.movement_date), sm.warehouse_id, sm.movement_type;

CREATE UNIQUE INDEX ON mv_daily_movements(movement_date, warehouse_id, movement_type);

-- Materialized view: supplier performance metrics
CREATE MATERIALIZED VIEW mv_supplier_performance AS
SELECT
    s.id                                                                AS supplier_id,
    s.legal_name                                                        AS supplier_name,
    s.quality_score,
    COUNT(DISTINCT i.id)                                                AS total_invoices,
    COUNT(DISTINCT ia.id)                                               AS total_adjustments,
    COALESCE(AVG(i.total_value), 0)                                     AS avg_invoice_value
FROM suppliers s
LEFT JOIN invoices i
       ON i.supplier_id = s.id AND i.status = 'POSTED'
LEFT JOIN invoice_adjustments ia
       ON ia.invoice_id = i.id
GROUP BY s.id, s.legal_name, s.quality_score;

CREATE UNIQUE INDEX ON mv_supplier_performance(supplier_id);

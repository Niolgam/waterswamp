CREATE TYPE stock_alert_type_enum AS ENUM (
    'LOW_STOCK',
    'BATCH_EXPIRING',
    'BATCH_EXPIRED',
    'REQUISITION_OVERDUE',
    'QUOTA_EXCEEDED'
);

CREATE TYPE stock_alert_status_enum AS ENUM (
    'OPEN',
    'ACKNOWLEDGED',
    'RESOLVED',
    'SLA_BREACHED'
);

CREATE TABLE stock_alerts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    alert_type stock_alert_type_enum NOT NULL,
    status stock_alert_status_enum NOT NULL DEFAULT 'OPEN',
    warehouse_id UUID REFERENCES warehouses(id) ON DELETE CASCADE,
    catalog_item_id UUID REFERENCES catmat_items(id) ON DELETE CASCADE,
    batch_number TEXT,
    requisition_id UUID REFERENCES requisitions(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    description TEXT,
    severity TEXT NOT NULL DEFAULT 'MEDIUM'
        CHECK (severity IN ('LOW', 'MEDIUM', 'HIGH', 'CRITICAL')),
    sla_deadline TIMESTAMPTZ,
    acknowledged_at TIMESTAMPTZ,
    acknowledged_by UUID REFERENCES users(id) ON DELETE SET NULL,
    resolved_at TIMESTAMPTZ,
    resolved_by UUID REFERENCES users(id) ON DELETE SET NULL,
    sla_breached_at TIMESTAMPTZ,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_stock_alerts_open ON stock_alerts(status, created_at DESC)
    WHERE status IN ('OPEN', 'ACKNOWLEDGED');
CREATE INDEX idx_stock_alerts_warehouse ON stock_alerts(warehouse_id, status);
CREATE INDEX idx_stock_alerts_sla ON stock_alerts(sla_deadline)
    WHERE status IN ('OPEN', 'ACKNOWLEDGED') AND sla_deadline IS NOT NULL;
CREATE INDEX idx_stock_alerts_type ON stock_alerts(alert_type, status);

CREATE TRIGGER set_timestamp_stock_alerts
BEFORE UPDATE ON stock_alerts
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

INSERT INTO system_settings (key, value, description) VALUES
    ('alerts.low_stock_enabled',      'true', 'Enable automatic LOW_STOCK alerts'),
    ('alerts.expiry_days_ahead',      '30',   'Days ahead to generate BATCH_EXPIRING alerts'),
    ('alerts.expired_check_enabled',  'true', 'Enable BATCH_EXPIRED daily check'),
    ('alerts.sla_hours_low_stock',    '48',   'SLA deadline in hours for LOW_STOCK alerts'),
    ('alerts.sla_hours_expiring',     '72',   'SLA deadline in hours for BATCH_EXPIRING alerts'),
    ('alerts.sla_hours_overdue',      '24',   'SLA deadline in hours for REQUISITION_OVERDUE alerts')
ON CONFLICT (key) DO NOTHING;

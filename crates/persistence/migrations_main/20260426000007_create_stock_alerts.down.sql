DELETE FROM system_settings WHERE key IN (
    'alerts.low_stock_enabled',
    'alerts.expiry_days_ahead',
    'alerts.expired_check_enabled',
    'alerts.sla_hours_low_stock',
    'alerts.sla_hours_expiring',
    'alerts.sla_hours_overdue'
);

DROP TABLE IF EXISTS stock_alerts;
DROP TYPE IF EXISTS stock_alert_status_enum;
DROP TYPE IF EXISTS stock_alert_type_enum;

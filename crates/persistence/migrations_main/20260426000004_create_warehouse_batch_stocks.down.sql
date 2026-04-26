DROP TABLE IF EXISTS warehouse_batch_stocks CASCADE;

DELETE FROM system_settings WHERE key IN (
    'fefo.enabled',
    'fefo.expiry_alert_days',
    'fefo.allow_expired_exit'
);

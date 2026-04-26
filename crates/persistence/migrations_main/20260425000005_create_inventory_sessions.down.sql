DROP TABLE IF EXISTS inventory_session_items;
DROP TABLE IF EXISTS inventory_sessions;
DROP TYPE IF EXISTS inventory_session_status_enum;
DELETE FROM system_settings WHERE key = 'inventory.tolerance_percentage';

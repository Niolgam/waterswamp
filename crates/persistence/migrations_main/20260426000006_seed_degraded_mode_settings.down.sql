DELETE FROM system_settings WHERE key IN (
    'circuit.comprasnet.failure_threshold',
    'circuit.comprasnet.recovery_seconds',
    'circuit.siorg.failure_threshold',
    'circuit.siorg.recovery_seconds',
    'throttle.comprasnet.max_calls_per_minute',
    'throttle.siorg.max_calls_per_minute',
    'degraded.notify_admin_email'
);

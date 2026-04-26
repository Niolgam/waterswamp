DELETE FROM system_settings WHERE key IN (
    'comprasnet.empenho_validation_enabled',
    'comprasnet.empenho_api_base_url',
    'comprasnet.empenho_strict_mode',
    'comprasnet.empenho_api_token'
);

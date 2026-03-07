DELETE FROM system_settings WHERE key IN (
    'compras_gov.validation_enabled',
    'compras_gov.catmat_api_base_url',
    'compras_gov.catser_api_base_url'
);

INSERT INTO system_settings (key, value, value_type, category, description)
VALUES
    ('comprasnet.empenho_validation_enabled', 'false', 'boolean', 'comprasnet',
     'Habilita validação de saldo de empenho via API Comprasnet antes do recebimento de NF (RF-030/RN-002)'),
    ('comprasnet.empenho_api_base_url', '"https://api.compras.gov.br"', 'string', 'comprasnet',
     'URL base da API Comprasnet para consulta de empenhos'),
    ('comprasnet.empenho_strict_mode', 'true', 'boolean', 'comprasnet',
     'Se true, bloqueia NF quando API Comprasnet está indisponível. Se false, registra aviso e prossegue.'),
    ('comprasnet.empenho_api_token', 'null', 'string', 'comprasnet',
     'Token Bearer para autenticação na API Comprasnet (opcional)')
ON CONFLICT (key) DO NOTHING;

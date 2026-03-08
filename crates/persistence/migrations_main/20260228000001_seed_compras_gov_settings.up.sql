-- Seed ComprasGov system settings
INSERT INTO system_settings (key, value, value_type, category, description)
VALUES
    ('compras_gov.validation_enabled', 'true'::jsonb, 'boolean', 'compras_gov', 'Habilita/desabilita a validação de itens de catálogo via API ComprasGov'),
    ('compras_gov.catmat_api_base_url', '"https://dadosabertos.compras.gov.br/modulo-material"'::jsonb, 'string', 'compras_gov', 'URL base da API de materiais do ComprasGov'),
    ('compras_gov.catser_api_base_url', '"https://dadosabertos.compras.gov.br/modulo-servico"'::jsonb, 'string', 'compras_gov', 'URL base da API de serviços do ComprasGov')
ON CONFLICT (key) DO NOTHING;

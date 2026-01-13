-- ============================================================================
-- Migration: Create System Settings Table
-- Description: Global system configuration with runtime modification support
-- ============================================================================

CREATE TABLE IF NOT EXISTS system_settings (
    key VARCHAR(100) PRIMARY KEY,
    value JSONB NOT NULL,
    value_type VARCHAR(20) DEFAULT 'string',  -- string, number, boolean, json
    description TEXT,
    category VARCHAR(50),                      -- Grouping (ex: 'siorg', 'inventory', 'units')
    is_sensitive BOOLEAN DEFAULT FALSE,        -- If true, don't display in logs

    -- Audit
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID,

    CONSTRAINT chk_value_type CHECK (value_type IN ('string', 'number', 'boolean', 'json'))
);

CREATE INDEX idx_system_settings_category ON system_settings(category);
CREATE INDEX idx_system_settings_sensitive ON system_settings(is_sensitive) WHERE is_sensitive = TRUE;

CREATE TRIGGER set_system_settings_updated_at
    BEFORE UPDATE ON system_settings
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

-- Seed initial settings
INSERT INTO system_settings (key, value, value_type, description, category) VALUES
-- SIORG Settings
('siorg.sync_enabled', 'true', 'boolean',
 'Habilita sincronização automática com SIORG', 'siorg'),
('siorg.sync_interval_hours', '24', 'number',
 'Intervalo em horas entre sincronizações automáticas', 'siorg'),
('siorg.auto_approve_updates', 'false', 'boolean',
 'Se true, aplica atualizações do SIORG automaticamente. Se false, enfileira para revisão.', 'siorg'),
('siorg.hierarchy_strict_mode', 'false', 'boolean',
 'Se true, rejeita unidades cuja hierarquia local diverge do SIORG', 'siorg'),
('siorg.api_base_url', '"https://api-siorg.economia.gov.br"', 'string',
 'URL base da API SIORG', 'siorg'),

-- Units Settings
('units.enforce_active_check', 'true', 'boolean',
 'Força verificação se a unidade está ativa em operações', 'units'),
('units.allow_custom_units', 'true', 'boolean',
 'Permite criar unidades sem código SIORG', 'units'),
('units.max_hierarchy_depth', '10', 'number',
 'Profundidade máxima da hierarquia organizacional', 'units'),

-- Inventory Settings
('inventory.price_divergence_threshold', '0.20', 'number',
 'Percentual máximo de diferença entre valor de entrada e custo médio (0.20 = 20%)', 'inventory'),
('inventory.allow_negative_stock', 'false', 'boolean',
 'Define se o sistema permite saídas sem saldo físico suficiente', 'inventory')

ON CONFLICT (key) DO NOTHING;

COMMENT ON TABLE system_settings IS 'Configurações globais do sistema, alteráveis em runtime via API';
COMMENT ON COLUMN system_settings.value IS 'Valor armazenado como JSONB. Para strings, usar formato "valor" com aspas.';
COMMENT ON COLUMN system_settings.is_sensitive IS 'Marca se o valor é sensível (ex: API keys). Não será exposto em logs.';

-- ==========================================================================
-- Ticket 1.3 — RF-ADM-01/02: Parametrização e Checklists de Vistoria
--
-- RF-ADM-01: Parâmetros globais do módulo frota (chave→valor tipado).
-- RF-ADM-02: Templates de checklist de vistoria reutilizáveis.
-- ==========================================================================

CREATE TABLE fleet_system_params (
    id          UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    chave       TEXT        NOT NULL UNIQUE,
    valor       TEXT        NOT NULL,
    descricao   TEXT,
    updated_by  UUID,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Inserts de parâmetros padrão (idempotente via ON CONFLICT)
INSERT INTO fleet_system_params (chave, valor, descricao) VALUES
    ('odometer.max_speed_kmh',        '200',  'Velocidade máxima plausível entre leituras (km/h). Acima disso → quarentena.'),
    ('odometer.projection_days',      '30',   'Horizonte de projeção do hodômetro em dias.'),
    ('depreciation.default_method',   'LINEAR', 'Método de depreciação padrão para novas configurações.')
ON CONFLICT (chave) DO NOTHING;

-- RF-ADM-02: Templates de checklist
CREATE TABLE fleet_checklist_templates (
    id          UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    nome        TEXT        NOT NULL,
    descricao   TEXT,
    ativo       BOOLEAN     NOT NULL DEFAULT TRUE,
    created_by  UUID,
    updated_by  UUID,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE fleet_checklist_items (
    id              UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    template_id     UUID        NOT NULL REFERENCES fleet_checklist_templates(id) ON DELETE CASCADE,
    descricao       TEXT        NOT NULL,
    obrigatorio     BOOLEAN     NOT NULL DEFAULT TRUE,
    ordem           INTEGER     NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_fci_template_id ON fleet_checklist_items(template_id);
CREATE INDEX idx_fct_ativo ON fleet_checklist_templates(ativo) WHERE ativo = TRUE;

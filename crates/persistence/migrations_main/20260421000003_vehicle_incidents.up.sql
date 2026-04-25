-- ==========================================================================
-- Ticket 1.1 — RF-AST-12: Sinistros de Veículos
--
-- Registro de sinistros (acidentes, roubo, incêndio, etc.).
-- Ao abrir um sinistro:
--   - operational_status → INDISPONIVEL (via serviço, não trigger)
--   - allocation_status  → LIVRE        (idem)
--   - depreciation suspenso             (idem — via flag em vehicles ou serviço)
-- ==========================================================================

CREATE TYPE vehicle_incident_type_enum AS ENUM (
    'ACIDENTE',
    'ROUBO_FURTO',
    'INCENDIO',
    'ALAGAMENTO',
    'VANDALISMO',
    'OUTRO'
);

CREATE TYPE vehicle_incident_status_enum AS ENUM (
    'ABERTO',
    'EM_APURACAO',
    'ENCERRADO_RECUPERADO',
    'ENCERRADO_PERDA_TOTAL'
);

CREATE TABLE vehicle_incidents (
    id                  UUID                            PRIMARY KEY DEFAULT uuid_generate_v4(),
    vehicle_id          UUID                            NOT NULL REFERENCES vehicles(id) ON DELETE RESTRICT,
    tipo                vehicle_incident_type_enum      NOT NULL,
    status              vehicle_incident_status_enum    NOT NULL DEFAULT 'ABERTO',
    data_ocorrencia     TIMESTAMPTZ                     NOT NULL,
    local_ocorrencia    TEXT,
    numero_bo           TEXT                            NOT NULL,   -- Boletim de Ocorrência obrigatório (RF-AST-12)
    numero_seguradora   TEXT,
    apolice_id          UUID,                                       -- FK futura para apólices
    descricao           TEXT,
    notas_resolucao     TEXT,
    encerrado_em        TIMESTAMPTZ,
    encerrado_por       UUID,
    version             INTEGER                         NOT NULL DEFAULT 1,
    created_by          UUID,
    updated_by          UUID,
    created_at          TIMESTAMPTZ                     NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ                     NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vehicle_incidents_vehicle_id ON vehicle_incidents(vehicle_id);
CREATE INDEX idx_vehicle_incidents_status     ON vehicle_incidents(status) WHERE status != 'ENCERRADO_RECUPERADO' AND status != 'ENCERRADO_PERDA_TOTAL';

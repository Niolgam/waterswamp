-- ==========================================================================
-- Ticket 1.2 — RF-AST-09/10: Processo de Baixa e Desfazimento Patrimonial
--
-- RF-AST-09: Inicia o processo (INDISPONIVEL + depreciação suspensa).
-- RF-AST-10: Cada etapa com documento SEI. Conclusão retira o veículo da FSM ativa.
--
-- FSM: INICIADO → EM_ANDAMENTO → CONCLUIDO | CANCELADO
-- RN14: Decreto 9.373/2018 — cada etapa vinculada ao SEI.
-- ==========================================================================

CREATE TYPE disposal_status_enum AS ENUM (
    'INICIADO',
    'EM_ANDAMENTO',
    'CONCLUIDO',
    'CANCELADO'
);

CREATE TYPE disposal_destination_enum AS ENUM (
    'DOACAO',
    'LEILAO',
    'SUCATA',
    'TRANSFERENCIA_OUTRO_ORGAO',
    'OUTRO'
);

CREATE TABLE vehicle_disposal_processes (
    id                  UUID                        PRIMARY KEY DEFAULT uuid_generate_v4(),
    vehicle_id          UUID                        NOT NULL UNIQUE REFERENCES vehicles(id) ON DELETE RESTRICT,
    status              disposal_status_enum        NOT NULL DEFAULT 'INICIADO',
    destino             disposal_destination_enum   NOT NULL,
    justificativa       TEXT                        NOT NULL,
    numero_laudo        TEXT                        NOT NULL,   -- laudo obrigatório (RF-AST-09)
    documento_sei       TEXT,
    concluido_em        TIMESTAMPTZ,
    concluido_por       UUID,
    cancelado_em        TIMESTAMPTZ,
    cancelado_por       UUID,
    motivo_cancelamento TEXT,
    version             INTEGER                     NOT NULL DEFAULT 1,
    created_by          UUID,
    updated_by          UUID,
    created_at          TIMESTAMPTZ                 NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ                 NOT NULL DEFAULT NOW()
);

CREATE TABLE vehicle_disposal_steps (
    id              UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    disposal_id     UUID        NOT NULL REFERENCES vehicle_disposal_processes(id) ON DELETE CASCADE,
    descricao       TEXT        NOT NULL,
    documento_sei   TEXT        NOT NULL,   -- obrigatório por etapa (RF-AST-10)
    data_execucao   DATE        NOT NULL,
    responsavel_id  UUID,
    notes           TEXT,
    created_by      UUID,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vdp_vehicle_id ON vehicle_disposal_processes(vehicle_id);
CREATE INDEX idx_vdp_status     ON vehicle_disposal_processes(status) WHERE status NOT IN ('CONCLUIDO', 'CANCELADO');
CREATE INDEX idx_vds_disposal_id ON vehicle_disposal_steps(disposal_id);

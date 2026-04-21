-- ==========================================================================
-- Ticket 0.2 — Odômetro como Série Temporal Imutável (DRS v3.2, seção 4.3)
--
-- `leituras_hodometro` é append-only: nenhum UPDATE ou DELETE é permitido
-- via API. Cada leitura carrega sua fonte, status de validação e o
-- request_id para idempotência (seção 4.4).
-- ==========================================================================

CREATE TYPE leituras_hodometro_fonte_enum AS ENUM (
    'CHECKIN_GESTOR',
    'CHECKIN_CONDUTOR',
    'CHECKOUT_CONDUTOR',
    'ABASTECIMENTO_IMPORTACAO',
    'ABASTECIMENTO_MANUAL'
);

CREATE TYPE leituras_hodometro_status_enum AS ENUM (
    'VALIDADO',
    'QUARENTENA',
    'REJEITADO'
);

CREATE TABLE leituras_hodometro (
    id              UUID                            PRIMARY KEY DEFAULT uuid_generate_v4(),
    veiculo_id      UUID                            NOT NULL REFERENCES vehicles(id) ON DELETE RESTRICT,
    valor_km        NUMERIC(10, 1)                  NOT NULL CHECK (valor_km >= 0),
    fonte           leituras_hodometro_fonte_enum   NOT NULL,
    referencia_id   UUID,          -- FK para viagem, abastecimento ou OS (nullable até esses módulos existirem)
    coletado_em     TIMESTAMPTZ                     NOT NULL,
    registrado_em   TIMESTAMPTZ                     NOT NULL DEFAULT NOW(),
    status          leituras_hodometro_status_enum  NOT NULL DEFAULT 'VALIDADO',
    motivo_quarentena TEXT,
    request_id      UUID                            NOT NULL,
    version         INTEGER                         NOT NULL DEFAULT 1,
    created_by      UUID
);

-- Idempotência: mesmo request_id não pode registrar duas leituras distintas
CREATE UNIQUE INDEX idx_leituras_hodometro_request_id ON leituras_hodometro(request_id);

-- Consulta do Odômetro Projetado (maior valor_km VALIDADO por veículo)
CREATE INDEX idx_leituras_hodometro_veiculo_status ON leituras_hodometro(veiculo_id, status, valor_km DESC);

-- Quarentena: revisão pelo Gestor de Frota (RF-INS-03)
CREATE INDEX idx_leituras_hodometro_quarentena ON leituras_hodometro(veiculo_id) WHERE status = 'QUARENTENA';

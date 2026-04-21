-- ==========================================================================
-- Ticket 0.2 — Idempotência de Comandos (DRS v3.2, seção 4.4)
--
-- Armazena (request_id, resultado) para deduplicar retries de rede.
-- TTL de 24h: entradas expiradas são limpas por job periódico ou por
-- consulta com filtro `expires_at > NOW()`.
-- ==========================================================================

CREATE TABLE idempotency_keys (
    id              UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    request_id      UUID        NOT NULL UNIQUE,
    endpoint        TEXT        NOT NULL,           -- ex: "POST /fleet/vehicles/{id}/odometer"
    response_status INTEGER     NOT NULL,
    response_body   JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at      TIMESTAMPTZ NOT NULL GENERATED ALWAYS AS (created_at + INTERVAL '24 hours') STORED
);

CREATE INDEX idx_idempotency_keys_request_id  ON idempotency_keys(request_id);
CREATE INDEX idx_idempotency_keys_expires_at  ON idempotency_keys(expires_at);

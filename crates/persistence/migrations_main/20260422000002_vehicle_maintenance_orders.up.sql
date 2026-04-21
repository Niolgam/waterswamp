-- ==========================================================================
-- Épico 3 — RF-MNT: Manutenção Preventiva e Corretiva
--
-- RF-MNT-01: Abertura de OS — veículo → MANUTENCAO (RN-FSM-01: só se LIVRE).
-- RF-MNT-02: Ciclo de vida: ABERTA → EM_EXECUCAO → CONCLUIDA | CANCELADA.
--             Conclusão retorna veículo → ATIVO.
-- RF-MNT-03: Itens de serviço vinculados ao catálogo (fleet_maintenance_services).
-- RF-MNT-04: Custo previsto vs. real; link com fornecedor (oficina).
-- ==========================================================================

CREATE TYPE maintenance_order_status_enum AS ENUM (
    'ABERTA',
    'EM_EXECUCAO',
    'CONCLUIDA',
    'CANCELADA'
);

CREATE TYPE maintenance_order_type_enum AS ENUM (
    'PREVENTIVA',
    'CORRETIVA',
    'RECALL',
    'SINISTRO'  -- vinculada a um sinistro RF-AST-12
);

CREATE TABLE vehicle_maintenance_orders (
    id                          UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    vehicle_id                  UUID        NOT NULL REFERENCES vehicles(id) ON DELETE RESTRICT,
    tipo                        maintenance_order_type_enum NOT NULL,
    status                      maintenance_order_status_enum NOT NULL DEFAULT 'ABERTA',
    titulo                      TEXT        NOT NULL,
    descricao                   TEXT,
    fornecedor_id               UUID,       -- FK lógica para suppliers(id) — evita acoplamento rígido
    data_abertura               DATE        NOT NULL DEFAULT CURRENT_DATE,
    data_prevista_conclusao     DATE,
    data_conclusao              DATE,
    km_abertura                 BIGINT,
    custo_previsto              NUMERIC(15,2),
    custo_real                  NUMERIC(15,2),
    numero_os_externo           TEXT,       -- número da OS na oficina
    documento_sei               TEXT,
    incident_id                 UUID,       -- link com vehicle_incidents (opcional)
    notas                       TEXT,
    -- Conclusão / cancelamento
    concluido_por               UUID,
    cancelado_por               UUID,
    cancelado_em                TIMESTAMPTZ,
    motivo_cancelamento         TEXT,
    -- OCC
    version                     INTEGER     NOT NULL DEFAULT 1,
    created_by                  UUID,
    updated_by                  UUID,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE vehicle_maintenance_order_items (
    id              UUID            PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id        UUID            NOT NULL REFERENCES vehicle_maintenance_orders(id) ON DELETE CASCADE,
    service_id      UUID            REFERENCES fleet_maintenance_services(id) ON DELETE SET NULL,
    descricao       TEXT            NOT NULL,
    quantidade      NUMERIC(10,3)   NOT NULL DEFAULT 1,
    custo_unitario  NUMERIC(15,2),
    custo_total     NUMERIC(15,2) GENERATED ALWAYS AS (
                        CASE
                            WHEN custo_unitario IS NOT NULL
                            THEN quantidade * custo_unitario
                            ELSE NULL
                        END
                    ) STORED,
    created_by      UUID,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vmaint_vehicle_id  ON vehicle_maintenance_orders(vehicle_id);
CREATE INDEX idx_vmaint_status      ON vehicle_maintenance_orders(status) WHERE status NOT IN ('CONCLUIDA', 'CANCELADA');
CREATE INDEX idx_vmaint_fornecedor  ON vehicle_maintenance_orders(fornecedor_id) WHERE fornecedor_id IS NOT NULL;
CREATE INDEX idx_vmaint_incident    ON vehicle_maintenance_orders(incident_id) WHERE incident_id IS NOT NULL;
CREATE INDEX idx_vmaint_items_order ON vehicle_maintenance_order_items(order_id);

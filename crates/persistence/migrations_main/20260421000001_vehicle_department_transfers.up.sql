-- ==========================================================================
-- Ticket 1.1 — RF-AST-06: Histórico de Transferências Departamentais
--
-- Registra cada mudança de departamento de um veículo preservando o histórico
-- completo. O campo department_id em vehicles reflete sempre o departamento
-- atual; esta tabela guarda a cadeia de custódia (documento SEI obrigatório).
-- ==========================================================================

CREATE TABLE vehicle_department_transfers (
    id              UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    vehicle_id      UUID        NOT NULL REFERENCES vehicles(id) ON DELETE RESTRICT,
    dept_origem_id  UUID        REFERENCES organizational_units(id),
    dept_destino_id UUID        NOT NULL REFERENCES organizational_units(id),
    data_efetiva    DATE        NOT NULL,
    motivo          TEXT        NOT NULL,
    documento_sei   TEXT,       -- número/link do processo no SEI
    notes           TEXT,
    created_by      UUID,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vdt_vehicle_id  ON vehicle_department_transfers(vehicle_id);
CREATE INDEX idx_vdt_data_efetiva ON vehicle_department_transfers(vehicle_id, data_efetiva DESC);

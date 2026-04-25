-- ==========================================================================
-- Ticket 0.1 — FSM de Dois Eixos (DRS v3.2, seção 4.1.1)
--
-- Substitui o status singular por dois eixos independentes:
--   operational_status → aptidão operacional (ATIVO / MANUTENCAO / INDISPONIVEL)
--   allocation_status  → vínculo a viagens   (LIVRE / RESERVADO / EM_USO)
--
-- O campo legado `status` é MANTIDO nesta migration para compatibilidade com
-- o código existente. Será removido quando todos os consumidores migrarem.
-- ==========================================================================

-- ── Eixo 1: aptidão operacional ────────────────────────────────────────────
CREATE TYPE operational_status_enum AS ENUM (
    'ATIVO',
    'MANUTENCAO',
    'INDISPONIVEL'
);

-- ── Eixo 2: vínculo a viagens ──────────────────────────────────────────────
CREATE TYPE allocation_status_enum AS ENUM (
    'LIVRE',
    'RESERVADO',
    'EM_USO'
);

-- ── Adiciona os dois eixos à tabela de veículos ────────────────────────────
ALTER TABLE vehicles
    ADD COLUMN operational_status operational_status_enum NOT NULL DEFAULT 'ATIVO',
    ADD COLUMN allocation_status  allocation_status_enum  NOT NULL DEFAULT 'LIVRE';

-- ── Popula a partir do status legado ──────────────────────────────────────
UPDATE vehicles
SET
    operational_status = CASE status
        WHEN 'ACTIVE'          THEN 'ATIVO'::operational_status_enum
        WHEN 'IN_MAINTENANCE'  THEN 'MANUTENCAO'::operational_status_enum
        WHEN 'RESERVED'        THEN 'ATIVO'::operational_status_enum
        WHEN 'INACTIVE'        THEN 'INDISPONIVEL'::operational_status_enum
        WHEN 'DECOMMISSIONING' THEN 'INDISPONIVEL'::operational_status_enum
        ELSE                        'ATIVO'::operational_status_enum
    END,
    allocation_status = CASE status
        WHEN 'RESERVED' THEN 'RESERVADO'::allocation_status_enum
        ELSE                 'LIVRE'::allocation_status_enum
    END;

-- ── Índices obrigatórios para consultas de disponibilidade (RNF-12) ────────
CREATE INDEX idx_vehicles_operational_status
    ON vehicles (operational_status)
    WHERE is_deleted = false;

CREATE INDEX idx_vehicles_allocation_status
    ON vehicles (allocation_status)
    WHERE is_deleted = false;

-- Índice composto: filtro principal em RF-VIG-03 (veículos disponíveis)
CREATE INDEX idx_vehicles_status_combo
    ON vehicles (operational_status, allocation_status)
    WHERE is_deleted = false;

-- ── Atualiza histórico para registrar ambos os eixos ──────────────────────
-- As colunas legadas (old_status / new_status) são preservadas.
ALTER TABLE vehicle_status_history
    ADD COLUMN old_operational_status operational_status_enum,
    ADD COLUMN new_operational_status operational_status_enum,
    ADD COLUMN old_allocation_status  allocation_status_enum,
    ADD COLUMN new_allocation_status  allocation_status_enum;

-- ── FSM de Condutor (DRS 4.1.3, RF-CND-03) ───────────────────────────────
CREATE TYPE credenciamento_status_enum AS ENUM (
    'ATIVO',
    'SUSPENSO',
    'PENDENTE_VALIDACAO',
    'REVOGADO'
);

ALTER TABLE drivers
    ADD COLUMN credenciamento_status credenciamento_status_enum NOT NULL DEFAULT 'ATIVO';

CREATE INDEX idx_drivers_credenciamento_status
    ON drivers (credenciamento_status);

-- Índice composto para validação de alocação (RNF-12):
-- cnh_validade é usado em RN02 (validade até data de retorno)
CREATE INDEX idx_drivers_status_cnh
    ON drivers (credenciamento_status, cnh_expiration);

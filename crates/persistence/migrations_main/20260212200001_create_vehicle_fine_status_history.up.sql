-- =============================================================================
-- Novo enum de status da multa (substitui fine_payment_status_enum)
-- Cobre todo o ciclo de vida: notificação, defesa, pagamento
-- =============================================================================
CREATE TYPE fine_status_enum AS ENUM (
    'PENDING_NOTIFICATION',          -- Aguardando notificação
    'NOTIFIED',                      -- Notificado
    'AWAITING_DRIVER_IDENTIFICATION',-- Aguardando indicação de condutor
    'DRIVER_IDENTIFIED',             -- Condutor indicado
    'UNDER_PRIOR_DEFENSE',           -- Em defesa prévia
    'UNDER_APPEAL_FIRST',            -- Em recurso 1ª instância (JARI)
    'UNDER_APPEAL_SECOND',           -- Em recurso 2ª instância (CETRAN)
    'DEFENSE_ACCEPTED',              -- Defesa deferida / multa cancelada
    'DEFENSE_REJECTED',              -- Defesa indeferida
    'PENDING_PAYMENT',               -- Aguardando pagamento
    'PAID',                          -- Paga
    'OVERDUE',                       -- Vencida
    'CANCELLED'                      -- Cancelada
);

-- =============================================================================
-- Substituir payment_status por status na tabela vehicle_fines
-- =============================================================================

-- Adicionar nova coluna status
ALTER TABLE vehicle_fines ADD COLUMN status fine_status_enum;

-- Migrar dados existentes do antigo payment_status
UPDATE vehicle_fines SET status = CASE
    WHEN payment_status = 'PENDING' THEN 'PENDING_NOTIFICATION'::fine_status_enum
    WHEN payment_status = 'PAID' THEN 'PAID'::fine_status_enum
    WHEN payment_status = 'OVERDUE' THEN 'OVERDUE'::fine_status_enum
    WHEN payment_status = 'CANCELLED' THEN 'CANCELLED'::fine_status_enum
    WHEN payment_status = 'UNDER_APPEAL' THEN 'UNDER_APPEAL_FIRST'::fine_status_enum
    ELSE 'PENDING_NOTIFICATION'::fine_status_enum
END;

-- Tornar NOT NULL com default
ALTER TABLE vehicle_fines ALTER COLUMN status SET NOT NULL;
ALTER TABLE vehicle_fines ALTER COLUMN status SET DEFAULT 'PENDING_NOTIFICATION'::fine_status_enum;

-- Remover coluna antiga
ALTER TABLE vehicle_fines DROP COLUMN payment_status;

-- Remover enum antigo
DROP TYPE fine_payment_status_enum;

-- Índice para o novo status
DROP INDEX IF EXISTS idx_vehicle_fines_payment_status;
CREATE INDEX idx_vehicle_fines_status ON vehicle_fines (status) WHERE is_deleted = false;

-- =============================================================================
-- Tabela de histórico de status da multa
-- =============================================================================
CREATE TABLE vehicle_fine_status_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_fine_id UUID NOT NULL REFERENCES vehicle_fines(id) ON DELETE CASCADE,
    old_status fine_status_enum,
    new_status fine_status_enum NOT NULL,
    reason TEXT,
    changed_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vehicle_fine_status_history_fine ON vehicle_fine_status_history (vehicle_fine_id);
CREATE INDEX idx_vehicle_fine_status_history_date ON vehicle_fine_status_history (created_at);

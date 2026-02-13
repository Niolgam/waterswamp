-- Remover tabela de histórico
DROP TABLE IF EXISTS vehicle_fine_status_history;

-- Restaurar coluna payment_status
CREATE TYPE fine_payment_status_enum AS ENUM (
    'PENDING',
    'PAID',
    'OVERDUE',
    'CANCELLED',
    'UNDER_APPEAL'
);

ALTER TABLE vehicle_fines ADD COLUMN payment_status fine_payment_status_enum;

UPDATE vehicle_fines SET payment_status = CASE
    WHEN status = 'PAID' THEN 'PAID'::fine_payment_status_enum
    WHEN status = 'OVERDUE' THEN 'OVERDUE'::fine_payment_status_enum
    WHEN status = 'CANCELLED' THEN 'CANCELLED'::fine_payment_status_enum
    WHEN status = 'DEFENCE_ACCEPTED' THEN 'CANCELLED'::fine_payment_status_enum
    WHEN status IN ('UNDER_PRIOR_DEFENSE', 'UNDER_APPEAL_FIRST', 'UNDER_APPEAL_SECOND') THEN 'UNDER_APPEAL'::fine_payment_status_enum
    ELSE 'PENDING'::fine_payment_status_enum
END;

ALTER TABLE vehicle_fines ALTER COLUMN payment_status SET NOT NULL;
ALTER TABLE vehicle_fines ALTER COLUMN payment_status SET DEFAULT 'PENDING'::fine_payment_status_enum;

-- Remover coluna e enum novo
DROP INDEX IF EXISTS idx_vehicle_fines_status;
ALTER TABLE vehicle_fines DROP COLUMN status;
DROP TYPE fine_status_enum;

-- Restaurar índice antigo
CREATE INDEX idx_vehicle_fines_payment_status ON vehicle_fines (payment_status) WHERE is_deleted = false;

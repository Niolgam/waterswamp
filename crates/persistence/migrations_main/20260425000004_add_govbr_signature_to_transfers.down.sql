-- Nota: valores de enum do PostgreSQL não podem ser removidos sem recriar o tipo.
-- Esta migração reverte apenas as colunas adicionadas; o valor AWAITING_GOVBR_SIGNATURE
-- permanece no enum (não causa problemas — apenas não é mais usado).
ALTER TABLE stock_transfers
    DROP COLUMN IF EXISTS govbr_signed_by,
    DROP COLUMN IF EXISTS govbr_signed_at,
    DROP COLUMN IF EXISTS requires_govbr_signature;

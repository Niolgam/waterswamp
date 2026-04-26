-- Ticket 1.2 (RF-049): Assinatura Gov.br em transferências entre almoxarifados.
-- Quando requires_govbr_signature = TRUE, a confirmação do destino coloca
-- a transferência em AWAITING_GOVBR_SIGNATURE; o TRANSFER_IN só ocorre
-- após confirm-govbr-signature.

ALTER TYPE stock_transfer_status_enum ADD VALUE 'AWAITING_GOVBR_SIGNATURE' BEFORE 'CONFIRMED';

ALTER TABLE stock_transfers
    ADD COLUMN requires_govbr_signature BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN govbr_signed_at          TIMESTAMPTZ,
    ADD COLUMN govbr_signed_by          UUID REFERENCES users(id) ON DELETE RESTRICT;

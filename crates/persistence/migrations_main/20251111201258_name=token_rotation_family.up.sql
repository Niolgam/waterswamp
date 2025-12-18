-- Add up migration script here
ALTER TABLE refresh_tokens
ADD COLUMN family_id UUID,
ADD COLUMN parent_token_hash VARCHAR(64) NULL;

UPDATE refresh_tokens SET family_id = id WHERE family_id IS NULL;

ALTER TABLE refresh_tokens
ALTER COLUMN family_id SET NOT NULL;

CREATE INDEX idx_refresh_tokens_family_id ON refresh_tokens(family_id);

COMMENT ON COLUMN refresh_tokens.family_id IS 'ID da família de tokens (para detecção de roubo)';
COMMENT ON COLUMN refresh_tokens.parent_token_hash IS 'Hash do token que gerou este (para rastreio)';

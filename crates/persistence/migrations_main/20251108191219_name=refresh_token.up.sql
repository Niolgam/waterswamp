-- Add up migration script here
CREATE TABLE IF NOT EXISTS refresh_tokens (
    -- ID único do refresh token
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- ID do usuário dono do token
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Hash do refresh token (SHA-256)
    -- Nunca armazenamos o token em texto plano por segurança
    token_hash VARCHAR(64) NOT NULL UNIQUE,
    
    -- Data de expiração do token (após essa data, não pode ser usado)
    expires_at TIMESTAMPTZ NOT NULL,
    
    -- Flag para revogar token manualmente (ex: logout, mudança de senha)
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Timestamps de auditoria
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Índices para performance
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Índice para buscar tokens por usuário
CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);

-- Índice para buscar tokens por hash (usado em toda renovação)
CREATE INDEX idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);

-- Índice para limpar tokens expirados (job de limpeza periódica)
CREATE INDEX idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);

-- Índice composto para verificar tokens válidos
CREATE INDEX idx_refresh_tokens_valid ON refresh_tokens(token_hash, revoked, expires_at);

-- Trigger para atualizar updated_at automaticamente
CREATE OR REPLACE FUNCTION update_updated_at_column_refresh_tokens()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_timestamp_refresh_tokens
BEFORE UPDATE ON refresh_tokens
FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column_refresh_tokens();

-- Comentários para documentação
COMMENT ON TABLE refresh_tokens IS 'Armazena refresh tokens para renovação de JWT';
COMMENT ON COLUMN refresh_tokens.token_hash IS 'SHA-256 do refresh token (nunca texto plano)';
COMMENT ON COLUMN refresh_tokens.revoked IS 'Flag para revogar token manualmente (logout, etc)';
COMMENT ON COLUMN refresh_tokens.expires_at IS 'Data de expiração (tipicamente 7-30 dias após criação)';

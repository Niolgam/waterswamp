-- Add up migration script here
-- Adiciona a coluna de papel (role) com padrão 'user'
ALTER TABLE users ADD COLUMN IF NOT EXISTS role VARCHAR(50) NOT NULL DEFAULT 'user';

-- Adiciona a coluna de data de verificação do email
ALTER TABLE users ADD COLUMN IF NOT EXISTS email_verified_at TIMESTAMPTZ;

-- Cria índice para buscas por role
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);

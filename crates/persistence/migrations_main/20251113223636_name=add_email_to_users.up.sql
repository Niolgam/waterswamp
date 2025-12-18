-- Add up migration script here
ALTER TABLE users ADD COLUMN email VARCHAR(255);

-- 2. Define um email placeholder para utilizadores existentes (alice, bob, etc.)
--    Isto é crucial para que o passo 3 (NOT NULL) não falhe.
UPDATE users SET email = username || '@temp.example.com' WHERE email IS NULL;

-- 3. Agora que não há nulos, aplica as restrições
ALTER TABLE users ALTER COLUMN email SET NOT NULL;
ALTER TABLE users ADD CONSTRAINT users_email_unique UNIQUE (email);

-- 4. (Opcional mas recomendado) Adiciona um índice para logins/buscas
--    case-insensitive, que também garante unicidade
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email_lower_unique
ON users (LOWER(email));

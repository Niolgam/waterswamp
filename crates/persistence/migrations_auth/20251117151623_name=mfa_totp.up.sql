
ALTER TABLE users
ADD COLUMN mfa_enabled BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN mfa_secret VARCHAR(255) NULL,
ADD COLUMN mfa_backup_codes TEXT[] NULL;

CREATE TABLE mfa_setup_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    secret VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT fk_user_mfa_setup FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE mfa_backup_code_usage (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code_hash VARCHAR(64) NOT NULL,
    used_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address INET NULL,
    
    CONSTRAINT fk_user_backup_code FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_mfa_setup_user_id ON mfa_setup_tokens(user_id);
CREATE INDEX idx_mfa_setup_expires ON mfa_setup_tokens(expires_at);
CREATE INDEX idx_mfa_backup_usage_user ON mfa_backup_code_usage(user_id);
CREATE INDEX idx_users_mfa_enabled ON users(mfa_enabled) WHERE mfa_enabled = TRUE;

COMMENT ON COLUMN users.mfa_enabled IS 'Whether MFA is enabled for this user';
COMMENT ON COLUMN users.mfa_secret IS 'Encrypted TOTP secret (NULL if MFA not enabled)';
COMMENT ON COLUMN users.mfa_backup_codes IS 'Array of hashed backup codes';
COMMENT ON TABLE mfa_setup_tokens IS 'Temporary storage for MFA setup process';
COMMENT ON TABLE mfa_backup_code_usage IS 'Audit trail for backup code usage';

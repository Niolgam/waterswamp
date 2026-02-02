-- ============================================================================
-- Migration: Create Sessions Table for Cookie-Based Authentication
-- Description: Implements secure server-side session storage for HttpOnly cookies
-- ============================================================================

-- Sessions table: stores session data referenced by cookie session_id
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Session identifier (stored in HttpOnly cookie, used as lookup key)
    -- This is a cryptographically random token, NOT the primary key
    session_token_hash VARCHAR(64) NOT NULL UNIQUE,

    -- User association
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Session metadata
    user_agent TEXT,
    ip_address INET,

    -- Token data (encrypted access token for API calls)
    -- Stored encrypted, decrypted only when needed
    access_token_encrypted TEXT NOT NULL,

    -- Refresh token reference (links to existing refresh_tokens table)
    refresh_token_id UUID REFERENCES refresh_tokens(id) ON DELETE SET NULL,

    -- Session lifecycle
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Session state
    is_revoked BOOLEAN NOT NULL DEFAULT FALSE,
    revoked_at TIMESTAMPTZ,
    revoked_reason VARCHAR(100),

    -- CSRF token for form submissions (separate from session token)
    csrf_token_hash VARCHAR(64) NOT NULL
);

-- Index for fast session lookup by token hash
CREATE INDEX idx_sessions_token_hash ON sessions(session_token_hash)
    WHERE is_revoked = FALSE;

-- Index for user's active sessions
CREATE INDEX idx_sessions_user_active ON sessions(user_id, last_activity_at DESC)
    WHERE is_revoked = FALSE;

-- Index for cleanup of expired sessions
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at)
    WHERE is_revoked = FALSE;

-- Index for security audit (sessions by IP)
CREATE INDEX idx_sessions_ip ON sessions(ip_address, created_at DESC);

-- ============================================================================
-- Session Key Rotation Table
-- Stores encryption keys for cookie signing/encryption with rotation support
-- ============================================================================

CREATE TABLE IF NOT EXISTS session_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Key identifier (used in cookie to identify which key to use)
    key_id VARCHAR(16) NOT NULL UNIQUE,

    -- The actual key (encrypted with master key from environment)
    -- In production, this should be encrypted at rest
    key_material BYTEA NOT NULL,

    -- Key type: 'signing' for HMAC, 'encryption' for AES
    key_type VARCHAR(20) NOT NULL CHECK (key_type IN ('signing', 'encryption')),

    -- Key lifecycle
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    -- Rotation metadata
    rotated_from_id UUID REFERENCES session_keys(id),
    rotation_reason VARCHAR(100)
);

-- Only one active key per type at a time
CREATE UNIQUE INDEX idx_session_keys_active ON session_keys(key_type)
    WHERE is_active = TRUE;

-- ============================================================================
-- Functions for Session Management
-- ============================================================================

-- Function to clean up expired sessions
CREATE OR REPLACE FUNCTION fn_cleanup_expired_sessions()
RETURNS INTEGER
LANGUAGE plpgsql
AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM sessions
    WHERE expires_at < NOW()
       OR (is_revoked = TRUE AND revoked_at < NOW() - INTERVAL '7 days');

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$;

-- Function to revoke all sessions for a user (logout everywhere)
CREATE OR REPLACE FUNCTION fn_revoke_user_sessions(
    p_user_id UUID,
    p_reason VARCHAR(100) DEFAULT 'user_logout_all'
)
RETURNS INTEGER
LANGUAGE plpgsql
AS $$
DECLARE
    revoked_count INTEGER;
BEGIN
    UPDATE sessions
    SET is_revoked = TRUE,
        revoked_at = NOW(),
        revoked_reason = p_reason
    WHERE user_id = p_user_id
      AND is_revoked = FALSE;

    GET DIAGNOSTICS revoked_count = ROW_COUNT;
    RETURN revoked_count;
END;
$$;

-- Function to update last activity (sliding expiration)
CREATE OR REPLACE FUNCTION fn_touch_session(
    p_session_token_hash VARCHAR(64),
    p_extend_minutes INTEGER DEFAULT 30
)
RETURNS BOOLEAN
LANGUAGE plpgsql
AS $$
DECLARE
    session_found BOOLEAN;
BEGIN
    UPDATE sessions
    SET last_activity_at = NOW(),
        expires_at = GREATEST(expires_at, NOW() + (p_extend_minutes || ' minutes')::INTERVAL)
    WHERE session_token_hash = p_session_token_hash
      AND is_revoked = FALSE
      AND expires_at > NOW();

    GET DIAGNOSTICS session_found = ROW_COUNT;
    RETURN session_found > 0;
END;
$$;

-- ============================================================================
-- Comments
-- ============================================================================

COMMENT ON TABLE sessions IS 'Server-side session storage for HttpOnly cookie authentication';
COMMENT ON TABLE session_keys IS 'Encryption/signing keys for cookies with rotation support';
COMMENT ON COLUMN sessions.session_token_hash IS 'SHA-256 hash of the session token stored in cookie';
COMMENT ON COLUMN sessions.csrf_token_hash IS 'SHA-256 hash of CSRF token for form protection';
COMMENT ON COLUMN sessions.access_token_encrypted IS 'AES-256-GCM encrypted JWT for API calls';

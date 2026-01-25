-- Add ban fields to users table
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS is_banned BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS banned_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS banned_reason TEXT;

-- Create index for faster ban status lookups
CREATE INDEX IF NOT EXISTS idx_users_is_banned ON users(is_banned) WHERE is_banned = TRUE;

COMMENT ON COLUMN users.is_banned IS 'Indicates if the user is banned from accessing the system';
COMMENT ON COLUMN users.banned_at IS 'Timestamp when the user was banned';
COMMENT ON COLUMN users.banned_reason IS 'Reason for banning the user';

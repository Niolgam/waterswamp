CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS audit_logs (
    -- Unique identifier for each audit entry
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- User who performed the action (NULL for system actions or anonymous)
    user_id UUID NULL,
    
    -- Username at the time of action (denormalized for historical accuracy)
    username VARCHAR(100) NULL,
    
    -- Type of action performed
    action VARCHAR(50) NOT NULL,
    
    -- Resource/endpoint that was accessed
    resource VARCHAR(255) NOT NULL,
    
    -- HTTP method used
    method VARCHAR(10) NULL,
    
    -- HTTP status code of the response
    status_code INTEGER NULL,
    
    -- Additional details about the action (JSON)
    details JSONB NULL,
    
    -- IP address of the client
    ip_address INET NULL,
    
    -- User agent string
    user_agent TEXT NULL,
    
    -- Request ID for correlation
    request_id UUID NULL,
    
    -- Duration of the request in milliseconds
    duration_ms INTEGER NULL,
    
    -- Timestamp when the action occurred
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for querying by user
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);

-- Index for querying by action type
CREATE INDEX idx_audit_logs_action ON audit_logs(action);

-- Index for querying by resource
CREATE INDEX idx_audit_logs_resource ON audit_logs(resource);

-- Index for time-based queries (most common)
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);

-- Composite index for common filter combinations
CREATE INDEX idx_audit_logs_user_action_time ON audit_logs(user_id, action, created_at DESC);

-- Index for IP-based queries (security investigations)
CREATE INDEX idx_audit_logs_ip_address ON audit_logs(ip_address);

-- Index for request correlation
CREATE INDEX idx_audit_logs_request_id ON audit_logs(request_id);

-- Partial index for failed actions (security monitoring)
CREATE INDEX idx_audit_logs_failures ON audit_logs(created_at DESC) 
WHERE status_code >= 400;

-- Comments for documentation
COMMENT ON TABLE audit_logs IS 'Centralized audit trail for all security-relevant actions';
COMMENT ON COLUMN audit_logs.user_id IS 'UUID of user who performed action (NULL for anonymous/system)';
COMMENT ON COLUMN audit_logs.username IS 'Username at time of action (denormalized for historical accuracy)';
COMMENT ON COLUMN audit_logs.action IS 'Type of action: login, logout, create_user, update_user, delete_user, password_change, permission_change, etc.';
COMMENT ON COLUMN audit_logs.resource IS 'Resource/endpoint accessed (e.g., /api/admin/users, /login)';
COMMENT ON COLUMN audit_logs.method IS 'HTTP method (GET, POST, PUT, DELETE, etc.)';
COMMENT ON COLUMN audit_logs.status_code IS 'HTTP response status code';
COMMENT ON COLUMN audit_logs.details IS 'Additional context as JSON (e.g., changed fields, error messages)';
COMMENT ON COLUMN audit_logs.ip_address IS 'Client IP address (supports IPv4 and IPv6)';
COMMENT ON COLUMN audit_logs.user_agent IS 'Client user agent string';
COMMENT ON COLUMN audit_logs.request_id IS 'Unique request ID for distributed tracing';
COMMENT ON COLUMN audit_logs.duration_ms IS 'Request duration in milliseconds';

-- Create a function to automatically partition old logs (optional, for large deployments)
-- This can be extended to create time-based partitions
CREATE OR REPLACE FUNCTION cleanup_old_audit_logs(retention_days INTEGER DEFAULT 90)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM audit_logs 
    WHERE created_at < NOW() - (retention_days || ' days')::INTERVAL;
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_old_audit_logs IS 'Removes audit logs older than specified retention period (default 90 days)';

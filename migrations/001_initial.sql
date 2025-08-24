-- GitHub MCP Server Database Schema

-- Users table for storing GitHub user information
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    github_id INTEGER UNIQUE NOT NULL,
    username TEXT NOT NULL,
    name TEXT,
    email TEXT,
    avatar_url TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- GitHub tokens table with encryption
CREATE TABLE IF NOT EXISTS github_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    username TEXT NOT NULL,
    encrypted_token TEXT NOT NULL,
    encrypted_refresh_token TEXT,
    expires_at DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (github_id) ON DELETE CASCADE,
    UNIQUE(user_id)
);

-- CSRF tokens for OAuth flow security
CREATE TABLE IF NOT EXISTS csrf_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    token TEXT UNIQUE NOT NULL,
    expires_at DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Audit log for security monitoring
CREATE TABLE IF NOT EXISTS audit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER,
    action TEXT NOT NULL,
    resource TEXT,
    ip_address TEXT,
    user_agent TEXT,
    success BOOLEAN NOT NULL DEFAULT TRUE,
    error_message TEXT,
    metadata TEXT, -- JSON blob for additional data
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (github_id) ON DELETE SET NULL
);

-- Rate limiting tracking
CREATE TABLE IF NOT EXISTS rate_limits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ip_address TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 1,
    window_start DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(ip_address, endpoint, window_start)
);

-- Session management
CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    session_token TEXT UNIQUE NOT NULL,
    expires_at DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_used_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    ip_address TEXT,
    user_agent TEXT,
    FOREIGN KEY (user_id) REFERENCES users (github_id) ON DELETE CASCADE
);

-- Workflow state tracking
CREATE TABLE IF NOT EXISTS workflow_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    repository TEXT NOT NULL,
    branch TEXT NOT NULL,
    workflow_type TEXT NOT NULL, -- 'push', 'scan_tasks', 'merge'
    state TEXT NOT NULL, -- JSON blob
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (github_id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_github_tokens_user_id ON github_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_github_tokens_expires_at ON github_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_csrf_tokens_expires_at ON csrf_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_rate_limits_ip_endpoint ON rate_limits(ip_address, endpoint);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_workflow_states_user_repo ON workflow_states(user_id, repository);

-- Cleanup triggers for expired data
CREATE TRIGGER IF NOT EXISTS cleanup_expired_csrf_tokens
    AFTER INSERT ON csrf_tokens
    BEGIN
        DELETE FROM csrf_tokens WHERE expires_at < datetime('now');
    END;

CREATE TRIGGER IF NOT EXISTS cleanup_expired_sessions
    AFTER INSERT ON sessions
    BEGIN
        DELETE FROM sessions WHERE expires_at < datetime('now');
    END;

CREATE TRIGGER IF NOT EXISTS cleanup_old_audit_logs
    AFTER INSERT ON audit_logs
    BEGIN
        DELETE FROM audit_logs WHERE created_at < datetime('now', '-90 days');
    END;
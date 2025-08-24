use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub github: GitHubConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub api_base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub rate_limit_requests_per_minute: u32,
    pub session_timeout_hours: u64,
    pub max_token_age_days: u64,
    pub audit_log_enabled: bool,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Environment variable not found: {0}")]
    MissingEnvVar(String),
    #[error("Configuration parsing error: {0}")]
    ParseError(String),
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok(); // Load .env file if present

        let config = Config {
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8443".to_string())
                .parse()
                .map_err(|e| ConfigError::ParseError(format!("Invalid port: {}", e)))?,
            
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:./data/github-mcp-server.db".to_string()),
            
            jwt_secret: env::var("JWT_SECRET")
                .map_err(|_| ConfigError::MissingEnvVar("JWT_SECRET".to_string()))?,
            
            github: GitHubConfig {
                client_id: env::var("GITHUB_CLIENT_ID")
                    .map_err(|_| ConfigError::MissingEnvVar("GITHUB_CLIENT_ID".to_string()))?,
                client_secret: env::var("GITHUB_CLIENT_SECRET")
                    .map_err(|_| ConfigError::MissingEnvVar("GITHUB_CLIENT_SECRET".to_string()))?,
                redirect_uri: env::var("GITHUB_REDIRECT_URI")
                    .unwrap_or_else(|_| "https://localhost:8443/auth/github/callback".to_string()),
                api_base_url: env::var("GITHUB_API_BASE_URL")
                    .unwrap_or_else(|_| "https://api.github.com".to_string()),
            },
            
            security: SecurityConfig {
                rate_limit_requests_per_minute: env::var("RATE_LIMIT_RPM")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .map_err(|e| ConfigError::ParseError(format!("Invalid rate limit: {}", e)))?,
                session_timeout_hours: env::var("SESSION_TIMEOUT_HOURS")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()
                    .map_err(|e| ConfigError::ParseError(format!("Invalid session timeout: {}", e)))?,
                max_token_age_days: env::var("MAX_TOKEN_AGE_DAYS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .map_err(|e| ConfigError::ParseError(format!("Invalid token age: {}", e)))?,
                audit_log_enabled: env::var("AUDIT_LOG_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .map_err(|e| ConfigError::ParseError(format!("Invalid audit log setting: {}", e)))?,
            },
        };

        Ok(config)
    }
}
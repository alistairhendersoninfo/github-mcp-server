use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    
    #[error("OAuth2 error: {0}")]
    OAuth2(String),
    
    #[error("GitHub API error: {0}")]
    GitHubApi(String),
    
    #[error("MCP protocol error: {0}")]
    McpProtocol(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    #[error("Rate limit exceeded")]
    RateLimit,
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Configuration error: {0}")]
    Config(#[from] crate::config::ConfigError),
    
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            AppError::HttpClient(_) => (StatusCode::BAD_GATEWAY, "External service error"),
            AppError::Json(_) => (StatusCode::BAD_REQUEST, "Invalid JSON"),
            AppError::Jwt(_) => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AppError::OAuth2(_) => (StatusCode::UNAUTHORIZED, "OAuth2 error"),
            AppError::GitHubApi(_) => (StatusCode::BAD_GATEWAY, "GitHub API error"),
            AppError::McpProtocol(_) => (StatusCode::BAD_REQUEST, "MCP protocol error"),
            AppError::Authentication(_) => (StatusCode::UNAUTHORIZED, "Authentication failed"),
            AppError::Authorization(_) => (StatusCode::FORBIDDEN, "Access denied"),
            AppError::RateLimit => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded"),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, "Validation error"),
            AppError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(json!({
            "error": error_message,
            "message": self.to_string(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }));

        // Log the error for debugging
        tracing::error!("Application error: {}", self);

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
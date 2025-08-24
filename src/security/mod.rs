use axum::{
    http::{HeaderValue, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use governor::{Quota, RateLimiter};
use std::{
    collections::HashMap,
    net::IpAddr,
    num::NonZeroU32,
    sync::Arc,
    time::Duration,
};
use tokio::sync::RwLock;
use tower::{Layer, Service};
use tower_http::set_header::SetResponseHeaderLayer;
use tracing::{debug, warn};

use crate::error::{AppError, Result};

// Rate limiting state
type RateLimiterMap = Arc<RwLock<HashMap<IpAddr, Arc<RateLimiter<governor::state::direct::NotKeyed, governor::clock::DefaultClock, governor::state::InMemoryState>>>>>;

pub fn security_headers_layer() -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::overriding(
        axum::http::header::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    )
    // Add more security headers
    .and(SetResponseHeaderLayer::overriding(
        axum::http::header::HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    ))
    .and(SetResponseHeaderLayer::overriding(
        axum::http::header::HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    ))
    .and(SetResponseHeaderLayer::overriding(
        axum::http::header::HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    ))
    .and(SetResponseHeaderLayer::overriding(
        axum::http::header::HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static("default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"),
    ))
}

pub fn rate_limiting_layer() -> RateLimitingLayer {
    RateLimitingLayer::new(60) // 60 requests per minute
}

#[derive(Clone)]
pub struct RateLimitingLayer {
    requests_per_minute: u32,
    limiters: RateLimiterMap,
}

impl RateLimitingLayer {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            limiters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get_or_create_limiter(&self, ip: IpAddr) -> Arc<RateLimiter<governor::state::direct::NotKeyed, governor::clock::DefaultClock, governor::state::InMemoryState>> {
        let mut limiters = self.limiters.write().await;
        
        if let Some(limiter) = limiters.get(&ip) {
            return limiter.clone();
        }

        let quota = Quota::per_minute(NonZeroU32::new(self.requests_per_minute).unwrap());
        let limiter = Arc::new(RateLimiter::direct(quota));
        limiters.insert(ip, limiter.clone());
        
        limiter
    }
}

impl<S> Layer<S> for RateLimitingLayer {
    type Service = RateLimitingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitingService {
            inner,
            layer: self.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RateLimitingService<S> {
    inner: S,
    layer: RateLimitingLayer,
}

impl<S, B> Service<Request<B>> for RateLimitingService<S>
where
    S: Service<Request<B>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let mut inner = self.inner.clone();
        let layer = self.layer.clone();

        Box::pin(async move {
            // Extract client IP
            let client_ip = extract_client_ip(&req).unwrap_or_else(|| "127.0.0.1".parse().unwrap());
            
            // Get rate limiter for this IP
            let limiter = layer.get_or_create_limiter(client_ip).await;
            
            // Check rate limit
            match limiter.check() {
                Ok(_) => {
                    debug!("Rate limit check passed for IP: {}", client_ip);
                    inner.call(req).await
                }
                Err(_) => {
                    warn!("Rate limit exceeded for IP: {}", client_ip);
                    let response = Response::builder()
                        .status(StatusCode::TOO_MANY_REQUESTS)
                        .header("content-type", "application/json")
                        .body(axum::body::Body::from(
                            r#"{"error":"Rate limit exceeded","message":"Too many requests"}"#
                        ))
                        .unwrap();
                    Ok(response)
                }
            }
        })
    }
}

fn extract_client_ip<B>(req: &Request<B>) -> Option<IpAddr> {
    // Check X-Forwarded-For header (from reverse proxy)
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse() {
                    return Some(ip);
                }
            }
        }
    }

    // Check X-Real-IP header (from reverse proxy)
    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse() {
                return Some(ip);
            }
        }
    }

    // Fallback to connection info (not available in this context)
    None
}

pub async fn audit_log_middleware<B>(
    req: Request<B>,
    next: Next<B>,
) -> std::result::Result<Response, StatusCode> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let user_agent = req.headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    let start_time = std::time::Instant::now();
    
    debug!("Request: {} {} - User-Agent: {}", method, uri, user_agent);
    
    let response = next.run(req).await;
    
    let duration = start_time.elapsed();
    let status = response.status();
    
    // Log the request/response
    if status.is_server_error() {
        warn!("Request completed: {} {} - {} - {:?}", method, uri, status, duration);
    } else {
        debug!("Request completed: {} {} - {} - {:?}", method, uri, status, duration);
    }

    // TODO: Store audit log in database for security monitoring
    
    Ok(response)
}

pub fn validate_jwt_token(token: &str, secret: &str) -> Result<JwtClaims> {
    use jsonwebtoken::{decode, DecodingKey, Validation};

    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub user_id: u64,
    pub username: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn hash_password(password: &str) -> Result<String> {
    use argon2::{Argon2, PasswordHasher};
    use argon2::password_hash::{rand_core::OsRng, SaltString};

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))?;

    Ok(password_hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    use argon2::{Argon2, PasswordVerifier};
    use argon2::password_hash::PasswordHash;

    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Invalid password hash: {}", e)))?;

    let argon2 = Argon2::default();
    
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

// Secure token generation for CSRF and other purposes
pub fn generate_secure_token() -> String {
    use rand::Rng;
    
    let mut rng = rand::thread_rng();
    let token: [u8; 32] = rng.gen();
    hex::encode(token)
}

// Input validation helpers
pub fn validate_github_username(username: &str) -> bool {
    // GitHub username rules: alphanumeric and hyphens, 1-39 characters
    username.len() <= 39 
        && username.len() >= 1
        && username.chars().all(|c| c.is_alphanumeric() || c == '-')
        && !username.starts_with('-')
        && !username.ends_with('-')
}

pub fn validate_project_number(project_number: &str) -> bool {
    // Project numbers should be numeric and reasonable length
    project_number.len() <= 10 
        && project_number.chars().all(|c| c.is_ascii_digit())
        && !project_number.is_empty()
}

pub fn sanitize_branch_name(branch: &str) -> String {
    // Sanitize branch names to prevent injection attacks
    branch
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '/' || *c == '.')
        .collect::<String>()
        .trim_matches('.')
        .to_string()
}
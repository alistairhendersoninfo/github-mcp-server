use axum::{
    extract::{Query, State},
    response::{Html, Redirect},
    Json,
};
use oauth2::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    AuthUrl, TokenUrl, Scope, basic::BasicClient,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{info, error};

use crate::{AppState, error::{AppError, Result}};

#[derive(Debug, Deserialize)]
pub struct GitHubCallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenRefreshRequest {
    refresh_token: String,
}

pub async fn github_oauth_start(State(state): State<AppState>) -> Result<Redirect> {
    info!("Starting GitHub OAuth flow");

    let client = create_oauth_client(&state)?;
    
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("repo".to_string()))
        .add_scope(Scope::new("read:user".to_string()))
        .add_scope(Scope::new("read:project".to_string()))
        .url();

    // Store CSRF token in database for validation
    store_csrf_token(&state.db, csrf_token.secret()).await?;

    info!("Redirecting to GitHub OAuth: {}", auth_url);
    Ok(Redirect::to(auth_url.as_str()))
}

pub async fn github_oauth_callback(
    State(state): State<AppState>,
    Query(params): Query<GitHubCallbackQuery>,
) -> Result<Html<String>> {
    info!("GitHub OAuth callback received");

    // Check for OAuth errors
    if let Some(error) = params.error {
        let description = params.error_description.unwrap_or_else(|| "Unknown error".to_string());
        error!("OAuth error: {} - {}", error, description);
        return Ok(Html(create_error_page(&error, &description)));
    }

    let code = params.code.ok_or_else(|| {
        AppError::OAuth2("No authorization code received".to_string())
    })?;

    let csrf_state = params.state.ok_or_else(|| {
        AppError::OAuth2("No CSRF state received".to_string())
    })?;

    // Validate CSRF token
    if !validate_csrf_token(&state.db, &csrf_state).await? {
        return Err(AppError::OAuth2("Invalid CSRF state".to_string()));
    }

    let client = create_oauth_client(&state)?;
    
    // Exchange code for token
    let token_result = client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .map_err(|e| AppError::OAuth2(format!("Token exchange failed: {}", e)))?;

    let access_token = token_result.access_token().secret();
    let refresh_token = token_result.refresh_token().map(|t| t.secret());

    // Get user info from GitHub
    let github_client = crate::github::api::GitHubClient::new(
        access_token.clone(),
        Some(state.config.github.api_base_url.clone()),
    )?;
    
    let user = github_client.get_user().await?;
    info!("GitHub user authenticated: {}", user.login);

    // Store tokens in database
    store_github_token(
        &state.db,
        user.id,
        &user.login,
        access_token,
        refresh_token.as_deref(),
    ).await?;

    // Generate JWT for session
    let jwt_token = generate_jwt_token(&state.config.jwt_secret, user.id, &user.login)?;

    Ok(Html(create_success_page(&user.login, &jwt_token)))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(request): Json<TokenRefreshRequest>,
) -> Result<Json<Value>> {
    info!("Refreshing GitHub token");

    // TODO: Implement token refresh logic
    // 1. Validate refresh token
    // 2. Exchange for new access token
    // 3. Update database
    // 4. Return new JWT

    Ok(Json(json!({
        "status": "success",
        "message": "Token refresh not implemented yet"
    })))
}

fn create_oauth_client(state: &AppState) -> Result<BasicClient> {
    let client = BasicClient::new(
        ClientId::new(state.config.github.client_id.clone()),
        Some(ClientSecret::new(state.config.github.client_secret.clone())),
        AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
            .map_err(|e| AppError::OAuth2(format!("Invalid auth URL: {}", e)))?,
        Some(
            TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
                .map_err(|e| AppError::OAuth2(format!("Invalid token URL: {}", e)))?,
        ),
    )
    .set_redirect_uri(
        RedirectUrl::new(state.config.github.redirect_uri.clone())
            .map_err(|e| AppError::OAuth2(format!("Invalid redirect URI: {}", e)))?,
    );

    Ok(client)
}

async fn store_csrf_token(db: &sqlx::SqlitePool, token: &str) -> Result<()> {
    sqlx::query!(
        "INSERT INTO csrf_tokens (token, expires_at) VALUES (?, datetime('now', '+10 minutes'))",
        token
    )
    .execute(db)
    .await?;

    Ok(())
}

async fn validate_csrf_token(db: &sqlx::SqlitePool, token: &str) -> Result<bool> {
    let row = sqlx::query!(
        "SELECT COUNT(*) as count FROM csrf_tokens WHERE token = ? AND expires_at > datetime('now')",
        token
    )
    .fetch_one(db)
    .await?;

    // Clean up used token
    sqlx::query!("DELETE FROM csrf_tokens WHERE token = ?", token)
        .execute(db)
        .await?;

    Ok(row.count > 0)
}

async fn store_github_token(
    db: &sqlx::SqlitePool,
    user_id: u64,
    username: &str,
    access_token: &str,
    refresh_token: Option<&str>,
) -> Result<()> {
    // TODO: Encrypt tokens before storing
    let encrypted_access_token = encrypt_token(access_token)?;
    let encrypted_refresh_token = refresh_token.map(encrypt_token).transpose()?;

    sqlx::query!(
        r#"
        INSERT OR REPLACE INTO github_tokens 
        (user_id, username, encrypted_token, encrypted_refresh_token, expires_at, created_at, updated_at)
        VALUES (?, ?, ?, ?, datetime('now', '+30 days'), datetime('now'), datetime('now'))
        "#,
        user_id,
        username,
        encrypted_access_token,
        encrypted_refresh_token
    )
    .execute(db)
    .await?;

    Ok(())
}

fn encrypt_token(token: &str) -> Result<String> {
    // TODO: Implement proper token encryption using a secure key
    // For now, return the token as-is (NOT SECURE - implement proper encryption)
    Ok(token.to_string())
}

fn generate_jwt_token(secret: &str, user_id: u64, username: &str) -> Result<String> {
    use jsonwebtoken::{encode, Header, EncodingKey};
    use serde::{Serialize};

    #[derive(Serialize)]
    struct Claims {
        sub: String,
        user_id: u64,
        username: String,
        exp: usize,
        iat: usize,
    }

    let now = chrono::Utc::now();
    let exp = now + chrono::Duration::hours(24);

    let claims = Claims {
        sub: user_id.to_string(),
        user_id,
        username: username.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )?;

    Ok(token)
}

fn create_success_page(username: &str, jwt_token: &str) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>GitHub MCP Server - Authentication Success</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }}
        .success {{ color: #28a745; }}
        .token {{ background: #f8f9fa; padding: 10px; border-radius: 5px; font-family: monospace; word-break: break-all; }}
        .copy-btn {{ margin-top: 10px; padding: 5px 10px; background: #007bff; color: white; border: none; border-radius: 3px; cursor: pointer; }}
    </style>
</head>
<body>
    <h1 class="success">✅ Authentication Successful!</h1>
    <p>Welcome, <strong>{}</strong>! Your GitHub account has been successfully connected to the MCP server.</p>
    
    <h3>Your Session Token:</h3>
    <div class="token" id="token">{}</div>
    <button class="copy-btn" onclick="copyToken()">Copy Token</button>
    
    <h3>Next Steps:</h3>
    <ol>
        <li>Copy the token above</li>
        <li>Configure your Claude/Cursor client with this token</li>
        <li>Start using the GitHub workflow commands: <code>push</code>, <code>scan tasks</code>, <code>merge</code></li>
    </ol>
    
    <p><em>This token will expire in 24 hours. You can refresh it using the MCP server.</em></p>
    
    <script>
        function copyToken() {{
            const token = document.getElementById('token').textContent;
            navigator.clipboard.writeText(token).then(() => {{
                alert('Token copied to clipboard!');
            }});
        }}
    </script>
</body>
</html>
        "#,
        username, jwt_token
    )
}

fn create_error_page(error: &str, description: &str) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>GitHub MCP Server - Authentication Error</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }}
        .error {{ color: #dc3545; }}
    </style>
</head>
<body>
    <h1 class="error">❌ Authentication Failed</h1>
    <p><strong>Error:</strong> {}</p>
    <p><strong>Description:</strong> {}</p>
    
    <p><a href="/auth/github">Try again</a></p>
</body>
</html>
        "#,
        error, description
    )
}
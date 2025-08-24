use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT}};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, error};

use crate::{AppState, error::{AppError, Result}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: u64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub owner: GitHubUser,
    pub default_branch: String,
    pub clone_url: String,
    pub ssh_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub labels: Vec<GitHubLabel>,
    pub assignee: Option<GitHubUser>,
    pub user: GitHubUser,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubLabel {
    pub id: u64,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubPullRequest {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub draft: bool,
    pub head: GitHubBranch,
    pub base: GitHubBranch,
    pub user: GitHubUser,
    pub html_url: String,
    pub mergeable: Option<bool>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubBranch {
    pub label: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub sha: String,
    pub repo: GitHubRepository,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubProjectItem {
    pub id: String,
    pub content: Option<GitHubProjectContent>,
    pub field_values: Option<Vec<GitHubProjectFieldValue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubProjectContent {
    pub id: String,
    pub title: String,
    pub body: Option<String>,
    pub url: String,
    #[serde(rename = "type")]
    pub content_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubProjectFieldValue {
    pub field: GitHubProjectField,
    pub value: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubProjectField {
    pub id: String,
    pub name: String,
    #[serde(rename = "dataType")]
    pub data_type: String,
}

pub struct GitHubClient {
    client: Client,
    base_url: String,
    token: String,
}

impl GitHubClient {
    pub fn new(token: String, base_url: Option<String>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))
                .map_err(|e| AppError::Internal(format!("Invalid token format: {}", e)))?,
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("github-mcp-server/1.0"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| AppError::HttpClient(e))?;

        Ok(Self {
            client,
            base_url: base_url.unwrap_or_else(|| "https://api.github.com".to_string()),
            token,
        })
    }

    pub async fn get_user(&self) -> Result<GitHubUser> {
        let url = format!("{}/user", self.base_url);
        debug!("Fetching GitHub user: {}", url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(AppError::HttpClient)?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("GitHub API error: {} - {}", status, text);
            return Err(AppError::GitHubApi(format!("Failed to get user: {} - {}", status, text)));
        }

        let user = response.json::<GitHubUser>().await.map_err(AppError::HttpClient)?;
        Ok(user)
    }

    pub async fn get_repository(&self, owner: &str, repo: &str) -> Result<GitHubRepository> {
        let url = format!("{}/repos/{}/{}", self.base_url, owner, repo);
        debug!("Fetching repository: {}", url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(AppError::HttpClient)?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::GitHubApi(format!("Failed to get repository: {} - {}", status, text)));
        }

        let repository = response.json::<GitHubRepository>().await.map_err(AppError::HttpClient)?;
        Ok(repository)
    }

    pub async fn list_issues(&self, owner: &str, repo: &str, state: Option<&str>) -> Result<Vec<GitHubIssue>> {
        let mut url = format!("{}/repos/{}/{}/issues", self.base_url, owner, repo);
        if let Some(state) = state {
            url.push_str(&format!("?state={}", state));
        }
        
        debug!("Fetching issues: {}", url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(AppError::HttpClient)?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::GitHubApi(format!("Failed to list issues: {} - {}", status, text)));
        }

        let issues = response.json::<Vec<GitHubIssue>>().await.map_err(AppError::HttpClient)?;
        Ok(issues)
    }

    pub async fn create_issue(&self, owner: &str, repo: &str, title: &str, body: Option<&str>, labels: Option<Vec<&str>>) -> Result<GitHubIssue> {
        let url = format!("{}/repos/{}/{}/issues", self.base_url, owner, repo);
        debug!("Creating issue: {}", url);

        let mut payload = serde_json::json!({
            "title": title
        });

        if let Some(body) = body {
            payload["body"] = serde_json::Value::String(body.to_string());
        }

        if let Some(labels) = labels {
            payload["labels"] = serde_json::Value::Array(
                labels.into_iter().map(|l| serde_json::Value::String(l.to_string())).collect()
            );
        }

        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(AppError::HttpClient)?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::GitHubApi(format!("Failed to create issue: {} - {}", status, text)));
        }

        let issue = response.json::<GitHubIssue>().await.map_err(AppError::HttpClient)?;
        Ok(issue)
    }

    pub async fn list_pull_requests(&self, owner: &str, repo: &str, state: Option<&str>) -> Result<Vec<GitHubPullRequest>> {
        let mut url = format!("{}/repos/{}/{}/pulls", self.base_url, owner, repo);
        if let Some(state) = state {
            url.push_str(&format!("?state={}", state));
        }
        
        debug!("Fetching pull requests: {}", url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(AppError::HttpClient)?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::GitHubApi(format!("Failed to list pull requests: {} - {}", status, text)));
        }

        let prs = response.json::<Vec<GitHubPullRequest>>().await.map_err(AppError::HttpClient)?;
        Ok(prs)
    }

    pub async fn create_pull_request(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        head: &str,
        base: &str,
        body: Option<&str>,
        draft: bool,
    ) -> Result<GitHubPullRequest> {
        let url = format!("{}/repos/{}/{}/pulls", self.base_url, owner, repo);
        debug!("Creating pull request: {}", url);

        let mut payload = serde_json::json!({
            "title": title,
            "head": head,
            "base": base,
            "draft": draft
        });

        if let Some(body) = body {
            payload["body"] = serde_json::Value::String(body.to_string());
        }

        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(AppError::HttpClient)?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::GitHubApi(format!("Failed to create pull request: {} - {}", status, text)));
        }

        let pr = response.json::<GitHubPullRequest>().await.map_err(AppError::HttpClient)?;
        Ok(pr)
    }

    pub async fn get_project_items(&self, project_number: &str) -> Result<Vec<GitHubProjectItem>> {
        // Note: This is a simplified implementation
        // In practice, you'd use the GraphQL API for GitHub Projects v2
        let query = format!(r#"
            query {{
                organization(login: "your-org") {{
                    projectV2(number: {}) {{
                        items(first: 100) {{
                            nodes {{
                                id
                                content {{
                                    ... on Issue {{
                                        id
                                        title
                                        body
                                        url
                                    }}
                                    ... on PullRequest {{
                                        id
                                        title
                                        body
                                        url
                                    }}
                                }}
                                fieldValues(first: 20) {{
                                    nodes {{
                                        ... on ProjectV2ItemFieldTextValue {{
                                            field {{
                                                ... on ProjectV2Field {{
                                                    id
                                                    name
                                                    dataType
                                                }}
                                            }}
                                            text
                                        }}
                                        ... on ProjectV2ItemFieldSingleSelectValue {{
                                            field {{
                                                ... on ProjectV2SingleSelectField {{
                                                    id
                                                    name
                                                    dataType
                                                }}
                                            }}
                                            name
                                        }}
                                    }}
                                }}
                            }}
                        }}
                    }}
                }}
            }}
        "#, project_number);

        let url = format!("{}/graphql", self.base_url);
        let payload = serde_json::json!({ "query": query });

        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(AppError::HttpClient)?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::GitHubApi(format!("Failed to get project items: {} - {}", status, text)));
        }

        // Parse GraphQL response and extract project items
        let response_data: Value = response.json().await.map_err(AppError::HttpClient)?;
        
        // This is a simplified parsing - in practice you'd need more robust GraphQL response handling
        let items = vec![]; // TODO: Parse actual GraphQL response
        
        Ok(items)
    }
}

pub async fn get_github_client(state: AppState, user_id: Option<u64>) -> Result<GitHubClient> {
    // Get GitHub token from database for the user
    let token = if let Some(user_id) = user_id {
        get_user_github_token(&state.db, user_id).await?
    } else {
        // For now, use a default token or return an error
        return Err(AppError::Authentication("No GitHub token available".to_string()));
    };

    GitHubClient::new(token, Some(state.config.github.api_base_url.clone()))
}

async fn get_user_github_token(db: &sqlx::SqlitePool, user_id: u64) -> Result<String> {
    let row = sqlx::query!(
        "SELECT encrypted_token FROM github_tokens WHERE user_id = ? AND expires_at > datetime('now')",
        user_id
    )
    .fetch_optional(db)
    .await?;

    match row {
        Some(row) => {
            // TODO: Decrypt the token
            let token = decrypt_token(&row.encrypted_token)?;
            Ok(token)
        }
        None => Err(AppError::Authentication("No valid GitHub token found".to_string())),
    }
}

fn decrypt_token(encrypted_token: &str) -> Result<String> {
    // TODO: Implement proper token decryption
    // For now, assume the token is stored in plain text (NOT SECURE - implement proper encryption)
    Ok(encrypted_token.to_string())
}
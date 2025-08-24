pub mod api;
pub mod workflows;

use axum::{
    extract::State,
    Json,
};
use serde_json::Value;

use crate::{AppState, error::Result, mcp::protocol::GitHubCommand};

pub async fn handle_push(State(state): State<AppState>) -> Result<Json<Value>> {
    let command = GitHubCommand::Push {
        branch: None,
        message: None,
        ready_for_review: None,
    };
    let result = execute_workflow_command(state, command).await?;
    Ok(Json(result))
}

pub async fn handle_scan_tasks(State(state): State<AppState>) -> Result<Json<Value>> {
    let command = GitHubCommand::ScanTasks {
        project_number: None,
        filter_type: None,
        status: None,
    };
    let result = execute_workflow_command(state, command).await?;
    Ok(Json(result))
}

pub async fn handle_merge(State(state): State<AppState>) -> Result<Json<Value>> {
    let command = GitHubCommand::Merge {
        branch: None,
        delete_branch: Some(true),
        cleanup_work_folder: None,
    };
    let result = execute_workflow_command(state, command).await?;
    Ok(Json(result))
}

pub async fn execute_workflow_command(state: AppState, command: GitHubCommand) -> Result<Value> {
    workflows::execute_command(state, command).await
}

pub async fn get_workflow_status(state: AppState) -> Result<Value> {
    workflows::get_status(state).await
}

pub async fn get_project_tasks(state: AppState) -> Result<Value> {
    workflows::get_tasks(state).await
}
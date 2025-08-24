use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::{AppState, error::{AppError, Result}};
use super::protocol::{
    McpRequest, McpResponse, McpTool, McpResource, ServerCapabilities,
    methods, error_codes, GitHubCommand, MCP_VERSION
};

pub async fn handle_request(state: AppState, request: McpRequest) -> Result<serde_json::Value> {
    debug!("Handling MCP request: method={}", request.method);

    let response = match request.method.as_str() {
        methods::INITIALIZE => handle_initialize(&request).await?,
        methods::TOOLS_LIST => handle_tools_list(&request).await?,
        methods::TOOLS_CALL => handle_tools_call(state, &request).await?,
        methods::RESOURCES_LIST => handle_resources_list(&request).await?,
        methods::RESOURCES_READ => handle_resources_read(state, &request).await?,
        methods::GITHUB_PUSH => handle_github_push(state, &request).await?,
        methods::GITHUB_SCAN_TASKS => handle_github_scan_tasks(state, &request).await?,
        methods::GITHUB_MERGE => handle_github_merge(state, &request).await?,
        _ => McpResponse::error(
            request.id,
            error_codes::METHOD_NOT_FOUND,
            format!("Method not found: {}", request.method),
            None,
        ),
    };

    Ok(serde_json::to_value(response)?)
}

pub async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    
    info!("WebSocket connection established");

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                debug!("Received WebSocket message: {}", text);
                
                match serde_json::from_str::<McpRequest>(&text) {
                    Ok(request) => {
                        match handle_request(state.clone(), request).await {
                            Ok(response) => {
                                if let Ok(response_text) = serde_json::to_string(&response) {
                                    if sender.send(Message::Text(response_text)).await.is_err() {
                                        error!("Failed to send WebSocket response");
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Error handling WebSocket request: {}", e);
                                let error_response = McpResponse::error(
                                    None,
                                    error_codes::INTERNAL_ERROR,
                                    e.to_string(),
                                    None,
                                );
                                if let Ok(error_text) = serde_json::to_string(&error_response) {
                                    let _ = sender.send(Message::Text(error_text)).await;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse WebSocket message: {}", e);
                        let error_response = McpResponse::error(
                            None,
                            error_codes::PARSE_ERROR,
                            "Invalid JSON".to_string(),
                            None,
                        );
                        if let Ok(error_text) = serde_json::to_string(&error_response) {
                            let _ = sender.send(Message::Text(error_text)).await;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket connection closed");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

async fn handle_initialize(request: &McpRequest) -> Result<McpResponse> {
    let result = json!({
        "protocolVersion": MCP_VERSION,
        "capabilities": ServerCapabilities::default(),
        "serverInfo": {
            "name": "github-mcp-server",
            "version": env!("CARGO_PKG_VERSION")
        }
    });

    Ok(McpResponse::success(request.id.clone(), result))
}

async fn handle_tools_list(request: &McpRequest) -> Result<McpResponse> {
    let tools = vec![
        McpTool {
            name: "github_push".to_string(),
            description: "Intelligent git push with PR management and workflow automation".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "branch": {
                        "type": "string",
                        "description": "Branch to push (defaults to current branch)"
                    },
                    "message": {
                        "type": "string",
                        "description": "Optional commit message if changes need to be committed"
                    },
                    "ready_for_review": {
                        "type": "boolean",
                        "description": "Mark PR as ready for review after push"
                    }
                }
            }),
        },
        McpTool {
            name: "github_scan_tasks".to_string(),
            description: "Scan GitHub Projects for tasks and present organized by type/priority".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_number": {
                        "type": "string",
                        "description": "GitHub Project number (optional, will auto-detect from TODO.md)"
                    },
                    "filter_type": {
                        "type": "string",
                        "enum": ["bug", "feature", "enhancement", "documentation", "refactor", "test", "chore"],
                        "description": "Filter tasks by type"
                    },
                    "status": {
                        "type": "string",
                        "description": "Filter tasks by status (In Progress, To Do, etc.)"
                    }
                }
            }),
        },
        McpTool {
            name: "github_merge".to_string(),
            description: "Complete merge workflow with tests, cleanup, and project updates".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "branch": {
                        "type": "string",
                        "description": "Branch to merge (defaults to current branch)"
                    },
                    "delete_branch": {
                        "type": "boolean",
                        "description": "Delete branch after merge (default: true)"
                    },
                    "cleanup_work_folder": {
                        "type": "boolean",
                        "description": "Clean up work folder after merge (default: ask user)"
                    }
                }
            }),
        },
    ];

    let result = json!({ "tools": tools });
    Ok(McpResponse::success(request.id.clone(), result))
}

async fn handle_tools_call(state: AppState, request: &McpRequest) -> Result<McpResponse> {
    let params = request.params.as_ref().ok_or_else(|| {
        AppError::McpProtocol("Missing parameters for tools/call".to_string())
    })?;

    let tool_name = params.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
        AppError::McpProtocol("Missing tool name".to_string())
    })?;

    let arguments = params.get("arguments").unwrap_or(&json!({}));

    let result = match tool_name {
        "github_push" => {
            let command = serde_json::from_value::<GitHubCommand>(json!({
                "Push": {
                    "branch": arguments.get("branch"),
                    "message": arguments.get("message"),
                    "ready_for_review": arguments.get("ready_for_review")
                }
            }))?;
            crate::github::execute_workflow_command(state, command).await?
        }
        "github_scan_tasks" => {
            let command = serde_json::from_value::<GitHubCommand>(json!({
                "ScanTasks": {
                    "project_number": arguments.get("project_number"),
                    "filter_type": arguments.get("filter_type"),
                    "status": arguments.get("status")
                }
            }))?;
            crate::github::execute_workflow_command(state, command).await?
        }
        "github_merge" => {
            let command = serde_json::from_value::<GitHubCommand>(json!({
                "Merge": {
                    "branch": arguments.get("branch"),
                    "delete_branch": arguments.get("delete_branch"),
                    "cleanup_work_folder": arguments.get("cleanup_work_folder")
                }
            }))?;
            crate::github::execute_workflow_command(state, command).await?
        }
        _ => {
            return Ok(McpResponse::error(
                request.id.clone(),
                error_codes::METHOD_NOT_FOUND,
                format!("Unknown tool: {}", tool_name),
                None,
            ));
        }
    };

    Ok(McpResponse::success(request.id.clone(), result))
}

async fn handle_resources_list(request: &McpRequest) -> Result<McpResponse> {
    let resources = vec![
        McpResource {
            uri: "github://workflow/status".to_string(),
            name: "Workflow Status".to_string(),
            description: Some("Current GitHub workflow status and active tasks".to_string()),
            mime_type: Some("application/json".to_string()),
        },
        McpResource {
            uri: "github://projects/tasks".to_string(),
            name: "Project Tasks".to_string(),
            description: Some("GitHub Project tasks with current status".to_string()),
            mime_type: Some("application/json".to_string()),
        },
    ];

    let result = json!({ "resources": resources });
    Ok(McpResponse::success(request.id.clone(), result))
}

async fn handle_resources_read(state: AppState, request: &McpRequest) -> Result<McpResponse> {
    let params = request.params.as_ref().ok_or_else(|| {
        AppError::McpProtocol("Missing parameters for resources/read".to_string())
    })?;

    let uri = params.get("uri").and_then(|v| v.as_str()).ok_or_else(|| {
        AppError::McpProtocol("Missing URI for resources/read".to_string())
    })?;

    let content = match uri {
        "github://workflow/status" => {
            crate::github::get_workflow_status(state).await?
        }
        "github://projects/tasks" => {
            crate::github::get_project_tasks(state).await?
        }
        _ => {
            return Ok(McpResponse::error(
                request.id.clone(),
                error_codes::METHOD_NOT_FOUND,
                format!("Unknown resource: {}", uri),
                None,
            ));
        }
    };

    let result = json!({
        "contents": [{
            "uri": uri,
            "mimeType": "application/json",
            "text": serde_json::to_string_pretty(&content)?
        }]
    });

    Ok(McpResponse::success(request.id.clone(), result))
}

async fn handle_github_push(state: AppState, request: &McpRequest) -> Result<McpResponse> {
    let params = request.params.as_ref().unwrap_or(&json!({}));
    
    let command = GitHubCommand::Push {
        branch: params.get("branch").and_then(|v| v.as_str()).map(String::from),
        message: params.get("message").and_then(|v| v.as_str()).map(String::from),
        ready_for_review: params.get("ready_for_review").and_then(|v| v.as_bool()),
    };

    let result = crate::github::execute_workflow_command(state, command).await?;
    Ok(McpResponse::success(request.id.clone(), result))
}

async fn handle_github_scan_tasks(state: AppState, request: &McpRequest) -> Result<McpResponse> {
    let params = request.params.as_ref().unwrap_or(&json!({}));
    
    let command = GitHubCommand::ScanTasks {
        project_number: params.get("project_number").and_then(|v| v.as_str()).map(String::from),
        filter_type: params.get("filter_type").and_then(|v| v.as_str()).map(String::from),
        status: params.get("status").and_then(|v| v.as_str()).map(String::from),
    };

    let result = crate::github::execute_workflow_command(state, command).await?;
    Ok(McpResponse::success(request.id.clone(), result))
}

async fn handle_github_merge(state: AppState, request: &McpRequest) -> Result<McpResponse> {
    let params = request.params.as_ref().unwrap_or(&json!({}));
    
    let command = GitHubCommand::Merge {
        branch: params.get("branch").and_then(|v| v.as_str()).map(String::from),
        delete_branch: params.get("delete_branch").and_then(|v| v.as_bool()),
        cleanup_work_folder: params.get("cleanup_work_folder").and_then(|v| v.as_bool()),
    };

    let result = crate::github::execute_workflow_command(state, command).await?;
    Ok(McpResponse::success(request.id.clone(), result))
}
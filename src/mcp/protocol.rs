use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// MCP Protocol Version
pub const MCP_VERSION: &str = "2024-11-05";

/// MCP Request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

/// MCP Response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

/// MCP Error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// MCP Resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// GitHub workflow commands supported by this MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GitHubCommand {
    Push {
        branch: Option<String>,
        message: Option<String>,
        ready_for_review: Option<bool>,
    },
    ScanTasks {
        project_number: Option<String>,
        filter_type: Option<String>, // "bug", "feature", "enhancement"
        status: Option<String>,      // "In Progress", "To Do", etc.
    },
    Merge {
        branch: Option<String>,
        delete_branch: Option<bool>,
        cleanup_work_folder: Option<bool>,
    },
}

impl McpResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, code: i32, message: String, data: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(McpError { code, message, data }),
        }
    }
}

/// Standard MCP error codes
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    
    // Custom error codes for GitHub MCP server
    pub const GITHUB_API_ERROR: i32 = -32000;
    pub const AUTHENTICATION_ERROR: i32 = -32001;
    pub const RATE_LIMIT_ERROR: i32 = -32002;
    pub const WORKFLOW_ERROR: i32 = -32003;
}

/// MCP method names
pub mod methods {
    pub const INITIALIZE: &str = "initialize";
    pub const TOOLS_LIST: &str = "tools/list";
    pub const TOOLS_CALL: &str = "tools/call";
    pub const RESOURCES_LIST: &str = "resources/list";
    pub const RESOURCES_READ: &str = "resources/read";
    pub const NOTIFICATIONS_INITIALIZED: &str = "notifications/initialized";
    
    // Custom GitHub workflow methods
    pub const GITHUB_PUSH: &str = "github/push";
    pub const GITHUB_SCAN_TASKS: &str = "github/scan-tasks";
    pub const GITHUB_MERGE: &str = "github/merge";
}

/// Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub tools: Option<ToolsCapability>,
    pub resources: Option<ResourcesCapability>,
    pub logging: Option<LoggingCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    pub subscribe: Option<bool>,
    #[serde(rename = "listChanged")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingCapability {
    pub level: Option<String>,
}

impl Default for ServerCapabilities {
    fn default() -> Self {
        Self {
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
            resources: Some(ResourcesCapability {
                subscribe: Some(false),
                list_changed: Some(true),
            }),
            logging: Some(LoggingCapability {
                level: Some("info".to_string()),
            }),
        }
    }
}
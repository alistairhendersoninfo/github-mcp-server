pub mod protocol;
pub mod handlers;

use axum::{
    extract::{State, WebSocketUpgrade},
    response::Response,
    Json,
};
use serde_json::Value;

use crate::{AppState, error::Result};
use protocol::McpRequest;

pub async fn handle_mcp_request(
    State(state): State<AppState>,
    Json(request): Json<McpRequest>,
) -> Result<Json<Value>> {
    handlers::handle_request(state, request).await
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handlers::handle_websocket(socket, state))
}
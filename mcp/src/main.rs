//! Tytus MCP Server — stdio-based Model Context Protocol server.
//!
//! Exposes pod management, status, and AI gateway info as MCP tools so any
//! MCP-compatible AI CLI (Claude Code, Kilocode, OpenCode, Archon, etc.)
//! can natively manage Tytus pods without shelling out.
//!
//! Protocol: JSON-RPC 2.0 over stdin/stdout (MCP stdio transport).

mod state;
mod tools;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write as IoWrite};

const SERVER_NAME: &str = "tytus";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

// ── JSON-RPC types ──────────────────────────────────────────

#[derive(Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Serialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

impl JsonRpcResponse {
    fn success(id: Value, result: Value) -> Self {
        Self { jsonrpc: "2.0".into(), id, result: Some(result), error: None }
    }
    fn error(id: Value, code: i64, message: String) -> Self {
        Self { jsonrpc: "2.0".into(), id, result: None, error: Some(JsonRpcError { code, message }) }
    }
}

// ── MCP types ───────────────────────────────────────────────

#[derive(Serialize)]
struct ToolInfo {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

#[derive(Serialize)]
struct ToolResult {
    content: Vec<ContentBlock>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    is_error: Option<bool>,
}

#[derive(Serialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

impl ToolResult {
    fn text(s: String) -> Self {
        Self {
            content: vec![ContentBlock { content_type: "text".into(), text: s }],
            is_error: None,
        }
    }
    fn error(s: String) -> Self {
        Self {
            content: vec![ContentBlock { content_type: "text".into(), text: s }],
            is_error: Some(true),
        }
    }
}

// ── Tool definitions ────────────────────────────────────────

fn tool_definitions() -> Vec<ToolInfo> {
    vec![
        ToolInfo {
            name: "tytus_status".into(),
            description: "Get current Tytus status: login state, plan tier, active pods with endpoints and API keys. Use this first to check if the user is connected.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        ToolInfo {
            name: "tytus_env".into(),
            description: "Get connection environment variables for a specific pod (AI gateway URL, API key, agent endpoint). Returns values ready to use with curl or any OpenAI-compatible client.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pod_id": {
                        "type": "string",
                        "description": "Pod ID (e.g. '01'). Omit for first available pod."
                    }
                },
                "required": []
            }),
        },
        ToolInfo {
            name: "tytus_models".into(),
            description: "List available AI models on the connected pod's gateway. Requires an active tunnel. Returns model IDs that can be used with the OpenAI-compatible API.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pod_id": {
                        "type": "string",
                        "description": "Pod ID. Omit for first available pod."
                    }
                },
                "required": []
            }),
        },
        ToolInfo {
            name: "tytus_chat".into(),
            description: "Send a chat completion request to the private AI gateway. Uses the OpenAI-compatible API on the connected pod. Requires an active tunnel (run `sudo tytus connect` first).".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "model": {
                        "type": "string",
                        "description": "Model ID (e.g. 'qwen3-8b', 'llama-3.1-8b-instruct'). Run tytus_models to see available models."
                    },
                    "messages": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "role": { "type": "string", "enum": ["system", "user", "assistant"] },
                                "content": { "type": "string" }
                            },
                            "required": ["role", "content"]
                        },
                        "description": "Chat messages array"
                    },
                    "max_tokens": {
                        "type": "integer",
                        "description": "Max tokens to generate (default: 1024)"
                    },
                    "temperature": {
                        "type": "number",
                        "description": "Sampling temperature (default: 0.7)"
                    },
                    "pod_id": {
                        "type": "string",
                        "description": "Pod ID. Omit for first available pod."
                    }
                },
                "required": ["model", "messages"]
            }),
        },
        ToolInfo {
            name: "tytus_revoke".into(),
            description: "Revoke (release) a specific pod, freeing its units for reallocation. The pod's tunnel must be disconnected first.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pod_id": {
                        "type": "string",
                        "description": "Pod ID to revoke (e.g. '01')"
                    }
                },
                "required": ["pod_id"]
            }),
        },
        ToolInfo {
            name: "tytus_setup_guide".into(),
            description: "Get setup instructions for Tytus. Use when the user is not logged in or has no active tunnel. Returns step-by-step instructions.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
    ]
}

// ── Main event loop ─────────────────────────────────────────

#[tokio::main]
async fn main() {
    // MCP servers MUST NOT write to stdout except JSON-RPC responses
    // Use stderr for logging
    tracing_subscriber::fmt()
        .with_env_filter("warn")
        .with_writer(io::stderr)
        .with_target(false)
        .init();

    let stdin = io::stdin();
    let stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() { continue; }

        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = JsonRpcResponse::error(
                    Value::Null, -32700, format!("Parse error: {}", e),
                );
                write_response(&stdout, &resp);
                continue;
            }
        };

        let id = req.id.clone().unwrap_or(Value::Null);
        let resp = handle_request(req).await;
        let resp = match resp {
            Ok(result) => JsonRpcResponse::success(id, result),
            Err(e) => JsonRpcResponse::error(id, -32603, e),
        };
        write_response(&stdout, &resp);
    }
}

fn write_response(stdout: &io::Stdout, resp: &JsonRpcResponse) {
    let mut handle = stdout.lock();
    let _ = serde_json::to_writer(&mut handle, resp);
    let _ = handle.write_all(b"\n");
    let _ = handle.flush();
}

async fn handle_request(req: JsonRpcRequest) -> Result<Value, String> {
    match req.method.as_str() {
        "initialize" => {
            Ok(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": SERVER_NAME,
                    "version": SERVER_VERSION
                }
            }))
        }
        "notifications/initialized" => {
            // Client acknowledged init — no response needed for notifications
            Ok(Value::Null)
        }
        "tools/list" => {
            let tools = tool_definitions();
            Ok(serde_json::json!({ "tools": tools }))
        }
        "tools/call" => {
            let params = req.params.unwrap_or(Value::Null);
            let tool_name = params.get("name")
                .and_then(|v| v.as_str())
                .ok_or("Missing tool name")?
                .to_string();
            let arguments = params.get("arguments")
                .cloned()
                .unwrap_or(serde_json::json!({}));

            let result = tools::call_tool(&tool_name, arguments).await;
            Ok(serde_json::to_value(result).unwrap_or(Value::Null))
        }
        "ping" => Ok(serde_json::json!({})),
        _ => Err(format!("Unknown method: {}", req.method)),
    }
}

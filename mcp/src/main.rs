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
            name: "tytus_docs".into(),
            description: "Return the comprehensive LLM-facing reference for tytus-cli (same content as `tytus llm-docs`). Read this BEFORE driving any other tytus operation in a fresh session — it covers the command surface, agent types (nemoclaw=1u, hermes=2u), plan tiers, the only available models (ail-compound, ail-image, ail-embed, minimax/ail-compound, minimax/ail-image), the stable URL/key model, and the standard recipes. Cache the output in your context for the rest of the session.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        ToolInfo {
            name: "tytus_status".into(),
            description: "Return the current state of the user's Tytus account: signed-in email, subscription plan tier (Explorer/Creator/Operator), active pods with their pod_id, droplet_id, agent_type, tunnel state, and the stable user key + stable AI endpoint. Always call this first in any new conversation to find out what the user actually has — branch on the result instead of guessing.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        ToolInfo {
            name: "tytus_env".into(),
            description: "Return the connection environment variables for a pod. Default output is the STABLE pair: OPENAI_BASE_URL=http://10.42.42.1:18080/v1 and OPENAI_API_KEY=sk-tytus-user-<32hex>. These values are constant across pod revoke/reallocate cycles. Use these in any user-visible config file. The legacy per-pod values (10.18.X.Y + sk-<pod>) are available via tytus env --raw and should only be used for debugging.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pod_id": {
                        "type": "string",
                        "description": "Pod ID (e.g. '02'). Omit for first connected pod."
                    }
                },
                "required": []
            }),
        },
        ToolInfo {
            name: "tytus_models".into(),
            description: "List the LLM models available on the user's pod gateway. Returns the small fixed catalog: ail-compound (MiniMax M2.7, text+vision+audio), ail-image (MiniMax image-01), ail-embed (mistral-embed via SwitchAI), and the minimax/-prefixed aliases. Requires an active tunnel — call tytus_status first and tytus_setup_guide if no pod is connected.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pod_id": { "type": "string", "description": "Pod ID. Omit for first connected pod." }
                },
                "required": []
            }),
        },
        ToolInfo {
            name: "tytus_chat".into(),
            description: "Send a chat completion through the user's private pod gateway. The request is OpenAI-compatible and is routed via WireGuard tunnel through the droplet's SwitchAILocal proxy to MiniMax (no customer LLM traffic ever traverses Traylinx Cloud). The model parameter MUST be one of: ail-compound (default text/vision/audio), ail-image, ail-embed, minimax/ail-compound, minimax/ail-image. Do NOT pass any other model id — it will fail. Requires an active tunnel.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "model": {
                        "type": "string",
                        "enum": ["ail-compound", "ail-image", "ail-embed", "minimax/ail-compound", "minimax/ail-image"],
                        "description": "One of the fixed model ids on the pod gateway. Default chat = ail-compound."
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
                        "description": "Chat messages array (OpenAI format)"
                    },
                    "max_tokens": {
                        "type": "integer",
                        "description": "Max tokens to generate (default 1024). MiniMax M2.7 can spend most tokens on reasoning_content before producing visible text — bump this to 200+ if you see empty content."
                    },
                    "temperature": {
                        "type": "number",
                        "description": "Sampling temperature (default 0.7)"
                    },
                    "pod_id": {
                        "type": "string",
                        "description": "Pod ID. Omit for first connected pod."
                    }
                },
                "required": ["model", "messages"]
            }),
        },
        ToolInfo {
            name: "tytus_revoke".into(),
            description: "DESTRUCTIVE. Revoke a pod allocation: frees its units in Scalesys AND wipes the pod's workspace state directory + container on the droplet. Cannot be undone. Always confirm with the user before calling this. The user can re-allocate later with tytus_status / tytus connect, but they will lose any sessions, skills, memories, and overlay config they had on the pod.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pod_id": {
                        "type": "string",
                        "description": "Pod ID to revoke (e.g. '02')."
                    }
                },
                "required": ["pod_id"]
            }),
        },
        ToolInfo {
            name: "tytus_setup_guide".into(),
            description: "Return human-readable setup instructions to show the user when they are not logged in or have no active pod. Use this as the response body when tytus_status returns logged_in=false or pods=[] — it tells the user exactly which `tytus` commands to run and in what order. Do NOT make up instructions; always pull from this tool.".into(),
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

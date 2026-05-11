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
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }
    fn error(id: Value, code: i64, message: String) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError { code, message }),
        }
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
            content: vec![ContentBlock {
                content_type: "text".into(),
                text: s,
            }],
            is_error: None,
        }
    }
    fn error(s: String) -> Self {
        Self {
            content: vec![ContentBlock {
                content_type: "text".into(),
                text: s,
            }],
            is_error: Some(true),
        }
    }
}

// ── Tool definitions ────────────────────────────────────────

fn tool_definitions() -> Vec<ToolInfo> {
    vec![
        ToolInfo {
            name: "tytus_docs".into(),
            description: "Return the comprehensive LLM-facing reference for tytus-cli (same content as `tytus llm-docs`). Read this BEFORE driving any other tytus operation in a fresh session — it covers the command surface, agent types (OpenClaw=1u, Hermes=2u), plan tiers, the only live AIL model aliases returned by the gateway, the stable URL/key model, and the standard recipes. Cache the output in your context for the rest of the session.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        ToolInfo {
            name: "tytus_os_docs".into(),
            description: "Return the complete Tytus OS user manual (same content as `tytus os-docs`). This is the source of truth for the *desktop OS*: boot/login phases, the desktop, top panel, dock, App Launcher, window management (drag/resize/snap/animations), keyboard shortcuts, the Settings app and every panel (account, plan, pods, agents, daemon, sharing, background, appearance, dock, languages, notifications, privacy, about), the Files window (vfs vs daemon backends, drag-out, multi-select, conflict resolution, partial-failure semantics, undo, trash), host clipboard (Cmd+V) per-browser support, all 50 installed apps in 8 categories, troubleshooting recipes. Read this BEFORE answering any question about how Tytus OS looks, behaves, or what its surfaces do. Companion to tytus_docs (which covers the CLI tool).".into(),
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
            description: "Return the connection environment variables for a pod. Default output is the STABLE pair: OPENAI_BASE_URL=http://10.42.42.1:18080/v1 and OPENAI_API_KEY=sk-tytus-user-<32hex>. These values are constant across pod revoke/reallocate cycles. Use these in any user-visible config file. The legacy per-pod values (10.18.X.Y + sk-<pod>) are available by passing raw=true and should only be used for debugging.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pod_id": {
                        "type": "string",
                        "description": "Pod ID (e.g. '02'). Omit for first connected pod."
                    },
                    "raw": {
                        "type": "boolean",
                        "default": false,
                        "description": "Return per-pod debug values (internal 10.18.X.Y endpoint + per-pod key) instead of the stable user-facing pair. Only set this if explicitly debugging routing or key propagation."
                    }
                },
                "required": []
            }),
        },
        ToolInfo {
            name: "tytus_models".into(),
            description: "List the live LLM model aliases available on the user's pod gateway. The gateway is AIL-configured; apps must discover model ids at runtime instead of hardcoding provider names. Requires an active tunnel — call tytus_status first and tytus_setup_guide if no pod is connected.".into(),
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
            description: "Send a chat completion through the user's private pod gateway. The request is OpenAI-compatible and routed via WireGuard tunnel through the droplet's SwitchAILocal/AIL proxy. Use a model id returned by tytus_models or the gateway model list; default to ail-compound if the caller does not specify one. Requires an active tunnel.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "model": {
                        "type": "string",
                        "description": "Model id returned by the live pod gateway model list. Default chat alias = ail-compound."
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
                        "description": "Max tokens to generate (default 1024). Some upstream models may spend tokens on reasoning_content before visible text — bump this if you see empty content."
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
        if line.trim().is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp =
                    JsonRpcResponse::error(Value::Null, -32700, format!("Parse error: {}", e));
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

fn pinned_account_error() -> Option<String> {
    let pinned = std::env::var("TYTUS_PINNED_ACCOUNT_EMAIL").ok()?;
    let state = state::CliState::load_file_only();
    let active = state.active_email.clone().or(state.email.clone());
    let matches = active
        .as_ref()
        .is_some_and(|a| a.trim().eq_ignore_ascii_case(pinned.trim()));
    if matches {
        return None;
    }
    let active_text = active.unwrap_or_else(|| "<none>".into());
    Some(format!(
        "Tytus MCP is pinned to {} but active account is {}. Run: tytus account switch {}. Or regenerate config: tytus mcp --account {}",
        pinned, active_text, pinned, active_text
    ))
}

fn write_response(stdout: &io::Stdout, resp: &JsonRpcResponse) {
    let mut handle = stdout.lock();
    let _ = serde_json::to_writer(&mut handle, resp);
    let _ = handle.write_all(b"\n");
    let _ = handle.flush();
}

async fn handle_request(req: JsonRpcRequest) -> Result<Value, String> {
    if req.method != "notifications/initialized" {
        if let Some(err) = pinned_account_error() {
            return Err(err);
        }
    }
    match req.method.as_str() {
        "initialize" => Ok(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": SERVER_NAME,
                "version": SERVER_VERSION
            }
        })),
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
            let tool_name = params
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or("Missing tool name")?
                .to_string();
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or(serde_json::json!({}));

            let result = tools::call_tool(&tool_name, arguments).await;
            Ok(serde_json::to_value(result).unwrap_or(Value::Null))
        }
        "ping" => Ok(serde_json::json!({})),
        _ => Err(format!("Unknown method: {}", req.method)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_state(active: &str, f: impl FnOnce()) {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = std::env::temp_dir().join(format!(
            "tytus-mcp-test-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("state.json");
        std::fs::write(
            &path,
            serde_json::json!({
                "schema_version": 2,
                "active_email": active,
                "accounts": [{"email": active}],
                "email": active,
                "pods": []
            })
            .to_string(),
        )
        .unwrap();
        std::env::set_var("TYTUS_STATE_PATH", &path);
        f();
        std::env::remove_var("TYTUS_STATE_PATH");
        std::env::remove_var("TYTUS_PINNED_ACCOUNT_EMAIL");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn pinned_account_matches_active_succeeds_and_mismatch_errors() {
        with_state("active@example.com", || {
            std::env::set_var("TYTUS_PINNED_ACCOUNT_EMAIL", "ACTIVE@example.com");
            assert!(pinned_account_error().is_none());
            std::env::set_var("TYTUS_PINNED_ACCOUNT_EMAIL", "other@example.com");
            let err = pinned_account_error().unwrap();
            assert!(err.contains("pinned to other@example.com"));
            assert!(err.contains("active account is active@example.com"));
        });
    }

    #[test]
    fn unpinned_uses_active() {
        with_state("active@example.com", || {
            std::env::remove_var("TYTUS_PINNED_ACCOUNT_EMAIL");
            assert!(pinned_account_error().is_none());
        });
    }
}

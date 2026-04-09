//! Tool implementations for the MCP server.
//!
//! Each tool reads the shared CLI state and (optionally) makes HTTP calls
//! to the Provider API or the pod's AI gateway.

use crate::state::CliState;
use crate::ToolResult;
use serde_json::Value;

pub async fn call_tool(name: &str, args: Value) -> ToolResult {
    match name {
        "tytus_status" => tool_status().await,
        "tytus_env" => tool_env(&args).await,
        "tytus_models" => tool_models(&args).await,
        "tytus_chat" => tool_chat(&args).await,
        "tytus_revoke" => tool_revoke(&args).await,
        "tytus_setup_guide" => tool_setup_guide().await,
        _ => ToolResult::error(format!("Unknown tool: {}", name)),
    }
}

async fn tool_status() -> ToolResult {
    let state = CliState::load();

    if !state.is_logged_in() {
        return ToolResult::text(serde_json::json!({
            "logged_in": false,
            "message": "Not logged in. User needs to run: tytus login"
        }).to_string());
    }

    let pods: Vec<Value> = state.pods.iter().map(|p| {
        serde_json::json!({
            "pod_id": p.pod_id,
            "agent_type": p.agent_type,
            "ai_endpoint": p.ai_endpoint,
            "agent_endpoint": p.agent_endpoint,
            "tunnel_active": p.tunnel_iface.is_some(),
            "tunnel_interface": p.tunnel_iface,
        })
    }).collect();

    ToolResult::text(serde_json::json!({
        "logged_in": true,
        "email": state.email,
        "tier": state.tier,
        "pod_count": state.pods.len(),
        "pods": pods,
        "has_active_tunnel": state.pods.iter().any(|p| p.tunnel_iface.is_some()),
    }).to_string())
}

async fn tool_env(args: &Value) -> ToolResult {
    let state = CliState::load();
    let pod_id = args.get("pod_id").and_then(|v| v.as_str());

    let pod = match state.find_pod(pod_id) {
        Some(p) => p,
        None => return ToolResult::error(
            "No pods available. User needs to run: sudo tytus connect".into()
        ),
    };

    let mut env = serde_json::Map::new();
    if let Some(ref ep) = pod.ai_endpoint {
        env.insert("TYTUS_AI_GATEWAY".into(), Value::String(ep.clone()));
        // Also provide OpenAI-compatible aliases
        env.insert("OPENAI_BASE_URL".into(), Value::String(format!("{}/v1", ep)));
    }
    if let Some(ref key) = pod.pod_api_key {
        env.insert("TYTUS_API_KEY".into(), Value::String(key.clone()));
        env.insert("OPENAI_API_KEY".into(), Value::String(key.clone()));
    }
    if let Some(ref ep) = pod.agent_endpoint {
        env.insert("TYTUS_AGENT_API".into(), Value::String(ep.clone()));
    }
    if let Some(ref at) = pod.agent_type {
        env.insert("TYTUS_AGENT_TYPE".into(), Value::String(at.clone()));
    }
    env.insert("TYTUS_POD_ID".into(), Value::String(pod.pod_id.clone()));
    env.insert("tunnel_active".into(), Value::Bool(pod.tunnel_iface.is_some()));

    if pod.tunnel_iface.is_none() {
        env.insert("warning".into(), Value::String(
            "Tunnel not active. Endpoints are allocated but not reachable. User needs: sudo tytus connect --pod ".to_string() + &pod.pod_id
        ));
    }

    ToolResult::text(Value::Object(env).to_string())
}

async fn tool_models(args: &Value) -> ToolResult {
    let state = CliState::load();
    let pod_id = args.get("pod_id").and_then(|v| v.as_str());

    let pod = match state.find_pod(pod_id) {
        Some(p) => p,
        None => return ToolResult::error("No pods available.".into()),
    };

    if pod.tunnel_iface.is_none() {
        return ToolResult::error(format!(
            "Tunnel not active for pod {}. User needs: sudo tytus connect --pod {}",
            pod.pod_id, pod.pod_id
        ));
    }

    let (gateway, api_key) = match (&pod.ai_endpoint, &pod.pod_api_key) {
        (Some(ep), Some(key)) => (ep.clone(), key.clone()),
        _ => return ToolResult::error("Pod missing endpoint or API key.".into()),
    };

    let url = format!("{}/v1/models", gateway);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_default();

    match client.get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.text().await {
                Ok(body) => {
                    // Parse and extract just model IDs for a cleaner response
                    if let Ok(parsed) = serde_json::from_str::<Value>(&body) {
                        if let Some(data) = parsed.get("data").and_then(|d| d.as_array()) {
                            let ids: Vec<&str> = data.iter()
                                .filter_map(|m| m.get("id").and_then(|i| i.as_str()))
                                .collect();
                            return ToolResult::text(serde_json::json!({
                                "model_count": ids.len(),
                                "models": ids,
                                "gateway": gateway,
                            }).to_string());
                        }
                    }
                    ToolResult::text(body)
                }
                Err(e) => ToolResult::error(format!("Failed to read response: {}", e)),
            }
        }
        Ok(resp) => ToolResult::error(format!("Gateway returned {}", resp.status())),
        Err(e) => ToolResult::error(format!(
            "Cannot reach gateway at {}. Is the tunnel running? Error: {}",
            gateway, e
        )),
    }
}

async fn tool_chat(args: &Value) -> ToolResult {
    let state = CliState::load();
    let pod_id = args.get("pod_id").and_then(|v| v.as_str());

    let pod = match state.find_pod(pod_id) {
        Some(p) => p,
        None => return ToolResult::error("No pods available.".into()),
    };

    if pod.tunnel_iface.is_none() {
        return ToolResult::error(format!(
            "Tunnel not active. User needs: sudo tytus connect --pod {}",
            pod.pod_id
        ));
    }

    let (gateway, api_key) = match (&pod.ai_endpoint, &pod.pod_api_key) {
        (Some(ep), Some(key)) => (ep.clone(), key.clone()),
        _ => return ToolResult::error("Pod missing endpoint or API key.".into()),
    };

    let model = match args.get("model").and_then(|v| v.as_str()) {
        Some(m) => m,
        None => return ToolResult::error("'model' is required.".into()),
    };
    let messages = match args.get("messages") {
        Some(m) => m.clone(),
        None => return ToolResult::error("'messages' is required.".into()),
    };
    let max_tokens = args.get("max_tokens").and_then(|v| v.as_i64()).unwrap_or(1024);
    let temperature = args.get("temperature").and_then(|v| v.as_f64()).unwrap_or(0.7);

    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "max_tokens": max_tokens,
        "temperature": temperature,
    });

    let url = format!("{}/v1/chat/completions", gateway);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .unwrap_or_default();

    match client.post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.text().await {
                Ok(body) => ToolResult::text(body),
                Err(e) => ToolResult::error(format!("Failed to read response: {}", e)),
            }
        }
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            ToolResult::error(format!("Gateway returned {}: {}", status, body))
        }
        Err(e) => ToolResult::error(format!("Request failed: {}", e)),
    }
}

async fn tool_revoke(args: &Value) -> ToolResult {
    let state = CliState::load();

    if !state.is_logged_in() {
        return ToolResult::error("Not logged in.".into());
    }

    let pod_id = match args.get("pod_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return ToolResult::error("'pod_id' is required.".into()),
    };

    // We need credentials to call the Provider API
    let (sk, auid) = match (&state.secret_key, &state.agent_user_id) {
        (Some(s), Some(a)) => (s.clone(), a.clone()),
        _ => return ToolResult::error("No Tytus credentials. Run: tytus login".into()),
    };

    let http = atomek_core::HttpClient::new();
    let client = atomek_pods::TytusClient::new(&http, &sk, &auid);

    match atomek_pods::revoke_pod(&client, pod_id).await {
        Ok(_) => ToolResult::text(serde_json::json!({
            "status": "revoked",
            "pod_id": pod_id,
            "message": format!("Pod {} revoked. Units freed.", pod_id)
        }).to_string()),
        Err(e) => ToolResult::error(format!("Revoke failed: {}", e)),
    }
}

async fn tool_setup_guide() -> ToolResult {
    let state = CliState::load();

    let mut steps = Vec::new();
    let mut step_num = 1;

    // Check if tytus binary exists
    steps.push(format!("{}. Install tytus CLI (if not already installed):\n   curl -fsSL https://tytus.traylinx.com/install.sh | sh\n   OR: cargo install --git https://github.com/traylinx/tytus-cli atomek-cli", step_num));
    step_num += 1;

    if !state.is_logged_in() {
        steps.push(format!("{}. Login (opens browser for one-time device auth):\n   tytus login", step_num));
        step_num += 1;
    } else {
        steps.push(format!("{}. Already logged in as {}", step_num, state.email.as_deref().unwrap_or("?")));
        step_num += 1;
    }

    let has_tunnel = state.pods.iter().any(|p| p.tunnel_iface.is_some());
    if !has_tunnel {
        steps.push(format!("{}. Allocate pod and activate tunnel (requires sudo for TUN device):\n   sudo tytus connect\n   # Or with Hermes agent (2 units): sudo tytus connect --agent hermes\n   # Keep this running — it blocks until Ctrl+C", step_num));
        step_num += 1;
    } else {
        steps.push(format!("{}. Tunnel is active!", step_num));
        step_num += 1;
    }

    steps.push(format!("{}. In another terminal, load connection vars:\n   eval $(tytus env --export)\n   # This sets TYTUS_AI_GATEWAY, TYTUS_API_KEY, etc.", step_num));
    step_num += 1;

    steps.push(format!("{}. Test the connection:\n   curl -s \"$TYTUS_AI_GATEWAY/v1/models\" -H \"Authorization: Bearer $TYTUS_API_KEY\" | jq '.data[].id' | head -5", step_num));
    step_num += 1;

    steps.push(format!("{}. Use with any OpenAI-compatible tool:\n   export OPENAI_API_KEY=$TYTUS_API_KEY\n   export OPENAI_BASE_URL=$TYTUS_AI_GATEWAY/v1", step_num));

    ToolResult::text(serde_json::json!({
        "current_state": {
            "logged_in": state.is_logged_in(),
            "email": state.email,
            "tier": state.tier,
            "pod_count": state.pods.len(),
            "has_active_tunnel": has_tunnel,
        },
        "setup_steps": steps,
    }).to_string())
}

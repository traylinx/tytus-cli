use atomek_core::AtomekError;
use serde::Deserialize;

use crate::client::TytusClient;

/// Response from `POST /pod/default` — the "AIL-gateway-only" path.
///
/// Default pods carry `agent_type: "none"`, cost zero units, and exist
/// solely to route the user's WG tunnel so they can reach the droplet's
/// dual-bound `10.42.42.1:18080` AIL gateway with their stable user key.
///
/// The endpoint is idempotent: a subsequent call while a default pod is
/// already active returns the same pod unchanged with `reused: true`
/// (HTTP 200 rather than 201). See SPRINT-AIL-DEFAULT-POD §4.2 / §8.1.
#[derive(Debug, Deserialize)]
pub struct DefaultPodAllocation {
    pub pod_id: String,
    pub droplet_id: String,
    pub droplet_ip: Option<String>,
    pub wireguard_port: Option<u16>,
    pub subnet: Option<String>,
    pub ai_endpoint: Option<String>,
    pub pod_api_key: Option<String>,
    pub agent_type: Option<String>,
    pub agent_units: Option<u32>,
    pub stable_ai_endpoint: Option<String>,
    pub stable_user_key: Option<String>,
    #[serde(default)]
    pub reused: bool,
}

/// Ensure the authenticated user has a default (agent-less, 0-unit) pod.
///
/// Safe to call on every login: idempotent on the server side. Callers
/// MUST NOT retry on non-network errors — even though /pod/default is
/// idempotent, a blind retry on e.g. a 503 that actually succeeded
/// upstream would waste a slot if the server's idempotency key expired.
pub async fn request_default_pod(client: &TytusClient) -> atomek_core::Result<DefaultPodAllocation> {
    let resp = client
        .post("/pod/default")
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| AtomekError::Network(e.to_string()))?;

    let status = resp.status().as_u16();
    if resp.status().is_success() {
        return resp
            .json::<DefaultPodAllocation>()
            .await
            .map_err(|e| AtomekError::Other(format!("Failed to parse default pod: {}", e)));
    }

    let body = resp.text().await.unwrap_or_default();
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
        match status {
            503 => Err(AtomekError::NoCapacity {
                retry_after: json["retry_after"].as_u64().unwrap_or(300),
            }),
            401 => Err(AtomekError::AuthExpired),
            _ => Err(AtomekError::ApiStatus {
                status,
                message: json["message"].as_str().unwrap_or(&body).to_string(),
            }),
        }
    } else {
        Err(AtomekError::ApiStatus { status, message: body })
    }
}

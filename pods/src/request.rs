use atomek_core::AtomekError;
use serde::Deserialize;
use crate::client::TytusClient;

#[derive(Debug, Deserialize)]
pub struct PodAllocation {
    pub status: String,
    pub pod_id: String,
    pub droplet_id: String,
    pub droplet_ip: Option<String>,
    pub wireguard_port: Option<u16>,
    pub subnet: Option<String>,
    pub ai_endpoint: Option<String>,
    pub pod_api_key: Option<String>,
    pub agent_type: Option<String>,
    pub agent_units: Option<u32>,
    pub agent_endpoint: Option<String>,
    pub agent_health_port: Option<u16>,
    pub agent_api_port: Option<u16>,
}

pub async fn request_pod(client: &TytusClient) -> atomek_core::Result<PodAllocation> {
    request_pod_with_agent(client, "nemoclaw").await
}

pub async fn request_pod_with_agent(client: &TytusClient, agent_type: &str) -> atomek_core::Result<PodAllocation> {
    // Don't use send_with_retry for POST /pod/request — it's not idempotent.
    // A retry could allocate two pods. Use single-shot instead.
    let resp = client.post("/pod/request")
        .json(&serde_json::json!({ "agent_type": agent_type }))
        .send().await
        .map_err(|e| AtomekError::Network(e.to_string()))?;

    let status = resp.status().as_u16();
    if resp.status().is_success() {
        return resp.json().await
            .map_err(|e| AtomekError::Other(format!("Failed to parse allocation: {}", e)));
    }

    // Parse error response
    let body = resp.text().await.unwrap_or_default();
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
        let error_key = json["error"].as_str().unwrap_or("");
        match (status, error_key) {
            (403, "plan_limit_reached") => Err(AtomekError::PodLimitReached {
                limit: json["max_units"].as_u64()
                    .or_else(|| json["limit"].as_u64())
                    .unwrap_or(0) as u32,
                current: json["units_used"].as_u64()
                    .or_else(|| json["current"].as_u64())
                    .unwrap_or(0) as u32,
            }),
            (403, "no_plan") => Err(AtomekError::NoSubscription),
            (503, _) => Err(AtomekError::NoCapacity {
                retry_after: json["retry_after"].as_u64().unwrap_or(300),
            }),
            (401, _) => Err(AtomekError::AuthExpired),
            _ => Err(AtomekError::ApiStatus {
                status,
                message: json["message"].as_str().unwrap_or(&body).to_string(),
            }),
        }
    } else {
        Err(AtomekError::ApiStatus { status, message: body })
    }
}

use crate::client::TytusClient;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PodStatus {
    pub has_plan: bool,
    pub tier_name: Option<String>,
    #[serde(alias = "max_units")]
    pub max_pods: u32,
    pub current_pods: u32,
    pub units_used: Option<u32>,
    pub pods: Vec<PodEntry>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PodEntry {
    pub pod_id: String,
    #[serde(default)]
    pub route_id: Option<String>,
    #[serde(default)]
    pub droplet_id: Option<String>,
    pub agent_type: Option<String>,
    pub agent_units: Option<u32>,
    #[serde(default)]
    pub display_name: Option<String>,
    pub created_at: Option<f64>,
    pub status: Option<String>,
    #[serde(default)]
    pub stable_ai_endpoint: Option<String>,
    #[serde(default)]
    pub stable_user_key: Option<String>,
    #[serde(default)]
    pub edge_public_url: Option<String>,
    #[serde(default)]
    pub pod_public_url: Option<String>,
    #[serde(default)]
    pub cortex_ready: Option<bool>,
}

pub async fn get_pod_status(client: &TytusClient) -> atomek_core::Result<PodStatus> {
    let resp = client.get_with_retry("/pod/status").await?;
    resp.json()
        .await
        .map_err(|e| atomek_core::AtomekError::Other(format!("Failed to parse pod status: {}", e)))
}

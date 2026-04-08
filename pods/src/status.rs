use serde::Deserialize;
use crate::client::TytusClient;

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
    pub droplet_id: String,
    pub agent_type: Option<String>,
    pub agent_units: Option<u32>,
    pub created_at: Option<f64>,
    pub status: Option<String>,
}

pub async fn get_pod_status(client: &TytusClient) -> atomek_core::Result<PodStatus> {
    let resp = client.get_with_retry("/pod/status").await?;
    resp.json().await.map_err(|e| atomek_core::AtomekError::Other(format!("Failed to parse pod status: {}", e)))
}

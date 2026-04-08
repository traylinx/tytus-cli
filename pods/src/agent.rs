use atomek_core::AtomekError;
use serde::Deserialize;
use crate::client::TytusClient;

#[derive(Debug, Deserialize)]
pub struct AgentStatus {
    pub pod_num: Option<u32>,
    pub agent_type: Option<String>,
    pub container_status: Option<String>,
    pub healthy: Option<bool>,
    pub uptime_seconds: Option<u64>,
    pub image: Option<String>,
    pub ports: Option<AgentPorts>,
}

#[derive(Debug, Deserialize)]
pub struct AgentPorts {
    pub health: Option<u16>,
    pub api: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct AgentDeployResult {
    pub pod_num: Option<u32>,
    pub agent_type: Option<String>,
    pub container_status: Option<String>,
    pub healthy: Option<bool>,
    pub warning: Option<String>,
    pub logs: Option<String>,
    pub ports: Option<AgentPorts>,
}

pub async fn get_agent_status(client: &TytusClient, pod_id: &str) -> atomek_core::Result<AgentStatus> {
    let resp = client.get_with_retry(&format!("/pod/agent/status?pod_id={}", pod_id)).await?;
    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(AtomekError::ApiStatus { status, message: body });
    }
    resp.json().await
        .map_err(|e| AtomekError::Other(format!("Failed to parse agent status: {}", e)))
}

pub async fn deploy_agent(client: &TytusClient, pod_id: &str, agent_type: &str) -> atomek_core::Result<AgentDeployResult> {
    let resp = client.post("/pod/agent/deploy")
        .json(&serde_json::json!({
            "pod_id": pod_id,
            "agent_type": agent_type,
        }))
        .send().await
        .map_err(|e| AtomekError::Network(e.to_string()))?;

    let status = resp.status().as_u16();
    if resp.status().is_success() || status == 201 {
        return resp.json().await
            .map_err(|e| AtomekError::Other(format!("Failed to parse deploy result: {}", e)));
    }

    let body = resp.text().await.unwrap_or_default();
    Err(AtomekError::ApiStatus { status, message: body })
}

pub async fn restart_agent(client: &TytusClient, pod_id: &str) -> atomek_core::Result<AgentStatus> {
    let resp = client.post("/pod/agent/restart")
        .json(&serde_json::json!({ "pod_id": pod_id }))
        .send().await
        .map_err(|e| AtomekError::Network(e.to_string()))?;

    let status = resp.status().as_u16();
    if resp.status().is_success() {
        return resp.json().await
            .map_err(|e| AtomekError::Other(format!("Failed to parse restart result: {}", e)));
    }

    let body = resp.text().await.unwrap_or_default();
    Err(AtomekError::ApiStatus { status, message: body })
}

pub async fn stop_agent(client: &TytusClient, pod_id: &str) -> atomek_core::Result<()> {
    let resp = client.post("/pod/agent/stop")
        .json(&serde_json::json!({ "pod_id": pod_id }))
        .send().await
        .map_err(|e| AtomekError::Network(e.to_string()))?;

    if resp.status().is_success() {
        return Ok(());
    }

    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    Err(AtomekError::ApiStatus { status, message: body })
}

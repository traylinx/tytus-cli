use atomek_core::AtomekError;
use serde::Deserialize;
use crate::client::TytusClient;

/// Strict pod-id validator for any function that interpolates `pod_id`
/// into a query string. Pod ids on this product are always
/// zero-padded decimal (e.g. "02", "04") — accepting anything else
/// here lets a crafted pod_id smuggle extra query parameters into the
/// Provider URL (codex review 2026-04-29 caught
/// `02&reveal=secrets` injection). 3 chars is enough for `0..=99`,
/// padded; tighten further if pod ids ever get a stricter spec.
fn validate_pod_id(pod_id: &str) -> Result<(), AtomekError> {
    if pod_id.is_empty() || pod_id.len() > 3 || !pod_id.chars().all(|c| c.is_ascii_digit()) {
        return Err(AtomekError::Other(format!(
            "invalid pod_id {:?} (expected 1-3 ASCII digits)",
            pod_id
        )));
    }
    Ok(())
}

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
    validate_pod_id(pod_id)?;
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

#[derive(Debug, Deserialize)]
pub struct ExecResult {
    pub exit_code: i64,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

pub async fn exec_in_agent(client: &TytusClient, pod_id: &str, command: &str, timeout: u32) -> atomek_core::Result<ExecResult> {
    let resp = client.post("/pod/agent/exec")
        .json(&serde_json::json!({
            "pod_id": pod_id,
            "command": command,
            "timeout": timeout,
        }))
        .send().await
        .map_err(|e| AtomekError::Network(e.to_string()))?;

    let status = resp.status().as_u16();
    if resp.status().is_success() {
        return resp.json().await
            .map_err(|e| AtomekError::Other(format!("Failed to parse exec result: {}", e)));
    }

    let body = resp.text().await.unwrap_or_default();
    Err(AtomekError::ApiStatus { status, message: body })
}

#[derive(Debug, Deserialize)]
pub struct AgentLogs {
    pub pod_num: Option<u32>,
    pub logs: String,
}

pub async fn agent_logs(client: &TytusClient, pod_id: &str, tail: u32) -> atomek_core::Result<AgentLogs> {
    validate_pod_id(pod_id)?;
    let tail = tail.clamp(1, 500);
    let path = format!("/pod/agent/logs?pod_id={}&tail={}", pod_id, tail);
    let resp = client.get_with_retry(&path).await?;
    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(AtomekError::ApiStatus { status, message: body });
    }
    resp.json().await
        .map_err(|e| AtomekError::Other(format!("Failed to parse agent logs: {}", e)))
}

#[derive(Debug, Deserialize)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
    /// One of: "channels", "agent_default", "operator_override", "runtime".
    /// Optional in the type so old Provider builds without the field still
    /// deserialize cleanly — render code falls back to "unknown".
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AgentEnv {
    pub pod_num: Option<u32>,
    pub agent_type: Option<String>,
    pub reveal_secrets: Option<bool>,
    pub vars: Vec<EnvVar>,
}

pub async fn agent_env(
    client: &TytusClient,
    pod_id: &str,
    reveal_secrets: bool,
) -> atomek_core::Result<AgentEnv> {
    validate_pod_id(pod_id)?;
    let mut path = format!("/pod/agent/env?pod_id={}", pod_id);
    if reveal_secrets {
        path.push_str("&reveal=secrets");
    }
    let resp = client.get_with_retry(&path).await?;
    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(AtomekError::ApiStatus { status, message: body });
    }
    resp.json().await
        .map_err(|e| AtomekError::Other(format!("Failed to parse agent env: {}", e)))
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

#[cfg(test)]
mod tests {
    use super::validate_pod_id;

    #[test]
    fn validate_pod_id_accepts_canonical_forms() {
        assert!(validate_pod_id("02").is_ok());
        assert!(validate_pod_id("04").is_ok());
        // Single-digit forms (rare but technically valid).
        assert!(validate_pod_id("9").is_ok());
        assert!(validate_pod_id("99").is_ok());
        assert!(validate_pod_id("999").is_ok());
    }

    #[test]
    fn validate_pod_id_rejects_query_injection_shapes() {
        // Codex review 2026-04-29 — these are the exploits that would
        // smuggle an extra query param into the Provider URL.
        assert!(validate_pod_id("02&reveal=secrets").is_err());
        assert!(validate_pod_id("02 ").is_err());
        assert!(validate_pod_id("../etc/passwd").is_err());
        assert!(validate_pod_id("02#frag").is_err());
        assert!(validate_pod_id("02?reveal=1").is_err());
    }

    #[test]
    fn validate_pod_id_rejects_empty_and_oversized() {
        assert!(validate_pod_id("").is_err());
        // 4+ digits are out of spec — pods are 0..=99 today (2 digits)
        // and the reserved block tops out at 99 in any plausible future.
        assert!(validate_pod_id("0099").is_err());
        assert!(validate_pod_id("12345").is_err());
    }
}

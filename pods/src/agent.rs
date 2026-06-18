use crate::client::TytusClient;
use atomek_core::AtomekError;
use serde::Deserialize;

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

fn validate_route_id(route_id: &str) -> Result<(), AtomekError> {
    let valid = route_id.len() == 10
        && route_id.chars().all(
            |c| matches!(c, '0'..='9' | 'a'..='h' | 'j'..='k' | 'm'..='n' | 'p'..='t' | 'v'..='z'),
        );
    if !valid {
        return Err(AtomekError::Other(format!(
            "invalid route_id {:?} (expected 10 lowercase Crockford-like chars)",
            route_id
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct AgentTarget<'a> {
    pub pod_id: &'a str,
    pub route_id: Option<&'a str>,
}

impl<'a> AgentTarget<'a> {
    pub fn new(pod_id: &'a str, route_id: Option<&'a str>) -> Self {
        Self { pod_id, route_id }
    }

    pub fn pod(pod_id: &'a str) -> Self {
        Self {
            pod_id,
            route_id: None,
        }
    }
}

fn validate_target(target: AgentTarget<'_>) -> Result<(), AtomekError> {
    validate_pod_id(target.pod_id)?;
    if let Some(route_id) = target.route_id {
        validate_route_id(route_id)?;
    }
    Ok(())
}

fn append_route_query(path: &mut String, target: AgentTarget<'_>) {
    if let Some(route_id) = target.route_id {
        path.push_str("&route_id=");
        path.push_str(route_id);
    }
}

#[derive(Debug, Deserialize)]
pub struct AgentStatus {
    pub pod_num: Option<u32>,
    pub agent_type: Option<String>,
    pub container_status: Option<String>,
    pub healthy: Option<bool>,
    pub uptime_seconds: Option<u64>,
    pub image: Option<String>,
    pub image_id: Option<String>,
    pub image_repo_digests: Option<Vec<String>>,
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

pub async fn get_agent_status(
    client: &TytusClient,
    pod_id: &str,
) -> atomek_core::Result<AgentStatus> {
    get_agent_status_target(client, AgentTarget::pod(pod_id)).await
}

pub async fn get_agent_status_target(
    client: &TytusClient,
    target: AgentTarget<'_>,
) -> atomek_core::Result<AgentStatus> {
    validate_target(target)?;
    let mut path = format!("/pod/agent/status?pod_id={}", target.pod_id);
    append_route_query(&mut path, target);
    let resp = client.get_with_retry(&path).await?;
    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(AtomekError::ApiStatus {
            status,
            message: body,
        });
    }
    resp.json()
        .await
        .map_err(|e| AtomekError::Other(format!("Failed to parse agent status: {}", e)))
}

pub async fn deploy_agent(
    client: &TytusClient,
    pod_id: &str,
    agent_type: &str,
) -> atomek_core::Result<AgentDeployResult> {
    let resp = client
        .post("/pod/agent/deploy")
        .json(&serde_json::json!({
            "pod_id": pod_id,
            "agent_type": agent_type,
        }))
        .send()
        .await
        .map_err(|e| AtomekError::Network(e.to_string()))?;

    let status = resp.status().as_u16();
    if resp.status().is_success() || status == 201 {
        return resp
            .json()
            .await
            .map_err(|e| AtomekError::Other(format!("Failed to parse deploy result: {}", e)));
    }

    let body = resp.text().await.unwrap_or_default();
    Err(AtomekError::ApiStatus {
        status,
        message: body,
    })
}

pub async fn restart_agent(client: &TytusClient, pod_id: &str) -> atomek_core::Result<AgentStatus> {
    restart_agent_target(client, AgentTarget::pod(pod_id)).await
}

pub async fn restart_agent_target(
    client: &TytusClient,
    target: AgentTarget<'_>,
) -> atomek_core::Result<AgentStatus> {
    validate_target(target)?;
    let body = agent_target_body(target);
    let resp = client
        .post("/pod/agent/restart")
        .json(&body)
        .send()
        .await
        .map_err(|e| AtomekError::Network(e.to_string()))?;

    let status = resp.status().as_u16();
    if resp.status().is_success() {
        return resp
            .json()
            .await
            .map_err(|e| AtomekError::Other(format!("Failed to parse restart result: {}", e)));
    }

    let body = resp.text().await.unwrap_or_default();
    Err(AtomekError::ApiStatus {
        status,
        message: body,
    })
}

#[derive(Debug, Deserialize)]
pub struct ExecResult {
    pub exit_code: i64,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

pub async fn exec_in_agent(
    client: &TytusClient,
    pod_id: &str,
    command: &str,
    timeout: u32,
) -> atomek_core::Result<ExecResult> {
    exec_in_agent_target(client, AgentTarget::pod(pod_id), command, timeout).await
}

pub async fn exec_in_agent_target(
    client: &TytusClient,
    target: AgentTarget<'_>,
    command: &str,
    timeout: u32,
) -> atomek_core::Result<ExecResult> {
    let body = exec_body(target, command, timeout)?;
    let resp = client
        .post("/pod/agent/exec")
        .json(&body)
        .send()
        .await
        .map_err(|e| AtomekError::Network(e.to_string()))?;

    let status = resp.status().as_u16();
    if resp.status().is_success() {
        return resp
            .json()
            .await
            .map_err(|e| AtomekError::Other(format!("Failed to parse exec result: {}", e)));
    }

    let body = resp.text().await.unwrap_or_default();
    Err(AtomekError::ApiStatus {
        status,
        message: body,
    })
}

#[derive(Debug, Deserialize)]
pub struct AgentLogs {
    pub pod_num: Option<u32>,
    pub logs: String,
}

pub async fn agent_logs(
    client: &TytusClient,
    pod_id: &str,
    tail: u32,
) -> atomek_core::Result<AgentLogs> {
    agent_logs_target(client, AgentTarget::pod(pod_id), tail).await
}

pub async fn agent_logs_target(
    client: &TytusClient,
    target: AgentTarget<'_>,
    tail: u32,
) -> atomek_core::Result<AgentLogs> {
    validate_target(target)?;
    let tail = tail.clamp(1, 500);
    let mut path = format!("/pod/agent/logs?pod_id={}&tail={}", target.pod_id, tail);
    append_route_query(&mut path, target);
    let resp = client.get_with_retry(&path).await?;
    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(AtomekError::ApiStatus {
            status,
            message: body,
        });
    }
    resp.json()
        .await
        .map_err(|e| AtomekError::Other(format!("Failed to parse agent logs: {}", e)))
}

fn agent_target_body(target: AgentTarget<'_>) -> serde_json::Value {
    let mut body = serde_json::json!({ "pod_id": target.pod_id });
    if let Some(route_id) = target.route_id {
        body["route_id"] = serde_json::Value::String(route_id.to_string());
    }
    body
}

fn exec_body(
    target: AgentTarget<'_>,
    command: &str,
    timeout: u32,
) -> Result<serde_json::Value, AtomekError> {
    validate_target(target)?;
    let mut body = agent_target_body(target);
    body["command"] = serde_json::Value::String(command.to_string());
    body["timeout"] = serde_json::Value::Number(serde_json::Number::from(timeout));
    Ok(body)
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

fn agent_env_path(target: AgentTarget<'_>, reveal_secrets: bool) -> Result<String, AtomekError> {
    validate_target(target)?;
    let mut path = format!("/pod/agent/env?pod_id={}", target.pod_id);
    append_route_query(&mut path, target);
    if reveal_secrets {
        path.push_str("&reveal=secrets");
    }
    Ok(path)
}

pub async fn agent_env(
    client: &TytusClient,
    pod_id: &str,
    reveal_secrets: bool,
) -> atomek_core::Result<AgentEnv> {
    agent_env_target(client, AgentTarget::pod(pod_id), reveal_secrets).await
}

pub async fn agent_env_target(
    client: &TytusClient,
    target: AgentTarget<'_>,
    reveal_secrets: bool,
) -> atomek_core::Result<AgentEnv> {
    let path = agent_env_path(target, reveal_secrets)?;
    let resp = client.get_with_retry(&path).await?;
    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(AtomekError::ApiStatus {
            status,
            message: body,
        });
    }
    resp.json()
        .await
        .map_err(|e| AtomekError::Other(format!("Failed to parse agent env: {}", e)))
}

pub async fn stop_agent(client: &TytusClient, pod_id: &str) -> atomek_core::Result<()> {
    let resp = client
        .post("/pod/agent/stop")
        .json(&serde_json::json!({ "pod_id": pod_id }))
        .send()
        .await
        .map_err(|e| AtomekError::Network(e.to_string()))?;

    if resp.status().is_success() {
        return Ok(());
    }

    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    Err(AtomekError::ApiStatus {
        status,
        message: body,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        agent_env_path, exec_body, validate_pod_id, validate_route_id, AgentStatus, AgentTarget,
    };

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

    #[test]
    fn validate_route_id_accepts_provider_shape() {
        assert!(validate_route_id("0e0ah755r3").is_ok());
        assert!(validate_route_id("eb2qvn3t4s").is_ok());
        assert!(validate_route_id("12gy79s7g0").is_ok());
    }

    #[test]
    fn validate_route_id_rejects_query_injection_shapes() {
        assert!(validate_route_id("0e0ah755r3&x=1").is_err());
        assert!(validate_route_id("0e0ah755").is_err());
        assert!(validate_route_id("0E0AH755R3").is_err());
        assert!(validate_route_id("../escape").is_err());
    }

    #[test]
    fn agent_status_deserializes_image_identity_fields() {
        let status: AgentStatus = serde_json::from_value(serde_json::json!({
            "pod_num": 1,
            "agent_type": "nemoclaw",
            "container_status": "running",
            "healthy": true,
            "uptime_seconds": 42,
            "image": "tytus-nemoclaw:latest",
            "image_id": "sha256:image-live",
            "image_repo_digests": ["tytus-nemoclaw@sha256:digest-live"],
            "ports": {"api": 3000, "health": 3000}
        }))
        .unwrap();

        assert_eq!(status.image_id.as_deref(), Some("sha256:image-live"));
        assert_eq!(
            status.image_repo_digests.as_ref().unwrap(),
            &vec!["tytus-nemoclaw@sha256:digest-live".to_string()]
        );
    }

    #[test]
    fn agent_env_path_includes_route_id_when_present() {
        let path = agent_env_path(AgentTarget::new("01", Some("0e0ah755r3")), false).unwrap();
        assert_eq!(path, "/pod/agent/env?pod_id=01&route_id=0e0ah755r3");
    }

    #[test]
    fn agent_env_path_preserves_reveal_secrets_after_route_id() {
        let path = agent_env_path(AgentTarget::new("01", Some("eb2qvn3t4s")), true).unwrap();
        assert_eq!(
            path,
            "/pod/agent/env?pod_id=01&route_id=eb2qvn3t4s&reveal=secrets"
        );
    }

    #[test]
    fn agent_env_path_rejects_route_injection() {
        assert!(agent_env_path(AgentTarget::new("01", Some("0e0ah755r3&x=1")), false).is_err());
    }

    #[test]
    fn exec_body_includes_route_id_when_present() {
        let body = exec_body(AgentTarget::new("01", Some("0e0ah755r3")), "whoami", 10).unwrap();
        assert_eq!(body["pod_id"], "01");
        assert_eq!(body["route_id"], "0e0ah755r3");
        assert_eq!(body["command"], "whoami");
        assert_eq!(body["timeout"], 10);
    }

    #[test]
    fn exec_body_omits_route_id_when_absent() {
        let body = exec_body(AgentTarget::pod("02"), "true", 5).unwrap();
        assert_eq!(body["pod_id"], "02");
        assert!(body.get("route_id").is_none());
        assert_eq!(body["command"], "true");
        assert_eq!(body["timeout"], 5);
    }
}

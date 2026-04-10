//! Shared state reader — reads from the same state.json as the CLI.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const STATE_DIR: &str = "tytus";
const STATE_FILE: &str = "state.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliState {
    pub email: Option<String>,
    pub refresh_token: Option<String>,
    pub access_token: Option<String>,
    pub expires_at_ms: Option<i64>,
    pub secret_key: Option<String>,
    pub agent_user_id: Option<String>,
    pub organization_id: Option<String>,
    pub tier: Option<String>,
    pub pods: Vec<PodEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodEntry {
    pub pod_id: String,
    pub droplet_id: String,
    pub droplet_ip: Option<String>,
    pub ai_endpoint: Option<String>,
    pub pod_api_key: Option<String>,
    pub agent_type: Option<String>,
    pub agent_endpoint: Option<String>,
    pub tunnel_iface: Option<String>,
}

impl CliState {
    pub fn load() -> Self {
        let config = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        let path = config.join(STATE_DIR).join(STATE_FILE);
        match std::fs::read_to_string(&path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn is_logged_in(&self) -> bool {
        self.email.as_ref().is_some_and(|e| !e.is_empty())
            && self.refresh_token.as_ref().is_some_and(|t| !t.is_empty())
    }

    #[allow(dead_code)]
    pub fn has_valid_token(&self) -> bool {
        if let (Some(_), Some(exp)) = (&self.access_token, self.expires_at_ms) {
            let now = chrono::Utc::now().timestamp_millis();
            (now + 300_000) < exp
        } else {
            false
        }
    }

    pub fn find_pod(&self, pod_id: Option<&str>) -> Option<&PodEntry> {
        if let Some(pid) = pod_id {
            self.pods.iter().find(|p| p.pod_id == pid)
        } else {
            // First connected, then first available
            self.pods.iter().find(|p| p.tunnel_iface.is_some())
                .or_else(|| self.pods.first())
        }
    }
}

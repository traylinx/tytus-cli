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
    // Stable endpoint + per-user stable API key for local tools.
    // The endpoint is always http://10.42.42.1:18080 (dual-bound WG address)
    // and the key persists across pod revoke/reallocate cycles.
    #[serde(default)]
    pub stable_ai_endpoint: Option<String>,
    #[serde(default)]
    pub stable_user_key: Option<String>,
}

impl CliState {
    fn state_path() -> PathBuf {
        // When running elevated (sudo/osascript), TYTUS_REAL_HOME points to the
        // original user's home so we read THEIR state, not root's.
        // Fallback chain: TYTUS_REAL_HOME → SUDO_USER's home → dirs::config_dir()
        let config = if let Ok(real_home) = std::env::var("TYTUS_REAL_HOME") {
            PathBuf::from(real_home).join(if cfg!(target_os = "macos") {
                "Library/Application Support"
            } else {
                ".config"
            })
        } else if let Ok(sudo_user) = std::env::var("SUDO_USER") {
            // Running under plain sudo without TYTUS_REAL_HOME
            if cfg!(target_os = "macos") {
                PathBuf::from(format!("/Users/{}/Library/Application Support", sudo_user))
            } else {
                PathBuf::from(format!("/home/{}/.config", sudo_user))
            }
        } else {
            dirs::config_dir().unwrap_or_else(|| PathBuf::from("."))
        };
        let dir = config.join(STATE_DIR);
        std::fs::create_dir_all(&dir).ok();
        dir.join(STATE_FILE)
    }

    pub fn load() -> Self {
        let path = Self::state_path();
        match std::fs::read_to_string(&path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        let path = Self::state_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, &data);
            // Restrict permissions: owner-only read/write (contains tokens)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
            }
        }
    }

    pub fn clear(&mut self) {
        *self = Self::default();
        self.save();
    }

    pub fn is_logged_in(&self) -> bool {
        self.email.as_ref().map_or(false, |e| !e.is_empty())
            && self.refresh_token.as_ref().map_or(false, |t| !t.is_empty())
    }

    pub fn has_valid_token(&self) -> bool {
        if let (Some(_), Some(exp)) = (&self.access_token, self.expires_at_ms) {
            let now = chrono::Utc::now().timestamp_millis();
            (now + 300_000) < exp // 5 min buffer
        } else {
            false
        }
    }
}

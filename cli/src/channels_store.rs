//! Channel credential storage.
//!
//! Two layers:
//! - **Keychain** (source of truth for secret *values*) — one entry per
//!   (pod, channel, env_var) triple. Service name reuses the
//!   existing `com.traylinx.atomek` namespace so the single "Always
//!   allow" ACL approval covers all Tytus secrets.
//! - **Local manifest** at `~/.tytus/channels.json` — maps which
//!   channels are configured for each pod. Keychain doesn't support
//!   enumeration, so we track the list separately.
//!
//! The pod-side artifact (`/app/workspace/.tytus/channels.json`) is
//! built from these two on every `tytus channels add/remove` and
//! pushed to the pod via `tytus exec`. DAM reads it at deploy time
//! and merges the values into the agent container's env vars.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

const KEYCHAIN_SERVICE: &str = "com.traylinx.atomek";

#[derive(Debug, thiserror::Error)]
pub enum ChannelStoreError {
    #[error("keychain error: {0}")]
    Keychain(String),
    #[error("manifest I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("manifest JSON: {0}")]
    Json(#[from] serde_json::Error),
}

/// One configured channel for one pod. `env_vars` lists which env
/// vars are stored — *values* live in the keychain, not here.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelEntry {
    pub env_vars: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelManifest {
    /// pod_id → channel_name → entry
    #[serde(default)]
    pub pods: BTreeMap<String, BTreeMap<String, ChannelEntry>>,
}

impl ChannelManifest {
    pub fn path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join(".tytus").join("channels.json")
    }

    pub fn load() -> Self {
        let path = Self::path();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), ChannelStoreError> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let raw = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, raw)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }

    pub fn channels_for(&self, pod_id: &str) -> Vec<(&String, &ChannelEntry)> {
        self.pods
            .get(pod_id)
            .map(|m| m.iter().collect())
            .unwrap_or_default()
    }

    pub fn add_channel(&mut self, pod_id: &str, channel: &str, env_vars: Vec<String>) {
        self.pods
            .entry(pod_id.to_string())
            .or_default()
            .insert(channel.to_string(), ChannelEntry { env_vars });
    }

    pub fn remove_channel(&mut self, pod_id: &str, channel: &str) -> Option<ChannelEntry> {
        let pod_map = self.pods.get_mut(pod_id)?;
        let removed = pod_map.remove(channel);
        if pod_map.is_empty() {
            self.pods.remove(pod_id);
        }
        removed
    }
}

/// Keychain account key for one (pod, channel, env_var) triple.
fn account_key(pod_id: &str, channel: &str, env_var: &str) -> String {
    format!("channels::{}::{}::{}", pod_id, channel, env_var)
}

pub fn store_secret(
    pod_id: &str,
    channel: &str,
    env_var: &str,
    value: &str,
) -> Result<(), ChannelStoreError> {
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, &account_key(pod_id, channel, env_var))
        .map_err(|e| ChannelStoreError::Keychain(e.to_string()))?;
    entry
        .set_password(value)
        .map_err(|e| ChannelStoreError::Keychain(e.to_string()))?;
    Ok(())
}

pub fn get_secret(
    pod_id: &str,
    channel: &str,
    env_var: &str,
) -> Result<String, ChannelStoreError> {
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, &account_key(pod_id, channel, env_var))
        .map_err(|e| ChannelStoreError::Keychain(e.to_string()))?;
    entry
        .get_password()
        .map_err(|e| ChannelStoreError::Keychain(e.to_string()))
}

pub fn delete_secret(
    pod_id: &str,
    channel: &str,
    env_var: &str,
) -> Result<(), ChannelStoreError> {
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, &account_key(pod_id, channel, env_var))
        .map_err(|e| ChannelStoreError::Keychain(e.to_string()))?;
    let _ = entry.delete_credential(); // not-found is fine
    Ok(())
}

/// Build the pod-side channels.json payload from the local manifest +
/// keychain. This is what gets pushed to `/app/workspace/.tytus/channels.json`
/// on the pod, which DAM reads at container deploy time.
pub fn render_pod_payload(
    manifest: &ChannelManifest,
    pod_id: &str,
) -> Result<serde_json::Value, ChannelStoreError> {
    let mut channels_obj = serde_json::Map::new();
    for (channel, entry) in manifest.channels_for(pod_id) {
        let mut creds = serde_json::Map::new();
        for env_var in &entry.env_vars {
            let value = get_secret(pod_id, channel, env_var)?;
            creds.insert(env_var.clone(), serde_json::Value::String(value));
        }
        channels_obj.insert(channel.clone(), serde_json::Value::Object(creds));
    }
    Ok(serde_json::json!({
        "version": 1,
        "channels": channels_obj,
    }))
}

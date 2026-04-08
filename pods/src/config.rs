use atomek_core::AtomekError;
use zeroize::Zeroize;
use crate::client::TytusClient;

/// Parsed WireGuard configuration — keys kept in memory only.
#[derive(Debug)]
pub struct WireGuardConfig {
    pub private_key: String,
    pub address: String,
    pub dns: Option<String>,
    pub public_key: String,
    pub preshared_key: Option<String>,
    pub endpoint: String,
    pub allowed_ips: String,
    pub persistent_keepalive: Option<u16>,
}

impl Drop for WireGuardConfig {
    fn drop(&mut self) {
        self.private_key.zeroize();
        if let Some(ref mut psk) = self.preshared_key {
            psk.zeroize();
        }
    }
}

/// Download and parse WireGuard config from Provider API.
/// Backend: GET /pod/config/download — clientId from A2A headers.
/// Retries up to 10 times with 3s delay — config generates asynchronously after pod allocation.
pub async fn download_config(client: &TytusClient) -> atomek_core::Result<WireGuardConfig> {
    let max_attempts = 10;
    for attempt in 1..=max_attempts {
        let resp = client.get_with_retry("/pod/config/download").await;
        match resp {
            Ok(r) => {
                let text = r.text().await
                    .map_err(|e| AtomekError::Other(format!("Failed to read config: {}", e)))?;
                if text.contains("[Interface]") {
                    return parse_wireguard_config(&text);
                }
                // Got a response but not a valid config yet
                tracing::debug!(attempt, "Config response not ready yet");
            }
            Err(AtomekError::ConfigNotReady) | Err(AtomekError::NoPod) => {
                tracing::debug!(attempt, "Config not ready, retrying...");
            }
            Err(e) => return Err(e),
        }

        if attempt < max_attempts {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }

    Err(AtomekError::ConfigNotReady)
}

/// Download config for a specific pod by pod_id.
/// Backend: GET /pod/config/download?pod_id=XX
pub async fn download_config_for_pod(client: &TytusClient, pod_id: &str) -> atomek_core::Result<WireGuardConfig> {
    let path = format!("/pod/config/download?pod_id={}", pod_id);
    let max_attempts = 10;
    for attempt in 1..=max_attempts {
        let resp = client.get_with_retry(&path).await;
        match resp {
            Ok(r) => {
                let text = r.text().await
                    .map_err(|e| AtomekError::Other(format!("Failed to read config: {}", e)))?;
                if text.contains("[Interface]") {
                    return parse_wireguard_config(&text);
                }
                tracing::debug!(attempt, pod_id, "Config response not ready yet");
            }
            Err(AtomekError::ConfigNotReady) | Err(AtomekError::NoPod) => {
                tracing::debug!(attempt, pod_id, "Config not ready, retrying...");
            }
            Err(e) => return Err(e),
        }

        if attempt < max_attempts {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }

    Err(AtomekError::ConfigNotReady)
}

fn parse_wireguard_config(conf: &str) -> atomek_core::Result<WireGuardConfig> {
    let mut private_key = None;
    let mut address = None;
    let mut dns = None;
    let mut public_key = None;
    let mut preshared_key = None;
    let mut endpoint = None;
    let mut allowed_ips = None;
    let mut persistent_keepalive = None;

    for line in conf.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('[') || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "PrivateKey" => private_key = Some(value.to_string()),
                "Address" => address = Some(value.to_string()),
                "DNS" => dns = Some(value.to_string()),
                "PublicKey" => public_key = Some(value.to_string()),
                "PresharedKey" => preshared_key = Some(value.to_string()),
                "Endpoint" => endpoint = Some(value.to_string()),
                "AllowedIPs" => allowed_ips = Some(value.to_string()),
                "PersistentKeepalive" => persistent_keepalive = value.parse().ok(),
                _ => {}
            }
        }
    }

    Ok(WireGuardConfig {
        private_key: private_key.ok_or_else(|| AtomekError::Other("Missing PrivateKey in config".into()))?,
        address: address.ok_or_else(|| AtomekError::Other("Missing Address in config".into()))?,
        dns,
        public_key: public_key.ok_or_else(|| AtomekError::Other("Missing PublicKey in config".into()))?,
        preshared_key,
        endpoint: endpoint.ok_or_else(|| AtomekError::Other("Missing Endpoint in config".into()))?,
        allowed_ips: allowed_ips.ok_or_else(|| AtomekError::Other("Missing AllowedIPs in config".into()))?,
        persistent_keepalive,
    })
}

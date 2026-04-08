use atomek_core::AtomekError;
use serde::Deserialize;
use crate::client::TytusClient;

#[derive(Debug, Deserialize)]
pub struct RevokeResult {
    pub status: String,
    pub pod_id: Option<String>,
    pub droplet_id: Option<String>,
}

/// Revoke (release) the user's active pod.
/// Backend: POST /pod/revoke — clientId from A2A headers.
pub async fn revoke_pod(client: &TytusClient) -> atomek_core::Result<RevokeResult> {
    let resp = client.post_with_retry("/pod/revoke").await?;
    resp.json().await
        .map_err(|e| AtomekError::Other(format!("Failed to parse revoke response: {}", e)))
}

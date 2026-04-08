use atomek_core::AtomekError;
use serde::Deserialize;
use crate::client::TytusClient;

#[derive(Debug, Deserialize)]
pub struct RevokeResult {
    pub status: String,
    pub pod_id: Option<String>,
    pub droplet_id: Option<String>,
}

/// Revoke (release) a specific pod by ID.
/// Backend: POST /pod/revoke { pod_id } — clientId from A2A headers.
pub async fn revoke_pod(client: &TytusClient, pod_id: &str) -> atomek_core::Result<RevokeResult> {
    let body = serde_json::json!({ "pod_id": pod_id });
    let resp = client.post("/pod/revoke")
        .json(&body)
        .send()
        .await
        .map_err(|e| AtomekError::Network(e.to_string()))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(AtomekError::ApiStatus {
            status: status.as_u16(),
            message: text,
        });
    }

    resp.json().await
        .map_err(|e| AtomekError::Other(format!("Failed to parse revoke response: {}", e)))
}

/// Revoke all active pods for this client (logout scenario).
pub async fn revoke_all_pods(client: &TytusClient) -> atomek_core::Result<()> {
    // Get all pods, revoke each
    match crate::get_pod_status(client).await {
        Ok(status) => {
            for pod in &status.pods {
                if let Err(e) = revoke_pod(client, &pod.pod_id).await {
                    tracing::warn!("Failed to revoke pod {}: {}", pod.pod_id, e);
                }
            }
            Ok(())
        }
        Err(_) => {
            // Fallback: try revoking without pod_id (server revokes first active)
            let _ = client.post("/pod/revoke")
                .send()
                .await;
            Ok(())
        }
    }
}

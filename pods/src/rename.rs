use crate::client::TytusClient;
use atomek_core::AtomekError;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RenameResult {
    pub status: String,
    #[serde(default)]
    pub pod: Option<serde_json::Value>,
}

/// Set or clear the user-facing display name for an owned pod.
/// Backend: PATCH /pod/{podId}/name { display_name, route_id? } — Provider
/// validates 1..=48 chars, no control chars, and resolves ownership from
/// A2A headers. Pass `None` for display_name to clear the name (server
/// stores NULL).
///
/// `route_id` disambiguates when multiple pods share the same pod_id
/// (e.g. one OpenClaw + one Hermes both allocated as "pod 01" on
/// different droplets). Without it, Provider's findPod picks the first
/// match and the rename lands on the wrong pod. Always pass route_id
/// when known.
pub async fn rename_pod(
    client: &TytusClient,
    pod_id: &str,
    route_id: Option<&str>,
    display_name: Option<&str>,
) -> atomek_core::Result<RenameResult> {
    let mut body = serde_json::json!({ "display_name": display_name });
    if let Some(rid) = route_id.filter(|s| !s.is_empty()) {
        body["route_id"] = serde_json::Value::String(rid.to_string());
    }
    let resp = client
        .patch(&format!("/pod/{}/name", pod_id))
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

    resp.json()
        .await
        .map_err(|e| AtomekError::Other(format!("Failed to parse rename response: {}", e)))
}

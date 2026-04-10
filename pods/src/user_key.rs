use atomek_core::AtomekError;
use serde::Deserialize;
use crate::client::TytusClient;

#[derive(Debug, Deserialize)]
struct UserKeyResponse {
    stable_ai_endpoint: Option<String>,
    stable_user_key: Option<String>,
}

/// Fetch the user's stable API key + stable AI endpoint from the Provider.
///
/// Returns `(endpoint, key)`. The endpoint is the dual-bound WG address
/// (currently `http://10.42.42.1:18080`) and the key is a per-user stable
/// token that persists across pod revoke/reallocate cycles.
///
/// The stable key is created on first pod allocation, so this endpoint
/// returns 404 if the user has never allocated a pod. Callers should
/// handle that by showing a friendly message ("run `tytus connect` first").
pub async fn get_user_key(client: &TytusClient) -> atomek_core::Result<(String, String)> {
    let resp = client.get_with_retry("/pod/user-key").await?;

    if resp.status().as_u16() == 404 {
        return Err(AtomekError::Other(
            "No stable user key yet — run `tytus connect` first".into(),
        ));
    }

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(AtomekError::ApiStatus { status, message: body });
    }

    let data: UserKeyResponse = resp
        .json()
        .await
        .map_err(|e| AtomekError::Other(format!("Failed to parse /pod/user-key: {}", e)))?;

    let endpoint = data
        .stable_ai_endpoint
        .unwrap_or_else(|| "http://10.42.42.1:18080".to_string());
    let key = data
        .stable_user_key
        .ok_or_else(|| AtomekError::Other("stable_user_key missing in response".into()))?;

    Ok((endpoint, key))
}

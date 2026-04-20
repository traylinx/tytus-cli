use atomek_core::AtomekError;
use serde::Deserialize;
use crate::client::TytusClient;

#[derive(Debug, Deserialize)]
struct UserKeyResponse {
    stable_ai_endpoint: Option<String>,
    stable_user_key: Option<String>,
    /// 12-char Crockford base32 slug used as the public-edge subdomain
    /// (`{slug}.tytus.traylinx.com`). Populated when EDGE_PATH_ENABLED=true
    /// on the Provider; null otherwise (CLI falls back to tunnel path).
    /// Sprint: dev/sprints/2026-04-20-public-https-per-pod.md
    #[serde(default)]
    slug: Option<String>,
    /// Pre-built public URL (`https://{slug}.{edge_base}`). Provider builds
    /// it from `slug` + the configured edge base so the CLI doesn't have to
    /// know the edge domain. Null when the edge path is disabled.
    #[serde(default)]
    public_url: Option<String>,
}

/// User-key fields needed by callers. `slug` and `public_url` are populated
/// only when the public-edge path is enabled on the Provider; everything
/// else is the long-standing tunnel-mode contract (preserved verbatim so
/// existing callers keep compiling without changes).
#[derive(Debug, Clone)]
pub struct UserKey {
    pub endpoint: String,
    pub key: String,
    pub slug: Option<String>,
    pub public_url: Option<String>,
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
///
/// **Backward-compat shim** — wraps `get_user_key_full()`. New callers
/// that need `slug`/`public_url` should use the full variant directly.
pub async fn get_user_key(client: &TytusClient) -> atomek_core::Result<(String, String)> {
    let full = get_user_key_full(client).await?;
    Ok((full.endpoint, full.key))
}

/// Fetch the user's stable API key plus the new edge-path fields (slug,
/// public_url). Use this when callers need to make the tunnel-vs-edge
/// routing decision.
pub async fn get_user_key_full(client: &TytusClient) -> atomek_core::Result<UserKey> {
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

    Ok(UserKey {
        endpoint,
        key,
        slug: data.slug,
        public_url: data.public_url,
    })
}

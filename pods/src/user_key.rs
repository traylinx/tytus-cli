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
    /// Per-pod subdomain URL template — the literal string
    /// `https://{slug}-p{pod_id}.{edge_base}` with `{pod_id}` as a placeholder
    /// the CLI substitutes with each pod's two-digit id. Each pod is its
    /// own browser origin so the OpenClaw SPA's localStorage doesn't
    /// collide when the user opens multiple pods in one browser.
    /// Sprint: dev/sprints/2026-04-23-per-pod-subdomain.md
    #[serde(default)]
    pod_public_url_template: Option<String>,
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
    /// Template `https://{slug}-p{pod_id}.{edge_base}` — substitute
    /// `{pod_id}` per pod to get that pod's origin URL.
    pub pod_public_url_template: Option<String>,
}

impl UserKey {
    /// Compose the per-pod subdomain URL for `pod_id` by substituting
    /// into `pod_public_url_template`. Returns `None` when the Provider
    /// hasn't emitted a template (edge disabled / Provider pre-sprint).
    pub fn compose_pod_public_url(&self, pod_id: &str) -> Option<String> {
        let tmpl = self.pod_public_url_template.as_ref()?;
        // pod_id in state.json is already two-digit ("01", "02"); just in
        // case a caller passes "1" or "2", zero-pad it here so the URL
        // matches the edge plugin's canonical form.
        let pod2 = if pod_id.len() < 2 {
            format!("{:0>2}", pod_id)
        } else {
            pod_id.to_string()
        };
        Some(tmpl.replace("{pod_id}", &pod2))
    }
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
        pod_public_url_template: data.pod_public_url_template,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_pod_public_url_substitutes_padded_pod_id() {
        let uk = UserKey {
            endpoint: "http://10.42.42.1:18080".to_string(),
            key: "sk-tytus-user-test".to_string(),
            slug: Some("njc9ctj3zgkn".to_string()),
            public_url: Some("https://njc9ctj3zgkn.tytus.traylinx.com".to_string()),
            pod_public_url_template: Some(
                "https://njc9ctj3zgkn-p{pod_id}.tytus.traylinx.com".to_string(),
            ),
        };
        assert_eq!(
            uk.compose_pod_public_url("02").as_deref(),
            Some("https://njc9ctj3zgkn-p02.tytus.traylinx.com"),
        );
        // Zero-pads a 1-digit id so callers don't have to.
        assert_eq!(
            uk.compose_pod_public_url("4").as_deref(),
            Some("https://njc9ctj3zgkn-p04.tytus.traylinx.com"),
        );
    }

    #[test]
    fn compose_pod_public_url_returns_none_without_template() {
        let uk = UserKey {
            endpoint: "http://10.42.42.1:18080".to_string(),
            key: "sk-tytus-user-test".to_string(),
            slug: None,
            public_url: None,
            pod_public_url_template: None,
        };
        assert!(uk.compose_pod_public_url("02").is_none());
    }
}

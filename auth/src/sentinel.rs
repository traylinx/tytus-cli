//! Sentinel Pass authentication for Atomek.
//!
//! Flow:
//! 1. After device auth, use Bearer token to fetch wannolot pass from
//!    GET /me/wannolot-pass (user-facing, returns secret_key for the user's own pass)
//! 2. Use secret_key + auth_user_id as A2A headers for Provider API calls
//!    X-Agent-Secret-Token: sk_xxx
//!    X-Agent-User-Id: auth_user_id

use atomek_core::{AtomekError, HttpClient};
use serde::Deserialize;
use zeroize::{Zeroize, ZeroizeOnDrop};

const METRICS_API_URL: &str = "https://api.makakoo.com/ma-metrics-wsp-ms/v1/api";

// See docs/SECURITY.md and the long comment in `login.rs::PUBLIC_CLIENT_API_KEY`
// for why this value is intentionally hardcoded. TL;DR: public client identifier,
// not a secret. Every endpoint that consumes it also requires user credentials.
const PUBLIC_CLIENT_API_KEY: &str = "2qQaEiyjeqd0F141C6cFeqpJ353Y7USl";

fn api_key() -> String {
    std::env::var("ATOMEK_API_KEY")
        .or_else(|_| std::env::var("MAKAKOO_API_KEY"))
        .unwrap_or_else(|_| PUBLIC_CLIENT_API_KEY.to_string())
}

/// Credentials for calling the Tytus Provider API.
/// Contains the raw secret_key from the wannolot SentinelPass.
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct SentinelCredentials {
    pub pass_id: String,
    pub secret_key: String,       // Raw sk_xxx — goes in X-Agent-Secret-Token
    pub agent_user_id: String,    // auth_user_id — goes in X-Agent-User-Id
    pub tier: String,
    pub organization_id: String,
}

#[derive(Debug, Clone)]
pub struct PlanStatus {
    pub has_plan: bool,
    pub tier_name: Option<String>,
    pub max_pods: u32,
    pub expires_at: Option<String>,
}

// ── Response types ──

#[derive(Deserialize)]
#[allow(dead_code)] // serde struct: keep all upstream fields even if currently unused
struct WannolotPassResponse {
    has_pass: bool,
    #[serde(default)]
    pass_id: Option<String>,
    #[serde(default)]
    secret_key: Option<String>,
    #[serde(default)]
    agent_user_id: Option<String>,
    #[serde(default)]
    organization_id: Option<String>,
    #[serde(default)]
    tier: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

// ── Public API ──

/// Fetch the user's wannolot Sentinel Pass credentials.
/// Calls GET /me/wannolot-pass with the user's Bearer token.
/// Returns the raw secret_key needed for Provider A2A authentication.
pub async fn fetch_wannolot_pass(
    http: &HttpClient,
    access_token: &str,
) -> atomek_core::Result<SentinelCredentials> {
    let url = format!("{}/me/wannolot-pass", METRICS_API_URL);

    let key = api_key();
    let resp = http.send_with_retry(|| {
        http.get(&url)
            .header("Api-Key", &key)
            .header("Authorization", format!("Bearer {}", access_token))
    }).await?;

    let body: WannolotPassResponse = resp.json().await
        .map_err(|e| AtomekError::Other(format!("Failed to parse wannolot-pass response: {}", e)))?;

    if !body.has_pass {
        let msg = body.message.unwrap_or_else(|| "No Tytus subscription".into());
        tracing::info!("No wannolot pass: {}", msg);
        return Err(AtomekError::NoSubscription);
    }

    let secret_key = body.secret_key
        .ok_or_else(|| AtomekError::Other("Pass exists but no secret_key returned".into()))?;
    let agent_user_id = body.agent_user_id
        .ok_or_else(|| AtomekError::Other("Pass exists but no agent_user_id returned".into()))?;
    let pass_id = body.pass_id.unwrap_or_default();
    let tier = body.tier.unwrap_or_else(|| "explorer".into());
    let org_id = body.organization_id.unwrap_or_default();

    tracing::info!(pass_id = %pass_id, tier = %tier, "Wannolot pass obtained");

    Ok(SentinelCredentials {
        pass_id,
        secret_key,
        agent_user_id,
        tier,
        organization_id: org_id,
    })
}

/// Fetch plan status — piggybacks on the Provider's /pod/status response.
/// Called AFTER we have SentinelCredentials, since plan info is in the pod status.
pub fn plan_from_pod_status(has_plan: bool, tier: Option<String>, max_pods: u32, expires_at: Option<String>) -> PlanStatus {
    PlanStatus { has_plan, tier_name: tier, max_pods, expires_at }
}

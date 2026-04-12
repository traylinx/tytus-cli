//! Device Authorization Grant Flow for Atomek.
//!
//! Same flow as traylinx-cli: user approves login in their browser,
//! Atomek polls for approval and receives tokens. No password ever touches Atomek.
//!
//! Flow:
//! 1. POST /devices → get device_id + verification_uri
//! 2. Open browser → user approves
//! 3. Poll GET /devices/{id}/status until "authorized"
//! 4. Extract access_token + refresh_token
//!
//! Sentinel API: https://sentinel.traylinx.com

use atomek_core::{AtomekError, HttpClient};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const SENTINEL_URL: &str = "https://sentinel.traylinx.com";
// Uses the same client name as traylinx-cli (registered on Sentinel)
const CLIENT_NAME: &str = "traylinx-cli";
const POLL_INTERVAL: Duration = Duration::from_secs(2);
const MAX_POLL_DURATION: Duration = Duration::from_secs(600); // 10 minutes

/// Result of initiating a device auth session.
#[derive(Debug, Clone, Serialize)]
pub struct DeviceAuthSession {
    pub device_id: String,
    pub verification_uri: String,
    pub user_code: String,
    pub expires_in: u64,
}

/// Result of a successful device authorization.
#[derive(Debug)]
pub struct DeviceAuthResult {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub user: DeviceAuthUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAuthUser {
    pub id: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

// ── API response types ──

#[derive(Deserialize)]
struct CreateDeviceResponse {
    device_id: String,
    verification_uri: String,
    #[serde(default)]
    verification_uri_complete: Option<String>,
    user_code: String,
    #[serde(default = "default_expires_in")]
    expires_in: u64,
}

fn default_expires_in() -> u64 { 600 }

#[derive(Deserialize)]
struct DeviceStatusResponse {
    status: String,
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    user: Option<DeviceAuthUser>,
}

#[derive(Deserialize)]
struct RefreshResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
}

// ── Public API ──

/// Step 1: Initiate a device auth session.
/// Returns session info including the URL to open in the browser.
pub async fn create_device_session(http: &HttpClient) -> atomek_core::Result<DeviceAuthSession> {
    let url = format!("{}/devices", SENTINEL_URL);

    let resp = http.send_with_retry(|| {
        http.post(&url)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "client": CLIENT_NAME }))
    }).await?;

    let body: CreateDeviceResponse = resp.json().await
        .map_err(|e| AtomekError::Other(format!("Failed to parse device session: {}", e)))?;

    Ok(DeviceAuthSession {
        device_id: body.device_id,
        // Use the complete URI (with code pre-filled) if available
        verification_uri: body.verification_uri_complete.unwrap_or(body.verification_uri),
        user_code: body.user_code,
        expires_in: body.expires_in,
    })
}

/// Step 2+3: Poll for user approval.
/// Blocks until authorized, denied, or timeout.
/// The `on_status` callback is called on each poll so the UI can update.
pub async fn poll_for_authorization(
    http: &HttpClient,
    device_id: &str,
    mut on_status: impl FnMut(&str),
) -> atomek_core::Result<DeviceAuthResult> {
    let url = format!("{}/devices/{}/status", SENTINEL_URL, device_id);
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > MAX_POLL_DURATION {
            return Err(AtomekError::Other("Device authorization timed out (10 minutes)".into()));
        }

        tokio::time::sleep(POLL_INTERVAL).await;

        let resp = match http.get(&url).send().await {
            Ok(r) => r,
            Err(_) => continue, // Network hiccup, keep polling
        };

        if !resp.status().is_success() {
            continue; // Server hiccup, keep polling
        }

        let body: DeviceStatusResponse = match resp.json().await {
            Ok(b) => b,
            Err(_) => continue,
        };

        on_status(&body.status);

        match body.status.as_str() {
            "authorized" => {
                let access_token = body.access_token
                    .ok_or_else(|| AtomekError::Other("Authorized but no access_token".into()))?;
                let refresh_token = body.refresh_token
                    .ok_or_else(|| AtomekError::Other("Authorized but no refresh_token".into()))?;
                let user = body.user
                    .ok_or_else(|| AtomekError::Other("Authorized but no user info".into()))?;

                return Ok(DeviceAuthResult {
                    access_token,
                    refresh_token,
                    expires_in: body.expires_in.unwrap_or(900),
                    user,
                });
            }
            "denied" => {
                return Err(AtomekError::Other("Login was denied by the user".into()));
            }
            "expired" => {
                return Err(AtomekError::AuthExpired);
            }
            "pending" => {
                // Keep polling
            }
            other => {
                tracing::debug!("Unknown device status: {}", other);
            }
        }
    }
}

/// Refresh an access token via Sentinel.
/// Tries the CLI-specific endpoint first, falls back to OAuth standard.
pub async fn refresh_access_token(
    http: &HttpClient,
    refresh_tok: &str,
) -> atomek_core::Result<DeviceAuthResult> {
    // Try CLI-specific refresh first
    let url = format!("{}/devices/refresh", SENTINEL_URL);
    let resp = http.post(&url)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "refresh_token": refresh_tok }))
        .send()
        .await;

    if let Ok(r) = resp {
        if r.status().is_success() {
            if let Ok(body) = r.json::<RefreshResponse>().await {
                return Ok(DeviceAuthResult {
                    access_token: body.access_token,
                    refresh_token: body.refresh_token.unwrap_or_else(|| refresh_tok.to_string()),
                    expires_in: body.expires_in.unwrap_or(900),
                    user: DeviceAuthUser {
                        id: String::new(),
                        email: String::new(),
                        first_name: None,
                        last_name: None,
                    },
                });
            }
        }
    }

    // Fallback to OAuth standard endpoint
    let url = format!("{}/oauth/token", SENTINEL_URL);
    let resp = http.post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!(
            "grant_type=refresh_token&refresh_token={}",
            urlencoding::encode(refresh_tok)
        ))
        .send()
        .await
        .map_err(|e| AtomekError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(AtomekError::AuthExpired);
    }

    let body: RefreshResponse = resp.json().await
        .map_err(|e| AtomekError::Other(format!("Failed to parse refresh response: {}", e)))?;

    Ok(DeviceAuthResult {
        access_token: body.access_token,
        refresh_token: body.refresh_token.unwrap_or_else(|| refresh_tok.to_string()),
        expires_in: body.expires_in.unwrap_or(900),
        user: DeviceAuthUser {
            id: String::new(),
            email: String::new(),
            first_name: None,
            last_name: None,
        },
    })
}

/// Server-side token validation result.
pub struct TokenValidation {
    /// Seconds until the token expires, as reported by the server.
    pub expires_in: u64,
}

/// Validate an access token against Sentinel's server-side check.
/// Returns Ok(TokenValidation) if valid, Err if expired/revoked/unreachable.
/// Uses GET /oauth/token/info — response includes expiresIn for clock-skew correction.
pub async fn validate_token(
    http: &HttpClient,
    access_token: &str,
) -> atomek_core::Result<TokenValidation> {
    let url = format!("{}/oauth/token/info", SENTINEL_URL);
    let resp = http.get(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| AtomekError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(AtomekError::AuthExpired);
    }

    // Parse expiresIn from the nested response: { "data": { "attributes": { "expiresIn": N } } }
    let body: serde_json::Value = resp.json().await
        .map_err(|e| AtomekError::Other(format!("Failed to parse token info: {}", e)))?;

    let expires_in = body
        .pointer("/data/attributes/expiresIn")
        .and_then(|v| v.as_u64())
        .unwrap_or(900); // Conservative fallback: 15 minutes

    Ok(TokenValidation { expires_in })
}

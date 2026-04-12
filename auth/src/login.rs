use atomek_core::{AtomekError, HttpClient};
use serde::{Deserialize, Serialize};

const AUTH_API_URL: &str = "https://api.makakoo.com/ma-authentication-ms/v1/api";
const CLIENT_ID: &str = "zsel0J1YBT6g0QXoqBpBiJt-gpRQ0wHQwZDKlGds4zg";

// ── Public client identifier (INTENTIONALLY hardcoded) ─────────────────────
//
// This is the Rails `Api-Key` header value. It is a **public client
// identifier**, not a secret. It identifies "request is coming from the Tytus
// CLI" for telemetry, per-client rate limiting, and feature gating. It is
// shipped in every public CLI binary exactly like:
//   - Firebase Web SDK API keys
//   - Auth0 client_id values
//   - Stripe publishable keys (pk_live_*)
//
// This is safe because every endpoint that consumes this value ALSO requires
// user credentials on top of it:
//   - /auth/login      → user email + password in body
//   - /auth/refresh    → user refresh token in body
//   - /me/wannolot-pass → user OAuth Bearer in Authorization header
//
// The Api-Key alone grants zero privileges on any of these endpoints. An
// attacker who extracts it from the binary gains exactly the same access
// surface a user gets by downloading the CLI: none, until they provide their
// own credentials.
//
// If this assumption is ever invalidated by a Rails-side change (e.g. adding
// an endpoint that trusts Api-Key without user creds), this ceases to be a
// public client ID and becomes a leaked secret. That would be a Rails-side
// regression — catch it in Rails review, not here.
//
// See docs/PENTEST-RESULTS-2026-04-12.md finding H1 and docs/SECURITY.md.
const PUBLIC_CLIENT_API_KEY: &str = "2qQaEiyjeqd0F141C6cFeqpJ353Y7USl";

fn api_key() -> String {
    // Env override lets us ship a different value for dev/staging builds
    // without recompiling. Production binaries always fall through to the
    // public client identifier above.
    std::env::var("ATOMEK_API_KEY")
        .or_else(|_| std::env::var("MAKAKOO_API_KEY"))
        .unwrap_or_else(|_| PUBLIC_CLIENT_API_KEY.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub auth_user_id: Option<String>,
}

#[derive(Debug)]
pub struct LoginResult {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserInfo,
}

#[derive(Serialize)]
struct LoginRequest {
    email: String,
    password: String,
    client_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
    user: UserInfo,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RefreshRequest {
    refresh_token: String,
}

pub async fn login(http: &HttpClient, email: &str, password: &str) -> atomek_core::Result<LoginResult> {
    let url = format!("{}/auth/login", AUTH_API_URL);

    let resp = http.send_with_retry(|| {
        http.post(&url)
            .header("Api-Key", &api_key())
            .header("Content-Type", "application/json")
            .json(&LoginRequest {
                email: email.to_string(),
                password: password.to_string(),
                client_id: CLIENT_ID.to_string(),
            })
    }).await;

    match resp {
        Ok(r) => {
            let body: LoginResponse = r.json().await
                .map_err(|e| AtomekError::Other(format!("Failed to parse login response: {}", e)))?;
            Ok(LoginResult {
                access_token: body.access_token,
                refresh_token: body.refresh_token,
                user: body.user,
            })
        }
        Err(AtomekError::ApiStatus { status: 401, .. }) => Err(AtomekError::InvalidCredentials),
        Err(AtomekError::ApiStatus { status: 403, .. }) => Err(AtomekError::AccountLocked),
        Err(e) => Err(e),
    }
}

pub async fn refresh_token(http: &HttpClient, refresh_tok: &str) -> atomek_core::Result<LoginResult> {
    let url = format!("{}/auth/refresh", AUTH_API_URL);

    let resp = http.send_with_retry(|| {
        http.post(&url)
            .header("Api-Key", &api_key())
            .header("Content-Type", "application/json")
            .json(&RefreshRequest {
                refresh_token: refresh_tok.to_string(),
            })
    }).await;

    match resp {
        Ok(r) => {
            let body: LoginResponse = r.json().await
                .map_err(|e| AtomekError::Other(format!("Failed to parse refresh response: {}", e)))?;
            Ok(LoginResult {
                access_token: body.access_token,
                refresh_token: body.refresh_token,
                user: body.user,
            })
        }
        Err(AtomekError::ApiStatus { status: 401, .. }) => Err(AtomekError::AuthExpired),
        Err(e) => Err(e),
    }
}

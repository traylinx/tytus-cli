use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const STATE_DIR: &str = "tytus";
const STATE_FILE: &str = "state.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliState {
    pub email: Option<String>,
    /// Refresh token is loaded from the OS keychain at `load()` time and is
    /// **never serialized back to disk**. Legacy state.json files that still
    /// contain a refresh_token are migrated on first load (see `load()`).
    ///
    /// See docs/PENTEST-RESULTS-2026-04-12.md finding E2/H2: keeping the RT
    /// in state.json let any same-user process read it and own the session
    /// permanently. Keychain requires explicit per-call access.
    #[serde(default, skip_serializing)]
    pub refresh_token: Option<String>,
    pub access_token: Option<String>,
    pub expires_at_ms: Option<i64>,
    pub secret_key: Option<String>,
    pub agent_user_id: Option<String>,
    pub organization_id: Option<String>,
    pub tier: Option<String>,
    pub pods: Vec<PodEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodEntry {
    pub pod_id: String,
    pub droplet_id: String,
    pub droplet_ip: Option<String>,
    pub ai_endpoint: Option<String>,
    pub pod_api_key: Option<String>,
    pub agent_type: Option<String>,
    pub agent_endpoint: Option<String>,
    pub tunnel_iface: Option<String>,
    // Stable endpoint + per-user stable API key for local tools.
    // The endpoint is always http://10.42.42.1:18080 (dual-bound WG address)
    // and the key persists across pod revoke/reallocate cycles.
    #[serde(default)]
    pub stable_ai_endpoint: Option<String>,
    #[serde(default)]
    pub stable_user_key: Option<String>,
    /// Per-user public-edge subdomain slug (12-char Crockford base32).
    /// Populated by Phase 1 (`/pod/user-key` response). Combined with
    /// `pod_id` becomes the full URL `https://<slug>.tytus.traylinx.com/p/<NN>`.
    #[serde(default)]
    pub edge_slug: Option<String>,
    /// Pre-built public URL — `https://<slug>.tytus.traylinx.com`. Provider
    /// builds it from the slug + edge base so the CLI doesn't have to know
    /// the edge domain. Used by `tytus env` to surface the public URL by
    /// default; `--tunnel` forces the legacy WG path.
    #[serde(default)]
    pub edge_public_url: Option<String>,
    /// Agent's internal gateway auth token (e.g. OpenClaw's
    /// `gateway.auth.token`). Fetched from the pod at agent install
    /// time and cached here so the forwarder can auto-inject it on
    /// every request — user never sees the "paste token" form.
    /// Rotated when the agent is reinstalled (old token goes stale).
    #[serde(default)]
    pub gateway_token: Option<String>,
}

impl CliState {
    pub fn state_path() -> PathBuf {
        // When running elevated (sudo/osascript), TYTUS_REAL_HOME points to the
        // original user's home so we read THEIR state, not root's.
        // Fallback chain: TYTUS_REAL_HOME → SUDO_USER's home → dirs::config_dir()
        let config = if let Ok(real_home) = std::env::var("TYTUS_REAL_HOME") {
            PathBuf::from(real_home).join(if cfg!(target_os = "macos") {
                "Library/Application Support"
            } else {
                ".config"
            })
        } else if let Ok(sudo_user) = std::env::var("SUDO_USER") {
            // Running under plain sudo without TYTUS_REAL_HOME
            if cfg!(target_os = "macos") {
                PathBuf::from(format!("/Users/{}/Library/Application Support", sudo_user))
            } else {
                PathBuf::from(format!("/home/{}/.config", sudo_user))
            }
        } else {
            dirs::config_dir().unwrap_or_else(|| PathBuf::from("."))
        };
        let dir = config.join(STATE_DIR);
        std::fs::create_dir_all(&dir).ok();
        dir.join(STATE_FILE)
    }

    /// Parse state.json without touching the OS keychain. Used by paths
    /// that need a fast, side-effect-free snapshot — notably the daemon's
    /// status RPC, which is polled ~every 1.5s by the tray and must not
    /// block on a 3s keychain timeout when the ACL dialog is pending.
    ///
    /// The returned state has `refresh_token == None` even if one exists
    /// in the keychain. Callers that need the RT (e.g. `ensure_token`)
    /// must use `load()` instead.
    pub fn load_file_only() -> Self {
        let path = Self::state_path();
        let raw = std::fs::read_to_string(&path).ok();
        raw.as_deref()
            .and_then(|data| serde_json::from_str(data).ok())
            .unwrap_or_default()
    }

    pub fn load() -> Self {
        let path = Self::state_path();
        let raw = std::fs::read_to_string(&path).ok();
        let mut state: Self = raw.as_deref()
            .and_then(|data| serde_json::from_str(data).ok())
            .unwrap_or_default();

        // refresh_token is keychain-only — see field comment.
        //
        // Migration: if state.json still contains a refresh_token field (legacy
        // file from before this commit), copy it into the OS keychain and
        // rewrite the file immediately without the token. We do this eagerly
        // in load() rather than waiting for a natural save() call because
        // command paths that fail early (e.g. `tytus status` on an expired
        // session) never reach a save(), and we must not leave plaintext
        // tokens on disk one millisecond longer than necessary.
        //
        // If the keychain write fails — e.g. on a newly signed binary the user
        // hasn't approved yet — we leave the file alone so the user is not
        // locked out. Next successful run retries.
        let file_had_rt = raw
            .as_deref()
            .map(|s| s.contains("\"refresh_token\""))
            .unwrap_or(false);

        if let Some(ref email) = state.email.clone() {
            if let Some(ref rt) = state.refresh_token.clone() {
                let stored = atomek_auth::KeychainStore::store_refresh_token(email, rt).is_ok();
                if stored && file_had_rt {
                    // Strip refresh_token from disk right now. `skip_serializing`
                    // on the field guarantees the rewritten file won't contain it.
                    let _ = state.save_critical();
                }
            } else if let Ok(rt) = atomek_auth::KeychainStore::get_refresh_token(email) {
                state.refresh_token = Some(rt);
            }
        }

        state
    }

    pub fn save(&self) {
        let path = Self::state_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, &data);
            // Restrict permissions: owner-only read/write (contains tokens)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
            }
        }
    }

    /// Save state to disk, returning an error on failure.
    /// Use this after token rotation — the old refresh token is dead server-side,
    /// so failure to persist the new one means the user is locked out on next launch.
    pub fn save_critical(&self) -> Result<(), std::io::Error> {
        let path = Self::state_path();
        let data = serde_json::to_string_pretty(self)
            .map_err(std::io::Error::other)?;
        std::fs::write(&path, &data)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        *self = Self::default();
        self.save();
    }

    /// True when we have a usable Tytus session — either a refresh token
    /// (we can mint fresh access tokens) OR a still-valid access token
    /// (we can call APIs until it expires, at which point the daemon
    /// will try to refresh).
    ///
    /// Falling back to a valid-AT-only state matters at cold boot: on
    /// macOS, the keychain ACL can take a few seconds to approve after
    /// login, and the LaunchAgent's `get_refresh_token` times out in
    /// 3s. If we insisted on the refresh token being present, autostart
    /// would silently fail even though we have a perfectly good access
    /// token in state.json that lasts ~1h. The daemon will keep
    /// retrying the keychain in the background; once it unblocks, the
    /// refresh token is recovered and normal flow resumes.
    ///
    /// Semantically: "logged in" means "has email + means to call the
    /// API". RT and a current AT both satisfy that; only the absence
    /// of BOTH means the user really needs to `tytus login`.
    pub fn is_logged_in(&self) -> bool {
        let has_email = self.email.as_ref().is_some_and(|e| !e.is_empty());
        let has_rt = self.refresh_token.as_ref().is_some_and(|t| !t.is_empty());
        has_email && (has_rt || self.has_valid_token())
    }

    pub fn has_valid_token(&self) -> bool {
        if let (Some(_), Some(exp)) = (&self.access_token, self.expires_at_ms) {
            let now = chrono::Utc::now().timestamp_millis();
            (now + 300_000) < exp // 5 min buffer
        } else {
            false
        }
    }
}

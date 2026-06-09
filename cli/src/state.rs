use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;

const STATE_DIR: &str = "tytus";
const STATE_FILE: &str = "state.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct AccountProfile {
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub expires_at_ms: Option<i64>,
    #[serde(default)]
    pub secret_key: Option<String>,
    #[serde(default)]
    pub agent_user_id: Option<String>,
    #[serde(default)]
    pub organization_id: Option<String>,
    #[serde(default)]
    pub tier: Option<String>,
    #[serde(default)]
    pub pods: Vec<PodEntry>,
    /// Backend identifier for this account's `device_sessions` row.
    /// Populated from the device-code grant response on `tytus login`
    /// (see sprint tytus-account-aware-detection-2026-05-22 Phase 1).
    /// Surfaced by the tray daemon's `/api/whoami` so the web app can
    /// reconcile "the local install at :4242 is paired to *this*
    /// device session" vs the user's other devices.
    #[serde(default)]
    pub device_session_id: Option<i64>,
    #[serde(default)]
    pub last_active_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CliState {
    #[serde(default)]
    pub schema_version: u8,
    #[serde(default)]
    pub active_email: Option<String>,
    #[serde(default)]
    pub accounts: Vec<AccountProfile>,

    #[serde(default)]
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
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub expires_at_ms: Option<i64>,
    #[serde(default)]
    pub secret_key: Option<String>,
    #[serde(default)]
    pub agent_user_id: Option<String>,
    #[serde(default)]
    pub organization_id: Option<String>,
    #[serde(default)]
    pub tier: Option<String>,
    #[serde(default)]
    pub pods: Vec<PodEntry>,
    /// Backend identifier for this install's `device_sessions` row.
    /// Persisted from the device-code grant response on `tytus login`
    /// (see sprint tytus-account-aware-detection-2026-05-22 Phase 1).
    /// Exposed via the tray daemon's `/api/whoami` so the web admin
    /// can reconcile the local install with the user's device list.
    /// Pre-Phase-2 backends return null here; the field stays None.
    #[serde(default)]
    pub device_session_id: Option<i64>,

    // ------------------------------------------------------------------
    // Local Cortex (sprint: 2026-05-21-chat-with-pods-local-cortex-parity).
    // All fields default to None / "cloud" so v0.6.x state files load
    // unchanged. Tray daemon reads `cortex_profile` to pick the chat upstream.
    // ------------------------------------------------------------------
    /// `None` (default) | Some("cloud") | Some("local"). Drives
    /// `/api/pods/{NN}/cortex/chat` upstream resolution in the tray daemon.
    /// `None` is treated identically to "cloud" by the daemon — kept
    /// optional so older state.json files load unchanged.
    #[serde(default)]
    pub cortex_profile: Option<String>,
    /// Per-user `ctx_*` token minted by local Cortex `POST /v1/users`.
    /// Used for `/v1/*` user-scoped endpoints only — NOT for `/tytus/chat`
    /// (that uses `cortex_internal_service_token`).
    #[serde(default)]
    pub cortex_local_token: Option<String>,
    /// UUID generated on first local Cortex enable. Identifies the local
    /// Cortex user across token rotations.
    #[serde(default)]
    pub cortex_local_user_id: Option<String>,
    /// Service-to-service shared secret mirrored from the local Cortex
    /// docker-compose env (`INTERNAL_SERVICE_TOKEN`). The tray daemon
    /// presents it on `/tytus/chat` calls. Never user-visible.
    #[serde(default)]
    pub cortex_internal_service_token: Option<String>,
    /// Host port the local Cortex API binds to (default 8098).
    #[serde(default)]
    pub cortex_local_port: Option<u16>,
    /// Pinned Cortex image tag for the local install. Bumped via
    /// `tytus cortex upgrade`.
    #[serde(default)]
    pub cortex_local_version_pinned: Option<String>,
    /// ISO 8601 timestamp of the most recent successful `cortex up`.
    #[serde(default)]
    pub cortex_local_started_at: Option<String>,
}

/// Canonical reading of `cortex_profile` — None and "cloud" both mean cloud.
pub fn cortex_profile_is_local(profile: &Option<String>) -> bool {
    matches!(profile.as_deref(), Some("local"))
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PodEntry {
    #[serde(default)]
    pub pod_id: String,
    /// Provider route identity. Pod numbers are only unique per droplet;
    /// route_id is globally unique for Traylinx/Tytus UI and chat routing.
    #[serde(default)]
    pub route_id: Option<String>,
    #[serde(default)]
    pub droplet_id: String,
    /// Stable selected-agent identity used to unify memory across channels.
    /// Falls back to route_id in newer tray surfaces when older Provider
    /// status snapshots do not yet emit this field.
    #[serde(default)]
    pub agent_identity_id: Option<String>,
    #[serde(default)]
    pub droplet_ip: Option<String>,
    #[serde(default)]
    pub ai_endpoint: Option<String>,
    #[serde(default)]
    pub pod_api_key: Option<String>,
    #[serde(default)]
    pub agent_type: Option<String>,
    #[serde(default)]
    pub agent_units: Option<u32>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub agent_endpoint: Option<String>,
    #[serde(default)]
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
    /// Per-pod subdomain URL (`https://<slug>-p<NN>.tytus.traylinx.com`).
    /// Each pod gets its own browser origin so the OpenClaw SPA's
    /// localStorage / cookies don't collide across pods when the user opens
    /// multiple pods in one browser. Populated from the allocation response
    /// (sprint 2026-04-23). When None, callers compose the legacy
    /// `<edge_public_url>/p/<pod_id>` shape as a fallback.
    #[serde(default)]
    pub pod_public_url: Option<String>,
}

impl AccountProfile {
    fn from_snapshot(state: &CliState) -> Option<Self> {
        let email = state.email.as_ref()?.trim();
        if email.is_empty() {
            return None;
        }
        Some(Self {
            email: email.to_string(),
            access_token: state.access_token.clone(),
            expires_at_ms: state.expires_at_ms,
            secret_key: state.secret_key.clone(),
            agent_user_id: state.agent_user_id.clone(),
            organization_id: state.organization_id.clone(),
            tier: state.tier.clone(),
            pods: state.pods.clone(),
            device_session_id: state.device_session_id,
            last_active_at: None,
        })
    }

    fn apply_to_snapshot(&self, state: &mut CliState) {
        state.email = Some(self.email.clone());
        state.access_token = self.access_token.clone();
        state.expires_at_ms = self.expires_at_ms;
        state.secret_key = self.secret_key.clone();
        state.agent_user_id = self.agent_user_id.clone();
        state.organization_id = self.organization_id.clone();
        state.tier = self.tier.clone();
        state.pods = self.pods.clone();
        state.device_session_id = self.device_session_id;
    }
}

pub struct StateMutationLock {
    #[allow(dead_code)]
    file: std::fs::File,
}

impl StateMutationLock {
    pub fn acquire() -> Result<Self, std::io::Error> {
        let path = CliState::state_lock_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)?;
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
            if rc != 0 {
                return Err(std::io::Error::last_os_error());
            }
        }
        Ok(Self { file })
    }
}

impl Drop for StateMutationLock {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            let _ = unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) };
        }
    }
}

impl CliState {
    pub fn state_path() -> PathBuf {
        if let Some(path) = std::env::var_os("TYTUS_STATE_PATH") {
            let path = PathBuf::from(path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            return path;
        }

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

    #[allow(dead_code)]
    pub fn state_lock_path() -> PathBuf {
        Self::state_path()
            .parent()
            .map(|p| p.join("state.lock"))
            .unwrap_or_else(|| PathBuf::from("state.lock"))
    }

    #[allow(dead_code)]
    pub fn from_legacy_v1(raw_json: &str) -> Self {
        let mut state: Self = serde_json::from_str(raw_json).unwrap_or_default();
        state.schema_version = 1;
        state.normalize_after_deserialize();
        state
    }

    #[allow(dead_code)]
    pub fn is_v1_shape(raw_json: &str) -> bool {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(raw_json) else {
            return false;
        };
        !v.get("accounts").is_some_and(|a| a.is_array())
            && (v.get("email").is_some() || v.get("pods").is_some())
    }

    fn normalize_after_deserialize(&mut self) {
        if self.schema_version == 0 {
            self.schema_version = if self.accounts.is_empty() { 1 } else { 2 };
        }

        if self.accounts.is_empty() {
            if let Some(profile) = AccountProfile::from_snapshot(self) {
                self.active_email = Some(profile.email.clone());
                self.accounts.push(profile);
            }
        }

        if self.active_email.is_none() {
            self.active_email = self
                .email
                .as_ref()
                .filter(|e| !e.is_empty())
                .cloned()
                .or_else(|| self.accounts.first().map(|a| a.email.clone()));
        }

        if self
            .active_email
            .as_ref()
            .is_some_and(|email| !self.accounts.iter().any(|a| &a.email == email))
        {
            self.active_email = self.accounts.first().map(|a| a.email.clone());
        }

        self.schema_version = 2;
        self.sync_active_snapshot();
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
        let mut state: Self = raw
            .as_deref()
            .and_then(|data| serde_json::from_str(data).ok())
            .unwrap_or_default();
        state.normalize_after_deserialize();
        state.refresh_token = None;
        state
    }

    pub fn load() -> Self {
        let path = Self::state_path();
        let raw = std::fs::read_to_string(&path).ok();
        let mut state: Self = raw
            .as_deref()
            .and_then(|data| serde_json::from_str(data).ok())
            .unwrap_or_default();
        state.normalize_after_deserialize();

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

    fn normalized_for_save(&self) -> Self {
        let mut state = self.clone();
        state.schema_version = 2;
        state.sync_account_from_snapshot();
        state.sync_active_snapshot();
        state.refresh_token = None;
        state
    }

    pub fn save(&self) {
        let _ = self.save_critical();
    }

    /// Save state to disk, returning an error on failure.
    /// Use this after token rotation — the old refresh token is dead server-side,
    /// so failure to persist the new one means the user is locked out on next launch.
    pub fn save_critical(&self) -> Result<(), std::io::Error> {
        let path = Self::state_path();
        let parent = path.parent().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "state path has no parent")
        })?;
        std::fs::create_dir_all(parent)?;

        let state = self.normalized_for_save();
        let data = serde_json::to_vec_pretty(&state).map_err(std::io::Error::other)?;
        let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
        tmp.write_all(&data)?;
        tmp.flush()?;
        tmp.as_file().sync_all()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(tmp.path(), std::fs::Permissions::from_mode(0o600))?;
        }
        tmp.persist(&path).map_err(|e| e.error)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }
        if let Ok(dir) = std::fs::File::open(parent) {
            let _ = dir.sync_all();
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        *self = Self::default();
        self.save();
    }

    pub fn active_account(&self) -> Option<&AccountProfile> {
        let email = self.active_email.as_ref()?;
        self.accounts.iter().find(|a| &a.email == email)
    }

    #[allow(dead_code)]
    pub fn active_account_mut(&mut self) -> Option<&mut AccountProfile> {
        let email = self.active_email.clone()?;
        self.accounts.iter_mut().find(|a| a.email == email)
    }

    pub fn find_account(&self, email: &str) -> Option<&AccountProfile> {
        self.accounts.iter().find(|a| a.email == email)
    }

    #[allow(dead_code)]
    pub fn find_pod(&self, pod_id: Option<&str>) -> Option<&PodEntry> {
        if let Some(pid) = pod_id {
            self.pods.iter().find(|p| p.pod_id == pid)
        } else {
            self.pods
                .iter()
                .find(|p| p.tunnel_iface.is_some())
                .or_else(|| self.pods.first())
        }
    }

    pub fn sync_active_snapshot(&mut self) {
        if let Some(account) = self.active_account().cloned() {
            account.apply_to_snapshot(self);
        } else if self.active_email.is_none() {
            self.email = None;
            self.access_token = None;
            self.expires_at_ms = None;
            self.secret_key = None;
            self.agent_user_id = None;
            self.organization_id = None;
            self.tier = None;
            self.pods.clear();
            self.device_session_id = None;
        }
    }

    pub fn sync_account_from_snapshot(&mut self) {
        let Some(profile) = AccountProfile::from_snapshot(self) else {
            return;
        };
        if self.active_email.is_none() {
            self.active_email = Some(profile.email.clone());
        }
        let active_email = self
            .active_email
            .clone()
            .unwrap_or_else(|| profile.email.clone());
        if active_email != profile.email && self.find_account(&active_email).is_some() {
            return;
        }
        self.active_email = Some(profile.email.clone());
        if let Some(existing) = self.accounts.iter_mut().find(|a| a.email == profile.email) {
            *existing = profile;
        } else {
            self.accounts.push(profile);
        }
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
    /// API". RT and a current AT both satisfy that. For Tytus pod status,
    /// the saved Sentinel pass also satisfies it: browser OAuth may need a
    /// keychain refresh while the user still has a valid Tytus pass for
    /// `/pod/status`, and local UI must not collapse to "0 pods".
    pub fn is_logged_in(&self) -> bool {
        let has_email = self.email.as_ref().is_some_and(|e| !e.is_empty());
        let has_rt = self.refresh_token.as_ref().is_some_and(|t| !t.is_empty());
        let has_tytus_pass = self.secret_key.as_ref().is_some_and(|s| !s.is_empty())
            && self.agent_user_id.as_ref().is_some_and(|s| !s.is_empty());
        has_email && (has_rt || self.has_valid_token() || has_tytus_pass)
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

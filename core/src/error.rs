use thiserror::Error;

/// Unified error type for the entire Atomek application.
/// Inspired by claurst's ClaudeError — semantic variants with classification methods.
#[derive(Error, Debug)]
pub enum AtomekError {
    // ── Auth ──
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Account locked")]
    AccountLocked,

    #[error("Authentication expired")]
    AuthExpired,

    #[error("No active Sentinel Pass")]
    NoSentinelPass,

    #[error("No Tytus subscription")]
    NoSubscription,

    // ── Pod ──
    #[error("Pod limit reached: {current}/{limit}")]
    PodLimitReached { limit: u32, current: u32 },

    #[error("No capacity available (retry in {retry_after}s)")]
    NoCapacity { retry_after: u64 },

    #[error("No active pod")]
    NoPod,

    #[error("Pod config not ready")]
    ConfigNotReady,

    // ── Tunnel ──
    #[error("Tunnel failed: {0}")]
    Tunnel(String),

    #[error("Elevated privileges required")]
    PrivilegesRequired,

    #[error("WireGuard handshake timeout")]
    HandshakeTimeout,

    // ── Network ──
    #[error("Network error: {0}")]
    Network(String),

    #[error("API error {status}: {message}")]
    ApiStatus { status: u16, message: String },

    #[error("Rate limited")]
    RateLimited,

    // ── System ──
    #[error("Keychain error: {0}")]
    Keychain(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("{0}")]
    Other(String),
}

impl AtomekError {
    /// Should the caller retry this operation?
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AtomekError::Network(_)
                | AtomekError::RateLimited
                | AtomekError::NoCapacity { .. }
                | AtomekError::ConfigNotReady
                | AtomekError::ApiStatus { status: 429, .. }
                | AtomekError::ApiStatus { status: 502, .. }
                | AtomekError::ApiStatus { status: 503, .. }
                | AtomekError::ApiStatus { status: 529, .. }
        )
    }

    /// Is this an auth error that requires re-login?
    pub fn is_auth_error(&self) -> bool {
        matches!(
            self,
            AtomekError::InvalidCredentials
                | AtomekError::AuthExpired
                | AtomekError::AccountLocked
                | AtomekError::ApiStatus { status: 401, .. }
        )
    }

    /// Is this a plan/limit error that requires an upgrade?
    pub fn is_plan_error(&self) -> bool {
        matches!(
            self,
            AtomekError::NoSubscription
                | AtomekError::NoSentinelPass
                | AtomekError::PodLimitReached { .. }
        )
    }
}

impl From<reqwest::Error> for AtomekError {
    fn from(e: reqwest::Error) -> Self {
        AtomekError::Network(e.to_string())
    }
}

impl From<serde_json::Error> for AtomekError {
    fn from(e: serde_json::Error) -> Self {
        AtomekError::Other(format!("JSON error: {}", e))
    }
}

pub type Result<T> = std::result::Result<T, AtomekError>;

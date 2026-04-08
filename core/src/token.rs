use chrono::Utc;

/// Token state with expiry tracking.
/// Inspired by claurst's OAuthTokens — 5-minute expiry buffer.
#[derive(Debug, Clone)]
pub struct TokenState {
    pub access_token: String,
    /// Unix timestamp in milliseconds when the access token expires.
    pub expires_at_ms: Option<i64>,
}

impl TokenState {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            expires_at_ms: None,
        }
    }

    pub fn with_expiry(access_token: String, expires_in_secs: u64) -> Self {
        let expires_at_ms = Utc::now().timestamp_millis() + (expires_in_secs as i64 * 1000);
        Self {
            access_token,
            expires_at_ms: Some(expires_at_ms),
        }
    }

    /// True if token is expired or will expire within 5 minutes.
    /// 5-minute buffer prevents using a token that expires mid-request.
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.expires_at_ms {
            let buffer_ms: i64 = 5 * 60 * 1000; // 5 minutes
            let now_ms = Utc::now().timestamp_millis();
            (now_ms + buffer_ms) >= exp
        } else {
            false // No expiry info = assume valid
        }
    }

    /// Seconds remaining until expiry, or None if no expiry set.
    pub fn remaining_secs(&self) -> Option<i64> {
        let exp = self.expires_at_ms?;
        let now = Utc::now().timestamp_millis();
        let diff = (exp - now) / 1000;
        if diff > 0 { Some(diff) } else { Some(0) }
    }
}

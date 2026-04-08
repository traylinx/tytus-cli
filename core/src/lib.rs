pub mod error;
pub mod http;
pub mod device;
pub mod token;

pub use error::{AtomekError, Result};
pub use http::HttpClient;
pub use device::device_fingerprint;
pub use token::TokenState;

/// Global session handoff for privilege escalation.
/// Set by connect() before tunnel activation, read by relaunch_with_privileges().
/// Contains (email, refresh_token, pod_id) so elevated process can auto-reconnect.
static SESSION_HANDOFF: std::sync::Mutex<Option<(String, String, String)>> = std::sync::Mutex::new(None);

pub fn set_session_handoff(email: String, refresh_token: String, pod_id: String) {
    if let Ok(mut guard) = SESSION_HANDOFF.lock() {
        *guard = Some((email, refresh_token, pod_id));
    }
}

pub fn take_session_handoff() -> Option<(String, String, String)> {
    SESSION_HANDOFF.lock().ok()?.take()
}

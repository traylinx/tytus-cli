use std::path::PathBuf;
use thiserror::Error;

const SERVICE_NAME: &str = "com.traylinx.atomek";
/// Fixed path accessible by both user and root processes on macOS.
/// std::env::temp_dir() differs per user (/var/folders/... vs /tmp).
const HANDOFF_PATH: &str = "/tmp/.atomek-session-handoff";

#[derive(Debug, Error)]
pub enum KeychainError {
    #[error("Keychain error: {0}")]
    Keychain(String),
    #[error("No stored credentials")]
    NotFound,
}

pub struct KeychainStore;

impl KeychainStore {
    /// Store refresh token in OS keychain
    pub fn store_refresh_token(email: &str, token: &str) -> Result<(), KeychainError> {
        let entry = keyring::Entry::new(SERVICE_NAME, email)
            .map_err(|e| KeychainError::Keychain(e.to_string()))?;
        entry
            .set_password(token)
            .map_err(|e| KeychainError::Keychain(e.to_string()))?;
        tracing::info!("Refresh token stored in keychain for {}", email);
        Ok(())
    }

    /// Retrieve refresh token from OS keychain
    pub fn get_refresh_token(email: &str) -> Result<String, KeychainError> {
        let entry = keyring::Entry::new(SERVICE_NAME, email)
            .map_err(|e| KeychainError::Keychain(e.to_string()))?;
        entry
            .get_password()
            .map_err(|_| KeychainError::NotFound)
    }

    /// Delete refresh token from OS keychain
    pub fn delete_refresh_token(email: &str) -> Result<(), KeychainError> {
        let entry = keyring::Entry::new(SERVICE_NAME, email)
            .map_err(|e| KeychainError::Keychain(e.to_string()))?;
        let _ = entry.delete_credential(); // Ignore error if not found
        tracing::info!("Refresh token removed from keychain for {}", email);
        Ok(())
    }

    /// Find any stored email (for auto-login)
    /// Returns the email if a refresh token exists
    pub fn find_stored_email() -> Option<String> {
        // keyring crate doesn't support enumeration
        // We store the last-used email in a separate entry
        let entry = keyring::Entry::new(SERVICE_NAME, "__last_email__").ok()?;
        entry.get_password().ok()
    }

    /// Store last-used email for auto-login detection
    pub fn store_last_email(email: &str) -> Result<(), KeychainError> {
        let entry = keyring::Entry::new(SERVICE_NAME, "__last_email__")
            .map_err(|e| KeychainError::Keychain(e.to_string()))?;
        entry
            .set_password(email)
            .map_err(|e| KeychainError::Keychain(e.to_string()))?;
        Ok(())
    }

    /// Write session handoff file so an elevated process can auto-login.
    /// File is written with restrictive permissions and deleted after read.
    pub fn write_session_handoff(email: &str, refresh_token: &str) -> Result<PathBuf, KeychainError> {
        let path = PathBuf::from(HANDOFF_PATH);
        let content = format!("{}\n{}", email, refresh_token);
        std::fs::write(&path, &content)
            .map_err(|e| KeychainError::Keychain(format!("Failed to write handoff: {}", e)))?;
        // Restrict permissions (owner-only read/write)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        }
        tracing::info!("Session handoff written to {:?}", path);
        Ok(path)
    }

    /// Read and delete session handoff file. Returns (email, refresh_token).
    pub fn read_session_handoff() -> Option<(String, String)> {
        let path = PathBuf::from(HANDOFF_PATH);
        let content = std::fs::read_to_string(&path).ok()?;
        let _ = std::fs::remove_file(&path); // Always delete after read
        let mut lines = content.lines();
        let email = lines.next()?.to_string();
        let token = lines.next()?.to_string();
        if email.is_empty() || token.is_empty() {
            return None;
        }
        tracing::info!("Session handoff read for {}", email);
        Some((email, token))
    }
}

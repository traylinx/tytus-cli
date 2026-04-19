use thiserror::Error;

const SERVICE_NAME: &str = "com.traylinx.atomek";

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

    /// Retrieve refresh token from OS keychain.
    ///
    /// Wraps the keyring call in a thread + 3-second deadline. macOS
    /// re-prompts for keychain ACL approval every time the binary's code
    /// signature changes (dev rebuilds, unsigned installs, Apple code-
    /// sign rotation). The GUI dialog is invisible to non-interactive
    /// / non-TTY callers — status, doctor, forwarder, daemon, anything
    /// spawned detached from the tray — so the call would block forever
    /// waiting for a button nobody can see. After 3s we give up and
    /// return NotFound; callers fall back to "no refresh token, treat
    /// as logged out", which is honest + recoverable via `tytus login`.
    pub fn get_refresh_token(email: &str) -> Result<String, KeychainError> {
        let email = email.to_string();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result: Result<String, KeychainError> = (|| {
                let entry = keyring::Entry::new(SERVICE_NAME, &email)
                    .map_err(|e| KeychainError::Keychain(e.to_string()))?;
                entry.get_password().map_err(|_| KeychainError::NotFound)
            })();
            let _ = tx.send(result);
        });
        match rx.recv_timeout(std::time::Duration::from_secs(3)) {
            Ok(res) => res,
            Err(_) => {
                tracing::warn!(
                    "keychain get_refresh_token timed out after 3s — likely a user-approval dialog is pending. \
                     Falling back to NotFound. Re-run `tytus login` to refresh after approving the dialog."
                );
                Err(KeychainError::NotFound)
            }
        }
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

}

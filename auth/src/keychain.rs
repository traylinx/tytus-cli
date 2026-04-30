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
        tracing::info!(
            "Refresh token stored in keychain for {}",
            atomek_core::redact_email(email)
        );
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

    /// Retrieve refresh token if present, without emitting user-facing scary logs on miss.
    pub fn try_get_refresh_token(email: &str) -> Option<String> {
        Self::get_refresh_token(email).ok()
    }

    /// Delete refresh token from OS keychain
    pub fn delete_refresh_token(email: &str) -> Result<(), KeychainError> {
        let entry = keyring::Entry::new(SERVICE_NAME, email)
            .map_err(|e| KeychainError::Keychain(e.to_string()))?;
        let _ = entry.delete_credential(); // Ignore error if not found
        tracing::info!(
            "Refresh token removed from keychain for {}",
            atomek_core::redact_email(email)
        );
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

    pub fn list_indexed_accounts() -> Vec<String> {
        let entry = match keyring::Entry::new(SERVICE_NAME, "__account_index__") {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };
        let raw = match entry.get_password() {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let mut emails: Vec<String> = serde_json::from_str(&raw).unwrap_or_default();
        emails.retain(|e| !e.trim().is_empty());
        emails.sort();
        emails.dedup();
        emails
    }

    pub fn add_indexed_account(email: &str) -> Result<(), KeychainError> {
        let canonical = email.trim().to_ascii_lowercase();
        if canonical.is_empty() {
            return Ok(());
        }
        let mut emails = Self::list_indexed_accounts();
        if !emails.iter().any(|e| e == &canonical) {
            emails.push(canonical);
            emails.sort();
        }
        let entry = keyring::Entry::new(SERVICE_NAME, "__account_index__")
            .map_err(|e| KeychainError::Keychain(e.to_string()))?;
        entry
            .set_password(&serde_json::to_string(&emails).unwrap_or_else(|_| "[]".into()))
            .map_err(|e| KeychainError::Keychain(e.to_string()))?;
        Ok(())
    }

    pub fn remove_indexed_account(email: &str) -> Result<(), KeychainError> {
        let canonical = email.trim().to_ascii_lowercase();
        let mut emails = Self::list_indexed_accounts();
        emails.retain(|e| e != &canonical);
        let entry = keyring::Entry::new(SERVICE_NAME, "__account_index__")
            .map_err(|e| KeychainError::Keychain(e.to_string()))?;
        entry
            .set_password(&serde_json::to_string(&emails).unwrap_or_else(|_| "[]".into()))
            .map_err(|e| KeychainError::Keychain(e.to_string()))?;
        Ok(())
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

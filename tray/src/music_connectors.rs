//! JULI3TA music connector credential/status layer.
//!
//! Secrets live in the OS keychain. The JSON manifest only stores redacted
//! status/account metadata because keychains cannot be enumerated safely.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const KEYCHAIN_SERVICE: &str = "com.traylinx.atomek";
const CONNECTOR_PREFIX: &str = "music-connectors";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorCredentialSpec {
    pub name: String,
    pub label: String,
    pub secret: bool,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorDefinition {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub state: String,
    pub message: String,
    pub credential_specs: Vec<ConnectorCredentialSpec>,
    pub oauth_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorRecord {
    pub provider: String,
    pub account: Option<String>,
    #[serde(default)]
    pub credential_names: Vec<String>,
    pub verified_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorStatus {
    pub provider: String,
    pub name: String,
    pub connected: bool,
    pub configurable: bool,
    pub oauth_required: bool,
    pub account: Option<String>,
    pub credential_specs: Vec<ConnectorCredentialSpec>,
    pub verified_at: Option<String>,
    pub last_error: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ConnectorManifest {
    #[serde(default)]
    connectors: BTreeMap<String, ConnectorRecord>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigureConnectorRequest {
    pub provider: String,
    #[serde(default)]
    pub credentials: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorProviderRequest {
    pub provider: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectorError {
    #[error("unsupported provider")]
    UnsupportedProvider,
    #[error("{0}")]
    Validation(String),
    #[error("keychain error: {0}")]
    Keychain(String),
    #[error("manifest I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("manifest JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("verification failed: {0}")]
    Verification(String),
}

impl ConnectorManifest {
    fn path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".tytus")
            .join("music_connectors.json")
    }

    fn load() -> Self {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    fn save(&self) -> Result<(), ConnectorError> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, serde_json::to_string_pretty(self)?)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }
}

fn now_isoish() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Machine-readable enough for this localhost status API; avoids adding chrono.
    format!("unix:{secs}")
}

pub fn definitions() -> Vec<ConnectorDefinition> {
    vec![
        ConnectorDefinition {
            id: "spotify".to_string(),
            name: "Spotify".to_string(),
            kind: "metadata".to_string(),
            state: "oauth_required".to_string(),
            message: "Spotify account linking requires OAuth PKCE; no token-paste flow is exposed.".to_string(),
            oauth_required: true,
            credential_specs: vec![],
        },
        ConnectorDefinition {
            id: "lastfm".to_string(),
            name: "Last.fm".to_string(),
            kind: "metadata".to_string(),
            state: "needs_credentials".to_string(),
            message: "Artist bios, tags and listener metadata. API key is stored in the OS keychain.".to_string(),
            oauth_required: false,
            credential_specs: vec![
                ConnectorCredentialSpec { name: "apiKey".to_string(), label: "API key".to_string(), secret: true, required: true },
                ConnectorCredentialSpec { name: "username".to_string(), label: "Username (optional, verifies your profile)".to_string(), secret: false, required: false },
            ],
        },
        ConnectorDefinition {
            id: "discogs".to_string(),
            name: "Discogs".to_string(),
            kind: "catalog".to_string(),
            state: "needs_credentials".to_string(),
            message: "Release/catalog metadata and album artwork. Personal access token is stored in the OS keychain.".to_string(),
            oauth_required: false,
            credential_specs: vec![
                ConnectorCredentialSpec { name: "token".to_string(), label: "Personal access token".to_string(), secret: true, required: true },
            ],
        },
    ]
}

pub fn definition(provider: &str) -> Option<ConnectorDefinition> {
    definitions().into_iter().find(|d| d.id == provider)
}

pub fn validate_provider(provider: &str) -> Result<String, ConnectorError> {
    let trimmed = provider.trim().to_ascii_lowercase();
    if definition(&trimmed).is_some() {
        Ok(trimmed)
    } else {
        Err(ConnectorError::UnsupportedProvider)
    }
}

fn keychain_account(provider: &str, name: &str) -> String {
    format!("{CONNECTOR_PREFIX}::{provider}::{name}")
}

fn store_secret(provider: &str, name: &str, value: &str) -> Result<(), ConnectorError> {
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, &keychain_account(provider, name))
        .map_err(|e| ConnectorError::Keychain(e.to_string()))?;
    entry
        .set_password(value)
        .map_err(|e| ConnectorError::Keychain(e.to_string()))
}

fn get_secret(provider: &str, name: &str) -> Result<String, ConnectorError> {
    if let Some(env_name) = env_var_name(provider, name) {
        if let Ok(value) = std::env::var(env_name) {
            if !value.trim().is_empty() {
                return Ok(value);
            }
        }
    }
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, &keychain_account(provider, name))
        .map_err(|e| ConnectorError::Keychain(e.to_string()))?;
    entry
        .get_password()
        .map_err(|e| ConnectorError::Keychain(e.to_string()))
}

fn delete_secret(provider: &str, name: &str) -> Result<(), ConnectorError> {
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, &keychain_account(provider, name))
        .map_err(|e| ConnectorError::Keychain(e.to_string()))?;
    let _ = entry.delete_credential();
    Ok(())
}

fn env_var_name(provider: &str, name: &str) -> Option<&'static str> {
    match (provider, name) {
        ("lastfm", "apiKey") => Some("LASTFM_API_KEY"),
        ("lastfm", "username") => Some("LASTFM_USERNAME"),
        ("discogs", "token") => Some("DISCOGS_TOKEN"),
        _ => None,
    }
}

fn env_record(provider: &str) -> Option<ConnectorRecord> {
    let def = definition(provider)?;
    if def.oauth_required {
        return None;
    }
    let mut credential_names = Vec::new();
    let mut missing_required = false;
    for spec in &def.credential_specs {
        if let Some(env) = env_var_name(provider, &spec.name) {
            let present = std::env::var(env)
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false);
            if present {
                credential_names.push(spec.name.clone());
            } else if spec.required {
                missing_required = true;
            }
        } else if spec.required {
            missing_required = true;
        }
    }
    if missing_required {
        None
    } else {
        Some(ConnectorRecord {
            provider: provider.to_string(),
            account: std::env::var(env_var_name(provider, "username").unwrap_or("")).ok(),
            credential_names,
            verified_at: None,
            last_error: Some("Configured from environment; not live-verified yet.".to_string()),
        })
    }
}

pub fn statuses() -> Vec<ConnectorStatus> {
    let manifest = ConnectorManifest::load();
    definitions()
        .into_iter()
        .map(|def| {
            let record = manifest
                .connectors
                .get(&def.id)
                .cloned()
                .or_else(|| env_record(&def.id));
            let connected = record
                .as_ref()
                .map(|r| r.verified_at.is_some() && r.last_error.is_none())
                .unwrap_or(false);
            ConnectorStatus {
                provider: def.id.clone(),
                name: def.name,
                connected,
                configurable: !def.oauth_required,
                oauth_required: def.oauth_required,
                account: record.as_ref().and_then(|r| r.account.clone()),
                credential_specs: def.credential_specs,
                verified_at: record.as_ref().and_then(|r| r.verified_at.clone()),
                last_error: record.as_ref().and_then(|r| r.last_error.clone()),
                message: if def.oauth_required {
                    def.message
                } else if connected {
                    "Connected and verified. Secrets are stored outside the browser.".to_string()
                } else if record.is_some() {
                    "Credentials saved; verification needs attention.".to_string()
                } else {
                    def.message
                },
            }
        })
        .collect()
}

pub fn status_for(provider: &str) -> Option<ConnectorStatus> {
    statuses().into_iter().find(|s| s.provider == provider)
}

fn clean_credential_value(
    name: &str,
    value: Option<&String>,
    required: bool,
) -> Result<Option<String>, ConnectorError> {
    let Some(raw) = value else {
        return if required {
            Err(ConnectorError::Validation(format!("{name} is required")))
        } else {
            Ok(None)
        };
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return if required {
            Err(ConnectorError::Validation(format!("{name} is required")))
        } else {
            Ok(None)
        };
    }
    if trimmed.chars().count() > 512 {
        return Err(ConnectorError::Validation(format!("{name} is too long")));
    }
    Ok(Some(trimmed.to_string()))
}

fn normalize_credentials(
    def: &ConnectorDefinition,
    credentials: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, ConnectorError> {
    if def.oauth_required {
        return Err(ConnectorError::Validation(
            "provider requires OAuth PKCE and cannot be configured with pasted credentials"
                .to_string(),
        ));
    }
    let allowed = def
        .credential_specs
        .iter()
        .map(|s| s.name.as_str())
        .collect::<Vec<_>>();
    for key in credentials.keys() {
        if !allowed
            .iter()
            .any(|allowed_key| allowed_key == &key.as_str())
        {
            return Err(ConnectorError::Validation(format!(
                "unknown credential: {key}"
            )));
        }
    }
    let mut out = BTreeMap::new();
    for spec in &def.credential_specs {
        if let Some(value) =
            clean_credential_value(&spec.name, credentials.get(&spec.name), spec.required)?
        {
            out.insert(spec.name.clone(), value);
        }
    }
    Ok(out)
}

fn verify_lastfm(credentials: &BTreeMap<String, String>) -> Result<Option<String>, ConnectorError> {
    let api_key = credentials
        .get("apiKey")
        .ok_or_else(|| ConnectorError::Validation("apiKey is required".to_string()))?;
    let mut url = reqwest::Url::parse("https://ws.audioscrobbler.com/2.0/")
        .map_err(|e| ConnectorError::Verification(e.to_string()))?;
    {
        let mut qp = url.query_pairs_mut();
        qp.append_pair(
            "method",
            if credentials
                .get("username")
                .filter(|s| !s.trim().is_empty())
                .is_some()
            {
                "user.getInfo"
            } else {
                "chart.getTopArtists"
            },
        );
        qp.append_pair("api_key", api_key);
        qp.append_pair("format", "json");
        if let Some(username) = credentials.get("username").filter(|s| !s.trim().is_empty()) {
            qp.append_pair("user", username);
        }
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("TytusOS-JULI3TA/1.0")
        .build()
        .map_err(|e| ConnectorError::Verification(e.to_string()))?;
    let res = client
        .get(url)
        .send()
        .map_err(|e| ConnectorError::Verification(e.to_string()))?;
    if !res.status().is_success() {
        return Err(ConnectorError::Verification(format!(
            "Last.fm returned {}",
            res.status()
        )));
    }
    let body: serde_json::Value = res
        .json()
        .map_err(|e| ConnectorError::Verification(e.to_string()))?;
    if let Some(err) = body.get("error") {
        let msg = body
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Last.fm rejected credentials");
        return Err(ConnectorError::Verification(format!(
            "Last.fm error {err}: {msg}"
        )));
    }
    Ok(credentials.get("username").cloned())
}

fn verify_discogs(
    credentials: &BTreeMap<String, String>,
) -> Result<Option<String>, ConnectorError> {
    let token = credentials
        .get("token")
        .ok_or_else(|| ConnectorError::Validation("token is required".to_string()))?;
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("TytusOS-JULI3TA/1.0 +https://github.com/traylinx/tytus-cli")
        .build()
        .map_err(|e| ConnectorError::Verification(e.to_string()))?;
    let res = client
        .get("https://api.discogs.com/oauth/identity")
        .header("Authorization", format!("Discogs token={token}"))
        .send()
        .map_err(|e| ConnectorError::Verification(e.to_string()))?;
    if !res.status().is_success() {
        return Err(ConnectorError::Verification(format!(
            "Discogs returned {}",
            res.status()
        )));
    }
    let body: serde_json::Value = res
        .json()
        .map_err(|e| ConnectorError::Verification(e.to_string()))?;
    Ok(body
        .get("username")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string()))
}

fn verify_credentials(
    provider: &str,
    credentials: &BTreeMap<String, String>,
) -> Result<Option<String>, ConnectorError> {
    match provider {
        "lastfm" => verify_lastfm(credentials),
        "discogs" => verify_discogs(credentials),
        _ => Err(ConnectorError::UnsupportedProvider),
    }
}

pub fn configure(req: ConfigureConnectorRequest) -> Result<ConnectorStatus, ConnectorError> {
    let provider = validate_provider(&req.provider)?;
    let def = definition(&provider).ok_or(ConnectorError::UnsupportedProvider)?;
    let credentials = normalize_credentials(&def, &req.credentials)?;
    let account = verify_credentials(&provider, &credentials)?;

    for (name, value) in &credentials {
        store_secret(&provider, name, value)?;
    }

    let mut manifest = ConnectorManifest::load();
    manifest.connectors.insert(
        provider.clone(),
        ConnectorRecord {
            provider: provider.clone(),
            account,
            credential_names: credentials.keys().cloned().collect(),
            verified_at: Some(now_isoish()),
            last_error: None,
        },
    );
    manifest.save()?;
    status_for(&provider).ok_or(ConnectorError::UnsupportedProvider)
}

pub fn verify(provider_raw: &str) -> Result<ConnectorStatus, ConnectorError> {
    let provider = validate_provider(provider_raw)?;
    let def = definition(&provider).ok_or(ConnectorError::UnsupportedProvider)?;
    if def.oauth_required {
        return Err(ConnectorError::Validation(
            "provider requires OAuth PKCE".to_string(),
        ));
    }
    let mut credentials = BTreeMap::new();
    for spec in &def.credential_specs {
        if spec.required || status_for(&provider).and_then(|s| s.account).is_some() {
            match get_secret(&provider, &spec.name) {
                Ok(value) if !value.trim().is_empty() => {
                    credentials.insert(spec.name.clone(), value);
                }
                Ok(_) | Err(_) if spec.required => {
                    return Err(ConnectorError::Validation(format!(
                        "{} is not stored",
                        spec.name
                    )));
                }
                _ => {}
            }
        }
    }
    let account = verify_credentials(&provider, &credentials)?;
    let mut manifest = ConnectorManifest::load();
    manifest.connectors.insert(
        provider.clone(),
        ConnectorRecord {
            provider: provider.clone(),
            account,
            credential_names: credentials.keys().cloned().collect(),
            verified_at: Some(now_isoish()),
            last_error: None,
        },
    );
    manifest.save()?;
    status_for(&provider).ok_or(ConnectorError::UnsupportedProvider)
}

pub fn disconnect(provider_raw: &str) -> Result<ConnectorStatus, ConnectorError> {
    let provider = validate_provider(provider_raw)?;
    if let Some(def) = definition(&provider) {
        for spec in &def.credential_specs {
            let _ = delete_secret(&provider, &spec.name);
        }
    }
    let mut manifest = ConnectorManifest::load();
    manifest.connectors.remove(&provider);
    manifest.save()?;
    status_for(&provider).ok_or(ConnectorError::UnsupportedProvider)
}

pub fn connector_error_status(err: &ConnectorError) -> u16 {
    match err {
        ConnectorError::UnsupportedProvider => 404,
        ConnectorError::Validation(_) => 400,
        ConnectorError::Verification(_) => 502,
        ConnectorError::Keychain(_) | ConnectorError::Io(_) | ConnectorError::Json(_) => 500,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_known_providers_only() {
        assert_eq!(validate_provider(" LastFM ").unwrap(), "lastfm");
        assert!(matches!(
            validate_provider("soundcloud"),
            Err(ConnectorError::UnsupportedProvider)
        ));
    }

    #[test]
    fn rejects_unknown_or_missing_credentials() {
        let def = definition("discogs").unwrap();
        let mut bad = BTreeMap::new();
        bad.insert("other".to_string(), "x".to_string());
        assert!(matches!(
            normalize_credentials(&def, &bad),
            Err(ConnectorError::Validation(_))
        ));
        assert!(matches!(
            normalize_credentials(&def, &BTreeMap::new()),
            Err(ConnectorError::Validation(_))
        ));
    }

    #[test]
    fn spotify_is_oauth_only() {
        let def = definition("spotify").unwrap();
        assert!(def.oauth_required);
        assert!(matches!(
            normalize_credentials(&def, &BTreeMap::new()),
            Err(ConnectorError::Validation(_))
        ));
    }
}

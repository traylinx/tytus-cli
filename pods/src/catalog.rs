//! Agent catalog client with local 5-minute TTL cache.
//!
//! The catalog lists installable agent types (NemoClaw, Hermes, …) so the
//! CLI's `tytus agent catalog`, `tytus agent install`, and the tray's
//! browser wizard can render a data-driven list instead of hardcoding.
//!
//! Endpoint is public (no A2A auth). Provider serves it with a 5-minute
//! `Cache-Control` — we mirror that on the client side so
//! `tytus agent catalog` on a flaky link still works, and so a blast of
//! rapid calls (e.g. the tray refreshing on every 5s poll) doesn't
//! hammer Provider.
//!
//! Spec: SPRINT-AIL-DEFAULT-POD-AND-AGENT-INSTALL.md §6 C4.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use atomek_core::{AtomekError, HttpClient};
use serde::{Deserialize, Serialize};

const CATALOG_URL: &str = "https://tytus.traylinx.com/catalog/agents";
const CATALOG_TTL_SECS: u64 = 300;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCatalog {
    pub version: String,
    pub agents: Vec<AgentCatalogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCatalogEntry {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub tagline: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    pub units: u32,
    #[serde(default)]
    pub api_port: Option<u16>,
    #[serde(default)]
    pub health_port: Option<u16>,
    #[serde(default)]
    pub health_path: Option<String>,
    #[serde(default)]
    pub docs_url: Option<String>,
    #[serde(default)]
    pub min_plan: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedCatalog {
    fetched_at: u64,
    catalog: AgentCatalog,
}

fn cache_path() -> PathBuf {
    // Same "config_dir → tytus/" convention as state.json, but a
    // separate file so corrupting the cache can never wedge login.
    let config = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = config.join("tytus");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("catalog.json")
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn read_cache() -> Option<CachedCatalog> {
    let raw = std::fs::read_to_string(cache_path()).ok()?;
    serde_json::from_str(&raw).ok()
}

fn write_cache(catalog: &AgentCatalog) {
    let entry = CachedCatalog {
        fetched_at: now_secs(),
        catalog: catalog.clone(),
    };
    if let Ok(raw) = serde_json::to_string_pretty(&entry) {
        let _ = std::fs::write(cache_path(), raw);
    }
}

/// Fetch the catalog, using the local cache when it's fresh.
///
/// `refresh=true` bypasses the cache (used by `tytus agent catalog --refresh`
/// and the browser wizard's "refresh" button). On network failure we fall
/// back to a stale cache if one exists, so a user whose Wi-Fi dropped can
/// still see agent options — the alternative is a hard error for a purely
/// read-only operation, which is worse UX.
pub async fn fetch_catalog(http: &HttpClient, refresh: bool) -> atomek_core::Result<AgentCatalog> {
    if !refresh {
        if let Some(cached) = read_cache() {
            if now_secs().saturating_sub(cached.fetched_at) < CATALOG_TTL_SECS {
                return Ok(cached.catalog);
            }
        }
    }

    let fetch_result = http
        .get(CATALOG_URL)
        .header("Accept", "application/json")
        .send()
        .await;

    match fetch_result {
        Ok(resp) if resp.status().is_success() => {
            let catalog: AgentCatalog = resp
                .json()
                .await
                .map_err(|e| AtomekError::Other(format!("Failed to parse catalog: {}", e)))?;
            write_cache(&catalog);
            Ok(catalog)
        }
        Ok(resp) => {
            let status = resp.status().as_u16();
            // Fall back to stale cache rather than failing — catalog is
            // read-only and low-risk; the alternative is a blocked user.
            if let Some(cached) = read_cache() {
                return Ok(cached.catalog);
            }
            Err(AtomekError::ApiStatus {
                status,
                message: "catalog fetch failed".to_string(),
            })
        }
        Err(e) => {
            if let Some(cached) = read_cache() {
                return Ok(cached.catalog);
            }
            Err(AtomekError::Network(e.to_string()))
        }
    }
}

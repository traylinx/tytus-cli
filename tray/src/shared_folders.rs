//! Tray integration for the garagetytus shared-folder UX.
//!
//! Wraps the bash helpers shipped in
//! `github.com/traylinx/garagetytus/bin/`:
//!   - garagetytus-folder-bind / -list / -status / -unbind / -conflicts
//!   - garagetytus-pod-refresh
//!   - garagetytus-refresh-watchdog
//!
//! Two integration surfaces:
//!   1. Per-pod entries appended to the existing Files submenu:
//!        - "Bind Mac folder to share with this pod…"
//!        - "Refresh shared-folder credentials"
//!   2. Top-level "Shared Folders" submenu (between Pods and Settings):
//!        - List all bindings…
//!        - Status (with pod check)…
//!        - Find conflicts…
//!        - Open ~/.cache/garagetytus
//!        - Run cred refresh now
//!
//! All actions shell out via `std::process::Command` in detached
//! threads so the tray UI stays responsive. Failures surface as
//! macOS notifications.

use crate::files::notify_transfer;

// ── Per-pod menu IDs ─────────────────────────────────────────

pub fn menu_id_bind_folder(pod_id: &str) -> String {
    format!("pod_{}_files_bind_folder", pod_id)
}

pub fn menu_id_refresh_creds(pod_id: &str) -> String {
    format!("pod_{}_files_refresh_creds", pod_id)
}

// ── Global menu IDs ──────────────────────────────────────────

pub const ID_LIST_BINDINGS: &str = "shared_folders_list";
pub const ID_STATUS: &str = "shared_folders_status";
pub const ID_CONFLICTS: &str = "shared_folders_conflicts";
pub const ID_OPEN_CACHE: &str = "shared_folders_open_cache";
pub const ID_REFRESH_ALL: &str = "shared_folders_refresh_all";

// Prefix for dynamic per-binding "open in Finder" items at the top
// of the Shared Folders submenu. The full menu ID is
// `shared_folders_open_<safe-name>` where safe-name comes from the
// sidecar. Click handler opens the binding's local_path in Finder.
pub const ID_OPEN_BINDING_PREFIX: &str = "shared_folders_open_binding_";

// ── Per-pod-id parser (for the two new pod-scoped IDs) ────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedFoldersPodAction {
    BindFolder,
    RefreshCreds,
}

pub fn parse_pod_action(id: &str) -> Option<(String, SharedFoldersPodAction)> {
    let rest = id.strip_prefix("pod_")?;
    if let Some(p) = rest.strip_suffix("_files_bind_folder") {
        return Some((p.to_string(), SharedFoldersPodAction::BindFolder));
    }
    if let Some(p) = rest.strip_suffix("_files_refresh_creds") {
        return Some((p.to_string(), SharedFoldersPodAction::RefreshCreds));
    }
    None
}

// ── osascript helpers ────────────────────────────────────────

/// Prompt the user for a bucket name with an osascript dialog.
/// Returns None on cancel or empty input. macOS-only.
#[cfg(target_os = "macos")]
pub fn prompt_bucket_name(default: Option<&str>) -> Option<String> {
    let default_clause = match default {
        Some(d) => format!(" default answer \"{}\"", d.replace('"', "\\\"")),
        None => " default answer \"\"".to_string(),
    };
    let script = format!(
        "set r to display dialog \"Bucket name for the shared folder?\\n\\n\
         Lowercase letters, digits, dot, hyphen. 3-63 chars. \
         Created on the droplet if it doesn't exist.\" \
         with title \"garagetytus folder bind\"{}\n\
         text returned of r",
        default_clause,
    );
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if name.is_empty() { None } else { Some(name) }
}

#[cfg(not(target_os = "macos"))]
pub fn prompt_bucket_name(_default: Option<&str>) -> Option<String> {
    None
}

// ── Helper resolution ────────────────────────────────────────

/// Locate the garagetytus helper script. Checks common install
/// paths in order:
///   1. /usr/local/bin/<name>           (homebrew x86, manual)
///   2. /opt/homebrew/bin/<name>        (homebrew arm64)
///   3. ~/garagetytus/bin/<name>        (dev checkout)
///   4. fallback: bare name (let PATH resolve)
fn helper_path(name: &str) -> String {
    let candidates = [
        format!("/usr/local/bin/{}", name),
        format!("/opt/homebrew/bin/{}", name),
        std::env::var("HOME")
            .map(|h| format!("{}/garagetytus/bin/{}", h, name))
            .unwrap_or_default(),
    ];
    for c in &candidates {
        if !c.is_empty() && std::path::Path::new(c).is_file() {
            return c.clone();
        }
    }
    name.to_string()
}

// ── Bind a Mac folder to a pod ───────────────────────────────

pub fn spawn_bind_folder(pod_id: &str, local_path: &str, bucket: &str) {
    let pod = pod_id.to_string();
    let local = local_path.to_string();
    let bucket = bucket.to_string();
    let script = helper_path("garagetytus-folder-bind");
    std::thread::spawn(move || {
        let output = std::process::Command::new(&script)
            .arg(&local)
            .arg(&bucket)
            .arg("--to")
            .arg(&pod)
            .arg("--auto-sync")
            .output();
        match output {
            Ok(out) if out.status.success() => {
                notify_transfer(
                    "garagetytus folder bind",
                    &format!("✓ {} ↔ {} (pod-{})", basename(&local), bucket, pod),
                    Some(std::path::Path::new(&local)),
                );
            }
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr).to_string();
                let trimmed: String = err.lines().last()
                    .unwrap_or("bind failed").chars().take(140).collect();
                notify_transfer("garagetytus folder bind FAILED", &trimmed, None);
            }
            Err(e) => {
                notify_transfer("garagetytus folder bind error", &e.to_string(), None);
            }
        }
    });
}

// ── Refresh creds for one pod ────────────────────────────────

pub fn spawn_refresh_creds(pod_id: &str) {
    let pod = pod_id.to_string();
    let script = helper_path("garagetytus-pod-refresh");
    std::thread::spawn(move || {
        let output = std::process::Command::new(&script).arg(&pod).output();
        match output {
            Ok(out) if out.status.success() => {
                notify_transfer(
                    "garagetytus refresh",
                    &format!("✓ pod-{} credentials rotated", pod),
                    None,
                );
            }
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr).to_string();
                let trimmed: String = err.lines().last()
                    .unwrap_or("refresh failed").chars().take(140).collect();
                notify_transfer("garagetytus refresh FAILED", &trimmed, None);
            }
            Err(e) => {
                notify_transfer("garagetytus refresh error", &e.to_string(), None);
            }
        }
    });
}

// ── Refresh-watchdog (one-shot scan across every pod) ────────

pub fn spawn_refresh_all() {
    let script = helper_path("garagetytus-refresh-watchdog");
    std::thread::spawn(move || {
        let output = std::process::Command::new(&script).output();
        match output {
            Ok(out) if out.status.success() => {
                let lines = String::from_utf8_lossy(&out.stderr).to_string();
                let summary: String = lines.lines()
                    .filter(|l| l.contains("watchdog done") || l.contains("rotated"))
                    .last().unwrap_or("done").chars().take(140).collect();
                notify_transfer("garagetytus refresh-all", &summary, None);
            }
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr).to_string();
                let trimmed: String = err.lines().last()
                    .unwrap_or("watchdog failed").chars().take(140).collect();
                notify_transfer("garagetytus refresh-all FAILED", &trimmed, None);
            }
            Err(e) => {
                notify_transfer("garagetytus refresh-all error", &e.to_string(), None);
            }
        }
    });
}

// ── Enumerate active bindings from sidecar JSONs ─────────────

/// Lightweight view of one bound folder. Read from the sidecar JSON
/// that `garagetytus folder bind` writes at
/// `~/.cache/garagetytus/bisync/<safe-name>.bindings.json`. Sidecar
/// shape is documented in
/// `github.com/traylinx/garagetytus/docs/MANUAL.md` §12.
#[derive(Debug, Clone)]
pub struct Binding {
    pub safe_name: String,
    pub bucket: String,
    pub local_path: String,
}

/// Walk `~/.cache/garagetytus/bisync/*.bindings.json` and return one
/// Binding per readable+parsable sidecar. Returns empty Vec when the
/// dir doesn't exist (no bindings yet) or jq isn't available — never
/// errors, so the menu always builds even on a fresh machine.
pub fn list_bindings() -> Vec<Binding> {
    let home = match std::env::var("HOME") { Ok(h) => h, Err(_) => return vec![] };
    let dir = format!("{}/.cache/garagetytus/bisync", home);
    let entries = match std::fs::read_dir(&dir) { Ok(e) => e, Err(_) => return vec![] };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let safe_name = match name.strip_suffix(".bindings.json") {
            Some(s) => s.to_string(),
            None => continue,
        };
        let raw = match std::fs::read_to_string(&path) { Ok(r) => r, Err(_) => continue };
        // Parse with serde_json (already a workspace dep). Tolerate
        // missing fields by falling back to empty strings — broken
        // sidecars shouldn't poison the menu.
        let json: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(j) => j, Err(_) => continue,
        };
        let bucket = json.get("bucket").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let local_path = json.get("local_path").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if bucket.is_empty() || local_path.is_empty() { continue; }
        out.push(Binding { safe_name, bucket, local_path });
    }
    // Sort for stable menu order (bucket name asc).
    out.sort_by(|a, b| a.bucket.cmp(&b.bucket));
    out
}

/// Build the menu item ID for "open this binding's local folder".
/// Round-trips with `parse_open_binding_id`.
pub fn menu_id_open_binding(safe_name: &str) -> String {
    format!("{}{}", ID_OPEN_BINDING_PREFIX, safe_name)
}

/// Inverse of `menu_id_open_binding`. Returns the binding's
/// safe_name; caller looks the local_path back up via list_bindings().
pub fn parse_open_binding_id(id: &str) -> Option<String> {
    id.strip_prefix(ID_OPEN_BINDING_PREFIX).map(String::from)
}

/// Open one binding's local folder in Finder. No-op if the path no
/// longer exists on disk (orphan sidecar).
pub fn open_binding_in_finder(safe_name: &str) {
    if let Some(b) = list_bindings().into_iter().find(|b| b.safe_name == safe_name) {
        if std::path::Path::new(&b.local_path).is_dir() {
            let _ = std::process::Command::new("open")
                .arg(&b.local_path)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
        }
    }
}

// ── Open ~/.cache/garagetytus in Finder ──────────────────────

pub fn open_cache_dir() {
    let path = std::env::var("HOME")
        .map(|h| format!("{}/.cache/garagetytus", h))
        .unwrap_or_else(|_| "/tmp".to_string());
    // Make sure it exists so Finder doesn't bounce
    let _ = std::fs::create_dir_all(&path);
    let _ = std::process::Command::new("open")
        .arg(&path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

// ── Helpers ──────────────────────────────────────────────────

fn basename(path: &str) -> &str {
    std::path::Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_id_bind_round_trips() {
        let id = menu_id_bind_folder("02");
        let parsed = parse_pod_action(&id).unwrap();
        assert_eq!(parsed.0, "02");
        assert_eq!(parsed.1, SharedFoldersPodAction::BindFolder);
    }

    #[test]
    fn menu_id_refresh_round_trips() {
        let id = menu_id_refresh_creds("04");
        let parsed = parse_pod_action(&id).unwrap();
        assert_eq!(parsed.0, "04");
        assert_eq!(parsed.1, SharedFoldersPodAction::RefreshCreds);
    }

    #[test]
    fn parse_returns_none_for_unrelated_id() {
        assert!(parse_pod_action("login").is_none());
        assert!(parse_pod_action("pod_02_files_push_file").is_none());
        assert!(parse_pod_action(ID_LIST_BINDINGS).is_none());
    }

    #[test]
    fn helper_path_falls_back_to_bare_name() {
        // No helper installed at /usr/local/bin/foo-nonexistent; we
        // expect the fallback bare name so PATH resolution can take
        // over at runtime.
        let p = helper_path("garagetytus-doesnotexist-xyz");
        assert_eq!(p, "garagetytus-doesnotexist-xyz");
    }

    #[test]
    fn open_binding_id_round_trips() {
        let id = menu_id_open_binding("work-Documents-work");
        let parsed = parse_open_binding_id(&id).unwrap();
        assert_eq!(parsed, "work-Documents-work");
        assert!(id.starts_with(ID_OPEN_BINDING_PREFIX));
    }

    #[test]
    fn parse_open_binding_id_rejects_unrelated() {
        assert!(parse_open_binding_id(ID_LIST_BINDINGS).is_none());
        assert!(parse_open_binding_id("pod_02_files_bind_folder").is_none());
        assert!(parse_open_binding_id("random").is_none());
    }
}

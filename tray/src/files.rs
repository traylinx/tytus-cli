// ============================================================
// files — tray-side file sharing surfaces for tytus push/pull
// ============================================================
// Phase 2 of SPRINT-tytus-shared-folders ships a menu-based
// flow because NSStatusItem doesn't accept NSDraggingDestination
// out of the box and the real "drop on the tray icon" handshake
// needs a subclassed status button with objc2 — deferred.
//
// What this module ships instead:
//   - osascript-driven file/folder picker ("Push file…")
//   - terminal drop-in for pod listing ("List inbox")
//   - local download dir opener ("Open download folder")
//   - notification helper with "Reveal in Finder" action
//
// All ops shell out to the `tytus` CLI binary so the CLI stays
// the single source of truth for how sharing behaves.
// ============================================================

use std::path::PathBuf;

/// Local staging dir for pulls — mirrors what the CLI uses when
/// the user pulls from the tray. `~/Downloads/tytus/pod-NN/`.
pub fn download_dir_for_pod(pod_id: &str) -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    home.join("Downloads").join("tytus").join(format!("pod-{}", pod_id))
}

/// Ensure the per-pod download dir exists. Best-effort; ignored
/// if filesystem refuses.
pub fn ensure_download_dir(pod_id: &str) -> PathBuf {
    let path = download_dir_for_pod(pod_id);
    let _ = std::fs::create_dir_all(&path);
    path
}

// ── osascript file/folder picker ─────────────────────────────

/// Ask the user to pick a file or folder via Finder. Returns the
/// POSIX path, or None if cancelled. macOS only — other OSes get
/// a no-op implementation that always returns None.
#[cfg(target_os = "macos")]
pub fn pick_path(kind: PickerKind) -> Option<String> {
    let verb = match kind {
        PickerKind::File => "choose file",
        PickerKind::Folder => "choose folder",
    };
    let script = format!(
        "POSIX path of ({} with prompt \"Pick a {} to push to your pod\")",
        verb,
        match kind {
            PickerKind::File => "file",
            PickerKind::Folder => "folder",
        },
    );
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()?;
    if !output.status.success() {
        // User cancelled → osascript exits non-zero. Treat as None.
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() { None } else { Some(path) }
}

#[cfg(not(target_os = "macos"))]
pub fn pick_path(_kind: PickerKind) -> Option<String> {
    // Non-macOS tray path: the tray crate is macOS-only today, but
    // keeping this compile-safe for future Linux builds.
    None
}

#[derive(Copy, Clone, Debug)]
pub enum PickerKind {
    File,
    Folder,
}

// ── Notification with Reveal in Finder ───────────────────────

/// Notify the user of a completed transfer. Adds a "Reveal in
/// Finder" action when a local path is given. macOS-native via
/// osascript — no UserNotifications API dependency.
#[cfg(target_os = "macos")]
pub fn notify_transfer(title: &str, body: &str, reveal: Option<&std::path::Path>) {
    let escaped_body = body.replace('"', "\\\"");
    let escaped_title = title.replace('"', "\\\"");
    let script = format!(
        "display notification \"{}\" with title \"{}\"",
        escaped_body, escaped_title,
    );
    let _ = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    if let Some(path) = reveal {
        // Brief pause so the notification banner appears before
        // Finder steals focus. Purely cosmetic.
        std::thread::sleep(std::time::Duration::from_millis(400));
        let _ = std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
}

#[cfg(not(target_os = "macos"))]
pub fn notify_transfer(_title: &str, _body: &str, _reveal: Option<&std::path::Path>) {}

// ── Push flow ──────────────────────────────────────────────

/// Kick off a push from the tray. Shells out to `tytus push` in
/// a detached thread so the tray UI stays responsive. Posts a
/// completion notification on success or failure.
pub fn spawn_push(pod_id: &str, local_path: &str) {
    let pod = pod_id.to_string();
    let local = local_path.to_string();
    std::thread::spawn(move || {
        let output = std::process::Command::new("tytus")
            .arg("push")
            .arg(&local)
            .arg("--pod")
            .arg(&pod)
            .arg("--json")
            .output();
        match output {
            Ok(out) if out.status.success() => {
                let basename = std::path::Path::new(&local)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&local);
                notify_transfer(
                    "Tytus push complete",
                    &format!("Pushed {} → pod-{}", basename, pod),
                    None,
                );
            }
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr).to_string();
                let trimmed: String = err.lines().last().unwrap_or("push failed").chars().take(120).collect();
                notify_transfer(
                    "Tytus push failed",
                    &trimmed,
                    None,
                );
            }
            Err(e) => {
                notify_transfer("Tytus push error", &e.to_string(), None);
            }
        }
    });
}

/// Kick off a pull from the tray. Pulls into
/// `~/Downloads/tytus/pod-NN/` and reveals in Finder on success.
/// Currently used indirectly via the CLI; kept pub for the future
/// SwiftUI list-view panel (Phase 2 follow-up).
#[allow(dead_code)]
pub fn spawn_pull(pod_id: &str, remote_path: &str) {
    let pod = pod_id.to_string();
    let remote = remote_path.to_string();
    let dest_dir = ensure_download_dir(&pod);
    std::thread::spawn(move || {
        let output = std::process::Command::new("tytus")
            .arg("pull")
            .arg(&remote)
            .arg("--pod")
            .arg(&pod)
            .arg("--to")
            .arg(&dest_dir)
            .arg("--json")
            .output();
        match output {
            Ok(out) if out.status.success() => {
                let basename = std::path::Path::new(&remote)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&remote);
                let local_path = dest_dir.join(basename);
                notify_transfer(
                    "Tytus pull complete",
                    &format!("Pulled {} from pod-{}", basename, pod),
                    Some(&local_path),
                );
            }
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr).to_string();
                let trimmed: String = err.lines().last().unwrap_or("pull failed").chars().take(120).collect();
                notify_transfer("Tytus pull failed", &trimmed, None);
            }
            Err(e) => {
                notify_transfer("Tytus pull error", &e.to_string(), None);
            }
        }
    });
}

// ── Local folder open ──────────────────────────────────────

pub fn open_download_dir(pod_id: &str) {
    let path = ensure_download_dir(pod_id);
    let _ = std::process::Command::new("open")
        .arg(&path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

// ── Menu-id helpers (keep ids in one place) ────────────────

pub fn menu_id_push_file(pod_id: &str) -> String {
    format!("pod_{}_files_push_file", pod_id)
}

pub fn menu_id_push_folder(pod_id: &str) -> String {
    format!("pod_{}_files_push_folder", pod_id)
}

pub fn menu_id_list_inbox(pod_id: &str) -> String {
    format!("pod_{}_files_list_inbox", pod_id)
}

pub fn menu_id_open_downloads(pod_id: &str) -> String {
    format!("pod_{}_files_open_downloads", pod_id)
}

pub fn parse_pod_from_files_id(id: &str) -> Option<(String, FilesAction)> {
    let rest = id.strip_prefix("pod_")?;
    if let Some(p) = rest.strip_suffix("_files_push_file") {
        return Some((p.to_string(), FilesAction::PushFile));
    }
    if let Some(p) = rest.strip_suffix("_files_push_folder") {
        return Some((p.to_string(), FilesAction::PushFolder));
    }
    if let Some(p) = rest.strip_suffix("_files_list_inbox") {
        return Some((p.to_string(), FilesAction::ListInbox));
    }
    if let Some(p) = rest.strip_suffix("_files_open_downloads") {
        return Some((p.to_string(), FilesAction::OpenDownloads));
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesAction {
    PushFile,
    PushFolder,
    ListInbox,
    OpenDownloads,
}

// ── Tests ──────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_dir_includes_pod_id() {
        let p = download_dir_for_pod("02");
        assert!(p.to_string_lossy().contains("tytus"));
        assert!(p.to_string_lossy().ends_with("pod-02"));
    }

    #[test]
    fn menu_id_roundtrip_push_file() {
        let id = menu_id_push_file("02");
        assert_eq!(
            parse_pod_from_files_id(&id),
            Some(("02".to_string(), FilesAction::PushFile))
        );
    }

    #[test]
    fn menu_id_roundtrip_push_folder() {
        let id = menu_id_push_folder("04");
        assert_eq!(
            parse_pod_from_files_id(&id),
            Some(("04".to_string(), FilesAction::PushFolder))
        );
    }

    #[test]
    fn menu_id_roundtrip_list_inbox() {
        let id = menu_id_list_inbox("02");
        assert_eq!(
            parse_pod_from_files_id(&id),
            Some(("02".to_string(), FilesAction::ListInbox))
        );
    }

    #[test]
    fn menu_id_roundtrip_open_downloads() {
        let id = menu_id_open_downloads("02");
        assert_eq!(
            parse_pod_from_files_id(&id),
            Some(("02".to_string(), FilesAction::OpenDownloads))
        );
    }

    #[test]
    fn parse_returns_none_for_unrelated_ids() {
        assert!(parse_pod_from_files_id("pod_02_channels_catalog").is_none());
        assert!(parse_pod_from_files_id("pod_02_restart").is_none());
        assert!(parse_pod_from_files_id("garbage").is_none());
    }

    #[test]
    fn parse_tolerates_multi_char_pod_ids() {
        let id = menu_id_push_file("abc");
        let parsed = parse_pod_from_files_id(&id).unwrap();
        assert_eq!(parsed.0, "abc");
    }
}

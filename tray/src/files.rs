// ============================================================
// files — tray-side file sharing surfaces for tytus push/pull
// ============================================================
// Phase 2 of SPRINT-tytus-shared-folders ships a menu-based
// flow because NSStatusItem doesn't accept NSDraggingDestination
// out of the box and the real "drop on the tray icon" handshake
// needs a subclassed status button with objc2 — deferred.
//
// What this module ships instead:
//   - platform file/folder picker ("Push file…")
//   - terminal drop-in for pod listing ("List inbox")
//   - local download dir opener ("Open download folder")
//   - notification helper with platform reveal action
//
// All ops shell out to the `tytus` CLI binary so the CLI stays
// the single source of truth for how sharing behaves.
// ============================================================

use std::path::PathBuf;

/// Local staging dir for pulls. New user-visible home is
/// `~/Tytus/Downloads/pod-NN/`. The old `~/Downloads/tytus/pod-NN/`
/// is left untouched for compatibility.
#[allow(dead_code)]
pub fn download_dir_for_pod(pod_id: &str) -> PathBuf {
    crate::workspace::download_dir_for_pod(pod_id)
}

/// Ensure the per-pod download dir exists. Best-effort; ignored
/// if filesystem refuses.
pub fn ensure_download_dir(pod_id: &str) -> PathBuf {
    crate::workspace::ensure_download_dir_for_pod(pod_id)
}

// ── Native file/folder picker ────────────────────────────────

/// Ask the user to pick a file or folder via the platform picker. Returns the
/// POSIX path, or None if cancelled/unsupported.
pub fn pick_path(kind: PickerKind) -> Option<String> {
    let platform_kind = match kind {
        PickerKind::File => atomek_core::platform::dialog::PickKind::File,
        PickerKind::Folder => atomek_core::platform::dialog::PickKind::Folder,
    };
    let label = match kind {
        PickerKind::File => "file",
        PickerKind::Folder => "folder",
    };
    atomek_core::platform::dialog::pick_path(
        platform_kind,
        &format!("Pick a {} to push to your pod", label),
    )
    .ok()
    .flatten()
}

#[derive(Copy, Clone, Debug)]
pub enum PickerKind {
    File,
    Folder,
}

// ── Notification with platform reveal ───────────────────────

/// Notify the user of a completed transfer and reveal the local path in the
/// platform file manager when one is given.
#[cfg(target_os = "macos")]
pub fn notify_transfer(title: &str, body: &str, reveal: Option<&std::path::Path>) {
    let _ = atomek_core::platform::dialog::notify(title, body);
    if let Some(path) = reveal {
        // Brief pause so the notification banner appears before the file
        // manager steals focus. Purely cosmetic.
        std::thread::sleep(std::time::Duration::from_millis(400));
        let _ = atomek_core::platform::open::reveal_path(path);
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
                let trimmed: String = err
                    .lines()
                    .last()
                    .unwrap_or("push failed")
                    .chars()
                    .take(120)
                    .collect();
                notify_transfer("Tytus push failed", &trimmed, None);
            }
            Err(e) => {
                notify_transfer("Tytus push error", &e.to_string(), None);
            }
        }
    });
}

/// Kick off a pull from the tray. Pulls into
/// `~/Tytus/Downloads/pod-NN/` and reveals in Finder on success.
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
                let trimmed: String = err
                    .lines()
                    .last()
                    .unwrap_or("pull failed")
                    .chars()
                    .take(120)
                    .collect();
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
    open_path(&path);
}

fn open_path(path: &std::path::Path) {
    let _ = atomek_core::platform::open::open_path(path);
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
        assert!(p.to_string_lossy().contains("Tytus"));
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

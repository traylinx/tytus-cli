//! User-visible Tytus workspace paths.
//!
//! Hidden/internal state stays in Application Support, Keychain, ~/.tytus,
//! and Library/Logs. This module owns only human-visible files.

use std::path::PathBuf;

pub const WORKSPACE_DIR_NAME: &str = "Tytus";

pub fn tytus_home() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(WORKSPACE_DIR_NAME)
}

#[allow(dead_code)]
pub fn downloads_dir() -> PathBuf {
    tytus_home().join("Downloads")
}

#[allow(dead_code)]
pub fn download_dir_for_pod(pod_id: &str) -> PathBuf {
    downloads_dir().join(format!("pod-{}", pod_id))
}

pub fn legacy_download_dir_for_pod(pod_id: &str) -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("Downloads")
        .join("tytus")
        .join(format!("pod-{}", pod_id))
}

pub fn ensure_tytus_home() -> PathBuf {
    let root = tytus_home();
    let dirs = [
        root.clone(),
        root.join("Inbox"),
        root.join("Outbox"),
        root.join("Downloads"),
        root.join("Pods"),
        root.join("Shared"),
        root.join("Projects"),
        root.join("Logs"),
    ];
    for dir in dirs {
        let _ = std::fs::create_dir_all(dir);
    }

    let readme = root.join("README.md");
    if !readme.exists() {
        let _ = std::fs::write(
            &readme,
            "# Tytus Home\n\nThis is your visible Tytus workspace.\n\n- `Inbox/` — local staging area for files you want to push to pods.\n- `Outbox/` — local staging area for files coming back from pods.\n- `Downloads/pod-NN/` — files pulled from each pod.\n- `Pods/pod-NN/` — optional per-pod notes and local workspace mirrors.\n- `Shared/` — user-selected shared-folder bindings.\n- `Projects/` — local project folders you want to use with Tytus.\n- `Logs/` — user-visible logs and exports.\n\nInternal state and secrets are not stored here. Tytus keeps app state in `~/Library/Application Support/tytus/` on macOS and secrets in Keychain. Remote pod files live under `/app/workspace/`.\n",
        );
    }
    root
}

pub fn ensure_download_dir_for_pod(pod_id: &str) -> PathBuf {
    let root = ensure_tytus_home();
    let path = root.join("Downloads").join(format!("pod-{}", pod_id));
    let _ = std::fs::create_dir_all(&path);

    // Leave the old folder in place if it exists. Do not symlink/move it yet:
    // users may have Finder windows, scripts, or backups pointing there.
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_root_is_visible_tytus_folder() {
        let p = tytus_home();
        assert!(p.ends_with("Tytus"));
        assert!(!p.to_string_lossy().contains(".tytus"));
    }

    #[test]
    fn download_dir_includes_pod_id_under_tytus_home() {
        let p = download_dir_for_pod("02");
        assert!(p.to_string_lossy().contains("Tytus"));
        assert!(p.to_string_lossy().ends_with("Downloads/pod-02"));
    }

    #[test]
    fn legacy_download_dir_remains_old_downloads_location() {
        let p = legacy_download_dir_for_pod("02");
        assert!(p.to_string_lossy().contains("Downloads/tytus"));
        assert!(p.to_string_lossy().ends_with("pod-02"));
    }
}

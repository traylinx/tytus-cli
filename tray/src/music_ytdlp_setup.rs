//! yt-dlp install/status manager for the tray music endpoints.

use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::music_ytdlp;

#[derive(Clone, Debug)]
struct SetupState {
    installing: bool,
    source: String,
    version: Option<String>,
    error: Option<String>,
    last_checked: u64,
}

impl Default for SetupState {
    fn default() -> Self {
        Self {
            installing: false,
            source: "none".to_string(),
            version: None,
            error: Some("yt-dlp not checked yet".to_string()),
            last_checked: 0,
        }
    }
}

static STATE: OnceLock<Mutex<SetupState>> = OnceLock::new();

fn state() -> &'static Mutex<SetupState> {
    STATE.get_or_init(|| Mutex::new(SetupState::default()))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn status() -> music_ytdlp::MusicStatus {
    let ytdlp = music_ytdlp::configured_status();
    let setup = state().lock().map(|g| g.clone()).unwrap_or_default();
    music_ytdlp::MusicStatus {
        ready: ytdlp.ready,
        installing: setup.installing,
        source: if ytdlp.ready {
            ytdlp.source
        } else {
            setup.source
        },
        version: ytdlp.version.or(setup.version),
        error: if ytdlp.ready {
            None
        } else {
            setup.error.or(ytdlp.error)
        },
    }
}

pub fn start_background_install() {
    if let Ok(mut s) = state().lock() {
        if s.installing || now_secs().saturating_sub(s.last_checked) < 30 {
            return;
        }
        s.installing = true;
        s.error = None;
        s.last_checked = now_secs();
    }

    thread::Builder::new()
        .name("tytus-music-ytdlp-setup".to_string())
        .spawn(|| {
            let result = ensure_installed();
            if let Ok(mut s) = state().lock() {
                s.installing = false;
                s.last_checked = now_secs();
                match result {
                    Ok((path, source, version)) => {
                        music_ytdlp::set_binary_path(path);
                        s.source = source;
                        s.version = version;
                        s.error = None;
                    }
                    Err(e) => {
                        music_ytdlp::clear_binary_path();
                        s.source = "none".to_string();
                        s.version = None;
                        s.error = Some(e);
                    }
                }
            }
        })
        .ok();
}

fn ensure_installed() -> Result<(PathBuf, String, Option<String>), String> {
    if let Some(version) = system_ytdlp_version() {
        return Ok((PathBuf::from("yt-dlp"), "system".to_string(), Some(version)));
    }

    let path = ytdlp_binary_path();
    if path.exists() {
        if let Some(version) = music_ytdlp::binary_version(&path) {
            return Ok((path, "bundled".to_string(), Some(version)));
        }
        let _ = std::fs::remove_file(&path);
    }

    download_ytdlp(&path)?;
    ensure_permissions(&path)?;
    let version = music_ytdlp::binary_version(&path);
    Ok((path, "bundled".to_string(), version))
}

fn system_ytdlp_version() -> Option<String> {
    let out = Command::new("yt-dlp").arg("--version").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if version.is_empty() {
        None
    } else {
        Some(version)
    }
}

fn ytdlp_binary_path() -> PathBuf {
    let base = dirs::data_dir()
        .unwrap_or_else(|| std::env::temp_dir())
        .join("tytus")
        .join("ytdlp");
    let file = if cfg!(target_os = "windows") {
        "yt-dlp.exe"
    } else {
        "yt-dlp"
    };
    base.join(file)
}

fn release_asset_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "yt-dlp.exe"
    } else if cfg!(target_os = "macos") {
        "yt-dlp_macos"
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        "yt-dlp_linux_aarch64"
    } else if cfg!(target_os = "linux") {
        "yt-dlp_linux"
    } else {
        "yt-dlp"
    }
}

fn download_ytdlp(path: &PathBuf) -> Result<(), String> {
    let url = format!(
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/{}",
        release_asset_name()
    );
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("cannot create yt-dlp dir: {e}"))?;
    }
    let tmp = path.with_extension("download");
    let mut response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .user_agent("tytus-tray/0.6 music setup")
        .build()
        .map_err(|e| format!("cannot create downloader: {e}"))?
        .get(&url)
        .send()
        .map_err(|e| format!("yt-dlp download failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "yt-dlp download failed with HTTP {}",
            response.status()
        ));
    }
    let mut file =
        std::fs::File::create(&tmp).map_err(|e| format!("cannot create yt-dlp file: {e}"))?;
    std::io::copy(&mut response, &mut file)
        .map_err(|e| format!("cannot write yt-dlp file: {e}"))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("cannot install yt-dlp file: {e}"))?;
    Ok(())
}

fn ensure_permissions(path: &PathBuf) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)
            .map_err(|e| format!("cannot stat yt-dlp: {e}"))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms).map_err(|e| format!("cannot chmod yt-dlp: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_release_asset_for_platform() {
        let asset = release_asset_name();
        assert!(asset.starts_with("yt-dlp"));
    }
}

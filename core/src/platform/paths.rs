use std::fs;
use std::path::{Path, PathBuf};

pub const APP_ID: &str = "tytus";
pub const VISIBLE_HOME: &str = "Tytus";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathSet {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub visible_home: PathBuf,
}

pub fn path_set() -> PathSet {
    let data = data_dir();
    PathSet {
        config_dir: config_dir(),
        cache_dir: cache_dir(),
        runtime_dir: runtime_dir(),
        logs_dir: logs_dir(),
        visible_home: visible_home(),
        data_dir: data,
    }
}

pub fn config_dir() -> PathBuf {
    if let Some(p) = env_path("TYTUS_CONFIG_DIR") {
        return p;
    }
    if let Some(p) = dirs::config_dir() {
        return p.join(APP_ID);
    }
    fallback_home().join(format!(".{}", APP_ID))
}

pub fn data_dir() -> PathBuf {
    if let Some(p) = env_path("TYTUS_DATA_DIR") {
        return p;
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            return home.join("Library/Application Support").join(APP_ID);
        }
    }
    if let Some(p) = dirs::data_dir() {
        return p.join(APP_ID);
    }
    fallback_home().join(format!(".local/share/{}", APP_ID))
}

pub fn cache_dir() -> PathBuf {
    if let Some(p) = env_path("TYTUS_CACHE_DIR") {
        return p;
    }
    if let Some(p) = dirs::cache_dir() {
        return p.join(APP_ID);
    }
    fallback_home().join(format!(".cache/{}", APP_ID))
}

pub fn runtime_dir() -> PathBuf {
    if let Some(p) = env_path("TYTUS_RUNTIME_DIR") {
        return p;
    }
    #[cfg(unix)]
    {
        if let Some(p) = env_path("XDG_RUNTIME_DIR") {
            return p.join(APP_ID);
        }
        // Migration compatibility: existing macOS/Linux daemons, pid files,
        // and tray port files live here. Keep as runtime fallback until the
        // localhost-control-plane migration is complete.
        legacy_runtime_dir()
    }
    #[cfg(windows)]
    {
        if let Some(p) = dirs::data_local_dir() {
            return p.join("Tytus").join("Runtime");
        }
        return std::env::temp_dir().join(APP_ID);
    }
    #[cfg(not(any(unix, windows)))]
    std::env::temp_dir().join(APP_ID)
}

pub fn logs_dir() -> PathBuf {
    if let Some(p) = env_path("TYTUS_LOG_DIR") {
        return p;
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            return home.join("Library/Logs").join(APP_ID);
        }
    }
    data_dir().join("logs")
}

pub fn visible_home() -> PathBuf {
    if let Some(p) = env_path("TYTUS_HOME") {
        return p;
    }
    dirs::home_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join(VISIBLE_HOME)
}

pub fn control_file() -> PathBuf {
    runtime_dir().join("control.json")
}

pub fn control_token_file() -> PathBuf {
    runtime_dir().join("control.token")
}

pub fn daemon_pid_file() -> PathBuf {
    runtime_dir().join("daemon.pid")
}

pub fn tray_pid_file() -> PathBuf {
    runtime_dir().join("tray.pid")
}

pub fn tray_web_port_file() -> PathBuf {
    runtime_dir().join("tray-web.port")
}

pub fn legacy_runtime_dir() -> PathBuf {
    PathBuf::from("/tmp/tytus")
}

pub fn legacy_daemon_socket() -> PathBuf {
    legacy_runtime_dir().join("daemon.sock")
}

pub fn ensure_private_dir(path: &Path) -> std::io::Result<()> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name).and_then(|v| {
        if v.is_empty() {
            None
        } else {
            Some(PathBuf::from(v))
        }
    })
}

fn fallback_home() -> PathBuf {
    dirs::home_dir().unwrap_or_else(std::env::temp_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_set_uses_tytus_named_dirs() {
        let p = path_set();
        assert!(p
            .config_dir
            .to_string_lossy()
            .to_lowercase()
            .contains(APP_ID));
        assert!(p.data_dir.to_string_lossy().to_lowercase().contains(APP_ID));
        assert!(p
            .cache_dir
            .to_string_lossy()
            .to_lowercase()
            .contains(APP_ID));
        assert!(p
            .runtime_dir
            .to_string_lossy()
            .to_lowercase()
            .contains(APP_ID));
        assert!(p.logs_dir.to_string_lossy().to_lowercase().contains(APP_ID));
        assert!(p.visible_home.ends_with(VISIBLE_HOME));
    }

    #[test]
    fn control_files_live_under_runtime_dir() {
        assert!(control_file().ends_with("control.json"));
        assert!(control_token_file().ends_with("control.token"));
        assert!(daemon_pid_file().ends_with("daemon.pid"));
        assert!(tray_pid_file().ends_with("tray.pid"));
        assert!(tray_web_port_file().ends_with("tray-web.port"));
    }

    #[test]
    fn legacy_paths_are_explicit_migration_surface() {
        assert_eq!(legacy_runtime_dir(), PathBuf::from("/tmp/tytus"));
        assert!(legacy_daemon_socket().ends_with("daemon.sock"));
    }
}

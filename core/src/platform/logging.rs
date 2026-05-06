use crate::platform::paths;
use std::path::PathBuf;

pub fn daemon_log_file() -> PathBuf {
    paths::logs_dir().join("daemon.log")
}

pub fn tray_log_file() -> PathBuf {
    paths::logs_dir().join("tray.log")
}

pub fn autostart_log_file() -> PathBuf {
    paths::logs_dir().join("autostart.log")
}

pub fn support_bundle_dir() -> PathBuf {
    paths::visible_home().join("Logs").join("Support Bundles")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logs_have_stable_names() {
        assert!(daemon_log_file().ends_with("daemon.log"));
        assert!(tray_log_file().ends_with("tray.log"));
        assert!(autostart_log_file().ends_with("autostart.log"));
        assert!(support_bundle_dir()
            .to_string_lossy()
            .contains("Support Bundles"));
    }
}

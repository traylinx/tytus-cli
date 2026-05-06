use std::path::Path;
use std::process::{Command, Output};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceBackend {
    Launchd,
    SystemdUser,
    WindowsService,
    TaskScheduler,
    Unsupported,
}

pub fn default_service_backend() -> ServiceBackend {
    #[cfg(target_os = "macos")]
    {
        ServiceBackend::Launchd
    }
    #[cfg(target_os = "linux")]
    {
        ServiceBackend::SystemdUser
    }
    #[cfg(target_os = "windows")]
    {
        ServiceBackend::WindowsService
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        ServiceBackend::Unsupported
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceNames {
    pub connect: &'static str,
    pub daemon: &'static str,
    pub tray: &'static str,
}

pub fn service_names() -> ServiceNames {
    ServiceNames {
        connect: "com.traylinx.tytus",
        daemon: "com.traylinx.tytus.daemon",
        tray: "com.traylinx.tytus.tray",
    }
}

pub fn launchd_unload_agent(plist_path: &Path, disable: bool) -> bool {
    let mut cmd = Command::new("launchctl");
    cmd.arg("unload");
    if disable {
        cmd.arg("-w");
    }
    cmd.arg(plist_path);
    output_success(cmd.output())
}

pub fn launchd_load_agent(plist_path: &Path) -> bool {
    let mut cmd = Command::new("launchctl");
    output_success(cmd.args(["load", "-w"]).arg(plist_path).output())
}

pub fn launchd_agent_loaded(label: &str) -> bool {
    output_success(Command::new("launchctl").args(["list", label]).output())
}

pub fn systemd_user_daemon_reload() -> bool {
    output_success(
        Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .output(),
    )
}

pub fn systemd_user_enable_now(unit: &str) -> bool {
    output_success(
        Command::new("systemctl")
            .args(["--user", "enable", "--now", unit])
            .output(),
    )
}

pub fn systemd_user_disable_now(unit: &str) -> bool {
    output_success(
        Command::new("systemctl")
            .args(["--user", "disable", "--now", unit])
            .output(),
    )
}

pub fn systemd_user_enabled(unit: &str) -> bool {
    output_success(
        Command::new("systemctl")
            .args(["--user", "is-enabled", unit])
            .output(),
    )
}

pub fn macos_register_launch_services(app_path: &Path) -> bool {
    output_success(
        Command::new(launch_services_register_path())
            .args(["-f"])
            .arg(app_path)
            .output(),
    )
}

pub fn macos_unregister_launch_services(app_path: &Path) -> bool {
    output_success(
        Command::new(launch_services_register_path())
            .args(["-u"])
            .arg(app_path)
            .output(),
    )
}

pub fn macos_open_app(app_path: &Path) -> bool {
    Command::new("/usr/bin/open")
        .arg("-a")
        .arg(app_path)
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn launch_services_register_path() -> &'static str {
    "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"
}

fn output_success(output: std::io::Result<Output>) -> bool {
    output.map(|o| o.status.success()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_names_are_stable() {
        let names = service_names();
        assert!(names.connect.contains("tytus"));
        assert!(names.daemon.contains("tytus"));
        assert!(names.tray.contains("tytus"));
    }

    #[test]
    fn launch_services_path_is_absolute() {
        assert!(launch_services_register_path().starts_with('/'));
        assert!(launch_services_register_path().contains("lsregister"));
    }
}

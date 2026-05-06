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
    pub daemon: &'static str,
    pub tray: &'static str,
}

pub fn service_names() -> ServiceNames {
    ServiceNames {
        daemon: "com.traylinx.tytus.daemon",
        tray: "com.traylinx.tytus.tray",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_names_are_stable() {
        let names = service_names();
        assert!(names.daemon.contains("tytus"));
        assert!(names.tray.contains("tytus"));
    }
}

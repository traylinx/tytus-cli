use std::io;
#[cfg(windows)]
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    Alive,
    NotFound,
    PermissionDenied,
}

pub fn process_status(pid: u32) -> ProcessStatus {
    if pid <= 1 {
        return ProcessStatus::NotFound;
    }
    #[cfg(unix)]
    {
        let rc = unsafe { libc::kill(pid as libc::pid_t, 0) };
        if rc == 0 {
            return ProcessStatus::Alive;
        }
        match io::Error::last_os_error().raw_os_error() {
            Some(libc::EPERM) => ProcessStatus::PermissionDenied,
            _ => ProcessStatus::NotFound,
        }
    }
    #[cfg(windows)]
    {
        let filter = format!("PID eq {}", pid);
        let out = Command::new("tasklist")
            .args(["/FI", &filter, "/NH"])
            .output();
        return match out {
            Ok(out) if out.status.success() => {
                let text = String::from_utf8_lossy(&out.stdout);
                if text.contains(&pid.to_string()) {
                    ProcessStatus::Alive
                } else {
                    ProcessStatus::NotFound
                }
            }
            _ => ProcessStatus::NotFound,
        };
    }
    #[cfg(not(any(unix, windows)))]
    {
        ProcessStatus::NotFound
    }
}

pub fn process_exists(pid: u32) -> bool {
    matches!(
        process_status(pid),
        ProcessStatus::Alive | ProcessStatus::PermissionDenied
    )
}

pub fn terminate_process(pid: u32) -> io::Result<()> {
    if pid <= 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "refusing to terminate pid <= 1",
        ));
    }
    #[cfg(unix)]
    {
        let rc = unsafe { libc::kill(pid as libc::pid_t, libc::SIGTERM) };
        if rc == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
    #[cfg(windows)]
    {
        let status = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "taskkill failed"))
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "process termination unsupported on this platform",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_process_is_alive() {
        assert!(process_exists(std::process::id()));
    }

    #[test]
    fn pid_zero_is_never_alive() {
        assert_eq!(process_status(0), ProcessStatus::NotFound);
        assert!(!process_exists(0));
    }
}

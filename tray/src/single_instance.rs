//! Single-instance guard for tytus-tray.
//!
//! Uses a PID file at /tmp/tytus/tray.pid. On startup:
//!   1. If no pidfile → take the lock, write our pid, return Ok.
//!   2. If pidfile exists and that pid is alive → return Err (caller exits).
//!   3. If pidfile exists but the pid is dead (stale) → overwrite, take lock.
//!
//! On clean exit, `release()` removes the pidfile. On crash, the next launch
//! sweeps the stale file via the liveness check — no zombies, no manual
//! intervention.
//!
//! This is deliberately simpler than a flock()-based lock:
//!   - No lingering file descriptors that might leak across fork().
//!   - Survives the tray being killed with SIGKILL.
//!   - Easy to `tail` or `cat` for debugging.

use std::io::Write;
use std::path::PathBuf;

const PID_DIR: &str = "/tmp/tytus";
const PID_FILE: &str = "tray.pid";

fn pidfile_path() -> PathBuf {
    PathBuf::from(PID_DIR).join(PID_FILE)
}

/// Acquire the single-instance lock. Returns Err with a user-facing message
/// if another tray is already running.
pub fn acquire() -> Result<(), String> {
    let _ = std::fs::create_dir_all(PID_DIR);
    let path = pidfile_path();

    // If a pidfile exists, check whether the owning process is actually alive.
    if let Ok(contents) = std::fs::read_to_string(&path) {
        if let Ok(pid) = contents.trim().parse::<i32>() {
            if pid > 0 && is_process_alive(pid) {
                return Err(format!(
                    "another tytus-tray is already running (pid {}). \
                     Click the T in the menu bar.",
                    pid
                ));
            }
            // Stale pidfile from a crashed run — fall through and overwrite.
        }
    }

    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .map_err(|e| format!("cannot write pidfile {}: {}", path.display(), e))?;
    writeln!(f, "{}", std::process::id())
        .map_err(|e| format!("cannot write pid: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }

    Ok(())
}

/// Remove our pidfile, but only if it still points at us (defensive: don't
/// delete a newer instance's lock if we somehow outlived a successor).
pub fn release() {
    let path = pidfile_path();
    let our_pid = std::process::id();
    if let Ok(contents) = std::fs::read_to_string(&path) {
        if let Ok(pid) = contents.trim().parse::<u32>() {
            if pid == our_pid {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
}

/// kill(pid, 0) probes existence without sending a signal.
/// Returns true if the process exists (even if we can't signal it — EPERM
/// means it's alive but owned by another user / different privilege level).
fn is_process_alive(pid: i32) -> bool {
    #[cfg(unix)]
    {
        let ret = unsafe { libc::kill(pid, 0) };
        if ret == 0 {
            return true;
        }
        let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
        errno == libc::EPERM
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        false
    }
}

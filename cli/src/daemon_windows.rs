//! Windows daemon substrate.
//!
//! The rich Unix-socket command server is not migrated yet, but Windows is no
//! longer a compile-only stub: `daemon run` writes the platform control file
//! and PID file, and status commands report that minimal daemon process.

use atomek_core::platform::{ipc, paths, process};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub code: Option<String>,
}

pub fn socket_path() -> PathBuf {
    paths::runtime_dir().join("daemon.sock")
}

pub fn pid_path() -> PathBuf {
    paths::daemon_pid_file()
}

pub fn pid_file_pid() -> Option<i32> {
    std::fs::read_to_string(pid_path())
        .ok()?
        .trim()
        .parse()
        .ok()
}

pub fn process_alive(pid: i32) -> bool {
    pid > 1 && process::process_exists(pid as u32)
}

pub fn pid_is_tytus_daemon(_pid: i32) -> bool {
    false
}

pub async fn is_daemon_running() -> bool {
    pid_file_pid().is_some_and(process_alive)
}

pub async fn run_daemon() {
    let runtime = paths::runtime_dir();
    if let Err(e) = paths::ensure_private_dir(&runtime) {
        eprintln!("tytus: failed to create runtime dir: {}", e);
        return;
    }
    let pid = std::process::id();
    let pid_file = pid_path();
    let _ = std::fs::write(&pid_file, pid.to_string());
    let control = ipc::ControlFile::new(0, pid);
    if let Err(e) = ipc::write_control_file(&paths::control_file(), &control) {
        eprintln!("tytus: failed to write control file: {}", e);
        return;
    }
    eprintln!(
        "tytus-daemon minimal Windows substrate running (pid {}, control {})",
        pid,
        paths::control_file().display()
    );
    std::future::pending::<()>().await;
}

pub async fn send_command(cmd: &str, args: serde_json::Value) -> Option<Response> {
    send_command_timeout(cmd, args, std::time::Duration::from_secs(2)).await
}

pub async fn send_command_timeout(
    cmd: &str,
    _args: serde_json::Value,
    _timeout: std::time::Duration,
) -> Option<Response> {
    match cmd {
        "status" => {
            let control = ipc::read_control_file(&paths::control_file()).ok();
            let pid = control.as_ref().map(|c| c.pid as i32).or_else(pid_file_pid);
            let running = pid.is_some_and(process_alive);
            Some(Response {
                status: "ok".into(),
                data: Some(serde_json::json!({
                    "daemon": if running { "running" } else { "stopped" },
                    "platform": "windows",
                    "ipc": "control_file_minimal",
                    "pid": pid,
                    "control_file": paths::control_file(),
                })),
                error: None,
                code: None,
            })
        }
        _ => Some(Response {
            status: "error".into(),
            data: None,
            error: Some(format!(
                "Windows daemon command '{}' awaits Phase 2 HTTP IPC migration",
                cmd
            )),
            code: Some("WINDOWS_DAEMON_HTTP_IPC_PENDING".into()),
        }),
    }
}

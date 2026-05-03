//! Windows build stub for the Unix-socket daemon.
//!
//! The tray/daemon runtime is still Unix-socket based. Windows release assets
//! ship the CLI/MCP binaries first; daemon lifecycle commands return a clear
//! unsupported response instead of pulling Unix-only Tokio types into the
//! Windows build.

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
    std::env::temp_dir().join("tytus").join("daemon.sock")
}

pub fn pid_path() -> PathBuf {
    std::env::temp_dir().join("tytus").join("daemon.pid")
}

pub fn pid_file_pid() -> Option<i32> {
    std::fs::read_to_string(pid_path())
        .ok()?
        .trim()
        .parse()
        .ok()
}

pub fn process_alive(pid: i32) -> bool {
    if pid <= 1 {
        return false;
    }
    std::process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/NH"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
        .unwrap_or(false)
}

pub fn pid_is_tytus_daemon(_pid: i32) -> bool {
    false
}

pub async fn is_daemon_running() -> bool {
    false
}

pub async fn run_daemon() {
    eprintln!("tytus daemon is not supported on Windows in this release.");
}

pub async fn send_command(cmd: &str, args: serde_json::Value) -> Option<Response> {
    send_command_timeout(cmd, args, std::time::Duration::from_secs(2)).await
}

pub async fn send_command_timeout(
    _cmd: &str,
    _args: serde_json::Value,
    _timeout: std::time::Duration,
) -> Option<Response> {
    None
}

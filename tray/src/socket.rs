//! Communication with tytus-daemon via Unix socket.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

const SOCKET_PATH: &str = "/tmp/tytus/daemon.sock";

/// Poll daemon status. Returns default state if daemon is not running.
pub fn poll_daemon_status() -> super::TrayState {
    let resp = match send_command("status") {
        Some(r) => r,
        None => return super::TrayState::default(),
    };

    let data = match resp.get("data") {
        Some(d) => d,
        None => return super::TrayState {
            daemon_running: true,
            ..Default::default()
        },
    };

    let daemon = data.get("daemon").cloned().unwrap_or_default();
    let auth = data.get("auth").cloned().unwrap_or_default();
    let pods = data.get("pods").and_then(|p| p.as_array()).cloned().unwrap_or_default();

    let tunnel_active = pods.iter().any(|p| {
        p.get("tunnel_iface").and_then(|v| v.as_str()).is_some()
    });

    super::TrayState {
        daemon_running: true,
        logged_in: auth.get("logged_in").and_then(|v| v.as_bool()).unwrap_or(false),
        token_valid: auth.get("token_valid").and_then(|v| v.as_bool()).unwrap_or(false),
        email: auth.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        tier: auth.get("tier").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        pod_count: pods.len(),
        tunnel_active,
        daemon_pid: daemon.get("pid").and_then(|v| v.as_u64()).unwrap_or(0),
        uptime_secs: daemon.get("uptime_secs").and_then(|v| v.as_u64()).unwrap_or(0),
    }
}

/// Send a raw command to the daemon and return the full JSON response.
pub fn send_raw_command(cmd: &str) -> Option<serde_json::Value> {
    send_command(cmd)
}

fn send_command(cmd: &str) -> Option<serde_json::Value> {
    let mut stream = UnixStream::connect(SOCKET_PATH).ok()?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(3))).ok()?;

    let req = serde_json::json!({"cmd": cmd});
    let mut buf = serde_json::to_vec(&req).ok()?;
    buf.push(b'\n');
    stream.write_all(&buf).ok()?;
    stream.shutdown(std::net::Shutdown::Write).ok()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    serde_json::from_str(&line).ok()
}

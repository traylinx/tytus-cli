//! Build a `TrayState` snapshot from three independent signals:
//!
//!   1. **Gateway probe** (source of truth for "is my pod reachable?")
//!   2. **Daemon socket** (`/tmp/tytus/daemon.sock` — rich status if alive)
//!   3. **State file** (`~/Library/Application Support/tytus/state.json` —
//!      fallback for email / tier / pods / agent types when the daemon is offline)
//!
//! The daemon is treated as optional. Previously the tray showed 🔴
//! "daemon not running" for users whose tunnels worked perfectly — the
//! daemon only manages auth refresh, it has no bearing on whether the
//! user can call their pod right now. This module now reports ground
//! truth regardless of daemon state.
//!
//! The gateway probe runs on every poll (2s timeout, rarely >100ms on a
//! healthy tunnel). State-file reads are cheap (a few KB, warm cache).

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use super::PodInfo;

const SOCKET_PATH: &str = "/tmp/tytus/daemon.sock";

/// Build a full TrayState. Merges daemon response (if any), state.json
/// (if present), and a live gateway reachability probe.
pub fn poll_daemon_status() -> super::TrayState {
    let gateway_reachable = super::gateway_probe::probe_gateway();
    let daemon_snapshot = daemon_status();
    let file_snapshot = read_state_file();

    let mut out = super::TrayState {
        autostart_installed: super::check_autostart_installed(),
        tray_autostart_installed: super::check_tray_autostart_installed(),
        app_bundle_installed: super::check_app_bundle_installed(),
        gateway_reachable,
        ..Default::default()
    };

    if let Some(d) = daemon_snapshot {
        out.daemon_running = true;
        out.daemon_pid = d.daemon_pid;
        out.uptime_secs = d.uptime_secs;
        out.logged_in = d.logged_in;
        out.token_valid = d.token_valid;
        out.email = d.email;
        out.tier = d.tier;
        out.pods = d.pods;
        out.tunnel_active = out.pods.iter().any(|p| p.tunnel_active);
    } else if let Some(f) = file_snapshot {
        out.daemon_running = false;
        out.logged_in = f.logged_in;
        out.token_valid = f.token_valid_local;
        out.email = f.email;
        out.tier = f.tier;
        out.pods = f.pods;
        out.tunnel_active = out.pods.iter().any(|p| p.tunnel_active);
    }

    // Derived fields: unit budget (used vs limit).
    out.units_used = out.pods.iter().map(|p| p.units()).sum();
    out.units_limit = super::units_for_tier(&out.tier);

    // If the gateway probe succeeded, treat the tunnel as active even
    // when state.json/daemon disagree — the probe is ground truth. This
    // is what fixes the utun5→utun4 renumbering mismatch without needing
    // to actively rewrite state.json.
    if gateway_reachable {
        out.tunnel_active = true;
    }

    out
}

/// Raw JSON body from the daemon, for callers that need fields the
/// TrayState struct doesn't surface (e.g. stable_ai_endpoint).
pub fn send_raw_command(cmd: &str) -> Option<serde_json::Value> {
    send_command(cmd)
}

// ── Daemon path ─────────────────────────────────────────────

struct DaemonSnap {
    daemon_pid: u64,
    uptime_secs: u64,
    logged_in: bool,
    token_valid: bool,
    email: String,
    tier: String,
    pods: Vec<PodInfo>,
}

fn daemon_status() -> Option<DaemonSnap> {
    let resp = send_command("status")?;
    let data = resp.get("data")?;
    let daemon = data.get("daemon").cloned().unwrap_or_default();
    let auth = data.get("auth").cloned().unwrap_or_default();
    let pods_json = data.get("pods").and_then(|p| p.as_array()).cloned().unwrap_or_default();
    let pods = pods_json.iter().map(|p| PodInfo {
        pod_id: p.get("pod_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        agent_type: p.get("agent_type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        tunnel_active: p.get("tunnel_iface").and_then(|v| v.as_str()).is_some(),
    }).collect();
    Some(DaemonSnap {
        daemon_pid: daemon.get("pid").and_then(|v| v.as_u64()).unwrap_or(0),
        uptime_secs: daemon.get("uptime_secs").and_then(|v| v.as_u64()).unwrap_or(0),
        logged_in: auth.get("logged_in").and_then(|v| v.as_bool()).unwrap_or(false),
        token_valid: auth.get("token_valid").and_then(|v| v.as_bool()).unwrap_or(false),
        email: auth.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        tier: auth.get("tier").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        pods,
    })
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

// ── State.json fallback ─────────────────────────────────────

struct FileSnap {
    logged_in: bool,
    token_valid_local: bool,
    email: String,
    tier: String,
    pods: Vec<PodInfo>,
}

fn state_file_path() -> PathBuf {
    let config = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    config.join("tytus").join("state.json")
}

fn read_state_file() -> Option<FileSnap> {
    let path = state_file_path();
    let raw = std::fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;

    let email = v.get("email").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let tier = v.get("tier").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let has_email = !email.is_empty();
    let has_access_token = v.get("access_token")
        .and_then(|x| x.as_str())
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let expires_at_ms = v.get("expires_at_ms").and_then(|x| x.as_i64());
    let token_valid_local = match (has_access_token, expires_at_ms) {
        (true, Some(exp)) => chrono_now_ms() + 300_000 < exp,
        _ => false,
    };

    let pods_json = v.get("pods").and_then(|x| x.as_array()).cloned().unwrap_or_default();
    let pods = pods_json.iter().map(|p| PodInfo {
        pod_id: p.get("pod_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        agent_type: p.get("agent_type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        tunnel_active: p.get("tunnel_iface").and_then(|v| v.as_str()).is_some(),
    }).collect();

    Some(FileSnap {
        logged_in: has_email,
        token_valid_local,
        email,
        tier,
        pods,
    })
}

fn chrono_now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

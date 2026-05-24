//! Build a `TrayState` snapshot from three independent signals:
//!
//!   1. **Gateway probe** (source of truth for "is my pod reachable?")
//!   2. **Daemon control plane** (localhost HTTP via `control.json`, falling
//!      back to legacy `/tmp/tytus/daemon.sock` on Unix)
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

#[cfg(unix)]
use std::io::{BufRead, BufReader, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use super::PodInfo;

/// Derive the per-pod gateway auth token — the same sha256 the edge
/// plugin + openclaw startup use — so the tray menu and the install
/// wizard agree on a working `?token=` even when state.json hasn't
/// been backfilled yet. Without this, a freshly-installed pod shows
/// "Connect  Open in Browser" (tunnel fallback) in the tray menu
/// because `public_ui_url()` bails on the null token, even though
/// the public edge path would work fine. Invariant-preserving: we
/// only derive when the CLI / daemon didn't already provide one.
fn derive_gateway_token(pod_api_key: &str, pod_id: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(pod_api_key.as_bytes());
    h.update(pod_id.as_bytes());
    hex::encode(&h.finalize()[..24])
}

/// Build a PodInfo from the raw JSON Provider/daemon/state returned
/// us, applying the two "the CLI forgot to populate this" fixes:
///   1. Slug inheritance — borrow another pod's `edge_public_url`
///      when this pod's is null (all a user's pods share one slug).
///   2. Gateway-token derivation — compute from pod_api_key when the
///      stored gateway_token is null. Deterministic formula.
///
/// `shared_base` is the first non-empty `edge_public_url` found across
/// the caller's pod set. `api_keys_by_pod` is a route_id/pod_id → pod_api_key
/// map supplied by the caller (typically built from state.json, which
/// always has the raw pod key — the daemon's JSON response REDACTS
/// pod_api_key as a secrets-hygiene measure, so derivation would
/// otherwise fail when the tray was reading from the live daemon).
fn build_pod_info(
    p: &serde_json::Value,
    shared_base: Option<&str>,
    api_keys_by_pod: &std::collections::HashMap<String, String>,
) -> PodInfo {
    let pod_id = p
        .get("pod_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let route_id = p
        .get("route_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let stored_edge = p
        .get("edge_public_url")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let edge_public_url = stored_edge.or_else(|| shared_base.map(|s| s.to_string()));
    let stored_token = p
        .get("gateway_token")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    // Try pod_api_key inline first (state.json path), then fall back
    // to the state-loaded map (daemon-socket path where the key got
    // stripped). Either way derivation is sha256(pod_api_key||pod_id).
    let pod_api_key: Option<String> = p
        .get("pod_api_key")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| {
            route_id
                .as_ref()
                .and_then(|rid| api_keys_by_pod.get(rid).cloned())
        })
        .or_else(|| api_keys_by_pod.get(&pod_id).cloned());
    let derived_token = pod_api_key
        .as_ref()
        .map(|k| derive_gateway_token(k, &pod_id));
    let gateway_token = stored_token.or(derived_token);

    let pod_public_url = p
        .get("pod_public_url")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    PodInfo {
        pod_id,
        route_id,
        custom_display_name: p
            .get("display_name")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string()),
        agent_type: p
            .get("agent_type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        agent_units: p
            .get("agent_units")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32),
        tunnel_active: p.get("tunnel_iface").and_then(|v| v.as_str()).is_some(),
        stable_ai_endpoint: p
            .get("stable_ai_endpoint")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        stable_user_key: p
            .get("stable_user_key")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        edge_public_url,
        gateway_token,
        pod_public_url,
    }
}

/// Build the route_id/pod_id → pod_api_key map from state.json. Cheap (a few
/// KB file read + JSON parse) and called only when a pod info
/// construction path needs to supply pod_api_key for derivation —
/// which is every tray poll, ~every 2 s. Fails silently to empty map
/// so `build_pod_info` just skips derivation when we can't read.
fn load_api_keys_from_state() -> std::collections::HashMap<String, String> {
    let path = state_file_path();
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return Default::default(),
    };
    let v: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return Default::default(),
    };
    let mut m = std::collections::HashMap::new();
    if let Some(arr) = v.get("pods").and_then(|x| x.as_array()) {
        for p in arr {
            let id = p.get("pod_id").and_then(|v| v.as_str()).unwrap_or("");
            let route = p.get("route_id").and_then(|v| v.as_str()).unwrap_or("");
            let key = p.get("pod_api_key").and_then(|v| v.as_str()).unwrap_or("");
            if !key.is_empty() {
                if !route.is_empty() {
                    m.insert(route.to_string(), key.to_string());
                }
                if !id.is_empty() {
                    m.entry(id.to_string()).or_insert_with(|| key.to_string());
                }
            }
        }
    }
    m
}

/// Companion for the "daemon gave us null edge URLs but state.json
/// has a good one" case — returns the first non-empty
/// `edge_public_url` from state.json. Same formula, different data
/// source.
fn load_shared_base_from_state() -> Option<String> {
    let path = state_file_path();
    let raw = std::fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    v.get("pods").and_then(|x| x.as_array()).and_then(|arr| {
        arr.iter().find_map(|p| {
            p.get("edge_public_url")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        })
    })
}

/// Find the first non-empty edge_public_url in a pod array. Used as
/// the "shared base URL" for slug inheritance — all a user's pods
/// share the same `<slug>.tytus.traylinx.com`.
fn shared_edge_base(pods: &[serde_json::Value]) -> Option<String> {
    pods.iter().find_map(|p| {
        p.get("edge_public_url")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    })
}

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
        // Optimistic default — only flipped to false when the daemon
        // explicitly reports a keychain problem. Prevents a spurious
        // warning on first paint before the first poll completes.
        keychain_healthy: true,
        ..Default::default()
    };

    // Seed from the daemon (runtime truth: pid, uptime, its view of auth +
    // pods).
    if let Some(ref d) = daemon_snapshot {
        out.daemon_running = true;
        out.daemon_pid = d.daemon_pid;
        out.uptime_secs = d.uptime_secs;
        out.logged_in = d.logged_in;
        out.token_valid = d.token_valid;
        out.email = d.email.clone();
        out.tier = d.tier.clone();
        out.pods = d.pods.clone();
        out.keychain_healthy = d.keychain_healthy;
        out.last_refresh_error = d.last_refresh_error.clone();
    } else if let Some(pid) = daemon_pidfile_live() {
        // Command channel can be temporarily unavailable while the daemon
        // process is still alive (socket/control restart, stale HTTP control
        // port, or slow auth loop). Keep `/api/state` aligned with
        // `/api/daemon/status`: liveness comes from the pidfile/process
        // substrate, richer auth fields come from the command response when
        // available. Uptime is unknown in this fallback, so leave it at 0.
        out.daemon_running = true;
        out.daemon_pid = pid;
    } else {
        // In the desktop app the tray process itself serves localhost:4242.
        // There may be no separate legacy tytus-daemon socket/pidfile, but
        // reporting "daemon offline" while this process is serving the OS is
        // wrong and scares users. Treat the current tray as the local daemon.
        out.daemon_running = true;
        out.daemon_pid = std::process::id() as u64;
    }

    // Overlay state.json. The file is the atomic source of truth for auth
    // — any CLI command (`tytus login`, `tytus connect`, `tytus revoke`)
    // writes it immediately. The daemon's in-memory cache can lag by up
    // to a refresh tick AND, critically, can get pinned to `NeedsLogin`
    // when the macOS keychain ACL prompt is pending — in that state the
    // daemon lies about `logged_in=false` while the file plainly has
    // fresh tokens. Trust the file whenever it disagrees on logged-in.
    // Daemon-exclusive fields (pid, uptime) stay from the daemon snapshot.
    if let Some(f) = file_snapshot {
        if f.logged_in && !out.logged_in {
            out.logged_in = true;
        }
        if f.token_valid_local {
            out.token_valid = true;
        }
        if out.email.is_empty() {
            out.email = f.email;
        }
        if out.tier.is_empty() {
            out.tier = f.tier;
        }
        // Prefer the file's pod list whenever present. The daemon can lag
        // after a sync and can strip/lose UI-critical metadata (route_id,
        // display_name, stable keys) while state.json is the atomic local
        // source of truth written by `tytus status/connect`.
        if !f.pods.is_empty() {
            out.pods = f.pods;
        }
    }

    // A successful browser/CLI login writes a fresh access token to state.json
    // immediately, but the long-running daemon can keep reporting its previous
    // refresh failure until its next auth tick. Do not let that stale socket
    // field pin TytusOS in "Session expired" while the tray/state file already
    // prove the session is usable. Real expiry still surfaces when the local
    // access token is no longer valid.
    out.last_refresh_error = effective_last_refresh_error(
        out.last_refresh_error,
        out.token_valid,
        out.keychain_healthy,
    );

    // Derived fields: unit budget (used vs limit).
    out.units_used = out.pods.iter().map(|p| p.units()).sum();
    out.units_limit = super::units_for_tier(&out.tier);

    // The gateway probe is ground truth — resolve state.json/daemon
    // disagreement in both directions:
    //   reachable + claim-inactive → tunnel IS active (utun renumbering
    //     mismatch; don't demand a Connect click that would no-op)
    //   !reachable + claim-active  → tunnel is stale (PID dead, state
    //     file not reaped). Show Connect, not a useless Disconnect.
    // Without this second branch, the menu reports "Pod unreachable" on
    // one line but offers only a Disconnect on the next — clicking does
    // nothing because `tytus disconnect` SIGTERMs a long-dead PID.
    out.tunnel_active = gateway_reachable;

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
    keychain_healthy: bool,
    last_refresh_error: Option<String>,
    // Captured for future surfacing in a "daemon stuck for Ns" diagnostic
    // row. Currently unused in the menu; keep the field so the parser
    // stays in sync with the daemon's schema.
    #[allow(dead_code)]
    stuck_for_secs: Option<u64>,
}

fn daemon_pidfile_live() -> Option<u64> {
    let pid_text = std::fs::read_to_string(atomek_core::platform::paths::daemon_pid_file()).ok()?;
    daemon_pidfile_live_from_text(Some(pid_text.trim()), |pid| {
        atomek_core::platform::process::process_exists(pid)
    })
}

fn daemon_pidfile_live_from_text(
    pid_text: Option<&str>,
    exists: impl Fn(u32) -> bool,
) -> Option<u64> {
    let pid = pid_text?.trim().parse::<u32>().ok()?;
    if pid <= 1 || !exists(pid) {
        return None;
    }
    Some(pid as u64)
}

fn daemon_status() -> Option<DaemonSnap> {
    let resp = send_command("status")?;
    let data = resp.get("data")?;
    let daemon = data.get("daemon").cloned().unwrap_or_default();
    let auth = data.get("auth").cloned().unwrap_or_default();
    let pods_json = data
        .get("pods")
        .and_then(|p| p.as_array())
        .cloned()
        .unwrap_or_default();
    // Daemon strips pod_api_key from its status response — pull it
    // from state.json so gateway_token derivation has something to
    // hash. Also fold state.json's edge URLs into shared_base since
    // the daemon path sometimes carries null edge URLs for pods that
    // state.json DOES have backfilled.
    let api_keys = load_api_keys_from_state();
    let shared_base = shared_edge_base(&pods_json).or_else(load_shared_base_from_state);
    let pods = pods_json
        .iter()
        .map(|p| build_pod_info(p, shared_base.as_deref(), &api_keys))
        .collect();
    Some(DaemonSnap {
        daemon_pid: daemon.get("pid").and_then(|v| v.as_u64()).unwrap_or(0),
        uptime_secs: daemon
            .get("uptime_secs")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        logged_in: auth
            .get("logged_in")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        token_valid: auth
            .get("token_valid")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        email: auth
            .get("email")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        tier: auth
            .get("tier")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        pods,
        // Missing fields (old daemon talking to a new tray) default
        // optimistic — `keychain_healthy: true` — so an out-of-date
        // daemon doesn't spuriously trip the warning row.
        keychain_healthy: daemon
            .get("keychain_healthy")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        last_refresh_error: daemon
            .get("last_refresh_error")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        stuck_for_secs: daemon.get("stuck_for_secs").and_then(|v| v.as_u64()),
    })
}

#[cfg(unix)]
fn send_command(cmd: &str) -> Option<serde_json::Value> {
    if let Some(v) = send_http_command(cmd) {
        return Some(v);
    }

    let socket_path = atomek_core::platform::paths::runtime_dir().join("daemon.sock");
    let mut stream = UnixStream::connect(socket_path).ok()?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(3)))
        .ok()?;

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

#[cfg(not(unix))]
fn send_command(cmd: &str) -> Option<serde_json::Value> {
    send_http_command(cmd)
}

fn send_http_command(cmd: &str) -> Option<serde_json::Value> {
    let control = atomek_core::platform::ipc::read_control_file(
        &atomek_core::platform::paths::control_file(),
    )
    .ok()?;
    if control.port == 0 || !control.bind.is_loopback() {
        return None;
    }
    let token = atomek_core::platform::ipc::read_control_token_file(&control.token_file).ok()?;
    let url = format!("http://{}:{}/v1/command", control.bind, control.port);
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .ok()?;
    client
        .post(url)
        .bearer_auth(token)
        .json(&serde_json::json!({"cmd": cmd, "args": serde_json::Value::Null}))
        .send()
        .ok()?
        .json::<serde_json::Value>()
        .ok()
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

    let email = v
        .get("email")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let tier = v
        .get("tier")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let has_email = !email.is_empty();
    let has_access_token = v
        .get("access_token")
        .and_then(|x| x.as_str())
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let expires_at_ms = v.get("expires_at_ms").and_then(|x| x.as_i64());
    let token_valid_local = match (has_access_token, expires_at_ms) {
        (true, Some(exp)) => chrono_now_ms() + 300_000 < exp,
        _ => false,
    };

    let pods_json = v
        .get("pods")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();
    let shared_base = shared_edge_base(&pods_json);
    // state.json has pod_api_key inline; caller already provides it.
    // Still pass a map (likely redundant but harmless) so both code
    // paths share one builder signature.
    let api_keys = pods_json
        .iter()
        .filter_map(|p| {
            let id = p.get("pod_id").and_then(|v| v.as_str()).unwrap_or("");
            let route = p.get("route_id").and_then(|v| v.as_str()).unwrap_or("");
            let key = p.get("pod_api_key").and_then(|v| v.as_str())?;
            if key.is_empty() {
                return None;
            }
            let map_key = if !route.is_empty() { route } else { id };
            if map_key.is_empty() {
                None
            } else {
                Some((map_key.to_string(), key.to_string()))
            }
        })
        .collect::<std::collections::HashMap<_, _>>();
    let pods = pods_json
        .iter()
        .map(|p| build_pod_info(p, shared_base.as_deref(), &api_keys))
        .collect();

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

fn refresh_error_requires_login(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    normalized.contains("refresh token expired")
        || normalized.contains("run `tytus login`")
        || normalized.contains("run 'tytus login'")
        || normalized.contains("no refresh token available")
}

fn effective_last_refresh_error(
    error: Option<String>,
    token_valid_local: bool,
    keychain_healthy: bool,
) -> Option<String> {
    match error {
        Some(err)
            if token_valid_local && keychain_healthy && refresh_error_requires_login(&err) =>
        {
            None
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        daemon_pidfile_live_from_text, effective_last_refresh_error, refresh_error_requires_login,
    };

    #[test]
    fn daemon_pidfile_live_accepts_existing_pid() {
        assert_eq!(
            daemon_pidfile_live_from_text(Some("4242"), |pid| pid == 4242),
            Some(4242)
        );
    }

    #[test]
    fn daemon_pidfile_live_rejects_missing_invalid_or_dead_pid() {
        assert_eq!(daemon_pidfile_live_from_text(None, |_| true), None);
        assert_eq!(daemon_pidfile_live_from_text(Some("nope"), |_| true), None);
        assert_eq!(daemon_pidfile_live_from_text(Some("1"), |_| true), None);
        assert_eq!(daemon_pidfile_live_from_text(Some("4242"), |_| false), None);
    }

    #[test]
    fn classifies_login_required_refresh_errors() {
        assert!(refresh_error_requires_login(
            "refresh token expired — run `tytus login`"
        ));
        assert!(refresh_error_requires_login(
            "No refresh token available — run 'tytus login'"
        ));
        assert!(!refresh_error_requires_login("temporary sentinel 502"));
    }

    #[test]
    fn suppresses_stale_expired_error_when_local_token_is_valid() {
        let err = Some("refresh token expired — run `tytus login`".to_string());
        assert_eq!(effective_last_refresh_error(err, true, true), None);
    }

    #[test]
    fn keeps_expired_error_when_local_token_is_not_valid() {
        let err = Some("refresh token expired — run `tytus login`".to_string());
        assert_eq!(effective_last_refresh_error(err.clone(), false, true), err);
    }

    #[test]
    fn keeps_non_auth_refresh_errors_even_with_valid_token() {
        let err = Some("temporary sentinel 502".to_string());
        assert_eq!(effective_last_refresh_error(err.clone(), true, true), err);
    }
}

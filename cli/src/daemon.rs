//! Tytus Daemon — persistent background process that owns tokens, tunnel, and health.
//!
//! The daemon listens on a Unix socket for JSON-line commands from the CLI.
//! It manages the token lifecycle (background refresh), state persistence
//! (sole writer to state.json), and tunnel monitoring.
//!
//! Design: Phase 1 — daemon handles auth + status. Tunnel ownership is Phase 2.

use crate::state::CliState;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::watch;

/// Default socket path. Lives next to PID files so cleanup is easy.
const SOCKET_DIR: &str = "/tmp/tytus";
const SOCKET_NAME: &str = "daemon.sock";

/// Daemon PID file for liveness detection by the CLI.
const PID_FILE: &str = "daemon.pid";

// ── Protocol types ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct Request {
    pub cmd: String,
    #[allow(dead_code)] // Used in Phase 2 for connect/disconnect args
    #[serde(default)]
    pub args: serde_json::Value,
}

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

impl Response {
    fn ok(data: serde_json::Value) -> Self {
        Self { status: "ok".into(), data: Some(data), error: None, code: None }
    }
    fn err(code: &str, msg: impl Into<String>) -> Self {
        Self { status: "error".into(), data: None, error: Some(msg.into()), code: Some(code.into()) }
    }
}

// ── Daemon state ────────────────────────────────────────────

pub struct DaemonState {
    pub cli_state: CliState,
    pub started_at: std::time::Instant,
    pub last_refresh: Option<std::time::Instant>,
    pub daemon_status: DaemonStatus,
}

/// Shared daemon context: Mutex-guarded state + immutable HttpClient.
pub struct DaemonCtx {
    pub state: tokio::sync::Mutex<DaemonState>,
    pub http: atomek_core::HttpClient,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DaemonStatus {
    Running,
    NeedsLogin,
    Refreshing,
}

// ── Socket path helpers ─────────────────────────────────────

pub fn socket_path() -> PathBuf {
    PathBuf::from(SOCKET_DIR).join(SOCKET_NAME)
}

pub fn pid_path() -> PathBuf {
    PathBuf::from(SOCKET_DIR).join(PID_FILE)
}

/// Check if the daemon is running by probing the socket.
pub async fn is_daemon_running() -> bool {
    let sock = socket_path();
    if !sock.exists() {
        return false;
    }
    match tokio::net::UnixStream::connect(&sock).await {
        Ok(_stream) => true,
        Err(_) => {
            let _ = std::fs::remove_file(&sock);
            false
        }
    }
}

// ── Daemon main loop ────────────────────────────────────────

pub async fn run_daemon() {
    let sock_dir = Path::new(SOCKET_DIR);
    let _ = std::fs::create_dir_all(sock_dir);
    // Security: tighten /tmp/tytus/ to owner-only. See PENTEST finding E5.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(sock_dir, std::fs::Permissions::from_mode(0o700));
    }
    let sock = socket_path();

    // Clean up stale socket
    if sock.exists() {
        if is_daemon_running().await {
            eprintln!("tytus: daemon is already running");
            std::process::exit(1);
        }
        let _ = std::fs::remove_file(&sock);
    }

    let listener = match UnixListener::bind(&sock) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("tytus: failed to bind daemon socket: {}", e);
            std::process::exit(1);
        }
    };

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&sock, std::fs::Permissions::from_mode(0o600));
    }

    let pid_file = pid_path();
    let _ = std::fs::write(&pid_file, format!("{}", std::process::id()));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&pid_file, std::fs::Permissions::from_mode(0o600));
    }

    let state = CliState::load();
    let http = atomek_core::HttpClient::new();
    let daemon_status = if state.is_logged_in() {
        DaemonStatus::Running
    } else {
        DaemonStatus::NeedsLogin
    };

    let ctx = std::sync::Arc::new(DaemonCtx {
        state: tokio::sync::Mutex::new(DaemonState {
            cli_state: state,
            started_at: std::time::Instant::now(),
            last_refresh: None,
            daemon_status,
        }),
        http,
    });

    // Shutdown signal: watch channel (false = running, true = shutting down)
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    tracing::info!("tytus-daemon started (pid {}), listening on {}", std::process::id(), sock.display());
    eprintln!("tytus-daemon running (pid {})", std::process::id());

    // Spawn token refresh background task
    let refresh_ctx = ctx.clone();
    let refresh_rx = shutdown_rx.clone();
    tokio::spawn(async move {
        token_refresh_loop(refresh_ctx, refresh_rx).await;
    });

    // Spawn SIGTERM/SIGINT handler
    let signal_tx = shutdown_tx.clone();
    tokio::spawn(async move {
        let mut sigterm = tokio::signal::unix::signal(
            tokio::signal::unix::SignalKind::terminate(),
        ).expect("Failed to register SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Daemon received SIGINT — shutting down");
            }
            _ = sigterm.recv() => {
                tracing::info!("Daemon received SIGTERM — shutting down");
            }
        }
        let _ = signal_tx.send(true);
    });

    // Accept loop
    let mut accept_shutdown = shutdown_rx.clone();
    loop {
        tokio::select! {
            accept = listener.accept() => {
                match accept {
                    Ok((stream, _addr)) => {
                        let st = ctx.clone();
                        let tx = shutdown_tx.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(stream, st, tx).await {
                                tracing::warn!("Connection handler error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        tracing::warn!("Accept error: {}", e);
                    }
                }
            }
            _ = accept_shutdown.changed() => {
                if *accept_shutdown.borrow() {
                    tracing::info!("Daemon shutting down gracefully");
                    break;
                }
            }
        }
    }

    // Cleanup
    let _ = std::fs::remove_file(&sock);
    let _ = std::fs::remove_file(&pid_file);
    tracing::info!("Daemon exited cleanly");
}

// ── Connection handler ──────────────────────────────────────

async fn handle_connection(
    stream: UnixStream,
    ctx: std::sync::Arc<DaemonCtx>,
    shutdown_tx: watch::Sender<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        let line = line.trim().to_string();
        if line.is_empty() { continue; }

        let req: Request = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = Response::err("PARSE_ERROR", format!("Invalid JSON: {}", e));
                let mut buf = serde_json::to_vec(&resp)?;
                buf.push(b'\n');
                writer.write_all(&buf).await?;
                continue;
            }
        };

        let is_shutdown = req.cmd == "shutdown";
        let resp = dispatch_command(&req, &ctx, &shutdown_tx).await;
        let mut buf = serde_json::to_vec(&resp)?;
        buf.push(b'\n');
        writer.write_all(&buf).await?;

        if is_shutdown { break; }
    }

    Ok(())
}

// ── Command dispatch ────────────────────────────────────────

async fn dispatch_command(
    req: &Request,
    ctx: &std::sync::Arc<DaemonCtx>,
    shutdown_tx: &watch::Sender<bool>,
) -> Response {
    match req.cmd.as_str() {
        "ping" => Response::ok(serde_json::json!({"pong": true})),

        "status" => {
            let ds = ctx.state.lock().await;
            let uptime = ds.started_at.elapsed().as_secs();
            let token_valid = ds.cli_state.has_valid_token();
            let logged_in = ds.cli_state.is_logged_in();
            // Security: emit only stable values over the daemon socket.
            // No internal pod IPs (ai_endpoint), no raw per-pod keys (pod_api_key),
            // no droplet identifiers. The CLI already redacts the same way in
            // print_*_status; the daemon must not leak more than the CLI does.
            // See docs/PENTEST-RESULTS-2026-04-12.md finding E4.
            let pods: Vec<_> = ds.cli_state.pods.iter().map(|p| {
                serde_json::json!({
                    "pod_id": p.pod_id,
                    "agent_type": p.agent_type,
                    "tunnel_iface": p.tunnel_iface,
                    "stable_ai_endpoint": p.stable_ai_endpoint,
                    "stable_user_key": p.stable_user_key,
                })
            }).collect();
            let last_refresh = ds.last_refresh.map(|t| t.elapsed().as_secs());

            Response::ok(serde_json::json!({
                "daemon": {
                    "pid": std::process::id(),
                    "uptime_secs": uptime,
                    "status": ds.daemon_status,
                    "last_refresh_secs_ago": last_refresh,
                },
                "auth": {
                    "logged_in": logged_in,
                    "token_valid": token_valid,
                    "email": ds.cli_state.email,
                    "tier": ds.cli_state.tier,
                    "expires_at_ms": ds.cli_state.expires_at_ms,
                },
                "pods": pods,
            }))
        }

        "refresh" => {
            let mut ds = ctx.state.lock().await;
            match super::ensure_token(&mut ds.cli_state, &ctx.http).await {
                Ok(()) => {
                    ds.last_refresh = Some(std::time::Instant::now());
                    ds.daemon_status = DaemonStatus::Running;
                    Response::ok(serde_json::json!({"refreshed": true}))
                }
                Err(e) => {
                    ds.daemon_status = DaemonStatus::NeedsLogin;
                    Response::err("AUTH_EXPIRED", format!("Token refresh failed: {}", e))
                }
            }
        }

        "shutdown" => {
            let _ = shutdown_tx.send(true);
            Response::ok(serde_json::json!({"shutting_down": true}))
        }

        other => Response::err("UNKNOWN_CMD", format!("Unknown command: {}", other)),
    }
}

// ── Background token refresh ────────────────────────────────

/// Nominal refresh cadence. Keeps the sliding RT window well inside Sentinel's
/// server-side RT TTL (~24h). Every tick, `ensure_token` decides whether to
/// actually hit the network — cheap no-op when the token is still fresh.
const REFRESH_TICK: std::time::Duration = std::time::Duration::from_secs(1800); // 30 min

/// Backoff schedule on transient failure. Never gives up — the whole point of
/// the daemon is to survive 24/7 across wakeups, Wi-Fi switches, VPN flaps.
const BACKOFF_STEPS: &[u64] = &[60, 300, 900, 1800, 3600]; // 1m, 5m, 15m, 30m, 1h cap

async fn token_refresh_loop(
    ctx: std::sync::Arc<DaemonCtx>,
    mut shutdown: watch::Receiver<bool>,
) {
    // Warm-up: do a refresh 10s after startup so we prove the tokens work
    // (and rotate them) before the user next invokes the CLI. Without this,
    // a freshly-booted machine waits 30 min before the first refresh, which
    // can leave the daemon blind to a keychain RT that's already expired.
    let mut next_wait = std::time::Duration::from_secs(10);
    let mut backoff_idx: usize = 0;

    loop {
        tokio::select! {
            _ = tokio::time::sleep(next_wait) => {
                let outcome = refresh_once(&ctx).await;
                match outcome {
                    RefreshOutcome::Ok => {
                        backoff_idx = 0;
                        next_wait = REFRESH_TICK;
                    }
                    RefreshOutcome::NotLoggedIn => {
                        // No credentials to refresh with — stay idle but keep the
                        // loop alive so `tytus login` + SIGHUP-free recovery works.
                        backoff_idx = 0;
                        next_wait = REFRESH_TICK;
                    }
                    RefreshOutcome::AuthExpired => {
                        // RT genuinely dead server-side. User must re-login.
                        // Don't hammer the server — back off long, but keep
                        // retrying in case the user runs `tytus login` and the
                        // daemon picks up the new RT from keychain on next tick.
                        next_wait = std::time::Duration::from_secs(*BACKOFF_STEPS.last().unwrap());
                    }
                    RefreshOutcome::Transient => {
                        // Network / DNS / server hiccup. Exponential backoff,
                        // never exit.
                        let step = BACKOFF_STEPS
                            .get(backoff_idx)
                            .copied()
                            .unwrap_or(*BACKOFF_STEPS.last().unwrap());
                        next_wait = std::time::Duration::from_secs(step);
                        backoff_idx = (backoff_idx + 1).min(BACKOFF_STEPS.len() - 1);
                    }
                }
            }
            _ = shutdown.changed() => {
                if *shutdown.borrow() {
                    tracing::debug!("Token refresh loop shutting down");
                    break;
                }
            }
        }
    }
}

enum RefreshOutcome {
    Ok,
    NotLoggedIn,
    AuthExpired,
    Transient,
}

async fn refresh_once(ctx: &std::sync::Arc<DaemonCtx>) -> RefreshOutcome {
    let mut ds = ctx.state.lock().await;

    // If we previously saw NeedsLogin, re-read state + keychain — the user may
    // have just run `tytus login` in another process, which writes a new RT
    // to the keychain that our in-memory copy doesn't know about. Without
    // this reload the daemon would keep trying the dead RT forever.
    if ds.daemon_status == DaemonStatus::NeedsLogin || !ds.cli_state.is_logged_in() {
        let fresh = CliState::load();
        if fresh.is_logged_in() {
            ds.cli_state = fresh;
            ds.daemon_status = DaemonStatus::Running;
        } else {
            ds.daemon_status = DaemonStatus::NeedsLogin;
            return RefreshOutcome::NotLoggedIn;
        }
    }

    ds.daemon_status = DaemonStatus::Refreshing;
    let result = super::ensure_token(&mut ds.cli_state, &ctx.http).await;

    match result {
        Ok(()) => {
            ds.last_refresh = Some(std::time::Instant::now());
            ds.daemon_status = DaemonStatus::Running;
            // Persist with save_critical: we may have rotated RT and the
            // new RT is already in keychain (via update_tokens), but we
            // want expires_at_ms / access_token durable in case of crash.
            if let Err(e) = ds.cli_state.save_critical() {
                tracing::error!("Daemon failed to save state after refresh: {}", e);
            }
            // Best-effort sync of subscription data — isolate its failure
            // from token refresh success.
            super::sync_tytus(&mut ds.cli_state, &ctx.http).await;
            let _ = ds.cli_state.save_critical();
            tracing::debug!("Background token refresh: OK");
            RefreshOutcome::Ok
        }
        Err(atomek_core::AtomekError::AuthExpired) => {
            ds.daemon_status = DaemonStatus::NeedsLogin;
            tracing::warn!("Background refresh: refresh token expired (needs re-login)");
            RefreshOutcome::AuthExpired
        }
        Err(e) => {
            // Do NOT flip to NeedsLogin — user auth is still valid, this is
            // a transient error (network, DNS, Sentinel hiccup). Stay in
            // Running so `tytus status` doesn't lie.
            tracing::warn!("Background token refresh (transient): {}", e);
            if ds.daemon_status != DaemonStatus::NeedsLogin {
                ds.daemon_status = DaemonStatus::Running;
            }
            RefreshOutcome::Transient
        }
    }
}

// ── Client helper (used by CLI to talk to daemon) ───────────

/// Send a command to the daemon and return the parsed response.
/// Returns None if daemon is not running.
pub async fn send_command(cmd: &str, args: serde_json::Value) -> Option<Response> {
    let sock = socket_path();
    let stream = tokio::net::UnixStream::connect(&sock).await.ok()?;
    let (reader, mut writer) = stream.into_split();

    let req = serde_json::json!({"cmd": cmd, "args": args});
    let mut buf = serde_json::to_vec(&req).ok()?;
    buf.push(b'\n');
    writer.write_all(&buf).await.ok()?;
    writer.shutdown().await.ok()?;

    let mut lines = BufReader::new(reader).lines();
    let line = lines.next_line().await.ok()??;
    serde_json::from_str(&line).ok()
}

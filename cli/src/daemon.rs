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
    /// Last observed mtime of state.json, used by the file watcher to
    /// suppress no-op reloads. `None` before first reload.
    pub last_state_mtime: Option<std::time::SystemTime>,
    /// Becomes `false` the moment a `get_refresh_token` keychain call
    /// times out or errors. Re-armed to `true` after a successful
    /// refresh. Surfaced to the tray via `status.auth.keychain_healthy`
    /// so the user sees "daemon can't reach keychain" instead of a
    /// silent `needs_login`.
    pub keychain_healthy: bool,
    /// Last non-OK outcome from the refresh loop (human-readable).
    /// Surfaced to the tray so the user can self-diagnose.
    pub last_refresh_error: Option<String>,
    /// First time we observed a logged-in state.json while the daemon
    /// itself was `NeedsLogin`. If this persists for more than
    /// `STUCK_THRESHOLD`, the daemon self-terminates and lets launchd
    /// respawn it with fresh in-memory state. `None` when healthy.
    pub stuck_since: Option<std::time::Instant>,
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

/// Remove PID files whose recorded PID is no longer a live process.
/// Targets `daemon.pid`, `tray.pid`, and `tunnel-*.pid` in SOCKET_DIR.
/// Safe to call at startup — the daemon hasn't written its own pidfile
/// yet. Uses `kill -0` semantics (signal 0 = check only, does not kill).
fn sweep_stale_pids(dir: &Path) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        let is_pidfile = name == "daemon.pid"
            || name == "tray.pid"
            || (name.starts_with("tunnel-") && name.ends_with(".pid"));
        if !is_pidfile { continue; }

        let pid = match std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| s.trim().parse::<i32>().ok())
        {
            Some(p) if p > 0 => p,
            _ => {
                // Unparseable PID file — treat as stale.
                let _ = std::fs::remove_file(&path);
                continue;
            }
        };

        // Signal 0: returns 0 iff the process exists AND we have
        // permission to signal it. ESRCH (3) means dead; EPERM (1)
        // means alive (but owned by someone else) — keep those.
        let alive = unsafe { libc::kill(pid, 0) } == 0
            || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM);
        if !alive {
            tracing::info!("sweeping stale pidfile {:?} (pid {} dead)", path.file_name(), pid);
            let _ = std::fs::remove_file(&path);
        }
    }
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

    // Sweep stale daemon + tunnel PID files. A previous daemon PID file
    // whose owning process has exited leaves us unable to detect "is
    // another daemon actually up?" — the self-heal hard-exit path
    // especially relies on clean respawn semantics. Similarly for
    // tunnel PID files: a crashed boringtun process leaves a stale PID
    // that downstream logic (tytus disconnect, tray tunnel_reaches_pod)
    // misinterprets as "tunnel healthy".
    sweep_stale_pids(sock_dir);

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

    let initial_mtime = std::fs::metadata(CliState::state_path())
        .and_then(|m| m.modified())
        .ok();

    let ctx = std::sync::Arc::new(DaemonCtx {
        state: tokio::sync::Mutex::new(DaemonState {
            cli_state: state,
            started_at: std::time::Instant::now(),
            last_refresh: None,
            daemon_status,
            last_state_mtime: initial_mtime,
            keychain_healthy: true,
            last_refresh_error: None,
            stuck_since: None,
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

    // Spawn state.json file watcher. Polls mtime every 500ms and hot-
    // reloads whenever another process (CLI `login`, `connect`, `revoke`)
    // updates the file. Without this the daemon drifts up to 30 min
    // from on-disk truth — the exact bug that shipped the 2026-04-20 tray
    // regression. mtime polling is intentionally chosen over kqueue/
    // fsevents: portable, zero new deps, negligible cost (one stat call
    // per half-second) and survives editors that rename-on-save.
    let watcher_ctx = ctx.clone();
    let watcher_rx = shutdown_rx.clone();
    tokio::spawn(async move {
        state_watcher_loop(watcher_ctx, watcher_rx).await;
    });

    // Spawn self-heal watchdog. Exits the daemon when it has been stuck
    // in NeedsLogin for more than STUCK_THRESHOLD while state.json is
    // plainly logged-in. launchd / systemd respawn us and the fresh
    // process starts from a clean in-memory view. Only path out of
    // corruption scenarios the in-process code can't detect (e.g. a
    // keychain ACL that pending-approval-times-out indefinitely).
    let heal_ctx = ctx.clone();
    let heal_rx = shutdown_rx.clone();
    let heal_shutdown = shutdown_tx.clone();
    tokio::spawn(async move {
        self_heal_loop(heal_ctx, heal_rx, heal_shutdown).await;
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
            let mut ds = ctx.state.lock().await;

            // Hot-reload state.json on every status call. Without this,
            // the daemon's in-memory cache can lag by up to 30 min (the
            // refresh tick cadence) — any other process running `tytus
            // login`, `tytus connect`, or `tytus revoke` updates the
            // file atomically, but the daemon wouldn't see it until
            // next tick. Worse: when the keychain ACL pends (very
            // common on macOS after a rebuild), the refresh tick never
            // succeeds, and the daemon stays pinned in `NeedsLogin`
            // forever — lying to the tray even after the user has
            // successfully re-logged in. This reload is file-only
            // (skips keychain) so it never blocks the status RPC.
            let fresh = CliState::load_file_only();
            if fresh.email.as_deref().is_some_and(|e| !e.is_empty()) {
                // Adopt email/tier/pods/AT unconditionally — the file is
                // always the source of truth for these. Preserve the
                // daemon's in-memory RT (fetched from keychain at
                // startup) because `load_file_only` doesn't touch the
                // keychain and we don't want to drop a good RT.
                let preserved_rt = ds.cli_state.refresh_token.clone();
                ds.cli_state = fresh;
                if ds.cli_state.refresh_token.is_none() {
                    ds.cli_state.refresh_token = preserved_rt;
                }
                // Unstick NeedsLogin when the file now looks healthy.
                // `is_logged_in()` returns true on either a valid AT or
                // a present RT — either is enough to serve the next
                // API call, so don't leave the tray displaying "Sign
                // In…" when we plainly have credentials.
                if ds.cli_state.is_logged_in() && ds.daemon_status == DaemonStatus::NeedsLogin {
                    tracing::info!("Status poll: state.json shows fresh credentials, clearing NeedsLogin");
                    ds.daemon_status = DaemonStatus::Running;
                }
            }

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
                    "last_refresh_error": ds.last_refresh_error,
                    "keychain_healthy": ds.keychain_healthy,
                    "stuck_for_secs": ds.stuck_since.map(|t| t.elapsed().as_secs()),
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
    //
    // Two-stage reload: first file-only (cannot block), then a keychain
    // attempt. If the keychain is still pending-approval (3s timeout), we
    // fall back to file-only credentials and stay Running as long as the
    // AT is valid. Better to serve stale-but-working tokens than to lie
    // to the user while their pod works perfectly. Next tick retries.
    if ds.daemon_status == DaemonStatus::NeedsLogin || !ds.cli_state.is_logged_in() {
        let fresh_file = CliState::load_file_only();
        if fresh_file.has_valid_token() {
            // File has a valid AT — adopt it. Attempt to overlay the RT
            // from keychain (non-fatal if it times out).
            let fresh_full = CliState::load();
            ds.cli_state = if fresh_full.refresh_token.is_some() {
                fresh_full
            } else {
                fresh_file
            };
            ds.daemon_status = DaemonStatus::Running;
        } else {
            let fresh_full = CliState::load();
            if fresh_full.is_logged_in() {
                ds.cli_state = fresh_full;
                ds.daemon_status = DaemonStatus::Running;
            } else {
                ds.daemon_status = DaemonStatus::NeedsLogin;
                return RefreshOutcome::NotLoggedIn;
            }
        }
    }

    // If the reload block above couldn't fetch a refresh token from the
    // keychain (email present but RT still None), the keychain is either
    // pending approval or plain inaccessible. Flag it so the tray can
    // surface an actionable warning instead of silently lying about the
    // user's login state.
    let email_present = ds.cli_state.email.as_ref().is_some_and(|e| !e.is_empty());
    let rt_present = ds.cli_state.refresh_token.as_ref().is_some_and(|t| !t.is_empty());
    if email_present && !rt_present {
        if ds.keychain_healthy {
            tracing::warn!("Keychain refresh token unavailable — marking daemon degraded");
        }
        ds.keychain_healthy = false;
    } else if email_present && rt_present {
        ds.keychain_healthy = true;
    }

    ds.daemon_status = DaemonStatus::Refreshing;
    let result = super::ensure_token(&mut ds.cli_state, &ctx.http).await;

    match result {
        Ok(()) => {
            ds.last_refresh = Some(std::time::Instant::now());
            ds.daemon_status = DaemonStatus::Running;
            ds.last_refresh_error = None;
            ds.keychain_healthy = true;
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
            ds.last_refresh_error = Some("refresh token expired — run `tytus login`".into());
            tracing::warn!("Background refresh: refresh token expired (needs re-login)");
            RefreshOutcome::AuthExpired
        }
        Err(e) => {
            // Do NOT flip to NeedsLogin — user auth is still valid, this is
            // a transient error (network, DNS, Sentinel hiccup). Stay in
            // Running so `tytus status` doesn't lie.
            let msg = format!("transient refresh error: {}", e);
            tracing::warn!("Background token refresh (transient): {}", e);
            ds.last_refresh_error = Some(msg);
            if ds.daemon_status != DaemonStatus::NeedsLogin {
                ds.daemon_status = DaemonStatus::Running;
            }
            RefreshOutcome::Transient
        }
    }
}

// ── File watcher + self-heal ────────────────────────────────

/// Poll state.json mtime every 500ms. On change, reload file-only state
/// and merge into `cli_state`. This keeps the daemon's in-memory view
/// consistent with on-disk truth within ~0.5s of any CLI write, without
/// waiting for the 30-min refresh tick. Never blocks on the keychain
/// (file-only load).
async fn state_watcher_loop(
    ctx: std::sync::Arc<DaemonCtx>,
    mut shutdown: watch::Receiver<bool>,
) {
    let path = CliState::state_path();
    let mut tick = tokio::time::interval(std::time::Duration::from_millis(500));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            _ = shutdown.changed() => {
                if *shutdown.borrow() { return; }
            }
            _ = tick.tick() => {
                let mtime = match std::fs::metadata(&path).and_then(|m| m.modified()) {
                    Ok(t) => t,
                    Err(_) => continue,
                };
                let mut ds = ctx.state.lock().await;
                if ds.last_state_mtime == Some(mtime) {
                    continue;
                }
                let fresh = CliState::load_file_only();
                // Preserve RT — load_file_only doesn't touch keychain.
                let preserved_rt = ds.cli_state.refresh_token.clone();
                let had_valid_creds = ds.cli_state.is_logged_in();
                ds.cli_state = fresh;
                if ds.cli_state.refresh_token.is_none() {
                    ds.cli_state.refresh_token = preserved_rt;
                }
                ds.last_state_mtime = Some(mtime);

                // Clear NeedsLogin the moment the file shows valid credentials.
                if ds.cli_state.is_logged_in() && ds.daemon_status == DaemonStatus::NeedsLogin {
                    tracing::info!("state.json changed — clearing NeedsLogin (file now logged in)");
                    ds.daemon_status = DaemonStatus::Running;
                    ds.last_refresh_error = None;
                }
                if !had_valid_creds && ds.cli_state.is_logged_in() {
                    tracing::info!("state.json watcher: credentials recovered from disk");
                }
            }
        }
    }
}

/// Daemon self-termination threshold. After this long observing a
/// logged-in state.json while pinned to NeedsLogin, we exit so launchd
/// / systemd can respawn us with a fresh in-memory view. Deliberately
/// longer than one full refresh tick + keychain retry window so we
/// don't thrash, but short enough that a user whose keychain is broken
/// doesn't have to `kill` the daemon manually.
const STUCK_THRESHOLD: std::time::Duration = std::time::Duration::from_secs(300);

/// Watchdog that detects "keychain won't come back and the status RPC
/// reload isn't unsticking us" scenarios and exits. Every 30s it
/// compares in-memory daemon_status against on-disk credentials.
async fn self_heal_loop(
    ctx: std::sync::Arc<DaemonCtx>,
    mut shutdown: watch::Receiver<bool>,
    shutdown_tx: watch::Sender<bool>,
) {
    let mut tick = tokio::time::interval(std::time::Duration::from_secs(30));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            _ = shutdown.changed() => {
                if *shutdown.borrow() { return; }
            }
            _ = tick.tick() => {
                let mut ds = ctx.state.lock().await;
                let file_logged_in = CliState::load_file_only()
                    .email.as_ref().is_some_and(|e| !e.is_empty())
                    && CliState::load_file_only().has_valid_token();
                let stuck = ds.daemon_status == DaemonStatus::NeedsLogin && file_logged_in;

                if stuck {
                    let since = *ds.stuck_since.get_or_insert_with(std::time::Instant::now);
                    if since.elapsed() > STUCK_THRESHOLD {
                        tracing::error!(
                            "Daemon stuck in NeedsLogin for {}s while state.json is logged in — \
                             self-terminating so launchd respawns us with fresh state",
                            since.elapsed().as_secs()
                        );
                        drop(ds);
                        let _ = shutdown_tx.send(true);
                        // Give the accept loop a moment to drain, then hard-exit
                        // in case shutdown plumbing is also stuck.
                        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                        std::process::exit(0);
                    }
                } else if ds.stuck_since.is_some() {
                    ds.stuck_since = None;
                }
            }
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

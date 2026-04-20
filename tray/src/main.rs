//! Tytus Tray — system tray icon for managing your private AI pod.
//!
//! Shows a menu bar icon (macOS) / system tray icon (Windows/Linux) with:
//! - Status line with colored dot (🟢 connected / 🔴 disconnected / 🟡 needs login)
//! - Connect / Disconnect
//! - Open in ▸ (launch any installed AI CLI against your pod)
//! - Sign in / Settings / Doctor / About
//! - Daemon controls
//!
//! Single-instance: enforced via a pidfile at /tmp/tytus/tray.pid. Launching
//! a second tray pops focus on the existing one and exits.
//!
//! Talks to tytus-daemon via Unix socket at /tmp/tytus/daemon.sock.

use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::TrayIconBuilder;
use std::sync::{Arc, Mutex};

mod icon;
mod launcher;
mod socket;
mod single_instance;
mod gateway_probe;
mod web_server;

/// Canonical documentation URL. `tytus.traylinx.com` is the Provider API
/// (returns 404 on `/`), not a docs site — point at the public README.
const DOCS_URL: &str = "https://github.com/traylinx/tytus-cli";

// ── Main-thread tray handle ─────────────────────────────────
//
// `tray_icon::TrayIcon` is !Send on macOS (wraps `Rc`). We stash it in a
// thread_local that only the main thread touches; the poll thread uses
// `dispatch_sync` to hop onto the main queue and then reads the cell.
// This keeps every TrayIcon access single-threaded — no unsafe impl Send.
#[cfg(target_os = "macos")]
thread_local! {
    static TRAY_CELL: std::cell::RefCell<Option<tray_icon::TrayIcon>>
        = const { std::cell::RefCell::new(None) };
}

/// Marshal a tray update onto the main thread and apply it synchronously.
/// Called from the poll thread. Blocks briefly while GCD runs the closure
/// on the main queue; deadlock-free because the closure doesn't re-enter
/// the poll thread.
#[cfg(target_os = "macos")]
fn apply_tray_update(dot: HealthDot, tooltip: String) {
    use dispatch2::Queue;
    Queue::main().exec_sync(move || {
        TRAY_CELL.with(|c| {
            if let Some(tray) = c.borrow().as_ref() {
                let _ = tray.set_icon(Some(icon::icon_for(dot)));
                let _ = tray.set_tooltip(Some(&tooltip));
            }
        });
    });
}

// ── Live refresh plumbing ───────────────────────────────────
//
// The poll loop waits on a (Mutex<bool>, Condvar) pair. Anyone can
// wake it by setting the bool true and calling notify_one. We stash
// a clone of the Arc in a thread_local on the main thread so menu-
// event handlers (and any future code) can trigger an immediate
// refresh without threading the pair through every function.

thread_local! {
    static REFRESH_PAIR: std::cell::RefCell<Option<Arc<(std::sync::Mutex<bool>, std::sync::Condvar)>>>
        = const { std::cell::RefCell::new(None) };
}

/// Wake the poll loop RIGHT NOW. Safe to call from any thread via the
/// `trigger_refresh_any_thread` variant; this one only works from the
/// main thread because REFRESH_PAIR is a thread_local. Used by main-
/// thread code that has the NSApp runloop.
#[allow(dead_code)]
fn trigger_refresh_from_main() {
    REFRESH_PAIR.with(|cell| {
        if let Some(pair) = cell.borrow().as_ref() {
            let (lock, cvar) = &**pair;
            *lock.lock().unwrap() = true;
            cvar.notify_one();
        }
    });
}

/// Global-accessible variant: stashes the Arc in a once-initialized
/// static so background threads (menu event handlers, action-refresh
/// timers) can wake the poll loop from any context.
static REFRESH_GLOBAL: std::sync::OnceLock<Arc<(std::sync::Mutex<bool>, std::sync::Condvar)>>
    = std::sync::OnceLock::new();

fn trigger_refresh() {
    if let Some(pair) = REFRESH_GLOBAL.get() {
        let (lock, cvar) = &**pair;
        *lock.lock().unwrap() = true;
        cvar.notify_one();
    }
}

/// After a menu action runs, schedule refreshes at 300 ms, 1 s, 3 s.
/// Catches both snappy actions (stop forwarder is essentially instant)
/// and slow ones (connect needs to spawn Terminal, run `tytus connect`,
/// pop Touch ID, bring tunnel up — often 2–4 s end to end).
fn schedule_refresh_after_action() {
    // Covers the full latency range: stop-forwarder <100ms, connect
    // with sudo+WG handshake can take 10-20s on cold boot. Without
    // the late ticks the menu would be stuck on pre-click state after
    // the slow action finishes (unless the user clicks the icon to
    // force a rebuild). Each tick is a cheap gateway probe (<100ms).
    for delay_ms in [300u64, 1000, 3000, 6000, 10_000, 15_000, 22_000] {
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
            trigger_refresh();
        });
    }
}

/// Cheap fingerprint of the filesystem state — JUST the things that
/// can change without network: tunnel pidfiles, ui-marker files, and
/// state.json's mtime. Used by the 200 ms watcher to wake the main
/// poll loop on external changes.
fn filesystem_signature() -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Ok(rd) = std::fs::read_dir("/tmp/tytus") {
        let mut entries: Vec<String> = rd
            .flatten()
            .filter_map(|e| {
                let n = e.file_name();
                let name = n.to_string_lossy().to_string();
                if !(name.starts_with("tunnel-") || name.starts_with("ui-")) {
                    return None;
                }
                let m = e.metadata().ok()?;
                let size = m.len();
                let mtime = m.modified().ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                Some(format!("{}|{}|{}", name, size, mtime))
            })
            .collect();
        entries.sort();
        parts.extend(entries);
    }
    if let Some(config) = dirs::config_dir() {
        let p = config.join("tytus").join("state.json");
        if let Ok(m) = std::fs::metadata(&p) {
            let mtime = m.modified().ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            parts.push(format!("state|{}|{}", m.len(), mtime));
        }
    }
    parts.join(";")
}

/// Cheap fingerprint of the state — covers everything that changes
/// menu content. If this is unchanged between two poll ticks, the
/// rendered menu would be identical, so we skip the main-thread
/// rebuild and save a few ms of dispatch.
fn menu_signature(s: &TrayState) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("gw={}", s.gateway_reachable));
    parts.push(format!("login={}", s.logged_in));
    parts.push(format!("tier={}", s.tier));
    parts.push(format!("tun={}", s.tunnel_active));
    parts.push(format!("uu={}/{}", s.units_used, s.units_limit));
    parts.push(format!("kc={}", s.keychain_healthy));
    parts.push(format!("err={}", s.last_refresh_error.as_deref().unwrap_or("")));
    for p in &s.pods {
        let fwd_live = existing_ui_forwarder(&p.pod_id).is_some();
        let tun_live = tunnel_reaches_pod(&p.pod_id);
        parts.push(format!("{}:{}:{}:{}", p.pod_id, p.agent_type, fwd_live as u8, tun_live as u8));
    }
    parts.join("|")
}

/// Rebuild the menu from the latest state and install it on the tray.
/// Without this, the menu is frozen at whatever `build_menu` produced at
/// startup — so a user who ran `tytus disconnect` from another shell
/// would still see "Disconnect" and a green dot until restarting the
/// tray. Called alongside `apply_tray_update` on every poll tick.
///
/// Called from any thread; must marshal onto the main thread because
/// TrayIcon / NSMenu are single-threaded on macOS.
#[cfg(target_os = "macos")]
fn apply_menu_rebuild(state: TrayState) {
    use dispatch2::Queue;
    Queue::main().exec_sync(move || {
        TRAY_CELL.with(|c| {
            if let Some(tray) = c.borrow().as_ref() {
                let menu = build_menu(&state);
                let _ = tray.set_menu(Some(Box::new(menu)));
            }
        });
    });
}

// ── State ───────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct TrayState {
    pub daemon_running: bool,
    pub logged_in: bool,
    pub token_valid: bool,
    pub email: String,
    pub tier: String,
    pub tunnel_active: bool,
    pub daemon_pid: u64,
    pub uptime_secs: u64,
    /// Concrete per-pod info (id + which agent is running). Drives the
    /// "Pods & Agents" submenu. Empty when the user has no allocations.
    pub pods: Vec<PodInfo>,
    /// Units currently consumed by allocated pods (NemoClaw=1, Hermes=2).
    /// Derived from `pods` but cached so menu building doesn't recompute.
    pub units_used: u32,
    /// Hard cap from the user's plan. Explorer=1, Creator=2, Operator=4.
    /// 0 means "unknown" (no subscription loaded yet).
    pub units_limit: u32,
    /// True when the launchd/systemd autostart hook for the daemon+tunnel is installed.
    pub autostart_installed: bool,
    /// True when the tray's own launch-at-login agent is installed
    /// (`com.traylinx.tytus.tray` on macOS).
    pub tray_autostart_installed: bool,
    /// True when /Applications/Tytus.app exists (the Spotlight-discoverable bundle).
    pub app_bundle_installed: bool,
    /// **Primary health signal.** True iff a live HTTP request to the stable
    /// dual-bound pod endpoint (10.42.42.1:18080) received an HTTP response
    /// within 2s. Independent of daemon / state.json / login status — this
    /// is the ground truth of "can I call my pod right now?".
    pub gateway_reachable: bool,
    /// False when the daemon can't read the refresh token from the OS
    /// keychain (pending-approval dialog, stale ACL after a rebuild, etc.).
    /// The data plane still works in this state, but the next `tytus
    /// login` is required before the access token expires. Surfaced as
    /// a yellow warning row in the menu.
    pub keychain_healthy: bool,
    /// Human-readable last refresh error the daemon observed. Displayed
    /// verbatim in the troubleshoot menu when present.
    pub last_refresh_error: Option<String>,
}

/// Per-pod info. Agent types beyond the two we ship are silently displayed
/// as their raw id — the unit cost falls back to 1 (the safe default) so
/// we never over-credit the budget.
#[derive(Debug, Clone, Default)]
pub struct PodInfo {
    pub pod_id: String,
    pub agent_type: String,
    pub tunnel_active: bool,
    /// Stable AI gateway URL — same across all pods (10.42.42.1:18080).
    /// Populated from the daemon's status response or state.json.
    pub stable_ai_endpoint: Option<String>,
    /// Per-user stable API key (sk-tytus-user-<32hex>). Survives pod
    /// revocation/reallocation and agent swaps.
    pub stable_user_key: Option<String>,
}

impl PodInfo {
    /// Unit cost — mirrors Scalesys: NemoClaw=1, Hermes=2, Default pod=0.
    pub fn units(&self) -> u32 {
        match self.agent_type.as_str() {
            "hermes" => 2,
            "none" => 0, // agent-less default pod (SPRINT §4.1)
            _ => 1,
        }
    }
    /// Human label for menus. Falls back to the raw id if we don't know it.
    pub fn display_name(&self) -> String {
        // User-facing names. Internal agent_type identifiers (nemoclaw =
        // the NemoClaw safety harness that runs OpenClaw inside) stay as
        // the Docker image + Scalesys enum, but the menu always renders
        // the public brand name ("OpenClaw"). Same for any future harness
        // rename — this is the one place to keep in sync.
        match self.agent_type.as_str() {
            "nemoclaw" => "OpenClaw".into(),
            "hermes" => "Hermes".into(),
            "none" => "Default (AIL only)".into(),
            other if !other.is_empty() => other.to_string(),
            _ => "Unknown".into(),
        }
    }
    /// True if this is the always-on, 0-unit default pod (agent-less,
    /// AIL-gateway-only). Added SPRINT §4.1.
    pub fn is_default(&self) -> bool {
        self.agent_type == "none"
    }
}

/// Map Traylinx plan tier → unit budget. Kept simple; the authoritative
/// limit lives in Scalesys but the user-visible cap is stable enough to
/// mirror here. Unknown tiers fall back to 0 (disables "Add Pod" entries).
fn units_for_tier(tier: &str) -> u32 {
    match tier.to_lowercase().as_str() {
        "explorer" => 1,
        "creator" => 2,
        "operator" => 4,
        _ => 0,
    }
}

/// Check whether the daemon/tunnel autostart hook is on disk.
/// Channels we surface in the tray's per-pod "Add X…" list. Kept in
/// lock-step with `cli/src/channels.rs` REGISTRY — any channel we add
/// there should also get an entry here so users see it in the menu.
/// Tuple: (short-name, human-label).
pub const CHANNEL_MENU_ENTRIES: &[(&str, &str)] = &[
    ("telegram", "Telegram"),
    ("discord", "Discord"),
    ("slack", "Slack (Socket Mode)"),
    ("line", "LINE"),
];

/// Human-friendly label for a channel short-name. Falls back to the
/// short-name capitalized so a channel added to the CLI registry but
/// not yet in `CHANNEL_MENU_ENTRIES` still renders readably.
pub fn channel_label(short: &str) -> String {
    CHANNEL_MENU_ENTRIES
        .iter()
        .find(|(n, _)| *n == short)
        .map(|(_, l)| l.to_string())
        .unwrap_or_else(|| {
            let mut chars = short.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
}

/// Read `~/.tytus/channels.json` (the CLI's local manifest written on
/// every `tytus channels add/remove`) and return `(channel_name,
/// credential_count)` pairs for a given pod. No network, no keychain —
/// just a small JSON parse. Returns an empty vec if the file is
/// missing or unparseable, so the menu degrades gracefully to "No
/// channels configured".
pub fn read_channels_for_pod(pod_id: &str) -> Vec<(String, usize)> {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return Vec::new(),
    };
    let path = std::path::PathBuf::from(&home).join(".tytus").join("channels.json");
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let v: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let pods = match v.get("pods").and_then(|x| x.as_object()) {
        Some(p) => p,
        None => return Vec::new(),
    };
    let pod = match pods.get(pod_id).and_then(|x| x.as_object()) {
        Some(p) => p,
        None => return Vec::new(),
    };
    pod.iter()
        .map(|(name, entry)| {
            let count = entry
                .get("env_vars")
                .and_then(|x| x.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            (name.clone(), count)
        })
        .collect()
}

fn check_autostart_installed() -> bool {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        std::path::PathBuf::from(&home)
            .join("Library/LaunchAgents/com.traylinx.tytus.plist")
            .exists()
    }
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        std::path::PathBuf::from(&home)
            .join(".config/systemd/user/tytus.service")
            .exists()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    { false }
}

/// Check whether the tray launch-at-login LaunchAgent is installed.
fn check_tray_autostart_installed() -> bool {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        std::path::PathBuf::from(&home)
            .join("Library/LaunchAgents/com.traylinx.tytus.tray.plist")
            .exists()
    }
    #[cfg(not(target_os = "macos"))]
    { false }
}

/// Check whether /Applications/Tytus.app exists.
fn check_app_bundle_installed() -> bool {
    #[cfg(target_os = "macos")]
    { std::path::Path::new("/Applications/Tytus.app").exists() }
    #[cfg(not(target_os = "macos"))]
    { false }
}

/// Coarse health state that drives the colored dot in the menu title.
///
/// **Data-plane first.** The health assessment is primarily driven by
/// whether the user can actually call their pod (`gateway_reachable`),
/// not by daemon uptime. A running daemon with an expired token and a
/// dead tunnel is RED; a stopped daemon with a live tunnel the user is
/// actively using is GREEN (with YELLOW escalation if auth degradation
/// is imminent).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthDot {
    /// Green: pod is reachable AND session state is healthy.
    Connected,
    /// Yellow: pod is reachable but some secondary concern will bite us
    /// (daemon down → no auto-refresh; token expired → may bite soon;
    /// not logged in but tunnel is up). Also used when tunnel is down
    /// but credentials are valid (reconnect is one click away).
    Warning,
    /// Red: pod is NOT reachable AND there's no easy path back
    /// (no credentials, or nothing running).
    Down,
}

impl HealthDot {
    fn emoji(self) -> &'static str {
        match self {
            HealthDot::Connected => "🟢",
            HealthDot::Warning => "🟡",
            HealthDot::Down => "🔴",
        }
    }

    /// Map observable state → coarse colour.
    ///
    /// **Gateway reachability is the only thing that matters for colour.**
    /// Sentinel auth (the Traylinx account token) only gates *Provider
    /// admin* operations — allocating new pods, revoking, etc. It has
    /// nothing to do with your ability to call the pod: LLM inference
    /// uses the WireGuard tunnel's per-pod keys plus the per-user stable
    /// gateway key, both of which persist independently of Sentinel
    /// session lifetime.
    ///
    /// An earlier revision of this function would go YELLOW when the
    /// daemon was offline or the access token had expired, even while
    /// `curl` to the pod worked perfectly. That was wrong — users saw a
    /// warning for a non-problem. Keep auth/daemon warnings to the
    /// secondary metadata row (they'll bite on the *next* admin op, not
    /// during daily use).
    fn from_state(s: &TrayState) -> Self {
        if s.gateway_reachable {
            // Escalate to yellow if the daemon can't reach the keychain.
            // The data plane works now, but we can't auto-refresh the
            // access token — if the user ignores this until AT expiry,
            // they'll lose API-plane access until they `tytus login`.
            // Not severe enough for red: the tunnel and LLM calls are
            // fully functional right this second.
            if !s.keychain_healthy { HealthDot::Warning } else { HealthDot::Connected }
        } else if s.logged_in {
            // Tunnel down but credentials are fine — click Connect.
            HealthDot::Warning
        } else {
            HealthDot::Down
        }
    }
}

// ── Main ────────────────────────────────────────────────────

fn main() {
    // Single-instance guard. If another tytus-tray is already running, exit
    // cleanly — don't create a duplicate menu bar icon. The existing instance
    // keeps handling clicks; the user sees exactly one T in the menu bar.
    if let Err(e) = single_instance::acquire() {
        eprintln!("tytus-tray: {}", e);
        std::process::exit(0);
    }

    // macOS: must set activation policy BEFORE creating any UI elements
    #[cfg(target_os = "macos")]
    {
        use objc2::MainThreadMarker;
        use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
        let mtm = MainThreadMarker::new().expect("must be called from main thread");
        let app = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    }

    let state = Arc::new(Mutex::new(TrayState::default()));

    // Spin up the localhost install wizard server on a random port. This
    // is the entry point for the browser-based "Install Agent…" flow.
    // Failure to bind is non-fatal: the menu action falls back to opening
    // a Terminal with `tytus agent install` if the port file is missing.
    // Tray-side integration for SPRINT §6 E1.
    if let Some(port) = web_server::start() {
        eprintln!("[tray] install wizard ready on http://127.0.0.1:{}/install", port);
    } else {
        eprintln!("[tray] install wizard not available (bind failed)");
    }

    // Initial poll
    {
        let new_state = socket::poll_daemon_status();
        *state.lock().unwrap() = new_state;
    }

    // Build menu + tray on the main thread (TrayIcon is !Send on macOS).
    let initial_dot = HealthDot::from_state(&state.lock().unwrap());
    let menu = build_menu(&state.lock().unwrap());
    let tray_icon = icon::icon_for(initial_dot);
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(tooltip_for(&state.lock().unwrap()))
        .with_icon(tray_icon)
        .build()
        .expect("Failed to create tray icon");

    // Stash the tray in a main-thread-local so `dispatch_sync` blocks from
    // the poll thread can read it without crossing a Send boundary.
    // Accessing TRAY_CELL from any other thread would panic on `with` (the
    // cell simply wouldn't exist there); we only `dispatch_sync` onto main
    // before touching it, which is the entire point of this dance.
    #[cfg(target_os = "macos")]
    TRAY_CELL.with(|c| *c.borrow_mut() = Some(tray));
    #[cfg(not(target_os = "macos"))]
    let _tray = tray;

    // Live refresh architecture — three threads, one shared shoot-now
    // Condvar.
    //
    // 1) Fast FS watcher (200 ms): reads filesystem-only signatures
    //    (pidfiles, marker files, state.json mtime). No network. On
    //    change, wakes the main poll loop immediately. Covers external
    //    events: another shell runs `tytus disconnect`, forwarder self-
    //    terminates on upstream loss, agent restart writes new markers.
    //
    // 2) Main poll loop: blocks on a Condvar with a 1.5 s timeout.
    //    Wakes on either the timer OR a REFRESH signal (from FS
    //    watcher, menu action, or external code). Does a full poll
    //    including the gateway probe, rebuilds the menu if signature
    //    changed, updates the icon.
    //
    // 3) Menu-action handler (already exists): each click now calls
    //    `schedule_refresh_after_action` which fires REFRESH signals
    //    at 300 ms, 1 s, 3 s to catch the action's async completion
    //    regardless of how long Terminal takes to spawn.
    //
    // Zero noticeable lag when the user clicks anything in the tray,
    // and ~200 ms detection of external state changes.
    let refresh_pair = Arc::new((std::sync::Mutex::new(false), std::sync::Condvar::new()));

    // Install globals so background threads (menu event handlers,
    // action-timer threads, FS watcher) can wake the poll loop from
    // any context without having to thread the Arc through every
    // function signature.
    let _ = REFRESH_GLOBAL.set(refresh_pair.clone());
    REFRESH_PAIR.with(|cell| *cell.borrow_mut() = Some(refresh_pair.clone()));

    // Fast FS watcher.
    let fs_pair = refresh_pair.clone();
    std::thread::spawn(move || {
        let mut last_fs_sig = String::new();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(200));
            let fs_sig = filesystem_signature();
            if fs_sig != last_fs_sig {
                last_fs_sig = fs_sig;
                let (lock, cvar) = &*fs_pair;
                *lock.lock().unwrap() = true;
                cvar.notify_one();
            }
        }
    });

    // Main poll loop.
    let poll_state = state.clone();
    let poll_pair = refresh_pair.clone();
    std::thread::spawn(move || {
        let mut last_sig: Option<(HealthDot, String)> = None;
        loop {
            // Wait up to 1.5 s or until someone signals REFRESH.
            let (lock, cvar) = &*poll_pair;
            {
                let guard = lock.lock().unwrap();
                let (mut guard, _) = cvar.wait_timeout(guard, std::time::Duration::from_millis(1500)).unwrap();
                *guard = false;
            }

            let new_state = socket::poll_daemon_status();
            let new_dot = HealthDot::from_state(&new_state);
            let new_tooltip = tooltip_for(&new_state);
            let new_sig = menu_signature(&new_state);
            *poll_state.lock().unwrap() = new_state.clone();

            let sig_key = (new_dot, new_sig);
            if last_sig.as_ref() == Some(&sig_key) {
                continue;
            }
            last_sig = Some(sig_key);

            #[cfg(target_os = "macos")]
            {
                apply_tray_update(new_dot, new_tooltip);
                apply_menu_rebuild(new_state);
            }
            #[cfg(not(target_os = "macos"))]
            let _ = new_tooltip;
        }
    });

    // Handle menu events in a background thread
    let event_state = state.clone();
    std::thread::spawn(move || {
        loop {
            if let Ok(event) = MenuEvent::receiver().recv() {
                handle_menu_event(event.id().0.as_str(), &event_state);
            }
        }
    });

    // Release the pidfile on clean exit.
    // We don't install a signal handler — on SIGTERM/SIGKILL the stale pidfile
    // is swept by single_instance::acquire on the next launch (PID-alive check).
    let _cleanup = scopeguard_lite::OnDrop::new(single_instance::release);

    // Run platform event loop (blocks forever)
    #[cfg(target_os = "macos")]
    {
        use objc2::MainThreadMarker;
        use objc2_app_kit::NSApplication;
        let mtm = MainThreadMarker::new().unwrap();
        let app = NSApplication::sharedApplication(mtm);
        app.run();
    }

    #[cfg(not(target_os = "macos"))]
    {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

// ── Menu construction ───────────────────────────────────────

/// Build the menu Docker-Desktop-style — every CLI action a non-dev user needs
/// is reachable without touching a terminal.
///
///   🟢 Connected (email)
///   Plan: Operator · 1 pod · up 2h
///   ──
///   Disconnect | Connect | Sign In…
///   Open in ▸
///   Copy Connection Info
///   Run Health Test
///   ──
///   Settings ▸
///     Configure Agent…
///     Auto-start at Login  [✓ / ·]
///     Sign Out
///   Troubleshoot ▸
///     Doctor
///     View Daemon Log
///     View Startup Log
///     Restart Daemon
///     Stop Daemon | Start Daemon
///   ──
///   About Tytus
///   Documentation
///   ──
///   Quit Tytus
fn build_menu(state: &TrayState) -> Menu {
    let menu = Menu::new();

    // ── Status line ────────────────────────────────────────
    // Primary line answers the data-plane question: can the user call
    // their pod right now? Secondary concerns (daemon down, Sentinel
    // token drift) go into the metadata row below as hints, not in the
    // dot colour.
    let dot = HealthDot::from_state(state);
    let who = if !state.email.is_empty() {
        format!(" ({})", state.email)
    } else {
        String::new()
    };
    let status_text = if state.gateway_reachable {
        format!("{} Connected{}", dot.emoji(), who)
    } else if !state.logged_in {
        format!("{} Not logged in", dot.emoji())
    } else if state.pods.is_empty() {
        format!("{} No pods allocated — click Connect", dot.emoji())
    } else {
        format!("{} Pod unreachable — click Connect{}", dot.emoji(), who)
    };
    let _ = menu.append(&MenuItem::with_id("status", &status_text, false, None));

    // ── Metadata line (Plan · pods · uptime, plus soft warnings) ─────
    // Soft warnings cover conditions that WILL eventually bite the user
    // (next `tytus connect`, RT server-side expiry, etc.) but aren't
    // affecting anything they can do right now. Prefixing with ⚠︎ keeps
    // the signal visible without hijacking the dot.
    {
        let mut bits: Vec<String> = Vec::new();
        if !state.tier.is_empty() {
            bits.push(format!("Plan: {}", state.tier));
        }
        if !state.pods.is_empty() {
            let n = state.pods.len();
            // Show each pod by its agent, e.g. "1 pod (NemoClaw)" — so the
            // user instantly knows what's deployed, not just how many.
            if n == 1 {
                bits.push(format!("1 pod ({})", state.pods[0].display_name()));
            } else {
                bits.push(format!("{} pods", n));
            }
        }
        if state.daemon_running && state.uptime_secs > 0 {
            bits.push(format!("up {}", format_uptime(state.uptime_secs)));
        }
        // Soft warnings — gateway works but something admin-y is stale.
        if state.gateway_reachable {
            if !state.daemon_running {
                bits.push("⚠︎ daemon offline".into());
            } else if !state.keychain_healthy {
                // Daemon is alive but the macOS keychain hasn't yielded
                // the refresh token. Data plane works; next `tytus login`
                // may be needed before the current AT expires. User-
                // actionable via the Troubleshoot menu.
                bits.push("⚠︎ keychain access pending — re-run `tytus login`".into());
            } else if state.logged_in && !state.token_valid {
                bits.push("⚠︎ token expiring — will auto-refresh".into());
            }
        }
        if !bits.is_empty() {
            let _ = menu.append(&MenuItem::with_id("meta", bits.join(" · "), false, None));
        }
    }

    let _ = menu.append(&PredefinedMenuItem::separator());

    // ── Primary action + tunnel utilities ─────────────────
    //
    // Connect / Disconnect must be independent of daemon_running. The
    // daemon is a background token-refresh process; the WG tunnel lives
    // in its own root process spawned by `tytus connect`. A dead daemon
    // means token refresh is stale, not that the tunnel is broken — and
    // conversely a user with an offline daemon still needs a path to
    // bring the tunnel up. Gating Connect on daemon_running is the bug
    // that made the previous tray tell users "click Connect" without
    // rendering the button (screenshot 2026-04-18).
    if !state.logged_in {
        let _ = menu.append(&MenuItem::with_id("login", "Sign In…", true, None));
        let _ = menu.append(&PredefinedMenuItem::separator());
    } else if state.tunnel_active {
        let _ = menu.append(&MenuItem::with_id("disconnect", "Disconnect", true, None));

        // "Open in ▸" submenu — only when tunnel is active
        let clis = launcher::detect_installed_clis();
        let open_sub = Submenu::new("Open in", true);
        for cli in &clis {
            let id = format!("launch_{}", cli.binary);
            let _ = open_sub.append(&MenuItem::with_id(&id, cli.name, true, None));
        }
        if !clis.is_empty() {
            let _ = open_sub.append(&PredefinedMenuItem::separator());
        }
        let _ = open_sub.append(&MenuItem::with_id("launch_terminal", "Terminal", true, None));
        let _ = menu.append(&open_sub);

        let _ = menu.append(&MenuItem::with_id("test", "Run Health Test", true, None));

        let _ = menu.append(&PredefinedMenuItem::separator());
    } else {
        // Logged in but tunnel not active — show Connect, regardless of
        // whether the daemon is running.
        let _ = menu.append(&MenuItem::with_id("connect", "Connect", true, None));
        let _ = menu.append(&PredefinedMenuItem::separator());
    }

    // ── AIL Connection Info ▸ ─────────────────────────────────
    //
    // Surfaces the stable endpoint + key so users can paste them into
    // Claude Code, Cursor, OpenCode, Codex — any OpenAI-compatible tool
    // — without running a terminal round-trip. Visible whenever the user
    // is logged in and has at least one pod (doesn't require the tunnel
    // to be up: users often pre-configure clients).
    //
    // Pulls values from the first pod with stable fields (all pods on a
    // user share the same stable endpoint + key, so the first one is
    // representative). Falls back to the canonical 10.42.42.1 URL if the
    // daemon hasn't yet populated the state.
    if state.logged_in {
        let primary = state.pods.iter().find(|p| p.stable_user_key.is_some());
        let endpoint = primary
            .and_then(|p| p.stable_ai_endpoint.clone())
            .unwrap_or_else(|| "http://10.42.42.1:18080".to_string());
        let key = primary.and_then(|p| p.stable_user_key.clone());

        let info_sub = Submenu::new("AIL Connection Info", true);

        // Display-only rows so the user can see what they'd copy.
        let _ = info_sub.append(&MenuItem::with_id(
            "ail_info_url",
            format!("URL: {}/v1", endpoint),
            false, None,
        ));
        if let Some(ref k) = key {
            // Preview first 14 chars of the key so the user can recognize
            // it without exposing the whole token in a screenshot.
            let preview = if k.len() > 18 {
                format!("{}…{}", &k[..14], &k[k.len() - 4..])
            } else {
                k.clone()
            };
            let _ = info_sub.append(&MenuItem::with_id(
                "ail_info_key",
                format!("Key: {}", preview),
                false, None,
            ));
        } else {
            let _ = info_sub.append(&MenuItem::with_id(
                "ail_info_key",
                "Key: (none yet — run `tytus login`)",
                false, None,
            ));
        }

        let _ = info_sub.append(&PredefinedMenuItem::separator());

        let has_key = key.is_some();
        let _ = info_sub.append(&MenuItem::with_id("copy_ail_url", "Copy AIL_URL", true, None));
        let _ = info_sub.append(&MenuItem::with_id("copy_ail_key", "Copy AIL_API_KEY", has_key, None));
        let _ = info_sub.append(&MenuItem::with_id("copy_ail_exports", "Copy export block (all aliases)", has_key, None));
        let _ = info_sub.append(&MenuItem::with_id("copy_openai_block", "Copy as OpenAI exports", has_key, None));
        let _ = info_sub.append(&MenuItem::with_id("copy_anthropic_block", "Copy as Anthropic exports", has_key, None));
        let _ = info_sub.append(&MenuItem::with_id("copy_ail_json", "Copy JSON ({url, api_key})", has_key, None));

        let _ = info_sub.append(&PredefinedMenuItem::separator());
        let _ = info_sub.append(&MenuItem::with_id("open_mcp_guide", "Paste into Claude Code / Cursor / OpenCode…", true, None));

        let _ = menu.append(&info_sub);
        let _ = menu.append(&PredefinedMenuItem::separator());
    }

    // ── Pods & Agents ▸ ───────────────────────────────────
    // Only visible once the user has credentials — the actions all hit
    // Provider, which needs an active Sentinel session.
    if state.logged_in {
        let pods_sub = Submenu::new("Pods & Agents", true);
        if state.pods.is_empty() {
            let _ = pods_sub.append(&MenuItem::with_id("no_pods", "No pods allocated", false, None));
            let _ = pods_sub.append(&PredefinedMenuItem::separator());
        } else {
            // Surface the default pod (agent-less, 0 units) on its own row
            // as informational — no actions. It's universal, costs
            // nothing, and auto-reprovisions on every `tytus login`, so
            // there's nothing a user ever gains from "revoking" it (the
            // next login would just allocate another one, churning the
            // slot and the stable key map without freeing units). Power
            // users who genuinely want to release the droplet slot can
            // still `tytus revoke <pod_id>` from the CLI. Per §4.1 +
            // user feedback 2026-04-19.
            for p in state.pods.iter().filter(|p| p.is_default()) {
                let header = format!(
                    "Default Pod {} — AIL only  (0 units)",
                    p.pod_id,
                );
                let _ = pods_sub.append(&MenuItem::with_id(
                    format!("pod_header_{}", p.pod_id),
                    &header,
                    false,
                    None,
                ));
                let _ = pods_sub.append(&PredefinedMenuItem::separator());
            }

            for p in state.pods.iter().filter(|p| !p.is_default()) {
                let header = format!("Pod {} — {}  ({} unit{})",
                    p.pod_id,
                    p.display_name(),
                    p.units(),
                    if p.units() == 1 { "" } else { "s" },
                );
                let _ = pods_sub.append(&MenuItem::with_id(
                    format!("pod_header_{}", p.pod_id),
                    &header, false, None,
                ));
                // Open the agent's web UI via tytus ui (localhost forwarder
                // + browser launch). Only meaningful for agents that
                // actually expose a web UI on the pod — currently both
                // OpenClaw (port 3000) and Hermes (port 8642). The tunnel
                // must be up; the handler prints a clear message if it
                // isn't. Per user request 2026-04-19 "we need to be able
                // to reach always the tytus pod openclaw or hermes agent
                // via the browser".
                let forwarder_live = existing_ui_forwarder(&p.pod_id).is_some();
                let tunnel_live = tunnel_reaches_pod(&p.pod_id);
                // B1: label reflects ground truth so the user knows
                // whether a click will (a) just open a tab, (b) start a
                // forwarder, or (c) also swap the tunnel (Touch ID prompt
                // possible). Three-state rendering keeps the menu honest.
                let open_label = match (forwarder_live, tunnel_live) {
                    (true, _)      => "  Open in Browser  ✓",
                    (false, true)  => "  Open in Browser",
                    (false, false) => "  Connect & Open in Browser",
                };
                let _ = pods_sub.append(&MenuItem::with_id(
                    format!("pod_{}_open", p.pod_id),
                    open_label, true, None,
                ));
                if forwarder_live {
                    let _ = pods_sub.append(&MenuItem::with_id(
                        format!("pod_{}_stop_forwarder", p.pod_id),
                        "  Stop Forwarder", true, None,
                    ));
                }
                let _ = pods_sub.append(&MenuItem::with_id(
                    format!("pod_{}_restart", p.pod_id),
                    "  Restart Agent", true, None,
                ));
                // Uninstall Agent: keeps the pod slot allocated (AIL still
                // works through it), drops the container. SPRINT §4.3.
                //
                // There is intentionally NO "Replace with X" action here.
                // The mental model is add + delete only — if a user wants
                // to change the agent on a pod, they revoke the pod and
                // install a fresh one with the new type. Prevents the
                // subtle trap of slot-preserving "replace" looking like a
                // safe in-place swap while still destroying container
                // workspace state. (Decision: 2026-04-18, post-sprint UX.)
                let _ = pods_sub.append(&MenuItem::with_id(
                    format!("pod_{}_uninstall", p.pod_id),
                    "  Uninstall Agent  (keeps pod)", true, None,
                ));

                // Channels submenu — the chat channels this pod's agent
                // uses to talk to the owner (Telegram, Discord, Slack,
                // …). Each click opens a Terminal window running the
                // `tytus channels …` CLI subcommand, so the UX never
                // requires the user to drop into a terminal themselves.
                //
                // Source of truth for "what's configured" is the local
                // manifest at `~/.tytus/channels.json` (the CLI writes
                // it whenever `tytus channels add/remove` succeeds).
                // The tray reads it cheaply on each menu rebuild — no
                // network, no daemon call, always fresh.
                let channel_sub = Submenu::new("  Channels", true);
                let configured = read_channels_for_pod(&p.pod_id);
                if configured.is_empty() {
                    let _ = channel_sub.append(&MenuItem::with_id(
                        format!("pod_{}_channels_empty", p.pod_id),
                        "No channels configured",
                        false, None,
                    ));
                    let _ = channel_sub.append(&PredefinedMenuItem::separator());
                } else {
                    for (name, cred_count) in &configured {
                        let _ = channel_sub.append(&MenuItem::with_id(
                            format!("pod_{}_channel_{}_info", p.pod_id, name),
                            format!(
                                "{}  ✓  ({} secret{})",
                                channel_label(name),
                                cred_count,
                                if *cred_count == 1 { "" } else { "s" },
                            ),
                            false, None,
                        ));
                    }
                    let _ = channel_sub.append(&PredefinedMenuItem::separator());
                    // Per-channel removal — one terminal shortcut each so
                    // users don't have to remember the channel name's
                    // exact spelling.
                    for (name, _) in &configured {
                        let _ = channel_sub.append(&MenuItem::with_id(
                            format!("pod_{}_channel_{}_remove", p.pod_id, name),
                            format!("Remove {}", channel_label(name)),
                            true, None,
                        ));
                    }
                    let _ = channel_sub.append(&PredefinedMenuItem::separator());
                }
                // Catalog + add entries always visible so the user can
                // always find the "add" affordance even when the pod has
                // no channels yet.
                let _ = channel_sub.append(&MenuItem::with_id(
                    format!("pod_{}_channels_catalog", p.pod_id),
                    "Browse available channels…",
                    true, None,
                ));
                for known in CHANNEL_MENU_ENTRIES {
                    // Skip channels that are already configured so the
                    // list doesn't duplicate them as "Add X".
                    if configured.iter().any(|(n, _)| n == known.0) { continue; }
                    let _ = channel_sub.append(&MenuItem::with_id(
                        format!("pod_{}_channel_{}_add", p.pod_id, known.0),
                        format!("Add {}…", known.1),
                        true, None,
                    ));
                }
                let _ = pods_sub.append(&channel_sub);

                let _ = pods_sub.append(&MenuItem::with_id(
                    format!("pod_{}_revoke", p.pod_id),
                    "  Revoke Pod", true, None,
                ));
                let _ = pods_sub.append(&PredefinedMenuItem::separator());
            }
        }

        // Install Agent — opens the browser wizard (SPRINT §6 E). The
        // single entry point covers the whole catalog and renders agent
        // cards dynamically, so we don't need to re-list agent types in
        // the menu itself. Legacy terminal-picker entries stay below as
        // quick shortcuts + fallback if the localhost server didn't bind.
        let _ = pods_sub.append(&MenuItem::with_id(
            "install_agent", "Install Agent…", true, None,
        ));
        let add_sub = Submenu::new("Install Agent (terminal)", true);
        let remaining = state.units_limit.saturating_sub(state.units_used);
        let nemo_ok = state.units_limit == 0 || remaining >= 1;
        let hermes_ok = state.units_limit == 0 || remaining >= 2;
        let _ = add_sub.append(&MenuItem::with_id(
            "install_agent_nemoclaw",
            format!("OpenClaw  (1 unit){}", if nemo_ok { "" } else { "  — not enough units" }),
            nemo_ok, None,
        ));
        let _ = add_sub.append(&MenuItem::with_id(
            "install_agent_hermes",
            format!("Hermes  (2 units){}", if hermes_ok { "" } else { "  — not enough units" }),
            hermes_ok, None,
        ));
        let _ = pods_sub.append(&add_sub);

        // Budget line at the bottom for context.
        if state.units_limit > 0 {
            let _ = pods_sub.append(&PredefinedMenuItem::separator());
            let _ = pods_sub.append(&MenuItem::with_id(
                "units_line",
                format!("Units: {} / {} used", state.units_used, state.units_limit),
                false, None,
            ));
        }

        let _ = menu.append(&pods_sub);
        let _ = menu.append(&PredefinedMenuItem::separator());
    }

    // ── Settings ▸ ────────────────────────────────────────
    let settings_sub = Submenu::new("Settings", true);
    if state.daemon_running && state.logged_in {
        let _ = settings_sub.append(&MenuItem::with_id("settings_configure", "Configure Agent…", true, None));
    }
    let autostart_label = if state.autostart_installed {
        "Start Tunnel at Login  ✓"
    } else {
        "Start Tunnel at Login"
    };
    let _ = settings_sub.append(&MenuItem::with_id("autostart_toggle", autostart_label, true, None));
    let tray_autostart_label = if state.tray_autostart_installed {
        "Launch Tray at Login  ✓"
    } else {
        "Launch Tray at Login"
    };
    let _ = settings_sub.append(&MenuItem::with_id("tray_autostart_toggle", tray_autostart_label, true, None));
    // Only surface the bundle installer when it's actually missing —
    // reinstalling over a good bundle is harmless but noisy.
    if !state.app_bundle_installed {
        let _ = settings_sub.append(&MenuItem::with_id("install_app", "Install Tytus in Applications…", true, None));
    }
    if state.logged_in {
        let _ = settings_sub.append(&PredefinedMenuItem::separator());
        let _ = settings_sub.append(&MenuItem::with_id("logout", "Sign Out", true, None));
    }
    let _ = menu.append(&settings_sub);

    // ── Troubleshoot ▸ ────────────────────────────────────
    let trouble_sub = Submenu::new("Troubleshoot", true);
    // Surface the daemon's most recent diagnostic so the user doesn't
    // have to `tail /tmp/tytus/daemon.log` to know what's wrong. Two
    // disabled info rows render as context above the actionable items.
    if !state.keychain_healthy {
        let _ = trouble_sub.append(&MenuItem::with_id(
            "diag_keychain",
            "⚠︎ Keychain access pending — approve dialog or re-run Sign In",
            false, None,
        ));
    }
    if let Some(ref err) = state.last_refresh_error {
        let truncated = if err.len() > 80 { format!("{}…", &err[..80]) } else { err.clone() };
        let _ = trouble_sub.append(&MenuItem::with_id(
            "diag_refresh_err",
            format!("Last refresh error: {}", truncated),
            false, None,
        ));
    }
    if !state.keychain_healthy || state.last_refresh_error.is_some() {
        let _ = trouble_sub.append(&PredefinedMenuItem::separator());
    }
    let _ = trouble_sub.append(&MenuItem::with_id("doctor", "Doctor", true, None));
    let _ = trouble_sub.append(&MenuItem::with_id("view_daemon_log", "View Daemon Log", true, None));
    let _ = trouble_sub.append(&MenuItem::with_id("view_startup_log", "View Startup Log", true, None));
    let _ = trouble_sub.append(&PredefinedMenuItem::separator());
    if state.daemon_running {
        let _ = trouble_sub.append(&MenuItem::with_id("daemon_restart", "Restart Daemon", true, None));
        let _ = trouble_sub.append(&MenuItem::with_id("daemon_stop", "Stop Daemon", true, None));
    } else {
        let _ = trouble_sub.append(&MenuItem::with_id("daemon_start", "Start Daemon", true, None));
    }
    let _ = menu.append(&trouble_sub);

    let _ = menu.append(&PredefinedMenuItem::separator());

    // ── About / Docs ──────────────────────────────────────
    let _ = menu.append(&MenuItem::with_id("docs", "Documentation", true, None));
    let _ = menu.append(&MenuItem::with_id("about", "About Tytus", true, None));

    let _ = menu.append(&PredefinedMenuItem::separator());
    // Label hints at how to come back: if the bundle + launch-at-login are
    // installed, "quit" is truly per-session (Spotlight or reboot brings it
    // back). Otherwise we tell the user they'll need to relaunch manually.
    let quit_label = if state.app_bundle_installed {
        "Quit Tytus"
    } else {
        "Quit Tytus (no autostart — see Settings)"
    };
    let _ = menu.append(&MenuItem::with_id("quit", quit_label, true, None));

    menu
}

fn tooltip_for(state: &TrayState) -> String {
    match HealthDot::from_state(state) {
        HealthDot::Connected => format!("Tytus — Connected ({})", state.email),
        HealthDot::Warning if !state.logged_in => "Tytus — Not logged in".into(),
        HealthDot::Warning => "Tytus — Needs attention".into(),
        HealthDot::Down => "Tytus — Daemon not running".into(),
    }
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    if days > 0 { format!("{}d {}h", days, hours) }
    else if hours > 0 { format!("{}h {}m", hours, mins) }
    else { format!("{}m", mins) }
}

// ── Menu event handler ──────────────────────────────────────

fn handle_menu_event(id: &str, state: &Arc<Mutex<TrayState>>) {
    // Every menu action gets followed by a refresh fan-out (300 ms /
    // 1 s / 3 s). That catches the action's actual completion across
    // the full latency range: stop-forwarder completes in <100 ms,
    // connect/disconnect typically 1–3 s once Terminal spawns and sudo
    // prompts resolve. Without this the user would see the menu
    // frozen on pre-click state for up to 1.5 s (next poll tick).
    schedule_refresh_after_action();
    match id {
        "connect" => {
            // Ollama-style: silent background run + system notifications.
            // No Terminal window, no "Press Enter to close" step. The
            // sudo elevation prompt is handled inline by `tytus connect`
            // via osascript when needed — that's a native macOS prompt
            // that surfaces on its own without needing a Terminal.
            //
            // A watcher thread polls the gateway probe every 2s for up
            // to 45s and fires a system notification with the outcome.
            // The user sees: "Connecting…" immediately → "Connected to
            // pod 01" OR "Connection failed — open Troubleshoot" after.
            notify("Tytus", "Connecting…");
            run_silent_with_notify(
                "tytus",
                &["connect"],
                "Connect",
                |was_reachable_before| {
                    // Outcome = "gateway is now reachable" (primary)
                    // with a fallback of "tunnel pidfile appeared" for
                    // agent-less default pods where the health probe
                    // path differs.
                    use gateway_probe::probe_gateway;
                    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(45);
                    while std::time::Instant::now() < deadline {
                        if probe_gateway() { return Ok("Connected. Tunnel is up.".to_string()); }
                        std::thread::sleep(std::time::Duration::from_secs(2));
                    }
                    if was_reachable_before {
                        Err("Connect didn't change state — tunnel may already have been active.".into())
                    } else {
                        Err("Couldn't verify connection within 45s. Open Troubleshoot ▸ Doctor.".into())
                    }
                },
            );
        }
        "disconnect" => {
            // Silent background run — disconnect is fast (<1s typical)
            // so we don't even bother with "Disconnecting…" prefix.
            notify("Tytus", "Disconnecting…");
            run_silent_with_notify(
                "tytus",
                &["disconnect"],
                "Disconnect",
                |_was_reachable_before| {
                    use gateway_probe::probe_gateway;
                    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
                    while std::time::Instant::now() < deadline {
                        if !probe_gateway() { return Ok("Disconnected. Tunnel is down.".to_string()); }
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                    Err("Disconnect command ran but tunnel still responds. Check `tytus status`.".into())
                },
            );
        }
        "login" => {
            // `tytus login` opens a browser — must run in terminal so the user
            // sees the verification code & prompts. Pipe through the Terminal.app
            // launcher the same way launch_terminal does.
            open_in_terminal_simple("tytus login");
        }
        "logout" => {
            // Destructive: revokes all pods and clears keychain. Confirm first.
            if confirm_dialog(
                "Sign out of Tytus?",
                "This revokes all your pods, clears stored credentials, and tears down any active tunnels. You'll need to sign in again to reconnect.",
            ) {
                open_in_terminal_simple("tytus logout; echo; echo 'Press Enter to close…'; read _");
            }
        }
        "daemon_start" => {
            spawn_detached("tytus", &["daemon", "run"]);
        }
        "daemon_stop" => {
            spawn_detached("tytus", &["daemon", "stop"]);
        }
        "daemon_restart" => {
            let _ = std::process::Command::new("tytus").args(["daemon", "stop"]).status();
            std::thread::sleep(std::time::Duration::from_millis(500));
            spawn_detached("tytus", &["daemon", "run"]);
        }
        "settings_configure" => {
            open_in_terminal_simple("tytus configure");
        }
        "autostart_toggle" => {
            // Read-on-click: avoid racing with a 5s-stale state field.
            let installed = check_autostart_installed();
            let cmd = if installed {
                "tytus autostart uninstall; echo; echo 'Press Enter to close…'; read _"
            } else {
                "tytus autostart install; echo; echo 'Press Enter to close…'; read _"
            };
            open_in_terminal_simple(cmd);
        }
        "tray_autostart_toggle" => {
            // Toggling the tray's own launch-at-login agent. `tytus tray install`
            // also installs /Applications/Tytus.app, so toggling it on from a
            // bundle-less machine is a full setup in one click.
            let installed = check_tray_autostart_installed();
            let cmd = if installed {
                "tytus tray uninstall; echo; echo 'Press Enter to close…'; read _"
            } else {
                "tytus tray install; echo; echo 'Press Enter to close…'; read _"
            };
            open_in_terminal_simple(cmd);
        }
        "install_app" => {
            open_in_terminal_simple("tytus tray install; echo; echo 'Press Enter to close…'; read _");
        }
        "copy_env" => {
            copy_connection_info(state);
        }
        "copy_ail_url" => {
            let (url, _key) = connection_pair(state);
            copy_to_clipboard(&format!("{}/v1", url));
            notify("Tytus", "AIL_URL copied to clipboard.");
        }
        "copy_ail_key" => {
            let (_url, key) = connection_pair(state);
            if let Some(k) = key {
                copy_to_clipboard(&k);
                notify("Tytus", "AIL_API_KEY copied to clipboard.");
            } else {
                notify("Tytus", "No stable key yet — run tytus login first.");
            }
        }
        "copy_ail_exports" => {
            copy_connection_info(state);
        }
        "copy_openai_block" => {
            let (url, key) = connection_pair(state);
            if let Some(k) = key {
                copy_to_clipboard(&format!(
                    "export OPENAI_BASE_URL=\"{}/v1\"\n\
                     export OPENAI_API_KEY=\"{}\"\n\
                     export OPENAI_API_BASE=\"{}/v1\"\n",
                    url, k, url
                ));
                notify("Tytus", "OpenAI exports copied to clipboard.");
            } else {
                notify("Tytus", "No stable key yet — run tytus login first.");
            }
        }
        "copy_anthropic_block" => {
            let (url, key) = connection_pair(state);
            if let Some(k) = key {
                // ANTHROPIC_BASE_URL is the bare origin (no /v1) — the
                // SDK appends /v1/messages itself, so double-prefixing
                // would 404.
                copy_to_clipboard(&format!(
                    "export ANTHROPIC_API_KEY=\"{}\"\n\
                     export ANTHROPIC_BASE_URL=\"{}\"\n",
                    k, url
                ));
                notify("Tytus", "Anthropic exports copied to clipboard.");
            } else {
                notify("Tytus", "No stable key yet — run tytus login first.");
            }
        }
        "copy_ail_json" => {
            let (url, key) = connection_pair(state);
            let json = serde_json::json!({
                "url": format!("{}/v1", url),
                "api_key": key.as_deref().unwrap_or(""),
            });
            copy_to_clipboard(&serde_json::to_string_pretty(&json).unwrap_or_default());
            notify("Tytus", "Config JSON copied to clipboard.");
        }
        "open_mcp_guide" => {
            let _ = std::process::Command::new("open")
                .arg("https://github.com/traylinx/tytus-cli#connect-from-claude-cursor-opencode")
                .status();
        }
        "test" => {
            open_in_terminal_simple("tytus test; echo; echo 'Press Enter to close…'; read _");
        }
        "view_daemon_log" => {
            open_log_file("/tmp/tytus/daemon.log");
        }
        "view_startup_log" => {
            open_log_file("/tmp/tytus/autostart.log");
        }
        "doctor" => {
            open_in_terminal_simple("tytus doctor; echo; echo 'Press Enter to close…'; read _");
        }
        "docs" => {
            let _ = std::process::Command::new("open")
                .arg(DOCS_URL)
                .status();
        }
        "about" => {
            let version = env!("CARGO_PKG_VERSION");
            let msg = format!(
                "Tytus Tray v{}\\n\\nPrivate AI pod for your terminal.\\nTraylinx / Makakoo.",
                version
            );
            // macOS: display via osascript; everywhere else: println.
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("osascript")
                    .arg("-e")
                    .arg(format!(
                        "display dialog \"{}\" with title \"About Tytus\" buttons {{\"OK\"}} default button 1 with icon note",
                        msg
                    ))
                    .status();
            }
            #[cfg(not(target_os = "macos"))]
            {
                println!("{}", msg);
            }
        }
        "launch_terminal" => {
            if let Some(conn) = get_pod_connection(state) {
                launcher::launch_terminal(&conn);
            }
        }
        "quit" => {
            single_instance::release();
            std::process::exit(0);
        }
        // Open the pod's agent web UI.
        //
        // Reuse path: if a forwarder is already running (marker file
        // /tmp/tytus/ui-<pod>.port with live pid + bound port), just
        // re-open the browser to the existing URL.
        //
        // Fresh path: spawn `tytus ui --pod N --no-open` DETACHED (no
        // Terminal, no TTY) so the forwarder survives the user closing
        // any window. Log to /tmp/tytus/ui-<pod>.log. Then poll the
        // marker up to ~3s for the port to come up, then `open` the
        // browser. The forwarder is stopped explicitly via the per-pod
        // "Stop Forwarder" menu item (or `tytus ui --stop --pod N`),
        // NOT by closing a Terminal that no longer exists.
        other if other.starts_with("pod_") && other.ends_with("_open") => {
            let pod_id = other.trim_start_matches("pod_").trim_end_matches("_open").to_string();
            if let Some(existing_url) = existing_ui_forwarder(&pod_id) {
                let _ = std::process::Command::new("open").arg(&existing_url).spawn();
            } else {
                spawn_detached_ui(&pod_id);
                // Poll the marker up to 3s, then open whatever URL it landed on.
                let pod_for_poll = pod_id.clone();
                std::thread::spawn(move || {
                    for _ in 0..30 {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        if let Some(url) = existing_ui_forwarder(&pod_for_poll) {
                            let _ = std::process::Command::new("open").arg(&url).spawn();
                            return;
                        }
                    }
                    // Timed out — the forwarder may need the tunnel swap path
                    // (sudo elevation dialog). Fall back to Terminal so the
                    // user can see what's happening.
                    let _ = std::process::Command::new("osascript")
                        .arg("-e")
                        .arg(format!(
                            "display notification \"Forwarder for pod {} didn't come up in 3s — opening Terminal for diagnostics.\" with title \"Tytus\"",
                            pod_for_poll
                        ))
                        .spawn();
                });
            }
        }
        // Stop the per-pod forwarder daemon.
        other if other.starts_with("pod_") && other.ends_with("_stop_forwarder") => {
            let pod_id = other.trim_start_matches("pod_").trim_end_matches("_stop_forwarder").to_string();
            // Run via CLI so we get the same marker cleanup + exit code path
            // that CLI users see. Detached — no Terminal window for this.
            spawn_detached("tytus", &["ui", "--stop", "--pod", &pod_id]);
        }
        // Agent container restart (no state loss).
        other if other.starts_with("pod_") && other.ends_with("_restart") => {
            let pod_id = other.trim_start_matches("pod_").trim_end_matches("_restart");
            open_in_terminal_simple(&format!(
                "tytus restart --pod {}; echo; echo 'Press Enter to close…'; read _",
                shell_escape(pod_id),
            ));
        }
        // Destructive: frees units + wipes pod workspace.
        other if other.starts_with("pod_") && other.ends_with("_revoke") => {
            let pod_id = other.trim_start_matches("pod_").trim_end_matches("_revoke");
            if confirm_dialog(
                &format!("Revoke pod {}?", pod_id),
                "This frees the units and permanently deletes anything saved inside this pod's agent workspace. You can allocate a new pod from the tray menu afterwards.",
            ) {
                open_in_terminal_simple(&format!(
                    "tytus revoke {}; echo; echo 'Press Enter to close…'; read _",
                    shell_escape(pod_id),
                ));
            }
        }
        // Channel catalog (informational) — user wants to see what they
        // can configure before picking one.
        other if other.starts_with("pod_") && other.ends_with("_channels_catalog") => {
            let pod_id = other
                .trim_start_matches("pod_")
                .trim_end_matches("_channels_catalog");
            open_in_terminal_simple(&format!(
                "tytus channels catalog; echo; echo 'To configure: tytus channels add --pod {} --type <NAME> --token <TOKEN>'; echo 'Press Enter to close…'; read _",
                shell_escape(pod_id),
            ));
        }
        // Add a specific channel — opens Terminal with the exact
        // command skeleton so the user only needs to paste their token.
        // The command stays on screen after completion so the user can
        // see the success/failure message before the window closes.
        other if other.starts_with("pod_") && other.contains("_channel_") && other.ends_with("_add") => {
            if let Some(rest) = other.strip_prefix("pod_") {
                if let Some(middle) = rest.strip_suffix("_add") {
                    // middle = "<pod>_channel_<type>"
                    if let Some((pod_id, rest)) = middle.split_once("_channel_") {
                        let channel = rest;
                        let pod_s = shell_escape(pod_id);
                        let chan_s = shell_escape(channel);
                        let label = channel_label(channel);
                        // Clean prompt flow. Spinner between steps so
                        // the user sees the CLI is working on a slow
                        // redeploy (~10s), and a clear colored result
                        // line at the end so they know it landed.
                        open_in_terminal_simple(&format!(
                            "clear 2>/dev/null; \
                             printf '\\033[1m{label}\\033[0m — pod {pod}\\n\\n'; \
                             tytus channels catalog 2>/dev/null | awk '/^  {chan} /,/^$/' | sed '1,/^$/!d'; \
                             echo; \
                             printf 'Paste your primary token (hidden): '; read -rs TOK; echo; \
                             if [ -z \"$TOK\" ]; then \
                               printf '\\033[33mAborted — no token entered.\\033[0m\\n'; \
                             else \
                               echo; echo 'Writing credential to pod and restarting agent (this takes ~10s)…'; echo; \
                               if tytus channels add --pod {pod} --type {chan} --token \"$TOK\"; then \
                                 printf '\\n\\033[32m✓ Done.\\033[0m Your agent on pod {pod} can now use {label}.\\n'; \
                               else \
                                 printf '\\n\\033[31m✗ Something went wrong.\\033[0m Check the message above, then retry.\\n'; \
                               fi; \
                             fi; \
                             echo; echo 'Press Enter to close…'; read _",
                            pod = pod_s,
                            chan = chan_s,
                            label = label,
                        ));
                    }
                }
            }
        }
        // Remove a configured channel — clears keychain + manifest +
        // redeploys agent.
        other if other.starts_with("pod_") && other.contains("_channel_") && other.ends_with("_remove") => {
            if let Some(rest) = other.strip_prefix("pod_") {
                if let Some(middle) = rest.strip_suffix("_remove") {
                    if let Some((pod_id, rest)) = middle.split_once("_channel_") {
                        let channel = rest;
                        let channel_label = channel_label(channel);
                        if confirm_dialog(
                            &format!("Remove {} from pod {}?", channel_label, pod_id),
                            "Clears the channel's credentials from the OS keychain, removes them from the pod's state volume, and redeploys the agent container so the channel stops operating. Re-adding later will require the credentials again.",
                        ) {
                            open_in_terminal_simple(&format!(
                                "tytus channels remove --pod {} --type {}; echo; echo 'Press Enter to close…'; read _",
                                shell_escape(pod_id),
                                shell_escape(channel),
                            ));
                        }
                    }
                }
            }
        }
        // Agent uninstall = stop container, keep pod slot (AIL still works).
        other if other.starts_with("pod_") && other.ends_with("_uninstall") => {
            let pod_id = other.trim_start_matches("pod_").trim_end_matches("_uninstall");
            if confirm_dialog(
                &format!("Uninstall agent on pod {}?", pod_id),
                "The agent container is stopped and removed but the pod slot stays allocated. AIL gateway access keeps working through the sidecar. Use 'Revoke Pod' to fully free units.",
            ) {
                open_in_terminal_simple(&format!(
                    "tytus agent uninstall {}; echo; echo 'Press Enter to close…'; read _",
                    shell_escape(pod_id),
                ));
            }
        }
        // Primary install entry point — opens the localhost wizard in
        // the user's default browser (SPRINT §6 E). The per-agent
        // terminal shortcuts below are legacy + fallback when the
        // localhost server isn't bound (rare).
        "install_agent" => {
            web_server::open_wizard();
        }
        // Install a specific agent via the terminal-picker fallback.
        "install_agent_nemoclaw" => {
            open_in_terminal_simple(
                "tytus agent install nemoclaw; echo; echo 'Press Enter to close…'; read _"
            );
        }
        "install_agent_hermes" => {
            open_in_terminal_simple(
                "tytus agent install hermes; echo; echo 'Press Enter to close…'; read _"
            );
        }
        other if other.starts_with("launch_") => {
            let binary = &other["launch_".len()..];
            let clis = launcher::detect_installed_clis();
            if let Some(cli) = clis.iter().find(|c| c.binary == binary) {
                if let Some(conn) = get_pod_connection(state) {
                    launcher::launch_in_terminal(cli, &conn);
                }
            }
        }
        _ => {}
    }
}

/// Minimal shell single-quote escaper. Safe for the restricted set of
/// strings we splice into the `.command` script (pod ids: `\d{2}`, agent
/// names: `[a-z]+`). We still escape defensively in case IDs change format.
/// If `tytus ui --pod <pod_id>` is already running for this pod, return
/// the URL the user should be sent to (so we can skip spawning a fresh
/// Terminal on repeat clicks of "Open in Browser"). Marker format is
/// written by cmd_ui in cli/src/main.rs:
///   /tmp/tytus/ui-<pod>.port = {"pid":N,"port":P, "upstream":"..."}
/// We trust the marker only when the pid is alive AND the port still
/// accepts a TCP connect. Anything else = stale → return None and let
/// the caller spawn a fresh forwarder.
fn existing_ui_forwarder(pod_id: &str) -> Option<String> {
    let path = format!("/tmp/tytus/ui-{}.port", pod_id);
    let raw = std::fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let pid = v.get("pid").and_then(|x| x.as_u64())? as i32;
    let port = v.get("port").and_then(|x| x.as_u64())? as u16;

    let pid_alive = unsafe { libc::kill(pid, 0) == 0 };
    if !pid_alive {
        let _ = std::fs::remove_file(&path);
        return None;
    }
    let addr: std::net::SocketAddr = ([127, 0, 0, 1], port).into();
    match std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(300)) {
        Ok(_) => Some(format!("http://localhost:{}/", port)),
        Err(_) => {
            let _ = std::fs::remove_file(&path);
            None
        }
    }
}

/// True if a WireGuard tunnel is currently up for this pod. We check
/// `/tmp/tytus/tunnel-<pod>.pid` — written by cmd_tunnel_up under the
/// elevated helper — AND verify the pid is actually alive. The pidfile
/// lingering without a running daemon is a known failure mode (crash
/// before cleanup), so liveness matters.
///
/// The tunnel daemon runs as root. A user-space `kill(pid, 0)` against
/// a root process returns -1 with errno=EPERM (not ESRCH), so we'd
/// incorrectly treat the process as dead. Fix: accept EPERM as "alive"
/// — we couldn't signal it but it definitely exists. Discovered
/// 2026-04-19 smoke test: tray always labelled pod rows "Connect & Open
/// in Browser" because this function returned false even with utun4
/// actively routing packets.
fn tunnel_reaches_pod(pod_id: &str) -> bool {
    let path = format!("/tmp/tytus/tunnel-{}.pid", pod_id);
    let raw = match std::fs::read_to_string(&path) { Ok(r) => r, Err(_) => return false };
    let pid: i32 = match raw.trim().parse() { Ok(p) => p, Err(_) => return false };
    if pid <= 1 { return false; }
    unsafe {
        if libc::kill(pid, 0) == 0 { return true; }
    }
    // libc::kill failed. Alive-but-EPERM means the daemon is running
    // under a different uid (root), which is the normal happy path.
    let errno = unsafe { *libc::__error() };
    errno == libc::EPERM
}

/// Start `tytus ui --pod <pod_id> --no-open` as a fully detached
/// background process. No TTY, no Terminal window. Output is appended
/// to /tmp/tytus/ui-<pod>.log so diagnostics survive without needing a
/// visible shell. The tray just launches it and forgets — the forwarder
/// stays alive until SIGTERM'd via "Stop Forwarder" or `tytus ui --stop`.
fn spawn_detached_ui(pod_id: &str) {
    use std::process::Stdio;
    let log_path = format!("/tmp/tytus/ui-{}.log", pod_id);
    let log = std::fs::OpenOptions::new()
        .create(true).append(true).open(&log_path);
    let stdout: Stdio = match &log {
        Ok(f) => Stdio::from(f.try_clone().unwrap_or_else(|_| std::fs::File::open("/dev/null").unwrap())),
        Err(_) => Stdio::null(),
    };
    let stderr: Stdio = match log {
        Ok(f) => Stdio::from(f),
        Err(_) => Stdio::null(),
    };
    let _ = std::process::Command::new("tytus")
        .args(["ui", "--pod", pod_id, "--no-open"])
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .spawn();
}

fn shell_escape(s: &str) -> String {
    let mut out = String::from("'");
    for c in s.chars() {
        if c == '\'' { out.push_str("'\\''"); } else { out.push(c); }
    }
    out.push('\'');
    out
}

fn spawn_detached(program: &str, args: &[&str]) {
    let _ = std::process::Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

/// Run a CLI command silently in a background thread and deliver the
/// outcome via macOS notification. Also writes stdout/stderr to
/// `~/.tytus/logs/<action>.log` for post-hoc debugging when something
/// fails and the user wants to see what happened.
///
/// `probe_outcome` runs after the process exits and returns a human
/// message. It receives the "was gateway reachable before we started"
/// flag, which lets the probe distinguish between "connect did
/// nothing because we were already connected" vs "connect failed".
///
/// Design note: we intentionally do NOT show the command's own stdout
/// to the user. Users don't need to see "Downloading tunnel config /
/// Activating WireGuard tunnel / Tunnel daemon running (pid …)" —
/// they need to know one of: "connected", "not connected, here's why,
/// open Troubleshoot". Ollama follows the same philosophy.
fn run_silent_with_notify<F>(
    program: &str,
    args: &[&str],
    action_label: &'static str,
    probe_outcome: F,
)
where
    F: Fn(bool) -> Result<String, String> + Send + 'static,
{
    let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let program_owned = program.to_string();
    let was_reachable_before = gateway_probe::probe_gateway();

    std::thread::spawn(move || {
        // Best-effort log capture. File failure is non-fatal — we
        // still want the notification at the end.
        let log_path = std::env::var("HOME")
            .ok()
            .map(|h| {
                let dir = std::path::PathBuf::from(h).join(".tytus/logs");
                let _ = std::fs::create_dir_all(&dir);
                dir.join(format!("{}.log", action_label.to_lowercase()))
            });
        let stdout: std::process::Stdio = match log_path.as_ref().and_then(|p| {
            std::fs::OpenOptions::new().create(true).append(true).open(p).ok()
        }) {
            Some(f) => std::process::Stdio::from(f),
            None => std::process::Stdio::null(),
        };
        let stderr: std::process::Stdio = match log_path.as_ref().and_then(|p| {
            std::fs::OpenOptions::new().create(true).append(true).open(p).ok()
        }) {
            Some(f) => std::process::Stdio::from(f),
            None => std::process::Stdio::null(),
        };

        let args_refs: Vec<&str> = args_owned.iter().map(|s| s.as_str()).collect();
        let status = std::process::Command::new(&program_owned)
            .args(&args_refs)
            .stdin(std::process::Stdio::null())
            .stdout(stdout)
            .stderr(stderr)
            .status();

        // Wake the poll loop now that the command has finished, so the
        // next menu rebuild picks up the new state regardless of the
        // schedule ticks.
        trigger_refresh();

        match status {
            Ok(s) if s.success() => {
                match probe_outcome(was_reachable_before) {
                    Ok(msg) => notify("Tytus", &msg),
                    Err(msg) => notify("Tytus", &msg),
                }
            }
            Ok(s) => {
                notify(
                    "Tytus",
                    &format!(
                        "{} exited with code {:?}. See ~/.tytus/logs/{}.log.",
                        action_label,
                        s.code(),
                        action_label.to_lowercase(),
                    ),
                );
            }
            Err(e) => {
                notify(
                    "Tytus",
                    &format!("Couldn't run tytus {}: {}", action_label.to_lowercase(), e),
                );
            }
        }
    });
}

/// Native yes/no dialog via osascript. Returns true iff the user clicked OK.
/// Used to gate destructive actions (Sign Out, future pod revocation).
#[cfg(target_os = "macos")]
fn confirm_dialog(title: &str, body: &str) -> bool {
    let script = format!(
        "display dialog \"{}\" with title \"{}\" buttons {{\"Cancel\", \"OK\"}} default button \"Cancel\" cancel button \"Cancel\" with icon caution",
        body.replace('"', "\\\"").replace('\n', " "),
        title.replace('"', "\\\""),
    );
    std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(not(target_os = "macos"))]
fn confirm_dialog(_title: &str, _body: &str) -> bool {
    // On Linux we can't guarantee a GUI confirm dialog is available (no
    // osascript equivalent everywhere). Skip confirmation — the terminal
    // window that actually runs the destructive command will prompt.
    true
}

/// Show a notification in the menu bar / notification center.
#[cfg(target_os = "macos")]
fn notify(title: &str, body: &str) {
    let script = format!(
        "display notification \"{}\" with title \"{}\"",
        body.replace('"', "\\\""),
        title.replace('"', "\\\""),
    );
    let _ = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

#[cfg(not(target_os = "macos"))]
fn notify(_title: &str, _body: &str) {}

/// Pull the stable (URL, API key) pair from TrayState.
///
/// All of a user's pods share the same stable pair (it's per-user, not
/// per-pod), so we return the first pod with the key populated. If the
/// state is sparse — e.g. the user is logged in but the daemon hasn't
/// yet hydrated stable fields — we fall back to the canonical URL and
/// return `None` for the key so the caller can surface a "run tytus
/// login" hint instead of copying an empty string.
fn connection_pair(state: &Arc<Mutex<TrayState>>) -> (String, Option<String>) {
    let s = state.lock().unwrap();
    let primary = s.pods.iter().find(|p| p.stable_user_key.is_some());
    let url = primary
        .and_then(|p| p.stable_ai_endpoint.clone())
        .unwrap_or_else(|| "http://10.42.42.1:18080".to_string());
    let key = primary.and_then(|p| p.stable_user_key.clone());
    (url, key)
}

/// Put arbitrary text on the system clipboard. Factored out so individual
/// menu items can copy URL / key / JSON independently rather than always
/// dumping the full export block.
fn copy_to_clipboard(text: &str) {
    #[cfg(target_os = "macos")]
    {
        use std::io::Write;
        if let Ok(mut child) = std::process::Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        use std::io::Write;
        for (bin, args) in [("xclip", &["-selection", "clipboard"][..]), ("xsel", &["--clipboard", "--input"][..])] {
            if let Ok(mut child) = std::process::Command::new(bin)
                .args(args)
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                }
                let _ = child.wait();
                return;
            }
        }
    }
}

/// Put the stable OpenAI-compatible env-var block on the clipboard.
/// The user can paste it directly into .env files, IDE settings, or another
/// shell — no terminal round-trip required.
///
/// Uses the state.pods snapshot (read via `connection_pair`) rather than
/// the daemon's live view, so this works even when the tunnel isn't up
/// yet — the stable URL + key pair doesn't depend on the tunnel being
/// active, and users often pre-configure their clients before first
/// connect.
fn copy_connection_info(state: &Arc<Mutex<TrayState>>) {
    let (url, key) = connection_pair(state);
    let Some(key) = key else {
        notify("Tytus", "No stable key yet — run `tytus login` first.");
        return;
    };

    // Anthropic's gateway path is /v1 too and its SDKs route calls to
    // {base}/v1/messages. We strip the trailing /v1 from AIL_URL when
    // setting ANTHROPIC_BASE_URL so the SDK doesn't double-append.
    let ail_bare = url.as_str();  // e.g. http://10.42.42.1:18080 (no /v1)
    let text = format!(
        "# AIL — your private AI gateway (canonical names)\n\
         export AIL_URL=\"{bare}/v1\"\n\
         export AIL_API_KEY=\"{key}\"\n\
         \n\
         # OpenAI-compatible aliases — used by Claude Code, Cursor,\n\
         # OpenCode, Continue, Aider and every tool that reads\n\
         # OPENAI_BASE_URL / OPENAI_API_KEY by convention.\n\
         export OPENAI_BASE_URL=\"$AIL_URL\"\n\
         export OPENAI_API_KEY=\"$AIL_API_KEY\"\n\
         export OPENAI_API_BASE=\"$AIL_URL\"\n\
         \n\
         # Anthropic-compatible aliases — used by the Anthropic SDK\n\
         # (anthropic Python/TS/Ruby), Claude Code with a custom base\n\
         # URL, and any Anthropic-native tooling. ANTHROPIC_BASE_URL is\n\
         # the bare origin (no /v1) because the SDK appends /v1/messages.\n\
         export ANTHROPIC_API_KEY=\"$AIL_API_KEY\"\n\
         export ANTHROPIC_BASE_URL=\"{bare}\"\n",
        bare = ail_bare,
        key = key,
    );

    #[cfg(target_os = "macos")]
    {
        use std::io::Write;
        if let Ok(mut child) = std::process::Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
            notify("Tytus", "Connection info copied to clipboard.");
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        // Best-effort: try xclip, then xsel.
        use std::io::Write;
        for (bin, args) in [("xclip", &["-selection", "clipboard"][..]), ("xsel", &["--clipboard", "--input"][..])] {
            if let Ok(mut child) = std::process::Command::new(bin)
                .args(args)
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                }
                let _ = child.wait();
                return;
            }
        }
        // Fallback: dump to a temp file so the user can read it.
        let _ = std::fs::write("/tmp/tytus/connection-info.sh", &text);
    }
}

/// Open a log file in the system's default viewer. On macOS that's the
/// Console app for .log files, which gives live tail + search for free.
fn open_log_file(path: &str) {
    if !std::path::Path::new(path).exists() {
        notify("Tytus", &format!("Log not found yet: {}", path));
        return;
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(path)
            .spawn();
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(path)
            .spawn();
    }
}

/// Open a shell command in a new terminal window.
///
/// Uses a `.command` file opened via `open(1)` on macOS — macOS launches
/// Terminal.app for `.command` files through LaunchServices, which does NOT
/// require Automation permission (unlike `osascript tell "Terminal" to do
/// script ...`, which silently fails if the user hasn't granted it).
///
/// This is why clicking Doctor did nothing before: tytus-tray had no
/// Automation entitlement for Terminal.app, so the AppleScript was rejected
/// with no visible prompt.
#[cfg(target_os = "macos")]
fn open_in_terminal_simple(cmd: &str) {
    let _ = std::fs::create_dir_all("/tmp/tytus");
    // Unique path per invocation so rapid clicks don't race on the same file.
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let script_path = format!("/tmp/tytus/launch-{}.command", nonce);

    // Why the PATH prepend: `tytus` lives in ~/bin and the user's login shell
    // only picks it up from .zshrc. A freshly-spawned Terminal window runs a
    // login shell that sources .zshrc, so usually PATH is correct — but we
    // prepend defensively so the menu works even on minimal shell configs.
    // The .command file also self-deletes at the end so /tmp doesn't fill up.
    let script = format!(
        "#!/bin/bash\n\
         export PATH=\"$HOME/bin:/usr/local/bin:/opt/homebrew/bin:$PATH\"\n\
         cd \"$HOME\"\n\
         {cmd}\n\
         rm -f \"{path}\"\n",
        cmd = cmd,
        path = script_path,
    );

    if std::fs::write(&script_path, &script).is_err() {
        return;
    }

    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(
        &script_path,
        std::fs::Permissions::from_mode(0o700),
    );

    let _ = std::process::Command::new("open")
        .arg(&script_path)
        .spawn();
}

#[cfg(not(target_os = "macos"))]
fn open_in_terminal_simple(cmd: &str) {
    // Best-effort: try common Linux terminals.
    for term in &["gnome-terminal", "konsole", "xterm"] {
        if std::process::Command::new(term)
            .args(["--", "sh", "-c", cmd])
            .spawn()
            .is_ok()
        {
            return;
        }
    }
}

/// Get the current pod connection info from the daemon.
fn get_pod_connection(state: &Arc<Mutex<TrayState>>) -> Option<launcher::PodConnection> {
    let s = state.lock().unwrap().clone();
    if !s.daemon_running || !s.tunnel_active {
        return None;
    }

    // Get stable endpoint + key from daemon status
    let resp = socket::send_raw_command("status")?;
    let data = resp.get("data")?;
    let pods = data.get("pods")?.as_array()?;
    let pod = pods.first()?;

    let gateway = pod.get("stable_ai_endpoint")
        .and_then(|v| v.as_str())
        .or_else(|| pod.get("ai_endpoint").and_then(|v| v.as_str()))?;
    let key = pod.get("stable_user_key")
        .and_then(|v| v.as_str())
        .or_else(|| pod.get("pod_api_key").and_then(|v| v.as_str()))
        .unwrap_or("sk-tytus");

    Some(launcher::PodConnection {
        ai_gateway: gateway.to_string(),
        api_key: key.to_string(),
        model: "ail-compound".to_string(),
    })
}

// Minimal drop-guard helper (avoid an extra crate dep).
mod scopeguard_lite {
    pub struct OnDrop<F: FnOnce()> {
        f: Option<F>,
    }
    impl<F: FnOnce()> OnDrop<F> {
        pub fn new(f: F) -> Self { Self { f: Some(f) } }
    }
    impl<F: FnOnce()> Drop for OnDrop<F> {
        fn drop(&mut self) {
            if let Some(f) = self.f.take() { f(); }
        }
    }
}

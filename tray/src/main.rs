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
}

/// Per-pod info. Agent types beyond the two we ship are silently displayed
/// as their raw id — the unit cost falls back to 1 (the safe default) so
/// we never over-credit the budget.
#[derive(Debug, Clone, Default)]
pub struct PodInfo {
    pub pod_id: String,
    pub agent_type: String,
    pub tunnel_active: bool,
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
        match self.agent_type.as_str() {
            "nemoclaw" => "NemoClaw".into(),
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
            HealthDot::Connected
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

    // Poll thread pushes `(HealthDot, tooltip)` updates to the main thread
    // whenever the dot actually changes. De-duping means we don't hammer
    // NSStatusItem for identical icons every 5s.
    let poll_state = state.clone();
    std::thread::spawn(move || {
        let mut last_dot: Option<HealthDot> = None;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(5));
            let new_state = socket::poll_daemon_status();
            let new_dot = HealthDot::from_state(&new_state);
            let new_tooltip = tooltip_for(&new_state);
            *poll_state.lock().unwrap() = new_state;

            if last_dot == Some(new_dot) {
                continue;
            }
            last_dot = Some(new_dot);

            #[cfg(target_os = "macos")]
            apply_tray_update(new_dot, new_tooltip);

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
    if !state.daemon_running {
        // Daemon-controls section below emits Start Daemon.
    } else if !state.logged_in {
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

        // Power-user conveniences that only make sense when connected.
        let _ = menu.append(&MenuItem::with_id("copy_env", "Copy Connection Info", true, None));
        let _ = menu.append(&MenuItem::with_id("test", "Run Health Test", true, None));

        let _ = menu.append(&PredefinedMenuItem::separator());
    } else {
        let _ = menu.append(&MenuItem::with_id("connect", "Connect", true, None));
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
            // so users see AIL is always-on and not confused by a mysterious
            // "Default (AIL only)" appearing in the agent swap/revoke UI
            // where those actions don't make sense. Per SPRINT §4.3.
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
                let _ = pods_sub.append(&MenuItem::with_id(
                    format!("pod_{}_revoke", p.pod_id),
                    "  Revoke Default Pod", true, None,
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
                let _ = pods_sub.append(&MenuItem::with_id(
                    format!("pod_{}_restart", p.pod_id),
                    "  Restart Agent", true, None,
                ));
                // Replace Agent: offer the OTHER agent type. `tytus agent
                // replace` keeps the slot allocated and swaps the container
                // only — unlike the pre-sprint "Switch" which did a full
                // revoke+reallocate that broke tooling locked to the
                // per-pod subnet IP.
                let other = match p.agent_type.as_str() {
                    "hermes" => ("nemoclaw", "NemoClaw", 1u32),
                    _ => ("hermes", "Hermes", 2u32),
                };
                let spare_after_free = state.units_used.saturating_sub(p.units());
                let can_swap = state.units_limit == 0
                    || state.units_limit.saturating_sub(spare_after_free) >= other.2;
                let swap_label = format!("  Replace with {} ({} unit{})",
                    other.1, other.2, if other.2 == 1 { "" } else { "s" });
                let _ = pods_sub.append(&MenuItem::with_id(
                    format!("pod_{}_replace_{}", p.pod_id, other.0),
                    &swap_label, can_swap, None,
                ));
                // Uninstall Agent: keeps the pod slot allocated (AIL still
                // works through it), drops the container. SPRINT §4.3.
                let _ = pods_sub.append(&MenuItem::with_id(
                    format!("pod_{}_uninstall", p.pod_id),
                    "  Uninstall Agent  (keeps pod)", true, None,
                ));
                let _ = pods_sub.append(&MenuItem::with_id(
                    format!("pod_{}_revoke", p.pod_id),
                    "  Revoke Pod", true, None,
                ));
                let _ = pods_sub.append(&PredefinedMenuItem::separator());
            }
        }

        // "Install Agent ▸" — replaces the pre-sprint "Add Pod ▸". Routes
        // through `tytus agent install` (new B1 subcommand) so default pods
        // aren't created here, only agent-bearing ones. Phase E will swap
        // the terminal-picker handlers for a browser wizard.
        let add_sub = Submenu::new("Install Agent", true);
        let remaining = state.units_limit.saturating_sub(state.units_used);
        let nemo_ok = state.units_limit == 0 || remaining >= 1;
        let hermes_ok = state.units_limit == 0 || remaining >= 2;
        let _ = add_sub.append(&MenuItem::with_id(
            "install_agent_nemoclaw",
            format!("NemoClaw  (1 unit){}", if nemo_ok { "" } else { "  — not enough units" }),
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
    match id {
        "connect" => {
            spawn_detached("tytus", &["connect"]);
        }
        "disconnect" => {
            spawn_detached("tytus", &["disconnect"]);
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
        // Agent replace = stop old container + deploy new on same slot.
        // `tytus agent replace` (Phase B) keeps the WG subnet stable —
        // unlike the pre-sprint "Switch" which revoked+reallocated.
        other if other.starts_with("pod_") && other.contains("_replace_") => {
            // Parse `pod_<id>_replace_<agent>`
            let rest = other.trim_start_matches("pod_");
            if let Some((pod_id, agent)) = rest.split_once("_replace_") {
                if confirm_dialog(
                    &format!("Replace agent on pod {} with {}?",
                        pod_id,
                        match agent { "hermes" => "Hermes", "nemoclaw" => "NemoClaw", o => o }),
                    "The pod slot stays allocated and keeps its subnet; only the agent container is replaced. Existing container state (volumes, in-memory sessions) is lost.",
                ) {
                    let script = format!(
                        "tytus agent replace {pid} {agent} --yes; \
                         echo; echo 'Press Enter to close…'; read _",
                        pid = shell_escape(pod_id),
                        agent = shell_escape(agent),
                    );
                    open_in_terminal_simple(&script);
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
        // Install a specific agent. Phase E will replace these with a
        // browser wizard; for now open the CLI in a terminal so the user
        // sees the streaming install logs.
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

/// Put the stable OpenAI-compatible env-var block on the clipboard.
/// The user can paste it directly into .env files, IDE settings, or another
/// shell — no terminal round-trip required.
fn copy_connection_info(state: &Arc<Mutex<TrayState>>) {
    let Some(conn) = get_pod_connection(state) else {
        notify("Tytus", "Not connected — nothing to copy.");
        return;
    };

    let text = format!(
        "export OPENAI_BASE_URL=\"{base}/v1\"\n\
         export OPENAI_API_KEY=\"{key}\"\n\
         export OPENAI_API_BASE=\"{base}/v1\"\n\
         export AI_GATEWAY=\"{base}\"\n",
        base = conn.ai_gateway,
        key = conn.api_key,
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

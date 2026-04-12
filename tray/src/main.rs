//! Tytus Tray — system tray icon for managing your private AI pod.
//!
//! Shows a menu bar icon (macOS) / system tray icon (Windows/Linux) with:
//! - Status line (daemon state, connection info)
//! - Connect / Disconnect
//! - Start / Stop daemon
//! - Quit
//!
//! Communicates with tytus-daemon via Unix socket at /tmp/tytus/daemon.sock.

use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::TrayIconBuilder;
use std::sync::{Arc, Mutex};

mod icon;
mod launcher;
mod socket;

// ── State ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TrayState {
    pub daemon_running: bool,
    pub logged_in: bool,
    pub token_valid: bool,
    pub email: String,
    pub tier: String,
    pub pod_count: usize,
    pub tunnel_active: bool,
    pub daemon_pid: u64,
    pub uptime_secs: u64,
}

impl Default for TrayState {
    #[allow(clippy::derivable_impls)]
    fn default() -> Self {
        Self {
            daemon_running: false,
            logged_in: false,
            token_valid: false,
            email: String::new(),
            tier: String::new(),
            pod_count: 0,
            tunnel_active: false,
            daemon_pid: 0,
            uptime_secs: 0,
        }
    }
}

// ── Main ────────────────────────────────────────────────────

fn main() {
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

    // Build menu + tray
    let menu = build_menu(&state.lock().unwrap());
    let tray_icon = icon::create_tray_icon();
    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Tytus — Private AI Pod")
        .with_icon(tray_icon)
        .build()
        .expect("Failed to create tray icon");

    // Spawn status polling thread — rebuilds tray menu every 5s
    let poll_state = state.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(5));
            let new_state = socket::poll_daemon_status();
            *poll_state.lock().unwrap() = new_state;
            // Rebuild menu with updated state
            let menu = build_menu(&poll_state.lock().unwrap());
            // NOTE: tray-icon doesn't support dynamic menu updates easily.
            // The menu is rebuilt but we'd need to set it on the tray again.
            // For Phase 1, the menu reflects state at click time via the
            // platform event loop's menu-will-open callback. This is a
            // known limitation — Phase 2 will use native NSMenu updates.
            let _ = menu; // consumed
        }
    });

    // Handle menu events in a background thread
    std::thread::spawn(move || {
        loop {
            if let Ok(event) = MenuEvent::receiver().recv() {
                handle_menu_event(event.id().0.as_str());
            }
        }
    });

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

fn build_menu(state: &TrayState) -> Menu {
    let menu = Menu::new();

    // Status line (disabled — just informational)
    let status_text = if !state.daemon_running {
        "Tytus: daemon not running".to_string()
    } else if !state.logged_in {
        "Tytus: not logged in".to_string()
    } else if state.tunnel_active {
        format!("● Connected ({})", state.email)
    } else {
        format!("○ Disconnected ({})", state.email)
    };
    let _ = menu.append(&MenuItem::with_id("status", &status_text, false, None));
    let _ = menu.append(&PredefinedMenuItem::separator());

    // Action items based on state
    if state.daemon_running && state.logged_in {
        if state.tunnel_active {
            let _ = menu.append(&MenuItem::with_id("disconnect", "Disconnect", true, None));

            // "Open in ▸" submenu — only when tunnel is active
            let clis = launcher::detect_installed_clis();
            if !clis.is_empty() {
                let open_sub = Submenu::new("Open in", true);
                for cli in &clis {
                    let id = format!("launch_{}", cli.binary);
                    let _ = open_sub.append(&MenuItem::with_id(&id, cli.name, true, None));
                }
                let _ = open_sub.append(&PredefinedMenuItem::separator());
                let _ = open_sub.append(&MenuItem::with_id("launch_terminal", "Terminal", true, None));
                let _ = menu.append(&open_sub);
            } else {
                let _ = menu.append(&MenuItem::with_id("launch_terminal", "Open Terminal", true, None));
            }
        } else {
            let _ = menu.append(&MenuItem::with_id("connect", "Connect", true, None));
        }
        let _ = menu.append(&PredefinedMenuItem::separator());
    }

    if state.daemon_running {
        let _ = menu.append(&MenuItem::with_id("daemon_stop", "Stop Daemon", true, None));
    } else {
        let _ = menu.append(&MenuItem::with_id("daemon_start", "Start Daemon", true, None));
    }

    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&MenuItem::with_id("quit", "Quit Tytus", true, None));

    menu
}

// ── Menu event handler ──────────────────────────────────────

fn handle_menu_event(id: &str) {
    match id {
        "connect" => {
            let _ = std::process::Command::new("tytus").args(["connect"]).spawn();
        }
        "disconnect" => {
            let _ = std::process::Command::new("tytus").args(["disconnect"]).spawn();
        }
        "daemon_start" => {
            let _ = std::process::Command::new("tytus")
                .args(["daemon", "run"])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
        }
        "daemon_stop" => {
            let _ = std::process::Command::new("tytus").args(["daemon", "stop"]).spawn();
        }
        "launch_terminal" => {
            if let Some(conn) = get_pod_connection() {
                launcher::launch_terminal(&conn);
            }
        }
        "quit" => {
            std::process::exit(0);
        }
        other if other.starts_with("launch_") => {
            let binary = &other["launch_".len()..];
            let clis = launcher::detect_installed_clis();
            if let Some(cli) = clis.iter().find(|c| c.binary == binary) {
                if let Some(conn) = get_pod_connection() {
                    launcher::launch_in_terminal(cli, &conn);
                }
            }
        }
        _ => {}
    }
}

/// Get the current pod connection info from the daemon.
fn get_pod_connection() -> Option<launcher::PodConnection> {
    let state = socket::poll_daemon_status();
    if !state.daemon_running || !state.tunnel_active {
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

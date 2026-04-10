mod state;
#[allow(dead_code)]
mod wizard;

use clap::{Parser, Subcommand, ValueEnum};
use state::{CliState, PodEntry};

#[derive(Parser)]
#[command(name = "tytus", about = "Tytus private AI pod — connect from any terminal", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Output as JSON (for programmatic use by AI CLIs)
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Clone, ValueEnum)]
enum AgentType {
    Nemoclaw,
    Hermes,
}

impl AgentType {
    fn as_str(&self) -> &str {
        match self { AgentType::Nemoclaw => "nemoclaw", AgentType::Hermes => "hermes" }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Full first-time setup wizard — login, allocate pod, configure, test
    Setup,
    /// Quick health test — runs a sample chat completion and verifies everything works
    Test,
    /// Interactive chat with your private AI pod
    Chat {
        /// Model to use (default: ail-compound)
        #[arg(short, long, default_value = "ail-compound")]
        model: String,
    },
    /// Configure your agent (OpenClaw / Hermes) interactively
    Configure,
    /// Login to Traylinx (opens browser for device auth)
    Login,
    /// Show current status: plan, pods, tunnels
    Status,
    /// Allocate a new pod and activate WireGuard tunnel
    Connect {
        /// Pod ID to reconnect (e.g. "01"). Omit to allocate new.
        #[arg(short, long)]
        pod: Option<String>,
        /// Agent type
        #[arg(short, long, value_enum, default_value = "nemoclaw")]
        agent: AgentType,
    },
    /// Clear stale tunnel state (tunnels are stopped via Ctrl+C in connect)
    Disconnect {
        /// Pod ID to disconnect. Omit to disconnect all.
        #[arg(short, long)]
        pod: Option<String>,
    },
    /// Revoke a specific pod (frees units)
    Revoke {
        /// Pod ID to revoke
        pod: String,
    },
    /// Logout and revoke all pods
    Logout,
    /// Print connection info for use in other tools
    Env {
        /// Pod ID (defaults to first connected pod)
        #[arg(short, long)]
        pod: Option<String>,
        /// Output as shell export statements
        #[arg(long)]
        export: bool,
    },
    /// Inject Tytus integration files into a project directory.
    /// Drops CLAUDE.md context, MCP config, custom commands, and AGENTS.md
    /// so any AI CLI can natively manage your private pod.
    Infect {
        /// Target project directory (defaults to current dir)
        #[arg(default_value = ".")]
        dir: String,
        /// Which integrations to inject (default: all)
        #[arg(short, long, value_delimiter = ',')]
        only: Option<Vec<String>>,
    },
    /// Print MCP server configuration for your AI CLI
    Mcp {
        /// Output format: claude, kilocode, opencode, archon, json
        #[arg(short, long, default_value = "claude")]
        format: String,
    },
    /// Run a command inside your pod's agent container
    Exec {
        /// Command to run (e.g. "openclaw config set gateway.port 3000")
        #[arg(trailing_var_arg = true, required = true)]
        command: Vec<String>,
        /// Pod ID (defaults to first pod)
        #[arg(short, long)]
        pod: Option<String>,
        /// Timeout in seconds (default: 30, max: 120)
        #[arg(short, long, default_value = "30")]
        timeout: u32,
    },
    /// Run diagnostics: check auth, tunnel, gateway connectivity
    Doctor,
    /// (internal) Activate tunnel from a temp config file — called by elevated helper
    #[command(hide = true)]
    TunnelUp {
        /// Path to temp JSON file with tunnel config
        config_file: String,
    },
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn,tytus=info".into()),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();
    let http = atomek_core::HttpClient::new();

    match cli.command {
        None => cmd_default(&http, cli.json).await,
        Some(Commands::Setup) => cmd_setup(&http, cli.json).await,
        Some(Commands::Test) => cmd_test(&http, cli.json).await,
        Some(Commands::Chat { model }) => cmd_chat(&http, &model, cli.json).await,
        Some(Commands::Configure) => cmd_configure(&http, cli.json).await,
        Some(Commands::Login) => cmd_login(&http, cli.json).await,
        Some(Commands::Status) => cmd_status(&http, cli.json).await,
        Some(Commands::Connect { pod, agent }) => cmd_connect(&http, pod, agent.as_str(), cli.json).await,
        Some(Commands::Disconnect { pod }) => cmd_disconnect(pod, cli.json).await,
        Some(Commands::Revoke { pod }) => cmd_revoke(&http, &pod, cli.json).await,
        Some(Commands::Logout) => cmd_logout(&http, cli.json).await,
        Some(Commands::Env { pod, export }) => cmd_env(pod, export, cli.json),
        Some(Commands::Infect { dir, only }) => cmd_infect(&dir, only, cli.json),
        Some(Commands::Mcp { format }) => cmd_mcp(&format, cli.json),
        Some(Commands::Exec { command, pod, timeout }) => cmd_exec(&http, command, pod, timeout, cli.json).await,
        Some(Commands::Doctor) => cmd_doctor(&http, cli.json).await,
        // Hidden subcommand: called by elevated helper to activate tunnel from a temp config file
        Some(Commands::TunnelUp { config_file }) => cmd_tunnel_up(&config_file, cli.json).await,
    }
}

fn shell_escape(s: &str) -> String {
    if s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '/' || c == '.') {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

// ── Login ────────────────────────────────────────────────────

async fn cmd_login(http: &atomek_core::HttpClient, json: bool) {
    let mut state = CliState::load();
    let original_email = state.email.clone();

    // Try auto-login with stored refresh token
    if let Some(ref rt) = state.refresh_token.clone() {
        match atomek_auth::refresh_access_token(http, rt).await {
            Ok(result) => {
                update_tokens(&mut state, &result, &original_email);
                sync_tytus(&mut state, http).await;
                state.save();
                if json { print_json_status(&state); }
                else { println!("✓ Logged in as {}", state.email.as_deref().unwrap_or("?")); }
                return;
            }
            Err(_) => {
                if !json { eprintln!("Stored token expired. Starting fresh login..."); }
            }
        }
    }

    // Device auth flow
    let session = match atomek_auth::create_device_session(http).await {
        Ok(s) => s,
        Err(e) => { eprintln!("Failed to start login: {}", e); std::process::exit(1); }
    };

    if !json {
        println!("Opening browser for authentication...");
        println!("If it doesn't open, visit: {}", session.verification_uri);
        println!("Code: {}", session.user_code);
    }
    let _ = open::that(&session.verification_uri);

    let result = match atomek_auth::poll_for_authorization(http, &session.device_id, |s| {
        if !json && !s.contains("pending") { eprintln!("{}", s); }
    }).await {
        Ok(r) => r,
        Err(e) => { eprintln!("Login failed: {}", e); std::process::exit(1); }
    };

    state.email = Some(result.user.email.clone());
    state.refresh_token = Some(result.refresh_token.clone());
    let email_clone = state.email.clone();
    update_tokens(&mut state, &result, &email_clone);

    // Store in keychain too (for cross-tool compatibility)
    let _ = atomek_auth::KeychainStore::store_refresh_token(&result.user.email, &result.refresh_token);
    let _ = atomek_auth::KeychainStore::store_last_email(&result.user.email);

    sync_tytus(&mut state, http).await;
    state.save();

    if json {
        print_json_status(&state);
    } else {
        println!("✓ Logged in as {}", result.user.email);
        if let Some(ref tier) = state.tier {
            println!("  Plan: {}", tier);
        }
    }
}

// ── Status ───────────────────────────────────────────────────

async fn cmd_status(http: &atomek_core::HttpClient, json: bool) {
    let mut state = CliState::load();

    if !state.is_logged_in() {
        if json { println!(r#"{{"logged_in":false}}"#); }
        else { println!("Not logged in. Run: tytus login"); }
        return;
    }

    ensure_token(&mut state, http).await;
    sync_tytus(&mut state, http).await;
    state.save();

    if json { print_json_status(&state); }
    else { print_human_status(&state); }
}

// ── Connect ──────────────────────────────────────────────────

async fn cmd_connect(http: &atomek_core::HttpClient, pod_id: Option<String>, agent: &str, json: bool) {
    let mut state = CliState::load();

    if !state.is_logged_in() {
        eprintln!("Not logged in. Run: tytus login");
        std::process::exit(1);
    }

    // Validate pod_id if given
    if let Some(ref pid) = pod_id {
        if pid.is_empty() {
            eprintln!("Pod ID cannot be empty");
            std::process::exit(1);
        }
    }

    // ── Phase 1: API calls (no root needed) ──
    ensure_token(&mut state, http).await;
    let (sk, auid) = get_credentials(&mut state, http).await;
    let client = atomek_pods::TytusClient::new(http, &sk, &auid);
    let target_pod_id: String;

    if let Some(ref pid) = pod_id {
        target_pod_id = pid.clone();
        if !json { eprintln!("Connecting to pod {}...", pid); }
    } else if let Some(existing) = state.pods.first() {
        // Reuse existing pod — keeps the IP stable (important for users who
        // configure the gateway URL in their tools)
        target_pod_id = existing.pod_id.clone();
        if !json { eprintln!("Reconnecting to pod {}...", target_pod_id); }
    } else {
        if !json { eprintln!("Allocating {} pod...", agent); }
        match atomek_pods::request_pod_with_agent(&client, agent).await {
            Ok(a) => {
                target_pod_id = a.pod_id.clone();
                state.pods.retain(|p| p.pod_id != a.pod_id);
                state.pods.push(PodEntry {
                    pod_id: a.pod_id.clone(),
                    droplet_id: a.droplet_id.clone(),
                    droplet_ip: a.droplet_ip.clone(),
                    ai_endpoint: a.ai_endpoint.clone(),
                    pod_api_key: a.pod_api_key.clone(),
                    agent_type: a.agent_type.clone(),
                    agent_endpoint: a.agent_endpoint.clone(),
                    tunnel_iface: None,
                });
                state.save();
                if !json { eprintln!("✓ Pod {} allocated", a.pod_id); }
            }
            Err(e) => {
                state.save();
                eprintln!("Allocation failed: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Download WireGuard config
    if !json { eprintln!("Downloading tunnel config..."); }
    let wg_config = match atomek_pods::download_config_for_pod(&client, &target_pod_id).await {
        Ok(c) => c,
        Err(e) => {
            state.save();
            eprintln!("Config download failed: {}", e);
            std::process::exit(1);
        }
    };

    // Fill in endpoints from WG config if not already set by allocation
    let parts: Vec<&str> = wg_config.address.split('/').next().unwrap_or("").split('.').collect();
    let ai_endpoint = if parts.len() == 4 {
        Some(format!("http://{}.{}.{}.1:18080", parts[0], parts[1], parts[2]))
    } else { None };

    if let Some(pod) = state.pods.iter_mut().find(|p| p.pod_id == target_pod_id) {
        if pod.ai_endpoint.is_none() { pod.ai_endpoint = ai_endpoint.clone(); }
        if pod.droplet_ip.is_none() {
            pod.droplet_ip = Some(wg_config.endpoint.split(':').next().unwrap_or("").to_string());
        }
        if pod.agent_endpoint.is_none() {
            if let (Some(ref ep), Some(ref at)) = (&pod.ai_endpoint, &pod.agent_type) {
                let port = if at == "hermes" { 8642 } else { 3000 };
                pod.agent_endpoint = ep.strip_suffix(":18080").map(|b| format!("{}:{}", b, port));
            }
        }
    }
    state.save();

    // ── Phase 2: Tunnel activation (needs root for TUN device) ──
    if !json { eprintln!("Activating WireGuard tunnel..."); }

    let is_root = unsafe { libc::geteuid() == 0 };

    if is_root {
        // Already root — activate directly
        activate_tunnel_inline(&mut state, &target_pod_id, &wg_config, json).await;
    } else {
        // Not root — write config to temp file and elevate only the tunnel-up step
        activate_tunnel_elevated(&mut state, &target_pod_id, &wg_config, json).await;
    }
}

/// Activate tunnel directly (when already running as root).
/// This path is used when the user explicitly runs as root. Same daemon behavior.
async fn activate_tunnel_inline(
    state: &mut CliState,
    target_pod_id: &str,
    wg_config: &atomek_pods::WireGuardConfig,
    json: bool,
) {
    let tunnel_config = atomek_tunnel::TunnelConfig {
        private_key: wg_config.private_key.clone(),
        address: wg_config.address.clone(),
        dns: wg_config.dns.clone(),
        peer_public_key: wg_config.public_key.clone(),
        preshared_key: wg_config.preshared_key.clone(),
        endpoint: wg_config.endpoint.clone(),
        allowed_ips: wg_config.allowed_ips.clone(),
        persistent_keepalive: wg_config.persistent_keepalive,
    };

    match atomek_tunnel::connect(tunnel_config).await {
        Ok(handle) => {
            let iface = handle.interface_name.clone();

            // Write PID + iface files (same as tunnel-up daemon path)
            let pid_dir = std::path::PathBuf::from("/tmp/tytus");
            std::fs::create_dir_all(&pid_dir).ok();
            let _ = std::fs::write(pid_dir.join(format!("tunnel-{}.pid", target_pod_id)), format!("{}", std::process::id()));
            let _ = std::fs::write(pid_dir.join(format!("tunnel-{}.iface", target_pod_id)), &iface);

            if let Some(pod) = state.pods.iter_mut().find(|p| p.pod_id == target_pod_id) {
                pod.tunnel_iface = Some(iface.clone());
            }
            state.save();

            if json {
                let pod = state.pods.iter().find(|p| p.pod_id == target_pod_id);
                println!("{}", serde_json::to_string_pretty(&pod).unwrap_or_default());
            } else {
                eprintln!("✓ Tunnel active on {}", iface);
                if let Some(pod) = state.pods.iter().find(|p| p.pod_id == target_pod_id) {
                    if let Some(ref ep) = pod.ai_endpoint {
                        println!("AI_GATEWAY={}", ep);
                    }
                    if let Some(ref ep) = pod.agent_endpoint {
                        println!("AGENT_API={}", ep);
                    }
                    if let Some(ref key) = pod.pod_api_key {
                        println!("API_KEY={}", key);
                    }
                }
                eprintln!("Tunnel daemon running (pid {}). Stop with: tytus disconnect", std::process::id());
            }

            // Block until signal — this process IS the daemon
            tokio::signal::ctrl_c().await.ok();
            handle.shutdown().await;

            let _ = std::fs::remove_file(pid_dir.join(format!("tunnel-{}.pid", target_pod_id)));
            let _ = std::fs::remove_file(pid_dir.join(format!("tunnel-{}.iface", target_pod_id)));
            if let Some(pod) = state.pods.iter_mut().find(|p| p.pod_id == target_pod_id) {
                pod.tunnel_iface = None;
            }
            state.save();
        }
        Err(e) => {
            state.save();
            eprintln!("Tunnel failed: {}", e);
            std::process::exit(1);
        }
    }
}

/// Write tunnel config to a temp file and elevate only `tytus tunnel-up <file>`
async fn activate_tunnel_elevated(
    state: &mut CliState,
    target_pod_id: &str,
    wg_config: &atomek_pods::WireGuardConfig,
    json: bool,
) {
    // Serialize tunnel config to temp file (will be read by elevated process)
    let tunnel_data = serde_json::json!({
        "private_key": wg_config.private_key,
        "address": wg_config.address,
        "dns": wg_config.dns,
        "peer_public_key": wg_config.public_key,
        "preshared_key": wg_config.preshared_key,
        "endpoint": wg_config.endpoint,
        "allowed_ips": wg_config.allowed_ips,
        "persistent_keepalive": wg_config.persistent_keepalive,
        "pod_id": target_pod_id,
    });

    let tmp_dir = std::env::temp_dir().join("tytus");
    std::fs::create_dir_all(&tmp_dir).ok();
    let config_path = tmp_dir.join(format!("tunnel-{}.json", target_pod_id));
    if let Err(e) = std::fs::write(&config_path, serde_json::to_string(&tunnel_data).unwrap()) {
        eprintln!("Failed to write tunnel config: {}", e);
        std::process::exit(1);
    }
    // Restrict permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600));
    }

    let exe = std::env::current_exe().unwrap_or_else(|_| {
        eprintln!("Cannot determine executable path");
        std::process::exit(1);
    });
    let exe_str = exe.display().to_string();
    let config_path_str = config_path.display().to_string();
    let json_flag = if json { " --json" } else { "" };

    let full_args = vec!["tunnel-up".to_string(), config_path_str.clone()];

    // Spawn tunnel-up as a detached background process with elevated privileges.
    // The subprocess writes PID + interface name to /tmp/tytus/ and prints TUNNEL_READY to stdout.
    // We capture stdout to detect when the tunnel is up, then return immediately.
    let child = try_spawn_elevated(&exe_str, &full_args, &config_path_str, json_flag);

    let mut child = match child {
        Ok(c) => c,
        Err(e) => {
            let _ = std::fs::remove_file(&config_path);
            eprintln!("Failed to start tunnel: {}", e);
            std::process::exit(1);
        }
    };

    // Wait for tunnel to signal readiness (reads stdout for "TUNNEL_READY")
    // Timeout after 15 seconds
    let stdout = child.stdout.take();
    let mut iface_name = None;
    let mut tunnel_pid = None;

    if let Some(stdout) = stdout {
        use std::io::BufRead;
        let reader = std::io::BufReader::new(stdout);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);

        for line in reader.lines() {
            if std::time::Instant::now() > deadline { break; }
            match line {
                Ok(l) if l.starts_with("TUNNEL_READY") => {
                    // Parse: TUNNEL_READY iface=utunX pid=12345
                    for part in l.split_whitespace() {
                        if let Some(v) = part.strip_prefix("iface=") { iface_name = Some(v.to_string()); }
                        if let Some(v) = part.strip_prefix("pid=") { tunnel_pid = v.parse::<u32>().ok(); }
                    }
                    break;
                }
                Ok(_) => continue,
                Err(_) => break,
            }
        }
    }

    // Clean up temp file (tunnel-up also removes it, but be safe)
    let _ = std::fs::remove_file(&config_path);

    if let Some(ref iface) = iface_name {
        if let Some(pod) = state.pods.iter_mut().find(|p| p.pod_id == target_pod_id) {
            pod.tunnel_iface = Some(iface.clone());
        }
        state.save();

        if json {
            let pod = state.pods.iter().find(|p| p.pod_id == target_pod_id);
            println!("{}", serde_json::to_string_pretty(&pod).unwrap_or_default());
        } else {
            eprintln!("✓ Tunnel active on {}", iface);
            if let Some(pod) = state.pods.iter().find(|p| p.pod_id == target_pod_id) {
                if let Some(ref ep) = pod.ai_endpoint {
                    println!("AI_GATEWAY={}", ep);
                }
                if let Some(ref ep) = pod.agent_endpoint {
                    println!("AGENT_API={}", ep);
                }
                if let Some(ref key) = pod.pod_api_key {
                    println!("API_KEY={}", key);
                }
            }
            if let Some(pid) = tunnel_pid {
                eprintln!("Tunnel daemon running (pid {}). Stop with: tytus disconnect", pid);
            }
        }
    } else {
        // Tunnel didn't signal readiness — check if the child exited with error
        let exit = child.try_wait().ok().flatten();
        if let Some(status) = exit {
            eprintln!("Tunnel failed (exit {}).", status.code().unwrap_or(1));
        } else {
            eprintln!("Tunnel did not start within 15 seconds.");
            let _ = child.kill();
        }
        std::process::exit(1);
    }
}

/// Try to spawn `tytus tunnel-up` with elevated privileges as a detached background process.
fn try_spawn_elevated(
    exe: &str,
    args: &[String],
    config_path: &str,
    json_flag: &str,
) -> Result<std::process::Child, String> {
    // Strategy 1: sudo -n (passwordless, works with sudoers entry)
    if let Ok(child) = std::process::Command::new("sudo")
        .arg("-n")
        .arg(exe)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped()) // capture TUNNEL_READY
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        return Ok(child);
    }

    // Strategy 2: osascript on macOS (GUI password dialog)
    #[cfg(target_os = "macos")]
    {
        let cmd = format!(
            "{} tunnel-up {}{}",
            shell_escape(exe),
            shell_escape(config_path),
            json_flag,
        );
        if let Ok(child) = std::process::Command::new("osascript")
            .args(["-e", &format!(
                "do shell script \"{}\" with administrator privileges",
                cmd.replace('\\', "\\\\").replace('"', "\\\"")
            )])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            return Ok(child);
        }
    }

    // Strategy 3: interactive sudo (terminal required)
    std::process::Command::new("sudo")
        .arg(exe)
        .args(args)
        .stdin(std::process::Stdio::inherit()) // needs terminal for password
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("All elevation strategies failed: {}", e))
}

/// Hidden subcommand: runs as root, reads tunnel config from temp file, activates tunnel.
/// Runs as a background daemon — writes PID file, detaches from terminal, handles SIGTERM.
async fn cmd_tunnel_up(config_file: &str, _json: bool) {
    let data = match std::fs::read_to_string(config_file) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to read tunnel config: {}", e);
            std::process::exit(1);
        }
    };
    let v: serde_json::Value = match serde_json::from_str(&data) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Invalid tunnel config: {}", e);
            std::process::exit(1);
        }
    };

    // Clean up the temp file immediately (contains private key)
    let _ = std::fs::remove_file(config_file);

    let pod_id = v["pod_id"].as_str().unwrap_or("00").to_string();

    let tunnel_config = atomek_tunnel::TunnelConfig {
        private_key: v["private_key"].as_str().unwrap_or("").to_string(),
        address: v["address"].as_str().unwrap_or("").to_string(),
        dns: v["dns"].as_str().map(|s| s.to_string()),
        peer_public_key: v["peer_public_key"].as_str().unwrap_or("").to_string(),
        preshared_key: v["preshared_key"].as_str().map(|s| s.to_string()),
        endpoint: v["endpoint"].as_str().unwrap_or("").to_string(),
        allowed_ips: v["allowed_ips"].as_str().unwrap_or("").to_string(),
        persistent_keepalive: v["persistent_keepalive"].as_u64().map(|n| n as u16),
    };

    match atomek_tunnel::connect(tunnel_config).await {
        Ok(handle) => {
            let iface = handle.interface_name.clone();

            // Write PID file so `tytus disconnect` can find and stop us
            let pid_dir = std::path::PathBuf::from("/tmp/tytus");
            std::fs::create_dir_all(&pid_dir).ok();
            let pid_file = pid_dir.join(format!("tunnel-{}.pid", pod_id));
            let _ = std::fs::write(&pid_file, format!("{}", std::process::id()));

            // Write interface name so parent process can read it
            let iface_file = pid_dir.join(format!("tunnel-{}.iface", pod_id));
            let _ = std::fs::write(&iface_file, &iface);

            // Signal to parent that tunnel is ready (print to stdout for capture)
            println!("TUNNEL_READY iface={} pid={}", iface, std::process::id());

            // Wait for SIGTERM (from `tytus disconnect`) or SIGINT (Ctrl+C)
            tokio::signal::ctrl_c().await.ok();
            handle.shutdown().await;

            // Clean up PID + iface files
            let _ = std::fs::remove_file(&pid_file);
            let _ = std::fs::remove_file(&iface_file);
        }
        Err(e) => {
            eprintln!("Tunnel failed: {}", e);
            std::process::exit(1);
        }
    }
}

// ── Revoke ───────────────────────────────────────────────────

async fn cmd_revoke(http: &atomek_core::HttpClient, pod_id: &str, json: bool) {
    let mut state = CliState::load();

    if !state.is_logged_in() {
        eprintln!("Not logged in. Run: tytus login");
        std::process::exit(1);
    }

    ensure_token(&mut state, http).await;
    let (sk, auid) = get_credentials(&mut state, http).await;
    let client = atomek_pods::TytusClient::new(http, &sk, &auid);

    match atomek_pods::revoke_pod(&client, pod_id).await {
        Ok(_) => {
            state.pods.retain(|p| p.pod_id != pod_id);
            state.save();
            if json { println!(r#"{{"status":"revoked","pod_id":"{}"}}"#, pod_id); }
            else { println!("✓ Pod {} revoked", pod_id); }
        }
        Err(e) => {
            eprintln!("Revoke failed: {}", e);
            std::process::exit(1);
        }
    }
}

// ── Disconnect ───────────────────────────────────────────────

async fn cmd_disconnect(pod_id: Option<String>, json: bool) {
    let mut state = CliState::load();
    let mut killed = 0u32;

    let pod_ids: Vec<String> = if let Some(ref pid) = pod_id {
        vec![pid.clone()]
    } else {
        state.pods.iter().map(|p| p.pod_id.clone()).collect()
    };

    for pid in &pod_ids {
        // Kill the tunnel daemon via PID file
        let pid_file = std::path::PathBuf::from(format!("/tmp/tytus/tunnel-{}.pid", pid));
        if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
            if let Ok(tunnel_pid) = pid_str.trim().parse::<i32>() {
                // Tunnel runs as root — use sudo -n to send SIGTERM
                let kill_ok = std::process::Command::new("sudo")
                    .args(["-n", "kill", "-TERM", &tunnel_pid.to_string()])
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);

                if kill_ok {
                    killed += 1;
                    if !json { eprintln!("Stopped tunnel for pod {} (pid {})", pid, tunnel_pid); }
                } else {
                    // Maybe process already dead, or sudo not available
                    let is_alive = unsafe { libc::kill(tunnel_pid, 0) } == 0;
                    if !is_alive {
                        if !json { eprintln!("Tunnel for pod {} already stopped", pid); }
                    } else {
                        eprintln!("Could not stop tunnel pid {}. Run: sudo kill {}", tunnel_pid, tunnel_pid);
                    }
                }
            }
            let _ = std::fs::remove_file(&pid_file);
        }
        let iface_file = std::path::PathBuf::from(format!("/tmp/tytus/tunnel-{}.iface", pid));
        let _ = std::fs::remove_file(&iface_file);

        // Clear state
        if let Some(pod) = state.pods.iter_mut().find(|p| p.pod_id == *pid) {
            pod.tunnel_iface = None;
        }
    }
    state.save();

    if json {
        println!(r#"{{"status":"disconnected","tunnels_stopped":{}}}"#, killed);
    } else if killed > 0 {
        println!("✓ {} tunnel(s) stopped", killed);
    } else {
        println!("✓ Tunnel state cleared (no active daemons found)");
    }
}

// ── Exec ────────────────────────────────────────────────────

async fn cmd_exec(http: &atomek_core::HttpClient, command: Vec<String>, pod_id: Option<String>, timeout: u32, json: bool) {
    let mut state = CliState::load();

    if !state.is_logged_in() {
        eprintln!("Not logged in. Run: tytus login");
        std::process::exit(1);
    }

    ensure_token(&mut state, http).await;
    let (sk, auid) = get_credentials(&mut state, http).await;
    let client = atomek_pods::TytusClient::new(http, &sk, &auid);

    let target_pod_id = pod_id.unwrap_or_else(|| {
        state.pods.first().map(|p| p.pod_id.clone()).unwrap_or_else(|| {
            eprintln!("No pods. Run: tytus connect");
            std::process::exit(1);
        })
    });

    let cmd_str = command.join(" ");
    if !json { eprintln!("Running on pod {}...", target_pod_id); }

    match atomek_pods::exec_in_agent(&client, &target_pod_id, &cmd_str, timeout.min(120)).await {
        Ok(result) => {
            if json {
                println!("{}", serde_json::json!({
                    "exit_code": result.exit_code,
                    "stdout": result.stdout,
                    "stderr": result.stderr,
                }));
            } else {
                if let Some(ref stdout) = result.stdout {
                    if !stdout.is_empty() { print!("{}", stdout); }
                }
                if let Some(ref stderr) = result.stderr {
                    if !stderr.is_empty() { eprint!("{}", stderr); }
                }
                if result.exit_code != 0 {
                    std::process::exit(result.exit_code as i32);
                }
            }
        }
        Err(e) => {
            eprintln!("Exec failed: {}", e);
            std::process::exit(1);
        }
    }
}

// ── Logout ───────────────────────────────────────────────────

async fn cmd_logout(http: &atomek_core::HttpClient, json: bool) {
    let mut state = CliState::load();

    if state.is_logged_in() {
        if let (Some(ref sk), Some(ref auid)) = (&state.secret_key, &state.agent_user_id) {
            let client = atomek_pods::TytusClient::new(http, sk, auid);
            if let Err(e) = atomek_pods::revoke_all_pods(&client).await {
                tracing::warn!("Revoke failed: {}", e);
            }
        }
        if let Some(ref email) = state.email {
            let _ = atomek_auth::KeychainStore::delete_refresh_token(email);
        }
    }

    state.clear();

    if json { println!(r#"{{"status":"logged_out"}}"#); }
    else { println!("✓ Logged out"); }
}

// ── Env (export connection info) ─────────────────────────────

fn cmd_env(pod_id: Option<String>, export: bool, json: bool) {
    let state = CliState::load();

    let pod = if let Some(ref pid) = pod_id {
        state.pods.iter().find(|p| p.pod_id == *pid)
    } else {
        // First connected pod, or first pod
        state.pods.iter().find(|p| p.tunnel_iface.is_some())
            .or_else(|| state.pods.first())
    };

    let Some(pod) = pod else {
        if json { println!(r#"{{"error":"no_pods"}}"#); }
        else { eprintln!("No pods. Run: tytus connect"); }
        std::process::exit(1);
    };

    if json {
        println!("{}", serde_json::to_string_pretty(pod).unwrap_or_default());
        return;
    }

    let prefix = if export { "export " } else { "" };

    if let Some(ref ep) = pod.ai_endpoint {
        println!("{}TYTUS_AI_GATEWAY={}", prefix, ep);
    }
    if let Some(ref ep) = pod.agent_endpoint {
        println!("{}TYTUS_AGENT_API={}", prefix, ep);
    }
    if let Some(ref key) = pod.pod_api_key {
        println!("{}TYTUS_API_KEY={}", prefix, key);
    }
    if let Some(ref at) = pod.agent_type {
        println!("{}TYTUS_AGENT_TYPE={}", prefix, at);
    }
    println!("{}TYTUS_POD_ID={}", prefix, pod.pod_id);
}

// ── Infect (drop integration files) ─────────────────────────

fn cmd_infect(dir: &str, only: Option<Vec<String>>, json: bool) {
    let base = std::path::Path::new(dir).canonicalize().unwrap_or_else(|_| {
        eprintln!("Directory not found: {}", dir);
        std::process::exit(1);
    });

    let tytus_bin = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("tytus-mcp").display().to_string()))
        .unwrap_or_else(|| "tytus-mcp".into());

    let should_inject = |name: &str| -> bool {
        only.as_ref().map_or(true, |list| list.iter().any(|s| s == name))
    };

    let mut injected = Vec::new();

    // 1. Claude Code: CLAUDE.md context + .claude/commands/ + .mcp.json
    if should_inject("claude") {
        // Append to existing CLAUDE.md or create new one
        let claude_md = base.join("CLAUDE.md");
        let tytus_block = CLAUDE_MD_BLOCK;
        if claude_md.exists() {
            let existing = std::fs::read_to_string(&claude_md).unwrap_or_default();
            if !existing.contains("## Tytus Private AI Pod") {
                let _ = std::fs::write(&claude_md, format!("{}\n\n{}", existing.trim(), tytus_block));
                injected.push("CLAUDE.md (appended)");
            } else {
                injected.push("CLAUDE.md (already present)");
            }
        } else {
            let _ = std::fs::write(&claude_md, tytus_block);
            injected.push("CLAUDE.md (created)");
        }

        // .claude/commands/tytus.md
        let cmd_dir = base.join(".claude").join("commands");
        let _ = std::fs::create_dir_all(&cmd_dir);
        let _ = std::fs::write(cmd_dir.join("tytus.md"), CLAUDE_COMMAND_TYTUS);
        injected.push(".claude/commands/tytus.md");

        // .mcp.json for Claude Code MCP
        let mcp_json = base.join(".mcp.json");
        let mcp_config = serde_json::json!({
            "mcpServers": {
                "tytus": {
                    "command": tytus_bin,
                    "args": [],
                    "alwaysAllow": [
                        "tytus_status",
                        "tytus_env",
                        "tytus_models",
                        "tytus_setup_guide"
                    ]
                }
            }
        });
        if mcp_json.exists() {
            // Merge into existing .mcp.json
            let existing = std::fs::read_to_string(&mcp_json).unwrap_or_default();
            if let Ok(mut existing_val) = serde_json::from_str::<serde_json::Value>(&existing) {
                if let Some(servers) = existing_val.get_mut("mcpServers").and_then(|s| s.as_object_mut()) {
                    if !servers.contains_key("tytus") {
                        servers.insert("tytus".into(), mcp_config["mcpServers"]["tytus"].clone());
                        let _ = std::fs::write(&mcp_json, serde_json::to_string_pretty(&existing_val).unwrap());
                        injected.push(".mcp.json (merged)");
                    } else {
                        injected.push(".mcp.json (tytus already present)");
                    }
                }
            }
        } else {
            let _ = std::fs::write(&mcp_json, serde_json::to_string_pretty(&mcp_config).unwrap());
            injected.push(".mcp.json (created)");
        }
    }

    // 2. AGENTS.md (Codex, Gemini, generic agents)
    if should_inject("agents") {
        let agents_md = base.join("AGENTS.md");
        let tytus_block = AGENTS_MD_BLOCK;
        if agents_md.exists() {
            let existing = std::fs::read_to_string(&agents_md).unwrap_or_default();
            if !existing.contains("## Tytus Private AI Pod") {
                let _ = std::fs::write(&agents_md, format!("{}\n\n{}", existing.trim(), tytus_block));
                injected.push("AGENTS.md (appended)");
            } else {
                injected.push("AGENTS.md (already present)");
            }
        } else {
            let _ = std::fs::write(&agents_md, tytus_block);
            injected.push("AGENTS.md (created)");
        }
    }

    // 3. Kilocode / OpenCode: .kilo/command/*.md
    if should_inject("kilocode") || should_inject("opencode") {
        let kilo_dir = base.join(".kilo").join("command");
        let _ = std::fs::create_dir_all(&kilo_dir);
        let _ = std::fs::write(kilo_dir.join("tytus.md"), KILO_COMMAND_TYTUS);
        injected.push(".kilo/command/tytus.md");

        // Also .kilo/mcp.json
        let kilo_mcp = base.join(".kilo").join("mcp.json");
        let mcp_config = serde_json::json!({
            "mcpServers": {
                "tytus": {
                    "command": tytus_bin,
                    "args": []
                }
            }
        });
        let _ = std::fs::write(&kilo_mcp, serde_json::to_string_pretty(&mcp_config).unwrap());
        injected.push(".kilo/mcp.json");
    }

    // 4. Archon: .archon/commands/tytus.md
    if should_inject("archon") {
        let archon_dir = base.join(".archon").join("commands");
        let _ = std::fs::create_dir_all(&archon_dir);
        let _ = std::fs::write(archon_dir.join("tytus.md"), ARCHON_COMMAND_TYTUS);
        injected.push(".archon/commands/tytus.md");
    }

    // 5. Shell env hook
    if should_inject("shell") {
        let shell_hook = base.join(".tytus-env.sh");
        let _ = std::fs::write(&shell_hook, SHELL_ENV_HOOK);
        injected.push(".tytus-env.sh");
    }

    if json {
        println!("{}", serde_json::json!({
            "status": "infected",
            "directory": base.display().to_string(),
            "files": injected,
        }));
    } else {
        println!("Tytus integration injected into {}", base.display());
        for file in &injected {
            println!("  + {}", file);
        }
        println!("\nAI CLIs in this directory now have native Tytus access.");
        println!("Run `tytus mcp` to see MCP server configuration.");
    }
}

// ── MCP config printer ─────────────────────────────────────

fn cmd_mcp(format: &str, json: bool) {
    let tytus_mcp = which_tytus_mcp();

    match format {
        "claude" => {
            let config = serde_json::json!({
                "mcpServers": {
                    "tytus": {
                        "command": tytus_mcp,
                        "args": [],
                        "alwaysAllow": [
                            "tytus_status",
                            "tytus_env",
                            "tytus_models",
                            "tytus_setup_guide"
                        ]
                    }
                }
            });
            if json {
                println!("{}", serde_json::to_string_pretty(&config).unwrap());
            } else {
                println!("Add to .mcp.json or ~/.claude/settings.json:\n");
                println!("{}", serde_json::to_string_pretty(&config).unwrap());
            }
        }
        "kilocode" | "opencode" | "kilo" => {
            let config = serde_json::json!({
                "mcpServers": {
                    "tytus": {
                        "command": tytus_mcp,
                        "args": []
                    }
                }
            });
            if json {
                println!("{}", serde_json::to_string_pretty(&config).unwrap());
            } else {
                println!("Add to .kilo/mcp.json or .kilocode/mcp.json:\n");
                println!("{}", serde_json::to_string_pretty(&config).unwrap());
            }
        }
        "archon" => {
            let config = serde_json::json!({
                "tytus": {
                    "command": tytus_mcp,
                    "args": []
                }
            });
            if json {
                println!("{}", serde_json::to_string_pretty(&config).unwrap());
            } else {
                println!("Add to .archon/mcp/<name>.json:\n");
                println!("{}", serde_json::to_string_pretty(&config).unwrap());
            }
        }
        _ => {
            let config = serde_json::json!({
                "server": "tytus",
                "transport": "stdio",
                "command": tytus_mcp,
                "args": [],
                "tools": [
                    "tytus_status",
                    "tytus_env",
                    "tytus_models",
                    "tytus_chat",
                    "tytus_revoke",
                    "tytus_setup_guide"
                ]
            });
            println!("{}", serde_json::to_string_pretty(&config).unwrap());
        }
    }
}

fn which_tytus_mcp() -> String {
    // Check common locations
    for path in &[
        "/usr/local/bin/tytus-mcp",
        "/opt/homebrew/bin/tytus-mcp",
    ] {
        if std::path::Path::new(path).exists() {
            return path.to_string();
        }
    }
    // Fallback: same dir as tytus binary
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("tytus-mcp").display().to_string()))
        .unwrap_or_else(|| "tytus-mcp".into())
}

// ── Integration file templates ──────────────────────────────

// ── Default view (first-run detection + dashboard) ──────────

async fn cmd_default(http: &atomek_core::HttpClient, json: bool) {
    let state = CliState::load();

    // First run: not logged in
    if !state.is_logged_in() {
        wizard::clear();
        wizard::print_logo();
        wizard::type_out("   Welcome! Let's get you set up in 60 seconds.");
        println!();
        wizard::print_info("Tytus gives you a private, encrypted AI pod — your own OpenAI-compatible gateway.");
        println!();

        if wizard::is_interactive() {
            match wizard::confirm("Ready to set up your Tytus pod?", true) {
                Ok(true) => {
                    cmd_setup(http, json).await;
                    return;
                }
                _ => {
                    println!();
                    wizard::print_hint("When you're ready, run: tytus setup");
                    return;
                }
            }
        } else {
            wizard::print_hint("Run: tytus setup");
            return;
        }
    }

    // Returning user: show dashboard
    show_dashboard(http, &state, json).await;
}

async fn show_dashboard(http: &atomek_core::HttpClient, _state: &CliState, _json: bool) {
    // Refresh state from server
    let mut state = CliState::load();
    ensure_token(&mut state, http).await;
    sync_tytus(&mut state, http).await;
    state.save();

    wizard::print_logo();

    let email = state.email.as_deref().unwrap_or("?");
    let tier = state.tier.as_deref().unwrap_or("free");

    println!("  {} Signed in as {}", wizard::icon_ok(), console::style(email).bold());
    println!("  {} Plan: {}", wizard::icon_info(), console::style(tier).cyan().bold());
    println!();

    if state.pods.is_empty() {
        wizard::print_warn("No pods allocated yet.");
        println!();
        wizard::print_hint("Start your pod:  tytus connect");
        return;
    }

    wizard::print_header("Your Pods");
    for pod in &state.pods {
        let agent = pod.agent_type.as_deref().unwrap_or("?");
        let tunnel_active = pod.tunnel_iface.is_some();
        let status_label = if tunnel_active {
            console::style("● CONNECTED").green().bold()
        } else {
            console::style("○ disconnected").dim()
        };
        println!("  Pod {} [{}]  {}", console::style(&pod.pod_id).bold(), agent, status_label);
        if let Some(ref ep) = pod.ai_endpoint {
            println!("    AI Gateway: {}", console::style(ep).cyan());
        }
        if let Some(ref ep) = pod.agent_endpoint {
            println!("    Agent API:  {}", console::style(ep).cyan());
        }
        if let Some(ref iface) = pod.tunnel_iface {
            println!("    Tunnel:     {}", console::style(iface).dim());
        }
        println!();
    }

    wizard::print_header("What would you like to do?");
    let has_tunnel = state.pods.iter().any(|p| p.tunnel_iface.is_some());
    if has_tunnel {
        wizard::print_hint("tytus chat       — Chat with your AI");
        wizard::print_hint("tytus test       — Run a quick health test");
        wizard::print_hint("tytus disconnect — Stop the tunnel");
    } else {
        wizard::print_hint("tytus connect    — Start your tunnel");
        wizard::print_hint("tytus doctor     — Diagnose issues");
    }
    wizard::print_hint("tytus configure  — Configure your agent");
    wizard::print_hint("tytus --help     — See all commands");
    println!();
}

// ── Setup wizard (full first-time setup) ────────────────────

async fn cmd_setup(http: &atomek_core::HttpClient, json: bool) {
    if json {
        eprintln!("Setup wizard is interactive. Use individual commands for scripting.");
        std::process::exit(1);
    }

    wizard::clear();
    wizard::print_logo();
    wizard::type_out("  Let's set up your private AI pod. This takes about 1 minute.");
    println!();

    let total_steps = 5;

    // ── Step 1: Login ──
    wizard::print_step(1, total_steps, "Sign in to Traylinx");
    let mut state = CliState::load();
    if state.is_logged_in() {
        ensure_token(&mut state, http).await;
        wizard::print_ok(&format!("Already signed in as {}", state.email.as_deref().unwrap_or("?")));
    } else {
        println!();
        wizard::print_info("We'll open your browser for a secure login.");
        if !wizard::confirm("Continue?", true).unwrap_or(false) {
            wizard::print_warn("Setup cancelled.");
            return;
        }
        cmd_login(http, false).await;
        state = CliState::load();
        if !state.is_logged_in() {
            wizard::print_fail("Login failed.");
            return;
        }
    }
    println!();

    // ── Step 2: Choose agent type ──
    wizard::print_step(2, total_steps, "Choose your AI agent");
    println!();
    wizard::print_info("NemoClaw — lightweight assistant (1 unit, good for most tasks)");
    wizard::print_info("Hermes   — advanced reasoning agent (2 units, better for complex tasks)");
    println!();

    let agent = if state.pods.is_empty() {
        match wizard::select("Which agent?", &["nemoclaw (recommended)", "hermes"]) {
            Ok(s) if s.starts_with("hermes") => "hermes",
            _ => "nemoclaw",
        }
    } else {
        let first_agent = state.pods[0].agent_type.clone().unwrap_or_else(|| "nemoclaw".to_string());
        wizard::print_ok(&format!("Using existing pod ({})", first_agent));
        // Leak is fine here — agent is used as &str for a single call
        Box::leak(first_agent.into_boxed_str())
    };
    println!();

    // ── Step 3: Allocate pod + activate tunnel ──
    wizard::print_step(3, total_steps, "Allocating your pod and starting tunnel");
    println!();
    cmd_connect(http, None, agent, false).await;
    println!();

    // Re-load state — connect updated it
    let state = CliState::load();

    // ── Step 4: Test the gateway ──
    wizard::print_step(4, total_steps, "Testing the AI gateway");
    println!();
    let pb = wizard::spinner("Running test query...");

    let test_result = if let Some(pod) = state.pods.first() {
        if let (Some(ref endpoint), Some(ref key)) = (&pod.ai_endpoint, &pod.pod_api_key) {
            test_chat_completion(endpoint, key, "ail-compound", "Say hello in 3 words").await
        } else {
            Err("Pod missing endpoint or API key".to_string())
        }
    } else {
        Err("No pod allocated".to_string())
    };

    match test_result {
        Ok(response) => {
            wizard::finish_ok(&pb, "Gateway responded successfully!");
            println!();
            wizard::print_info(&format!("AI said: \"{}\"", response.trim()));
        }
        Err(e) => {
            wizard::finish_fail(&pb, &format!("Test failed: {}", e));
            wizard::print_hint("Run `tytus doctor` to diagnose");
            return;
        }
    }
    println!();

    // ── Step 5: Show integration hints ──
    wizard::print_step(5, total_steps, "Setup complete!");
    println!();
    wizard::print_success_banner("Your Tytus pod is ready to use!");

    if let Some(pod) = state.pods.first() {
        if let (Some(ref ep), Some(ref key)) = (&pod.ai_endpoint, &pod.pod_api_key) {
            wizard::print_box("Your Connection Info", &[
                &format!("API URL: {}", ep),
                &format!("API Key: {}...{}", &key[..10.min(key.len())], &key[key.len().saturating_sub(4)..]),
                "",
                "Compatible with any OpenAI SDK.",
            ]);
        }
    }

    println!();
    wizard::print_header("What's next?");
    wizard::print_hint("tytus chat           — Try chatting with your AI");
    wizard::print_hint("tytus test           — Run a quick health check");
    wizard::print_hint("tytus infect .       — Add Tytus to this project");
    wizard::print_hint("tytus env --export   — Get shell environment vars");
    println!();
}

// ── Test command (quick health check) ───────────────────────

async fn cmd_test(http: &atomek_core::HttpClient, json: bool) {
    let mut state = CliState::load();

    if !state.is_logged_in() {
        if json {
            println!(r#"{{"ok":false,"error":"not_logged_in"}}"#);
        } else {
            wizard::print_fail("Not logged in. Run: tytus setup");
        }
        std::process::exit(1);
    }

    ensure_token(&mut state, http).await;
    sync_tytus(&mut state, http).await;

    if !json { wizard::print_header("Running Tytus health test"); }

    // Check 1: logged in
    let pb = wizard::spinner("Checking authentication");
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    wizard::finish_ok(&pb, &format!("Signed in as {}", state.email.as_deref().unwrap_or("?")));

    // Check 2: has pod
    let pb = wizard::spinner("Checking pod allocation");
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    if state.pods.is_empty() {
        wizard::finish_fail(&pb, "No pod allocated");
        wizard::print_hint("Run: tytus connect");
        std::process::exit(1);
    }
    let pod = &state.pods[0].clone();
    wizard::finish_ok(&pb, &format!("Pod {} allocated", pod.pod_id));

    // Check 3: tunnel active
    let pb = wizard::spinner("Checking tunnel");
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    if pod.tunnel_iface.is_none() {
        wizard::finish_fail(&pb, "Tunnel not running");
        wizard::print_hint("Run: tytus connect");
        std::process::exit(1);
    }
    wizard::finish_ok(&pb, &format!("Tunnel active on {}", pod.tunnel_iface.as_deref().unwrap_or("?")));

    // Check 4: gateway reachable
    let pb = wizard::spinner("Testing AI gateway");
    let endpoint = pod.ai_endpoint.as_deref().unwrap_or("");
    let key = pod.pod_api_key.as_deref().unwrap_or("");

    match test_chat_completion(endpoint, key, "ail-compound", "Say hello").await {
        Ok(response) => {
            wizard::finish_ok(&pb, "Gateway responded!");
            println!();
            wizard::print_info(&format!("AI said: \"{}\"", response.trim()));
            println!();
            if json {
                println!(r#"{{"ok":true}}"#);
            } else {
                wizard::print_success_banner("Everything is working!");
            }
        }
        Err(e) => {
            wizard::finish_fail(&pb, &format!("Gateway failed: {}", e));
            if json {
                println!(r#"{{"ok":false,"error":"gateway_failed"}}"#);
            }
            std::process::exit(1);
        }
    }
}

/// Helper: send a chat completion and return the assistant's response text.
async fn test_chat_completion(endpoint: &str, key: &str, model: &str, prompt: &str) -> Result<String, String> {
    let url = format!("{}/v1/chat/completions", endpoint);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.post(&url)
        .header("Authorization", format!("Bearer {}", key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": 30,
        }))
        .send().await
        .map_err(|e| format!("network error: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let body: serde_json::Value = resp.json().await.map_err(|e| format!("bad JSON: {}", e))?;

    // Extract the content from choices[0].message.content OR reasoning_content (MiniMax style)
    let content = body["choices"][0]["message"]["content"].as_str().unwrap_or("");
    let reasoning = body["choices"][0]["message"]["reasoning_content"].as_str().unwrap_or("");

    let text = if !content.is_empty() {
        content.to_string()
    } else if !reasoning.is_empty() {
        reasoning.to_string()
    } else {
        "(empty response)".to_string()
    };

    Ok(text)
}

// ── Chat command (interactive REPL) ─────────────────────────

async fn cmd_chat(http: &atomek_core::HttpClient, model: &str, json: bool) {
    if json {
        eprintln!("Chat is interactive. Use the API directly for scripting.");
        std::process::exit(1);
    }

    let mut state = CliState::load();
    if !state.is_logged_in() {
        wizard::print_fail("Not logged in. Run: tytus setup");
        std::process::exit(1);
    }
    ensure_token(&mut state, http).await;
    sync_tytus(&mut state, http).await;

    let pod = match state.pods.first() {
        Some(p) if p.tunnel_iface.is_some() => p.clone(),
        Some(_) => {
            wizard::print_fail("Tunnel not running. Run: tytus connect");
            std::process::exit(1);
        }
        None => {
            wizard::print_fail("No pod allocated. Run: tytus setup");
            std::process::exit(1);
        }
    };

    let endpoint = pod.ai_endpoint.as_deref().unwrap_or("");
    let key = pod.pod_api_key.as_deref().unwrap_or("");

    wizard::print_logo();
    wizard::print_header(&format!("Chat — {} (pod {})", model, pod.pod_id));
    wizard::print_info("Type your message and press Enter. Type /quit to exit, /help for commands.");
    println!();

    let mut history: Vec<serde_json::Value> = Vec::new();

    loop {
        let input = match inquire::Text::new(">").prompt() {
            Ok(s) => s,
            Err(_) => break,
        };

        let trimmed = input.trim();
        if trimmed.is_empty() { continue; }

        match trimmed {
            "/quit" | "/exit" | "/q" => break,
            "/help" => {
                wizard::print_info("/quit  — exit chat");
                wizard::print_info("/clear — clear conversation history");
                wizard::print_info("/help  — show this help");
                continue;
            }
            "/clear" => {
                history.clear();
                wizard::print_ok("History cleared");
                continue;
            }
            _ => {}
        }

        history.push(serde_json::json!({"role": "user", "content": trimmed}));

        let pb = wizard::spinner("Thinking...");
        let url = format!("{}/v1/chat/completions", endpoint);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap();

        let resp = client.post(&url)
            .header("Authorization", format!("Bearer {}", key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": model,
                "messages": history,
                "max_tokens": 500,
            }))
            .send().await;

        match resp {
            Ok(r) if r.status().is_success() => {
                let body: serde_json::Value = match r.json().await {
                    Ok(v) => v,
                    Err(e) => {
                        wizard::finish_fail(&pb, &format!("Bad response: {}", e));
                        continue;
                    }
                };
                let content = body["choices"][0]["message"]["content"].as_str().unwrap_or("");
                let reasoning = body["choices"][0]["message"]["reasoning_content"].as_str().unwrap_or("");
                let reply = if !content.is_empty() { content } else { reasoning };
                pb.finish_and_clear();
                println!("{} {}", console::style("ai:").green().bold(), reply);
                println!();
                history.push(serde_json::json!({"role": "assistant", "content": reply}));
            }
            Ok(r) => {
                wizard::finish_fail(&pb, &format!("HTTP {}", r.status()));
            }
            Err(e) => {
                wizard::finish_fail(&pb, &format!("Network: {}", e));
            }
        }
    }

    println!();
    wizard::print_ok("Bye!");
}

// ── Configure command (agent setup wizard) ──────────────────

async fn cmd_configure(http: &atomek_core::HttpClient, json: bool) {
    if json {
        eprintln!("Configure is interactive. Use `tytus exec` for scripting.");
        std::process::exit(1);
    }

    let mut state = CliState::load();
    if !state.is_logged_in() {
        wizard::print_fail("Not logged in. Run: tytus setup");
        std::process::exit(1);
    }
    ensure_token(&mut state, http).await;
    sync_tytus(&mut state, http).await;

    let pod = match state.pods.first() {
        Some(p) => p.clone(),
        None => {
            wizard::print_fail("No pod allocated. Run: tytus setup");
            std::process::exit(1);
        }
    };

    wizard::print_header("Configure your agent");
    wizard::print_info(&format!("Pod: {} — Agent: {}", pod.pod_id, pod.agent_type.as_deref().unwrap_or("?")));
    println!();

    let options = vec![
        "Test agent is running",
        "View agent logs",
        "Restart agent",
        "Advanced: run custom command",
        "Cancel",
    ];

    match wizard::select("What would you like to do?", &options) {
        Ok("Test agent is running") => {
            let pb = wizard::spinner("Checking agent...");
            let (sk, auid) = get_credentials(&mut state, http).await;
            let client = atomek_pods::TytusClient::new(http, &sk, &auid);
            match atomek_pods::exec_in_agent(&client, &pod.pod_id, "openclaw --version 2>&1 || echo 'not installed'", 10).await {
                Ok(result) => {
                    let out = result.stdout.unwrap_or_default();
                    wizard::finish_ok(&pb, "Agent responded");
                    println!();
                    wizard::print_info(&out.trim());
                }
                Err(e) => {
                    wizard::finish_fail(&pb, &format!("Failed: {}", e));
                }
            }
        }
        Ok("View agent logs") => {
            wizard::print_info("Use: tytus exec 'tail -50 /var/log/openclaw.log'");
        }
        Ok("Restart agent") => {
            if wizard::confirm("Restart the agent container?", true).unwrap_or(false) {
                wizard::print_info("Restart via DAM — use `tytus exec` for custom commands or contact support.");
            }
        }
        Ok("Advanced: run custom command") => {
            let cmd = wizard::text_input("Command to run:", None).unwrap_or_default();
            if !cmd.is_empty() {
                let (sk, auid) = get_credentials(&mut state, http).await;
                let client = atomek_pods::TytusClient::new(http, &sk, &auid);
                match atomek_pods::exec_in_agent(&client, &pod.pod_id, &cmd, 30).await {
                    Ok(result) => {
                        if let Some(out) = result.stdout {
                            if !out.is_empty() { println!("{}", out); }
                        }
                        if let Some(err) = result.stderr {
                            if !err.is_empty() { eprintln!("{}", err); }
                        }
                    }
                    Err(e) => wizard::print_fail(&e.to_string()),
                }
            }
        }
        _ => {
            wizard::print_info("Cancelled");
        }
    }
}

// ── Doctor (diagnostics) ────────────────────────────────────

async fn cmd_doctor(_http: &atomek_core::HttpClient, json: bool) {
    let mut checks: Vec<(&str, bool, String)> = Vec::new();
    let state = CliState::load();

    // 1. State file
    let state_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("tytus")
        .join("state.json");
    checks.push(("state_file", state_path.exists(),
        if state_path.exists() { state_path.display().to_string() }
        else { "Not found. Run: tytus login".into() }
    ));

    // 2. Login
    checks.push(("logged_in", state.is_logged_in(),
        if state.is_logged_in() { format!("as {}", state.email.as_deref().unwrap_or("?")) }
        else { "Run: tytus login".into() }
    ));

    // 3. Token validity
    let token_valid = state.has_valid_token();
    checks.push(("token_valid", token_valid,
        if token_valid { "Access token current".into() }
        else if state.refresh_token.is_some() { "Expired (will auto-refresh)".into() }
        else { "No token".into() }
    ));

    // 4. Tytus subscription
    checks.push(("subscription", state.secret_key.is_some(),
        if let Some(ref tier) = state.tier { format!("Plan: {}", tier) }
        else { "No subscription. Upgrade at traylinx.com".into() }
    ));

    // 5. Pods
    checks.push(("pods", !state.pods.is_empty(),
        if state.pods.is_empty() { "No pods. Run: tytus connect".into() }
        else { format!("{} pod(s)", state.pods.len()) }
    ));

    // 6. Tunnel
    let has_tunnel = state.pods.iter().any(|p| p.tunnel_iface.is_some());
    checks.push(("tunnel", has_tunnel,
        if has_tunnel {
            let ifaces: Vec<&str> = state.pods.iter()
                .filter_map(|p| p.tunnel_iface.as_deref())
                .collect();
            format!("Active on {}", ifaces.join(", "))
        } else if !state.pods.is_empty() {
            "Not running. Run: tytus connect --pod <id>".into()
        } else {
            "No pods".into()
        }
    ));

    // 7. Gateway reachability (only if tunnel active)
    if has_tunnel {
        if let Some(pod) = state.pods.iter().find(|p| p.tunnel_iface.is_some()) {
            if let (Some(ref ep), Some(ref key)) = (&pod.ai_endpoint, &pod.pod_api_key) {
                let url = format!("{}/v1/models", ep);
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(5))
                    .build()
                    .unwrap_or_default();
                match client.get(&url)
                    .header("Authorization", format!("Bearer {}", key))
                    .send().await
                {
                    Ok(resp) if resp.status().is_success() => {
                        let body = resp.text().await.unwrap_or_default();
                        let count = serde_json::from_str::<serde_json::Value>(&body)
                            .ok()
                            .and_then(|v| v.get("data").and_then(|d| d.as_array().map(|a| a.len())))
                            .unwrap_or(0);
                        checks.push(("gateway", true, format!("{} models available at {}", count, ep)));
                    }
                    Ok(resp) => {
                        checks.push(("gateway", false, format!("HTTP {} from {}", resp.status(), ep)));
                    }
                    Err(e) => {
                        checks.push(("gateway", false, format!("Unreachable: {}. Is tunnel running?", e)));
                    }
                }
            }
        }
    }

    // 8. MCP server
    let mcp_exists = std::process::Command::new("which")
        .arg("tytus-mcp")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    checks.push(("mcp_server", mcp_exists,
        if mcp_exists { "tytus-mcp installed".into() }
        else { "Not found. Install for AI CLI integration".into() }
    ));

    if json {
        let results: Vec<serde_json::Value> = checks.iter().map(|(name, ok, msg)| {
            serde_json::json!({ "check": name, "ok": ok, "message": msg })
        }).collect();
        let all_ok = checks.iter().all(|(_, ok, _)| *ok);
        println!("{}", serde_json::json!({ "healthy": all_ok, "checks": results }));
    } else {
        println!("Tytus Doctor\n");
        for (name, ok, msg) in &checks {
            let icon = if *ok { "OK" } else { "!!" };
            println!("  [{}] {}: {}", icon, name, msg);
        }
        let all_ok = checks.iter().all(|(_, ok, _)| *ok);
        println!();
        if all_ok {
            println!("All checks passed.");
        } else {
            println!("Some checks failed. Follow the suggestions above.");
        }
    }
}

const CLAUDE_MD_BLOCK: &str = r#"## Tytus Private AI Pod

This project has access to a Tytus private AI pod — an encrypted, OpenAI-compatible gateway with 383+ models.

### Quick Start
```bash
eval $(tytus env --export)  # Load connection vars
```

### Available via MCP tools (if tytus MCP server is configured):
- `tytus_status` — Check login state, plan, active pods
- `tytus_env` — Get connection URLs and API keys
- `tytus_models` — List available models on the pod
- `tytus_chat` — Send chat completions through private pod
- `tytus_setup_guide` — Step-by-step setup if not connected

### Manual usage:
```bash
# List models
curl -s "$TYTUS_AI_GATEWAY/v1/models" -H "Authorization: Bearer $TYTUS_API_KEY" | jq '.data[].id'

# Chat completion
curl "$TYTUS_AI_GATEWAY/v1/chat/completions" \
  -H "Authorization: Bearer $TYTUS_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"qwen3-8b","messages":[{"role":"user","content":"hello"}]}'
```

### OpenAI-compatible env (use with any OpenAI SDK):
```bash
export OPENAI_API_KEY=$TYTUS_API_KEY
export OPENAI_BASE_URL=${TYTUS_AI_GATEWAY}/v1
```
"#;

const AGENTS_MD_BLOCK: &str = r#"## Tytus Private AI Pod

This project has Tytus pod access — private, encrypted OpenAI-compatible AI gateway.

### Setup
```bash
eval $(tytus env --export)     # Load TYTUS_AI_GATEWAY, TYTUS_API_KEY, etc.
export OPENAI_API_KEY=$TYTUS_API_KEY
export OPENAI_BASE_URL=${TYTUS_AI_GATEWAY}/v1
```

### Commands
```bash
tytus status --json            # Pod and plan info (JSON)
tytus env --json               # Connection details (JSON)
tytus env --export             # Shell-sourceable exports
tytus connect             # Allocate pod + tunnel (blocks until Ctrl+C)
tytus revoke <pod_id>          # Free pod units
```

### API (OpenAI-compatible)
- Gateway: `$TYTUS_AI_GATEWAY/v1`
- Auth: `Bearer $TYTUS_API_KEY`
- Models: 383+ (qwen3-8b, llama-3.1-8b-instruct, etc.)
"#;

const CLAUDE_COMMAND_TYTUS: &str = r#"---
description: "Check Tytus pod status, connection info, and available models"
---

Check the current Tytus private AI pod status and provide a summary.

Run these commands:
1. `tytus status --json` to get current state
2. If connected, run `tytus env --json` to get connection details
3. If tunnel is active, test connectivity: `curl -s "$TYTUS_AI_GATEWAY/v1/models" -H "Authorization: Bearer $TYTUS_API_KEY" | jq '.data | length'`

Report:
- Login status and plan tier
- Active pods and their agent types
- Whether the tunnel is running
- AI gateway URL and model count (if reachable)
- Any issues or recommended actions
"#;

const KILO_COMMAND_TYTUS: &str = r#"---
description: "Check Tytus private AI pod status and connectivity"
---

Check the current Tytus private AI pod status.

Steps:
1. Run `tytus status --json` for current state
2. If connected, run `tytus env --export` and source the vars
3. Test: `curl -s "$TYTUS_AI_GATEWAY/v1/models" -H "Authorization: Bearer $TYTUS_API_KEY" | jq '.data | length'`

Report login status, active pods, tunnel state, and gateway reachability.
"#;

const ARCHON_COMMAND_TYTUS: &str = r#"---
description: "Check Tytus pod status and report connectivity"
---

Check Tytus private AI pod status and connectivity.

1. `tytus status --json`
2. `tytus env --json` (if pods exist)
3. Test gateway if tunnel active

Report: login state, pods, tunnel, gateway reachability, recommended actions.
"#;

const SHELL_ENV_HOOK: &str = r#"#!/bin/sh
# Tytus environment loader — source this to inject pod connection vars.
# Usage: source .tytus-env.sh
#    or: eval $(tytus env --export)

if command -v tytus >/dev/null 2>&1; then
    _tytus_env=$(tytus env --export 2>/dev/null)
    if [ -n "$_tytus_env" ]; then
        eval "$_tytus_env"
        # Also set OpenAI-compatible aliases
        export OPENAI_API_KEY="${TYTUS_API_KEY}"
        export OPENAI_BASE_URL="${TYTUS_AI_GATEWAY}/v1"
    fi
    unset _tytus_env
fi
"#;

// ── Helpers ──────────────────────────────────────────────────

/// Update tokens from API response. Preserves email if API returns empty.
fn update_tokens(state: &mut CliState, result: &atomek_auth::DeviceAuthResult, fallback_email: &Option<String>) {
    state.access_token = Some(result.access_token.clone());
    state.refresh_token = Some(result.refresh_token.clone());
    state.expires_at_ms = Some(
        chrono::Utc::now().timestamp_millis() + (result.expires_in as i64 * 1000)
    );
    // refresh_access_token returns empty user — preserve existing email
    if !result.user.email.is_empty() {
        state.email = Some(result.user.email.clone());
    } else if let Some(ref email) = fallback_email {
        state.email = Some(email.clone());
    }
}

async fn ensure_token(state: &mut CliState, http: &atomek_core::HttpClient) {
    if state.has_valid_token() { return; }
    let email_backup = state.email.clone();
    if let Some(ref rt) = state.refresh_token.clone() {
        match atomek_auth::refresh_access_token(http, rt).await {
            Ok(result) => {
                update_tokens(state, &result, &email_backup);
                state.save();
            }
            Err(e) => {
                tracing::warn!("Token refresh failed: {}", e);
            }
        }
    }
}

async fn get_credentials(state: &mut CliState, http: &atomek_core::HttpClient) -> (String, String) {
    if let (Some(s), Some(a)) = (&state.secret_key, &state.agent_user_id) {
        return (s.clone(), a.clone());
    }
    sync_tytus(state, http).await;
    match (&state.secret_key, &state.agent_user_id) {
        (Some(s), Some(a)) => (s.clone(), a.clone()),
        _ => {
            eprintln!("No Tytus subscription. Upgrade at traylinx.com");
            std::process::exit(1);
        }
    }
}

async fn sync_tytus(state: &mut CliState, http: &atomek_core::HttpClient) {
    let token = match &state.access_token {
        Some(t) => t.clone(),
        None => return,
    };

    match atomek_auth::fetch_wannolot_pass(http, &token).await {
        Ok(creds) => {
            state.secret_key = Some(creds.secret_key.clone());
            state.agent_user_id = Some(creds.agent_user_id.clone());
            state.organization_id = Some(creds.organization_id.clone());
            state.tier = Some(creds.tier.clone());
        }
        Err(atomek_core::AtomekError::NoSubscription) => {
            state.tier = None;
            return;
        }
        Err(_) => return,
    }

    if let (Some(ref sk), Some(ref auid)) = (&state.secret_key, &state.agent_user_id) {
        let client = atomek_pods::TytusClient::new(http, sk, auid);
        if let Ok(status) = atomek_pods::get_pod_status(&client).await {
            let server_ids: Vec<String> = status.pods.iter().map(|p| p.pod_id.clone()).collect();
            // Remove pods no longer on server, but preserve local endpoint data
            state.pods.retain(|p| server_ids.contains(&p.pod_id));
            // Add new pods from server (don't overwrite existing entries with richer data)
            for pod in &status.pods {
                if !state.pods.iter().any(|p| p.pod_id == pod.pod_id) {
                    state.pods.push(PodEntry {
                        pod_id: pod.pod_id.clone(),
                        droplet_id: pod.droplet_id.clone(),
                        droplet_ip: None,
                        ai_endpoint: None,
                        pod_api_key: None,
                        agent_type: pod.agent_type.clone(),
                        agent_endpoint: None,
                        tunnel_iface: None,
                    });
                }
            }
        }
    }
}

fn print_json_status(state: &CliState) {
    // Redact sensitive fields for JSON output
    let mut out = serde_json::json!({
        "logged_in": state.is_logged_in(),
        "email": state.email,
        "tier": state.tier,
        "pods": state.pods,
    });
    // Don't leak tokens in JSON output
    if let Some(obj) = out.as_object_mut() {
        obj.remove("refresh_token");
        obj.remove("access_token");
        obj.remove("secret_key");
    }
    println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
}

fn print_human_status(state: &CliState) {
    println!("Tytus — {}", state.email.as_deref().unwrap_or("?"));
    if let Some(ref tier) = state.tier {
        println!("Plan: {}", tier);
    }

    if state.pods.is_empty() {
        println!("No pods. Run: tytus connect");
    } else {
        for pod in &state.pods {
            let agent = pod.agent_type.as_deref().unwrap_or("?");
            let status = if pod.tunnel_iface.is_some() { "connected" } else { "disconnected" };
            println!("\nPod {} [{}] {}", pod.pod_id, agent, status);
            if let Some(ref ep) = pod.ai_endpoint {
                println!("  AI Gateway:    {}", ep);
            }
            if let Some(ref ep) = pod.agent_endpoint {
                println!("  Agent API:     {}", ep);
            }
            if let Some(ref key) = pod.pod_api_key {
                println!("  API Key:       {}...{}", &key[..10.min(key.len())], &key[key.len().saturating_sub(4)..]);
            }
            if let Some(ref iface) = pod.tunnel_iface {
                println!("  Tunnel:        {}", iface);
            }
        }
    }
}

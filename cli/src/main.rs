mod state;

use clap::{Parser, Subcommand, ValueEnum};
use state::{CliState, PodEntry};

#[derive(Parser)]
#[command(name = "tytus", about = "Tytus private AI pod — connect from any terminal", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

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
    /// Disconnect a pod's tunnel
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
        Commands::Login => cmd_login(&http, cli.json).await,
        Commands::Status => cmd_status(&http, cli.json).await,
        Commands::Connect { pod, agent } => cmd_connect(&http, pod, agent.as_str(), cli.json).await,
        Commands::Disconnect { pod } => cmd_disconnect(pod, cli.json).await,
        Commands::Revoke { pod } => cmd_revoke(&http, &pod, cli.json).await,
        Commands::Logout => cmd_logout(&http, cli.json).await,
        Commands::Env { pod, export } => cmd_env(pod, export, cli.json),
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

    ensure_token(&mut state, http).await;
    let (sk, auid) = get_credentials(&mut state, http).await;
    let client = atomek_pods::TytusClient::new(http, &sk, &auid);
    let target_pod_id: String;

    if let Some(ref pid) = pod_id {
        target_pod_id = pid.clone();
        if !json { eprintln!("Connecting to pod {}...", pid); }
    } else {
        if !json { eprintln!("Allocating {} pod...", agent); }
        match atomek_pods::request_pod_with_agent(&client, agent).await {
            Ok(a) => {
                target_pod_id = a.pod_id.clone();
                // Remove stale entry if exists, then add fresh
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

    // Activate WireGuard tunnel
    if !json { eprintln!("Activating WireGuard tunnel..."); }
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
            if let Some(pod) = state.pods.iter_mut().find(|p| p.pod_id == target_pod_id) {
                pod.tunnel_iface = Some(iface.clone());
            }
            state.save();

            if json {
                // JSON output: pod info to stdout (status messages went to stderr)
                let pod = state.pods.iter().find(|p| p.pod_id == target_pod_id);
                println!("{}", serde_json::to_string_pretty(&pod).unwrap_or_default());
            } else {
                eprintln!("✓ Tunnel active on {}", iface);
                if let Some(pod) = state.pods.iter().find(|p| p.pod_id == target_pod_id) {
                    println!();
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
                eprintln!("\nTunnel running. Press Ctrl+C to disconnect.");
            }

            // Keep running until Ctrl+C
            tokio::signal::ctrl_c().await.ok();
            if !json { eprintln!("\nShutting down tunnel..."); }
            handle.shutdown().await;
            if let Some(pod) = state.pods.iter_mut().find(|p| p.pod_id == target_pod_id) {
                pod.tunnel_iface = None;
            }
            state.save();
            if !json { eprintln!("✓ Disconnected"); }
        }
        Err(e) => {
            state.save();
            let msg = e.to_string();
            if msg.contains("permission") || msg.contains("Operation not permitted") {
                eprintln!("TUN device requires root. Run with sudo:\n");
                eprintln!("  sudo tytus connect{}", pod_id.map(|p| format!(" --pod {}", p)).unwrap_or_default());
            } else {
                eprintln!("Tunnel failed: {}", msg);
            }
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

    match atomek_pods::revoke_pod(&client).await {
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

    if let Some(ref pid) = pod_id {
        if let Some(pod) = state.pods.iter_mut().find(|p| p.pod_id == *pid) {
            pod.tunnel_iface = None;
        }
    } else {
        for pod in &mut state.pods {
            pod.tunnel_iface = None;
        }
    }
    state.save();

    if json { println!(r#"{{"status":"disconnected"}}"#); }
    else { println!("✓ Tunnel state cleared"); }
}

// ── Logout ───────────────────────────────────────────────────

async fn cmd_logout(http: &atomek_core::HttpClient, json: bool) {
    let mut state = CliState::load();

    if state.is_logged_in() {
        if let (Some(ref sk), Some(ref auid)) = (&state.secret_key, &state.agent_user_id) {
            let client = atomek_pods::TytusClient::new(http, sk, auid);
            // Revoke all pods (API revokes all for this client)
            let _ = atomek_pods::revoke_pod(&client).await;
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
        else { eprintln!("No pods. Run: sudo tytus connect"); }
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
        println!("No pods. Run: sudo tytus connect");
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

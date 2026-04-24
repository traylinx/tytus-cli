mod channels;
mod channels_store;
mod cmd_transfer;
mod daemon;
mod state;
mod transfer;
#[allow(dead_code)]
mod wizard;

// `tunnel_reap` lives in the `atomek_cli` lib target so integration tests
// can exercise it directly. Re-export the module path here so the rest of
// main.rs can reference it as `tunnel_reap::...` unchanged.
use atomek_cli::tunnel_reap;

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

    /// Force non-interactive mode (skip browser auth, log to /tmp/tytus/autostart.log).
    /// Also triggered by TYTUS_HEADLESS=1 env var. Use in LaunchAgents and cron.
    #[arg(long, global = true)]
    headless: bool,
}

#[derive(Clone, ValueEnum)]
enum AgentType {
    Nemoclaw,
    Hermes,
}

#[derive(Clone, ValueEnum, Debug)]
enum AutostartAction {
    /// Install the auto-start hook (macOS LaunchAgent / Linux systemd --user)
    Install,
    /// Remove the auto-start hook
    Uninstall,
    /// Show whether auto-start is currently installed
    Status,
}

impl AgentType {
    fn as_str(&self) -> &str {
        match self { AgentType::Nemoclaw => "nemoclaw", AgentType::Hermes => "hermes" }
    }
}

#[derive(Clone, ValueEnum, Debug)]
enum DaemonAction {
    /// Start the daemon in foreground (for launchd/systemd)
    Run,
    /// Stop a running daemon
    Stop,
    /// Check daemon status
    Status,
}

#[derive(Subcommand, Debug)]
enum AgentAction {
    /// Install an agent. Without --pod, allocates a new pod. With --pod,
    /// deploys into that pod (use --force to replace an existing agent).
    Install {
        /// Agent type — accepts `openclaw` (aliased to `nemoclaw` on the
        /// backend), `hermes`, or any other id from `tytus agent catalog`.
        name: String,
        /// Existing pod slot to install into. Omit to allocate a new slot.
        #[arg(short, long)]
        pod: Option<String>,
        /// Replace an existing agent in the pod (destroys container state).
        #[arg(long)]
        force: bool,
    },
    /// Stop + remove the agent container. Pod slot stays allocated so AIL
    /// keeps working through it — use `tytus revoke` to free the slot.
    Uninstall {
        /// Pod ID to uninstall from
        pod: String,
    },
    /// List all pods with their agent status (default + agent-bearing).
    List,
    /// Show the installable agent catalog (cached locally for 5 min).
    Catalog {
        /// Bypass the cache and force a live fetch.
        #[arg(long)]
        refresh: bool,
    },
}

#[derive(Subcommand, Debug)]
enum ChannelsAction {
    /// Configure a new chat channel for the pod's agent. Stores
    /// credentials in the OS keychain, writes them to the pod's
    /// state volume, and redeploys the agent container so the
    /// channel's plugin picks up the env vars at startup.
    Add {
        /// Pod ID (e.g. "02")
        #[arg(short, long)]
        pod: String,
        /// Channel type: telegram, discord, slack, line
        /// (run `tytus channels catalog` to list supported channels).
        #[arg(long, value_name = "CHANNEL")]
        r#type: String,
        /// Primary credential. For Telegram/Discord/single-token
        /// channels this is the only flag you need. Stored in the
        /// OS keychain, never logged.
        #[arg(long)]
        token: Option<String>,
        /// Slack: app-level token (xapp-...). Required for Socket Mode.
        #[arg(long)]
        app_token: Option<String>,
        /// Slack: user-scoped token (xoxp-...). Optional.
        #[arg(long)]
        user_token: Option<String>,
        /// LINE: channel secret (pairs with --token).
        #[arg(long)]
        channel_secret: Option<String>,
    },
    /// Show which channels are configured for a given pod. Reads
    /// the pod's channels.json state file; does NOT show credential
    /// values (those live in your keychain).
    List {
        /// Pod ID (e.g. "02")
        #[arg(short, long)]
        pod: String,
    },
    /// Remove a channel's credentials from a pod. Clears the OS
    /// keychain entries, removes from channels.json on the pod, and
    /// redeploys the agent so the channel stops operating.
    Remove {
        /// Pod ID (e.g. "02")
        #[arg(short, long)]
        pod: String,
        /// Channel type to remove.
        #[arg(long, value_name = "CHANNEL")]
        r#type: String,
    },
    /// List all channels this CLI knows how to configure, with
    /// per-channel credential requirements and inbound-delivery
    /// model. Read this before running `tytus channels add`.
    Catalog,
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
    /// Activate the WireGuard tunnel. With no flags, uses your default pod
    /// (agent-less, AIL-only) — allocating it if needed. Use `--pod` to
    /// connect to a specific slot, or `--agent` as a deprecated shim for
    /// `tytus agent install X`.
    Connect {
        /// Pod ID to reconnect (e.g. "01"). Omit to use the default pod.
        #[arg(short, long)]
        pod: Option<String>,
        /// DEPRECATED — delegates to `tytus agent install <TYPE>`. Kept as
        /// a shim because internal scripts and docs still reference it.
        #[arg(short, long, value_enum)]
        agent: Option<AgentType>,
    },
    /// Manage agents: install into a pod, uninstall, replace, list, catalog.
    /// Decouples pod allocation from agent deployment — default pods
    /// (agent-less, AIL-only) come for free on `tytus login`; agents cost
    /// plan units and are installed explicitly.
    Agent {
        #[command(subcommand)]
        action: AgentAction,
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
        /// Emit raw per-pod values (unstable, changes when pod rotates).
        /// Default is the stable 10.42.42.1 endpoint + per-user stable key
        /// that never changes unless you call `tytus rotate-key`.
        #[arg(long)]
        raw: bool,
        /// Force the legacy WireGuard tunnel endpoint (10.42.42.1) even
        /// when the public-HTTPS URL is available. Use if your network
        /// blocks outbound TLS or you explicitly want the encrypted tunnel.
        #[arg(long)]
        tunnel: bool,
    },
    /// Print the full LLM-facing reference (for AI agents driving tytus-cli)
    LlmDocs,
    /// Discover the pod's AI gateway catalog — models + provider-native
    /// tools (e.g. MiniMax M2.7's autonomous web_search). One call covers
    /// every model the pod exposes via `/v1/models`.
    ///
    /// Default output is a human tree; `--json` (global flag) passes the
    /// gateway's raw response through verbatim so scripts and other AI
    /// CLIs can parse it. Useful for AI agents that want to discover the
    /// exact native-tool surface they can splice into their caller tools[].
    Capabilities {
        /// Pod ID (defaults to first connected pod).
        #[arg(short, long)]
        pod: Option<String>,
    },
    /// Print a short setup prompt you can paste into any AI tool (Claude Code,
    /// OpenCode, Cursor, etc.) to teach it how to drive Tytus natively.
    BootstrapPrompt,
    /// Hidden: validated SIGTERM helper for tunnel daemons. Verifies the PID
    /// matches a known tunnel-NN.pid file under /tmp/tytus before killing.
    /// Used by `tytus disconnect` via passwordless sudoers (replaces the old
    /// `/bin/kill -TERM *` entry which allowed killing any process as root).
    #[command(hide = true)]
    TunnelDown {
        /// PID to validate and SIGTERM
        pid: i32,
    },
    /// Link a project to Tytus — drops CLAUDE.md / AGENTS.md / .mcp.json /
    /// slash commands into the target directory so any AI CLI (Claude Code,
    /// OpenCode, KiloCode, Archon) natively knows how to drive your private
    /// Tytus pod from that project.
    #[command(alias = "infect")]
    Link {
        /// Target project directory (defaults to current dir)
        #[arg(default_value = ".")]
        dir: String,
        /// Which integrations to drop (default: all). Options:
        /// claude, agents, kilocode, opencode, archon, shell
        #[arg(short, long, value_delimiter = ',')]
        only: Option<Vec<String>>,
    },
    /// Print MCP server configuration for your AI CLI
    Mcp {
        /// Output format: claude, kilocode, opencode, archon, json
        #[arg(short, long, default_value = "claude")]
        format: String,
    },
    /// Restart the agent container (applies config changes)
    Restart {
        /// Pod ID (defaults to first pod)
        #[arg(short, long)]
        pod: Option<String>,
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
    /// Drive a pod-hosted agent (OpenClaw, Hermes, …) as a lope teammate.
    /// Thin shim around `python3 -m tytus_sdk` — the SDK is the source of
    /// truth for the WS + Ed25519 protocol. See docs/DESIGN-TYTUS-LOPE-
    /// TEAMMATES.md.
    Lope {
        /// Subcommand: ask | install | uninstall | list | identity | lope_validate
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// HarveyBridge — the reverse channel (pod agent → Harvey's brain).
    /// Starts an HTTP listener + per-pod outbox pollers.
    /// Subcommands: run | status | rotate-token | test.
    /// Thin shim around `python3 -m tytus_sdk bridge`.
    Bridge {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Add / remove / list chat channels that the pod's agent should
    /// use to talk to you. Pod agents have built-in extensions for
    /// Telegram, Discord, Slack (Socket Mode), LINE, etc. — this
    /// command supplies the bot tokens / API secrets they need.
    ///
    /// Credentials are stored in the OS keychain and synced to the pod
    /// via the agent state volume. The agent container is redeployed
    /// with the new env vars so channel plugins pick them up at
    /// startup. Requires an HTTPS-capable pod (droplet must have the
    /// 2026-04-20 egress bridge applied).
    Channels {
        #[command(subcommand)]
        action: ChannelsAction,
    },
    /// Install/uninstall/check the auto-start-on-boot hook so your tunnel
    /// re-establishes automatically when you log back in after a reboot.
    /// Your apps configured with the stable `http://10.42.42.1:18080` +
    /// `sk-tytus-user-*` pair keep working across restarts with zero
    /// re-configuration — just like Ollama.
    Autostart {
        #[arg(value_enum, default_value = "status")]
        action: AutostartAction,
    },
    /// Open the OpenClaw control UI in your browser via a localhost forwarder.
    /// Browsers require HTTPS or localhost for WebCrypto / device identity
    /// APIs, so a direct `http://10.X.Y.1:3000` URL gets blocked. This command
    /// starts a 127.0.0.1 TCP forwarder pointing at the pod's agent port,
    /// opens the browser, and blocks until Ctrl+C.
    Ui {
        /// Pod ID (defaults to first connected pod)
        #[arg(short, long)]
        pod: Option<String>,
        /// Local port to bind the forwarder on. If omitted, defaults to
        /// `18700 + pod_num` (pod 01 → 18701, pod 02 → 18702, …). Port
        /// 3000 used to be the default but conflicted with every React /
        /// Next / Rails dev server; the 18700s are a quiet neighborhood
        /// and the per-pod offset makes URLs bookmarkable.
        #[arg(short = 'P', long)]
        port: Option<u16>,
        /// Don't open the browser automatically — just print the URL
        #[arg(long)]
        no_open: bool,
        /// Stop an already-running forwarder for this pod instead of starting one.
        /// Reads /tmp/tytus/ui-<pod>.port and sends SIGTERM to the pid. With no
        /// --pod, stops every live forwarder.
        #[arg(long)]
        stop: bool,
    },
    /// Run diagnostics: check auth, tunnel, gateway connectivity
    Doctor,
    /// Manage the tytus background daemon (token refresh, health monitoring).
    /// Use 'run' for foreground (launchd/systemd), 'stop' to send shutdown.
    Daemon {
        #[arg(value_enum, default_value = "status")]
        action: DaemonAction,
    },
    /// Install / uninstall the macOS Tytus.app bundle + launch-at-login hook.
    /// Makes tytus-tray findable in Spotlight, draggable to the Dock, and
    /// auto-starts it on every login (single-instance guard prevents dupes).
    /// Same model as Ollama / Docker Desktop: quit for the session, restart
    /// via Spotlight, comes back on reboot.
    Tray {
        #[arg(value_enum, default_value = "status")]
        action: TrayAction,
    },
    /// (internal) Activate tunnel from a temp config file — called by elevated helper
    #[command(hide = true)]
    TunnelUp {
        /// Path to temp JSON file with tunnel config
        config_file: String,
    },
    /// Push a file or directory from your Mac into a pod's workspace.
    /// Directories are tarred+streamed automatically. Default destination
    /// is `/app/workspace/inbox/` (auto-created). Refuses transfers > 100 MB
    /// (use the Garage-backed shared filesystem in v0.7 for GB-scale).
    Push {
        /// Local file or directory to push.
        local: String,
        /// Pod ID to push to. Omit if exactly one pod is connected.
        #[arg(short, long)]
        pod: Option<String>,
        /// Remote destination. Default `/app/workspace/inbox/`. Must live
        /// under `/app/workspace/` — pod rootfs is read-only outside.
        #[arg(long)]
        to: Option<String>,
        /// Suppress the progress bar (otherwise shown on stderr for >1 MB).
        #[arg(long)]
        quiet: bool,
    },
    /// Pull a file or directory from a pod's workspace to your Mac.
    Pull {
        /// Remote path (must live under /app/workspace/).
        remote: String,
        /// Pod ID to pull from. Omit if exactly one pod is connected.
        #[arg(short, long)]
        pod: Option<String>,
        /// Local destination. Default: current directory, basename preserved.
        #[arg(long)]
        to: Option<String>,
        /// Suppress the progress bar.
        #[arg(long)]
        quiet: bool,
    },
    /// List contents of a remote path under /app/workspace/. Default
    /// PATH is `/app/workspace/inbox/`. Columns: mode, size, mtime, name.
    Ls {
        /// Remote path. Default `/app/workspace/inbox/`.
        path: Option<String>,
        /// Pod ID. Omit if exactly one pod is connected.
        #[arg(short, long)]
        pod: Option<String>,
    },
    /// Delete a remote path under /app/workspace/. Requires --recursive
    /// for directories. Refuses any path outside /app/workspace/.
    Rm {
        /// Remote path to delete.
        remote: String,
        /// Pod ID. Omit if exactly one pod is connected.
        #[arg(short, long)]
        pod: Option<String>,
        /// Required to delete a directory.
        #[arg(long)]
        recursive: bool,
    },
    /// Show the local push/pull/rm audit log (JSONL). Defaults to the last
    /// 20 events; `--tail 0` prints all.
    Transfers {
        /// How many trailing rows to show. Use 0 for all.
        #[arg(long, default_value = "20")]
        tail: usize,
        /// Filter by pod id.
        #[arg(short, long)]
        pod: Option<String>,
    },
}

#[derive(Clone, ValueEnum, Debug)]
enum TrayAction {
    /// Install Tytus.app in /Applications + LaunchAgent for auto-start
    Install,
    /// Remove the .app bundle and LaunchAgent
    Uninstall,
    /// Show what's installed
    Status,
    /// Start the tray right now (open /Applications/Tytus.app if present,
    /// otherwise fall back to ~/bin/tytus-tray)
    Start,
}

/// Initialize tracing so structured log noise doesn't pollute
/// interactive CLI output. Three modes:
///
/// - `RUST_LOG` is set explicitly → honor it verbatim, emit to stderr
///   (developer/debug path).
/// - `--json` or non-interactive (stdout+stderr piped) → emit WARN+ to
///   stderr. Machine consumers want structured logs inline; they can
///   filter as needed.
/// - Default (humans running `tytus` in a terminal) → route WARN+ to
///   `~/.tytus/logs/tytus.log` (rotating, mode 0600). Stderr stays
///   clean so "Paste your credentials" prompts, "✓ Telegram
///   configured" confirmations, etc. aren't buried under transient
///   keychain-timeout warnings.
///
/// The big win: users on a broken-keychain machine (ACL pending
/// approval) no longer see the `WARN keychain get_refresh_token
/// timed out after 3s` line bleeding into the tytus output they're
/// actively reading. Warnings still hit the log file for post-hoc
/// debugging via `View Daemon Log` in the tray.
fn init_tracing() {
    let explicit = std::env::var("RUST_LOG").is_ok();
    let json_mode = std::env::args().any(|a| a == "--json");
    let interactive_stderr = console::Term::stderr().is_term();

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "warn,tytus=info".into());

    if explicit || json_mode || !interactive_stderr {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .init();
        return;
    }

    // Interactive human CLI run. Try to open the log file; fall back
    // to silent stderr (nothing worse than warnings printing AFTER we
    // committed to hiding them).
    let log_path = std::env::var("HOME")
        .ok()
        .map(|h| std::path::PathBuf::from(h).join(".tytus/logs/tytus.log"));
    let mut writer: Option<std::fs::File> = None;
    if let Some(path) = log_path.as_ref() {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
            }
            writer = Some(f);
        }
    }

    match writer {
        Some(file) => {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_target(false)
                .with_ansi(false)
                .with_writer(std::sync::Mutex::new(file))
                .init();
        }
        None => {
            // Couldn't open the log file — degrade to ERROR-only on
            // stderr so users never see transient WARN noise.
            tracing_subscriber::fmt()
                .with_env_filter("error")
                .with_target(false)
                .init();
        }
    }
}

#[tokio::main]
async fn main() {
    init_tracing();

    let cli = Cli::parse();

    // Propagate --headless to env so wizard::is_interactive() picks it up
    // everywhere (including library code that can't see CLI args).
    // LaunchAgent plists can also set TYTUS_HEADLESS=1 directly.
    if cli.headless {
        std::env::set_var("TYTUS_HEADLESS", "1");
    }

    let http = atomek_core::HttpClient::new();

    match cli.command {
        None => cmd_default(&http, cli.json).await,
        Some(Commands::Setup) => cmd_setup(&http, cli.json).await,
        Some(Commands::Test) => cmd_test(&http, cli.json).await,
        Some(Commands::Chat { model }) => cmd_chat(&http, &model, cli.json).await,
        Some(Commands::Configure) => cmd_configure(&http, cli.json).await,
        Some(Commands::Login) => cmd_login(&http, cli.json).await,
        Some(Commands::Status) => cmd_status(&http, cli.json).await,
        Some(Commands::Connect { pod, agent }) => {
            // `--agent X` is a shim for `tytus agent install X` + tunnel-up
            // (see SPRINT §6 B2). Pre-sprint, `tytus connect --agent X` did
            // allocate+deploy+tunnel; we preserve that by chaining into
            // cmd_connect with the newly-installed pod's id. Without this
            // chain, cmd_connect would default to the agent-less default
            // pod on the next invocation and the user's new agent would
            // never get a tunnel.
            if let Some(a) = agent {
                let new_pod = cmd_agent_install(&http, a.as_str(), pod.clone(), false, cli.json).await;
                cmd_connect(&http, new_pod.or(pod), cli.json).await;
            } else {
                cmd_connect(&http, pod, cli.json).await;
            }
        }
        Some(Commands::Agent { action }) => cmd_agent(&http, action, cli.json).await,
        Some(Commands::Disconnect { pod }) => cmd_disconnect(pod, cli.json).await,
        Some(Commands::Revoke { pod }) => cmd_revoke(&http, &pod, cli.json).await,
        Some(Commands::Logout) => cmd_logout(&http, cli.json).await,
        Some(Commands::Env { pod, export, raw, tunnel }) => cmd_env(pod, export, raw, tunnel, cli.json, &http).await,
        Some(Commands::LlmDocs) => { print!("{}", LLM_DOCS); }
        Some(Commands::Capabilities { pod }) => cmd_capabilities(&http, pod, cli.json).await,
        Some(Commands::BootstrapPrompt) => { print!("{}", BOOTSTRAP_PROMPT); }
        Some(Commands::TunnelDown { pid }) => cmd_tunnel_down(pid),
        Some(Commands::Link { dir, only }) => cmd_link(&dir, only, cli.json),
        Some(Commands::Mcp { format }) => cmd_mcp(&format, cli.json),
        Some(Commands::Restart { pod }) => cmd_restart(&http, pod, cli.json).await,
        Some(Commands::Exec { command, pod, timeout }) => cmd_exec(&http, command, pod, timeout, cli.json).await,
        Some(Commands::Lope { args }) => cmd_lope_passthrough("lope", args, cli.json).await,
        Some(Commands::Bridge { args }) => cmd_lope_passthrough("bridge", args, cli.json).await,
        Some(Commands::Channels { action }) => cmd_channels(&http, action, cli.json).await,
        Some(Commands::Autostart { action }) => cmd_autostart(action, cli.json),
        Some(Commands::Ui { pod, port, no_open, stop }) => {
            if stop { cmd_ui_stop(pod, cli.json).await; }
            else    { cmd_ui(&http, pod, port, no_open, cli.json).await; }
        }
        Some(Commands::Doctor) => cmd_doctor(&http, cli.json).await,
        Some(Commands::Daemon { action }) => cmd_daemon(action, cli.json).await,
        Some(Commands::Tray { action }) => cmd_tray(action, cli.json),
        // Hidden subcommand: called by elevated helper to activate tunnel from a temp config file
        Some(Commands::TunnelUp { config_file }) => cmd_tunnel_up(&config_file, cli.json).await,
        Some(Commands::Push { local, pod, to, quiet }) => {
            cmd_transfer::cmd_push(&http, local, pod, to, quiet, cli.json).await
        }
        Some(Commands::Pull { remote, pod, to, quiet }) => {
            cmd_transfer::cmd_pull(&http, remote, pod, to, quiet, cli.json).await
        }
        Some(Commands::Ls { path, pod }) => {
            cmd_transfer::cmd_ls(&http, path, pod, cli.json).await
        }
        Some(Commands::Rm { remote, pod, recursive }) => {
            cmd_transfer::cmd_rm(&http, remote, pod, recursive, cli.json).await
        }
        Some(Commands::Transfers { tail, pod }) => {
            cmd_transfer::cmd_transfers(tail, pod, cli.json).await
        }
    }
}

async fn cmd_daemon(action: DaemonAction, json: bool) {
    match action {
        DaemonAction::Run => {
            daemon::run_daemon().await;
        }
        DaemonAction::Stop => {
            match daemon::send_command("shutdown", serde_json::Value::Null).await {
                Some(resp) if resp.status == "ok" => {
                    if json { println!(r#"{{"daemon":"stopped"}}"#); }
                    else { println!("Daemon stopped."); }
                }
                Some(resp) => {
                    eprintln!("Daemon error: {}", resp.error.unwrap_or_default());
                    std::process::exit(1);
                }
                None => {
                    if json { println!(r#"{{"daemon":"not_running"}}"#); }
                    else { println!("Daemon is not running."); }
                }
            }
        }
        DaemonAction::Status => {
            match daemon::send_command("status", serde_json::Value::Null).await {
                Some(resp) if resp.status == "ok" => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&resp.data).unwrap_or_default());
                    } else if let Some(data) = &resp.data {
                        let pid = data.pointer("/daemon/pid").and_then(|v| v.as_u64()).unwrap_or(0);
                        let uptime = data.pointer("/daemon/uptime_secs").and_then(|v| v.as_u64()).unwrap_or(0);
                        let status = data.pointer("/daemon/status").and_then(|v| v.as_str()).unwrap_or("?");
                        let token = data.pointer("/auth/token_valid").and_then(|v| v.as_bool()).unwrap_or(false);
                        let email = data.pointer("/auth/email").and_then(|v| v.as_str()).unwrap_or("?");
                        println!("Daemon:  ● running (pid {}, uptime {}s)", pid, uptime);
                        println!("Status:  {}", status);
                        println!("Auth:    {} ({})", if token { "● valid" } else { "○ expired" }, email);
                        let pods = data.pointer("/pods").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                        println!("Pods:    {}", pods);
                    }
                }
                Some(resp) => {
                    eprintln!("Daemon error: {}", resp.error.unwrap_or_default());
                }
                None => {
                    if json { println!(r#"{{"daemon":"not_running"}}"#); }
                    else { println!("Daemon is not running. Start with: tytus daemon run"); }
                }
            }
        }
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
                let default_msg = ensure_default_pod(&mut state, http).await;
                state.save();
                if json { print_json_status(&state); }
                else {
                    println!("✓ Logged in as {}", state.email.as_deref().unwrap_or("?"));
                    if let Some(msg) = default_msg { println!("{}", msg); }
                }
                return;
            }
            Err(_) => {
                if !json { eprintln!("Stored token expired. Starting fresh login..."); }
            }
        }
    }

    // Device auth flow — refuse in headless context (LaunchAgent, cron, pipe)
    if !wizard::is_interactive() {
        let msg = "Cannot open browser for login in non-interactive context. Run 'tytus login' from a terminal.";
        append_autostart_log(&format!("cmd_login BLOCKED: {}", msg));
        eprintln!("tytus: {}", msg);
        std::process::exit(1);
    }

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
    let default_msg = ensure_default_pod(&mut state, http).await;
    state.save();

    if json {
        print_json_status(&state);
    } else {
        println!("✓ Logged in as {}", result.user.email);
        if let Some(ref tier) = state.tier {
            println!("  Plan: {}", tier);
        }
        if let Some(msg) = default_msg { println!("{}", msg); }
    }
}

/// Idempotently ensure the user has a default (agent-less, 0-unit) pod so
/// AIL access at http://10.42.42.1:18080 works right after `tytus login`.
///
/// Returns a human-readable status line for the login output, or `None`
/// if we can't or shouldn't show one (no subscription yet, silent
/// capacity failure, etc). Never panics and never fails the login: a
/// default-pod hiccup must not lock the user out of their account.
///
/// Called on every login (fresh browser flow AND RT auto-refresh) per
/// SPRINT-AIL-DEFAULT-POD phase A6.
async fn ensure_default_pod(state: &mut CliState, http: &atomek_core::HttpClient) -> Option<String> {
    let (sk, auid) = match (state.secret_key.as_ref(), state.agent_user_id.as_ref()) {
        (Some(s), Some(a)) => (s.clone(), a.clone()),
        _ => return None, // no subscription / Wannolot Pass yet
    };

    // Always POST /pod/default. The endpoint is idempotent server-side, and
    // unconditionally calling it is the only way to guarantee we pick up a
    // fresh `stable_user_key` — `sync_tytus`'s pod list (from /pod/status)
    // returns a "thin" entry without the user key, so relying on its
    // presence to short-circuit would leave the user unable to use AIL.
    let client = atomek_pods::TytusClient::new(http, &sk, &auid);
    match atomek_pods::request_default_pod(&client).await {
        Ok(alloc) => {
            // PRESERVE tunnel_iface on refresh. If this pod_id already
            // exists in state with a live tunnel, keeping the iface is
            // essential — a blind retain+push zeroed it on every login
            // and made the tray think the tunnel was down even when
            // boringtun was alive and routing traffic. Reported
            // 2026-04-19 ("tunnel_iface null in state but tunnel works").
            let preserved_iface = state.pods.iter()
                .find(|p| p.pod_id == alloc.pod_id)
                .and_then(|p| p.tunnel_iface.clone());
            state.pods.retain(|p| p.pod_id != alloc.pod_id);
            state.pods.push(PodEntry {
                pod_id: alloc.pod_id.clone(),
                droplet_id: alloc.droplet_id.clone(),
                droplet_ip: alloc.droplet_ip.clone(),
                ai_endpoint: alloc.ai_endpoint.clone(),
                pod_api_key: alloc.pod_api_key.clone(),
                agent_type: Some("none".to_string()),
                agent_endpoint: None,
                tunnel_iface: preserved_iface,
                stable_ai_endpoint: alloc.stable_ai_endpoint.clone(),
                stable_user_key: alloc.stable_user_key.clone(),
                gateway_token: None,
                edge_slug: None,
                edge_public_url: None,
                pod_public_url: alloc.pod_public_url.clone(),
            });
            // Message verbatim per sprint doc §6 A6 acceptance criterion.
            // The stable endpoint at 10.42.42.1:18080 is reachable once the
            // tunnel is up (Phase B2 makes `tytus connect` activate the
            // default pod's tunnel; the tray auto-probes and can activate
            // on its own). A6 scope is allocation only.
            Some(format!(
                "✓ Default pod ready at {}",
                alloc.stable_ai_endpoint.as_deref().unwrap_or("http://10.42.42.1:18080")
            ))
        }
        Err(atomek_core::AtomekError::NoCapacity { .. }) => {
            Some("⚠ Default pod unavailable (no capacity). Retry with: tytus connect".to_string())
        }
        Err(e) => {
            // Don't fail login on a default-pod blip; the user can retry via
            // `tytus connect` once the backend recovers.
            Some(format!("⚠ Default pod not provisioned: {}. Retry with: tytus connect", e))
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

    if let Err(e) = ensure_token(&mut state, http).await {
        if json { println!(r#"{{"logged_in":true,"token_error":"{}"}}"#, e); }
        else { eprintln!("Token refresh failed: {}. Run: tytus login", e); }
        return;
    }
    sync_tytus(&mut state, http).await;

    // Detect stale tunnels: state says tunnel is up but interface/daemon is dead
    reap_dead_tunnels(&mut state);
    state.save();

    if json { print_json_status(&state); }
    else { print_human_status(&state); }
}

// ── Connect ──────────────────────────────────────────────────

async fn cmd_connect(http: &atomek_core::HttpClient, pod_id: Option<String>, json: bool) {
    let mut state = CliState::load();
    let headless = !wizard::is_interactive();

    // Structured diagnostic: log startup state in headless context
    if headless {
        let expires_desc = state.expires_at_ms.map(|ms| {
            let secs = ms / 1000;
            chrono::DateTime::from_timestamp(secs, 0)
                .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
                .unwrap_or_else(|| format!("{}ms", ms))
        });
        append_autostart_log(&format!(
            "cmd_connect START: email={}, has_rt={}, has_at={}, expires_at={}, pods={}, pod_id={:?}",
            state.email.as_deref().unwrap_or("none"),
            state.refresh_token.is_some(),
            state.access_token.is_some(),
            expires_desc.as_deref().unwrap_or("none"),
            state.pods.len(),
            pod_id,
        ));
    }

    if !state.is_logged_in() {
        let msg = "Not logged in. Run: tytus login";
        if !wizard::is_interactive() {
            append_autostart_log(&format!("cmd_connect FAILED: {}", msg));
        }
        eprintln!("{}", msg);
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
    //
    // Try a silent refresh first. If the RT itself is dead server-side
    // (Sentinel ~24h TTL, user hasn't touched the CLI in a while), fall
    // back to the interactive browser device-auth flow — but ONLY if we
    // have a controlling terminal. A headless connect (LaunchAgent /
    // cron) should fail with the clear remediation instead of trying to
    // open a browser where no one can approve it. User reported
    // 2026-04-18: "it should try refresh and when not possible login".
    if let Err(e) = ensure_token(&mut state, http).await {
        let auth_expired = matches!(e, atomek_core::AtomekError::AuthExpired)
            || e.to_string().contains("Authentication expired");
        if auth_expired && wizard::is_interactive() {
            if !json {
                eprintln!("Refresh token expired — opening browser to re-authenticate...");
            }
            // cmd_login handles the device-auth flow end-to-end: browser
            // open, poll Sentinel, persist new tokens to keychain, run
            // sync_tytus, and (post-A6) re-provision the default pod.
            // It exits on failure, so if it returns, we're good.
            cmd_login(http, false).await;
            // Reload state from disk — cmd_login wrote fresh tokens +
            // possibly a new default pod entry there.
            state = CliState::load();
            if !state.is_logged_in() {
                eprintln!("Login cancelled. Run: tytus login");
                std::process::exit(1);
            }
        } else {
            eprintln!("Token refresh failed: {}. Run: tytus login", e);
            std::process::exit(1);
        }
    }
    let (sk, auid) = get_credentials(&mut state, http).await;
    let client = atomek_pods::TytusClient::new(http, &sk, &auid);
    let target_pod_id: String;

    if let Some(ref pid) = pod_id {
        target_pod_id = pid.clone();
        if !json { eprintln!("Connecting to pod {}...", pid); }
    } else if let Some(default_pod) = state.pods.iter().find(|p| p.agent_type.as_deref() == Some("none")) {
        // Prefer the user's default pod (agent-less, AIL-only) as the
        // tunnel target — it's universal and free, matches the spirit of
        // SPRINT §6 B2. Fall through to the existing-pod reuse if there
        // isn't one yet.
        target_pod_id = default_pod.pod_id.clone();
        if !json { eprintln!("Connecting to default pod {}...", target_pod_id); }
    } else if let Some(existing) = state.pods.first() {
        // No default pod yet, but the user has agent-bearing pods — reuse
        // the first one to keep the IP stable.
        target_pod_id = existing.pod_id.clone();
        if !json { eprintln!("Reconnecting to pod {}...", target_pod_id); }
    } else {
        // No pods at all — allocate a default pod (free, 0 units) so a
        // user who logged in without provisioning (e.g. early-access path)
        // still gets working AIL access.
        if !json { eprintln!("Allocating default pod..."); }
        match atomek_pods::request_default_pod(&client).await {
            Ok(a) => {
                target_pod_id = a.pod_id.clone();
                let preserved_iface = state.pods.iter()
                    .find(|p| p.pod_id == a.pod_id)
                    .and_then(|p| p.tunnel_iface.clone());
                state.pods.retain(|p| p.pod_id != a.pod_id);
                state.pods.push(PodEntry {
                    pod_id: a.pod_id.clone(),
                    droplet_id: a.droplet_id.clone(),
                    droplet_ip: a.droplet_ip.clone(),
                    ai_endpoint: a.ai_endpoint.clone(),
                    pod_api_key: a.pod_api_key.clone(),
                    agent_type: Some("none".to_string()),
                    agent_endpoint: None,
                    tunnel_iface: preserved_iface,
                    stable_ai_endpoint: a.stable_ai_endpoint.clone(),
                    stable_user_key: a.stable_user_key.clone(),
                    gateway_token: None,
                    edge_slug: None,
                    edge_public_url: None,
                    pod_public_url: a.pod_public_url.clone(),
                });
                state.save();
                if !json { eprintln!("✓ Default pod {} allocated", a.pod_id); }
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
                let port = agent_ui_port(at);
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
            let pid_dir = secure_tytus_tmp_dir();
            let pid_f = pid_dir.join(format!("tunnel-{}.pid", target_pod_id));
            let iface_f = pid_dir.join(format!("tunnel-{}.iface", target_pod_id));
            let _ = std::fs::write(&pid_f, format!("{}", std::process::id()));
            secure_chmod_600(&pid_f);
            let _ = std::fs::write(&iface_f, &iface);
            secure_chmod_600(&iface_f);

            if let Some(pod) = state.pods.iter_mut().find(|p| p.pod_id == target_pod_id) {
                pod.tunnel_iface = Some(iface.clone());
            }
            state.save();

            if json {
                let pod = state.pods.iter().find(|p| p.pod_id == target_pod_id);
                println!("{}", serde_json::to_string_pretty(&pod).unwrap_or_default());
            } else {
                eprintln!("✓ Tunnel active on {}", iface);
                if !wizard::is_interactive() { append_autostart_log(&format!("cmd_connect OK: tunnel active on {}", iface)); }
                // SECURITY: Only print stable endpoint, never internal IPs or raw keys
                if let Some(pod) = state.pods.iter().find(|p| p.pod_id == target_pod_id) {
                    if let Some(ref ep) = pod.stable_ai_endpoint {
                        println!("ENDPOINT={}", ep);
                    } else if let Some(ref ep) = pod.ai_endpoint {
                        println!("ENDPOINT={}", ep);
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
    // Duplicate-tunnel guard: if a daemon for this pod is already
    // running, do NOT spawn a second one. Two boringtun instances on
    // the same WireGuard config share the UDP socket and each decrypts
    // every packet → halved throughput, ~2 minute page loads, browser
    // hangs. Discovered 2026-04-19: user clicked Open in Browser on a
    // pod that already had a tunnel, cmd_ui's auto-swap fired anyway
    // and we ended up with two utunN interfaces both at 10.18.2.2.
    //
    // Three ways a daemon can be live for this pod:
    //   1. Pidfile exists + pid alive + same pod (happy path).
    //   2. Pidfile missing but an orphan `tytus tunnel-up /tmp/tytus/
    //      tunnel-NN.json` process still runs (cleaned /tmp, crash).
    //   3. Pidfile exists but points at a different pid (stale). We
    //      skip this case — the caller's disconnect path handles it.
    let existing_alive = {
        let pidfile_pid = std::fs::read_to_string(format!("/tmp/tytus/tunnel-{}.pid", target_pod_id))
            .ok()
            .and_then(|s| s.trim().parse::<i32>().ok())
            .filter(|&pid| pid > 1 && unsafe {
                if libc::kill(pid, 0) == 0 { true }
                else { std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM) }
            });
        let orphan_pods = tunnel_reap::list_orphan_tunnel_pods();
        pidfile_pid.is_some() || orphan_pods.iter().any(|p| p == target_pod_id)
    };
    if existing_alive {
        // Dead-tunnel detection. `existing_alive` only means the
        // tunnel process is still running — it says NOTHING about
        // whether packets flow through the tunnel. A boringtun
        // tunnel can be "process-alive" but traffic-dead after a
        // network change, peer restart, or the 20-minute idle bug.
        // If we can't reach the gateway within 2s, the tunnel is
        // effectively dead — reap it and fall through to a fresh
        // activation instead of telling the user to run an extra
        // command for no reason.
        let gateway_reachable = probe_stable_gateway();
        if !gateway_reachable {
            if !json {
                eprintln!("✓ Tunnel for pod {} exists but gateway unreachable — reaping dead tunnel...", target_pod_id);
            }
            // Fire-and-forget disconnect so the kill path runs
            // through the same sudoers entry as a normal
            // `tytus disconnect`. Sleep briefly so the pidfile is
            // cleaned up before we try to re-bind.
            let _ = std::process::Command::new("tytus")
                .args(["disconnect", "--pod", &target_pod_id])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            std::thread::sleep(std::time::Duration::from_secs(1));
            // Fall through to the normal activation path below.
        } else {
            if json {
                println!("{}", serde_json::json!({
                    "pod_id": target_pod_id,
                    "status": "tunnel_already_up",
                    "action": "no-op",
                }));
            } else {
                eprintln!("✓ Tunnel for pod {} is already up — skipping duplicate activation", target_pod_id);
                eprintln!("  To replace: `tytus disconnect --pod {}` first, then reconnect.", target_pod_id);
            }
            return;
        }
    }

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

    // CRITICAL: the sudoers entry allows
    //   /Users/<user>/bin/tytus tunnel-up /tmp/tytus/tunnel-*.json
    // so the config MUST live under /tmp/tytus — not under the per-user
    // TMPDIR that std::env::temp_dir() picks up on macOS (which resolves
    // to /var/folders/<hash>/T/). If the paths differ, sudo's NOPASSWD
    // rule won't match, sudo falls through to the blanket `(ALL) ALL`
    // rule (which requires a password), and cmd_connect dies with a
    // confusing "sudo: a password is required" — every single time for
    // every single user, regardless of whether they installed the
    // sudoers drop-in. Root cause of the 2026-04-19 connect-from-tray
    // failure. Use the shared /tmp/tytus helper everywhere.
    let tmp_dir = secure_tytus_tmp_dir();
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
            if !wizard::is_interactive() { append_autostart_log(&format!("cmd_connect OK: tunnel active on {} (elevated)", iface)); }
            // SECURITY: Only print stable endpoint, never internal IPs or raw keys
            if let Some(pod) = state.pods.iter().find(|p| p.pod_id == target_pod_id) {
                if let Some(ref ep) = pod.stable_ai_endpoint {
                    println!("ENDPOINT={}", ep);
                } else if let Some(ref ep) = pod.ai_endpoint {
                    println!("ENDPOINT={}", ep);
                }
            }
            if let Some(pid) = tunnel_pid {
                eprintln!("Tunnel daemon running (pid {}). Stop with: tytus disconnect", pid);
            }
        }
    } else {
        // Tunnel didn't signal readiness — check if the child exited with error.
        // Drain stderr so the user sees the actual failure reason instead of
        // a bare "exit 1" — that hid bugs like "sudoers not installed" and
        // "osascript user cancelled" behind an identical error message.
        let exit = child.try_wait().ok().flatten();
        let mut stderr_dump = String::new();
        if let Some(stderr) = child.stderr.take() {
            use std::io::Read;
            let _ = std::io::BufReader::new(stderr).read_to_string(&mut stderr_dump);
        }
        let stderr_trim = stderr_dump.trim();
        if let Some(status) = exit {
            if !stderr_trim.is_empty() {
                eprintln!("Tunnel failed (exit {}):", status.code().unwrap_or(1));
                for line in stderr_trim.lines() {
                    eprintln!("  {}", line);
                }
            } else {
                eprintln!(
                    "Tunnel failed (exit {}). No stderr captured — check /tmp/tytus/tunnel-{}.log if it exists, \
                     or run `sudo -n {} tunnel-up {}` manually to see the error.",
                    status.code().unwrap_or(1),
                    target_pod_id, exe_str, config_path_str,
                );
            }
        } else {
            eprintln!("Tunnel did not start within 15 seconds.");
            if !stderr_trim.is_empty() {
                for line in stderr_trim.lines() {
                    eprintln!("  {}", line);
                }
            }
            let _ = child.kill();
        }
        std::process::exit(1);
    }
}

/// Try to spawn `tytus tunnel-up` with elevated privileges as a detached
/// background process.
///
/// The pre-sprint implementation here had a latent bug: strategy 1 only
/// checked whether `Command::spawn()` succeeded, which reports "process
/// launched" — NOT "process succeeded". Sudo itself can launch fine, then
/// reject the rule at runtime and exit with "sudo: a password is required".
/// The caller sees `Ok(child)` and never reaches strategies 2 and 3, so
/// the osascript GUI dialog + interactive-sudo fallbacks were effectively
/// dead code. Observed live 2026-04-19: passwordless rule present in
/// sudoers AND validated by manual `sudo -n` from the shell, but the Rust
/// spawn path still hit "password required" — likely because the child's
/// process group / timestamp context differs from an interactive shell.
///
/// Fix: use `sudo -n -l <argv>` as a side-effect-free pre-check. It exits
/// 0 iff the exact command is allowed without a password under this
/// user's current sudo context. If the precheck fails, we skip straight
/// to strategy 2 (GUI) without wasting a doomed spawn attempt and without
/// returning a child that will die with an unhelpful "exit 1".
fn try_spawn_elevated(
    exe: &str,
    args: &[String],
    config_path: &str,
    json_flag: &str,
) -> Result<std::process::Child, String> {
    // Strategy 1: `sudo -n` (passwordless). Unlike the pre-sprint version,
    // we actually verify sudo succeeded rather than trusting spawn(). Flow:
    //   1. `output()` the sudo command — blocks until sudo exits (or tytus
    //      tunnel-up signals TUNNEL_READY then exits).
    // Wait — output() won't work because we need the child ALIVE to read
    // TUNNEL_READY and to hold the tunnel daemon. So instead:
    //   1. Spawn sudo -n
    //   2. Give it a brief moment (~250ms) to either exit with an error OR
    //      emit its first stdout byte / still be running happily.
    //   3. If it already exited with non-zero, capture stderr and decide
    //      whether to retry via osascript (for "password required") or
    //      bubble the error up.
    // This is the reliable way to distinguish "sudo exec'd tytus" from
    // "sudo refused the rule" with this version of macOS sudo.
    match std::process::Command::new("sudo")
        .arg("-n")
        .arg(exe)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            // Short settle window. Sudo exits almost instantly on rule
            // mismatch (< 50ms). If the child is still alive after 250ms,
            // sudo exec'd the inner command and we should return the child
            // to the caller who will read TUNNEL_READY from its stdout.
            std::thread::sleep(std::time::Duration::from_millis(250));
            match child.try_wait() {
                Ok(None) => return Ok(child), // still running — sudo passed through
                Ok(Some(status)) if status.success() => return Ok(child),
                Ok(Some(_status)) => {
                    // Sudo or the inner command exited quickly. If stderr
                    // indicates "password required" it's a sudoers mismatch
                    // and strategy 2 (osascript) can still save us. For
                    // any other exit reason (tytus tunnel-up itself
                    // failed) re-raise — the outer caller prints stderr.
                    let mut stderr = String::new();
                    if let Some(s) = child.stderr.take() {
                        use std::io::Read;
                        let _ = std::io::BufReader::new(s).read_to_string(&mut stderr);
                    }
                    let needs_password = stderr.contains("password is required")
                        || stderr.contains("no tty present")
                        || stderr.contains("may not run");
                    if !needs_password {
                        return Err(format!("sudo -n failed: {}", stderr.trim()));
                    }
                    // Fall through to strategy 2. Note: child is already dead.
                    tracing::info!("sudo -n declined ({}), trying osascript", stderr.trim());
                }
                Err(e) => {
                    tracing::warn!("sudo -n try_wait failed: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::warn!("sudo -n spawn failed: {} — falling back to osascript", e);
        }
    }

    // Strategy 2: osascript on macOS (GUI password dialog / Touch ID).
    // This is the primary user-facing path when the sudoers rule either
    // doesn't exist (fresh install) or refuses to match the Rust-spawned
    // argv for process-group / canonicalization reasons.
    #[cfg(target_os = "macos")]
    {
        let cmd = format!(
            "{} tunnel-up {}{}",
            shell_escape(exe),
            shell_escape(config_path),
            json_flag,
        );
        match std::process::Command::new("osascript")
            .args(["-e", &format!(
                "do shell script \"{}\" with administrator privileges",
                cmd.replace('\\', "\\\\").replace('"', "\\\"")
            )])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(child) => return Ok(child),
            Err(e) => {
                tracing::warn!("osascript spawn failed: {} — falling back to interactive sudo", e);
            }
        }
    }

    // Strategy 3: interactive sudo (terminal required). Last resort when
    // the user has no sudoers rule AND we can't pop a GUI dialog (Linux,
    // or a macOS context with no WindowServer — very unusual).
    std::process::Command::new("sudo")
        .arg(exe)
        .args(args)
        .stdin(std::process::Stdio::inherit()) // needs terminal for password
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("All elevation strategies failed: {}", e))
}

// (Removed: the earlier `sudo -n -l` precheck produced false positives.
// A blanket `(ALL) ALL` rule causes `sudo -l` to exit 0 even when the
// real invocation would fall through to that rule and demand a password.
// try_spawn_elevated now spawns + settles + inspects stderr instead.)

/// Hidden subcommand: runs as root, reads tunnel config from temp file, activates tunnel.
/// Runs as a background daemon — writes PID file, detaches from terminal, handles SIGTERM.
async fn cmd_tunnel_up(config_file: &str, _json: bool) {
    // FIX-5: proper daemon detach.
    //
    // The previous implementation inherited the parent shell's session, so
    // when the user (or Claude Code, or systemd, or anything) closed their
    // terminal, the session-wide SIGHUP also killed our tunnel daemon. A
    // real paying customer running `tytus connect` in their own terminal
    // would lose their tunnel the moment they closed the window.
    //
    // setsid() creates a new session with this process as the session leader.
    // The new session has no controlling terminal, so SIGHUP from the old
    // controlling TTY is no longer delivered. The daemon lives independent
    // of whoever spawned it, as a proper Unix daemon should.
    //
    // Also ignore SIGHUP and SIGPIPE explicitly:
    //   - SIGHUP: belt-and-suspenders in case setsid() fails for some reason.
    //   - SIGPIPE: CRITICAL. The daemon's stdout/stderr are piped back to the
    //     spawning `tytus connect` process so it can read TUNNEL_READY. When
    //     the parent exits (moments after reading that line), those pipes
    //     are closed. The first subsequent write from the daemon — any
    //     `tracing::warn!`, `println!`, keepalive log, or watchdog message —
    //     would hit a broken pipe and the default SIGPIPE handler would
    //     terminate the daemon. Observed live: tunnels died 3-4 minutes
    //     after `tytus connect` returned, exactly when the first post-setup
    //     log line fired.
    //
    // Safety: setsid() is safe to call from a non-session-leader (which we
    // are, because sudo is our parent and sudo is the session leader).
    #[cfg(unix)]
    unsafe {
        libc::setsid();
        libc::signal(libc::SIGHUP, libc::SIG_IGN);
        libc::signal(libc::SIGPIPE, libc::SIG_IGN);
    }

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

    // FIX-4: post-mortem log file so we can diagnose silent packet-loop exits.
    // Daemon stdout/stderr get orphaned once `tytus connect` returns; without a
    // persistent log, we have no way to see why the packet loop died. Write
    // everything (tracing + our own println!s) to /tmp/tytus/tunnel-NN.log
    // so users + support can recover context without re-running with debug env.
    let pid_dir = secure_tytus_tmp_dir();
    let log_file_path = pid_dir.join(format!("tunnel-{}.log", pod_id));
    // Use a tracing-subscriber appender writing to this file; if it fails we
    // silently fall back to the existing stderr subscriber (already init'd in main).
    if let Ok(log_file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
    {
        // Best-effort: attach a file writer on top of the existing subscriber.
        // We do this via a one-shot println so the file is at least touched
        // and users can tail -f it.
        use std::io::Write as _;
        let mut lf = &log_file;
        let _ = writeln!(
            lf,
            "[{}] tunnel-up pod={} pid={} starting",
            chrono_now_utc_iso(),
            pod_id,
            std::process::id()
        );
        secure_chmod_600(&log_file_path);
    }

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
        Ok(mut handle) => {
            let iface = handle.interface_name.clone();

            // Write PID file so `tytus disconnect` can find and stop us.
            // 0644 (NOT 0600): the tunnel daemon runs as root, but the
            // user-space `tytus disconnect` and `tytus-tray` read this
            // file to locate + SIGTERM the daemon. With 0600 root:wheel
            // the read_to_string silently fails (permission denied), the
            // reap function reports "NoPidfile" and the daemon lingers
            // forever. Pid isn't secret — `ps` exposes every pid on the
            // machine. Discovered 2026-04-19 during sprint smoke test:
            // disconnect said "nothing to reap" yet utun5 kept routing
            // + a root boringtun process kept running.
            let pid_file = pid_dir.join(format!("tunnel-{}.pid", pod_id));
            let _ = std::fs::write(&pid_file, format!("{}", std::process::id()));
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&pid_file, std::fs::Permissions::from_mode(0o644));
            }

            // Write interface name so parent process can read it.
            // Same 0644 reasoning — interface name is advertised by
            // ifconfig already, no secrecy value.
            let iface_file = pid_dir.join(format!("tunnel-{}.iface", pod_id));
            let _ = std::fs::write(&iface_file, &iface);
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&iface_file, std::fs::Permissions::from_mode(0o644));
            }

            // Signal to parent that tunnel is ready (print to stdout for capture)
            println!("TUNNEL_READY iface={} pid={}", iface, std::process::id());
            use std::io::Write as _;
            let _ = std::io::stdout().flush();

            // FIX-5 continued: redirect stdout/stderr to /dev/null so that
            // the moment the spawning `tytus connect` process exits (and
            // its end of the pipe closes), we don't blow up on the first
            // subsequent write. We kept the original fds open just long
            // enough to print TUNNEL_READY above; now we swap them out.
            // Tracing's existing subscriber (pointed at stderr) will now
            // silently discard events — the real diagnostic path is the
            // /tmp/tytus/tunnel-NN.log file opened by FIX-4.
            #[cfg(unix)]
            unsafe {
                let devnull = libc::open(c"/dev/null".as_ptr(), libc::O_RDWR);
                if devnull >= 0 {
                    libc::dup2(devnull, 0); // stdin
                    libc::dup2(devnull, 1); // stdout
                    libc::dup2(devnull, 2); // stderr
                    if devnull > 2 {
                        libc::close(devnull);
                    }
                }
            }

            // FIX-4: race ctrl_c AGAINST the packet-loop task. Previously we only
            // waited on ctrl_c, so if the packet loop exited silently (TUN drop,
            // panic, unrecoverable error) the daemon sat here forever pretending
            // to be alive while utun was gone. Now we observe both and exit
            // loudly on unexpected task completion.
            let log_path_clone = log_file_path.clone();
            let mut task = handle.take_task();

            // SIGTERM handler: the standard "please exit" signal. Without this,
            // SIGTERM kills the daemon instantly — no log, no PID cleanup.
            // macOS sends SIGTERM on system sleep, shutdown, launchd stop, and
            // when sudo's session expires. This was the root cause of silent
            // tunnel deaths during the headless-auth sprint testing.
            let mut sigterm = tokio::signal::unix::signal(
                tokio::signal::unix::SignalKind::terminate(),
            ).expect("Failed to register SIGTERM handler");

            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    append_log(&log_path_clone, &format!("tunnel-up pod={} pid={} received SIGINT — shutting down cleanly", pod_id, std::process::id()));
                    handle.cancel_token().cancel();
                    let _ = (&mut task).await;
                }
                _ = sigterm.recv() => {
                    append_log(&log_path_clone, &format!("tunnel-up pod={} pid={} received SIGTERM — shutting down cleanly", pod_id, std::process::id()));
                    handle.cancel_token().cancel();
                    let _ = (&mut task).await;
                }
                res = &mut task => {
                    let msg = match res {
                        Ok(()) => "packet_loop exited unexpectedly (Ok) — TUN device is dropped, tunnel is effectively dead".to_string(),
                        Err(e) => format!("packet_loop task join failed: {}", e),
                    };
                    eprintln!("[tunnel-up] {}", msg);
                    append_log(&log_path_clone, &format!("FATAL tunnel-up pod={} pid={}: {}", pod_id, std::process::id(), msg));
                    // Clean up pidfile so disconnect/connect can recover
                    let _ = std::fs::remove_file(&pid_file);
                    let _ = std::fs::remove_file(&iface_file);
                    std::process::exit(2);
                }
            }

            // Clean up PID + iface files
            let _ = std::fs::remove_file(&pid_file);
            let _ = std::fs::remove_file(&iface_file);
        }
        Err(e) => {
            eprintln!("Tunnel failed: {}", e);
            append_log(&log_file_path, &format!("FATAL tunnel-up pod={} failed to connect: {}", pod_id, e));
            std::process::exit(1);
        }
    }
}

fn chrono_now_utc_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("epoch={}", secs)
}

fn append_log(path: &std::path::Path, msg: &str) {
    use std::io::Write as _;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = writeln!(f, "[{}] {}", chrono_now_utc_iso(), msg);
        secure_chmod_600(path);
    }
}

/// Ensure `/tmp/tytus/` (or caller-supplied equivalent) exists with mode 0700.
///
/// Security: files under this directory include tunnel PID/iface/log files,
/// autostart diagnostic logs, and the daemon socket. World-readable defaults
/// would let any local user list tunnel state and read diagnostic output
/// (pod IDs, timestamps, error messages). See PENTEST finding E5.
///
/// This is best-effort: if the directory already exists and is owned by a
/// different uid (e.g. root created it during an earlier tunnel-up run), the
/// chmod may silently fail. That is acceptable — the per-file 0600 chmod
/// below is the actual enforcement layer.
pub(crate) fn secure_tytus_tmp_dir() -> std::path::PathBuf {
    let dir = std::path::PathBuf::from("/tmp/tytus");
    let _ = std::fs::create_dir_all(&dir);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
    }
    dir
}

/// Best-effort chmod to 0600 on a just-created file. Call after every write
/// into `/tmp/tytus/` so pod metadata never becomes world-readable.
pub(crate) fn secure_chmod_600(path: &std::path::Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    { let _ = path; }
}

// ── Tunnel down (validated SIGTERM, replaces direct sudo kill) ──
//
// SECURITY: this subcommand exists so the passwordless sudoers entry
// can be scoped to `tytus tunnel-down *` instead of `/bin/kill -TERM *`.
// The previous design let any local user SIGTERM ANY process (including
// PID 1, system services, other users' processes) as root. This helper
// validates the PID is one of OUR own tunnel daemons before signalling.
//
// Validation:
//   1. The PID must appear in /tmp/tytus/tunnel-*.pid (the daemon's
//      own breadcrumb) — if no file references it, refuse.
//   2. The process must currently exist (kill -0 returns 0).
// We deliberately do NOT call `ps`/`/proc/PID/exe` for portability and
// to avoid TOCTOU between the comm check and the kill — the PID-file
// check is sufficient because only root could have written that file
// (the daemon runs as root, the file lives in a sticky-bit /tmp dir).
fn cmd_tunnel_down(pid: i32) {
    if pid <= 1 {
        eprintln!("tunnel-down: refusing to signal PID {}", pid);
        std::process::exit(1);
    }

    let pid_dir = std::path::PathBuf::from("/tmp/tytus");
    let entries = match std::fs::read_dir(&pid_dir) {
        Ok(e) => e,
        Err(_) => {
            eprintln!("tunnel-down: no tunnel daemons known (no /tmp/tytus dir)");
            std::process::exit(1);
        }
    };

    let mut matched = false;
    let mut matched_path: Option<std::path::PathBuf> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !(name.starts_with("tunnel-") && name.ends_with(".pid")) {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(file_pid) = content.trim().parse::<i32>() {
                if file_pid == pid {
                    matched = true;
                    matched_path = Some(path.clone());
                    break;
                }
            }
        }
    }

    if !matched {
        eprintln!("tunnel-down: PID {} is not a registered tytus tunnel daemon", pid);
        std::process::exit(1);
    }

    // Verify the process still exists (kill -0 = signal 0 = check only)
    let alive = unsafe { libc::kill(pid, 0) } == 0;
    if !alive {
        // Stale PID file — clean it up and exit success
        if let Some(p) = matched_path { let _ = std::fs::remove_file(p); }
        eprintln!("tunnel-down: PID {} already exited (stale pidfile cleaned)", pid);
        std::process::exit(0);
    }

    // Send SIGTERM
    let result = unsafe { libc::kill(pid, libc::SIGTERM) };
    if result == 0 {
        eprintln!("tunnel-down: SIGTERM sent to PID {}", pid);
        std::process::exit(0);
    } else {
        let err = std::io::Error::last_os_error();
        eprintln!("tunnel-down: kill({}, SIGTERM) failed: {}", pid, err);
        std::process::exit(1);
    }
}

// ── Agent commands (install / uninstall / replace / list / catalog) ─
//
// Decouples pod allocation from agent deployment. Default pods (agent-less,
// AIL-only, 0 units) come free on login; agents cost plan units and are
// installed explicitly per SPRINT-AIL-DEFAULT-POD §4.1 / §6 B.
//
// These implementations lean on the existing Provider endpoints (/pod/request,
// /pod/agent/deploy, /pod/agent/stop) rather than the proposed
// /agent/install/uninstall/replace triad in §8 — the existing endpoints
// cover the full behavior, and keeping the Provider surface unchanged in
// this sprint lets Phase B ship independently of a Provider deploy.

async fn cmd_agent(http: &atomek_core::HttpClient, action: AgentAction, json: bool) {
    match action {
        AgentAction::Install { name, pod, force } => {
            let _ = cmd_agent_install(http, &name, pod, force, json).await;
        }
        AgentAction::Uninstall { pod } => cmd_agent_uninstall(http, &pod, json).await,
        AgentAction::List => cmd_agent_list(http, json).await,
        AgentAction::Catalog { refresh } => cmd_agent_catalog(http, refresh, json).await,
    }
}

/// Normalize a user-provided agent name to the backend (Scalesys + DAM)
/// identifier. The public brand name is "OpenClaw" but the Docker image +
/// agent_type enum is `nemoclaw` (the NemoClaw safety harness that runs
/// OpenClaw inside). Accepting the alias lets users type either name.
///
/// Keep this list short. If the catalog grows, make the Provider return
/// an `aliases` array per agent and resolve client-side from there.
fn normalize_agent_name(input: &str) -> String {
    match input.to_ascii_lowercase().as_str() {
        "openclaw" => "nemoclaw".to_string(),
        other => other.to_string(),
    }
}

/// Install an agent. Returns the pod id the agent landed on so callers
/// (notably the `tytus connect --agent X` shim) can activate the tunnel
/// targetting the right slot instead of defaulting back to the default pod.
async fn cmd_agent_install(
    http: &atomek_core::HttpClient,
    name: &str,
    pod_id: Option<String>,
    force: bool,
    json: bool,
) -> Option<String> {
    // Accept `openclaw` as the public name; backend still speaks
    // `nemoclaw` (the internal harness identifier).
    let name = normalize_agent_name(name);
    let name = name.as_str();
    let mut state = CliState::load();
    if !state.is_logged_in() {
        eprintln!("Not logged in. Run: tytus login");
        std::process::exit(1);
    }
    if let Err(e) = ensure_token(&mut state, http).await {
        eprintln!("Token refresh failed: {}. Run: tytus login", e);
        std::process::exit(1);
    }
    let (sk, auid) = get_credentials(&mut state, http).await;
    let client = atomek_pods::TytusClient::new(http, &sk, &auid);

    match pod_id {
        None => {
            // Allocate new pod + deploy agent atomically via /pod/request.
            if !json { eprintln!("Allocating pod with {}...", name); }
            match atomek_pods::request_pod_with_agent(&client, name).await {
                Ok(a) => {
                    let returned_pod_id = a.pod_id.clone();
                    // Preserve tunnel_iface if this pod_id is being
                    // re-provisioned (rare — typically only happens when
                    // Scalesys stable-reuse returns a slot that had a
                    // live tunnel). See ensure_default_pod for the same
                    // pattern + rationale.
                    let preserved_iface = state.pods.iter()
                        .find(|p| p.pod_id == a.pod_id)
                        .and_then(|p| p.tunnel_iface.clone());
                    state.pods.retain(|p| p.pod_id != a.pod_id);
                    state.pods.push(PodEntry {
                        pod_id: a.pod_id.clone(),
                        droplet_id: a.droplet_id.clone(),
                        droplet_ip: a.droplet_ip.clone(),
                        ai_endpoint: a.ai_endpoint.clone(),
                        pod_api_key: a.pod_api_key.clone(),
                        agent_type: a.agent_type.clone().or_else(|| Some(name.to_string())),
                        agent_endpoint: a.agent_endpoint.clone(),
                        tunnel_iface: preserved_iface,
                        stable_ai_endpoint: a.stable_ai_endpoint.clone(),
                        stable_user_key: a.stable_user_key.clone(),
                        gateway_token: None,
                        edge_slug: None,
                        edge_public_url: None,
                        pod_public_url: a.pod_public_url.clone(),
                    });
                    state.save();
                    if json {
                        println!("{}", serde_json::json!({
                            "pod_id": a.pod_id, "agent_type": name,
                            "stable_ai_endpoint": a.stable_ai_endpoint,
                        }));
                    } else {
                        println!("✓ {} installed on pod {}", name, a.pod_id);
                        println!("  Activate: tytus connect --pod {}", a.pod_id);
                    }
                    // Zero-config hook: patch the agent to not demand a
                    // gateway token. See `configure_agent_for_zero_auth`
                    // for rationale. Best-effort, non-fatal on failure.
                    let _ = configure_agent_for_zero_auth(&client, &a.pod_id, name, json).await;
                    Some(returned_pod_id)
                }
                Err(e) => {
                    eprintln!("Install failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(pid) => {
            // Deploy into an existing pod. Refuse to clobber unless --force.
            let existing = state.pods.iter().find(|p| p.pod_id == pid);
            if let Some(p) = existing {
                match p.agent_type.as_deref() {
                    Some("none") | None => {} // empty slot — safe to install
                    Some(other) if !force => {
                        eprintln!(
                            "Pod {} already has {} installed. Use --force to replace \
                             (or `tytus agent replace {} {}`).",
                            pid, other, pid, name
                        );
                        std::process::exit(1);
                    }
                    _ => {} // --force given
                }
            } else {
                eprintln!("Pod {} not found in local state. Run: tytus status", pid);
                std::process::exit(1);
            }

            if !json { eprintln!("Installing {} on pod {}...", name, pid); }
            match atomek_pods::deploy_agent(&client, &pid, name).await {
                Ok(_) => {
                    if let Some(p) = state.pods.iter_mut().find(|p| p.pod_id == pid) {
                        p.agent_type = Some(name.to_string());
                        p.agent_endpoint = None; // will be rehydrated on next connect
                    }
                    state.save();
                    if json {
                        println!("{}", serde_json::json!({"pod_id": pid, "agent_type": name}));
                    } else {
                        println!("✓ {} installed on pod {}", name, pid);
                    }
                    let _ = configure_agent_for_zero_auth(&client, &pid, name, json).await;
                    Some(pid)
                }
                Err(e) => {
                    eprintln!("Deploy failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

/// Zero-config hook run after every successful `tytus agent install`.
///
/// OpenClaw's default config requires a per-install random token that
/// the user has to manually paste into the web UI's login form. Since
/// the pod is only reachable through the user's private WireGuard
/// tunnel, that extra token is redundant at the threat-model level
/// (nothing else on the network can even route to 10.X.Y.1:3000), but
/// the friction derails the "click Open in Browser → use it" flow.
/// Additionally, allowedOrigins defaults to the pod's own IP and
/// localhost:3000 — not our forwarder's per-pod port.
///
/// This hook (a) flips `gateway.auth.mode` to `"none"` so the WebSocket
/// upgrade accepts any client, and (b) expands allowedOrigins to
/// include `http://localhost:187NN` + `http://127.0.0.1:187NN` so
/// future port changes don't re-introduce the origin-not-allowed
/// bounce. Both live-patch the in-container config.json then restart
/// the agent so the new config takes effect.
///
/// Best-effort: on any failure (container still starting, provider
/// unreachable, script parse error) we log a warning and leave the
/// user to deal with the default auth manually. We never abort the
/// install for this.
async fn configure_agent_for_zero_auth(
    client: &atomek_pods::TytusClient,
    pod_id: &str,
    agent_type: &str,
    json: bool,
) -> Result<(), String> {
    match agent_type {
        "hermes" => configure_hermes_for_zero_auth(client, pod_id, json).await,
        // nemoclaw is the default; other values fall through to the
        // openclaw-style path since they're all likely OpenClaw-derived.
        _ => configure_nemoclaw_for_zero_auth(client, pod_id, json).await,
    }
}

/// Zero-config hook for Hermes pods. Hermes is OpenAI-compatible HTTP
/// (no pairing, no browser chat UI) and auth is a single shared secret
/// in the `API_SERVER_KEY` env var. The updated entrypoint.sh derives
/// that key deterministically from `AIL_API_KEY + TYTUS_POD_ID` and
/// writes it to `/app/workspace/.hermes/api_server_key`. We read that
/// file here and stash it in state.json as `gateway_token` so the
/// forwarder can inject `Authorization: Bearer <key>` on every SDK /
/// curl request — user never pastes it.
async fn configure_hermes_for_zero_auth(
    client: &atomek_pods::TytusClient,
    pod_id: &str,
    json: bool,
) -> Result<(), String> {
    let script = "cat /app/workspace/.hermes/api_server_key 2>/dev/null";
    match atomek_pods::exec_in_agent(client, pod_id, script, 10).await {
        Ok(r) if r.exit_code == 0 => {
            let key = r.stdout.as_deref().unwrap_or("").trim().to_string();
            if key.is_empty() {
                if !json { eprintln!("  (hermes zero-config: API_SERVER_KEY file empty — is this an older image?)"); }
                return Err("api_server_key file empty".into());
            }
            let mut state = CliState::load();
            if let Some(p) = state.pods.iter_mut().find(|p| p.pod_id == pod_id) {
                p.gateway_token = Some(key.clone());
                state.save();
            }
            if !json {
                println!("  Zero-config: Hermes API key cached ({} chars). Use any OpenAI SDK against localhost:187NN.", key.len());
            }
            Ok(())
        }
        Ok(r) => {
            let err = r.stderr.as_deref().unwrap_or("").trim().to_string();
            if !json { eprintln!("  (hermes zero-config: exec exit {}: {})", r.exit_code, err); }
            Err(format!("exit {}: {}", r.exit_code, err))
        }
        Err(e) => {
            if !json { eprintln!("  (hermes zero-config: exec failed: {})", e); }
            Err(e.to_string())
        }
    }
}

async fn configure_nemoclaw_for_zero_auth(
    client: &atomek_pods::TytusClient,
    pod_id: &str,
    json: bool,
) -> Result<(), String> {
    // 1. Write a `config.user.json` overlay that adds the forwarder's
    //    per-pod localhost port to `gateway.controlUi.allowedOrigins`.
    //    Critical: the previous implementation patched `config.json`
    //    directly, but nemoclaw-configure.sh REGENERATES that file
    //    from scratch on every container restart — wiping our patch.
    //    The overlay pattern (`config.user.json`) is deep-merged on
    //    top on every restart, so the origins stick forever.
    // 2. Fetch the agent's gateway.auth.token — the agent keeps its
    //    token across restarts (deterministic from AIL_API_KEY) so
    //    this is stable. Stash in state.json so the forwarder can
    //    seed it into the browser URL and skip the "paste token"
    //    form on first load.
    //
    // The controlUi origins must include both `localhost:<port>` AND
    // `127.0.0.1:<port>` — browsers differ on which they use when the
    // user hits `http://localhost:...`. Silent local pairing fires
    // when origin + host BOTH resolve to loopback (see
    // isControlUiBrowserContainerLocalEquivalent in server.impl).
    let pod_num: u16 = pod_id.parse().unwrap_or(0);
    let fwd_port = 18700u16.saturating_add(pod_num);
    // Per-pod-subdomain sprint (2026-04-23): the browser may also load the
    // UI from `https://{slug}-pNN.tytus.traylinx.com` (per-pod) or the
    // legacy `https://{slug}.tytus.traylinx.com/p/NN/...`. OpenClaw's
    // origin check is exact-string, not pattern — both URLs must be in
    // allowedOrigins or the WebSocket upgrade fails with "origin not
    // allowed". Pull from state so we don't have to rebuild the pattern.
    let (pod_public_url, edge_public_url) = {
        let st = CliState::load();
        let p = st.pods.into_iter().find(|p| p.pod_id == pod_id);
        match p {
            Some(p) => (p.pod_public_url, p.edge_public_url),
            None => (None, None),
        }
    };
    let mut origins: Vec<serde_json::Value> = vec![
        serde_json::Value::String("http://localhost:3000".into()),
        serde_json::Value::String("http://127.0.0.1:3000".into()),
        serde_json::Value::String(format!("http://10.18.{}.1:3000", pod_num)),
        serde_json::Value::String(format!("http://localhost:{}", fwd_port)),
        serde_json::Value::String(format!("http://127.0.0.1:{}", fwd_port)),
    ];
    if let Some(u) = pod_public_url.as_deref() { origins.push(serde_json::Value::String(u.to_string())); }
    if let Some(u) = edge_public_url.as_deref() { origins.push(serde_json::Value::String(u.to_string())); }
    let overlay_json = serde_json::json!({
        "gateway": {
            "controlUi": {
                "allowedOrigins": origins
            }
        }
    }).to_string();
    // Base64 the overlay to sidestep every shell-quoting pitfall (single
    // quotes inside shell commands, heredoc delimiter choice, JSON with
    // embedded backslashes). The node side decodes once and writes the
    // file atomically. On success we print the current gateway.auth.token
    // from config.json so the caller can stash it in state.json.
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(overlay_json.as_bytes());
    let script = format!(
        "node -e \"const fs=require('fs'); \
         fs.writeFileSync('/app/workspace/.openclaw/config.user.json', Buffer.from('{b64}','base64').toString('utf8')); \
         const c=JSON.parse(fs.readFileSync('/app/workspace/.openclaw/config.json','utf8')); \
         process.stdout.write((c.gateway&&c.gateway.auth&&c.gateway.auth.token)||'')\"",
        b64 = b64,
    );

    match atomek_pods::exec_in_agent(client, pod_id, &script, 15).await {
        Ok(r) if r.exit_code == 0 => {
            let token = r.stdout.as_deref().unwrap_or("").trim().to_string();
            if !token.is_empty() {
                let mut state = CliState::load();
                if let Some(p) = state.pods.iter_mut().find(|p| p.pod_id == pod_id) {
                    p.gateway_token = Some(token.clone());
                    state.save();
                }
            }
            if let Err(e) = atomek_pods::restart_agent(client, pod_id).await {
                if !json { eprintln!("  (zero-config: restart failed: {})", e); }
                return Err(e.to_string());
            }
            if !json {
                println!(
                    "  Zero-config: allowedOrigins patched, gateway token cached ({} chars).",
                    token.len()
                );
            }
            Ok(())
        }
        Ok(r) => {
            let err = r.stderr.as_deref().unwrap_or("").trim().to_string();
            if !json {
                eprintln!("  (zero-config: script exit {}: {})", r.exit_code, err);
            }
            Err(format!("exit {}: {}", r.exit_code, err))
        }
        Err(e) => {
            if !json { eprintln!("  (zero-config: exec failed: {})", e); }
            Err(e.to_string())
        }
    }
}

async fn cmd_agent_uninstall(http: &atomek_core::HttpClient, pod_id: &str, json: bool) {
    let mut state = CliState::load();
    if !state.is_logged_in() {
        eprintln!("Not logged in. Run: tytus login");
        std::process::exit(1);
    }
    if let Err(e) = ensure_token(&mut state, http).await {
        eprintln!("Token refresh failed: {}. Run: tytus login", e);
        std::process::exit(1);
    }
    let (sk, auid) = get_credentials(&mut state, http).await;
    let client = atomek_pods::TytusClient::new(http, &sk, &auid);

    if !json { eprintln!("Stopping agent on pod {}...", pod_id); }
    match atomek_pods::stop_agent(&client, pod_id).await {
        Ok(()) => {
            if let Some(p) = state.pods.iter_mut().find(|p| p.pod_id == pod_id) {
                // Keep the slot allocated — AIL still works through the
                // sidecar. Caller uses `tytus revoke` to fully free units.
                p.agent_type = Some("none".to_string());
                p.agent_endpoint = None;
            }
            state.save();
            if json {
                println!("{}", serde_json::json!({"pod_id": pod_id, "agent_type": serde_json::Value::Null}));
            } else {
                println!("✓ Agent stopped on pod {}. Pod slot retained.", pod_id);
                println!("  To fully free units: tytus revoke {}", pod_id);
            }
        }
        Err(e) => {
            eprintln!("Uninstall failed: {}", e);
            std::process::exit(1);
        }
    }
}

async fn cmd_agent_list(http: &atomek_core::HttpClient, json: bool) {
    let mut state = CliState::load();
    if !state.is_logged_in() {
        eprintln!("Not logged in. Run: tytus login");
        std::process::exit(1);
    }
    if let Err(e) = ensure_token(&mut state, http).await {
        eprintln!("Token refresh failed: {}. Run: tytus login", e);
        std::process::exit(1);
    }
    sync_tytus(&mut state, http).await;
    state.save();

    if json {
        let pods: Vec<_> = state.pods.iter().map(|p| {
            serde_json::json!({
                "pod_id": p.pod_id,
                "agent_type": p.agent_type,
                "tunnel_iface": p.tunnel_iface,
                "stable_ai_endpoint": p.stable_ai_endpoint,
            })
        }).collect();
        println!("{}", serde_json::to_string_pretty(&pods).unwrap_or_default());
        return;
    }

    if state.pods.is_empty() {
        println!("No pods. Run: tytus connect (for AIL) or tytus agent install <name>");
        return;
    }
    println!("{:<6} {:<12} {:<10} ENDPOINT", "POD", "AGENT", "TUNNEL");
    for p in &state.pods {
        let agent = p.agent_type.as_deref().unwrap_or("-");
        let tunnel = p.tunnel_iface.as_deref().unwrap_or("down");
        let endpoint = p.stable_ai_endpoint.as_deref().unwrap_or("http://10.42.42.1:18080");
        // Display the public brand name; keep the internal identifier
        // consistent in --json output (elsewhere) for scripting.
        let label = match agent {
            "none" => "default",
            "nemoclaw" => "OpenClaw",
            "hermes" => "Hermes",
            other => other,
        };
        println!("{:<6} {:<12} {:<10} {}", p.pod_id, label, tunnel, endpoint);
    }
}

async fn cmd_agent_catalog(http: &atomek_core::HttpClient, refresh: bool, json: bool) {
    match atomek_pods::fetch_catalog(http, refresh).await {
        Ok(cat) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&cat).unwrap_or_default());
                return;
            }
            println!("Agent catalog (version {})", cat.version);
            println!();
            for a in &cat.agents {
                let tagline = a.tagline.as_deref().unwrap_or("");
                let min_plan = a.min_plan.as_deref().unwrap_or("any");
                println!("  {} — {} unit(s), min plan: {}", a.name, a.units, min_plan);
                if !tagline.is_empty() { println!("    {}", tagline); }
                if let Some(ref desc) = a.description { println!("    {}", desc); }
                println!("    Install: tytus agent install {}", a.id);
                println!();
            }
        }
        Err(e) => {
            eprintln!("Catalog fetch failed: {}", e);
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

    if let Err(e) = ensure_token(&mut state, http).await {
        eprintln!("Token refresh failed: {}. Run: tytus login", e);
        std::process::exit(1);
    }
    let (sk, auid) = get_credentials(&mut state, http).await;
    let client = atomek_pods::TytusClient::new(http, &sk, &auid);

    if !json {
        println!("Revoking pod {}...", pod_id);
    }

    // FIX-3: Reap the root-owned tunnel daemon BEFORE calling the Provider
    // API. This prevents the zombie-daemon leak where `tytus revoke` wiped
    // local state but left `tytus tunnel-up` running, holding the utun
    // interface and routes. If the reap fails we log a warning and press on
    // — the user explicitly asked to destroy this pod, so an orphan daemon
    // should never block the API call.
    let reap_outcome = tunnel_reap::reap_tunnel_for_pod(pod_id);
    match &reap_outcome {
        tunnel_reap::ReapOutcome::Reaped { pid } => {
            tracing::info!("revoke: reaped tunnel daemon pid={} for pod {}", pid, pod_id);
        }
        tunnel_reap::ReapOutcome::StalePidfile { pid } => {
            tracing::info!(
                "revoke: cleaned stale pidfile (pid={} already dead) for pod {}",
                pid,
                pod_id
            );
        }
        tunnel_reap::ReapOutcome::NoPidfile => {
            tracing::debug!("revoke: no tunnel pidfile for pod {} — nothing to reap", pod_id);
        }
        tunnel_reap::ReapOutcome::ReapFailed { pid, reason } => {
            tracing::warn!(
                "revoke: could not reap tunnel daemon pid={} for pod {}: {} — \
                 proceeding with revoke anyway",
                pid,
                pod_id,
                reason
            );
        }
    }

    match atomek_pods::revoke_pod(&client, pod_id).await {
        Ok(_) => {
            state.pods.retain(|p| p.pod_id != pod_id);
            state.save();
            if json {
                let (reap_status, reap_pid) = match &reap_outcome {
                    tunnel_reap::ReapOutcome::Reaped { pid } => ("reaped", Some(*pid)),
                    tunnel_reap::ReapOutcome::StalePidfile { pid } => ("stale", Some(*pid)),
                    tunnel_reap::ReapOutcome::NoPidfile => ("none", None),
                    tunnel_reap::ReapOutcome::ReapFailed { pid, .. } => {
                        ("failed", Some(*pid))
                    }
                };
                let payload = serde_json::json!({
                    "status": "revoked",
                    "pod_id": pod_id,
                    "reap": {
                        "status": reap_status,
                        "pid": reap_pid,
                    }
                });
                println!("{}", payload);
            } else {
                let suffix = reap_outcome.human_suffix();
                if suffix.is_empty() {
                    println!("✓ Pod {} revoked", pod_id);
                } else {
                    println!("✓ Pod {} revoked\n{}", pod_id, suffix);
                }
            }
        }
        Err(e) => {
            eprintln!("Revoke failed: {}", e);
            std::process::exit(1);
        }
    }
}

// ── Disconnect ───────────────────────────────────────────────
//
// FIX-2 (sprint SPRINT-TYTUS-PAYING-CUSTOMER-READY.md): `tytus disconnect`
// must reap daemons by pidfile, not by `state.pods[].tunnel_iface`, because
// `tytus revoke` wipes state while leaving the root-owned daemon running.
//
// Flow:
// 1. Enumerate candidates: either `[--pod NN]` (single-target) or every
//    `tunnel-*.pid` currently on disk under `/tmp/tytus`.
// 2. Also union in any pod IDs from `state.pods[]` — belt and braces, in
//    case a pidfile got nuked out from under us but state still thinks we
//    have a pod.
// 3. For each pod, call `tunnel_reap::reap_tunnel_for_pod(pod_num)` which
//    reads the pidfile, checks liveness, invokes scoped `sudo -n tytus
//    tunnel-down <pid>`, and cleans up on success.
// 4. Emit a per-pod message using the FIX-2 wording from the sprint doc.
// 5. Always clear local state (`tunnel_iface = None` / drop from vec) even
//    if reap failed — the user asked for disconnect, state must converge.

async fn cmd_disconnect(pod_id: Option<String>, json: bool) {
    let mut state = CliState::load();

    // 1. Build the candidate pod list. The pidfile directory is authoritative
    //    — it sees daemons that exist even when `state.pods[]` has been
    //    wiped by revoke. We also union in state.pods[].pod_id so we
    //    successfully clear stale state even when the pidfile is already gone.
    let mut candidates: Vec<String> = Vec::new();
    if let Some(ref filter) = pod_id {
        candidates.push(filter.clone());
    } else {
        for (pod_num, _path) in tunnel_reap::list_pod_pidfiles() {
            candidates.push(pod_num);
        }
        for pod in &state.pods {
            if !candidates.iter().any(|c| c == &pod.pod_id) {
                candidates.push(pod.pod_id.clone());
            }
        }
        // Orphan scan: any `tytus tunnel-up /tmp/tytus/tunnel-<pod>.json`
        // process whose pidfile has vanished AND whose pod isn't in state
        // (e.g. revoke wiped state while daemon kept running). Without
        // this, a bare `tytus disconnect` silently leaves the daemon +
        // tunnel iface behind forever.
        for pod_num in tunnel_reap::list_orphan_tunnel_pods() {
            if !candidates.iter().any(|c| c == &pod_num) {
                candidates.push(pod_num);
            }
        }
    }

    if candidates.is_empty() {
        if json {
            println!(r#"{{"status":"disconnected","tunnels_stopped":0,"pods":[]}}"#);
        } else {
            println!("→ No pidfiles and no state pods — nothing to disconnect");
        }
        return;
    }

    // Deduplicate while preserving order.
    {
        let mut seen = std::collections::HashSet::new();
        candidates.retain(|c| seen.insert(c.clone()));
    }

    let mut reaped_ok = 0u32;
    let mut reap_failed = 0u32;
    let mut json_entries: Vec<serde_json::Value> = Vec::new();

    for pod_num in &candidates {
        let outcome = tunnel_reap::reap_tunnel_for_pod(pod_num);
        let msg = tunnel_reap::disconnect_message(pod_num, &outcome);
        if !json {
            println!("{}", msg);
        }

        match &outcome {
            tunnel_reap::ReapOutcome::Reaped { .. } => reaped_ok += 1,
            tunnel_reap::ReapOutcome::ReapFailed { .. } => {
                reap_failed += 1;
                // Leave the user a recovery hint for the zero-tolerance case.
                if !json {
                    eprintln!(
                        "  hint: retry with `tytus disconnect --pod {}` or \
                         run `sudo kill $(cat /tmp/tytus/tunnel-{}.pid)`",
                        pod_num, pod_num
                    );
                }
            }
            _ => {}
        }

        if json {
            let (status, pid_val) = match &outcome {
                tunnel_reap::ReapOutcome::Reaped { pid } => ("reaped", Some(*pid)),
                tunnel_reap::ReapOutcome::NoPidfile => ("no_pidfile", None),
                tunnel_reap::ReapOutcome::StalePidfile { pid } => ("stale", Some(*pid)),
                tunnel_reap::ReapOutcome::ReapFailed { pid, .. } => ("failed", Some(*pid)),
            };
            json_entries.push(serde_json::json!({
                "pod_id": pod_num,
                "status": status,
                "pid": pid_val,
            }));
        }

        // 5. ALWAYS clear local state for this pod, regardless of reap
        //    outcome. Partial failure must still converge — the user
        //    asked to tear down. If the daemon is still alive after this,
        //    state.json lies briefly, but the next disconnect will see
        //    the pidfile and retry.
        if let Some(pod) = state.pods.iter_mut().find(|p| p.pod_id == *pod_num) {
            pod.tunnel_iface = None;
        }
        // A2: tear down any UI forwarder for this pod too. Without this,
        // a Disconnect leaves localhost:3000 bound to a dead upstream,
        // and the next browser click errors silently.
        stop_ui_forwarder(pod_num);
        // Intentionally do NOT wipe /tmp/tytus/ui-<pod>-cache here —
        // every disconnect+reconnect cycle would then force a 2–3 minute
        // full bundle re-download through the slow tunnel. Vite-built
        // apps use content-hashed filenames, so a different agent
        // version naturally produces different /assets/<hash>.js URLs
        // and misses cache cleanly. Cache eviction belongs to the
        // agent-lifecycle commands (install/uninstall/revoke), not the
        // tunnel-lifecycle ones. Discovered 2026-04-19 after user hit
        // 2+ minute loads on every reconnect.
    }
    state.save();

    if json {
        let payload = serde_json::json!({
            "status": "disconnected",
            "tunnels_stopped": reaped_ok,
            "failures": reap_failed,
            "pods": json_entries,
        });
        println!("{}", payload);
    } else {
        let summary = match (reaped_ok, reap_failed) {
            (0, 0) => "✓ Tunnel state cleared (no live daemons found)".to_string(),
            (n, 0) => format!("✓ {} tunnel(s) stopped", n),
            (n, f) => format!("⚠ {} stopped, {} failed — see messages above", n, f),
        };
        println!("{}", summary);
        if reap_failed > 0 {
            // Non-fatal exit code: state is cleared, but a daemon may
            // still be alive. The summary above told the user exactly
            // which pods to retry. We don't `exit(1)` here because the
            // user asked for convergence and we did converge state.
        }
    }
}

// ── Exec ────────────────────────────────────────────────────

async fn cmd_restart(http: &atomek_core::HttpClient, pod_id: Option<String>, json: bool) {
    let mut state = CliState::load();
    if !state.is_logged_in() {
        wizard::print_fail("Not logged in. Run: tytus login");
        std::process::exit(1);
    }
    if let Err(e) = ensure_token(&mut state, http).await {
        wizard::print_fail(&format!("Token refresh failed: {}. Run: tytus login", e));
        std::process::exit(1);
    }
    let (sk, auid) = get_credentials(&mut state, http).await;
    let client = atomek_pods::TytusClient::new(http, &sk, &auid);

    let target_pod_id = pod_id.unwrap_or_else(|| {
        state.pods.first().map(|p| p.pod_id.clone()).unwrap_or_else(|| {
            wizard::print_fail("No pods. Run: tytus connect");
            std::process::exit(1);
        })
    });

    if !json { wizard::print_info(&format!("Restarting agent on pod {}...", target_pod_id)); }
    let pb = wizard::spinner("Restarting container");

    match atomek_pods::restart_agent(&client, &target_pod_id).await {
        Ok(status) => {
            wizard::finish_ok(&pb, "Agent restarted");
            if json {
                println!("{}", serde_json::json!({
                    "pod_id": target_pod_id,
                    "agent_type": status.agent_type,
                    "container_status": status.container_status,
                    "healthy": status.healthy,
                }));
            } else {
                wizard::print_info(&format!("Container: {}", status.container_status.as_deref().unwrap_or("?")));
                if let Some(healthy) = status.healthy {
                    if healthy { wizard::print_ok("Agent is healthy"); }
                    else { wizard::print_warn("Agent not yet healthy (may still be starting)"); }
                }
                wizard::print_hint("Config file changes are now applied.");
            }
        }
        Err(e) => {
            wizard::finish_fail(&pb, &format!("Restart failed: {}", e));
            std::process::exit(1);
        }
    }
}

async fn cmd_exec(http: &atomek_core::HttpClient, command: Vec<String>, pod_id: Option<String>, timeout: u32, json: bool) {
    let mut state = CliState::load();

    if !state.is_logged_in() {
        eprintln!("Not logged in. Run: tytus login");
        std::process::exit(1);
    }

    if let Err(e) = ensure_token(&mut state, http).await {
        eprintln!("Token refresh failed: {}. Run: tytus login", e);
        std::process::exit(1);
    }
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

// ── Channels ────────────────────────────────────────────────
//
// Configure chat-channel credentials (Telegram/Discord/Slack/LINE)
// for the pod's agent. Writes to the pod's state volume at
// `/app/workspace/.tytus/channels.json`; DAM picks this up on the
// next `/agent/<pod>/deploy` and merges the values into the
// container's env vars. Secrets live in the OS keychain, never on
// disk in plaintext.
//
// See also:
//   - `cli/src/channels.rs` — registry of known channels + credentials
//   - `cli/src/channels_store.rs` — keychain + local manifest
//   - `services/wannolot-infrastructure/agent-manager/app.py` — DAM's
//     reader that merges channels.json into container env_vars
//   - `services/tytus-cli/dev/design/2026-04-20-unblock-openclaw-channels.md`

async fn cmd_channels(http: &atomek_core::HttpClient, action: ChannelsAction, json: bool) {
    match action {
        ChannelsAction::Catalog => {
            cmd_channels_catalog(json);
        }
        ChannelsAction::List { pod } => {
            cmd_channels_list(&pod, json);
        }
        ChannelsAction::Add {
            pod,
            r#type,
            token,
            app_token,
            user_token,
            channel_secret,
        } => {
            cmd_channels_add(
                http,
                &pod,
                &r#type,
                token,
                app_token,
                user_token,
                channel_secret,
                json,
            )
            .await;
        }
        ChannelsAction::Remove { pod, r#type } => {
            cmd_channels_remove(http, &pod, &r#type, json).await;
        }
    }
}

fn cmd_channels_catalog(json: bool) {
    if json {
        let entries: Vec<serde_json::Value> = channels::REGISTRY
            .iter()
            .map(|c| {
                serde_json::json!({
                    "name": c.name,
                    "label": c.label,
                    "blurb": c.blurb,
                    "agent_types": c.agent_types,
                    "inbound_model": c.inbound_model,
                    "credentials": c.credentials.iter().map(|cr| {
                        serde_json::json!({
                            "env_var": cr.env_var,
                            "label": cr.label,
                            "cli_flag": cr.cli_flag,
                        })
                    }).collect::<Vec<_>>(),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&entries).unwrap());
        return;
    }
    println!("Channels `tytus channels add` knows how to configure:\n");
    for c in channels::REGISTRY {
        println!("  {} — {}", console::style(c.name).cyan().bold(), c.label);
        println!("    Inbound: {}", c.inbound_model);
        println!("    {}", c.blurb);
        println!("    Credentials:");
        for cred in c.credentials {
            let flag = if cred.cli_flag == "token" {
                "--token".to_string()
            } else {
                format!("--{}", cred.cli_flag)
            };
            println!("      {} — {} (env: {})", flag, cred.label, cred.env_var);
        }
        println!("    Agents: {}", c.agent_types.join(", "));
        println!();
    }
}

fn cmd_channels_list(pod_id: &str, json: bool) {
    let manifest = channels_store::ChannelManifest::load();
    let channels = manifest.channels_for(pod_id);

    if json {
        let entries: Vec<serde_json::Value> = channels
            .iter()
            .map(|(name, entry)| {
                serde_json::json!({
                    "channel": name,
                    "env_vars": entry.env_vars,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "pod": pod_id,
                "channels": entries,
            }))
            .unwrap()
        );
        return;
    }

    if channels.is_empty() {
        println!("No channels configured for pod {}.", pod_id);
        println!("Run `tytus channels add --pod {} --type telegram --token ...` to add one.", pod_id);
        return;
    }
    println!("Channels configured for pod {}:", pod_id);
    for (name, entry) in channels {
        println!(
            "  {} — {} credential(s): {}",
            console::style(name).cyan().bold(),
            entry.env_vars.len(),
            entry.env_vars.join(", ")
        );
    }
}

#[allow(clippy::too_many_arguments)]
async fn cmd_channels_add(
    http: &atomek_core::HttpClient,
    pod_id: &str,
    channel_type: &str,
    token: Option<String>,
    app_token: Option<String>,
    user_token: Option<String>,
    channel_secret_flag: Option<String>,
    json: bool,
) {
    // 1. Look up the channel spec.
    let spec = match channels::find(channel_type) {
        Some(s) => s,
        None => {
            eprintln!(
                "Unknown channel type '{}'. Run `tytus channels catalog` to see supported channels.",
                channel_type
            );
            std::process::exit(1);
        }
    };

    // 2. Collect values for each credential the spec requires.
    // Map CLI flag → supplied value.
    let mut cli_values: std::collections::HashMap<&str, String> = std::collections::HashMap::new();
    if let Some(v) = token { cli_values.insert("token", v); }
    if let Some(v) = app_token { cli_values.insert("app-token", v); }
    if let Some(v) = user_token { cli_values.insert("user-token", v); }
    if let Some(v) = channel_secret_flag { cli_values.insert("channel-secret", v); }

    let mut collected: Vec<(String, String)> = Vec::new();
    for cred in spec.credentials {
        match cli_values.remove(cred.cli_flag) {
            Some(value) if !value.is_empty() => {
                collected.push((cred.env_var.to_string(), value));
            }
            _ => {
                let flag = if cred.cli_flag == "token" {
                    "--token".to_string()
                } else {
                    format!("--{}", cred.cli_flag)
                };
                eprintln!(
                    "Missing {} for channel '{}' — pass {} (credential: {}).",
                    cred.env_var, spec.name, flag, cred.label
                );
                std::process::exit(1);
            }
        }
    }

    // 3. Persist each secret to the OS keychain.
    let env_var_names: Vec<String> = collected.iter().map(|(k, _)| k.clone()).collect();
    for (env_var, value) in &collected {
        if let Err(e) = channels_store::store_secret(pod_id, spec.name, env_var, value) {
            eprintln!("Failed to store '{}' in keychain: {}", env_var, e);
            std::process::exit(1);
        }
    }

    // 4. Update the local manifest.
    let mut manifest = channels_store::ChannelManifest::load();
    manifest.add_channel(pod_id, spec.name, env_var_names.clone());
    if let Err(e) = manifest.save() {
        eprintln!("Failed to update local channel manifest: {}", e);
        std::process::exit(1);
    }

    // 5. Build the pod-side payload and push it to the pod.
    if let Err(e) = push_channels_to_pod(http, &manifest, pod_id).await {
        eprintln!("Failed to push channels.json to pod {}: {}", pod_id, e);
        eprintln!("Keychain + local manifest are updated; you can retry with `tytus channels list --pod {}` + rerun add.", pod_id);
        std::process::exit(1);
    }

    // 6. Redeploy the agent so it picks up the new env vars.
    if let Err(e) = redeploy_agent(http, pod_id).await {
        eprintln!(
            "channels.json pushed but agent redeploy failed: {}. Run `tytus agent catalog` + retry.",
            e
        );
        std::process::exit(1);
    }

    if json {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "pod": pod_id,
                "channel": spec.name,
                "env_vars": env_var_names,
            })
        );
    } else {
        println!(
            "{} {} configured on pod {} ({} credential{}). Agent restarted.",
            console::style("✓").green(),
            spec.label,
            pod_id,
            env_var_names.len(),
            if env_var_names.len() == 1 { "" } else { "s" },
        );
        println!("  {}", spec.inbound_model);
    }
}

async fn cmd_channels_remove(
    http: &atomek_core::HttpClient,
    pod_id: &str,
    channel_type: &str,
    json: bool,
) {
    let spec = match channels::find(channel_type) {
        Some(s) => s,
        None => {
            eprintln!("Unknown channel type '{}'.", channel_type);
            std::process::exit(1);
        }
    };

    let mut manifest = channels_store::ChannelManifest::load();
    let removed = manifest.remove_channel(pod_id, spec.name);

    if removed.is_none() {
        eprintln!(
            "No {} channel configured for pod {}. Nothing to remove.",
            spec.label, pod_id
        );
        std::process::exit(0);
    }

    // Delete keychain entries.
    if let Some(ref entry) = removed {
        for env_var in &entry.env_vars {
            let _ = channels_store::delete_secret(pod_id, spec.name, env_var);
        }
    }

    if let Err(e) = manifest.save() {
        eprintln!("Failed to update local manifest: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = push_channels_to_pod(http, &manifest, pod_id).await {
        eprintln!("Keychain + manifest cleared but pod sync failed: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = redeploy_agent(http, pod_id).await {
        eprintln!("Pod sync OK but agent redeploy failed: {}", e);
        std::process::exit(1);
    }

    if json {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "pod": pod_id,
                "channel": spec.name,
                "removed": true,
            })
        );
    } else {
        println!(
            "{} {} removed from pod {}. Agent restarted.",
            console::style("✓").green(),
            spec.label,
            pod_id
        );
    }
}

/// Load state for channel operations without triggering a keychain
/// lookup. Channel management only needs a valid access token to call
/// Provider; the refresh token is the daemon's job. Keeping these
/// flows keychain-free means:
///   - No 3-second timeout on machines with a pending ACL dialog
///   - No `WARN keychain get_refresh_token timed out` log noise
///     bleeding into the user's terminal while they're being
///     prompted for a bot token
/// If the access token has expired, we fall back to `load()` (which
/// DOES touch keychain) so the normal refresh path still works.
fn load_for_channel_op() -> CliState {
    let file_state = CliState::load_file_only();
    if file_state.has_valid_token() {
        file_state
    } else {
        CliState::load()
    }
}

/// Build the channels.json payload from the local manifest + keychain
/// and write it to `/app/workspace/.tytus/channels.json` on the pod.
/// Uses `tytus exec` (which shells to DAM's exec endpoint under the
/// hood) to keep the write path identical to what users do manually.
async fn push_channels_to_pod(
    http: &atomek_core::HttpClient,
    manifest: &channels_store::ChannelManifest,
    pod_id: &str,
) -> Result<(), String> {
    let payload = channels_store::render_pod_payload(manifest, pod_id)
        .map_err(|e| format!("rendering pod payload: {}", e))?;
    let payload_str = serde_json::to_string(&payload)
        .map_err(|e| format!("serializing pod payload: {}", e))?;

    // Base64-encode the payload to survive shell quoting. base64 is
    // available on every Linux pod image we ship (busybox or coreutils).
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(payload_str.as_bytes());
    let script = format!(
        "mkdir -p /app/workspace/.tytus && \
         echo '{}' | base64 -d > /app/workspace/.tytus/channels.json.tmp && \
         chmod 600 /app/workspace/.tytus/channels.json.tmp && \
         mv /app/workspace/.tytus/channels.json.tmp /app/workspace/.tytus/channels.json && \
         echo OK",
        b64
    );

    let mut state = load_for_channel_op();
    if state.email.as_deref().unwrap_or("").is_empty() {
        return Err("not logged in".to_string());
    }
    ensure_token(&mut state, http).await.map_err(|e| format!("token refresh: {}", e))?;
    let (sk, auid) = get_credentials(&mut state, http).await;
    let client = atomek_pods::TytusClient::new(http, &sk, &auid);

    let result = atomek_pods::exec_in_agent(&client, pod_id, &script, 15)
        .await
        .map_err(|e| format!("exec_in_agent: {}", e))?;

    if result.exit_code != 0 {
        let stderr = result.stderr.unwrap_or_default();
        return Err(format!(
            "pod write exited {}: {}",
            result.exit_code,
            stderr.trim()
        ));
    }
    Ok(())
}

/// Redeploy the agent so the channel credentials land in the container env.
/// Uses the existing Provider `/pod/agent/deploy` endpoint which calls
/// DAM's deploy (stops existing container + re-reads channels.json + starts
/// new container).
async fn redeploy_agent(
    http: &atomek_core::HttpClient,
    pod_id: &str,
) -> Result<(), String> {
    let mut state = load_for_channel_op();
    ensure_token(&mut state, http).await.map_err(|e| format!("token refresh: {}", e))?;
    let (sk, auid) = get_credentials(&mut state, http).await;
    let client = atomek_pods::TytusClient::new(http, &sk, &auid);

    // Look up the current agent_type for this pod from state.
    let agent_type = state
        .pods
        .iter()
        .find(|p| p.pod_id == pod_id)
        .and_then(|p| p.agent_type.clone())
        .unwrap_or_else(|| "nemoclaw".to_string());

    atomek_pods::deploy_agent(&client, pod_id, &agent_type)
        .await
        .map_err(|e| format!("deploy_agent: {}", e))?;
    Ok(())
}

// ── Logout ───────────────────────────────────────────────────

async fn cmd_logout(http: &atomek_core::HttpClient, json: bool) {
    let mut state = CliState::load();

    // A3: kill every live UI forwarder before we revoke pods. Otherwise
    // logout leaves orphan forwarders bound to 127.0.0.1 ports pointing
    // at endpoints that no longer exist, and subsequent fresh logins get
    // "stale marker" warnings.
    for pod in list_ui_forwarder_pods() {
        stop_ui_forwarder(&pod);
    }

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

async fn cmd_env(pod_id: Option<String>, export: bool, raw: bool, tunnel: bool, json: bool, http: &atomek_core::HttpClient) {
    let mut state = CliState::load();

    let pod_idx = if let Some(ref pid) = pod_id {
        state.pods.iter().position(|p| p.pod_id == *pid)
    } else {
        // First connected pod, or first pod
        state.pods.iter().position(|p| p.tunnel_iface.is_some())
            .or(if state.pods.is_empty() { None } else { Some(0) })
    };

    let Some(idx) = pod_idx else {
        if json { println!(r#"{{"error":"no_pods"}}"#); }
        else { eprintln!("No pods. Run: tytus connect"); }
        std::process::exit(1);
    };

    // Refresh stable key + edge fields from the Provider if we're missing
    // either. The same endpoint also returns slug + public_url when
    // EDGE_PATH_ENABLED=1, so one call covers both paths.
    if !raw {
        let needs_refresh = state.pods[idx].stable_user_key.is_none()
            || state.pods[idx].edge_public_url.is_none()
            || state.pods[idx].pod_public_url.is_none();
        if needs_refresh {
            if let (Some(st), Some(aid)) = (state.secret_key.as_ref(), state.agent_user_id.as_ref()) {
                let client = atomek_pods::TytusClient::new(http, st, aid);
                if let Ok(uk) = atomek_pods::get_user_key_full(&client).await {
                    let pod_id_for_template = state.pods[idx].pod_id.clone();
                    let composed = uk.compose_pod_public_url(&pod_id_for_template);
                    if let Some(p) = state.pods.get_mut(idx) {
                        p.stable_ai_endpoint = Some(uk.endpoint);
                        p.stable_user_key = Some(uk.key);
                        if let Some(s) = uk.slug { p.edge_slug = Some(s); }
                        if let Some(u) = uk.public_url { p.edge_public_url = Some(u); }
                        if let Some(u) = composed { p.pod_public_url = Some(u); }
                    }
                    state.save();
                }
            }
        }
    }

    let pod = &state.pods[idx];

    if json {
        println!("{}", serde_json::to_string_pretty(pod).unwrap_or_default());
        return;
    }

    let prefix = if export { "export " } else { "" };

    if raw {
        // Unstable per-pod values — changes on pod rotation.
        if let Some(ref ep) = pod.ai_endpoint {
            println!("{}AIL_URL={}/v1", prefix, ep);
            println!("{}OPENAI_BASE_URL={}/v1", prefix, ep);
            println!("{}ANTHROPIC_BASE_URL={}", prefix, ep);
            println!("{}TYTUS_AI_GATEWAY={}", prefix, ep);
        }
        if let Some(ref ep) = pod.agent_endpoint {
            println!("{}TYTUS_AGENT_API={}", prefix, ep);
        }
        if let Some(ref key) = pod.pod_api_key {
            println!("{}AIL_API_KEY={}", prefix, key);
            println!("{}OPENAI_API_KEY={}", prefix, key);
            println!("{}ANTHROPIC_API_KEY={}", prefix, key);
            println!("{}TYTUS_API_KEY={}", prefix, key);
        }
    } else {
        // Stable values — the pair to paste into Cursor / Claude Desktop /
        // etc. Canonical name is AIL_*; OPENAI_* / ANTHROPIC_* / TYTUS_*
        // kept as aliases so OpenAI-compatible clients, the Anthropic
        // SDK, and legacy scripts all keep working without user config.
        // Note: ANTHROPIC_BASE_URL is the bare origin (no /v1) because
        // the Anthropic SDK appends /v1/messages itself.
        //
        // Endpoint selection:
        //   --tunnel → legacy WG URL (10.42.42.1:18080) — requires active tunnel
        //   else     → edge_public_url + /p/<pod_id> if present, else WG
        //
        // The public edge URL does NOT need the tunnel. One user, one URL,
        // works from any network that can reach the public internet.
        let wg_endpoint = pod.stable_ai_endpoint.as_deref()
            .unwrap_or("http://10.42.42.1:18080")
            .to_string();
        let endpoint = if tunnel {
            wg_endpoint.clone()
        } else if let Some(ref pod_url) = pod.pod_public_url {
            // Per-pod subdomain (sprint 2026-04-23) — preferred.
            pod_url.trim_end_matches('/').to_string()
        } else if let Some(ref public) = pod.edge_public_url {
            // Legacy `{slug}.{base}/p/{NN}` composition — kept as
            // back-compat for state.json entries written before the sprint.
            format!("{}/p/{}", public.trim_end_matches('/'), pod.pod_id)
        } else {
            wg_endpoint.clone()
        };
        let key = pod.stable_user_key.as_deref()
            .or(pod.pod_api_key.as_deref())
            .unwrap_or("");
        println!("{}AIL_URL={}/v1", prefix, endpoint);
        println!("{}AIL_API_KEY={}", prefix, key);
        println!("{}OPENAI_BASE_URL={}/v1", prefix, endpoint);
        println!("{}OPENAI_API_KEY={}", prefix, key);
        println!("{}ANTHROPIC_BASE_URL={}", prefix, endpoint);
        println!("{}ANTHROPIC_API_KEY={}", prefix, key);
        println!("{}TYTUS_AI_GATEWAY={}", prefix, endpoint);
        println!("{}TYTUS_API_KEY={}", prefix, key);
    }

    if let Some(ref at) = pod.agent_type {
        println!("{}TYTUS_AGENT_TYPE={}", prefix, at);
    }
    println!("{}TYTUS_POD_ID={}", prefix, pod.pod_id);
}

// ── Capabilities (discover /v1/models) ──────────────────────

/// Render the pod's AI gateway model catalog + per-model provider-native
/// tools. Human tree by default; raw JSON passthrough when `json` is set.
/// Resolves the endpoint/key pair the same way `tytus env` does, so
/// everything works through the WG tunnel or the public-edge path.
async fn cmd_capabilities(http: &atomek_core::HttpClient, pod_id: Option<String>, json: bool) {
    let mut state = CliState::load();

    let pod_idx = if let Some(ref pid) = pod_id {
        state.pods.iter().position(|p| p.pod_id == *pid)
    } else {
        state.pods.iter().position(|p| p.tunnel_iface.is_some())
            .or(if state.pods.is_empty() { None } else { Some(0) })
    };
    let Some(idx) = pod_idx else {
        if json { println!(r#"{{"error":"no_pods"}}"#); }
        else { eprintln!("No pods. Run: tytus connect"); }
        std::process::exit(1);
    };

    // Lazily hydrate stable_user_key + edge_public_url from Provider if
    // we've never cached them — same conditional refresh cmd_env uses so
    // the two commands agree on the endpoint/key pair every time.
    let needs_refresh = state.pods[idx].stable_user_key.is_none()
        || state.pods[idx].edge_public_url.is_none()
        || state.pods[idx].pod_public_url.is_none();
    if needs_refresh {
        if let (Some(st), Some(aid)) = (state.secret_key.as_ref(), state.agent_user_id.as_ref()) {
            let client = atomek_pods::TytusClient::new(http, st, aid);
            if let Ok(uk) = atomek_pods::get_user_key_full(&client).await {
                let pod_id_for_template = state.pods[idx].pod_id.clone();
                let composed = uk.compose_pod_public_url(&pod_id_for_template);
                if let Some(p) = state.pods.get_mut(idx) {
                    p.stable_ai_endpoint = Some(uk.endpoint);
                    p.stable_user_key = Some(uk.key);
                    if let Some(s) = uk.slug { p.edge_slug = Some(s); }
                    if let Some(u) = uk.public_url { p.edge_public_url = Some(u); }
                    if let Some(u) = composed { p.pod_public_url = Some(u); }
                }
                state.save();
            }
        }
    }

    let pod = &state.pods[idx];

    // Prefer the public-edge URL when available — matches `tytus env`'s
    // default. Falls back to the WG tunnel endpoint otherwise. The edge
    // URL works from any network, no tunnel required, so `tytus
    // capabilities` is usable in CI/sandbox contexts where the user
    // hasn't brought the tunnel up.
    let wg_endpoint = pod.stable_ai_endpoint.as_deref()
        .unwrap_or("http://10.42.42.1:18080")
        .to_string();
    let endpoint = if let Some(ref pod_url) = pod.pod_public_url {
        pod_url.trim_end_matches('/').to_string()
    } else if let Some(ref public) = pod.edge_public_url {
        format!("{}/p/{}", public.trim_end_matches('/'), pod.pod_id)
    } else {
        wg_endpoint
    };
    let key = pod.stable_user_key.as_deref()
        .or(pod.pod_api_key.as_deref())
        .unwrap_or("");

    if key.is_empty() {
        if json { println!(r#"{{"error":"no_api_key"}}"#); }
        else { eprintln!("No API key cached for pod {}. Run: tytus env --pod {}", pod.pod_id, pod.pod_id); }
        std::process::exit(1);
    }

    let url = format!("{}/v1/models", endpoint);
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            if json { println!(r#"{{"error":"http_client_build: {}"}}"#, e); }
            else { eprintln!("Failed to build HTTP client: {}", e); }
            std::process::exit(1);
        }
    };

    let resp = match client.get(&url)
        .header("Authorization", format!("Bearer {}", key))
        .send().await
    {
        Ok(r) => r,
        Err(e) => {
            if json { println!(r#"{{"error":"network: {}"}}"#, e); }
            else { eprintln!("Network error reaching {}: {}", url, e); }
            std::process::exit(1);
        }
    };

    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        if json {
            println!(r#"{{"error":"http_{}","body":{}}}"#, status.as_u16(), serde_json::to_string(&body_text).unwrap_or_else(|_| "\"\"".into()));
        } else {
            eprintln!("Gateway returned HTTP {} from {}:", status, url);
            eprintln!("{}", body_text);
        }
        std::process::exit(1);
    }

    // --json: passthrough the raw /v1/models body, verbatim. Keeps the
    // shape byte-identical with what a direct curl would see, so AI
    // agents don't have to guess at a wrapper schema.
    if json {
        print!("{}", body_text);
        return;
    }

    // Human tree view. Parse as loose serde_json::Value so unknown
    // fields (from future switchailocal versions) don't break rendering.
    let v: serde_json::Value = match serde_json::from_str(&body_text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Gateway returned unparseable JSON: {}", e);
            eprintln!("{}", body_text);
            std::process::exit(1);
        }
    };

    let tier = state.tier.as_deref().unwrap_or("unknown");
    let agent = pod.agent_type.as_deref().unwrap_or("none");
    print!("{}", render_capabilities_tree(&pod.pod_id, agent, tier, &v));
}

/// Pure formatter for `tytus capabilities` tree output. Takes the gateway's
/// parsed `/v1/models` body and emits a newline-terminated human rendering.
/// Extracted from `cmd_capabilities` so unit tests can pin the output
/// shape without standing up a live gateway. Tolerates missing / partial
/// fields so future switchailocal shape additions don't crash the CLI.
fn render_capabilities_tree(pod_id: &str, agent: &str, tier: &str, v: &serde_json::Value) -> String {
    let mut out = String::new();
    out.push_str(&format!("Pod {} ({}, {} plan)\n", pod_id, agent, tier));

    let empty = Vec::new();
    let data = v.get("data").and_then(|d| d.as_array()).unwrap_or(&empty);
    if data.is_empty() {
        out.push_str("  (gateway returned an empty model list)\n");
        return out;
    }

    // Compute the width of the longest ID so the second column aligns
    // cleanly without pulling in a table-printer dependency.
    let id_width = data.iter()
        .filter_map(|m| m.get("id").and_then(|i| i.as_str()))
        .map(|s| s.chars().count())
        .max()
        .unwrap_or(12);

    for model in data {
        let id = model.get("id").and_then(|i| i.as_str()).unwrap_or("?");
        let owned = model.get("owned_by").and_then(|o| o.as_str()).unwrap_or("");
        let display = model.get("display_name").and_then(|d| d.as_str()).unwrap_or("");
        let ctx = model.get("context_length").and_then(|c| c.as_i64());

        // Right side: "<owner> <display>" plus context-window tag when
        // the gateway advertises one. Keeps each row scannable at a glance.
        let mut right = String::new();
        if !display.is_empty() && display != id {
            right.push_str(display);
        } else if !owned.is_empty() {
            right.push_str(owned);
        }
        if let Some(n) = ctx {
            if n > 0 {
                if !right.is_empty() { right.push_str(", "); }
                if n >= 1000 { right.push_str(&format!("{}k ctx", n / 1000)); }
                else { right.push_str(&format!("{} ctx", n)); }
            }
        }

        out.push_str(&format!("  {:<width$}  {}\n", id, right, width = id_width));

        // Native tools subtree. Only render when non-empty — matches the
        // gateway's key-presence convention (empty / missing means the
        // model has no provider-native tools to splice into tools[]).
        if let Some(nt) = model.get("native_tools").and_then(|n| n.as_array()) {
            for tool in nt {
                let ttype = tool.get("type").and_then(|t| t.as_str()).unwrap_or("?");
                let tdesc = tool.get("description").and_then(|d| d.as_str()).unwrap_or("");
                if tdesc.is_empty() {
                    out.push_str(&format!("  {:<width$}    ↳ native: {}\n", "", ttype, width = id_width));
                } else {
                    // Truncate long descriptions to keep the tree compact.
                    // Agents who need the full text should use `--json`.
                    let short = if tdesc.chars().count() > 80 {
                        let t: String = tdesc.chars().take(77).collect();
                        format!("{}…", t)
                    } else {
                        tdesc.to_string()
                    };
                    out.push_str(&format!("  {:<width$}    ↳ native: {} — {}\n", "", ttype, short, width = id_width));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod capabilities_tests {
    use super::render_capabilities_tree;
    use serde_json::json;

    /// Snapshot the tree rendering for a realistic /v1/models body so a
    /// future shape regression (e.g. native_tools disappearing or a new
    /// top-level field silently shadowing the ID column) trips this test.
    #[test]
    fn renders_tree_with_native_tools() {
        let body = json!({
            "object": "list",
            "data": [
                {
                    "id": "ail-compound",
                    "object": "model",
                    "owned_by": "minimax",
                    "context_length": 200000,
                    "native_tools": [
                        {
                            "type": "web_search",
                            "description": "MiniMax native web search"
                        }
                    ]
                },
                {
                    "id": "ail-image",
                    "object": "model",
                    "owned_by": "minimax"
                }
            ]
        });
        let out = render_capabilities_tree("02", "nemoclaw", "operator", &body);
        assert!(out.contains("Pod 02 (nemoclaw, operator plan)"), "header missing:\n{}", out);
        assert!(out.contains("ail-compound"), "ail-compound row missing:\n{}", out);
        assert!(out.contains("200k ctx"), "context-window tag missing:\n{}", out);
        assert!(out.contains("↳ native: web_search"), "native_tools subtree missing:\n{}", out);
        assert!(out.contains("MiniMax native web search"), "description missing:\n{}", out);
        // Model without native_tools should NOT emit a native-tools subtree.
        assert!(!out.lines().any(|l| l.contains("ail-image") && l.contains("↳ native")),
                "ail-image should not have native subtree:\n{}", out);
    }

    /// Pin the empty-data behavior: a gateway that returns `{data: []}`
    /// gets a clear "no models" message rather than a blank render.
    #[test]
    fn renders_empty_catalog_friendly() {
        let body = json!({"object": "list", "data": []});
        let out = render_capabilities_tree("02", "nemoclaw", "operator", &body);
        assert!(out.contains("empty model list"), "empty-catalog message missing:\n{}", out);
    }

    /// Pin that long descriptions truncate so the tree stays scannable.
    /// Callers who need the full text should use `--json`.
    #[test]
    fn truncates_long_descriptions() {
        let long = "a".repeat(200);
        let body = json!({
            "data": [{
                "id": "x",
                "owned_by": "y",
                "native_tools": [{"type": "t", "description": long.clone()}]
            }]
        });
        let out = render_capabilities_tree("02", "nemoclaw", "operator", &body);
        assert!(out.contains("…"), "truncation ellipsis missing:\n{}", out);
        assert!(!out.contains(&long), "full long description should not appear:\n{}", out);
    }
}

// ── Infect (drop integration files) ─────────────────────────

fn cmd_link(dir: &str, only: Option<Vec<String>>, json: bool) {
    let base = std::path::Path::new(dir).canonicalize().unwrap_or_else(|_| {
        eprintln!("Directory not found: {}", dir);
        std::process::exit(1);
    });

    let tytus_bin = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("tytus-mcp").display().to_string()))
        .unwrap_or_else(|| "tytus-mcp".into());

    let should_inject = |name: &str| -> bool {
        only.as_ref().is_none_or(|list| list.iter().any(|s| s == name))
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
                        "tytus_docs",
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
            "status": "linked",
            "directory": base.display().to_string(),
            "files": injected,
        }));
    } else {
        println!("Tytus linked into {}", base.display());
        for file in &injected {
            println!("  + {}", file);
        }
        println!("\nAI CLIs in this directory now natively know how to drive Tytus.");
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
                            "tytus_docs",
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
    if let Err(e) = ensure_token(&mut state, http).await {
        wizard::print_fail(&format!("Token refresh failed: {}. Run: tytus login", e));
        return;
    }
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
        if ensure_token(&mut state, http).await.is_err() {
            wizard::print_fail("Session expired — let's sign in again.");
            state.clear();
        } else {
            wizard::print_ok(&format!("Already signed in as {}", state.email.as_deref().unwrap_or("?")));
        }
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
    wizard::print_info("OpenClaw — lightweight assistant (1 unit, good for most tasks)");
    wizard::print_info("Hermes   — advanced reasoning agent (2 units, better for complex tasks)");
    println!();

    let agent = if state.pods.is_empty() {
        match wizard::select("Which agent?", &["OpenClaw (recommended)", "Hermes"]) {
            Ok(s) if s.to_ascii_lowercase().starts_with("hermes") => "hermes",
            _ => "nemoclaw", // backend identifier; public brand is OpenClaw
        }
    } else {
        let first_agent = state.pods[0].agent_type.clone().unwrap_or_else(|| "nemoclaw".to_string());
        let display = match first_agent.as_str() {
            "nemoclaw" => "OpenClaw",
            "hermes" => "Hermes",
            "none" => "Default (AIL only)",
            other => other,
        };
        wizard::print_ok(&format!("Using existing pod ({})", display));
        // Leak is fine here — agent is used as &str for a single call
        Box::leak(first_agent.into_boxed_str())
    };
    println!();

    // ── Step 3: Allocate pod + activate tunnel ──
    wizard::print_step(3, total_steps, "Allocating your pod and starting tunnel");
    println!();
    // If no pods yet, install the picked agent so the first-run flow still
    // ends up with a real agent runtime (not just AIL). cmd_agent_install
    // allocates a pod AND deploys the agent in one shot via /pod/request;
    // cmd_connect then brings the tunnel up to whatever was allocated.
    if state.pods.is_empty() {
        let _ = cmd_agent_install(http, agent, None, false, false).await;
    }
    cmd_connect(http, None, false).await;
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
    wizard::print_hint("tytus link .         — Link Tytus into this project (AI CLI integration)");
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

    if let Err(e) = ensure_token(&mut state, http).await {
        if json { println!(r#"{{"ok":false,"error":"token_refresh_failed: {}"}}"#, e); }
        else { wizard::print_fail(&format!("Token refresh failed: {}. Run: tytus login", e)); }
        std::process::exit(1);
    }
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
    if let Err(e) = ensure_token(&mut state, http).await {
        wizard::print_fail(&format!("Token refresh failed: {}. Run: tytus login", e));
        std::process::exit(1);
    }
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
    if let Err(e) = ensure_token(&mut state, http).await {
        wizard::print_fail(&format!("Token refresh failed: {}. Run: tytus login", e));
        std::process::exit(1);
    }
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
                    wizard::print_info(out.trim());
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

// ── Autostart (macOS LaunchAgent + Linux systemd --user) ────

/// FIX-6: auto-start on boot.
///
/// After a reboot, the tunnel daemon is gone — but the user's apps (Cursor,
/// Claude Desktop, Ollama-compatible scripts) are all configured with the
/// stable pair `http://10.42.42.1:18080/v1` + `sk-tytus-user-*`. Without
/// auto-start, the user has to manually `tytus connect` every boot. With
/// auto-start, the LaunchAgent/systemd unit runs `tytus connect` at login
/// and the same URLs/keys just work.
///
/// Thin pass-through to `python3 -m tytus_sdk`. The SDK handles the OpenClaw
/// WebSocket + Ed25519 handshake, session lifecycle, and reply collection.
/// See docs/DESIGN-TYTUS-LOPE-TEAMMATES.md for architecture and
/// tytus_sdk/adapters/openclaw.py for wire details.
///
/// `kind` is "lope" (for `tytus lope ask/install/...`) or "bridge" (for
/// `tytus bridge run/...`). For "lope" we shell through to the SDK's top-
/// level subcommands directly (the SDK exposes ask/install/identity at
/// top level). For "bridge" we prepend "bridge" so `tytus bridge run` →
/// `python3 -m tytus_sdk bridge run`.
async fn cmd_lope_passthrough(kind: &str, args: Vec<String>, _json: bool) {
    // Locate the SDK. In a dev checkout it lives next to the cli/ crate at
    // the workspace root. In a distributed binary, it would be under
    // ~/.tytus/sdk/ or installed site-packages. For v0.5.0-alpha we only
    // support the dev-checkout path.
    let exe = std::env::current_exe().ok();
    let sdk_root = exe
        .as_ref()
        .and_then(|p| p.parent())   // target/release
        .and_then(|p| p.parent())   // target
        .and_then(|p| p.parent())   // workspace root
        .map(|p| p.to_path_buf())
        .or_else(|| {
            // Fallback: look in the known dev path.
            let cwd = std::env::current_dir().ok()?;
            Some(cwd)
        })
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    let sdk_dir = sdk_root.join("tytus_sdk");
    let pythonpath = if sdk_dir.exists() {
        sdk_root.as_os_str().to_os_string()
    } else {
        // Installed path — trust PYTHONPATH / site-packages.
        std::env::var_os("PYTHONPATH").unwrap_or_default()
    };

    let mut cmd = std::process::Command::new("python3");
    cmd.arg("-m").arg("tytus_sdk");
    if kind == "bridge" {
        cmd.arg("bridge");
    }
    cmd.args(&args).env("PYTHONPATH", &pythonpath);
    let status = cmd.status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => std::process::exit(s.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("Failed to invoke python3 -m tytus_sdk: {}", e);
            eprintln!("Ensure Python 3.9+ is installed with: pip install websockets cryptography httpx");
            std::process::exit(1);
        }
    }
}

/// macOS: ~/Library/LaunchAgents/com.traylinx.tytus.plist + launchctl load
/// Linux: ~/.config/systemd/user/tytus.service + systemctl --user enable --now
fn cmd_autostart(action: AutostartAction, json: bool) {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/unknown".to_string());
        let plist_dir = std::path::PathBuf::from(&home).join("Library/LaunchAgents");
        // Two agents: one oneshot that brings the tunnel up at login, one
        // persistent daemon that keeps refreshing tokens 24/7 so the RT never
        // expires server-side. Both are managed atomically by this subcommand.
        let plist_path = plist_dir.join("com.traylinx.tytus.plist");
        let daemon_plist_path = plist_dir.join("com.traylinx.tytus.daemon.plist");
        let exe = std::env::current_exe()
            .ok()
            .and_then(|p| p.to_str().map(String::from))
            .unwrap_or_else(|| "/Users/sebastian/bin/tytus".to_string());

        match action {
            AutostartAction::Install => {
                if let Err(e) = std::fs::create_dir_all(&plist_dir) {
                    eprintln!("Failed to create LaunchAgents dir: {}", e);
                    std::process::exit(1);
                }
                // Oneshot: run `tytus connect` once at login to activate tunnel.
                let plist = format!(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.traylinx.tytus</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
        <string>connect</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
    <key>StandardOutPath</key>
    <string>/tmp/tytus/autostart.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/tytus/autostart.log</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>HOME</key>
        <string>{home}</string>
        <key>PATH</key>
        <string>/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin</string>
        <key>TYTUS_HEADLESS</key>
        <string>1</string>
    </dict>
</dict>
</plist>
"#
                );
                if let Err(e) = std::fs::write(&plist_path, plist) {
                    eprintln!("Failed to write plist: {}", e);
                    std::process::exit(1);
                }
                // Persistent daemon: `tytus daemon run` keeps refreshing tokens
                // forever. KeepAlive restarts it if it ever exits; ThrottleInterval
                // prevents tight respawn loops if something is genuinely broken.
                let daemon_plist = format!(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.traylinx.tytus.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
        <string>daemon</string>
        <string>run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
        <key>Crashed</key>
        <true/>
    </dict>
    <key>ThrottleInterval</key>
    <integer>30</integer>
    <key>ProcessType</key>
    <string>Background</string>
    <key>StandardOutPath</key>
    <string>/tmp/tytus/daemon.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/tytus/daemon.log</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>HOME</key>
        <string>{home}</string>
        <key>PATH</key>
        <string>/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin</string>
        <key>TYTUS_HEADLESS</key>
        <string>1</string>
    </dict>
</dict>
</plist>
"#
                );
                if let Err(e) = std::fs::write(&daemon_plist_path, daemon_plist) {
                    eprintln!("Failed to write daemon plist: {}", e);
                    std::process::exit(1);
                }
                // (Re)load both agents.
                for p in [&plist_path, &daemon_plist_path] {
                    let _ = std::process::Command::new("launchctl")
                        .args(["unload", p.to_str().unwrap_or_default()])
                        .output();
                }
                let load_connect = std::process::Command::new("launchctl")
                    .args(["load", "-w", plist_path.to_str().unwrap_or_default()])
                    .output();
                let load_daemon = std::process::Command::new("launchctl")
                    .args(["load", "-w", daemon_plist_path.to_str().unwrap_or_default()])
                    .output();
                let ok_connect = load_connect.map(|o| o.status.success()).unwrap_or(false);
                let ok_daemon = load_daemon.map(|o| o.status.success()).unwrap_or(false);
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "action": "install",
                            "plist_path": plist_path.to_string_lossy(),
                            "daemon_plist_path": daemon_plist_path.to_string_lossy(),
                            "loaded": ok_connect,
                            "daemon_loaded": ok_daemon
                        })
                    );
                } else {
                    println!("✓ LaunchAgent installed at {}", plist_path.display());
                    println!("✓ Token-refresh daemon installed at {}", daemon_plist_path.display());
                    println!("  Auto-start on every login: enabled");
                    println!("  Background token refresh: enabled (KeepAlive)");
                    println!("  Your stable endpoint http://10.42.42.1:18080 + sk-tytus-user-* will");
                    println!("  keep working across reboots — no more expired-token prompts.");
                }
            }
            AutostartAction::Uninstall => {
                for p in [&plist_path, &daemon_plist_path] {
                    let _ = std::process::Command::new("launchctl")
                        .args(["unload", "-w", p.to_str().unwrap_or_default()])
                        .output();
                    let _ = std::fs::remove_file(p);
                }
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "action": "uninstall",
                            "plist_path": plist_path.to_string_lossy(),
                            "daemon_plist_path": daemon_plist_path.to_string_lossy()
                        })
                    );
                } else {
                    println!("✓ LaunchAgents removed. Auto-start and daemon disabled.");
                }
            }
            AutostartAction::Status => {
                let installed = plist_path.exists();
                let daemon_installed = daemon_plist_path.exists();
                let loaded = std::process::Command::new("launchctl")
                    .args(["list", "com.traylinx.tytus"])
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);
                let daemon_loaded = std::process::Command::new("launchctl")
                    .args(["list", "com.traylinx.tytus.daemon"])
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "action": "status",
                            "installed": installed,
                            "loaded": loaded,
                            "daemon_installed": daemon_installed,
                            "daemon_loaded": daemon_loaded,
                            "plist_path": plist_path.to_string_lossy(),
                            "daemon_plist_path": daemon_plist_path.to_string_lossy()
                        })
                    );
                } else {
                    println!("Auto-start status:");
                    println!("  plist:          {} {}", plist_path.display(), if installed { "[installed]" } else { "[missing]" });
                    println!("  loaded:         {}", if loaded { "yes" } else { "no" });
                    println!("  daemon plist:   {} {}", daemon_plist_path.display(), if daemon_installed { "[installed]" } else { "[missing]" });
                    println!("  daemon loaded:  {}", if daemon_loaded { "yes" } else { "no" });
                    if !installed || !daemon_installed {
                        println!();
                        println!("To enable auto-start + background refresh: tytus autostart install");
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/unknown".to_string());
        let unit_dir = std::path::PathBuf::from(&home).join(".config/systemd/user");
        let unit_path = unit_dir.join("tytus.service");
        let daemon_unit_path = unit_dir.join("tytus-daemon.service");
        let exe = std::env::current_exe()
            .ok()
            .and_then(|p| p.to_str().map(String::from))
            .unwrap_or_else(|| "/usr/local/bin/tytus".to_string());

        match action {
            AutostartAction::Install => {
                if let Err(e) = std::fs::create_dir_all(&unit_dir) {
                    eprintln!("Failed to create user systemd dir: {}", e);
                    std::process::exit(1);
                }
                let unit = format!(
                    "[Unit]\nDescription=Tytus private AI pod tunnel (auto-start on login)\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nType=oneshot\nExecStart={exe} connect\nRemainAfterExit=yes\nEnvironment=TYTUS_HEADLESS=1\nStandardOutput=append:/tmp/tytus/autostart.log\nStandardError=append:/tmp/tytus/autostart.log\n\n[Install]\nWantedBy=default.target\n"
                );
                if let Err(e) = std::fs::write(&unit_path, unit) {
                    eprintln!("Failed to write unit: {}", e);
                    std::process::exit(1);
                }
                // Persistent token-refresh daemon — restart forever on crash.
                let daemon_unit = format!(
                    "[Unit]\nDescription=Tytus token-refresh daemon (background)\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nType=simple\nExecStart={exe} daemon run\nRestart=always\nRestartSec=30\nEnvironment=TYTUS_HEADLESS=1\nStandardOutput=append:/tmp/tytus/daemon.log\nStandardError=append:/tmp/tytus/daemon.log\n\n[Install]\nWantedBy=default.target\n"
                );
                if let Err(e) = std::fs::write(&daemon_unit_path, daemon_unit) {
                    eprintln!("Failed to write daemon unit: {}", e);
                    std::process::exit(1);
                }
                let _ = std::process::Command::new("systemctl")
                    .args(["--user", "daemon-reload"])
                    .output();
                let r = std::process::Command::new("systemctl")
                    .args(["--user", "enable", "--now", "tytus.service"])
                    .output();
                let rd = std::process::Command::new("systemctl")
                    .args(["--user", "enable", "--now", "tytus-daemon.service"])
                    .output();
                let ok = r.map(|o| o.status.success()).unwrap_or(false);
                let ok_daemon = rd.map(|o| o.status.success()).unwrap_or(false);
                if json {
                    println!("{}", serde_json::json!({
                        "action":"install",
                        "unit_path":unit_path.to_string_lossy(),
                        "daemon_unit_path":daemon_unit_path.to_string_lossy(),
                        "enabled":ok,
                        "daemon_enabled":ok_daemon
                    }));
                } else {
                    println!("✓ systemd --user unit installed at {}", unit_path.display());
                    println!("✓ token-refresh daemon installed at {}", daemon_unit_path.display());
                    println!("  Auto-start on every login + 24/7 background refresh: enabled");
                }
            }
            AutostartAction::Uninstall => {
                let _ = std::process::Command::new("systemctl")
                    .args(["--user", "disable", "--now", "tytus.service"])
                    .output();
                let _ = std::process::Command::new("systemctl")
                    .args(["--user", "disable", "--now", "tytus-daemon.service"])
                    .output();
                let _ = std::fs::remove_file(&unit_path);
                let _ = std::fs::remove_file(&daemon_unit_path);
                if json {
                    println!("{}", serde_json::json!({
                        "action":"uninstall",
                        "unit_path":unit_path.to_string_lossy(),
                        "daemon_unit_path":daemon_unit_path.to_string_lossy()
                    }));
                } else {
                    println!("✓ systemd --user units removed. Auto-start and daemon disabled.");
                }
            }
            AutostartAction::Status => {
                let installed = unit_path.exists();
                let daemon_installed = daemon_unit_path.exists();
                let active = std::process::Command::new("systemctl")
                    .args(["--user", "is-enabled", "tytus.service"])
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);
                let daemon_active = std::process::Command::new("systemctl")
                    .args(["--user", "is-enabled", "tytus-daemon.service"])
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);
                if json {
                    println!("{}", serde_json::json!({
                        "action":"status",
                        "installed":installed,
                        "enabled":active,
                        "daemon_installed":daemon_installed,
                        "daemon_enabled":daemon_active
                    }));
                } else {
                    println!("Auto-start status:");
                    println!("  unit:           {} {}", unit_path.display(), if installed { "[installed]" } else { "[missing]" });
                    println!("  enabled:        {}", if active { "yes" } else { "no" });
                    println!("  daemon unit:    {} {}", daemon_unit_path.display(), if daemon_installed { "[installed]" } else { "[missing]" });
                    println!("  daemon enabled: {}", if daemon_active { "yes" } else { "no" });
                }
            }
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = action;
        let _ = json;
        eprintln!("Autostart is only supported on macOS and Linux.");
        std::process::exit(1);
    }
}

// ── Tray .app bundle + launch-at-login ──────────────────────

/// Manage `/Applications/Tytus.app` + the tray-launch-at-login LaunchAgent.
///
/// The .app bundle is what gives tytus-tray real macOS-citizen status:
///   * Findable in Spotlight (⌘+Space "Tytus")
///   * Draggable to the Dock
///   * Can be added to System Settings → Login Items
///   * LaunchServices handles double-click → launches the bundled executable
///
/// Internally the bundle's `Contents/MacOS/Tytus` is a copy of `~/bin/tytus-tray`
/// (not a symlink — Gatekeeper is flaky with symlinked binaries in .app bundles).
///
/// The LaunchAgent opens the bundle on login; combined with the single-instance
/// pidfile guard in tytus-tray itself, you can quit the tray any time and
/// reliably get exactly one tray back on next login.
#[cfg(target_os = "macos")]
fn cmd_tray(action: TrayAction, json: bool) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/unknown".into());
    let app_path = std::path::PathBuf::from("/Applications/Tytus.app");
    let macos_dir = app_path.join("Contents/MacOS");
    let resources_dir = app_path.join("Contents/Resources");
    let info_plist = app_path.join("Contents/Info.plist");
    let bundle_exe = macos_dir.join("Tytus");
    let plist_dir = std::path::PathBuf::from(&home).join("Library/LaunchAgents");
    let tray_plist_path = plist_dir.join("com.traylinx.tytus.tray.plist");

    match action {
        TrayAction::Install => {
            // Locate the tray binary. Prefer ~/bin/tytus-tray (where the
            // installer drops it); fall back to PATH; finally a sibling of
            // the running tytus binary (common when running from a build dir).
            let src = find_tray_binary(&home);
            let Some(src) = src else {
                eprintln!("tytus-tray binary not found. Install it first:");
                eprintln!("  cp target/release/tytus-tray ~/bin/tytus-tray");
                std::process::exit(1);
            };

            // Build bundle skeleton.
            for d in [&macos_dir, &resources_dir] {
                if let Err(e) = std::fs::create_dir_all(d) {
                    eprintln!("Failed to create {}: {}", d.display(), e);
                    std::process::exit(1);
                }
            }

            // Copy (not symlink — Gatekeeper rejects symlinked bundle executables).
            // If an old copy exists, replace it atomically via remove+copy.
            let _ = std::fs::remove_file(&bundle_exe);
            if let Err(e) = std::fs::copy(&src, &bundle_exe) {
                eprintln!("Failed to copy binary into bundle: {}", e);
                std::process::exit(1);
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(
                    &bundle_exe,
                    std::fs::Permissions::from_mode(0o755),
                );
            }

            // Info.plist — LSUIElement=true keeps the T out of the Dock
            // (same technique Ollama uses). Version string tracks the CLI.
            let plist = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDisplayName</key>
    <string>Tytus</string>
    <key>CFBundleName</key>
    <string>Tytus</string>
    <key>CFBundleExecutable</key>
    <string>Tytus</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
    <key>CFBundleIdentifier</key>
    <string>com.traylinx.tytus</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>{version}</string>
    <key>CFBundleVersion</key>
    <string>{version}</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.developer-tools</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>LSUIElement</key>
    <true/>
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>Tytus URL</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>tytus</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
"#,
                version = env!("CARGO_PKG_VERSION"),
            );
            if let Err(e) = std::fs::write(&info_plist, plist) {
                eprintln!("Failed to write Info.plist: {}", e);
                std::process::exit(1);
            }

            // App icon: best-effort. Failure doesn't block install — macOS
            // just falls back to the generic Exec icon.
            match generate_app_icon(&resources_dir) {
                Ok(()) => { /* Info.plist already names icon.icns */ }
                Err(e) => {
                    tracing::warn!("Skipping .icns generation: {}", e);
                }
            }

            // Poke LaunchServices so Spotlight picks up the bundle immediately
            // instead of after the next mds re-scan (which can take minutes).
            let _ = std::process::Command::new("/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister")
                .args(["-f", app_path.to_str().unwrap_or_default()])
                .output();

            // LaunchAgent: open Tytus.app at login. Using `open -a` lets
            // launchd reuse a running instance instead of racing against
            // the tray's single-instance guard.
            if let Err(e) = std::fs::create_dir_all(&plist_dir) {
                eprintln!("Failed to create LaunchAgents dir: {}", e);
                std::process::exit(1);
            }
            let tray_plist = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.traylinx.tytus.tray</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/bin/open</string>
        <string>-a</string>
        <string>{app}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
    <key>EnvironmentVariables</key>
    <dict>
        <key>HOME</key>
        <string>{home}</string>
    </dict>
</dict>
</plist>
"#,
                app = app_path.display(),
                home = home,
            );
            if let Err(e) = std::fs::write(&tray_plist_path, tray_plist) {
                eprintln!("Failed to write tray plist: {}", e);
                std::process::exit(1);
            }
            let _ = std::process::Command::new("launchctl")
                .args(["unload", tray_plist_path.to_str().unwrap_or_default()])
                .output();
            let loaded = std::process::Command::new("launchctl")
                .args(["load", "-w", tray_plist_path.to_str().unwrap_or_default()])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            // Start the tray right now so the user sees the T immediately.
            let _ = std::process::Command::new("/usr/bin/open")
                .args(["-a", app_path.to_str().unwrap_or_default()])
                .status();

            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "action": "install",
                        "app_path": app_path.to_string_lossy(),
                        "tray_plist_path": tray_plist_path.to_string_lossy(),
                        "loaded": loaded,
                    })
                );
            } else {
                println!("✓ /Applications/Tytus.app installed ({})", bundle_exe.display());
                println!("✓ Launch-at-login agent installed ({})", tray_plist_path.display());
                println!("✓ Tytus is now running in your menu bar");
                println!();
                println!("You can now:");
                println!("  • Find Tytus in Spotlight (⌘+Space, type 'Tytus')");
                println!("  • Drag Tytus.app to your Dock from /Applications");
                println!("  • Quit the tray anytime — it comes back on next login");
            }
        }
        TrayAction::Uninstall => {
            let _ = std::process::Command::new("launchctl")
                .args(["unload", "-w", tray_plist_path.to_str().unwrap_or_default()])
                .output();
            let _ = std::fs::remove_file(&tray_plist_path);
            let _ = std::fs::remove_dir_all(&app_path);
            // Best-effort: kill any running tray so /Applications/Tytus.app
            // doesn't linger in LaunchServices' cache.
            let _ = std::process::Command::new("pkill")
                .args(["-f", "tytus-tray"])
                .status();
            let _ = std::process::Command::new("/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister")
                .args(["-u", app_path.to_str().unwrap_or_default()])
                .output();
            if json {
                println!("{}", serde_json::json!({"action":"uninstall"}));
            } else {
                println!("✓ Removed /Applications/Tytus.app and launch-at-login agent.");
            }
        }
        TrayAction::Status => {
            let app_installed = app_path.exists();
            let plist_installed = tray_plist_path.exists();
            let loaded = std::process::Command::new("launchctl")
                .args(["list", "com.traylinx.tytus.tray"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            // Probe /tmp/tytus/tray.pid (the tray's single-instance lock)
            // and verify the pid is actually alive. More reliable than pgrep,
            // which has different process names for bundle vs raw binary.
            let running = std::fs::read_to_string("/tmp/tytus/tray.pid")
                .ok()
                .and_then(|s| s.trim().parse::<i32>().ok())
                .map(|pid| unsafe { libc::kill(pid, 0) } == 0)
                .unwrap_or(false);
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "app_installed": app_installed,
                        "launch_at_login": plist_installed,
                        "loaded": loaded,
                        "running": running,
                    })
                );
            } else {
                println!("Tray status:");
                println!("  /Applications/Tytus.app: {}", if app_installed { "[installed]" } else { "[missing]" });
                println!("  launch at login:        {}", if plist_installed && loaded { "yes" } else { "no" });
                println!("  running:                {}", if running { "yes" } else { "no" });
                if !app_installed {
                    println!();
                    println!("To install: tytus tray install");
                }
            }
        }
        TrayAction::Start => {
            if app_path.exists() {
                let _ = std::process::Command::new("/usr/bin/open")
                    .args(["-a", app_path.to_str().unwrap_or_default()])
                    .status();
            } else if let Some(fallback) = find_tray_binary(&home) {
                let _ = std::process::Command::new(fallback)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn();
            } else {
                eprintln!("tytus-tray not found. Install it: tytus tray install");
                std::process::exit(1);
            }
        }
    }
}

/// Generate `icon.icns` inside the bundle's Resources directory.
///
/// Strategy: draw a single 1024×1024 PNG of the T glyph on a macOS-like
/// rounded-rect tile, then let macOS' own `sips` resize it to the sizes
/// Apple's iconset format requires. `iconutil -c icns` composes the
/// final `.icns`. We rely on two binaries that are always present on
/// macOS — no third-party tooling.
///
/// The tile colour is a muted teal that matches the Traylinx brand
/// without being gaudy; a white T sits on top with a slight offset
/// so it looks like a flat-design glyph rather than a font render.
#[cfg(target_os = "macos")]
fn generate_app_icon(resources_dir: &std::path::Path) -> Result<(), String> {
    use std::process::Command;

    // Draw the master image (1024×1024 RGBA).
    let master = render_app_icon_rgba(1024);
    let iconset_dir = resources_dir.join("Tytus.iconset");
    std::fs::create_dir_all(&iconset_dir)
        .map_err(|e| format!("create iconset dir: {}", e))?;

    // Encode master PNG.
    let master_path = iconset_dir.join("icon_512x512@2x.png");
    write_png(&master_path, &master, 1024, 1024)
        .map_err(|e| format!("write master png: {}", e))?;

    // Apple's iconset requires these sizes (name → pixels):
    //   icon_16x16.png            16
    //   icon_16x16@2x.png         32
    //   icon_32x32.png            32
    //   icon_32x32@2x.png         64
    //   icon_128x128.png         128
    //   icon_128x128@2x.png      256
    //   icon_256x256.png         256
    //   icon_256x256@2x.png      512
    //   icon_512x512.png         512
    //   icon_512x512@2x.png     1024  (already written)
    let sizes: &[(&str, u32)] = &[
        ("icon_16x16.png",      16),
        ("icon_16x16@2x.png",   32),
        ("icon_32x32.png",      32),
        ("icon_32x32@2x.png",   64),
        ("icon_128x128.png",   128),
        ("icon_128x128@2x.png",256),
        ("icon_256x256.png",   256),
        ("icon_256x256@2x.png",512),
        ("icon_512x512.png",   512),
    ];
    for (name, px) in sizes {
        let out = iconset_dir.join(name);
        let status = Command::new("sips")
            .args(["-z", &px.to_string(), &px.to_string(),
                   master_path.to_str().unwrap_or_default(),
                   "--out", out.to_str().unwrap_or_default()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("sips: {}", e))?;
        if !status.success() {
            return Err(format!("sips failed resizing to {}px", px));
        }
    }

    // Compose .icns. `iconutil` reads the .iconset/ directory whose layout
    // we just produced and emits a single .icns file.
    let icns_path = resources_dir.join("icon.icns");
    let status = Command::new("iconutil")
        .args(["-c", "icns",
               iconset_dir.to_str().unwrap_or_default(),
               "-o", icns_path.to_str().unwrap_or_default()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("iconutil: {}", e))?;
    if !status.success() {
        return Err("iconutil failed".into());
    }

    // Clean up the intermediate iconset now that .icns is built.
    let _ = std::fs::remove_dir_all(&iconset_dir);
    Ok(())
}

/// Encode an RGBA buffer as a PNG file. Thin wrapper around the `png` crate.
#[cfg(target_os = "macos")]
fn write_png(
    path: &std::path::Path,
    rgba: &[u8],
    width: u32,
    height: u32,
) -> Result<(), std::io::Error> {
    let file = std::fs::File::create(path)?;
    let w = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()
        .map_err(|e| std::io::Error::other(format!("png header: {}", e)))?;
    writer.write_image_data(rgba)
        .map_err(|e| std::io::Error::other(format!("png data: {}", e)))?;
    Ok(())
}

/// Draw the Tytus.app icon into an RGBA buffer of the given size.
///
/// A macOS-squircle background (rounded rect with continuous corners) in
/// brand teal, with a bold white "T" glyph centered on it. Done with
/// raw pixel ops — no font rendering, no image crate — because we only
/// need one glyph at one size and `sips` handles downscaling for us.
#[cfg(target_os = "macos")]
fn render_app_icon_rgba(size: u32) -> Vec<u8> {
    let s = size as i32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let bg = [24u8, 128, 128, 255];      // Traylinx teal
    let bg_edge = [16u8, 96, 96, 255];    // subtle darker edge for depth
    let fg = [255u8, 255, 255, 255];      // white T

    // Continuous-corner squircle approximation: a superellipse with n≈5.
    // For each pixel, test (|dx|^n + |dy|^n) < r^n where r is half-size
    // minus a margin. This gives macOS Big Sur-style soft-corner tiles.
    // Using n=4 (close to iOS app icon mask). Exponent via powf on f32.
    let margin = (s as f32) * 0.08;
    let r = (s as f32) / 2.0 - margin;
    let cx = (s as f32) / 2.0;
    let cy = (s as f32) / 2.0;
    let n = 4.0f32;

    for y in 0..s {
        for x in 0..s {
            let dx = (x as f32 + 0.5 - cx).abs() / r;
            let dy = (y as f32 + 0.5 - cy).abs() / r;
            let d = dx.powf(n) + dy.powf(n);
            if d <= 1.0 {
                // Inside the squircle — base fill with a subtle radial
                // darkening near the edge so the icon reads as a 3D tile.
                let t = d.clamp(0.0, 1.0);
                let mix = |a: u8, b: u8| -> u8 {
                    (a as f32 * (1.0 - t * 0.25) + b as f32 * (t * 0.25)) as u8
                };
                let px = [mix(bg[0], bg_edge[0]), mix(bg[1], bg_edge[1]), mix(bg[2], bg_edge[2]), 255];
                let i = ((y * s + x) * 4) as usize;
                rgba[i..i+4].copy_from_slice(&px);
            }
        }
    }

    // T glyph: crossbar + stem, centred, with proportions that read well
    // at 16px (the smallest iconset size).
    //   crossbar: spans ~56% of the icon width, height ~14% of icon
    //   stem:     width ~18% of icon, runs from crossbar to ~80% height
    let cb_half_w = (s as f32 * 0.28) as i32;
    let cb_top = (s as f32 * 0.28) as i32;
    let cb_bot = (s as f32 * 0.42) as i32;
    let stem_half_w = (s as f32 * 0.09) as i32;
    let stem_top = cb_bot;
    let stem_bot = (s as f32 * 0.80) as i32;
    let icx = s / 2;

    for y in cb_top..cb_bot {
        for x in (icx - cb_half_w)..(icx + cb_half_w) {
            let i = ((y * s + x) * 4) as usize;
            if i + 4 <= rgba.len() {
                rgba[i..i+4].copy_from_slice(&fg);
            }
        }
    }
    for y in stem_top..stem_bot {
        for x in (icx - stem_half_w)..(icx + stem_half_w) {
            let i = ((y * s + x) * 4) as usize;
            if i + 4 <= rgba.len() {
                rgba[i..i+4].copy_from_slice(&fg);
            }
        }
    }

    rgba
}

#[cfg(target_os = "macos")]
fn find_tray_binary(home: &str) -> Option<std::path::PathBuf> {
    let candidates = [
        std::path::PathBuf::from(home).join("bin/tytus-tray"),
        std::path::PathBuf::from("/usr/local/bin/tytus-tray"),
        std::path::PathBuf::from("/opt/homebrew/bin/tytus-tray"),
    ];
    for c in &candidates {
        if c.exists() { return Some(c.clone()); }
    }
    // Sibling of the running tytus binary (common during dev).
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let sibling = dir.join("tytus-tray");
            if sibling.exists() { return Some(sibling); }
        }
    }
    // PATH lookup as last resort.
    std::process::Command::new("which")
        .arg("tytus-tray")
        .output()
        .ok()
        .and_then(|o| if o.status.success() {
            let p = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !p.is_empty() { Some(std::path::PathBuf::from(p)) } else { None }
        } else { None })
}

#[cfg(not(target_os = "macos"))]
fn cmd_tray(action: TrayAction, _json: bool) {
    // Linux/Windows don't have the same .app-bundle model. tytus-tray is
    // a regular binary on PATH; `tytus autostart install` already creates
    // the user-unit on Linux. For now, point users at that path.
    let _ = action;
    eprintln!("The Tytus.app bundle is a macOS feature.");
    eprintln!("On Linux, run: tytus autostart install  (systemd user unit)");
    eprintln!("Windows support is not yet implemented.");
}

// ── UI (localhost forwarder to OpenClaw control UI) ─────────

/// Start a TCP forwarder from 127.0.0.1:local_port → upstream, open the browser,
/// and block until Ctrl+C. Fixes the "browser refuses WebCrypto on non-localhost"
/// problem by giving the control UI a localhost secure context.
async fn cmd_ui(
    http: &atomek_core::HttpClient,
    pod_id: Option<String>,
    port_override: Option<u16>,
    no_open: bool,
    json: bool,
) {
    use std::process::Command;
    use tokio::net::{TcpListener, TcpStream};

    let state = CliState::load();
    // cmd_ui is a pure TCP forwarder — it never calls any Sentinel API,
    // so it doesn't actually need a valid refresh token. Checking only
    // `email + pods` lets the forwarder stay useful even when the
    // keychain ACL has lapsed (new code signature, rebuilt binary) and
    // the RT couldn't be loaded. Everything the forwarder touches —
    // tunnel iface, agent endpoint, upstream port — is in state.json
    // directly. If the tunnel has actually dropped, the upstream probe
    // (A1) cleans the forwarder up within ~15s.
    let has_email = state.email.as_deref().map(|e| !e.is_empty()).unwrap_or(false);
    if !has_email || state.pods.is_empty() {
        eprintln!("Not logged in or no pods configured. Run: tytus login && tytus connect");
        std::process::exit(1);
    }

    // Pick the pod: explicit --pod, else first in state
    let pod = match pod_id.as_deref() {
        Some(pid) => state.pods.iter().find(|p| p.pod_id == pid).cloned(),
        None => state.pods.first().cloned(),
    };
    let pod = match pod {
        Some(p) => p,
        None => {
            eprintln!("No pod available. Run: tytus connect");
            std::process::exit(1);
        }
    };

    // Refuse to try opening the UI for a pod whose agent doesn't exist.
    // The agent-less default pod (agent_type=="none") only serves AIL
    // through the sidecar's socat forwarder on 10.42.42.1:18080 — there
    // is no `/` web UI to show. Silently forwarding would land the user
    // on a blank page with no explanation.
    if pod.agent_type.as_deref() == Some("none") {
        eprintln!(
            "Pod {} is the default pod (AIL-only, no agent installed).\n\
             There's no agent UI to open. Either install an agent:\n  \
               tytus agent install openclaw --pod {}\n\
             or use the stable AIL endpoint directly:\n  \
               http://10.42.42.1:18080/v1",
            pod.pod_id, pod.pod_id,
        );
        std::process::exit(1);
    }

    // Each WG tunnel routes exactly one pod's /24 (e.g. pod 02 →
    // 10.18.2.0/24). Cross-pod traffic is blocked by sidecar iptables per
    // the security invariants — so if the current tunnel doesn't target
    // the pod the user asked for, the forwarder would just time out
    // silently. When that happens we auto-swap: disconnect the current
    // tunnel and reconnect targeting the requested pod. The user already
    // said "open pod N" by running `tytus ui --pod N`, so reinterpret the
    // intent as "get me into pod N whatever that takes".
    let pod_subnet_prefix = pod.ai_endpoint.as_deref()
        .and_then(|s| s.strip_prefix("http://"))
        .and_then(|s| s.split(':').next())
        .and_then(|host| {
            let parts: Vec<&str> = host.split('.').collect();
            if parts.len() == 4 { Some(format!("{}.{}.{}.", parts[0], parts[1], parts[2])) }
            else { None }
        });
    let tunnel_reaches_this_pod = pod_subnet_prefix.as_ref().map(|prefix| {
        let out = std::process::Command::new("ifconfig").output();
        match out {
            Ok(o) => String::from_utf8_lossy(&o.stdout).lines().any(|l| l.contains(prefix)),
            Err(_) => false,
        }
    }).unwrap_or(false);

    if !tunnel_reaches_this_pod {
        if !json {
            println!("→ Tunnel isn't routing to pod {} yet — switching now.", pod.pod_id);
            println!("  (each WireGuard tunnel serves one pod; cross-pod traffic is firewalled)");
        }
        // Tear down whatever is up, then bring up a tunnel for the target
        // pod. cmd_disconnect with no filter reaps every live pidfile;
        // cmd_connect handles sudo elevation the same way a fresh
        // `tytus connect --pod NN` would.
        cmd_disconnect(None, false).await;
        cmd_connect(http, Some(pod.pod_id.clone()), false).await;
    }

    // Re-resolve `pod` after the potential swap so upstream resolution
    // sees post-connect agent_endpoint / ai_endpoint / tunnel_iface.
    let pod = {
        let fresh = CliState::load();
        fresh.pods.iter().find(|p| p.pod_id == pod.pod_id).cloned().unwrap_or(pod)
    };

    // Resolve upstream: agent_endpoint is "10.X.Y.1:3000" (nemoclaw) or
    // "10.X.Y.1:8642" (hermes). If missing, derive from ai_endpoint.
    // Strip any http:// prefix — copy_bidirectional wants a raw host:port.
    let upstream = match pod.agent_endpoint.clone() {
        Some(ep) => ep.strip_prefix("http://").unwrap_or(&ep).to_string(),
        None => {
            match pod.ai_endpoint.as_deref() {
                Some(ai) => {
                    let default_port = agent_ui_port(pod.agent_type.as_deref().unwrap_or("nemoclaw"));
                    ai.strip_prefix("http://")
                        .and_then(|s| s.split(':').next())
                        .map(|host| format!("{}:{}", host, default_port))
                        .unwrap_or_else(|| {
                            eprintln!("Could not derive agent endpoint from state");
                            std::process::exit(1);
                        })
                }
                None => {
                    eprintln!("Pod has no agent_endpoint in state. Try: tytus connect");
                    std::process::exit(1);
                }
            }
        }
    };

    // Reuse an existing forwarder for this same pod if one is already
    // running. Without this check, every click of "Open in Browser"
    // spawned a fresh `tytus ui`, the old port 3000 was still held, and
    // the new one bound 3001 → user had to track N browser tabs.
    // Marker format: /tmp/tytus/ui-<pod>.port = JSON {"pid":N,"port":P}.
    // Stale markers (dead pid OR nothing listening on port) are ignored.
    let marker_path = std::path::PathBuf::from(format!("/tmp/tytus/ui-{}.port", pod.pod_id));
    if let Ok(raw) = std::fs::read_to_string(&marker_path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            let pid = v.get("pid").and_then(|x| x.as_u64()).unwrap_or(0);
            let port = v.get("port").and_then(|x| x.as_u64()).unwrap_or(0) as u16;
            let pid_alive = pid > 0 && unsafe { libc::kill(pid as i32, 0) == 0 };
            let port_alive = port > 0 && TcpStream::connect(("127.0.0.1", port)).await.is_ok();
            if pid_alive && port_alive {
                let existing_url = format!("http://localhost:{}/", port);
                if json {
                    println!("{}", serde_json::json!({
                        "local_url": existing_url,
                        "upstream": upstream,
                        "pod_id": pod.pod_id,
                        "status": "reused",
                        "forwarder_pid": pid,
                    }));
                } else {
                    println!("→ Forwarder for pod {} is already running (pid {}) on {}", pod.pod_id, pid, existing_url);
                    println!("  Reusing it — close the other Terminal window to stop it.");
                }
                if !no_open {
                    #[cfg(target_os = "macos")]
                    let _ = Command::new("open").arg(&existing_url).spawn();
                    #[cfg(target_os = "linux")]
                    let _ = Command::new("xdg-open").arg(&existing_url).spawn();
                }
                return;
            } else {
                // Stale marker — remove so we don't keep tripping on it.
                let _ = std::fs::remove_file(&marker_path);
            }
        }
    }

    // Port selection. In order:
    //   1. Explicit --port / -P wins and is used as-is.
    //   2. Derived port `18700 + pod_num` (pod 01 → 18701, pod 02 → 18702).
    //      The 18700s are a quiet neighborhood free of React/Next/Rails
    //      defaults, and the per-pod offset makes bookmarks deterministic.
    //   3. On collision, try +100 / +200 / +300 / +400 — same pod offset,
    //      walking up the decade. Preserves per-pod layout.
    //   4. Last resort: kernel-assigned ephemeral (bind port 0). The
    //      marker records whichever port we actually got.
    let pod_num: u16 = pod.pod_id.parse().unwrap_or(0);
    let derived_base: u16 = 18700u16.saturating_add(pod_num);
    let candidates: Vec<u16> = match port_override {
        Some(p) => vec![p],
        None => vec![
            derived_base,
            derived_base.saturating_add(100),
            derived_base.saturating_add(200),
            derived_base.saturating_add(300),
            derived_base.saturating_add(400),
        ],
    };

    let mut listener: Option<TcpListener> = None;
    let mut local_port: u16 = 0;
    for p in &candidates {
        if let Ok(l) = TcpListener::bind(("127.0.0.1", *p)).await {
            local_port = *p;
            listener = Some(l);
            break;
        }
    }
    if listener.is_none() {
        // All preferred ports taken. Kernel ephemeral — still works, just
        // not bookmarkable. The marker file + reuse path cover the
        // "reopen to same URL" case, so the cost is only the first URL
        // being e.g. localhost:49213 instead of localhost:18702.
        match TcpListener::bind(("127.0.0.1", 0)).await {
            Ok(l) => {
                if let Ok(addr) = l.local_addr() { local_port = addr.port(); }
                listener = Some(l);
            }
            Err(e) => {
                eprintln!("Could not bind any localhost port (preferred: {:?}): {}", candidates, e);
                std::process::exit(1);
            }
        }
    }
    let listener = listener.expect("listener bound above");

    // Publish our port marker so the next `tytus ui --pod N` click
    // discovers us instead of spawning a fresh forwarder on port+1.
    let marker_body = serde_json::json!({
        "pid": std::process::id(),
        "port": local_port,
        "upstream": upstream,
    });
    let _ = std::fs::write(&marker_path, marker_body.to_string());

    // C1: cap the forwarder log at 1 MB. Since the tray starts cmd_ui
    // detached with stderr piped to /tmp/tytus/ui-<pod>.log, a long-lived
    // session + a per-request eprintln (upstream failure spam) can grow
    // the file unbounded. Truncate on startup — we'd rather lose old
    // diagnostics than leak disk.
    let log_path = std::path::PathBuf::from(format!("/tmp/tytus/ui-{}.log", pod.pod_id));
    if let Ok(meta) = std::fs::metadata(&log_path) {
        if meta.len() > 1_048_576 {
            let _ = std::fs::File::create(&log_path);
        }
    }

    let url = format!("http://localhost:{}/", local_port);
    let upstream_clone = upstream.clone();

    if json {
        let out = serde_json::json!({
            "local_url": url,
            "upstream": upstream_clone,
            "pod_id": pod.pod_id,
            "status": "forwarding"
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
    } else {
        println!("Tytus UI — localhost forwarder");
        println!("  Pod:       {}", pod.pod_id);
        println!("  Upstream:  {}", upstream_clone);
        println!("  Local URL: {}", url);
        println!();
        println!("Browsers require HTTPS or localhost for WebCrypto — this forwarder");
        println!("gives the OpenClaw control UI a localhost secure context.");
        println!();
        println!("Press Ctrl+C to stop.");
    }

    // Open the browser unless --no-open. On macOS use `open`, on Linux `xdg-open`.
    if !no_open {
        #[cfg(target_os = "macos")]
        let _ = Command::new("open").arg(&url).spawn();
        #[cfg(target_os = "linux")]
        let _ = Command::new("xdg-open").arg(&url).spawn();
    }

    // A1: upstream health monitor. TCP-probe the upstream every 10s and
    // log transitions (healthy ↔ unreachable). We intentionally do NOT
    // shut the forwarder down on upstream failure: a userspace WireGuard
    // tunnel at ~5 KB/s drops probes regularly under load, and a brief
    // pod restart is not a reason to kill the listener. Keeping the
    // socket bound means the browser's reconnect loop survives the blip;
    // per-request upstream errors surface as 502 which the UI handles.
    let upstream_probe = upstream_clone.clone();
    tokio::spawn(async move {
        let mut last_healthy: bool = true;
        let mut tick = tokio::time::interval(std::time::Duration::from_secs(10));
        tick.tick().await; // burn the immediate tick; first real probe at 10s
        loop {
            tick.tick().await;
            let probe = tokio::time::timeout(
                std::time::Duration::from_secs(3),
                TcpStream::connect(&upstream_probe),
            ).await;
            let healthy = matches!(probe, Ok(Ok(_)));
            if healthy != last_healthy {
                if healthy {
                    eprintln!("[tytus ui] upstream {} reachable again", upstream_probe);
                } else {
                    eprintln!("[tytus ui] upstream {} transient probe fail — forwarder staying up", upstream_probe);
                }
                last_healthy = healthy;
            }
        }
    });

    let upstream_for_accept = upstream_clone.clone();
    // Per-pod static-asset cache. First fetch of a hashed-filename
    // bundle (Vite immutable /assets/<hash>.js) goes through the tunnel;
    // every subsequent fetch is served from /tmp/tytus/ui-<pod>-cache/.
    // Throughput over userspace WireGuard on macOS is ~3 KB/s sustained
    // for the 689 KB bundle OpenClaw ships — that's ~4 minutes for the
    // initial paint, every single tab reload. Disk cache makes the
    // second-and-beyond experience effectively instant (~0ms).
    let cache_dir = std::path::PathBuf::from(format!("/tmp/tytus/ui-{}-cache", pod.pod_id));
    let _ = std::fs::create_dir_all(&cache_dir);

    // Forwarder auto-injects the agent's gateway token on every
    // request. The token was stashed into state.json's PodEntry
    // during agent install (via configure_agent_for_zero_auth).
    // Without this, the browser would see "unauthorized: gateway
    // token missing" because OpenClaw's gateway enforces auth on
    // the WebSocket upgrade and we never let the user paste it.
    //
    // Self-heal: if state.json has no gateway_token (old state file,
    // manual edit, user wiped ~/.config/tytus, etc.), fetch the
    // current token directly from the agent via Provider /pod/agent/
    // exec. Provider A2A headers (secret_key + agent_user_id) live in
    // state.json and do NOT require the keychain refresh token — so
    // this path works even when keychain is broken (dev rebuilds,
    // background processes that can't show the approval dialog). On
    // success we persist the token back so the next start is instant.
    let mut gateway_token = {
        let fresh = CliState::load();
        fresh.pods.iter()
            .find(|p| p.pod_id == pod.pod_id)
            .and_then(|p| p.gateway_token.clone())
    };
    if gateway_token.as_deref().map(|t| t.is_empty()).unwrap_or(true) {
        if let Some(t) = fetch_gateway_token_via_provider(http, &pod.pod_id).await {
            if !t.is_empty() {
                gateway_token = Some(t.clone());
                let mut st = CliState::load();
                for p in st.pods.iter_mut() {
                    if p.pod_id == pod.pod_id { p.gateway_token = Some(t.clone()); }
                }
                st.save();
            }
        }
    }

    // Safety net for pods provisioned before the config.user.json overlay
    // writer landed: check the live agent config for our forwarder port
    // in `gateway.controlUi.allowedOrigins`; if missing, write the overlay
    // and trigger one restart. Idempotent — on pods installed with the new
    // code this is a cheap read-only probe.
    //
    // We do this in a background task so the forwarder starts serving the
    // 302-redirect + asset cache immediately. First browser WS upgrade
    // might race the restart by ~2 seconds; the UI's reconnect-with-
    // backoff handles that gracefully.
    {
        let http2 = http.clone();
        let pod_id2 = pod.pod_id.clone();
        tokio::spawn(async move {
            if let Err(e) = ensure_controlui_overlay(&http2, &pod_id2).await {
                eprintln!("[tytus ui] overlay ensure: {e}");
            }
        });
    }

    let agent_type_for_accept = pod.agent_type.clone().unwrap_or_else(|| "nemoclaw".into());
    let stable_user_key_for_accept = pod.stable_user_key.clone();
    let pod_id_for_accept = pod.pod_id.clone();
    let local_port_for_accept = local_port;

    let accept_loop = async move {
        loop {
            match listener.accept().await {
                Ok((client, _addr)) => {
                    let upstream_addr = upstream_for_accept.clone();
                    let cache_dir = cache_dir.clone();
                    let gateway_token = gateway_token.clone();
                    let agent_type = agent_type_for_accept.clone();
                    let stable_user_key = stable_user_key_for_accept.clone();
                    let pod_id = pod_id_for_accept.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_forwarder_connection(
                            client,
                            upstream_addr.clone(),
                            cache_dir,
                            gateway_token,
                            &agent_type,
                            &pod_id,
                            local_port_for_accept,
                            stable_user_key.as_deref(),
                        ).await {
                            eprintln!("[tytus ui] connection error (upstream {}): {}", upstream_addr, e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("[tytus ui] accept error: {}", e);
                    break;
                }
            }
        }
    };

    // Tell the compiler http is used (it's held for future needs — token fetch, etc.)
    let _ = http;

    // We want the forwarder to outlive the Terminal (or detached
    // spawn_detached) that started it. That means graceful shutdown on
    // BOTH SIGINT (user pressed Ctrl+C in Terminal) AND SIGTERM (the
    // "Stop Forwarder" menu item in the tray sends one). Without SIGTERM
    // in the select!, `tytus ui stop` would have to `kill -9` us and the
    // marker file would leak.
    use tokio::signal::unix::{signal, SignalKind};
    let mut sigterm = signal(SignalKind::terminate()).ok();
    tokio::select! {
        _ = accept_loop => {}
        _ = tokio::signal::ctrl_c() => {
            if !json { println!("\n✓ Forwarder stopped."); }
        }
        _ = async { if let Some(ref mut s) = sigterm { s.recv().await; } else { std::future::pending::<()>().await } } => {
            if !json { println!("\n✓ Forwarder stopped (SIGTERM)."); }
        }
    }

    // Clear our port marker so the next click doesn't try to reuse a
    // forwarder that's about to exit. Best-effort — a crash leaves the
    // marker, but the reuse probe also checks `kill(pid, 0)` before
    // trusting it.
    let _ = std::fs::remove_file(&marker_path);
}

/// Ensures the pod's `config.user.json` overlay contains the forwarder's
/// localhost port in `gateway.controlUi.allowedOrigins`. Without it, the
/// browser's WS upgrade fails origin-check (even though Host + Origin are
/// loopback) and silent local pairing can't fire.
///
/// Idempotent — does nothing if the overlay already has our entry.
/// Restarts the agent only when the overlay had to be written or fixed,
/// so "Open in Browser" on an already-provisioned pod is a no-op.
async fn ensure_controlui_overlay(
    http: &atomek_core::HttpClient,
    pod_id: &str,
) -> Result<(), String> {
    let st = CliState::load();
    // Skip for non-OpenClaw agents: Hermes has no browser UI, no
    // allowedOrigins gate, and doesn't care about Origin headers —
    // silent-pairing locality logic is OpenClaw-specific.
    let agent_type = st.pods.iter().find(|p| p.pod_id == pod_id)
        .and_then(|p| p.agent_type.clone()).unwrap_or_default();
    if agent_type == "hermes" || agent_type == "none" {
        return Ok(());
    }
    let secret = st.secret_key.as_deref().ok_or("no secret_key in state")?;
    let uid = st.agent_user_id.as_deref().ok_or("no agent_user_id in state")?;
    // Pull public-URL fields from the same PodEntry so the overlay always
    // reflects the current assignment. Per-pod-subdomain sprint (2026-04-23)
    // made pod_public_url the canonical browser origin; edge_public_url is
    // the 7-day legacy back-compat form we keep in the list until the
    // subdomain rip-out gate lifts.
    let (pod_public_url, edge_public_url) = {
        let p = st.pods.iter().find(|p| p.pod_id == pod_id);
        match p {
            Some(p) => (p.pod_public_url.clone(), p.edge_public_url.clone()),
            None => (None, None),
        }
    };
    let client = atomek_pods::TytusClient::new(http, secret, uid);
    let pod_num: u16 = pod_id.parse().unwrap_or(0);
    let fwd_port = 18700u16.saturating_add(pod_num);
    let wanted = format!("http://localhost:{}", fwd_port);
    let wanted2 = format!("http://127.0.0.1:{}", fwd_port);

    // Probe the live config. If all the origins we need are already in
    // allowedOrigins, skip the write + restart — no-op on already-healthy
    // pods. If ANY is missing (e.g. pods provisioned before the per-pod-
    // subdomain sprint had pod_public_url added to the overlay), rewrite.
    let probe_cmd = "cat /app/workspace/.openclaw/config.json 2>/dev/null | node -e \
         \"let d='';process.stdin.on('data',c=>d+=c).on('end',()=>{\
         try{const c=JSON.parse(d);process.stdout.write(JSON.stringify(c.gateway?.controlUi?.allowedOrigins||[]))}\
         catch(e){process.stdout.write('[]')}})\"";
    let probe = atomek_pods::exec_in_agent(&client, pod_id, probe_cmd, 10)
        .await
        .map_err(|e| e.to_string())?;
    let origins: Vec<String> = serde_json::from_str(probe.stdout.as_deref().unwrap_or("[]"))
        .unwrap_or_default();
    let has = |needle: &str| origins.iter().any(|o| o == needle);
    let public_ok = pod_public_url.as_deref().map_or(true, |u| has(u));
    let edge_ok = edge_public_url.as_deref().map_or(true, |u| has(u));
    if has(&wanted) && has(&wanted2) && public_ok && edge_ok {
        return Ok(());
    }

    // Missing — write the overlay and restart.
    let mut origin_list: Vec<serde_json::Value> = vec![
        serde_json::Value::String("http://localhost:3000".into()),
        serde_json::Value::String("http://127.0.0.1:3000".into()),
        serde_json::Value::String(format!("http://10.18.{}.1:3000", pod_num)),
        serde_json::Value::String(wanted.clone()),
        serde_json::Value::String(wanted2.clone()),
    ];
    if let Some(u) = pod_public_url.as_deref() { origin_list.push(serde_json::Value::String(u.to_string())); }
    if let Some(u) = edge_public_url.as_deref() { origin_list.push(serde_json::Value::String(u.to_string())); }
    let overlay = serde_json::json!({
        "gateway": {
            "controlUi": {
                "allowedOrigins": origin_list
            }
        }
    }).to_string();
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(overlay.as_bytes());
    let write_cmd = format!(
        "node -e \"require('fs').writeFileSync('/app/workspace/.openclaw/config.user.json', \
         Buffer.from('{b64}','base64').toString('utf8'))\""
    );
    let _ = atomek_pods::exec_in_agent(&client, pod_id, &write_cmd, 10)
        .await
        .map_err(|e| e.to_string())?;
    atomek_pods::restart_agent(&client, pod_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Best-effort fetch of the agent's current auth token via Provider's
/// `/pod/agent/exec` route, using the A2A creds that already live in
/// state.json (`secret_key` + `agent_user_id`). No keychain / refresh-
/// token path involved — so this works even when keychain ACLs are
/// broken (typical for background-spawned forwarders that can't show
/// the macOS approval dialog). Returns None on any failure.
///
/// Branches on agent_type:
///   nemoclaw: reads gateway.auth.token from /app/workspace/.openclaw/
///             config.json (JSON, key regenerated on each restart from
///             a deterministic formula so it's always present).
///   hermes:   reads /app/workspace/.hermes/api_server_key (plain file
///             with the 48-hex API_SERVER_KEY the entrypoint derived).
async fn fetch_gateway_token_via_provider(
    http: &atomek_core::HttpClient,
    pod_id: &str,
) -> Option<String> {
    let st = CliState::load();
    let secret = st.secret_key.as_deref()?;
    let user_id = st.agent_user_id.as_deref()?;
    let agent_type = st.pods.iter()
        .find(|p| p.pod_id == pod_id)
        .and_then(|p| p.agent_type.clone())
        .unwrap_or_else(|| "nemoclaw".into());
    let client = atomek_pods::TytusClient::new(http, secret, user_id);
    let script = if agent_type == "hermes" {
        "cat /app/workspace/.hermes/api_server_key 2>/dev/null"
    } else {
        "cat /app/workspace/.openclaw/config.json 2>/dev/null | \
         node -e \"let d='';process.stdin.on('data',c=>d+=c).on('end',()=>{\
         try{const c=JSON.parse(d);\
         process.stdout.write((c.gateway&&c.gateway.auth&&c.gateway.auth.token)||'');}\
         catch(e){process.exit(0);}})\""
    };
    let res = atomek_pods::exec_in_agent(&client, pod_id, script, 10).await.ok()?;
    let token = res.stdout.unwrap_or_default().trim().to_string();
    if token.is_empty() { None } else { Some(token) }
}

/// Send SIGTERM to the UI forwarder for one pod (if running). Best-effort:
/// silently returns if no marker or dead pid. Used by A2 (`cmd_disconnect`
/// teardown) and A3 (`cmd_logout` teardown) to keep the forwarder's
/// lifecycle bound to the tunnel's / session's.
fn stop_ui_forwarder(pod_id: &str) {
    let path = format!("/tmp/tytus/ui-{}.port", pod_id);
    let raw = match std::fs::read_to_string(&path) { Ok(r) => r, Err(_) => return };
    let v: serde_json::Value = match serde_json::from_str(&raw) { Ok(v) => v, Err(_) => {
        let _ = std::fs::remove_file(&path); return;
    }};
    let pid = v.get("pid").and_then(|x| x.as_u64()).unwrap_or(0) as i32;
    if pid > 0 && unsafe { libc::kill(pid, 0) } == 0 {
        unsafe { libc::kill(pid, libc::SIGTERM); }
    }
    // Let cmd_ui clean its own marker on SIGTERM exit. Give it 250 ms;
    // if still there, remove to keep the tray from reusing a zombie.
    std::thread::sleep(std::time::Duration::from_millis(250));
    if std::path::Path::new(&path).exists() {
        let _ = std::fs::remove_file(&path);
    }
}

/// Enumerate pod_ids of every currently-running forwarder.
fn list_ui_forwarder_pods() -> Vec<String> {
    let mut pods = Vec::new();
    if let Ok(rd) = std::fs::read_dir("/tmp/tytus") {
        for entry in rd.flatten() {
            let n = entry.file_name();
            let name = n.to_string_lossy();
            if let Some(pod) = name.strip_prefix("ui-").and_then(|s| s.strip_suffix(".port")) {
                pods.push(pod.to_string());
            }
        }
    }
    pods
}

/// `tytus ui --stop [--pod N]`: send SIGTERM to any running UI forwarder
/// so the browser tab stays the address of record until the user chooses
/// to shut the forwarder down. Reads /tmp/tytus/ui-*.port markers to
/// discover pids. With --pod, acts on that one pod; without, stops them
/// all.
/// Handle one incoming client connection on the forwarder.
///
/// Reads the request head, decides "cacheable static asset GET" vs
/// "everything else", and dispatches accordingly:
///
///   - Cacheable GET: SHA-like path-to-filename translation → lookup in
///     cache_dir. On hit, serve bytes verbatim (we already wrote a full
///     HTTP/1.1 response when caching). On miss, open an upstream TCP
///     socket, forward the request, stream the response to the browser,
///     and — if status is 2xx — atomically persist the full response to
///     the cache. Connection closes after the response (no keep-alive on
///     the cache path; browsers handle this fine by opening new TCPs).
///
///   - Anything else (POST, WS upgrade, HTML root, API JSON, etc.):
///     fall through to the old raw-TCP copy_bidirectional path. We
///     already consumed the client's request head, so we replay it
///     verbatim to upstream before wiring the two sockets together.
///
/// Error handling: on any io error, the client connection is dropped.
/// Cache write failures are swallowed silently — we'd rather serve a
/// correct but uncached response than fail the request.
async fn handle_forwarder_connection(
    mut client: tokio::net::TcpStream,
    upstream_addr: String,
    cache_dir: std::path::PathBuf,
    gateway_token: Option<String>,
    agent_type: &str,
    pod_id: &str,
    local_port: u16,
    stable_user_key: Option<&str>,
) -> std::io::Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    // Read the request head (up to \r\n\r\n) with a bounded buffer so a
    // malformed client can't OOM us. 16 KB covers any realistic HTTP/1.1
    // headers — browsers cap at ~8 KB in practice.
    let mut buf: Vec<u8> = Vec::with_capacity(4096);
    let mut tmp = [0u8; 2048];
    let head_end = loop {
        if let Some(pos) = find_crlf2(&buf) {
            break pos;
        }
        if buf.len() >= 16 * 1024 {
            return Ok(()); // headers too large, drop the connection
        }
        let n = match tokio::time::timeout(
            std::time::Duration::from_secs(10),
            client.read(&mut tmp),
        ).await {
            Ok(Ok(n)) if n > 0 => n,
            _ => return Ok(()), // timeout or EOF before head complete
        };
        buf.extend_from_slice(&tmp[..n]);
    };

    // Parse the request line: `METHOD /path HTTP/1.1\r\n`.
    let (method, path) = match parse_request_line(&buf[..head_end]) {
        Some(v) => v,
        None => {
            // Unparseable — dump everything to upstream and bridge raw.
            return raw_proxy(client, &buf, &upstream_addr, gateway_token.as_deref()).await;
        }
    };
    let method = method.to_string();
    let path = path.to_string();

    // Per-agent request handling. Two distinct shapes:
    //
    // OpenClaw (nemoclaw): single listener serves both the chat-UI
    //   SPA and the WS RPC on the same port (3000). The UI reads
    //   `?token=<T>` from the URL on first load, stashes it in
    //   settings, then strips it via history.replaceState. Without
    //   the seed the UI dead-ends at "gateway token missing" because
    //   browsers can't set custom headers on `new WebSocket()`. So
    //   we 302-redirect HTML shell GETs with `?token=<T>` appended.
    //
    // Hermes: two separate listeners inside the pod — `hermes gateway
    //   run` on 8642 (OpenAI-compat API + cron) and `hermes dashboard`
    //   on 9119 (Vite/React management SPA). We multiplex on path:
    //     `/v1/*`, `/api/jobs*`, `/health*` → gateway (8642); inject
    //       `Authorization: Bearer <API_SERVER_KEY>` so SDKs don't
    //       need a key.
    //     everything else → dashboard (9119); no auth injection —
    //       the dashboard embeds its own ephemeral session token in
    //       the HTML (`window.__HERMES_SESSION_TOKEN__`) and the SPA
    //       reads it from there.
    //   No landing page / 302 dance needed — the dashboard IS the
    //   landing experience, self-contained.
    let head_bytes = &buf[..head_end];
    let is_ws_upgrade = is_websocket_upgrade(head_bytes);
    let path_has_token = path.contains("token=");
    let has_ext = is_cacheable_asset(&path);

    // For Hermes: split upstream by path; only inject auth when we
    // route to the gateway (API). Dashboard requests MUST pass through
    // with whatever Authorization the SPA decides to send (its own
    // session token) — overriding would 401 every /api/* call.
    let (effective_upstream, should_inject_auth): (String, bool) = if agent_type == "hermes" {
        if hermes_path_needs_api_upstream(&path) {
            (with_port(&upstream_addr, agent_api_port(agent_type)), true)
        } else {
            (with_port(&upstream_addr, agent_ui_port(agent_type)), false)
        }
    } else {
        (upstream_addr.clone(), true)
    };
    let forwarder_token = if should_inject_auth { gateway_token.as_deref() } else { None };

    if agent_type != "hermes"
        && method.eq_ignore_ascii_case("GET")
        && !is_ws_upgrade
        && !has_ext
        && !path_has_token
    {
        if let Some(ref t) = gateway_token {
            if !t.is_empty() {
                let sep = if path.contains('?') { '&' } else { '?' };
                let location = format!("{}{}token={}", path, sep, t);
                let resp = format!(
                    "HTTP/1.1 302 Found\r\nLocation: {}\r\nContent-Length: 0\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
                    location,
                );
                client.write_all(resp.as_bytes()).await?;
                return Ok(());
            }
        }
    }

    // Silence "unused variable" warnings for args that are only read
    // in the hermes landing path (which has now been removed — the
    // dashboard supersedes it).
    let _ = (local_port, pod_id, stable_user_key);

    let is_cacheable = method.eq_ignore_ascii_case("GET") && has_ext;
    if !is_cacheable {
        return raw_proxy(client, &buf, &effective_upstream, forwarder_token).await;
    }

    // Cache lookup.
    let cache_file = cache_dir.join(cache_key_for(&path));
    if let Ok(cached) = tokio::fs::read(&cache_file).await {
        client.write_all(&cached).await?;
        return Ok(());
    }

    // Cache miss — open upstream, forward request, stream response,
    // buffer response to cache for next time.
    let mut upstream = match TcpStream::connect(&upstream_addr).await {
        Ok(s) => s,
        Err(e) => {
            let body = format!("upstream connect failed: {}", e);
            let resp = format!(
                "HTTP/1.1 502 Bad Gateway\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(), body
            );
            let _ = client.write_all(resp.as_bytes()).await;
            return Ok(());
        }
    };
    // Synthesise a minimal HTTP/1.1 request with `Connection: close`
    // rather than replaying the browser's raw bytes. Two reasons:
    //   1. The browser sent `Connection: keep-alive`; with keep-alive
    //      upstream will hold the socket open for its 5s timeout after
    //      sending the body. `read_to_end` then blocks for that 5s
    //      extra per request — meaningful when the user is reloading.
    //   2. We're caching; we don't need cookies / Accept-Encoding / UA
    //      quirks to vary the response.
    // `Host` is cosmetic for this path since the upstream is a raw
    // TCP socket already resolved, but we set it so agent log lines
    // aren't blank.
    // Include the Authorization header so even cache-miss requests
    // go through auth. Belt-and-suspenders; most cacheable assets
    // don't require auth but the agent can be configured to require
    // auth on /assets/* too and we'd never want to cache a 401.
    let auth_line = match gateway_token.as_deref() {
        Some(t) if !t.is_empty() => format!("Authorization: Bearer {}\r\n", t),
        _ => String::new(),
    };
    let synthetic_req = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nAccept: */*\r\n{}Connection: close\r\n\r\n",
        path, upstream_addr, auth_line,
    );
    upstream.write_all(synthetic_req.as_bytes()).await?;
    let _ = &buf; // we read the client's request head to parse it, but don't replay it

    // Stream upstream → client in 16 KB chunks while accumulating the
    // full body in memory for the cache write at EOF. Previously we
    // buffered the entire response before writing anything to the
    // client: for a 689 KB bundle at ~5 KB/s that's 130 s of zero
    // browser progress followed by a burst. Browsers interpret long
    // silence as "stuck" and keep the spinner running forever even
    // though bytes would eventually arrive. Streaming lets the parser
    // start on the head of the JS while the tail is still on the wire.
    let mut response: Vec<u8> = Vec::with_capacity(64 * 1024);
    let mut chunk = [0u8; 16 * 1024];
    let mut client_abandoned = false;
    loop {
        let n = upstream.read(&mut chunk).await?;
        if n == 0 { break; }
        response.extend_from_slice(&chunk[..n]);
        if !client_abandoned {
            if client.write_all(&chunk[..n]).await.is_err() {
                // Client closed mid-stream. Keep reading upstream so
                // the cache file can still be complete for the next
                // request — no wasted work.
                client_abandoned = true;
            }
        }
    }

    // Only cache 2xx responses that are COMPLETE. The completeness check
    // matters: `upstream.read_to_end()` returns successfully even if the
    // upstream socket closes mid-body (tunnel hiccup, agent restart in
    // progress, WG re-keying glitch). Caching that truncated response
    // then serving it with its original Content-Length header makes
    // every subsequent browser request hang — the browser reads the
    // bytes we have, then waits forever for the missing remainder.
    // Observed 2026-04-19: main bundle cached at 111 KB with
    // Content-Length: 689625 after agent restart → page permanently
    // stuck at "loading".
    let cacheable_ok = response_is_2xx(&response)
        && !response.is_empty()
        && response_is_complete(&response);
    if cacheable_ok {
        // The cache dir may have been removed out from under us (user
        // deleted /tmp/tytus manually, periodic /tmp sweep, etc).
        // Re-create before the atomic write or we silently skip caching.
        if let Some(parent) = cache_file.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }
        // Atomic write: tmp file in same dir → rename. Avoids half-written
        // cache entries being served after a crash mid-write.
        let tmp_path = cache_file.with_extension("tmp");
        if tokio::fs::write(&tmp_path, &response).await.is_ok() {
            let _ = tokio::fs::rename(&tmp_path, &cache_file).await;
        }

        // Chunk-graph prefetch. Vite-built apps use dynamic `import()`
        // for code-splitting; the main bundle holds a __vite__mapDeps
        // table listing every chunk it may load at runtime, and those
        // chunks are NOT referenced in the HTML. Without prefetching,
        // the browser runs the main JS, hits an import(), and THAT
        // triggers a fresh tunnel round-trip for 10+ KB of JS at
        // ~3 KB/s — which is why the OpenClaw control UI stays blank
        // for 30–60 s after a "cached" page load. Fix: every time we
        // cache a JS response, scan the body for hashed-filename
        // refs and enqueue background prefetches for anything not
        // already on disk. Fire-and-forget, rate-limited by TCP.
        if path.ends_with(".js") || path.ends_with(".mjs") {
            spawn_chunk_prefetch(&response, cache_dir.clone(), upstream_addr.clone()).await;
        }
    } else if response_is_2xx(&response) && !response.is_empty() {
        eprintln!("[tytus ui] skipped caching truncated response for {}", path);
    }
    // Response was already streamed to the client during the read loop.
    // Nothing more to send.
    Ok(())
}

/// True when the response's body length matches its declared length.
/// Rejects:
///   - Content-Length header present but body shorter (truncation).
///   - Transfer-Encoding: chunked (we don't dechunk; don't cache).
/// Accepts:
///   - Content-Length present AND body == declared length.
///   - No length headers at all (rare; HTTP/1.0-style). We assume
///     connection-close-delimited and the whole thing was read.
fn response_is_complete(response: &[u8]) -> bool {
    let head_end = match response.windows(4).position(|w| w == b"\r\n\r\n") {
        Some(p) => p + 4,
        None => return false,
    };
    let head = &response[..head_end];
    let body_len = response.len() - head_end;

    // Parse headers case-insensitively for Transfer-Encoding / Content-Length.
    let head_str = match std::str::from_utf8(head) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let mut content_length: Option<usize> = None;
    for line in head_str.split("\r\n") {
        let lower = line.to_ascii_lowercase();
        if let Some(v) = lower.strip_prefix("content-length:") {
            content_length = v.trim().parse::<usize>().ok();
        }
        if let Some(v) = lower.strip_prefix("transfer-encoding:") {
            if v.contains("chunked") { return false; }
        }
    }
    match content_length {
        Some(cl) => body_len == cl,
        None => true,
    }
}

/// Scan a just-cached JS response body for Vite-style hashed asset
/// references (e.g. `agents-Bg94Sj_g.js`, `channel-config-extras-
/// VzujQvi4.js`) and fire-and-forget a background fetch for any that
/// aren't already cached. Limits concurrency to 3 so we don't saturate
/// the WireGuard tunnel and starve the foreground request.
async fn spawn_chunk_prefetch(
    response: &[u8],
    cache_dir: std::path::PathBuf,
    upstream_addr: String,
) {
    // Locate end of HTTP headers, then scan body for hashed filenames.
    let idx = match response.windows(4).position(|w| w == b"\r\n\r\n") {
        Some(i) => i + 4,
        None => return,
    };
    let body = match std::str::from_utf8(&response[idx..]) {
        Ok(s) => s,
        Err(_) => return,
    };

    // Hashed filename pattern: <name>-<8+ hash chars>.<ext>
    // Kept deliberately permissive; the cache lookup below is the real
    // filter (we only fetch if it's a known cacheable extension that
    // isn't already on disk).
    let mut candidates: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (start_idx, _) in body.match_indices(|c: char| c == '"' || c == '\'') {
        let rest = &body[start_idx + 1..];
        let end = match rest.find(|c: char| c == '"' || c == '\'') {
            Some(e) => e,
            None => continue,
        };
        let raw = &rest[..end];
        // Vite emits module IDs as "./<name>-<hash>.<ext>"; strip any
        // leading "./" or "/" so the rest of the filter sees a bare
        // filename. Without this, every dynamic import fell through
        // the "contains('/')" check and never prefetched.
        let candidate = raw.trim_start_matches("./").trim_start_matches('/');
        if !candidate.contains('-') { continue; }
        if candidate.contains('/') { continue; } // still multi-path? skip
        let ext = match candidate.rfind('.') { Some(p) => &candidate[p..], None => continue };
        if !matches!(ext, ".js" | ".mjs" | ".css" | ".svg" | ".png" | ".woff2" | ".wasm") {
            continue;
        }
        candidates.insert(candidate.to_string());
    }

    if candidates.is_empty() { return; }

    let (tx, rx) = tokio::sync::mpsc::channel::<String>(candidates.len().max(1));
    for c in candidates {
        let _ = tx.send(c).await;
    }
    drop(tx);

    // Pool of 3 workers. Each pulls from the queue and fetches any item
    // that isn't yet cached. Fetch uses the same synthetic minimal GET
    // we use for foreground misses.
    let rx = std::sync::Arc::new(tokio::sync::Mutex::new(rx));
    for _ in 0..3 {
        let rx = rx.clone();
        let cache_dir = cache_dir.clone();
        let upstream_addr = upstream_addr.clone();
        tokio::spawn(async move {
            loop {
                let name = {
                    let mut guard = rx.lock().await;
                    match guard.recv().await {
                        Some(n) => n,
                        None => return,
                    }
                };
                let asset_path = format!("/assets/{}", name);
                let cache_file = cache_dir.join(cache_key_for(&asset_path));
                if cache_file.exists() { continue; }
                let _ = fetch_and_cache_asset(&upstream_addr, &asset_path, &cache_file).await;
            }
        });
    }
}

/// One prefetch round-trip: synthetic GET to upstream, write the full
/// HTTP response to the cache file. Failures are swallowed — prefetch
/// is best-effort; a miss just turns into a foreground fetch later.
async fn fetch_and_cache_asset(
    upstream_addr: &str,
    path: &str,
    cache_file: &std::path::Path,
) -> std::io::Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut upstream = tokio::net::TcpStream::connect(upstream_addr).await?;
    let req = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nAccept: */*\r\nConnection: close\r\n\r\n",
        path, upstream_addr
    );
    upstream.write_all(req.as_bytes()).await?;
    let mut buf: Vec<u8> = Vec::with_capacity(64 * 1024);
    upstream.read_to_end(&mut buf).await?;
    if !response_is_2xx(&buf) || buf.is_empty() {
        return Ok(());
    }
    if let Some(parent) = cache_file.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    let tmp = cache_file.with_extension("tmp");
    if tokio::fs::write(&tmp, &buf).await.is_ok() {
        let _ = tokio::fs::rename(&tmp, cache_file).await;
    }
    Ok(())
}

/// Raw TCP proxy path: forward the already-consumed request head to
/// upstream, then bidirectional-copy until either side closes.
///
/// Only the Authorization header is injected. We deliberately do NOT
/// rewrite Host / Origin / Referer: OpenClaw's gateway requires a
/// loopback `Host` and `Origin` to grant "silent local pairing" for
/// browser Control UI clients (see isControlUiBrowserContainerLocal-
/// Equivalent and isLoopbackHost in server.impl). If we rewrote them
/// to the pod's `10.X.Y.1:3000`, origin-check would pass but pairing
/// would require a manual approval step that blocks the zero-config
/// flow. Instead we add `http://localhost:187NN` and
/// `http://127.0.0.1:187NN` to `gateway.controlUi.allowedOrigins` at
/// agent-install time, which gets BOTH the origin check AND silent
/// pairing through in one pass.
async fn raw_proxy(
    mut client: tokio::net::TcpStream,
    request_head: &[u8],
    upstream_addr: &str,
    gateway_token: Option<&str>,
) -> std::io::Result<()> {
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpStream;
    let mut upstream = match TcpStream::connect(upstream_addr).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[tytus ui] upstream connect to {} failed: {}", upstream_addr, e);
            return Ok(());
        }
    };
    let with_auth = inject_auth_header(request_head, gateway_token);
    upstream.write_all(&with_auth).await?;
    let _ = tokio::io::copy_bidirectional(&mut client, &mut upstream).await;
    Ok(())
}

/// Set `Authorization: Bearer <token>` on the request — removing any
/// existing Authorization header first so user-supplied dummies (SDK
/// defaults like `api_key="any-string"`) get replaced with the real
/// forwarder-injected credential.
///
/// Override semantics matter for the Hermes flow: the OpenAI Python /
/// Node SDKs refuse to construct a client without an api_key, so users
/// pass `api_key="sk-placeholder"`. If we preserved that, the upstream
/// (Hermes with API_SERVER_KEY auth) would reject every request with
/// "Invalid API key". Override means the SDK argument is cosmetic — the
/// forwarder is the source of truth.
///
/// No-op if `gateway_token` is None or empty; in that case the original
/// head passes through untouched so users with their own auth story
/// (e.g. plain wss proxy) aren't clobbered.
fn inject_auth_header(request_head: &[u8], gateway_token: Option<&str>) -> Vec<u8> {
    let Some(token) = gateway_token else { return request_head.to_vec(); };
    if token.is_empty() { return request_head.to_vec(); }
    let text = match std::str::from_utf8(request_head) {
        Ok(s) => s,
        Err(_) => return request_head.to_vec(),
    };
    // Drop any existing Authorization line(s), then append ours before
    // the blank-line terminator.
    let mut body_sep = match text.find("\r\n\r\n") {
        Some(p) => p,
        None => return request_head.to_vec(),
    };
    let (head_text, tail_text) = text.split_at(body_sep);
    let filtered_head: String = head_text
        .split("\r\n")
        .filter(|line| !line.to_ascii_lowercase().starts_with("authorization:"))
        .collect::<Vec<_>>()
        .join("\r\n");
    // Recompute body separator position after filtering.
    body_sep = filtered_head.len();
    let mut out = String::with_capacity(text.len() + 64);
    out.push_str(&filtered_head);
    out.push_str(&format!("\r\nAuthorization: Bearer {}", token));
    out.push_str(tail_text);
    let _ = body_sep;
    out.into_bytes()
}

/// Rewrite `Origin:` and `Referer:` header values so they look like the
/// request came from the upstream agent, not from localhost:<port>.
/// Case-insensitive header match, preserves the line terminator style
/// (\r\n). Operates on raw bytes to avoid UTF-8 re-encoding the entire
/// request. Non-HTTP / malformed requests fall through with no change.
///
/// NOTE: no longer called by raw_proxy. Kept for the unit tests that
/// cover its behavior, and for a possible future code path that needs
/// the rewrite (e.g. a non-OpenClaw agent that doesn't do silent-local
/// pairing and only checks allowedOrigins). Delete if we're sure.
#[allow(dead_code)]
fn rewrite_origin_headers(request_head: &[u8], upstream_addr: &str) -> Vec<u8> {
    let body = match std::str::from_utf8(request_head) {
        Ok(s) => s,
        Err(_) => return request_head.to_vec(),
    };
    let upstream_origin = format!("http://{}", upstream_addr);
    let mut out = String::with_capacity(body.len());
    for line in body.split_inclusive("\r\n") {
        let trimmed = line.trim_end_matches("\r\n");
        // Match "Origin: …", "Referer: …", "Host: …" case-insensitively.
        // Host matters for Node/Express-style apps that use req.hostname
        // for routing or signed cookies; without rewriting, the upstream
        // sees Host: localhost:18702 and a typo-prone fraction of apps
        // refuse to serve. Origin is the CSRF gate (OpenClaw's
        // allowedOrigins); Referer is a secondary hint some apps log.
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("origin:") {
            out.push_str(&format!("Origin: {}\r\n", upstream_origin));
        } else if lower.starts_with("host:") {
            out.push_str(&format!("Host: {}\r\n", upstream_addr));
        } else if lower.starts_with("referer:") {
            // Keep the original path component; only swap the scheme+host.
            // `Referer: http://localhost:18702/chat?session=main` →
            // `Referer: http://10.18.2.1:3000/chat?session=main`.
            let val = trimmed["referer:".len()..].trim_start();
            if let Some(path_start) = val.find("://").and_then(|i| val[i + 3..].find('/')) {
                let i = val.find("://").unwrap() + 3 + path_start;
                out.push_str(&format!("Referer: {}{}\r\n", upstream_origin, &val[i..]));
            } else {
                out.push_str(&format!("Referer: {}\r\n", upstream_origin));
            }
        } else {
            out.push_str(line);
        }
    }
    out.into_bytes()
}

/// Kept as dead code after the Hermes dashboard-proxy landed. Before
/// that change, Hermes pods had no browser UI so the forwarder served
/// this local landing page at GET /. Now that the pod runs
/// `hermes dashboard` on 9119 and we proxy to it, the function is
/// superseded — but we keep the renderer around as a fallback in case
/// we ever need to show SDK snippets from a pod that only runs the
/// gateway (e.g. --gateway-only flag, minimal image variants).
#[allow(dead_code)]
fn render_hermes_landing(
    local_port: u16,
    pod_id: &str,
    forwarder_key: Option<&str>,
    stable_user_key: Option<&str>,
) -> String {
    let base = format!("http://localhost:{}", local_port);
    // The forwarder injects Authorization on every request, so from the
    // user's side any non-empty placeholder works. Show a real hint if
    // we have one though, in case they want to bypass the forwarder.
    let sdk_key = "sk-any-string-the-forwarder-injects-the-real-one";
    let fwd = forwarder_key.unwrap_or("(not cached yet — restart the agent)");
    let stable = stable_user_key.unwrap_or("(no stable key — run `tytus env`)");
    format!(r#"<!doctype html><html><head><meta charset="utf-8">
<title>Tytus · Hermes · pod {pod}</title>
<meta name="viewport" content="width=device-width, initial-scale=1">
<style>
body{{font:16px/1.55 -apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;max-width:780px;margin:40px auto;padding:0 24px;background:#0b0d10;color:#d8dee4}}
h1{{font-size:22px;margin:0 0 8px;color:#fff}}
h2{{font-size:15px;text-transform:uppercase;letter-spacing:.04em;color:#8a99ab;margin:32px 0 10px;font-weight:600}}
p{{color:#b7c2d0}}
code,pre{{font:13px/1.55 ui-monospace,SFMono-Regular,Menlo,monospace;background:#151a20;border:1px solid #222a33;border-radius:8px}}
code{{padding:2px 6px;color:#e9eef5}}
pre{{padding:14px 16px;overflow-x:auto;color:#e9eef5;margin:8px 0}}
.k{{color:#6cb;user-select:all}}
.dim{{color:#7d8896}}
.card{{background:#11151a;border:1px solid #222a33;border-radius:10px;padding:18px 20px;margin:14px 0}}
a{{color:#7cbff0}}
</style></head><body>
<h1>⚕ Hermes on Tytus pod {pod}</h1>
<p>OpenAI-compatible gateway proxied on <code>{base}</code>. No chat UI — point any OpenAI SDK here and it just works. The forwarder injects auth on every request; SDKs only need a non-empty <code>api_key</code>.</p>

<h2>curl</h2>
<pre>curl {base}/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{{"model":"ail-compound","messages":[{{"role":"user","content":"hello"}}]}}'</pre>

<h2>Python (openai)</h2>
<pre>from openai import OpenAI
client = OpenAI(base_url="{base}/v1", api_key="{sdk_key}")
r = client.chat.completions.create(model="ail-compound", messages=[{{"role":"user","content":"hi"}}])
print(r.choices[0].message.content)</pre>

<h2>Node (openai)</h2>
<pre>import OpenAI from "openai";
const client = new OpenAI({{ baseURL: "{base}/v1", apiKey: "{sdk_key}" }});
const r = await client.chat.completions.create({{
  model: "ail-compound",
  messages: [{{ role: "user", content: "hi" }}],
}});
console.log(r.choices[0].message.content);</pre>

<h2>Env vars (for any tool)</h2>
<pre>export OPENAI_BASE_URL="{base}/v1"
export OPENAI_API_KEY="{sdk_key}"</pre>

<div class="card">
<h2 style="margin-top:0">Fallback / direct access</h2>
<p class="dim">The forwarder handles auth for you. If you need to bypass it:</p>
<p class="dim">Hermes API key (forwarder-injected): <code class="k">{fwd}</code></p>
<p class="dim">SwitchAILocal stable key (any pod): <code class="k">{stable}</code></p>
</div>

<p class="dim">Stop the forwarder: <code>tytus ui --pod {pod} --stop</code> · Docs: <code>tytus llm-docs</code></p>
</body></html>"#, pod = pod_id, base = base, sdk_key = sdk_key, fwd = fwd, stable = stable)
}

/// Position of the \r\n\r\n header terminator + 4 (start of body), if any.
fn find_crlf2(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n").map(|p| p + 4)
}

/// Per-agent port convention. Two ports matter:
///
///   - UI port — where the browser lands when the user clicks
///     "Open in Browser". This is what `agent_endpoint` in state.json
///     points at and the default upstream for the forwarder.
///   - API port — where OpenAI-compatible traffic (`/v1/*`) and other
///     programmatic endpoints live. For single-listener agents this
///     is the same port as the UI; for Hermes it's a separate server
///     (hermes gateway run on 8642 alongside hermes dashboard on 9119).
///
/// Keeping these as pure functions (not state) means the convention
/// lives in one place and the forwarder can multiplex without any
/// provider/DAM round-trip.
fn agent_ui_port(agent_type: &str) -> u16 {
    match agent_type { "hermes" => 9119, _ => 3000 }
}

fn agent_api_port(agent_type: &str) -> u16 {
    match agent_type { "hermes" => 8642, _ => 3000 }
}

/// For the given request path on a Hermes pod, decide which upstream
/// the forwarder should target. OpenAI-compat routes and the gateway
/// health probes go to the gateway; everything else (SPA, /api/* for
/// dashboard config, static assets) goes to the dashboard.
fn hermes_path_needs_api_upstream(path: &str) -> bool {
    let p = path.split('?').next().unwrap_or(path);
    p.starts_with("/v1/")
        || p == "/health"
        || p == "/health/detailed"
        || p.starts_with("/api/jobs")
}

/// Swap the port component of a `host:port` upstream string.
fn with_port(upstream: &str, port: u16) -> String {
    match upstream.rfind(':') {
        Some(i) => format!("{}:{}", &upstream[..i], port),
        None => format!("{}:{}", upstream, port),
    }
}

/// True if this request is a WebSocket upgrade (`Upgrade: websocket` +
/// `Connection: upgrade`, case-insensitive, possibly with other tokens in
/// Connection like `keep-alive, Upgrade`). We must NOT serve a 302
/// redirect for these: WS clients don't follow HTTP redirects, the
/// handshake just fails.
fn is_websocket_upgrade(head: &[u8]) -> bool {
    let s = match std::str::from_utf8(head) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let mut has_ws_upgrade = false;
    let mut has_conn_upgrade = false;
    for line in s.split("\r\n") {
        let lower = line.to_ascii_lowercase();
        if let Some(v) = lower.strip_prefix("upgrade:") {
            if v.trim() == "websocket" {
                has_ws_upgrade = true;
            }
        }
        if let Some(v) = lower.strip_prefix("connection:") {
            if v.split(',').any(|tok| tok.trim() == "upgrade") {
                has_conn_upgrade = true;
            }
        }
    }
    has_ws_upgrade && has_conn_upgrade
}

/// Parse `METHOD path HTTP/1.1\r\n…` → (method, path). Returns None for
/// malformed input. The trailing CRLF is guaranteed by find_crlf2's
/// caller contract.
fn parse_request_line(head: &[u8]) -> Option<(&str, &str)> {
    let line_end = head.iter().position(|&b| b == b'\r')?;
    let line = std::str::from_utf8(&head[..line_end]).ok()?;
    let mut parts = line.splitn(3, ' ');
    let method = parts.next()?;
    let path = parts.next()?;
    Some((method, path))
}

/// True if `path` points at a cacheable static asset.  Vite and similar
/// bundlers embed a content hash in the filename (e.g.
/// /assets/index-Dts6VHgr.js), so content for a given path is
/// effectively immutable — caching forever is safe. We deliberately
/// skip .json: an app API endpoint could return JSON at an arbitrary
/// path (/api/sessions.json would match otherwise).
fn is_cacheable_asset(path: &str) -> bool {
    let p = path.split('?').next().unwrap_or(path);
    let ext = match p.rfind('.') {
        Some(pos) => &p[pos..],
        None => return false,
    };
    matches!(ext,
        ".js" | ".mjs" | ".css" | ".svg" | ".png" | ".jpg" | ".jpeg" |
        ".gif" | ".webp" | ".ico" | ".woff" | ".woff2" | ".ttf" | ".otf" |
        ".map" | ".wasm"
    )
}

/// URL path → deterministic, filesystem-safe cache filename.
/// `/assets/index-Dts6VHgr.js` becomes `_assets_index-Dts6VHgr.js`,
/// readable in `ls` for easy debugging. Query string is dropped (asset
/// URLs with ?v=… busting are rare and would only fragment the cache).
fn cache_key_for(path: &str) -> String {
    let p = path.split('?').next().unwrap_or(path);
    p.chars()
        .map(|c| match c {
            '/' => '_',
            c if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' => c,
            _ => '-',
        })
        .collect()
}

/// True if response starts with "HTTP/1.1 2xx" (or HTTP/1.0 2xx).
fn response_is_2xx(response: &[u8]) -> bool {
    // "HTTP/1.1 200 OK\r\n…"
    if response.len() < 12 { return false; }
    // Find the status code — three digits starting at byte 9.
    let code = &response[9..12];
    code[0] == b'2' && code[1].is_ascii_digit() && code[2].is_ascii_digit()
}

async fn cmd_ui_stop(pod_id: Option<String>, json: bool) {
    let dir = std::path::PathBuf::from("/tmp/tytus");
    let mut stopped: Vec<(String, i32)> = Vec::new();
    let mut stale: Vec<String> = Vec::new();

    let markers: Vec<std::path::PathBuf> = match std::fs::read_dir(&dir) {
        Ok(rd) => rd.filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| {
                let n = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                n.starts_with("ui-") && n.ends_with(".port")
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    for path in markers {
        let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
        let pod = fname.trim_start_matches("ui-").trim_end_matches(".port").to_string();
        if let Some(ref want) = pod_id {
            if &pod != want { continue; }
        }
        let raw = match std::fs::read_to_string(&path) { Ok(r) => r, Err(_) => continue };
        let v: serde_json::Value = match serde_json::from_str(&raw) { Ok(v) => v, Err(_) => {
            let _ = std::fs::remove_file(&path); stale.push(pod); continue;
        }};
        let pid = v.get("pid").and_then(|x| x.as_u64()).unwrap_or(0) as i32;
        if pid <= 0 || unsafe { libc::kill(pid, 0) } != 0 {
            let _ = std::fs::remove_file(&path);
            stale.push(pod);
            continue;
        }
        unsafe { libc::kill(pid, libc::SIGTERM); }
        // Give it a moment to clean up its own marker; if it's stuck, we
        // remove the file ourselves so the tray doesn't reuse it.
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        if unsafe { libc::kill(pid, 0) } == 0 {
            let _ = std::fs::remove_file(&path);
        }
        stopped.push((pod, pid));
    }

    if json {
        println!("{}", serde_json::json!({
            "stopped": stopped.iter().map(|(p, pid)| serde_json::json!({"pod_id": p, "pid": pid})).collect::<Vec<_>>(),
            "stale_markers_cleaned": stale,
        }));
    } else {
        if stopped.is_empty() && stale.is_empty() {
            match pod_id {
                Some(p) => println!("No forwarder running for pod {}", p),
                None    => println!("No forwarders running"),
            }
        }
        for (p, pid) in &stopped { println!("✓ Stopped forwarder for pod {} (pid {})", p, pid); }
        for p in &stale { println!("→ Cleaned stale marker for pod {}", p); }
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

    // 3. Token validity.
    // Be honest about what the user should do. The old "will auto-refresh"
    // text was misleading when the refresh token itself had expired
    // server-side — the daemon would spin forever without progress. We
    // can't cheaply prove the RT is dead without burning a refresh call,
    // so we hint at both paths: if the daemon's alive and reachable, it's
    // probably transient; if not, re-login.
    let token_valid = state.has_valid_token();
    let daemon_alive = std::path::Path::new("/tmp/tytus/daemon.sock").exists();
    checks.push(("token_valid", token_valid,
        if token_valid {
            "Access token current".into()
        } else if state.refresh_token.is_some() && daemon_alive {
            "Expired — daemon will auto-refresh within 30 min (if this persists, run: tytus login)".into()
        } else if state.refresh_token.is_some() {
            "Expired — daemon not running. Try: tytus daemon run, or run: tytus login".into()
        } else {
            "No token — run: tytus login".into()
        }
    ));

    // 4. Tytus subscription
    checks.push(("subscription", state.secret_key.is_some(),
        if let Some(ref tier) = state.tier { format!("Plan: {}", tier) }
        else { "No subscription. Upgrade at traylinx.com".into() }
    ));

    // 5. Default pod (added SPRINT §6 B3). Separate check from "any pods"
    // so the doctor distinguishes "no AIL access" from "pods but no agent".
    let default_pod = state.pods.iter().find(|p| p.agent_type.as_deref() == Some("none"));
    checks.push(("default_pod", default_pod.is_some(),
        if let Some(p) = default_pod {
            format!("Pod {} (AIL-only, 0 units)", p.pod_id)
        } else if state.is_logged_in() {
            "Missing — run: tytus login (auto-provisions) or tytus connect".into()
        } else {
            "No login yet".into()
        }
    ));

    // 6. Pods
    checks.push(("pods", !state.pods.is_empty(),
        if state.pods.is_empty() { "No pods. Run: tytus connect".into() }
        else { format!("{} pod(s)", state.pods.len()) }
    ));

    // 6. Tunnel — union of state.json's tunnel_iface + live pidfiles
    // under /tmp/tytus/tunnel-*.pid. state.json alone misreports "Not
    // running" whenever tunnel_iface didn't get preserved across a
    // login/install cycle; the pidfile is the authoritative signal for
    // "daemon alive".
    let mut live_tunnel_pods: Vec<String> = Vec::new();
    let mut live_tunnel_ifaces: Vec<String> = Vec::new();
    for pod in &state.pods {
        let mut tunnel_ok = pod.tunnel_iface.is_some();
        if !tunnel_ok {
            let pidfile = format!("/tmp/tytus/tunnel-{}.pid", pod.pod_id);
            if let Ok(raw) = std::fs::read_to_string(&pidfile) {
                if let Ok(pid) = raw.trim().parse::<i32>() {
                    let alive = pid > 1 && unsafe {
                        if libc::kill(pid, 0) == 0 { true }
                        else { std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM) }
                    };
                    if alive { tunnel_ok = true; }
                }
            }
        }
        if tunnel_ok {
            live_tunnel_pods.push(pod.pod_id.clone());
            if let Some(ref iface) = pod.tunnel_iface {
                live_tunnel_ifaces.push(iface.clone());
            } else if let Ok(iface) = std::fs::read_to_string(format!("/tmp/tytus/tunnel-{}.iface", pod.pod_id)) {
                live_tunnel_ifaces.push(iface.trim().to_string());
            }
        }
    }
    let has_tunnel = !live_tunnel_pods.is_empty();
    checks.push(("tunnel", has_tunnel,
        if has_tunnel {
            if live_tunnel_ifaces.is_empty() {
                format!("Active for pod(s) {}", live_tunnel_pods.join(", "))
            } else {
                format!("Active on {}", live_tunnel_ifaces.join(", "))
            }
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

    // C2: UI forwarders — show what localhost:port → pod mappings are
    // currently bound. Orphaned forwarders from previous sessions show
    // up here, so the user can kill them with `tytus ui --stop`.
    let fwd_pods = list_ui_forwarder_pods();
    if fwd_pods.is_empty() {
        checks.push(("ui_forwarders", true, "None running".into()));
    } else {
        let mut lines: Vec<String> = Vec::new();
        let mut any_stale = false;
        for pod in &fwd_pods {
            let path = format!("/tmp/tytus/ui-{}.port", pod);
            if let Ok(raw) = std::fs::read_to_string(&path) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                    let pid = v.get("pid").and_then(|x| x.as_u64()).unwrap_or(0) as i32;
                    let port = v.get("port").and_then(|x| x.as_u64()).unwrap_or(0);
                    let alive = pid > 0 && unsafe { libc::kill(pid, 0) == 0 };
                    if alive {
                        lines.push(format!("pod {} → 127.0.0.1:{} (pid {})", pod, port, pid));
                    } else {
                        any_stale = true;
                        lines.push(format!("pod {} → STALE marker (pid {} dead)", pod, pid));
                    }
                }
            }
        }
        checks.push(("ui_forwarders", !any_stale, lines.join("; ")));
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

// ─────────────────────────────────────────────────────────────────────────
// LLM_DOCS — the canonical, structured reference for AI agents driving
// tytus-cli. Same content is exposed by:
//   * `tytus llm-docs` (this binary)
//   * MCP tool `tytus_docs` (the tytus-mcp binary)
// Source file lives at the workspace root so both crates can include it
// via include_str! and stay in sync without duplication.
// ─────────────────────────────────────────────────────────────────────────
const LLM_DOCS: &str = include_str!("../../llm-docs.md");

// Short "paste this into any AI tool" bootstrap prompt. Mirrors the 2md
// pattern: a single URL + instruction that teaches any AI agent how to
// drive the product natively. Printed by `tytus bootstrap-prompt`.
const BOOTSTRAP_PROMPT: &str = r#"Read https://get.traylinx.com/SKILL.md and follow the instructions to drive Tytus natively. Tytus is my private AI pod on Traylinx — a WireGuard-tunneled, OpenAI-compatible LLM gateway managed by the `tytus` CLI. Check `command -v tytus` first; if missing, install with:

curl -fsSL https://get.traylinx.com/install.sh | bash

Then start with `tytus status` and walk me through anything that's not ready.
"#;

const CLAUDE_MD_BLOCK: &str = r#"## Tytus Private AI Pod (driven via tytus-cli)

You have access to a Tytus private AI pod — an isolated, WireGuard-tunneled,
OpenAI-compatible LLM gateway running on the user's account. You can use it
either as your own model backend OR as a deployable agent runtime that lives
inside the pod itself.

The user runs the `tytus` CLI and you can drive every part of it. Always
prefer `tytus` commands over hand-crafting curl calls — `tytus` knows the
current state, the stable endpoint, and the per-user key.

### One-command full reference (read this first)
```bash
tytus llm-docs              # comprehensive structured reference for AI agents
```
Run this whenever you need the authoritative command surface, error
messages, troubleshooting recipes, and known caveats.

### Mental model
- **Tytus** = customer name for the private AI pod product (Traylinx brand)
- **Pod** = one user's isolated slice: a WireGuard sidecar + an agent container
- **Agents** (containerised AIs running INSIDE a pod):
  - `nemoclaw` = OpenClaw runtime + NemoClaw sandboxing blueprint (1 unit, port 3000)
  - `hermes` = Nous Research Hermes gateway (2 units, port 8642)
- **Plan tiers**: Explorer (1 unit), Creator (2 units), Operator (4 units).
  Unit budget is enforced atomically by Scalesys; you cannot overspend.
- **SwitchAILocal**: the OpenAI-compatible LLM gateway on every droplet.
  Available models on this droplet: `ail-compound`, `ail-image`, `ail-embed`,
  `minimax/ail-compound`, `minimax/ail-image` (proxied to MiniMax M2.7).

### Stable URL + stable user key (do not invent your own values)
```bash
eval "$(tytus env --export)"
echo $OPENAI_BASE_URL    # → http://10.42.42.1:18080/v1   (constant forever)
echo $OPENAI_API_KEY     # → sk-tytus-user-<32hex>         (per-user, persists)
```
Both values are stable across pod revoke/reallocate, agent swaps, droplet
migration. Never hardcode them in source — always read from `tytus env`.

For per-pod debug values (the legacy raw pair) use `tytus env --raw`.

### Command surface (every subcommand)
```bash
# Identity
tytus login                  # browser device-auth via Sentinel
tytus logout                 # revoke all pods + clear local state
tytus status [--json]        # plan, pods, units, tunnel state
tytus doctor                 # full diagnostic (auth, tunnel, gateway, MCP)

# Pods
tytus setup                  # interactive wizard: auth → pick → tunnel → test
tytus connect [--agent nemoclaw|hermes] [--pod NN]
tytus disconnect [--pod NN]  # tear down tunnel daemon, leave allocation
tytus revoke <pod_id>        # free units (does NOT need disconnect first)
tytus restart [--pod NN]     # restart agent container (re-runs entry script)

# Working with the pod's gateway
tytus env [--export] [--raw] # connection vars (default: stable, --raw: per-pod)
tytus test                   # full E2E health: auth + tunnel + gateway + chat
tytus chat [--model ail-compound]
tytus exec [--pod NN] [--timeout N] "<shell command in agent container>"
tytus configure              # interactive overlay editor for agent config

# Integrations
tytus link [DIR] [--only claude|agents|kilocode|opencode|archon|shell]
tytus mcp [--format claude|kilocode|opencode|archon|json]
tytus bootstrap-prompt       # paste this into any AI tool to enable Tytus
tytus llm-docs               # the doc you should read before driving Tytus
```

### Recipe: ensure the user has a working pod, then chat
```bash
tytus status --json | jq -e '.pods | length > 0' \
    || tytus connect --agent nemoclaw
tytus test                                                  # confirm green
eval "$(tytus env --export)"                                # load stable pair
curl -sS "$OPENAI_BASE_URL/chat/completions" \
    -H "Authorization: Bearer $OPENAI_API_KEY" \
    -H "Content-Type: application/json" \
    -d '{"model":"ail-compound","messages":[{"role":"user","content":"hi"}]}'
```

### Recipe: deploy an agent INSIDE the pod (so it can run autonomously)
The agent is a containerised AI with its own filesystem and config.
```bash
tytus connect --agent nemoclaw                 # OpenClaw with NemoClaw sandbox
# OR
tytus connect --agent hermes                   # Nous Research Hermes (2 units)

# Customise the agent without rebuilding the image:
tytus exec --pod 02 "cat /app/workspace/config.user.json.example"
tytus exec --pod 02 "cat > /app/workspace/.openclaw/config.user.json <<'JSON'
{ \"agents\": { \"defaults\": { \"contextTokens\": 64000, \"timeoutSeconds\": 300 } } }
JSON"
tytus restart --pod 02                          # picks up the overlay merge
```

### Available MCP tools (if `.mcp.json` is wired up)
| Tool | Purpose |
|---|---|
| `tytus_status` | Login state, plan, pods, tunnel — start here |
| `tytus_env` | Stable + raw connection details for the active pod |
| `tytus_models` | Live model list from the pod gateway |
| `tytus_chat` | Send a chat completion through the user's pod |
| `tytus_revoke` | Free a pod's units |
| `tytus_setup_guide` | What to tell the user when nothing is connected |
| `tytus_docs` | Returns the full LLM-facing reference (same as `tytus llm-docs`) |

### Troubleshooting cheat sheet
| Symptom | Cause | Fix |
|---|---|---|
| `Invalid API key` from gateway | Stable map not synced yet (race after fresh connect) | Wait 2s and retry, or `tytus restart` |
| Tunnel up but `curl 10.42.42.1` times out | Routing collision with another VPN on macOS | `tytus disconnect` other VPNs, then `tytus connect` |
| `No pods. Run: tytus connect` | User hasn't allocated yet | Run `tytus connect` (or `tytus setup` for the wizard) |
| OpenClaw falls back to `gpt-5.4` | `agents.defaults.model` typo in user overlay | Check JSON path is `agents.defaults.model` (singular `defaults`) |
| `Tunnel daemon already running` | Stale PID file | `tytus disconnect` then retry, or `kill $(cat /tmp/tytus/tunnel-NN.pid)` |
| `Pod config not ready` from /pod/request | peer.conf missing on droplet | Backend issue — escalate, do not retry blindly |

### Hard rules
1. Never paste a raw per-pod key into project files. Always reference
   `OPENAI_API_KEY` from `tytus env --export`.
2. Never hardcode `10.18.X.Y` addresses — they change. Use `10.42.42.1`.
3. Never run `tytus revoke` without telling the user — it costs them
   their workspace state on the pod.
4. Never call `sudo` directly to manipulate the tunnel; `tytus connect`
   handles elevation transparently via osascript / sudo -n / interactive sudo.
5. Treat `tytus llm-docs` as the source of truth. This block is a teaser.
"#;

const AGENTS_MD_BLOCK: &str = r#"## Tytus Private AI Pod (driven via tytus-cli)

You are an AI agent (OpenCode / Codex / Gemini / similar) running in a project
that has access to a Tytus private AI pod. Tytus is a WireGuard-tunneled,
isolated AI runtime owned by the user. The `tytus` CLI is your interface to it.

### Read this first
```bash
tytus llm-docs             # full structured reference for AI agents
```

### What is Tytus
- **Pod** = one user's isolated slice (WireGuard sidecar + agent container)
- **Two agent types** runnable inside a pod:
  - `nemoclaw` (1 unit, port 3000) — OpenClaw + NemoClaw sandbox blueprint
  - `hermes` (2 units, port 8642) — Nous Research Hermes
- **Plan tiers**: Explorer=1u, Creator=2u, Operator=4u
- **Models** on the gateway: `ail-compound`, `ail-image`, `ail-embed`,
  `minimax/ail-compound`, `minimax/ail-image`

### Stable connection (the pair to use in tools)
```bash
eval "$(tytus env --export)"
# OPENAI_BASE_URL=http://10.42.42.1:18080/v1   ← stable forever
# OPENAI_API_KEY=sk-tytus-user-<32hex>          ← stable per user
```

### Commands you should know
```bash
tytus status [--json]                       # account + pods + tunnel
tytus doctor                                # full health diagnostic
tytus connect [--agent nemoclaw|hermes]     # allocate + tunnel up
tytus disconnect [--pod NN]                 # tear down tunnel
tytus revoke <pod_id>                       # free units
tytus restart [--pod NN]                    # restart the agent container
tytus env [--export] [--raw]                # connection vars
tytus test                                  # E2E health check
tytus chat                                  # REPL against the pod
tytus exec [--pod NN] "<shell command>"     # run inside the agent container
tytus configure                             # interactive overlay editor
tytus setup                                 # full first-run wizard
```

### Recipe
```bash
# Make sure there's a pod
tytus status --json | jq -e '.pods | length > 0' || tytus connect

# Use it
eval "$(tytus env --export)"
curl "$OPENAI_BASE_URL/chat/completions" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"ail-compound","messages":[{"role":"user","content":"hi"}]}'
```

### Hard rules
1. Always read connection vars from `tytus env`, never hardcode.
2. The stable URL is always `10.42.42.1:18080`. Per-pod IPs (`10.18.X.Y`) change.
3. `tytus revoke` is destructive — it wipes the pod's workspace state. Confirm
   with the user first.
4. If `tytus llm-docs` exists, prefer it as the source of truth over this block.
"#;

const CLAUDE_COMMAND_TYTUS: &str = r#"---
description: "Drive the Tytus private AI pod — status, connect, test, chat"
argument-hint: "[status|connect|test|chat|exec|env|deploy AGENT|disconnect|revoke|setup|docs]"
---

You are driving the user's Tytus private AI pod via the `tytus` CLI.
Tytus is a WireGuard-tunneled, isolated LLM gateway running on the user's
Traylinx subscription. The CLI handles everything: auth, allocation, tunnel,
agent lifecycle, and stable endpoint management.

**Read the full reference before doing anything:**
```bash
tytus llm-docs
```
That command prints the authoritative documentation as Markdown — command
surface, models, plans, recipes, error catalog. Cache it in your context for
the rest of the session.

Then dispatch on `$ARGUMENTS`:

- **status** (default if no argument): `tytus status` — show plan, pods,
  tunnel state. If `--json` is needed for parsing, use `tytus status --json`.
  Always run `tytus doctor` if anything looks off.

- **connect**: `tytus connect [--agent nemoclaw|hermes]`. Default agent is
  nemoclaw (1 unit). Hermes costs 2 units. Confirm with the user before
  spending units.

- **test**: `tytus test` — full E2E health check (auth → pod → tunnel →
  gateway → sample chat). Use this to confirm everything is wired up.

- **chat**: `tytus chat [--model ail-compound]` — interactive REPL against
  the pod. Or run a one-shot chat completion via curl using the stable env.

- **exec "<command>"**: `tytus exec --pod NN "<command>"` runs a shell
  command inside the agent container. Useful for inspecting agent config,
  reading logs, or editing the user overlay file.

- **env**: `tytus env --export` prints the stable OPENAI_BASE_URL +
  OPENAI_API_KEY pair. Use `--raw` for the legacy per-pod values.

- **deploy AGENT** or **--agent AGENT**: shorthand for `tytus connect
  --agent <nemoclaw|hermes>`. Verify the user understands the unit cost.

- **disconnect**: `tytus disconnect` — tears down the tunnel daemon, leaves
  the allocation alive. Cheap to reconnect.

- **revoke**: `tytus revoke <pod_id>` — DESTRUCTIVE. Frees the units AND
  wipes the pod's workspace state. Always confirm with the user first.

- **setup**: `tytus setup` — full interactive wizard (login → plan → agent
  pick → tunnel → test). Best for first-run experiences.

- **docs**: `tytus llm-docs` — print the full reference (this is what you
  should consult before any non-trivial operation).

After running the requested command, summarize:
- Plan tier + units used / remaining
- Active pods (id, agent_type, tunnel state)
- The stable endpoint pair (don't print the full key in logs unless asked)
- Any actions the user should take next
"#;

const KILO_COMMAND_TYTUS: &str = r#"---
description: "Drive the Tytus private AI pod via tytus-cli (status / connect / test / chat / exec)"
---

You are an OpenCode/KiloCode agent with access to the user's Tytus
private AI pod via the `tytus` CLI. Read the full reference first:

```bash
tytus llm-docs
```

That command outputs the authoritative documentation: every subcommand,
the stable URL/key model, the agent types (nemoclaw=1u, hermes=2u),
the plan tiers (Explorer=1u, Creator=2u, Operator=4u), the models on the
gateway (ail-compound, ail-image, ail-embed), and a troubleshooting
catalog. Read it, then act.

Common flow:

```bash
tytus status                                   # what does the user have?
tytus connect [--agent nemoclaw|hermes]        # if no pod yet
tytus test                                     # E2E health
eval "$(tytus env --export)"                   # load OPENAI_* envs
tytus chat                                     # REPL, OR
tytus exec --pod NN "<shell command>"          # poke at the agent container
```

Stable endpoint after `tytus env --export`:
- `OPENAI_BASE_URL=http://10.42.42.1:18080/v1`
- `OPENAI_API_KEY=sk-tytus-user-<32hex>`

Hard rules:
1. Always go through `tytus`, never raw curl with hardcoded IPs.
2. `tytus revoke` is destructive — confirm first.
3. Prefer `tytus llm-docs` over this command body when in doubt.

Report: plan, units, pods, tunnel state, stable endpoint readiness, next steps.
"#;

const ARCHON_COMMAND_TYTUS: &str = r#"---
description: "Drive the user's Tytus private AI pod via tytus-cli"
---

You have the `tytus` CLI available. It manages a private AI pod on the
user's Traylinx subscription. Read the full reference before acting:

```bash
tytus llm-docs
```

Quick recipe:
```bash
tytus status                       # account + pods
tytus connect                      # allocate + tunnel (default: nemoclaw)
tytus test                         # E2E sanity
eval "$(tytus env --export)"       # OPENAI_BASE_URL + OPENAI_API_KEY
```

Stable endpoint pair (constant across pod rotations):
- URL: `http://10.42.42.1:18080/v1`
- Key: `sk-tytus-user-<32hex>` (one per user, persisted by Scalesys)

Agents you can deploy in a pod (`tytus connect --agent <name>`):
- `nemoclaw` (1 unit) — OpenClaw + NemoClaw sandbox blueprint
- `hermes` (2 units) — Nous Research Hermes

`tytus revoke <pod_id>` is destructive — confirm with the user.
Report login state, pods, tunnel, gateway reachability, and recommended next action.
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

/// Returns true if the token is still valid but expires within 10 minutes.
/// Used for opportunistic proactive refresh — failure is non-fatal.
fn should_proactively_refresh(state: &CliState) -> bool {
    if let (Some(_), Some(exp)) = (&state.access_token, state.expires_at_ms) {
        let now = chrono::Utc::now().timestamp_millis();
        // Token is valid (has_valid_token passed) but expires within 10 min
        (now + 600_000) >= exp
    } else {
        false
    }
}

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
    // Persist rotated RT to keychain. Sentinel invalidates the old RT on every
    // refresh; without this write the next process start loads the stale RT
    // from keychain and is forced into re-login. RT lives *only* in keychain
    // (state.json has skip_serializing on the field — see state.rs), so the
    // keychain is the one persistence point that matters.
    if let Some(ref email) = state.email {
        if !email.is_empty() {
            if let Err(e) = atomek_auth::KeychainStore::store_refresh_token(email, &result.refresh_token) {
                tracing::error!(
                    "CRITICAL: failed to persist rotated refresh token to keychain: {}. \
                     Next restart will require re-login.",
                    e
                );
                if !wizard::is_interactive() {
                    append_autostart_log(&format!(
                        "CRITICAL: keychain write failed after token rotation: {}", e
                    ));
                }
            }
        }
    }
}

/// Cheap TCP-level probe of the stable dual-bound gateway endpoint.
/// Returns true iff we got any HTTP response within 2s. Identical in
/// spirit to `tray/src/gateway_probe.rs` but kept local so the CLI
/// crate doesn't depend on the tray crate.
///
/// Used by `cmd_connect` to distinguish between "tunnel process is
/// alive and routing packets" (skip — already connected) and "tunnel
/// process is alive but gateway unreachable" (reap + re-activate).
/// Without this, a boringtun session that went dead under the idle-
/// bug or a network change would keep the user stuck on a dead
/// tunnel until they manually ran `tytus disconnect`.
fn probe_stable_gateway() -> bool {
    use std::io::{Read, Write};
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;
    let addr: SocketAddr = match "10.42.42.1:18080".parse() {
        Ok(a) => a,
        Err(_) => return false,
    };
    let timeout = Duration::from_secs(2);
    let mut stream = match TcpStream::connect_timeout(&addr, timeout) {
        Ok(s) => s,
        Err(_) => return false,
    };
    if stream.set_read_timeout(Some(timeout)).is_err() { return false; }
    if stream.set_write_timeout(Some(timeout)).is_err() { return false; }
    let req = b"GET /v1/models HTTP/1.0\r\nHost: 10.42.42.1:18080\r\nConnection: close\r\n\r\n";
    if stream.write_all(req).is_err() { return false; }
    let mut buf = [0u8; 16];
    let n = match stream.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return false,
    };
    n >= 5 && buf[..5] == *b"HTTP/"
}

pub(crate) async fn ensure_token(state: &mut CliState, http: &atomek_core::HttpClient) -> Result<(), atomek_core::AtomekError> {
    let headless = !wizard::is_interactive();

    if state.has_valid_token() {
        // Server-side validation: confirm the server agrees the token is valid.
        // If server says expired (clock skew or revoked), fall through to refresh.
        // On success, sync local expires_at_ms with server truth to fix clock drift.
        // trust_token: true means we believe the token is usable for this call.
        // Set to true on: (a) server confirmed valid, (b) network error but local
        // says valid (availability > correctness — blocking a paying user because
        // Sentinel is unreachable is worse than a downstream 401 that gets retried).
        // Set to false only when server explicitly says AuthExpired.
        let mut trust_token = false;
        if let Some(ref at) = state.access_token.clone() {
            match atomek_auth::validate_token(http, at).await {
                Ok(info) => {
                    // Sync local expiry with server-reported TTL
                    state.expires_at_ms = Some(
                        chrono::Utc::now().timestamp_millis() + (info.expires_in as i64 * 1000)
                    );
                    state.save();
                    trust_token = true;
                }
                Err(atomek_core::AtomekError::AuthExpired) => {
                    // Server says token is dead — fall through to refresh
                    tracing::warn!("Server rejected locally-valid token (clock skew or revoked)");
                    state.access_token = None;
                    state.expires_at_ms = None;
                    // Don't return — fall through to refresh below
                }
                Err(_) => {
                    // Network error hitting validation endpoint — trust local state.
                    // Design decision: availability over correctness. If Sentinel is
                    // unreachable, don't lock out a paying user. A downstream 401
                    // from the actual API will trigger re-auth if the token is truly dead.
                    tracing::debug!("Token validation endpoint unreachable, trusting local expiry");
                    trust_token = true;
                }
            }
        }

        // Re-check after possible server-side invalidation.
        // If we trust the token (server confirmed or network error with valid local),
        // attempt proactive refresh if expiring soon, but don't fall through to
        // mandatory refresh which would needlessly rotate the RT.
        if state.has_valid_token() || trust_token {
            if should_proactively_refresh(state) || (trust_token && !state.has_valid_token()) {
                // Proactive refresh: token is expiring soon. Non-fatal — token still works.
                let email_backup = state.email.clone();
                if let Some(ref rt) = state.refresh_token.clone() {
                    match atomek_auth::refresh_access_token(http, rt).await {
                        Ok(result) => {
                            update_tokens(state, &result, &email_backup);
                            // Critical save: RT was rotated server-side, old RT is dead
                            if let Err(e) = state.save_critical() {
                                tracing::error!("CRITICAL: Failed to save rotated tokens: {}. Re-login may be required.", e);
                                if headless {
                                    append_autostart_log(&format!("CRITICAL: save_critical failed after proactive refresh: {}", e));
                                }
                            }
                            tracing::debug!("Proactively refreshed token (was expiring soon)");
                        }
                        Err(e) => {
                            // Non-fatal: token still has some life left
                            tracing::debug!("Proactive refresh failed (non-fatal): {}", e);
                            if headless {
                                append_autostart_log(&format!("ensure_token: proactive refresh failed (non-fatal): {}", e));
                            }
                        }
                    }
                }
            }
            return Ok(());
        }
    }

    // Mandatory refresh: token is expired or server rejected it
    let email_backup = state.email.clone();
    let result = match state.refresh_token.clone() {
        Some(rt) => {
            match atomek_auth::refresh_access_token(http, &rt).await {
                Ok(result) => {
                    update_tokens(state, &result, &email_backup);
                    // Critical save: RT was rotated server-side, old RT is dead
                    if let Err(e) = state.save_critical() {
                        tracing::error!("CRITICAL: Failed to save rotated tokens: {}. Re-login may be required.", e);
                        if headless {
                            append_autostart_log(&format!("CRITICAL: save_critical failed after mandatory refresh: {}", e));
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    tracing::warn!("Token refresh failed: {}", e);
                    Err(e)
                }
            }
        }
        None => Err(atomek_core::AtomekError::Other(
            "No refresh token available — run 'tytus login' to re-authenticate".into(),
        )),
    };
    if headless {
        if let Err(ref e) = result {
            append_autostart_log(&format!(
                "ensure_token FAILED: {}. email={}, has_rt={}, has_at={}, expires_at_ms={:?}",
                e,
                state.email.as_deref().unwrap_or("none"),
                state.refresh_token.is_some(),
                state.access_token.is_some(),
                state.expires_at_ms,
            ));
        } else {
            append_autostart_log("ensure_token OK: token refreshed successfully");
        }
    }
    result
}

/// Detect and clean up stale tunnels: state says tunnel is active but the
/// daemon is dead or the interface no longer exists. Clears tunnel_iface on
/// affected pods so status/connect don't lie about connectivity.
fn reap_dead_tunnels(state: &mut CliState) {
    for pod in &mut state.pods {
        if let Some(ref iface) = pod.tunnel_iface {
            let pid_file = format!("/tmp/tytus/tunnel-{}.pid", pod.pod_id);
            let daemon_alive = std::fs::read_to_string(&pid_file)
                .ok()
                .and_then(|s| s.trim().parse::<u32>().ok())
                .map(|pid| {
                    // kill(pid, 0) checks if process exists without sending a signal.
                    // Returns 0 if we have permission, -1 with:
                    //   EPERM = process exists but we can't signal it (it's root) → alive
                    //   ESRCH = no such process → dead
                    let ret = unsafe { libc::kill(pid as i32, 0) };
                    if ret == 0 { return true; }
                    // EPERM means "exists but you're not root" — daemon is alive
                    let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
                    errno == libc::EPERM
                })
                .unwrap_or(false);

            if !daemon_alive {
                tracing::debug!(
                    "Stale tunnel on pod {}: iface={} but daemon is dead — clearing",
                    pod.pod_id, iface
                );
                pod.tunnel_iface = None;
                // Clean up stale PID/iface files
                let _ = std::fs::remove_file(&pid_file);
                let _ = std::fs::remove_file(format!("/tmp/tytus/tunnel-{}.iface", pod.pod_id));
            }
        }
    }
}

/// Append a timestamped line to /tmp/tytus/autostart.log for headless diagnostics.
fn append_autostart_log(msg: &str) {
    use std::io::Write;
    let dir = secure_tytus_tmp_dir();
    let log_path = dir.join("autostart.log");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let _ = writeln!(f, "[{}] {}", ts, msg);
        secure_chmod_600(&log_path);
    }
}

pub(crate) async fn get_credentials(state: &mut CliState, http: &atomek_core::HttpClient) -> (String, String) {
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
                        stable_ai_endpoint: None,
                        stable_user_key: None,
                        gateway_token: None,
                        edge_slug: None,
                        edge_public_url: None,
                        pod_public_url: None,
                    });
                }
            }
        }
    }
}

fn print_json_status(state: &CliState) {
    // SECURITY: Only expose user-facing fields. Never leak infrastructure details
    // (droplet_id, droplet_ip, internal pod IPs, raw per-pod keys).
    // Use `tytus env --raw` for debugging (explicit opt-in).
    let pods: Vec<_> = state.pods.iter().map(|p| {
        serde_json::json!({
            "pod_id": p.pod_id,
            "agent_type": p.agent_type,
            "tunnel_iface": p.tunnel_iface,
            "stable_ai_endpoint": p.stable_ai_endpoint,
            "stable_user_key": p.stable_user_key,
        })
    }).collect();

    let out = serde_json::json!({
        "logged_in": state.is_logged_in(),
        "email": state.email,
        "tier": state.tier,
        "pods": pods,
    });
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
            // SECURITY: Only show stable endpoint (never internal IPs or raw keys)
            if let Some(ref ep) = pod.stable_ai_endpoint {
                println!("  Endpoint:      {}", ep);
            }
            if let Some(ref key) = pod.stable_user_key {
                println!("  API Key:       {}...{}", &key[..15.min(key.len())], &key[key.len().saturating_sub(4)..]);
            }
            if let Some(ref iface) = pod.tunnel_iface {
                println!("  Tunnel:        {}", iface);
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────
// Unit tests for the forwarder's header + cache-key helpers.
// The full integration test (real tunnel + browser) can't run in
// CI; these tests cover the pure logic that has to stay correct
// across every possible pod/IP/port combination.
// ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod forwarder_tests {
    use super::*;

    #[test]
    fn origin_header_rewritten_for_any_upstream() {
        // Matrix: three pods across two droplets, two agent types.
        let scenarios = [
            ("10.18.1.1:3000",  "pod 01 on droplet 18, openclaw"),
            ("10.18.2.1:3000",  "pod 02 on droplet 18, openclaw"),
            ("10.18.2.1:8642",  "pod 02 on droplet 18, hermes"),
            ("10.42.7.1:3000",  "pod 07 on droplet 42, openclaw"),
            ("10.99.99.1:8642", "pod 99 on droplet 99, hermes"),
        ];
        for (upstream, desc) in scenarios {
            let req = format!(
                "GET /ws HTTP/1.1\r\n\
                 Host: localhost:18702\r\n\
                 Origin: http://localhost:18702\r\n\
                 Referer: http://localhost:18702/chat?s=main\r\n\
                 Connection: Upgrade\r\n\
                 Upgrade: websocket\r\n\r\n"
            );
            let rewritten = rewrite_origin_headers(req.as_bytes(), upstream);
            let text = std::str::from_utf8(&rewritten).expect("utf8");
            let expected_origin = format!("Origin: http://{}", upstream);
            let expected_host = format!("Host: {}", upstream);
            let expected_referer = format!("Referer: http://{}/chat?s=main", upstream);
            assert!(text.contains(&expected_origin), "{}: missing {}", desc, expected_origin);
            assert!(text.contains(&expected_host),   "{}: missing {}", desc, expected_host);
            assert!(text.contains(&expected_referer),"{}: missing {}", desc, expected_referer);
            assert!(text.contains("Connection: Upgrade"), "{}: lost Connection header", desc);
            assert!(text.contains("Upgrade: websocket"),  "{}: lost Upgrade header", desc);
        }
    }

    #[test]
    fn origin_rewrite_case_insensitive() {
        let req = b"GET / HTTP/1.1\r\norigin: http://localhost:18702\r\nHOST: localhost:18702\r\n\r\n";
        let rewritten = rewrite_origin_headers(req, "10.18.2.1:3000");
        let text = std::str::from_utf8(&rewritten).unwrap();
        assert!(text.contains("Origin: http://10.18.2.1:3000"));
        assert!(text.contains("Host: 10.18.2.1:3000"));
    }

    #[test]
    fn origin_rewrite_preserves_unrelated_headers() {
        let req = b"GET / HTTP/1.1\r\nAccept: */*\r\nOrigin: http://localhost:18702\r\nCookie: session=abc\r\nUser-Agent: curl\r\n\r\n";
        let rewritten = rewrite_origin_headers(req, "10.18.2.1:3000");
        let text = std::str::from_utf8(&rewritten).unwrap();
        assert!(text.contains("Accept: */*"));
        assert!(text.contains("Cookie: session=abc"));
        assert!(text.contains("User-Agent: curl"));
        assert!(text.contains("Origin: http://10.18.2.1:3000"));
    }

    #[test]
    fn origin_rewrite_noop_when_no_origin_header() {
        let req = b"GET / HTTP/1.1\r\nHost: localhost:18702\r\nAccept: */*\r\n\r\n";
        let rewritten = rewrite_origin_headers(req, "10.18.2.1:3000");
        let text = std::str::from_utf8(&rewritten).unwrap();
        // Host still rewritten, everything else preserved.
        assert!(text.contains("Host: 10.18.2.1:3000"));
        assert!(text.contains("Accept: */*"));
        assert!(!text.contains("Origin:"));
    }

    #[test]
    fn origin_rewrite_handles_forwarder_port_variants() {
        // User might set --port, hit fallback 18800+, or land on ephemeral
        // port. Rewrite just replaces the whole Origin value — source
        // port doesn't matter.
        for forwarder_port in [18701, 18702, 18799, 18802, 19102, 49213, 3000] {
            let req = format!(
                "GET / HTTP/1.1\r\nOrigin: http://localhost:{}\r\n\r\n",
                forwarder_port
            );
            let rewritten = rewrite_origin_headers(req.as_bytes(), "10.18.2.1:3000");
            let text = std::str::from_utf8(&rewritten).unwrap();
            assert!(text.contains("Origin: http://10.18.2.1:3000"), "failed for fp={}", forwarder_port);
            assert!(!text.contains(&format!("localhost:{}", forwarder_port)),
                "leaked source port {} in rewritten request", forwarder_port);
        }
    }

    #[test]
    fn cache_key_is_deterministic_and_safe() {
        assert_eq!(cache_key_for("/assets/index-Dts6VHgr.js"), "_assets_index-Dts6VHgr.js");
        assert_eq!(cache_key_for("/assets/index-Dts6VHgr.js?v=2"), "_assets_index-Dts6VHgr.js");
        assert_eq!(cache_key_for("/deep/path/with-hash-1A2B.css"), "_deep_path_with-hash-1A2B.css");
        // No /, no weird chars
        assert!(!cache_key_for("/anything").contains('/'));
    }

    #[test]
    fn is_cacheable_asset_accepts_expected_extensions() {
        for p in ["/x.js", "/x.mjs", "/x.css", "/x.svg", "/x.png", "/x.woff2", "/x.wasm"] {
            assert!(is_cacheable_asset(p), "should cache {}", p);
        }
        for p in ["/api/foo", "/data.json", "/", "/chat", "/__openclaw/control-ui-config.json"] {
            assert!(!is_cacheable_asset(p), "should NOT cache {}", p);
        }
    }

    #[test]
    fn response_2xx_detector() {
        assert!(response_is_2xx(b"HTTP/1.1 200 OK\r\n\r\n"));
        assert!(response_is_2xx(b"HTTP/1.1 204 No Content\r\n\r\n"));
        assert!(response_is_2xx(b"HTTP/1.0 201 Created\r\n\r\n"));
        assert!(!response_is_2xx(b"HTTP/1.1 404 Not Found\r\n\r\n"));
        assert!(!response_is_2xx(b"HTTP/1.1 500 Internal Server Error\r\n\r\n"));
        assert!(!response_is_2xx(b""));
        assert!(!response_is_2xx(b"garbage"));
    }
}

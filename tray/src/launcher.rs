//! AI CLI detection and terminal launcher.
//!
//! Detects installed AI CLIs on PATH and launches them in a new terminal
//! window with Tytus pod environment variables pre-configured.
//! Before launching, runs `tytus link --only <filter>` to inject the right
//! documentation, MCP configs, and slash commands for that CLI.

use std::process::Command;

/// An AI CLI that can be launched with Tytus pod connection.
#[derive(Debug, Clone)]
pub struct AiCli {
    /// Menu display name
    pub name: &'static str,
    /// Binary name on PATH
    pub binary: &'static str,
    /// Command to run (may differ from binary)
    pub command: &'static str,
    /// The `--only` filter for `tytus link` (which integration files to inject)
    pub link_filter: &'static str,
}

/// All known AI CLIs we can detect and launch.
const KNOWN_CLIS: &[AiCli] = &[
    AiCli {
        name: "Claude Code",
        binary: "claude",
        command: "claude",
        link_filter: "claude",
    },
    AiCli {
        name: "OpenCode",
        binary: "opencode",
        command: "opencode",
        link_filter: "opencode",
    },
    AiCli {
        name: "Gemini CLI",
        binary: "gemini",
        command: "gemini",
        link_filter: "agents",
    },
    AiCli {
        name: "Codex",
        binary: "codex",
        command: "codex",
        link_filter: "agents",
    },
    AiCli {
        name: "Aider",
        binary: "aider",
        command: "aider --model openai/ail-compound",
        link_filter: "shell",
    },
    AiCli {
        name: "Cursor",
        binary: "cursor",
        command: "cursor .",
        link_filter: "claude",
    },
    AiCli {
        name: "Vibe",
        binary: "vibe",
        command: "vibe",
        link_filter: "agents",
    },
    AiCli {
        name: "Cody",
        binary: "cody",
        command: "cody",
        link_filter: "agents",
    },
    AiCli {
        name: "Amp",
        binary: "amp",
        command: "amp",
        link_filter: "agents",
    },
];

/// Detect which AI CLIs are installed on the system.
pub fn detect_installed_clis() -> Vec<AiCli> {
    KNOWN_CLIS
        .iter()
        .filter(|cli| is_on_path(cli.binary))
        .cloned()
        .collect()
}

fn is_on_path(binary: &str) -> bool {
    Command::new("which")
        .arg(binary)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Connection info needed to configure env vars for a launched CLI.
#[derive(Debug, Clone)]
pub struct PodConnection {
    pub ai_gateway: String,
    pub api_key: String,
    pub model: String,
}

/// Launch an AI CLI in a new terminal window with Tytus env vars.
/// First injects the right integration files via `tytus link`, then opens
/// the CLI in a new terminal window with pod env vars pre-set.
pub fn launch_in_terminal(cli: &AiCli, conn: &PodConnection) {
    // The shell command that will run in the new terminal window.
    // Steps:
    //   1. cd to the user's home (safe default working directory)
    //   2. Set OpenAI-compatible env vars so the CLI talks through Tytus
    //   3. Run `tytus link . --only <filter>` to inject docs/MCP/commands
    //   4. Show a banner so the user knows what happened
    //   5. Launch the CLI
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".into());
    let shell_cmd = format!(
        concat!(
            "cd '{}' && ",
            "export OPENAI_API_KEY='{}' ",
            "OPENAI_BASE_URL='{}/v1' ",
            "OPENAI_API_BASE='{}/v1' ",
            "AI_GATEWAY='{}' && ",
            "tytus link . --only {} >/dev/null 2>&1 ; ",
            "echo '' && ",
            "echo '  \\033[36m🦞 Tytus pod connected\\033[0m' && ",
            "echo '  \\033[2mGateway: {} | Model: {} | Key: ...{}\\033[0m' && ",
            "echo '' && ",
            "{}"
        ),
        home,
        conn.api_key,
        conn.ai_gateway,
        conn.ai_gateway,
        conn.ai_gateway,
        cli.link_filter,
        conn.ai_gateway,
        conn.model,
        // Last 8 chars of API key for identification
        if conn.api_key.len() > 8 {
            &conn.api_key[conn.api_key.len() - 8..]
        } else {
            &conn.api_key
        },
        cli.command,
    );

    open_in_terminal(&shell_cmd);
}

/// Open a plain terminal with Tytus env vars set.
pub fn launch_terminal(conn: &PodConnection) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".into());
    let shell_cmd = format!(
        concat!(
            "cd '{}' && ",
            "export OPENAI_API_KEY='{}' ",
            "OPENAI_BASE_URL='{}/v1' ",
            "OPENAI_API_BASE='{}/v1' ",
            "AI_GATEWAY='{}' && ",
            "tytus link . --only shell >/dev/null 2>&1 ; ",
            "echo '' && ",
            "echo '  \\033[36m🦞 Tytus pod connected\\033[0m' && ",
            "echo '  \\033[2mGateway: {} | Model: ail-compound\\033[0m' && ",
            "echo '  \\033[2mRun: curl $AI_GATEWAY/v1/chat/completions -H \"Authorization: Bearer $OPENAI_API_KEY\" ...\\033[0m' && ",
            "echo '' && ",
            "exec $SHELL"
        ),
        home,
        conn.api_key,
        conn.ai_gateway,
        conn.ai_gateway,
        conn.ai_gateway,
        conn.ai_gateway,
    );

    open_in_terminal(&shell_cmd);
}

/// Open a command in a new terminal window.
fn open_in_terminal(shell_command: &str) {
    if let Err(err) = atomek_core::platform::terminal::open_shell_command(shell_command) {
        eprintln!("[tray] Failed to open terminal: {err}");
    }
}

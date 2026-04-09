# Tytus CLI

Connect to your **private AI pod** from any terminal. Tytus provides a WireGuard-encrypted tunnel to your own AI infrastructure with an OpenAI-compatible gateway running 383+ models.

```bash
tytus login                    # One-time browser auth
sudo tytus connect             # Allocate pod + activate tunnel
eval $(tytus env --export)     # Export connection vars
curl $TYTUS_AI_GATEWAY/v1/models -H "Authorization: Bearer $TYTUS_API_KEY"
```

## Installation

### Quick install (macOS / Linux)

```bash
curl -fsSL https://tytus.traylinx.com/install.sh | sh
```

Installs two binaries:
- `tytus` — CLI for pod management
- `tytus-mcp` — MCP server for AI CLI integration

### From GitHub Releases

Download from [Releases](https://github.com/traylinx/tytus-cli/releases):

| Platform | Asset |
|----------|-------|
| macOS (Apple Silicon) | `tytus-macos-aarch64.tar.gz` |
| macOS (Intel) | `tytus-macos-x86_64.tar.gz` |
| Linux (x86_64) | `tytus-linux-x86_64.tar.gz` |

```bash
tar xzf tytus-macos-aarch64.tar.gz
sudo mv tytus tytus-mcp /usr/local/bin/
```

### From source

```bash
git clone https://github.com/traylinx/tytus-cli.git
cd tytus-cli
cargo build --release -p atomek-cli -p tytus-mcp
sudo cp target/release/tytus target/release/tytus-mcp /usr/local/bin/
```

## Quick Start

```bash
# 1. Login (opens browser, one-time)
tytus login

# 2. Connect (allocates pod + WireGuard tunnel)
sudo tytus connect

# 3. Use your private AI
eval $(tytus env --export)
curl "$TYTUS_AI_GATEWAY/v1/chat/completions" \
  -H "Authorization: Bearer $TYTUS_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"qwen3-8b","messages":[{"role":"user","content":"hello"}]}'
```

## AI CLI Integration (The Zombie Fungus)

Tytus is designed to **parasitize any AI CLI** — Claude Code, Kilocode, OpenCode, Archon, Codex, Gemini CLI, or anything that speaks MCP or reads env vars. One command infects a project with all the integration files needed.

### One-command setup for any project

```bash
cd your-project
tytus infect
```

This drops integration files for **every major AI CLI**:

| File | Purpose | Used by |
|------|---------|---------|
| `.mcp.json` | MCP server config (native tool access) | Claude Code, Kilocode |
| `CLAUDE.md` (appended) | Context + instructions | Claude Code |
| `AGENTS.md` (appended) | Context + instructions | Codex, Gemini CLI, generic agents |
| `.claude/commands/tytus.md` | `/tytus` slash command | Claude Code |
| `.kilo/command/tytus.md` | `/tytus` command | Kilocode, OpenCode |
| `.kilo/mcp.json` | MCP config | Kilocode |
| `.archon/commands/tytus.md` | Tytus command | Archon |
| `.tytus-env.sh` | Shell env loader | Any terminal |

Selective injection:
```bash
tytus infect --only claude         # Only Claude Code files
tytus infect --only agents,shell   # AGENTS.md + shell hook
tytus infect --only kilocode       # Kilocode/OpenCode files
```

### MCP Server (deepest integration)

The `tytus-mcp` binary is a stdio-based [MCP](https://modelcontextprotocol.io/) server. Any MCP-compatible AI CLI gets **native tool access** to your pod:

| Tool | Description |
|------|-------------|
| `tytus_status` | Login state, plan tier, active pods |
| `tytus_env` | Connection URLs, API keys, OpenAI-compat aliases |
| `tytus_models` | List 383+ models on the pod |
| `tytus_chat` | Chat completions through the private gateway |
| `tytus_revoke` | Release a pod and free units |
| `tytus_setup_guide` | Step-by-step setup instructions |

Print MCP config for your CLI:
```bash
tytus mcp                      # Claude Code format
tytus mcp --format kilocode    # Kilocode format
tytus mcp --format archon      # Archon format
tytus mcp --format json        # Generic JSON
```

Manual config (Claude Code `~/.claude/settings.json`):
```json
{
  "mcpServers": {
    "tytus": {
      "command": "/usr/local/bin/tytus-mcp",
      "args": [],
      "alwaysAllow": ["tytus_status", "tytus_env", "tytus_models", "tytus_setup_guide"]
    }
  }
}
```

### Environment Variables (universal)

Works with anything that reads `OPENAI_API_KEY`:

```bash
eval $(tytus env --export)
export OPENAI_API_KEY=$TYTUS_API_KEY
export OPENAI_BASE_URL=${TYTUS_AI_GATEWAY}/v1
```

Or source the hook file:
```bash
source .tytus-env.sh
```

### Programmatic (JSON mode)

Every command supports `--json`:

```bash
tytus status --json | jq .tier
tytus env --json | jq -r .ai_endpoint
sudo tytus connect --json 2>/dev/null | jq .pod_id
```

## Commands

| Command | Description | Sudo |
|---------|-------------|------|
| `tytus login` | Browser-based device auth | No |
| `tytus status` | Plan, pods, tunnel state | No |
| `sudo tytus connect` | Allocate pod + tunnel (blocks until Ctrl+C) | Yes |
| `tytus disconnect` | Clear stale tunnel state | No |
| `tytus revoke <pod>` | Release pod, free units | No |
| `tytus logout` | Revoke all + clear auth | No |
| `tytus env` | Connection info (shell vars) | No |
| `tytus infect [dir]` | Inject integration files | No |
| `tytus mcp` | Print MCP server config | No |

### `tytus connect` options

```bash
sudo tytus connect                     # OpenClaw agent (1 unit)
sudo tytus connect --agent hermes      # Hermes agent (2 units)
sudo tytus connect --pod 02            # Reconnect existing pod
sudo tytus connect --json              # JSON output
```

### `tytus env` options

```bash
tytus env                # KEY=VALUE
tytus env --export       # export KEY=VALUE (source-able)
tytus env --json         # Full JSON
tytus env --pod 02       # Specific pod
```

| Variable | Example | Description |
|----------|---------|-------------|
| `TYTUS_AI_GATEWAY` | `http://10.18.1.1:18080` | OpenAI-compatible gateway |
| `TYTUS_AGENT_API` | `http://10.18.1.1:3000` | Agent API endpoint |
| `TYTUS_API_KEY` | `sk-566cecd...09a0` | Bearer token |
| `TYTUS_AGENT_TYPE` | `nemoclaw` | Agent type |
| `TYTUS_POD_ID` | `01` | Pod identifier |

## Agent Types

| Agent | Units | Port | Use Case |
|-------|-------|------|----------|
| **OpenClaw** (`nemoclaw`) | 1 | 3000 | Lightweight sandboxed agent. Fast startup. |
| **Hermes** (`hermes`) | 2 | 8642 | Full-featured. 60+ tools, self-improving. |

Plan budgets: Explorer=1, Creator=2, Operator=4 units.

## Architecture

```
Your Terminal ──> tytus CLI ──> WireGuard Tunnel ──> Private Droplet
                                (boringtun)              |
                                                   +-----+-----+
                                                   | SwitchAI   | <-- 383 models
                                                   | Local      |   (Qwen, Llama, etc.)
                                                   | :18080     |
                                                   +------------+
                                                   | Agent      | <-- OpenClaw or Hermes
                                                   | Container  |
                                                   | :3000/8642 |
                                                   +------------+

AI CLIs ──> tytus-mcp ──> reads state.json ──> exposes MCP tools
            (stdio)       (no network needed)    to the AI agent
```

Crate structure:

| Crate | Purpose |
|-------|---------|
| `cli` | Binary: `tytus` command |
| `mcp` | Binary: `tytus-mcp` MCP server |
| `core` | HTTP client (retry/backoff), error types |
| `auth` | Device auth (Sentinel), keychain, token refresh |
| `pods` | Provider API: allocate, status, config, revoke |
| `tunnel` | WireGuard tunnel via boringtun |

## Security

- State file: `~/.config/tytus/state.json` (owner-only, 0600)
- Refresh tokens: OS keychain (`com.traylinx.atomek`)
- WireGuard keys: zeroed on drop (Zeroize trait), never on disk
- Config: parsed in memory, never written to disk
- Pod isolation: separate subnet per pod, iptables blocks cross-pod

## Troubleshooting

| Problem | Solution |
|---------|----------|
| "TUN device requires root" | `sudo tytus connect` |
| "No Tytus subscription" | Upgrade at traylinx.com |
| "Config download failed" | Pod provisioning. Wait, then `sudo tytus connect --pod XX` |
| "Token refresh failed" | `tytus logout && tytus login` |
| Debug logging | `RUST_LOG=debug sudo tytus connect` |

## Development

```bash
cargo build -p atomek-cli -p tytus-mcp   # Debug build
cargo build --release -p atomek-cli       # Release CLI only
cargo test --all                          # Tests
cargo clippy --all                        # Lint
```

## License

MIT - Traylinx

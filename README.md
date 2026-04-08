# Tytus CLI

Connect to your **private AI pod** from any terminal. Tytus provides a WireGuard-encrypted tunnel to your own AI infrastructure with an OpenAI-compatible gateway running 383+ models.

```bash
tytus login                    # One-time browser auth
sudo tytus connect             # Allocate pod + activate tunnel
eval $(tytus env --export)     # Export connection vars
curl $TYTUS_AI_GATEWAY/v1/models -H "Authorization: Bearer $TYTUS_API_KEY"
```

## Installation

### From source (requires Rust toolchain)

```bash
git clone https://github.com/traylinx/tytus-cli.git
cd tytus-cli
cargo build --release -p atomek-cli
sudo cp target/release/tytus /usr/local/bin/
```

### Verify

```bash
tytus --version
tytus --help
```

## Quick Start

```bash
# 1. Login (opens browser, one-time)
tytus login

# 2. Connect (allocates pod + WireGuard tunnel)
sudo tytus connect

# 3. Use your private AI
curl http://10.18.1.1:18080/v1/chat/completions \
  -H "Authorization: Bearer $(tytus env --json | jq -r .pod_api_key)" \
  -H "Content-Type: application/json" \
  -d '{"model":"qwen3-8b","messages":[{"role":"user","content":"hello"}]}'
```

## Commands

### `tytus login`

Authenticate with Traylinx via device auth (opens browser).

```bash
tytus login          # Interactive — opens browser
tytus login --json   # JSON output for scripts
```

If you have a stored session, auto-refreshes without opening the browser.

### `tytus status`

Show current plan, pods, and tunnel state.

```bash
tytus status
tytus status --json
```

Example output:
```
Tytus — sebastian@example.com
Plan: operator
 
Pod 02 [nemoclaw] connected
  AI Gateway:    http://10.18.2.1:18080
  Agent API:     http://10.18.2.1:3000
  API Key:       sk-c939e2...2318
  Tunnel:        utun4
```

### `sudo tytus connect`

Allocate a new pod and activate the WireGuard tunnel. Requires `sudo` for TUN device creation.

```bash
sudo tytus connect                     # Default: OpenClaw agent (1 unit)
sudo tytus connect --agent hermes      # Hermes agent (2 units)
sudo tytus connect --pod 02            # Reconnect existing pod
sudo tytus connect --json              # JSON output (pod info to stdout)
```

The tunnel runs until you press **Ctrl+C**. Connection info is printed on start:

```
AI_GATEWAY=http://10.18.1.1:18080
AGENT_API=http://10.18.1.1:3000
API_KEY=sk-566cecd...09a0

Tunnel running. Press Ctrl+C to disconnect.
```

### `tytus env`

Print connection info for the current pod. Designed for shell integration.

```bash
tytus env                # KEY=VALUE format
tytus env --export       # export KEY=VALUE (source-able)
tytus env --json         # Full pod entry as JSON
tytus env --pod 02       # Specific pod
```

Shell integration:
```bash
eval $(tytus env --export)
# Now use $TYTUS_AI_GATEWAY, $TYTUS_API_KEY, etc.
```

Environment variables exported:
| Variable | Example | Description |
|----------|---------|-------------|
| `TYTUS_AI_GATEWAY` | `http://10.18.1.1:18080` | OpenAI-compatible LLM gateway |
| `TYTUS_AGENT_API` | `http://10.18.1.1:3000` | Agent API endpoint |
| `TYTUS_API_KEY` | `sk-566cecd...09a0` | Bearer token for the gateway |
| `TYTUS_AGENT_TYPE` | `nemoclaw` | Agent running on the pod |
| `TYTUS_POD_ID` | `01` | Pod identifier |

### `tytus revoke <pod_id>`

Release a pod and free its units back to your account.

```bash
tytus revoke 02
```

### `tytus disconnect`

Clear local tunnel state (the actual tunnel is stopped via Ctrl+C in `connect`).

```bash
tytus disconnect           # All pods
tytus disconnect --pod 02  # Specific pod
```

### `tytus logout`

Revoke all pods and clear all local auth state.

```bash
tytus logout
```

## Agent Types

| Agent | Units | Port | Use Case |
|-------|-------|------|----------|
| **OpenClaw** (`nemoclaw`) | 1 | 3000 | Lightweight sandboxed agent. Fast startup, minimal tools. |
| **Hermes** (`hermes`) | 2 | 8642 | Full-featured agent. 60+ tools, self-improving, multi-platform. |

Unit budgets per plan:
- **Explorer**: 1 unit
- **Creator**: 2 units
- **Operator**: 4 units

## Integration with AI CLIs

### Any OpenAI-compatible tool

```bash
# Start tunnel in background terminal
sudo tytus connect

# In your working terminal
eval $(tytus env --export)
export OPENAI_API_KEY=$TYTUS_API_KEY
export OPENAI_BASE_URL=${TYTUS_AI_GATEWAY}/v1
```

Now any tool reading `OPENAI_API_KEY` and `OPENAI_BASE_URL` routes through your private pod.

### Claude Code

Add to your project's `CLAUDE.md`:
```markdown
## AI Gateway
Run `eval $(tytus env --export)` to load Tytus connection.
AI Gateway: $TYTUS_AI_GATEWAY (OpenAI-compatible, 383+ models)
API Key: $TYTUS_API_KEY
```

### Programmatic (JSON mode)

```bash
# Get pod info as JSON
tytus env --json | jq .

# Use in scripts
API_KEY=$(tytus env --json | jq -r .pod_api_key)
GATEWAY=$(tytus env --json | jq -r .ai_endpoint)
```

## Architecture

```
Your Terminal ──→ tytus CLI ──→ WireGuard Tunnel ──→ Private Droplet
                                (boringtun)              │
                                                   ┌─────┴─────┐
                                                   │ SwitchAI   │ ← 383 models
                                                   │ Local      │   (Qwen, Llama, etc.)
                                                   │ :18080     │
                                                   ├────────────┤
                                                   │ Agent      │ ← OpenClaw or Hermes
                                                   │ Container  │
                                                   │ :3000/8642 │
                                                   └────────────┘
```

- **WireGuard tunnel**: Userspace via [boringtun](https://github.com/cloudflare/boringtun) — no kernel module, no `wg-quick`, no config files on disk
- **Encryption**: Noise protocol (Curve25519, ChaCha20-Poly1305)
- **Keys**: Zeroed on drop (Zeroize trait), never written to disk
- **Pod isolation**: Each pod has its own subnet, iptables blocks cross-pod traffic

## State & Security

**State file**: `~/.config/tytus/state.json` (permissions `0600`)

**Keychain**: Refresh tokens also stored in OS keychain (`com.traylinx.atomek`) for cross-tool compatibility.

**Security properties**:
- State file owner-only read/write
- WireGuard keys zeroed on drop
- Config never written to disk (parsed in memory)
- Tokens never logged or printed to stdout (except `env` command)
- A2A credentials in headers, not query parameters

## Troubleshooting

### "TUN device requires root"
```bash
sudo tytus connect   # TUN creation needs root privileges
```

### "No Tytus subscription"
Visit [traylinx.com](https://traylinx.com) to upgrade your plan.

### "Config download failed"
The pod may still be provisioning. Wait a few seconds and retry:
```bash
sudo tytus connect --pod 02
```

### "Token refresh failed"
```bash
tytus logout
tytus login    # Fresh login
```

### Debug logging
```bash
RUST_LOG=debug sudo tytus connect
RUST_LOG=trace sudo tytus connect   # Very verbose
```

## Development

```bash
cargo build -p atomek-cli           # Debug build
cargo build --release -p atomek-cli # Release build
cargo test --all                    # Run tests
cargo clippy --all                  # Lint
```

### Crate structure

| Crate | Purpose |
|-------|---------|
| `cli` | Binary — the `tytus` command |
| `core` | HTTP client (retry/backoff), error types, token state |
| `auth` | Device auth (Sentinel), keychain, token refresh |
| `pods` | Provider API: allocate, status, config download, revoke |
| `tunnel` | WireGuard tunnel via boringtun (async, cross-platform) |

## License

MIT - Traylinx

# tytus-cli

> CLI for **Tytus** by Traylinx — your private AI pod, driven from any terminal.

`tytus` is a Rust CLI that opens a userspace WireGuard tunnel from your laptop
to your private Tytus pod and exposes its OpenAI-compatible LLM gateway through
a stable URL + stable API key. The pair you paste into Cursor / Claude Desktop /
OpenCode / any OpenAI-compatible tool **never changes** — even if your pod gets
rotated, your droplet migrates, or you switch agent runtimes.

```bash
curl -fsSL https://get.traylinx.com/install.sh | bash
tytus login && tytus connect
```

That's it. `tytus login` provisions a **default pod** (agent-less, zero
plan units) automatically, and `tytus connect` brings the WG tunnel up to
it. AIL gateway access at `http://10.42.42.1:18080` works immediately —
no agent install required, no plan units spent.

```bash
eval "$(tytus env --export)"
echo $OPENAI_BASE_URL    # http://10.42.42.1:18080/v1   (constant forever)
echo $OPENAI_API_KEY     # sk-tytus-user-<32hex>          (per user, persistent)
```

Need a specific agent runtime (NemoClaw, Hermes) on top? Install one
separately — pod allocation and agent deployment are decoupled:

```bash
tytus agent catalog              # list installable agents
tytus agent install nemoclaw     # allocate a pod slot + deploy NemoClaw
tytus agent list                 # see what's running where
```

Or `tytus setup` walks you through login + default pod + agent picker in
one interactive wizard.

---

## What is Tytus?

Tytus is a **private AI pod** product. Each Traylinx subscriber gets their own
isolated slice of a droplet — a WireGuard sidecar plus a containerised AI
agent (OpenClaw or Hermes) — and an OpenAI-compatible LLM gateway
(`SwitchAILocal`) that proxies to upstream providers.

```
your laptop ── WireGuard tunnel ── pod sidecar ── agent container
                                       └── SwitchAILocal (OpenAI-compatible)
                                             └── upstream LLM (MiniMax)
```

**No customer LLM traffic ever traverses Traylinx Cloud.** Prompts and
responses go directly between your laptop and your pod over WireGuard. The
Traylinx control plane (auth, billing, allocation) only sees that you have
a pod — never the contents of your conversations.

---

## Install (early access)

```bash
curl -fsSL https://get.traylinx.com/install.sh | bash
```

> **Early access.** Tytus is under active development. The installer builds
> from source against `main` so every fix reaches you immediately. Prebuilt
> binaries and `brew install traylinx/tap/tytus` will return once the CLI is
> stable. Requires a Rust toolchain (the installer can install it for you).

What the installer does:

1. Detects your OS and architecture (macOS / Linux, x86_64 / aarch64)
2. Ensures a Rust toolchain is present — offers to install rustup if missing
3. Builds `tytus` and `tytus-mcp` from the `main` branch via
   `cargo install --git` (~3 minutes first build)
4. Sets up a tightly-scoped passwordless sudoers entry so `tytus connect`
   never prompts you for a password (opt-out with `TYTUS_SKIP_SUDOERS=1`)
5. Verifies and prints next steps

Override the install location with `TYTUS_INSTALL_DIR=/opt/tytus/bin` if you
want it somewhere other than `~/.cargo/bin`.

If you already have a release from before we switched back to source-only,
you can force that path with `TYTUS_USE_RELEASE=1` — but you'll miss any
fixes that landed on `main` since that tag.

### From source

```bash
git clone https://github.com/traylinx/tytus-cli.git
cd tytus-cli
cargo install --path cli --bin tytus --bin tytus-mcp
```

---

## Quick start

```bash
# 1. Interactive first-run (recommended)
tytus setup

# 2. Or manually
tytus login                  # browser device-auth via Sentinel
tytus connect                # allocate a pod + open WG tunnel
tytus test                   # E2E health check
tytus chat                   # REPL against your private pod
```

After connecting, use the stable env in any tool:

```bash
eval "$(tytus env --export)"
curl -sS "$OPENAI_BASE_URL/chat/completions" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"ail-compound","messages":[{"role":"user","content":"hello"}]}'
```

---

## Plans and agent types

Each subscription tier has a fixed **unit budget**. Agents cost units when
allocated:

| Plan | Unit budget |
|---|---|
| Explorer | 1 |
| Creator | 2 |
| Operator | 4 |

| Agent | Cost | Gateway port | Description |
|---|---|---|---|
| `nemoclaw` | 1 unit | 3000 | OpenClaw runtime with the NemoClaw sandboxing blueprint |
| `hermes` | 2 units | 8642 | Nous Research Hermes gateway |

You can mix and match within your budget. For example, an Operator user
can run 4 nemoclaws, or 2 hermes, or 2 nemoclaw + 1 hermes.

```bash
tytus connect --agent nemoclaw    # default — 1 unit
tytus connect --agent hermes      # 2 units
```

## Models on the pod gateway

| Model id | Backed by | Capabilities |
|---|---|---|
| `ail-compound` | MiniMax M2.7 | text, vision, audio (default chat model) |
| `minimax/ail-compound` | MiniMax M2.7 | text |
| `ail-image` | MiniMax image-01 | image generation |
| `minimax/ail-image` | MiniMax image-01 | image generation |
| `ail-embed` | mistral-embed via SwitchAI | embeddings |

Pass any of these as the `model` field in OpenAI-compatible requests. Other
model ids (`gpt-4`, `claude-*`, etc.) are not available on this product.

---

## Command reference

```text
tytus login                          Browser device-auth via Sentinel
tytus logout                         Revoke all pods + clear local state
tytus status [--json]                Plan, pods, units, tunnel state
tytus doctor                         Full diagnostic
tytus setup                          Interactive first-run wizard

tytus connect [--agent T] [--pod NN] Allocate pod + activate tunnel
tytus disconnect [--pod NN]          Tear down tunnel, keep allocation
tytus revoke <pod_id>                Free units (DESTRUCTIVE — wipes state)
tytus restart [--pod NN]             Restart agent container

tytus env [--export] [--raw]         Connection vars (stable by default)
tytus test                           E2E health check
tytus chat [--model ail-compound]    Interactive REPL
tytus exec [--pod NN] "<command>"    Run shell command in agent container
tytus configure                      Interactive overlay editor

tytus push <LOCAL> [--pod NN] [--to ...]    Push file/dir to pod (see docs/file-sharing.md)
tytus pull <REMOTE> [--pod NN] [--to ...]   Pull file/dir from pod
tytus ls   [PATH] [--pod NN]                List pod files under /app/workspace/
tytus rm   <REMOTE> [--pod NN] [--recursive]  Delete pod files (refuses outside /app/workspace/)
tytus transfers [--tail N] [--pod NN]       Show local JSONL audit log

tytus link [DIR] [--only ...]        Link a project so AI CLIs in it know Tytus
tytus mcp [--format ...]             Print MCP server config for an AI tool
tytus bootstrap-prompt               Print the paste prompt for any AI tool
tytus llm-docs                       Print the full LLM-facing reference

tytus daemon run|stop|status         Background token-refresh daemon
tytus autostart install|uninstall    LaunchAgent (macOS) / systemd (Linux) for
                                     tunnel + daemon — keeps your pod alive
                                     24/7 with auto-refreshing credentials
tytus tray install|uninstall|status  macOS only — installs /Applications/Tytus.app
                                     (Spotlight-discoverable menu-bar app) plus
                                     a launch-at-login agent, so you can quit
                                     the tray anytime and it comes back on reboot
```

Run `tytus <command> --help` for per-command details.

---

## File sharing

Move files and folders between your Mac and any pod:

```bash
tytus push ~/report.pdf --pod 02            # Mac → pod inbox
tytus pull /app/workspace/out/result.md --pod 02   # pod → Mac
tytus ls --pod 02                           # list pod files
tytus rm /app/workspace/inbox/old --pod 02 --recursive   # delete
```

Default destination is `/app/workspace/inbox/` (auto-created).
Directories are tarred + gzipped transparently. The 100 MB ceiling is
intentional — bulk transfers are the job of the v0.7 Garage sprint.

Full reference: [`docs/file-sharing.md`](docs/file-sharing.md). Drives
the tray "Files ▸" submenu per pod and the portable
`skill-tytus-files` skill (human-language triggers in EN + ES on every
infected AI CLI).

---

## Menu-bar app (macOS)

Once-only setup for a Docker-Desktop-style experience:

```bash
tytus tray install    # creates /Applications/Tytus.app + launches it now
tytus login           # browser device-auth (once, then daemon keeps it fresh)
tytus autostart install   # tunnel + token-refresh daemon at every login
```

You now have a colored T in the menu bar with:

| Dot | Meaning |
|---|---|
| 🟢 Connected | Pod reachable, tokens valid, daemon refreshing |
| 🟡 Connected — daemon offline | Pod reachable but no background refresh (RT will die in ~24h) |
| 🟡 Connected — token expired | Tunnel up; daemon will refresh on next tick |
| 🟡 Not logged in / Pod unreachable | Credentials present but tunnel down → click **Connect** |
| 🔴 Not logged in | No credentials at all → click **Sign In…** |

The dot is driven by a **live HTTP probe** to the stable pod endpoint
`http://10.42.42.1:18080` — not by daemon or state-file inspection, so it
reflects ground truth even if the daemon is stopped or the kernel renumbered
the tunnel interface.

Menu actions: Connect/Disconnect, Open in ▸ (Claude Code, OpenCode, Gemini,
Codex, Cursor, Aider, Vibe, Cody, Amp, or Terminal), Copy Connection Info,
Run Health Test, Configure Agent, Sign Out, Doctor, View Daemon/Startup Log,
Start/Stop/Restart Daemon, Auto-start toggles, Documentation, About, Quit.

Most non-interactive actions (`Run Health Test`, `Doctor`, per-pod
`Restart` / `Uninstall` / `Revoke` / `Stop forwarder`, `Channels catalog`,
`Add channel`) now run **in the Tower web UI** instead of opening a
Terminal window. The tray menu deep-links the browser at
`http://127.0.0.1:<port>/tower#/<action>`; output streams in-page via
SSE. Sudo / browser-auth / interactive-wizard commands (`Connect`,
`Sign In`, `Configure Agent`, `tytus tray install`, editor launches)
still spawn a Terminal because they need a TTY.

Each pod has its own subpage at `#/pod/<NN>` with three tabs:
**Overview** (URL strip + per-pod actions), **Output** (live log of
the latest streamed action — Restart / Stop forwarder / Uninstall /
Revoke), and **Channels** (add / remove the messengers each pod can
talk through). A purple dot appears next to a pod's row on the Tower
overview while a streamed action is running on it. Adding a channel
opens a native `<dialog>` in the page for the bot token — no more
Terminal `read -rs` prompt.

---

## Native AI tool integration

Tytus is designed so that **any AI CLI on your laptop** can drive it. Two
patterns are supported.

### Pattern A — Hosted skill file (zero config)

Copy this prompt into Claude Code, OpenCode, Cursor, KiloCode, or any AI
tool that can read URLs:

```bash
tytus bootstrap-prompt
```

Output:

```
Read https://raw.githubusercontent.com/traylinx/tytus-cli/main/.agents/skills/tytus/SKILL.md
and follow the instructions to drive Tytus natively. ...
```

Paste it once. The agent fetches the hosted skill file and learns the full
command surface, the model catalog, the stable URL/key model, the recipes,
and the error catalog. Then it can drive Tytus end-to-end on its own.

### Pattern B — Per-project linking

If you want the integration files dropped directly into a project (so the
AI tool sees them without a URL fetch), run:

```bash
cd your-project
tytus link .
```

This drops:

| File | Used by |
|---|---|
| `CLAUDE.md` (appended) | Claude Code |
| `AGENTS.md` (appended) | OpenCode, Codex, Gemini CLI, generic agents |
| `.claude/commands/tytus.md` | Claude Code `/tytus` slash command |
| `.kilo/command/tytus.md` | KiloCode / OpenCode `/tytus` command |
| `.kilo/mcp.json` | KiloCode MCP config |
| `.archon/commands/tytus.md` | Archon `/tytus` command |
| `.mcp.json` | Claude Code MCP config (auto-allows safe tools) |
| `.tytus-env.sh` | Shell hook (`source .tytus-env.sh`) |

Filter what gets dropped:

```bash
tytus link . --only claude          # only Claude Code files
tytus link . --only kilocode,shell  # KiloCode + shell hook
```

### MCP server (deepest integration)

`tytus-mcp` is a stdio-based [MCP](https://modelcontextprotocol.io/) server
that exposes Tytus to any MCP-compatible AI tool as native tools:

| Tool | Purpose |
|---|---|
| `tytus_docs` | Returns the full LLM-facing reference (call this first) |
| `tytus_status` | Login state, plan, pods, tunnel — call this second |
| `tytus_env` | Stable + raw connection details |
| `tytus_models` | Live model list from the pod gateway |
| `tytus_chat` | Send chat completions through the user's pod |
| `tytus_revoke` | Free a pod's units (destructive) |
| `tytus_setup_guide` | What to tell the user when nothing is connected |

Print the MCP config block for your tool:

```bash
tytus mcp                       # Claude Code format
tytus mcp --format kilocode     # KiloCode / OpenCode
tytus mcp --format archon       # Archon
tytus mcp --format json         # generic JSON
```

---

## Architecture

```
crates/
├── cli      Binary: `tytus` command
├── mcp      Binary: `tytus-mcp` MCP server
├── core     HTTP client (retry/backoff), error types, device fingerprint
├── auth     Sentinel device auth, OS keychain, token refresh
├── pods     Provider API: allocation, status, config, agent control
└── tunnel   WireGuard via boringtun (userspace, cross-platform)
```

The tunnel uses [`boringtun`](https://github.com/cloudflare/boringtun) for
the Noise protocol and the [`tun`](https://crates.io/crates/tun) crate for
the OS-level TUN device. No `wg-quick`, no kernel module. Privilege
escalation for opening the TUN device is handled transparently via a
three-strategy chain: `sudo -n` (passwordless via the sudoers entry the
installer adds) → `osascript` (macOS GUI dialog) → interactive `sudo`.

---

## Security

| Surface | How it's handled |
|---|---|
| State file | `~/.config/tytus/state.json` (Linux) or `~/Library/Application Support/tytus/state.json` (macOS), mode `0o600` |
| Refresh tokens | OS keychain (`com.traylinx.atomek` service) — never in plain files |
| WireGuard private keys | Parsed in memory only, never written to disk; `WireGuardConfig` implements `Zeroize` and zeroes on drop |
| Sentinel pass | `WannolotPassResponse` is `Zeroize` + `ZeroizeOnDrop` |
| TUN privilege | Tightly-scoped sudoers: only `tytus tunnel-up *` and `tytus tunnel-down *` (the `tunnel-down` helper internally validates the target PID against `/tmp/tytus/tunnel-*.pid` so it cannot be abused to SIGTERM other processes) |
| Tunnel daemon | Runs as root only for the lifetime of the WG socket; deletes its temp config file before opening the tunnel; auto-cleans PID + iface files on shutdown |
| HTTP client | `reqwest` with rustls + WebPKI roots + HTTP/2 + macOS SystemConfiguration; no `native-tls`, no plaintext fallback |

A full pre-public-release security audit is in
[`docs/SECURITY-AUDIT.md`](docs/SECURITY-AUDIT.md).

---

## Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| `No pods. Run: tytus connect` | No allocation | `tytus setup` (or `tytus connect`) |
| `Tunnel daemon already running` | Stale PID file | `tytus disconnect` then retry |
| `401 Invalid API key` from gateway | Stable key map sync race during first connect | Wait 2s and retry; `tytus restart` if persistent |
| `403 plan_limit_reached` | Unit budget would be exceeded | Revoke an existing pod or upgrade your plan |
| `503 no_capacity` | All droplets full | Wait or contact support |
| Tunnel up but `curl` times out | Routing collision with another VPN on macOS | Disconnect other VPNs, then `tytus connect` |
| Anything weird | — | Run `tytus doctor` first |

For deep AI-agent troubleshooting, run `tytus llm-docs` and feed the output
to your assistant.

---

## Development

```bash
cargo build -p atomek-cli -p tytus-mcp     # debug build
cargo build --release                       # release build
cargo test --workspace                      # run all tests
cargo clippy --workspace --all-targets      # lint
cargo audit                                 # vulnerability scan
```

Workspace dependencies are pinned in `Cargo.toml`. The `Cargo.lock` is
checked in.

---

## License

MIT — Traylinx

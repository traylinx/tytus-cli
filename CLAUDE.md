# CLAUDE.md

## Project: tytus-cli

CLI for connecting to **Tytus** private AI pods. Part of the Traylinx platform.
Two binaries: `tytus` (the CLI) and `tytus-mcp` (a stdio MCP server).

> **For AI agents driving Tytus:** run `tytus llm-docs` for the full
> structured reference. This file is just for engineers working ON the CLI.

> **In-flight architectural work:** `dev/` holds audits, design docs,
> and sprint plans for the next Tytus evolution. Start at
> [`dev/README.md`](dev/README.md). Completed sprints and reference
> material still live in `docs/`.

## Architecture

```
cli/      Binary crate: the `tytus` command + `daemon` subcommand (token-refresh
          background process) + `tray install/uninstall` (creates the
          Tytus.app bundle and its launch-at-login LaunchAgent)
mcp/      Binary crate: the `tytus-mcp` MCP server (stdio JSON-RPC 2.0)
core/     Error types, HTTP client (retry/backoff), token state, device fingerprint
auth/     Sentinel device auth, OS keychain integration, token refresh.
          `update_tokens()` persists the ROTATED refresh token to keychain
          on every refresh — without this, process restart reads stale RT
          and is forced into re-login. Critical invariant (2026-04-18 fix).
pods/     Provider API client: allocation, status, config, agent control, user-key
tunnel/   WireGuard tunnel via boringtun (userspace, cross-platform)
tray/     Binary crate: `tytus-tray` (menu-bar app). Single-instance guard
          via /tmp/tytus/tray.pid. Health dot is driven by a live HTTP
          probe to 10.42.42.1:18080 — NOT by daemon-status inspection.
          The tunnel lives independently of the daemon (boringtun is a
          separate root process), so a dead daemon does NOT mean 🔴.

          Hosts the localhost Tower web UI: tiny_http on 127.0.0.1:0
          (kernel-picked port, written to `/tmp/tytus/tray-web.port`)
          + an SSE Registry for streamed subprocess output. Tray menu
          items deep-link the user's browser at `<port>/tower#/<route>`
          rather than spawning Terminal.app for non-interactive
          actions (Run Doctor / Test, per-pod Restart / Revoke /
          Uninstall / Stop forwarder, Channels catalog, Add channel).
          Sudo / browser-auth / interactive-wizard commands still
          spawn Terminal because they need a TTY.
```

Workspace docs:
- `llm-docs.md` — full LLM-facing reference, included into both binaries via `include_str!`
- `.agents/skills/tytus/SKILL.md` — hosted skill file (raw.githubusercontent.com URL)
- `install.sh` — curl|sh installer (try release → fall back to cargo install --git)
- `README.md` — public-facing project README

## Build & run

```bash
cargo build -p atomek-cli                 # debug CLI
cargo build -p atomek-cli -p tytus-mcp    # debug both
cargo build --release                     # release both
target/release/tytus connect              # run (elevation handled internally)
```

## Test + lint + audit

```bash
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets
cargo audit                                # vulnerability scan
```

## Key commands (the CLI itself)

```bash
tytus setup                          # interactive wizard (login → pod → tunnel → test)
tytus login / logout                 # device auth via Sentinel / revoke + clear state
tytus status [--json]                # plan, pods, units, tunnel state
tytus doctor                         # full diagnostic
tytus connect [--agent T] [--pod NN] # allocate + tunnel up
tytus disconnect [--pod NN]          # tear down tunnel daemon (allocation kept)
tytus revoke <pod_id>                # DESTRUCTIVE: free units + wipe state
tytus restart [--pod NN]             # restart agent container
tytus env [--export] [--raw]         # connection vars (stable by default)
tytus test                           # E2E health check
tytus chat / configure / exec        # interactive REPL / overlay editor / shell exec
tytus link [DIR]                     # drop AI integration files into a project
tytus mcp [--format ...]             # print MCP server config for an AI tool
tytus bootstrap-prompt               # the paste prompt that points at the hosted SKILL.md
tytus autostart install|uninstall|status  # LaunchAgent / systemd autostart
                                          # Installs TWO plists on macOS:
                                          #   com.traylinx.tytus (oneshot connect)
                                          #   com.traylinx.tytus.daemon (KeepAlive)
tytus daemon run|stop|status         # Background token-refresh daemon. Never
                                     # calls Sentinel on its own schedule unless
                                     # ensure_token() decides a refresh is due.
                                     # Exponential backoff (60s→3600s cap) on
                                     # transient failure; AuthExpired puts us in
                                     # NeedsLogin and we reload from keychain on
                                     # subsequent ticks in case the user re-logged.
tytus tray install|uninstall|status  # macOS: creates /Applications/Tytus.app
                                     # (LSUIElement=true, icon.icns generated
                                     # at install time via sips + iconutil)
                                     # and com.traylinx.tytus.tray LaunchAgent.
tytus llm-docs                       # full LLM-facing reference
```

**Global flags:** `--json` (machine output), `--headless` (force non-interactive mode, also set via `TYTUS_HEADLESS=1` env var).

Hidden subcommands (used internally):
- `tytus tunnel-up <config_file>` — runs the tunnel daemon as root
- `tytus tunnel-down <pid>` — validated SIGTERM helper for the daemon

## State + secrets

- `~/Library/Application Support/tytus/state.json` (macOS) or `~/.config/tytus/state.json` (Linux), mode `0o600`
- OS keychain entry: service `com.traylinx.atomek` (legacy name; do not change without migration)
- Tunnel daemon PID files: `/tmp/tytus/tunnel-NN.pid` (cleaned up on exit)
- Token-refresh daemon socket + PID: `/tmp/tytus/daemon.sock`, `/tmp/tytus/daemon.pid`
- Tray single-instance lock: `/tmp/tytus/tray.pid`
- Diagnostic logs:
  - `/tmp/tytus/autostart.log` — headless-mode token refresh results, startup state, tunnel success/failure
  - `/tmp/tytus/daemon.log` — persistent daemon stdout/stderr (launchd-captured)
  - `/tmp/tytus/tray.log` — tray stdout/stderr
- LaunchAgents (macOS, user scope):
  - `~/Library/LaunchAgents/com.traylinx.tytus.plist` — oneshot tunnel-up at login
  - `~/Library/LaunchAgents/com.traylinx.tytus.daemon.plist` — KeepAlive'd refresh daemon
  - `~/Library/LaunchAgents/com.traylinx.tytus.tray.plist` — launch-at-login tray app
- App bundle (macOS): `/Applications/Tytus.app` with `LSUIElement=true`, `CFBundleIdentifier=com.traylinx.tytus`

## Security invariants

- State file MUST be 0600.
- Refresh tokens go to the OS keychain, never to plain files.
- WireGuard private keys parsed in memory only; `WireGuardConfig` and `WannolotPassResponse` implement `Zeroize`.
- Sudoers entry is tightly scoped: only `tytus tunnel-up *` and `tytus tunnel-down *`. The `tunnel-down` helper validates the target PID against `/tmp/tytus/tunnel-*.pid` so it cannot be abused as an arbitrary `kill` primitive.
- `reqwest` uses rustls + WebPKI roots (no `native-tls`, no plaintext fallback).
- All hardcoded URLs in source point at production Traylinx SaaS endpoints (api.makakoo.com, sentinel.traylinx.com, tytus.traylinx.com). These are public by design.

## Production endpoints (consumed by the CLI)

- Provider gateway: `https://tytus.traylinx.com`
- Sentinel device auth: `https://sentinel.traylinx.com`
- Auth API: `https://api.makakoo.com/ma-authentication-ms/v1/api`
- Metrics / Wannolot Pass: `https://api.makakoo.com/ma-metrics-wsp-ms/v1/api`

## Stable endpoint model

Inside the WireGuard tunnel, every droplet exposes a dual-bound address
`10.42.42.1:18080` that is the same on every droplet. Combined with the
per-user stable key (`sk-tytus-user-<32hex>`, persisted in Scalesys'
`user_stable_keys` table and rewritten by nginx via a `map` directive),
the user gets one URL + one key that never changes across pod
revoke/reallocate, agent swaps, droplet migration. The CLI's `tytus env`
emits this pair by default; `--raw` falls back to per-pod values for
debugging.

## Contributing notes

- Prefer modifying `llm-docs.md` over inlining new constants — both
  binaries `include_str!` from it so changes propagate automatically.
- When adding a subcommand, update: `Commands` enum, the dispatcher
  match arm, the `--help` description, and the relevant section in
  `llm-docs.md`. Slash-command bodies in `main.rs` are secondary.
- All security-sensitive changes need to be documented in
  `docs/SECURITY-AUDIT.md` and re-validated against the audit gate
  before publishing a release.

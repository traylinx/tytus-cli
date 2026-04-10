# CLAUDE.md

## Project: tytus-cli

CLI for connecting to **Tytus** private AI pods. Part of the Traylinx platform.
Two binaries: `tytus` (the CLI) and `tytus-mcp` (a stdio MCP server).

> **For AI agents driving Tytus:** run `tytus llm-docs` for the full
> structured reference. This file is just for engineers working ON the CLI.

## Architecture

```
cli/      Binary crate: the `tytus` command
mcp/      Binary crate: the `tytus-mcp` MCP server (stdio JSON-RPC 2.0)
core/     Error types, HTTP client (retry/backoff), token state, device fingerprint
auth/     Sentinel device auth, OS keychain integration, token refresh
pods/     Provider API client: allocation, status, config, agent control, user-key
tunnel/   WireGuard tunnel via boringtun (userspace, cross-platform)
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
tytus llm-docs                       # full LLM-facing reference
```

Hidden subcommands (used internally):
- `tytus tunnel-up <config_file>` — runs the tunnel daemon as root
- `tytus tunnel-down <pid>` — validated SIGTERM helper for the daemon

## State + secrets

- `~/Library/Application Support/tytus/state.json` (macOS) or `~/.config/tytus/state.json` (Linux), mode `0o600`
- OS keychain entry: service `com.traylinx.atomek` (legacy name; do not change without migration)
- Tunnel daemon PID files: `/tmp/tytus/tunnel-NN.pid` (cleaned up on exit)

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

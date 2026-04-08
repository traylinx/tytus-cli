# CLAUDE.md

## Project: tytus-cli

CLI for connecting to Tytus private AI pods. Part of the Traylinx platform.

## Architecture

```
cli/     — Binary crate (the `tytus` command)
core/    — Error types, HTTP client, token state, session handoff
auth/    — Device auth (Sentinel), keychain, token refresh
pods/    — Provider API: pod allocation, status, config, revoke
tunnel/  — WireGuard tunnel via boringtun (userspace, cross-platform)
```

## Build & Run

```bash
cargo build -p atomek-cli          # Debug build
cargo build --release -p atomek-cli # Release build
sudo target/release/tytus connect   # Run (needs sudo for TUN)
```

## Key Commands

```bash
tytus login                    # Device auth (opens browser)
tytus status [--json]          # Show plan + pods
sudo tytus connect             # Allocate pod + tunnel (Ctrl+C to stop)
sudo tytus connect --agent hermes  # Hermes agent (2 units)
sudo tytus connect --pod 01   # Reconnect existing pod
tytus env --export             # Print connection env vars
tytus revoke <pod_id>          # Free a pod's units
tytus logout                   # Revoke all + logout
```

## State

- `~/.config/tytus/state.json` (permissions 0600)
- OS keychain for refresh tokens (cross-tool compatibility)

## Security Invariants

- State file must be owner-only (0600)
- Tokens never logged or printed to stdout (except --json for env command)
- WireGuard keys zeroed on drop (Zeroize trait)
- Tunnel runs in userspace via boringtun — no kernel module needed

## API Endpoints

- Provider: `https://tytus.traylinx.com`
- Scalesys: `https://scalesys.traylinx.com`
- Auth: `https://api.makakoo.com/ma-metrics-wsp-ms/v1/api`
- Sentinel: `https://traylinx.com/devices/`

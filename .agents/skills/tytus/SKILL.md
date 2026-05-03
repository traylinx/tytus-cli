---
name: tytus
description: Use `tytus` by Traylinx — a CLI, tray, and TytusOS desktop for private AI pods, OpenAI-compatible gateway access, pod lifecycle, Files, channels, terminal, and MCP integration.
---

# tytus — Agent Instructions

You have access to **Tytus by Traylinx** via the `tytus` CLI on the user's machine. Tytus is the user's private AI pod ecosystem: CLI + tray daemon + TytusOS browser desktop + OpenAI-compatible gateway.

Prefer `tytus` commands over raw curl. The CLI knows auth state, stable gateway values, pods, agents, tunnel, and local daemon state.

## Step 0 — Verify install

```bash
command -v tytus >/dev/null && tytus --version
```

If missing:

```bash
curl -fsSL https://get.traylinx.com/install.sh | bash
```

macOS users can also install with Homebrew:

```bash
brew install traylinx/tap/tytus
```

Platform note for v0.6.9: macOS and Linux are the production CLI/daemon path. Windows release binaries exist for CLI/MCP, but full daemon/tray/tunnel runtime parity is preview.

## Step 1 — Load fresh references

```bash
tytus llm-docs   # CLI, models, commands, errors
tytus os-docs    # TytusOS desktop, Files, Pod Inspector, Settings
```

Treat those as source of truth for the current installed binary.

## Step 2 — Check state before acting

```bash
tytus status --json
```

Interpretation:

- not logged in or session expired: run `tytus login`, then re-check status.
- no pods: run `tytus setup` or `tytus agent install nemoclaw` after user intent is clear.
- pods present: inspect readiness before claiming they are ready.

Use `tytus doctor` for diagnostics. Do not delete pods to fix login/session expiry.

## Step 3 — Stable gateway values

```bash
eval "$(tytus env --export)"
echo "$OPENAI_BASE_URL"   # http://10.42.42.1:18080/v1
echo "$OPENAI_API_KEY"    # sk-tytus-user-<32hex>
```

These are the only values to paste into user configs. Never hardcode per-pod URLs or raw per-pod keys except for debugging.

## TytusOS facts

- TytusOS is the primary UI. Legacy Tower is hidden rollback only via `TYTUS_ENABLE_LEGACY_TOWER=1`.
- Open with `tytus tray install` then `open -a Tytus` on macOS, or open the tray web port.
- Tytus Home is `~/Tytus` with `Downloads`, `Inbox`, `Logs`, `Outbox`, `Pods`, `Projects`, and `Shared`.
- Pod Inspector owns readiness, Open, Restart, Doctor, Logs, Env, Uninstall, Revoke.
- Files owns local Tytus Home, shared folders, pod workspace, inbox, downloads.
- Terminal is host-backed and should default to Tytus Home.
- Session expired means re-auth needed. Pods keep running.

## Plans and units

| Plan | Units |
|---|---|
| Explorer | 1 |
| Creator | 2 |
| Operator | 4 |

| Agent | Cost | UI/API | Notes |
|---|---:|---|---|
| `nemoclaw` | 1 | OpenClaw/NemoClaw on port 3000 | Best default autonomous coding pod |
| `hermes` | 2 | dashboard 9119, gateway 8642 | Nous Hermes dashboard + gateway |

## Models

Run `tytus llm-docs` for exact current catalog. Default chat is `ail-compound`; image generation is `ail-image`; embeddings are `ail-embed`. Do not invent OpenAI/Claude/Qwen model IDs.

## Common commands

```bash
tytus setup
tytus login
tytus status --json
tytus doctor
tytus tray install
tytus env --export
tytus os-docs
tytus agent catalog
tytus agent install nemoclaw
tytus agent install hermes
tytus agent list
tytus exec --pod 01 "pwd && ls -la /app/workspace"
tytus push ./file.pdf --pod 01
tytus pull /app/workspace/out/result.md --pod 01
tytus link .
tytus mcp --format json
```

## Safety rules

1. `tytus revoke`, `tytus logout`, uninstall, and destructive file deletes require explicit user confirmation.
2. Never fix session expiry by revoking pods.
3. Never treat a public URL as readiness. Check Pod Inspector/readiness/doctor.
4. Never expose secrets in chat or docs. Redact keys unless the user explicitly asks to reveal them.
5. Prefer friendly user actions: open TytusOS, run `tytus login`, run `tytus doctor`, inspect readiness.

# Tytus ecosystem guide

Last updated: 2026-05-03. Applies to `tytus` v0.6.12.

Tytus is three pieces that should feel like one product:

1. **Tytus CLI** (`tytus`) handles login, pod lifecycle, tunnels, file transfer, diagnostics, MCP, and project linking.
2. **Tytus Tray** (`Tytus.app` on macOS, daemon/tray on supported desktop platforms) keeps the local daemon running and exposes quick actions.
3. **TytusOS** is the local browser desktop served by the tray. It is now the primary UI. Legacy Tower is hidden behind `TYTUS_ENABLE_LEGACY_TOWER=1` only for rollback.

## Current release status

| Surface | Status in v0.6.12 |
|---|---|
| macOS | Full CLI + tray + TytusOS path. Homebrew and release zip available. |
| Linux | CLI, daemon, tunnel, MCP, TytusOS browser path, and release zip available. Desktop shell integration varies by distro. |
| Windows | Release zip + installer path exist for CLI/MCP. Full daemon/tray/tunnel parity is still a future sprint, so treat Windows desktop runtime as preview. |
| Tower | Not the user-facing product anymore. Keep only for hidden rollback until the deletion gate. |

## Install on a fresh machine

### macOS

```bash
brew install traylinx/tap/tytus
# or
curl -fsSL https://get.traylinx.com/install.sh | bash

tytus setup
tytus tray install
open -a Tytus
```

### Linux

```bash
curl -fsSL https://get.traylinx.com/install.sh | bash
tytus setup
```

If the desktop tray is not packaged for your distro yet, run TytusOS from the local daemon URL printed by `tytus status` or open the tray web port manually:

```bash
cat /tmp/tytus/tray-web.port
open "http://127.0.0.1:$(cat /tmp/tytus/tray-web.port)/" 2>/dev/null || xdg-open "http://127.0.0.1:$(cat /tmp/tytus/tray-web.port)/"
```

### Windows

Download `tytus-windows-x86_64.zip` from the GitHub release or use the installer with `TYTUS_USE_RELEASE=1` when instructed. CLI/MCP packaging is present in v0.6.12; full daemon/tray/tunnel runtime is not yet production-complete on Windows.

## First-run flow

```bash
tytus setup
```

The wizard signs in through Sentinel, checks the plan, creates or reuses the default gateway pod, optionally installs an agent pod, brings the tunnel up, and runs a smoke test.

After setup, these values are stable and are what users paste into tools:

```bash
eval "$(tytus env --export)"
echo "$OPENAI_BASE_URL"   # http://10.42.42.1:18080/v1
echo "$OPENAI_API_KEY"    # sk-tytus-user-<32hex>
```

Do not teach users to paste per-pod URLs or per-pod keys. Those rotate.

## TytusOS daily use

Open TytusOS from the tray menu or `http://127.0.0.1:<tray-port>/`.

Core apps:

| App | User job |
|---|---|
| Pod Inspector | See all pods, install agents, open agent UI, inspect readiness, restart, doctor, logs, env, revoke/uninstall with confirmation. |
| Files | Browse `~/Tytus`, shared folders, pod workspaces, inbox, downloads, and safe root-anchored sources. |
| Channels | Configure Telegram/Discord/Slack/LINE-style pod channels without exposing tokens in URLs. |
| Terminal | Real host-backed shell with Tytus Home as default cwd. Use for `tytus`, `claude`, `opencode`, project commands. |
| Settings | Account, plan, daemon, sharing, background, appearance, dock, languages, notifications, privacy, about. |
| Music Creator / JULI3TA | Uses the included AIL gateway and host-backed library persistence. |

## Tytus Home

Fresh installs create `~/Tytus` as the user-facing workspace:

```text
~/Tytus/
├── README.md
├── Downloads/
├── Inbox/
├── Logs/
├── Outbox/
├── Pods/
├── Projects/
└── Shared/
```

Use this as the default cwd for the TytusOS terminal and the default Files landing page. It is safe to show to users. Internal config stays under platform config dirs, not inside `~/Tytus`.

## Session expiry

A session can expire while pods keep running. This is not data loss.

Expected UI behavior:

- Top bar shows **Session expired**.
- Settings → Daemon shows a re-authentication card.
- User clicks **Sign in again**. Browser opens the Sentinel one-time login.
- After approval, TytusOS must refresh state and clear the expired badge.
- Pods remain online. Local files are not deleted. Do not revoke pods to fix login.

CLI fallback:

```bash
tytus login
tytus status
tytus doctor
```

## Pod readiness truth

Do not call a pod ready just because it has a URL. Readiness has layers:

1. Pod allocated.
2. Tunnel/public URL assigned.
3. Agent HTTP/API health reachable.
4. Agent UI route reachable.
5. Tytus bootstrap/smoke health passed.
6. Shared storage helper configured when applicable.

TytusOS should show `Running` when the agent is usable, `Not ready` when smoke checks fail, and `Degraded` when optional integrations such as shared storage are missing. The **Open** button must only imply that the URL can be opened, not that every integration is healthy.

## Files and sharing

User-visible file paths:

- Local home: `~/Tytus`
- Local shared drop-zone: `~/Tytus/Shared` unless a Garage binding maps another folder
- Pod workspace: `/app/workspace`
- Pod inbox: `/app/workspace/inbox`
- Pod downloads/outbox: `/app/workspace/out` or agent-specific output folders

Rules:

- Never expose raw CLI stderr for an empty folder. Show a friendly empty state.
- Path operations must stay root-anchored. No `..`, symlink escape, null byte, or double-encoding escape.
- Destructive file operations need explicit confirmation.
- Large or continuous shared-folder sync goes through Garage/garagetytus, not base64 `tytus push`.

## Agent defaults and autonomy

A fresh OpenClaw or Hermes pod should have enough environment to work without manual bootstrap:

- `AIL_URL` / `AIL_INFERENCE_URL` / `OPENAI_BASE_URL`
- `AIL_API_KEY` / `OPENAI_API_KEY`
- `TYTUS_POD_ID`, `TYTUS_AGENT_TYPE`, workspace paths
- shared-storage helper when the image supports it
- clear docs in `/app/workspace/README.md`

Known managed limitations to document honestly:

- `ail-speech` can return provider 429 when quota is exhausted.
- Vision should prefer public URLs or uploaded files. Inline base64 data URLs are not guaranteed.
- Windows runtime is preview until daemon/tray/tunnel parity lands.

## Commands users actually need

```bash
tytus setup                  # first run
tytus status                 # current state
tytus doctor                 # diagnose
tytus login                  # refresh session
tytus tray install           # install desktop app on macOS
tytus env --export           # stable SDK env
tytus agent catalog          # available agents
tytus agent install nemoclaw # allocate/install OpenClaw/NemoClaw
tytus agent install hermes   # allocate/install Hermes
tytus ls /app/workspace --pod 01
tytus push ./file.pdf --pod 01
tytus pull /app/workspace/out/result.md --pod 01
tytus os-docs                # TytusOS manual for AI agents
```

## Support checklist

When a user says "Tytus is broken", check in this order:

1. `tytus status --json`
2. Settings → Daemon session card, or `tytus login` if expired
3. `tytus doctor`
4. Pod Inspector → pod readiness tab
5. `tytus agent list` / `tytus agent env --pod NN`
6. Files → `~/Tytus` and pod `/app/workspace` visibility
7. Browser console only after the daemon/CLI state is known

Never fix a session issue by deleting pods. Never fix pod readiness by pretending the URL is enough.

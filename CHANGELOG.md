# Changelog

All notable changes to the `tytus` CLI, `tytus-mcp` server, and
`Tytus.app` menu bar tray. Follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
conventions; versioning is [SemVer](https://semver.org/) — pre-1.0 minor
bumps are allowed to break compat.

## [Unreleased]

## [0.5.4] — 2026-04-26

Tower learns about garagetytus shared folders. Read-only parity
with the v0.5.2/v0.5.3 tray submenu — list bindings, status,
conflicts, refresh credentials — plus a new per-pod "Refresh creds"
button on the Output toolbar. Bind stays tray-only because the
browser sandbox can't surface a real OS folder path.

### Tower additions
- **New "Shared Folders" details section** between Settings and
  Troubleshoot. Auto-refreshing list of bindings (one click =
  open in Finder) reads `~/.cache/garagetytus/bisync/*.bindings.json`
  via the new `GET /api/shared-folders/list` endpoint. Empty case
  shows guidance pointing at the tray + CLI for binding.
- **Streamed action buttons** for Status / Conflicts / List
  (full table) / Refresh-all wire to a new shared `sf-panel` log
  surface mirroring the Doctor / Health Test pattern. SSE event
  stream from `POST /api/shared-folders/run-streamed?action=…`
  via the existing job-event Registry.
- **"Open ~/.cache/garagetytus" button** for direct Finder access
  to bisync workdir + sidecar metadata.
- **Per-pod "Refresh creds" button** added to the Output toolbar
  alongside Restart / Stop forwarder / Uninstall / Revoke. Streams
  `garagetytus-pod-refresh <NN>` into the same per-pod log panel.

### Backend additions
- `GET  /api/shared-folders/list` — returns sidecar JSON list
- `POST /api/shared-folders/run-streamed?action=<status|conflicts|list|refresh-all>`
  — returns `{ job_id }`, output via existing `/api/jobs/<id>/stream`
- `POST /api/shared-folders/open` — body `{local_path}`, opens in
  Finder; 404 on orphan sidecar so the UI can flag it
- `POST /api/shared-folders/open-cache` — opens `~/.cache/garagetytus`
- `POST /api/pod/refresh-creds?pod=NN` — returns `{ job_id }` for a
  per-pod cred rotation
- New `resolve_garagetytus_helper(name)` mirrors the resolver in
  `tray/src/shared_folders.rs::helper_path` so backend + tray agree
  on which binary to invoke (`/usr/local/bin/`, `/opt/homebrew/bin/`,
  `~/garagetytus/bin/`, then bare PATH lookup)
- New generic `spawn_external_command(job, bin, args)` mirrors
  `spawn_pod_action`'s line-buffered stdout+stderr streaming but
  parameterized over the binary path, so any garagetytus-* helper
  (or future external) can stream into Tower's job channel

### Out of scope (intentional)
- **Bind from Tower** — needs a real OS folder path picker, which
  the browser sandbox doesn't expose. Stays tray-only; the empty-
  state hint links to the tray and the CLI.
- **Unbind from Tower** — destructive op better with native modal
  confirmations than a web button.

## [0.5.3] — 2026-04-26

The Shared Folders submenu becomes useful at a glance. Each active
binding now appears at the top of the submenu as a click target —
clicking opens the bound local folder in Finder. No more "where
did I bind that to again?"

- **Dynamic per-binding entries** at the top of the Shared Folders
  submenu. One row per active binding in the format
  `<bucket>  ↔  <local-path>` (home prefix compressed to `~/`).
  Click opens the local folder in Finder. Sidecar JSONs at
  `~/.cache/garagetytus/bisync/*.bindings.json` (written by
  `garagetytus folder bind` v0.5.3+) are the source of truth.
  Empty case shows a disabled "No folders bound yet" item so the
  user can tell the difference between "nothing bound" and
  "garagetytus not installed".
- New `shared_folders::list_bindings()`, `Binding` struct,
  `menu_id_open_binding`, `parse_open_binding_id`, and
  `open_binding_in_finder` helpers — all in
  `tray/src/shared_folders.rs`. 2 new unit tests (round-trip the
  ID + reject unrelated IDs); 17 tray tests total.

## [0.5.2] — 2026-04-26

The tray learns about garagetytus shared folders. Two integration
surfaces — a per-pod entry under each pod's Files submenu, and a
new top-level "Shared Folders" submenu for global ops — wrap the
v0.5.3 garagetytus bash helpers (`garagetytus-folder-bind`,
`-list`, `-status`, `-conflicts`, `garagetytus-pod-refresh`,
`garagetytus-refresh-watchdog`).

- **Per-pod Files submenu** gains 2 entries at the bottom:
  - "Bind a Mac folder to this pod…" — folder picker → osascript
    bucket-name dialog → spawn `garagetytus-folder-bind <path>
    <bucket> --to <pod-id> --auto-sync` in a detached thread →
    macOS notification on completion (Reveal in Finder on success).
  - "Refresh shared-folder credentials" — spawn
    `garagetytus-pod-refresh <pod-id>` and notify on completion.
    Pod's wrapper re-reads creds on every call so no pod restart.
- **New top-level "Shared Folders" submenu** between Pods and
  Settings:
  - List bindings…              `garagetytus-folder-list`
  - Status (with pod check)…    `garagetytus-folder-status --check-pods`
  - Find conflicts…             `garagetytus-folder-conflicts`
  - Open ~/.cache/garagetytus   (Finder)
  - Run cred refresh now (every pod)  `garagetytus-refresh-watchdog`
- **Graceful absence.** Both surfaces probe for the
  `garagetytus-folder-bind` helper at `/usr/local/bin`,
  `/opt/homebrew/bin`, and `~/garagetytus/bin/`. When no helper is
  found, the menu items still appear (so the integration is
  discoverable) but disabled with the suffix
  "(install garagetytus first)". No silent failure on click.
- New `tray/src/shared_folders.rs` module mirrors the `files.rs`
  pattern — menu_id helpers, per-pod action enum + parser,
  `spawn_*` functions in detached threads, 4 unit tests.

## [0.5.1] — 2026-04-25

Tower becomes the output surface for non-interactive tray actions.
No more cascade of Terminal windows for routine operations; per-pod
state, action streams, and channel-add token entry all live inside
one local web page now.

- **Tower in-page actions + per-pod subpages** — tray menu items that
  used to spawn Terminal.app for `test`, `doctor`, per-pod
  `restart` / `revoke` / `uninstall` / `stop-forwarder`,
  `channels catalog`, `channel-remove`, and `channel-add` now
  deep-link the user's browser into the local Tower web UI. New
  hash-route grammar:
    - `#/run/<action>` — global commands (`test`, `doctor`,
      `channels-catalog`). The doctor route also opens Troubleshoot.
    - `#/pod/<NN>` — per-pod subpage with Overview / Output / Channels
      tabs. The Output tab streams subprocess stdout+stderr live via
      the existing SSE Registry/JobEvent infrastructure (no PTY, no
      WebSocket, no xterm.js).
    - `#/pod/<NN>/<action>` — runs a per-pod streamed action
      (`restart`, `revoke`, `uninstall`, `stop-forwarder`); concurrent
      attempts on the same pod return HTTP 409 from the new
      `Registry::create_pod`. Successful revoke auto-navigates back
      to Tower so the user isn't stranded on a 'pod not found' page.
    - `#/pod/<NN>/channels?action=add&type=<channel>` — opens an
      in-page native `<dialog>` to collect the bot token. The token
      rides only the local POST body (127.0.0.1) and is forwarded to
      the `tytus` subprocess as an argv element — no shell, no
      Terminal `read -rs`, no logs. (Threat-model note in the
      handler: the token is briefly visible to local processes via
      `ps aux` for the ~10–15s subprocess lifetime; same exposure as
      the prior Terminal flow.)
  Backend additions: `Job::pod_id`, `JobEvent::Exit { code }`,
  `Registry::create_pod` (busy-check returns 409), `Registry::active_pods`
  (surfaced as `active_jobs_per_pod` in `/api/state` for the running-
  job dot in the overview), strict action whitelist via
  `pod_action_argv` (no free-form strings ever reach the shell —
  Command::arg per token; doctor/test rejected because they aren't
  pod-scoped), `open_tower_at(fragment)` helper that auto-appends a
  nonce so repeat tray clicks force `hashchange` to fire.
  Tray-menu rewires: `test`, `doctor`, `pod_NN_restart`,
  `pod_NN_revoke`, `pod_NN_uninstall`, `pod_NN_channels_catalog`,
  `pod_NN_channel_X_add`, `pod_NN_channel_X_remove` migrated from
  `open_in_terminal_simple` to `web_server::open_tower_at`. Tower-side
  `handle_channels_catalog` and `handle_channels_remove` migrated from
  Terminal-spawn to inline `run_tytus_inline`. `handle_channels_add`
  rewritten to accept JSON `{pod, channel, token}` body; parse-error
  path returns only `{"error":"bad json"}` (never echoes raw body —
  would leak the token).
  Stays in Terminal.app (intentional, requires TTY): `connect` (sudo),
  `login` (Sentinel browser-auth), `logout`, `tray install` (sudo +
  bundler), `configure` (multi-step interactive wizard), autostart
  toggles (sudo plist), editor launches (`launch_<editor>`,
  `launch_terminal`), the "Try Again" reconnect dialog, and the
  install-agent terminal fallback when the localhost Tower server
  isn't bound. CSP is unchanged (`connect-src 'self'`); no new heavy
  deps (no `portable-pty`, no `tokio-tungstenite`, no `xterm.js`).
  Binary delta ~15KB; release build 6.7M. 11/11 tray tests
  (4 new in `web_server::tests`); 68/68 workspace tests; `cargo
  clippy` clean (no new warnings). Verified across 5 lope review
  rounds with `pi` + `qwen` validators.
  Sprint doc: `~/Projects/makakoo/sprints/wannolot-embedded-terminal-2026-04-24/SPRINT.md`.

- **fix(tower): Run Doctor / Run Health Test stream output line-by-line**
  — both endpoints now respond `{job_id}` (HTTP 202) and stream
  subprocess output via SSE on `/api/jobs/<id>/stream`, same pipeline
  as the install flow + per-pod streamed actions. Pre-fix the page
  showed an empty `<pre>` for ~10 s then dumped the whole output at
  once, because `handle_test`/`handle_doctor` used `Command::output()`
  which blocks until exit. Refactored to spawn-piped + Registry
  pattern. Required pairing with the next two fixes (`wizard::flush()`
  + `with_chunked_threshold(0)`) to actually reach the browser as
  bytes arrive.

- **fix(cli): wizard helpers flush stdout per line**
  (`cli/src/wizard.rs::flush()`) — when `tytus test` / `tytus doctor`
  are spawned with `Stdio::piped()` (which the tray now does for the
  streaming pipeline), Rust block-buffers `println!` output. Every
  spinner / finish_ok / print_* line accumulated in stdout's
  `BufWriter` and only flushed at process exit. Added an explicit
  flush after every println in the non-TTY branches of `spinner` /
  `finish_ok` / `finish_fail` / `print_box` plus unconditionally in
  `print_header` / `print_step` / `print_ok` / `print_fail` /
  `print_warn` / `print_info` / `print_hint` / `print_logo` /
  `print_success_banner`. No-op when running in a TTY (already
  line-buffered).

- **fix(tower): force chunked transfer encoding on SSE responses**
  (`tray/src/web_server.rs::sse_response` →
  `.with_chunked_threshold(0)`) — tiny_http's default chunked
  threshold is 32 KB. When the response body length is unknown AND
  total output is shorter, tiny_http BUFFERS the entire body to
  compute `Content-Length` — defeating SSE streaming entirely.
  `tytus test` output is ~1 KB so all frames were buffered to the
  end. Setting `chunked_threshold(0)` forces chunked from the first
  byte: each pipe `read()` produces a chunk that flushes immediately.
  Verified end-to-end with `curl -sN /api/jobs/<id>/stream` piped
  through a millisecond-precision timestamper — frames now spread
  across the subprocess's actual runtime instead of clustering at
  process exit. The install flow's apparent streaming under the old
  code was coincidental: install output crosses the 32 KB threshold
  mid-run, so it switched to chunked then. All other SSE consumers
  (test, doctor, per-pod actions) silently 100% buffered until now.

Hard-won lessons captured as durable memories in the project's
auto-memory store: `feedback_tray_binary_staleness` (rebuild + swap
recipe after tray edits) and `feedback_tiny_http_chunked_threshold`
(the one-line streaming gate).

## [0.5.0] — 2026-04-24

Four themes consolidated from the three in-flight `v0.5.x-alpha`
drops plus the just-shipped shared-folders sprint:

- **Shared folders** — `tytus push / pull / ls / rm / transfers`
  move files between your Mac and any pod. Tray Files submenu per pod
  with osascript picker + macOS notifications. Portable
  `skill-tytus-files` skill with 53 EN+ES trigger phrasings so every
  infected AI CLI can translate "push the PDF to pod 2" / "manda el
  reporte al pod-04" into the right invocation. Full doc at
  `docs/file-sharing.md`.
- **Channels** — `tytus channels add/list/remove/catalog` wires
  OpenClaw's Telegram / Discord / Slack / LINE extensions without a
  browser. Pod egress bridge + DOCKER-USER iptables rules enable
  outbound chat-API reachability without exposing cross-pod traffic
  or the metadata endpoint.
- **Daemon hardening** — state.json mtime watcher + self-heal
  watchdog + stale-PID sweeper + degradation surfaced in tray. Fixes
  the cache-coherence class of bugs (stale in-memory creds, silent
  keychain timeouts, tray showing "Sign In" while logged in).
- **Lope teammates** — `tytus lope ask/install/list/identity` turns
  pod agents (OpenClaw, Hermes) into lope validators; `tytus bridge`
  ships the reverse channel for pod → Harvey notifications via brain
  journal + superbrain event store. Python SDK at `tytus_sdk/`.

### Shared folders (new in 0.5.0)

#### Added

- **`tytus push <LOCAL> [--pod NN] [--to /app/workspace/PATH]`** —
  file or directory push. Directories are tarred + gzipped locally and
  unpacked on the pod. Default destination is `/app/workspace/inbox/`
  (auto-created). Smart `--pod` default: one pod connected → auto-
  picked; multiple → refuses with the list.
- **`tytus pull <REMOTE> [--pod NN] [--to LOCAL]`** — inverse. Files
  + whole directories supported.
- **`tytus ls [PATH] [--pod NN] [--json]`** — list pod contents under
  `/app/workspace/`. Columns: mode, size, mtime, name.
- **`tytus rm <REMOTE> [--pod NN] [--recursive]`** — delete. Refuses
  directories without `--recursive`; refuses anything outside
  `/app/workspace/` unconditionally.
- **`tytus transfers [--tail N] [--pod NN] [--json]`** — reads the
  append-only JSONL audit log. Every `push`/`pull`/`rm` invocation
  (success OR failure) appends exactly one line.
- **Progress bar** on stderr for transfers > 1 MB (`indicatif`),
  suppressed with `--quiet`.
- **100 MB ceiling per transfer** — refused with a clear pointer to
  the planned v0.7 Garage-backed shared filesystem. Deliberate;
  docker-exec base64 streaming is the wrong foundation for GB-scale.
- **`cli/src/transfer.rs`** — shared helpers: path validation
  (rejects outside `/app/workspace/`, `..` segments, NUL bytes), size
  ceiling, `flock`-serialised JSONL transfer log, shell escaping.
- **`cli/src/cmd_transfer.rs`** — command implementations. Chunked
  256 KB base64 via the existing `tytus exec` pipeline (no new infra,
  fits `dash` ARG_MAX on the pod side).
- **Tray Files submenu** — `Pods ▸ pod-NN ▸ Files ▸ Push file… /
  Push folder… / List inbox in Terminal / Open local download
  folder`. Uses osascript for the file picker; posts macOS
  notifications on transfer completion and reveals pulled files in
  Finder.
- **`plugins-core/skill-tytus-files/`** (in `makakoo-os`) — portable
  SKILL.md with 53 EN + ES trigger phrasings ("push PDF to pod 2",
  "manda el reporte al pod 4", "qué hay en el pod 02", "descarga el
  log del pod-04", etc.), decision table, cross-CLI routing notes,
  and a regex-based trigger-corpus test (7 tests green).
- **`docs/file-sharing.md`** — canonical reference: mental model,
  quickstart, tray tour, skill discovery, when-NOT-to-use
  cross-references (to `harvey_knowledge_ingest` for RAG and the v0.7
  Garage sprint for bulk data), troubleshooting cheatsheet.

#### Why

Before this release the only path to move a file onto a pod was a
hand-crafted `tytus exec base64 -d` pipeline. This release makes the
feature feel like `scp` with zero ceremony — one command, smart pod
defaults, refuses path escapes before sending bytes. The 100 MB cap
is the boundary where the base64-over-exec transport stops being a
good idea; past that, wait for Garage.

### Known limitations

- **Drag-and-drop onto the menu bar icon** is deferred. The
  `tray-icon` crate wraps `NSStatusItem` without the
  `NSDraggingDestination` protocol, and subclassing via `objc2` is
  non-trivial. The menu-based file picker covers the same user
  intent in one extra click. Tracked for a follow-up release.
- **Live-pod integration tests** are manual. Unit + concurrency
  tests cover the helper surface (34 tests green across the CLI +
  tray + skill); end-to-end "push → pull → md5-match" is verified by
  hand before cutting.

### Channels (was v0.5.2-alpha)

Unblock OpenClaw's existing channel extensions (Telegram, Discord,
Slack Socket Mode, LINE). Two layers: infrastructure change so pods
can actually reach chat APIs, CLI change so users can configure
credentials without a browser UI.

### Added

- **`tytus channels` subcommand** — `add` / `list` / `remove` / `catalog`.
  Stores chat-channel credentials in the OS keychain, writes the
  per-pod view to `/app/workspace/.tytus/channels.json` via `tytus
  exec`, redeploys the agent container via DAM so the channel
  extension picks up the env vars at startup. Supports Telegram,
  Discord, Slack (Socket Mode), and LINE at launch. Adding more is a
  ~3-line change to `cli/src/channels.rs`.
- **`cli/src/channels.rs`** — static registry of known channels with
  their required env-var mappings, derived from each OpenClaw
  extension's `openclaw.plugin.json` → `channelEnvVars`.
- **`cli/src/channels_store.rs`** — keychain-backed secret storage +
  local manifest at `~/.tytus/channels.json` that tracks which
  channels are configured per pod.
- **DAM channel merging** — `agent_manager/app.py:agent_deploy`
  reads `state_dir/.tytus/channels.json` on every container deploy
  and merges credentials into the container's env. Non-fatal on
  missing/invalid files; only accepts `UPPER_SNAKE_CASE=string`
  entries. Schema is versioned (`"version": 1`) for future additions.
- **Pod-egress bridge** — sidecars now attach to a Docker bridge
  network (`pod-egress`, 172.30.0.0/16) in addition to WireGuard, so
  the OpenClaw/Hermes extensions can reach external chat APIs. No
  published ports → no inbound internet exposure. Cross-pod traffic
  still blocked at the host iptables FORWARD chain; metadata endpoint
  (169.254.169.254) still blocked; outbound allowlist enforced via
  new DOCKER-USER rules.
- **`scripts/e2e-channels.sh`** — 8-flow harness (3 static, 5 live).
  Static flows: binary surface, catalog contents, JSON output shape.
  Live flows (opt-in via `E2E_TELEGRAM_BOT_TOKEN`): add → channels.json
  on pod → container env → api.telegram.org reachable → remove.

### Changed

- **`services/wannolot-infrastructure/docker-compose.pod.j2`** —
  sidecars no longer `network_mode: none`. They now join the
  `pod-egress` bridge network so the pod container (which shares the
  sidecar's netns via `network_mode=container:...`) inherits a default
  route to the internet via Docker's NAT.
- **`services/wannolot-infrastructure/user-data.strato-eu-001.yml`**
  adds DOCKER-USER iptables rules scoped to `172.30.0.0/16`:
  allowlist TCP/443, UDP/53, TCP/53; block metadata + cross-bridge;
  deny everything else.

### Why

OpenClaw already ships first-class chat channel extensions for 20+
chat apps (Telegram, Discord, Slack, Signal, WhatsApp, iMessage,
Line, Matrix, Teams, Feishu, GoogleChat, etc.). They were
unreachable from Tytus pods because:
(a) pods had no internet egress (iptables DROP catch-all +
`network_mode: none` on the sidecar — no default route), and
(b) users had no way to configure bot tokens without the slow
browser UI tunnel.
This release fixes both at the smallest possible surface area: one
iptables chain + one bridge network + one CLI subcommand. No new
services, no new auth model, no broker. The deferred "TML messaging
layer" direction from earlier design drafts is correctly rejected
— see `dev/design/2026-04-20-unblock-openclaw-channels.md` for the
full reasoning.

### Deploy notes (not automatic)

The infrastructure change requires redeploying the sidecar containers
on each droplet:

1. Pull latest `wannolot-infrastructure` on the droplet
2. `cd /opt/wannolot-infrastructure && bootstrap/02-render-compose.sh`
3. `docker compose -f docker-compose.pod.yml down && docker compose -f docker-compose.pod.yml up -d`
4. Re-apply iptables (either reboot or re-run the egress-filter block
   from `user-data.strato-eu-001.yml`)
5. DAM gets the `channels.json` reader via a normal pull + restart of
   its container on the droplet

Until these steps run, `tytus channels add` will write keychain +
push to the pod, but the agent container will come up without the
new env vars (the channel extension will log "missing
TELEGRAM_BOT_TOKEN" and no-op).

### Daemon hardening (was v0.5.1-alpha)

Production-hardening pass against the class of bugs that shipped the
2026-04-20 tray regression: stale in-memory daemon state, broken
keychain ACL silently pinning the daemon to `NeedsLogin`, and tray
reading the stale view instead of state.json. Fix is systemic, not
just the immediate symptom.

### Added

- **`CliState::load_file_only()`** — side-effect-free state.json parse that
  skips the OS keychain. Safe on the status RPC hot path (~1 ms).
- **Daemon state watcher task** (`state_watcher_loop`) — polls state.json
  mtime every 500 ms; hot-reloads the daemon's in-memory credentials
  whenever any other process (`tytus login`, `tytus connect`, `tytus
  revoke`) updates the file. Ends the 30-min drift window.
- **Daemon self-heal watchdog** (`self_heal_loop`) — when the daemon has
  been stuck in `NeedsLogin` for >5 min while state.json is plainly
  logged in, exits so launchd/systemd can respawn with fresh state.
- **Stale-PID sweep** (`sweep_stale_pids`) — at daemon startup, reaps
  `daemon.pid` / `tray.pid` / `tunnel-*.pid` whose owning process is
  dead. Uses `kill -0` semantics (ESRCH ⇒ stale, EPERM ⇒ keep).
- **Daemon health telemetry in status RPC** — new `daemon` fields:
  `keychain_healthy` (bool), `last_refresh_error` (Option<String>),
  `stuck_for_secs` (Option<u64>).
- **Tray surfaces daemon degradation** — `TrayState` carries
  `keychain_healthy` + `last_refresh_error`; menu renders yellow dot +
  "⚠︎ keychain access pending — re-run `tytus login`" row on metadata
  line; Troubleshoot submenu shows last refresh error verbatim.
- **`scripts/e2e-multiprocess.sh`** — 7-flow harness for daemon ↔
  state.json ↔ tray coherence: ping, status-RPC latency, health-field
  presence, mtime-driven reload, NeedsLogin self-clearing, sweep-linked-
  into-binary, tray-merge agreement. Would have caught the 2026-04-20
  regression on the first run.

### Fixed

- **Daemon now hot-reloads state.json** on every `status` RPC call AND
  on file-mtime change (500 ms watcher). Before: up to 30 min staleness
  + indefinite pin when keychain ACL pended.
- **Tray no longer trusts the daemon over state.json for auth.** Merge
  in `tray/src/socket.rs`: file wins on `logged_in`; daemon contributes
  runtime fields (pid, uptime, pods).
- **`refresh_once` is keychain-resilient.** Two-stage reload: file-only
  first; if that yields a valid AT, stay `Running` regardless of
  keychain outcome. Transient keychain failures no longer flip the
  daemon into `NeedsLogin`.

### Why

The tray was showing "Sign In…" while the user was clearly logged in,
the tunnel was up, and pods were allocated. Root cause wasn't a tray
bug — it was a cache-coherence bug across three processes with no
invalidation protocol: daemon memory ↔ state.json ↔ keychain. Silent
keychain timeouts (logged, but invisible to the user) had pinned the
daemon to `NeedsLogin` 19 hours before the user noticed. Fix is the
watcher + self-heal + health surfacing together — each alone is
insufficient.

### Lope teammates (was v0.5.0-alpha)

Tytus pod agents are now first-class lope teammates with a reusable
Python SDK and a bidirectional bridge back to Harvey (brain journal +
superbrain event store). `tytus lope install` pairs a device on the pod
and registers a `subprocess` provider in `~/.lope/config.json` so
`lope negotiate --validators tytus-openclaw-<pod>` Just Works.

### Added

- **`tytus_sdk/` Python package** — reusable adapter SDK. Files:
  - `adapter.py` (`AgentAdapter` Protocol with `ask/stream/notify/identify`)
  - `identity.py` (Ed25519 keypair at `~/.tytus/openclaw/device.json`, 0600)
  - `adapters/openclaw.py` (OpenClaw WS v3 + v2-canonical Ed25519 handshake, fresh session per ask, `chat{state:"final"}` terminal detection)
  - `install.py` (pod device pairing via `tytus exec` + `~/.lope/config.json` merge)
  - `lope_bridge.py` (VERDICT-emitting subprocess validator with defensive fallback block when the agent skips the rubric)
  - `bridge_daemon.py` (HTTP listener `127.0.0.1:18099`, per-pod outbox pollers, lifecycle guard)
  - `cli.py` (argparse dispatcher — `ask / identity / install / uninstall / list / lope_validate / bridge`)
- **`tytus lope ask --pod NN "…"`** — direct WS ask against OpenClaw. Live reply verified against pod 02 (MiniMax M2.7).
- **`tytus lope install --pod NN`** — idempotent: adds our Ed25519 device to the pod's `/app/workspace/.openclaw/devices/paired.json` with `operator.{read,write,admin}` scopes, registers the `tytus-openclaw-NN` provider in lope.
- **`tytus lope uninstall` / `tytus lope list` / `tytus lope identity`** — inverse + inventory + pubkey dump.
- **`tytus bridge run`** — daemon: binds `127.0.0.1:18099`, spawns per-pod outbox pollers, drains `/app/workspace/.harvey-outbox.jsonl` every 10 s via `tytus exec`, writes to today's Brain journal + best-effort `superbrain remember`. Shared-secret auth via `X-Tytus-Bridge-Token` (kept at `~/.tytus/bridge.token`, mode 0600). Rate limit 30 notifies/pod/hour.
- **`tytus bridge status / rotate-token / test`** — ops surface.
- **`scripts/e2e-lope-teammate.sh`** — 10-flow harness covering SDK imports, identity, ask, VERDICT emission, lope registration, bridge auth (reject + accept), outbox polling end-to-end, lifecycle guard. Verified 10/10 green on pod 02.
- **`docs/DESIGN-TYTUS-LOPE-TEAMMATES.md`** + lope-negotiated sprint doc + **`docs/SECURITY-TEAMMATES.md`** covering device-key 0600, bridge-token isolation, rate-limit invariants, threat model with 7 open items tracked for v0.6+.

### Changed

- **Rust CLI gains `Commands::Lope` + `Commands::Bridge`** — thin pass-through subcommands that shell out to `python3 -m tytus_sdk`. SDK is the source of truth for protocol work; Rust side only handles CLI parsing, PYTHONPATH detection, and subprocess dispatch. Keeps v0.5 changes out of the Rust build surface.

### Phase 1 implementation notes (hard-won lessons)

- **Silent-local-pairing is unreachable over WG.** Server's `isLocalDirectRequest` requires loopback `req.socket.remoteAddress`; WG traffic arrives with the peer's WG IP.
- **Token-only connects get all scopes stripped.** `clearUnboundScopes` fires whenever `!device && authMethod==="token"`. Device identity is mandatory for write scopes.
- **`deviceId` must be `sha256(pub_raw).hex()`** — full 64 hex chars, matching `deriveDeviceIdFromPublicKey`.
- **`client.id` enum is strict.** `gateway-client` + `client.mode="backend"` avoids the Control-UI device-identity gate while keeping operator scope semantics.
- **`thinking` is required string**, not nullable; `"off"` disables reasoning.
- **Fresh session per ask.** Reusing `key="main"` binds to the pod's long-running `agent:main:main` and inherits full agent-orchestration loop. Unique `tytus-lope-<uuid>` key + unique label per ask.
- **Terminal signal is `event:"chat", state:"final"`** scoped to the sessions.send `runId`, not `session.message.status`.
- **Brain-outbox parser gotcha.** Python's `splitlines()` strips trailing `\n` — rebuilding with `"\n".join()` loses the "this line is complete" signal. Fixed by preserving the raw stdout from `tail -c +N` and testing `"\n" in body` directly.

### Known gaps (tracked for v0.6.0)

- `HermesAdapter` REST path — not shipped; design valid, just not coded.
- Keychain-backed bridge + device tokens (currently 0600 flat files).
- Pod-side `tytus_notify.py` helper not bundled in agent images (agents must append to outbox manually until v0.6 infra rebuild).
- Forwarder reverse-tunnel (Option 1 in §7.2 of design doc) — still polling JSONL via `tytus exec`.
- Audit log on pod for device-pair adds/removes.

## [0.4.0] — 2026-04-19

Zero-config Hermes + OpenClaw "one click → working chat" across the full
browser + SDK surface, plus a cold-boot reliability fix for macOS.

### Added

- **Hermes agent zero-config.** `tytus connect --agent hermes` now
  yields a working dashboard + API out of the box. The forwarder
  proxies `http://localhost:18700+pod_num/` to both the Hermes
  dashboard (Vite/React SPA, port 9119) and the Hermes gateway
  (OpenAI-compatible API, port 8642), multiplexing by path:
  `/v1/*`, `/api/jobs*`, `/health*` → gateway; everything else →
  dashboard. Auth (`API_SERVER_KEY`) auto-injected on gateway routes;
  dashboard's own session token is baked into the HTML by
  `hermes dashboard` itself. Commits `fbf1da9`, `0fc13f4`.
- **OpenClaw silent local pairing.** Browser connections to a
  nemoclaw pod now complete handshake without the "pairing required"
  prompt and without the user pasting a gateway token. Forwarder
  issues a 302 that seeds `?token=<T>` for the UI to strip via
  `history.replaceState`, keeps Host/Origin loopback so
  `isControlUiBrowserContainerLocalEquivalent` fires, and writes a
  `config.user.json` overlay that adds
  `http://localhost:18700+N` to `gateway.controlUi.allowedOrigins`
  (survives agent restart). Commits `b633c96`, `81d3c4a`, `3db77be`,
  `1603167`, `fb912e7`.
- **Forwarder self-heal.** On startup, the forwarder verifies the
  overlay is present and `gateway_token` is populated; recovers
  silently if either is missing by fetching from the pod via
  Provider's A2A path (no keychain round-trip needed). Commit
  `fb912e7`.
- **E2E flow harness.** `scripts/e2e-flows.sh` runs 35 flows across
  AUTH / POD / UI (nemoclaw + hermes) / ENV / DIAGNOSTICS / TRAY /
  HERMES-SIM. Safe to re-run, no destructive actions.
  `scripts/FLOWS.md` is the human matrix. Commit `2b86077`.
- **Sprint planning docs.** `docs/SPRINT-2026-04-19.md` (solo),
  `docs/SPRINT-2026-04-19-negotiated.md` (3-round lope-negotiated),
  `docs/SPRINT-P1-SHIP-v0.4.0.md` (focused ship plan). The
  negotiated versions apply validator feedback from claude / gemini
  / pi / qwen panel.

### Changed

- **`is_logged_in` now accepts a valid access token without a
  refresh token.** Previously required both. On macOS cold boot the
  keychain ACL can take seconds to approve after login and
  `get_refresh_token` times out in 3s — the old check saw `has_rt=
  false` and refused to connect even with a currently-valid AT. The
  daemon still retries the keychain in the background; once it
  unblocks, normal RT refresh resumes. Commit `b9d44df`.
- **Forwarder's `Authorization` header handling is now override,
  not preserve.** OpenAI SDK clients always send a placeholder
  `Bearer <api_key>`; preserving it meant upstream rejected every
  request. The forwarder is now the source of truth — any
  client-supplied Authorization is replaced with the real per-pod
  secret. Commit `0fc13f4`.
- **Forwarder no longer rewrites `Host` / `Origin`.** These must
  stay loopback for OpenClaw's silent-local-pairing path to fire.
  Commit `3db77be`.
- **Forwarder streams responses** instead of buffering the full
  body in memory before writing to the client. Browsers parse the
  bundle head while the tail is still on the wire; observed
  first-byte time dropped from ~130s to ~3s on cold-cache loads
  over `~5 KB/s` boringtun tunnels. Commit `ecd35da`.
- **Forwarder auto-invalidates stale config overlay.** The
  nemoclaw-configure.sh script regenerates `config.json` on every
  restart; the forwarder's overlay writer now uses
  `config.user.json` (deep-merged at restart) instead of mutating
  the regenerated file. Commit `1603167`.
- **`tytus ui` is production-ready as a daemon.** Detaches via
  `setsid`, ignores SIGHUP, survives Terminal close. Per-pod static
  asset cache at `/tmp/tytus/ui-<pod>-cache/` for instant reloads.
  Commits `ea5e0ba`, `ad176fd`, `e59782d`.
- **Forwarder prefetches the Vite chunk graph** after caching the
  main bundle, so dynamic imports don't blow up the tunnel with
  serial small requests. Commit `8b6cf10`.
- **Tray menu reflects state changes within ~1 second**, driven by
  a filesystem-signature watcher + action fan-out rather than pure
  polling. Commits `a7783da`, `d0e8836`.

### Fixed

- **Duplicate tunnel daemons.** Prevented at connect time via a
  pidfile pre-check; stale daemons mopped up after disconnect with
  a bounded iteration. Earlier this manifested as two boringtun
  instances fighting over the same WG socket; 2+ minute page load
  pathology. Commit `961676a`.
- **Doctor's tunnel check** now uses live pidfile + ps-p liveness
  rather than just state.json. Commit `1346dde`.
- **Three production-blockers** found during a sprint smoke test:
  racy tunnel teardown, leaked temp files under `$TMPDIR`, missing
  `Origin` rewrite on specific request paths. Commits `9554c14`,
  `603c333`.
- **Tray "Open in Browser"** reuses an existing forwarder instead
  of spawning a new one on port+1, and no longer pops a Terminal
  window on repeat clicks. Commits `708aeed`, `f772cd5`, `54f1885`.

### Shipped with (infrastructure)

Companion `wannolot-infrastructure` repo changes land in the same
deploy wave (push `main` → `production` on that repo to apply):

- `153e216` — hermes pod runs gateway (8642) + dashboard (9119) via
  both-servers entrypoint; DAM returns `ports.ui` alongside
  `ports.api` from `/agent/<N>/status`.
- `4c0021d` — hermes API_SERVER_KEY auto-derived from
  `sha256(AIL_API_KEY + TYTUS_POD_ID)[:48]` if not injected; written
  to `/app/workspace/.hermes/api_server_key` for the forwarder.
- `a3d4021` — switchailocal pin v0.4.0 → v0.4.1 (capability bridge
  fix).
- `5cd43f5` — switchailocal pin v0.3.1 → v0.4.0 (prerequisite of
  the above).

### Known gaps

- **LaunchAgent oneshot tunnel reap on cold boot** — see
  `docs/SPRINT-2026-04-19-negotiated.md` Phase 2. Workaround until
  fixed: manual `tytus connect --pod NN` after login, or
  `sudo -n tytus tunnel-up /tmp/tytus/tunnel-NN.json` from a shell.
  Planned fix is either `AbandonProcessGroup=true` on the plist or
  a dedicated `tytus tunnel-supervise` KeepAlive=true service.
- **Unsigned binaries** — Apple Developer enrollment is a
  prerequisite. Keychain ACL re-approval on every binary update is
  invisible to LaunchAgents and hits silent-failure cold-boot
  scenarios. Planned fix is Phase 3 of the negotiated sprint.
- **Cross-repo item** — Hermes gateway telemetry schema tracked as
  an issue in `traylinx/wannolot-provider` (see CHANGELOG cross-ref
  once filed).

### Upgrade notes

- Users running OpenClaw / Hermes pods allocated on **pre-v0.4.0
  tytus-hermes image** will hit forwarder-multiplex mismatches
  until the droplet rebuilds the image. After infra main →
  production promotion + `bootstrap/03-pull-images.sh`, restart
  existing hermes pods via `tytus restart --pod NN`.
- Existing users on unpatched v0.3.x should upgrade to v0.4.0 to
  pick up `is_logged_in` AT-only fallback before their next macOS
  reboot. Without the fix, keychain-slow cold boots silently fail
  autostart.

## [0.3.0] — 2026-04-13

Earlier work included; see `git log v0.2.0..v0.3.0 --oneline` for
commit-level detail. This file starts at v0.4.0 as the canonical
release log.

[0.4.0]: https://github.com/traylinx/tytus-cli/releases/tag/v0.4.0
[0.3.0]: https://github.com/traylinx/tytus-cli/releases/tag/v0.3.0

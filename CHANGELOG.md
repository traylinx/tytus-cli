# Changelog

All notable changes to the `tytus` CLI, `tytus-mcp` server, and
`Tytus.app` menu bar tray. Follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
conventions; versioning is [SemVer](https://semver.org/) — pre-1.0 minor
bumps are allowed to break compat.

## [Unreleased] — v0.5.2-alpha

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

## [Unreleased] — v0.5.1-alpha

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

## [Unreleased] — v0.5.0-alpha

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

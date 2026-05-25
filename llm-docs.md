# tytus-cli — LLM-facing reference


> You are an AI agent (Claude Code, OpenCode, KiloCode, Gemini CLI, etc.)
> running on a user's machine that has the `tytus` CLI installed. The user
> has a Traylinx subscription with a Tytus private AI pod entitlement. Your
> job is to drive `tytus` so the user can connect to their pod, run agents
> inside it, and call its OpenAI-compatible LLM gateway from local tools.

## 1. What is Tytus

Tytus is a **private AI pod product**. Each subscriber gets one or more
isolated pods that they reach via a **userspace WireGuard tunnel**. Inside
each pod runs an **agent container** (OpenClaw or Hermes
from Nous Research). Behind the agent is **SwitchAILocal**, an OpenAI-
compatible gateway that proxies to upstream providers (MiniMax today).

```
laptop ── WireGuard tunnel ── pod sidecar ── agent container
                                  └── SwitchAILocal (OpenAI-compatible)
                                        └── upstream LLM (MiniMax)
```

**No customer LLM traffic ever leaves the user's tunnel ↔ droplet path.**
Provider/Scalesys/Rails (the control plane) handle allocation and billing
only — they never see prompts or responses.

## 2. Names and concepts

| Term | Meaning |
|---|---|
| Tytus | Customer name for the private AI pod product |
| Traylinx | Platform brand (subscriptions, auth, billing) |
| Wannolot | Internal engineering codename |
| Pod | One user's isolated slice: WG sidecar + agent container |
| Agent | The AI runtime inside the pod (OpenClaw or Hermes) |
| Sidecar | The WireGuard container holding the netns |
| Unit | Resource accounting unit; agents have a unit cost |
| Plan | Subscription tier with a fixed unit budget |
| Stable URL | `http://10.42.42.1:18080` — constant per-droplet endpoint |
| Stable user key | `sk-tytus-user-<32hex>` — per-user, persistent across pods |


## 2b. TytusOS and tray desktop

TytusOS is the primary user-facing desktop served by the local tray daemon. It replaces visible Tower flows. Legacy Tower is only a hidden rollback path behind `TYTUS_ENABLE_LEGACY_TOWER=1`.

Use `tytus os-docs` for the bundled TytusOS manual. Key facts:

- TytusOS opens from the tray or `http://127.0.0.1:<tray-web-port>/`.
- Pod Inspector is the source of truth for readiness, actions, logs, env, and destructive confirmations.
- Files lands at `~/Tytus` and can browse local Tytus Home, shared folders, and pod workspaces.
- Terminal is host-backed and defaults to Tytus Home.
- Session expiry is fixed by `tytus login`; pods stay online. Never revoke pods to refresh a session.

### 2c. Documentation as skills for agentic apps

The local tray daemon exposes Tytus documentation as first-class skills so
Atomek and future agentic apps can load product context instead of relying on
stale model memory.

Available through the host API:

- `host.skills.list()`
- `host.skills.get(id)`
- `host.skills.resolve({ prompt })`

Same-origin HTTP mirrors:

- `GET /api/skills`
- `GET /api/skills/{id}`
- `POST /api/skills/resolve`
- `GET /api/resources` also includes each skill as `kind: "app-skill"` so
  Resource Fabric UIs can attach docs to a mission.

Built-in documentation skill ids:

| Skill id | Body |
|---|---|
| `tytus.docs.cli-reference` | Bundled `tytus llm-docs` reference for CLI, tray, MCP, gateway, Cortex, install, update, and troubleshooting |
| `tytus.docs.os-manual` | Bundled `tytus os-docs` user manual for TytusOS desktop, Pod Inspector, Chat, Files, Channels, Settings, shared folders, and app workflows |
| `tytus.docs.agentic-app-skills` | Short guide explaining how apps should resolve and attach Tytus docs/skills |

Agentic apps should resolve docs from the user's prompt, fetch the selected
skill body, and include it in the local app chat context before answering
Tytus product questions.

## 3. Plans and unit budgets

| Plan | Units |
|---|---|
| Explorer | 1 |
| Creator | 2 |
| Operator | 4 |

Agents cost units when allocated:

| Agent | Image | Cost | UI port | API port | Health path |
|---|---|---|---|---|---|
| OpenClaw | `tytus-openclaw` / legacy backend image | 1 unit | 3000 | 3000 | `/healthz` |
| Hermes | `tytus-hermes` | 2 units | 9119 | 8642 | `/health` |

`tytus connect --agent <name>` is rejected by the control plane if the
user would exceed their unit budget. The check is atomic in Scalesys
(`BEGIN IMMEDIATE` transaction).

**Two-port Hermes pods.** Hermes runs two HTTP servers inside the pod:
the gateway (port 8642, OpenAI-compatible `/v1/*` + `/api/jobs*`) and
the dashboard (port 9119, Vite/React management SPA with Config, Env,
Sessions, Skills, Logs, Cron, Analytics, Status pages). OpenClaw runs
one server that serves both UI and API on 3000. The tytus forwarder
**multiplexes** on hermes pods: requests matching `/v1/*`,
`/api/jobs*`, or `/health*` go to the gateway (:8642), everything else
goes to the dashboard (:9119). From the user's side it's still one
URL (`http://localhost:18700+pod_num/`).

## 4. Models on the SwitchAILocal gateway

The gateway model list is dynamic. Treat `/v1/models`, `tytus models`/MCP, or the global AIL route configuration as the source of truth. Apps must not hardcode provider model IDs.

Stable AIL aliases commonly exposed by the route include:

| Model id | Capabilities |
|---|---|
| `ail-compound` | default text / multimodal chat alias for the configured AIL route |
| `ail-image` | image generation alias for the configured AIL route |
| `ail-embed` | embeddings alias for the configured AIL route |

Provider-specific aliases may appear or disappear when AIL is reconfigured. Discover them at runtime.

## 5. The stable URL + stable user key

```bash
eval "$(tytus env --export)"
# Canonical names (AIL = the private AI gateway this CLI fronts):
# → AIL_URL=http://10.42.42.1:18080/v1
# → AIL_API_KEY=sk-tytus-user-<32hex>
# OpenAI-compatible aliases (required by Claude Code, Cursor, OpenCode,
# Continue, Aider — every tool that reads OPENAI_BASE_URL/OPENAI_API_KEY
# by convention; these always track AIL_* above):
# → OPENAI_BASE_URL=http://10.42.42.1:18080/v1
# → OPENAI_API_KEY=sk-tytus-user-<32hex>
# Anthropic-compatible aliases (Anthropic Python/TS/Ruby SDK, Claude
# Code with a custom base URL, Anthropic-native tooling). Note the
# BASE_URL has no /v1 suffix — the Anthropic SDK appends it itself:
# → ANTHROPIC_BASE_URL=http://10.42.42.1:18080
# → ANTHROPIC_API_KEY=sk-tytus-user-<32hex>
# Legacy TYTUS_* aliases kept for pre-sprint scripts:
# → TYTUS_AI_GATEWAY=http://10.42.42.1:18080
# → TYTUS_API_KEY=sk-tytus-user-<32hex>
# → TYTUS_AGENT_TYPE=<backend agent id>
# → TYTUS_POD_ID=02
```

`10.42.42.1` is a dual-bound WireGuard address present on every sidecar's
`wg0` interface. The user's tunnel adds it to the kernel routing table on
`tytus connect`. The address is constant across all pods and droplets, so
it never changes when Scalesys rotates the user's pod slot.

`sk-tytus-user-<32hex>` is a per-user key persisted in Scalesys's
`user_stable_keys` table. nginx on the droplet (in front of SwitchAILocal)
maps it via a `map` directive to the user's current real pod key. The
mapping is rebuilt by DAM (`/user-keys/sync`) on every allocation /
revocation, plus a 60-second periodic reconcile. The user never sees or
needs the real per-pod key.

`tytus env --raw` will print the per-pod values for debugging (URL like
`http://10.X.Y.1:18080`, key like `sk-<48 hex>`). These change on every
pod rotation, droplet migration, or octet reassignment.
**Do not use `--raw` values in user-visible config files** — they break
on the next pod rotation.

## 5b. Default pod (AIL gateway, zero units)

Every authenticated user has a **default pod** — an agent-less pod
allocated at `tytus login` that exists solely to route the WG tunnel so
the stable URL + stable key pair reach the droplet's SwitchAILocal
gateway. Properties:

- Costs **0 plan units** (independent of Explorer / Creator / Operator).
- `agent_type: "none"` in `state.json`.
- Allocated via `POST /pod/default` — idempotent, returns the same pod
  on every call until revoked.
- No agent container: the sidecar's built-in socat forwarder on
  `10.42.42.1:18080` is what serves the AIL gateway.
- Survives `tytus agent uninstall` on any pod: uninstalling an agent
  stops the container but keeps the slot, so even a single-slot plan
  never loses AIL access when swapping agents.

When users ask "how do I just call your models without setting up an
agent", the answer is: they already can — right after `tytus login`,
the default pod + stable env pair from §5 just work.

## 5c. Zero-config auth — no token pasting, ever

Both agent types use a **deterministic per-pod shared secret** the CLI
forwarder injects automatically. Users never see or paste a token.

| Agent | Secret name | Derivation | Where it's read |
|---|---|---|---|
| OpenClaw | `gateway.auth.token` | generated during OpenClaw setup from `sha256(AIL_API_KEY + TYTUS_POD_ID)[:48]`; written into `config.json` | `/app/workspace/.openclaw/config.json` |
| hermes | `API_SERVER_KEY` | same formula, set as env var by `hermes/entrypoint.sh` when not injected externally | `/app/workspace/.hermes/api_server_key` |

The forwarder (`tytus ui --pod NN`, or the tray's "Open in Browser"
action) fetches this secret via Provider's `/pod/agent/exec` using A2A
creds from `state.json` (secret_key + agent_user_id — no keychain
round-trip needed) and stashes it in `state.json`'s `gateway_token`
field. On every proxied request, the forwarder **overrides** any
client-side Authorization header with `Bearer <gateway_token>` — so
SDK placeholders like `OpenAI(api_key="any-string")` work out of the
box on hermes pods, and OpenClaw silent-local-pairing fires on
OpenClaw without the browser ever seeing the token form.

## 5d. Agent config overlays (survive container restart)

Both agents support a deep-merge overlay file next to their generated
config. On every container restart the auto-generator regenerates the
base config, then merges the overlay on top. Add only the fields you
want to change.

| Agent | Base (regenerated) | User overlay | Format |
|---|---|---|---|
| OpenClaw | `/app/workspace/.openclaw/config.json` | `/app/workspace/.openclaw/config.user.json` | JSON |
| hermes | `/app/workspace/config.yaml` | `/app/workspace/config.user.yaml` | YAML |

Precedence: **overlay wins on conflicts** for scalars; for arrays and
maps, it's a recursive deep-merge (maps merge key-wise, arrays are
replaced wholesale — not appended).

The CLI writes `config.user.json` for OpenClaw at agent-install time
to add the forwarder's `http://localhost:18700+N` origin to
`gateway.controlUi.allowedOrigins` so the browser's WS upgrade passes
origin-check AND satisfies silent-local-pairing (requires loopback
`Host` + `Origin`). For hermes, no CLI-side overlay is needed today
— configure with `tytus configure` or edit `config.user.yaml` directly
to customize `model.default`, `terminal.timeout`, etc.

## 5e. `is_logged_in` semantics

A user is "logged in" (`CliState::is_logged_in` returns true) when
their state has an email AND **either** a refresh token **or** a
currently-valid access token. This matters at macOS cold boot: the
keychain ACL can take a few seconds to approve after login, and the
`get_refresh_token` call times out in 3s. The autostart LaunchAgent
falls through to AT-only mode — the existing access token keeps pod
ops working while the daemon retries the keychain in the background.
`tytus doctor` surfaces RT presence distinctly, so a user with
AT-only coverage sees a warning ("keychain not readable — re-run
`tytus login` if this persists") rather than a red fail.

## 6. Full command reference

```text
tytus login                        Browser device-auth via Sentinel.
                                   Stores access_token + refresh_token in
                                   the OS keychain and ~/.config/tytus/state.json.

tytus logout                       Revoke all pods + clear local state +
                                   delete keychain entries.

tytus status [--json]              Plan, pods, units, tunnel state.
                                   Default = human; --json = machine.

tytus doctor [--pod NN]            Without --pod: full daemon-wide
                                   diagnostic (state file, logged_in,
                                   token_valid, subscription, pods,
                                   tunnel, mcp_server). Some checks may
                                   fail before connect — that's normal.
                                   With --pod NN: per-pod diagnostic —
                                   container status, healthy flag,
                                   uptime, image, and ports for that
                                   pod (proxies via Provider →
                                   DAM /agent/<pod_num>/status). One
                                   fact per stdout line so the tray's
                                   SSE relay can surface each as a
                                   discrete `log` event in the Pod
                                   Inspector Doctor pane.

tytus setup                        Interactive wizard: login (if needed),
                                   plan check, agent pick, allocation,
                                   tunnel, sample chat. Use this for
                                   first-run experiences.

tytus connect [--pod NN] [--agent openclaw|hermes]
                                   No flags: bring the tunnel up to the
                                   user's default pod (agent-less, 0
                                   units, always available). Allocates
                                   the default pod on-the-fly if login
                                   didn't (rare race).
                                   --pod NN: connect to that specific pod.
                                   --agent X: deprecated shim — equivalent
                                   to `tytus agent install X && tytus
                                   connect --pod <new>`. The tunnel is
                                   activated via elevation (osascript /
                                   sudo -n / interactive sudo); the daemon
                                   writes its PID to /tmp/tytus/tunnel-NN.pid.

tytus agent install <name> [--pod NN] [--force]
                                   Install an agent runtime (openclaw,
                                   hermes, …). Without --pod: allocate a
                                   new pod slot and deploy the agent in
                                   one shot (costs plan units per the
                                   catalog). With --pod: deploy into that
                                   existing slot; --force replaces an
                                   existing agent on the slot.

tytus agent uninstall <pod>        Stop + remove the agent container. The
                                   pod slot stays allocated so AIL keeps
                                   working through it; use `tytus revoke`
                                   to fully free units.

                                   There is no `tytus agent replace`. If
                                   a user wants a different agent on a
                                   slot, they `tytus revoke <pod>` and
                                   `tytus agent install <new>` — add and
                                   delete, never switch.

tytus agent list [--json]          Print all pods (default + agent-bearing)
                                   with agent + tunnel status.

tytus agent catalog [--refresh] [--json]
                                   Fetch the installable-agent catalog
                                   from Provider. Cached locally for 5
                                   minutes; --refresh forces a live
                                   fetch. Works offline against the
                                   stale cache.

tytus disconnect [--pod NN]        Read the PID file, send SIGTERM to the
                                   tunnel daemon. Allocation is preserved
                                   in Scalesys — `tytus connect` brings
                                   the same pod back without spending units.

tytus revoke <pod_id>              DESTRUCTIVE. Free the units in Scalesys
                                   AND tell DAM to wipe the workspace
                                   state directory + container. Cannot be
                                   undone. Confirm with the user first.

tytus restart [--pod NN]           Restart the agent container via DAM.
                                   Re-runs the entry script which
                                   regenerates the base config and merges
                                   the user overlay file. Useful after
                                   editing config.user.json or .yaml.

tytus logs [--pod NN] [--lines N]  Tail the last N (default 200, max 500)
                                   lines of the agent container's
                                   stdout/stderr. Proxies to DAM's
                                   `/agent/<pod_num>/logs?tail=N`. Output
                                   streams one line at a time so the
                                   tray's SSE relay (POST `/api/pod/<NN>/
                                   run-streamed` `{action:"logs"}`) can
                                   surface each log line as a discrete
                                   `log` event in the Pod Inspector
                                   Logs tab.

tytus env [--export] [--raw] [--pod NN] [--json]
                                   Default: stable values
                                   (10.42.42.1 + sk-tytus-user-*).
                                   --export: shell-sourceable.
                                   --raw: per-pod legacy values.
                                   --json: full pod state as JSON.

tytus test                         End-to-end health: auth, pod, tunnel,
                                   gateway, sample chat. Print "Everything
                                   is working!" on success.

tytus capabilities [--pod NN]      Discover the pod gateway's model
                                   catalog + provider-native tools (e.g.
                                   upstream autonomous web_search).
                                   Reads GET /v1/models using the same
                                   stable endpoint/key pair `tytus env`
                                   emits, so it works over the WG tunnel
                                   OR the public-edge path.
                                   Default output: human tree showing
                                   `id`, upstream name, context window,
                                   and a ↳ native: <type> — <description>
                                   subtree per provider-native tool.
                                   --json (global): passthrough the raw
                                   /v1/models body verbatim — use this
                                   when an AI agent wants to splice
                                   discovered native_tools entries into
                                   its own tools[] at chat-completion
                                   time.

tytus chat [--model ail-compound]  Interactive REPL against the pod gateway.

tytus exec [--pod NN] [--timeout N] "<command>"
                                   Run a shell command inside the agent
                                   container via DAM. Max timeout 120s.

tytus configure                    Interactive overlay editor. Walks
                                   through agent config knobs and writes
                                   ~/.tytus or the agent's config.user.*
                                   overlay file.

tytus link [DIR] [--only ...]      Link a project to Tytus — drops AI
                                   integration files into a project:
                                   CLAUDE.md, AGENTS.md, .claude/commands/
                                   tytus.md, .mcp.json, .kilo/, .archon/,
                                   shell hook. Filter with --only claude|
                                   agents|kilocode|opencode|archon|shell.
                                   Aliased as `tytus infect` for backwards
                                   compatibility.

tytus mcp [--format claude|kilocode|opencode|archon|json]
                                   Print an MCP server config stanza for
                                   the chosen AI tool. Stick it into the
                                   tool's mcp.json (or use `tytus link`
                                   which does it for you).

tytus bootstrap-prompt             Print a one-liner you can paste into
                                   any AI tool (Claude Code, OpenCode,
                                   Cursor, etc.) to teach it how to drive
                                   Tytus natively — it references the
                                   hosted SKILL.md on GitHub.

tytus autostart install             Install two LaunchAgents (macOS) /
                                   systemd user units (Linux):
                                   (a) `tytus connect` at every login —
                                       brings the WireGuard tunnel up.
                                   (b) `tytus daemon run` keep-alive —
                                       continuously refreshes the Sentinel
                                       access + refresh tokens so the RT
                                       never expires server-side (normally
                                       ~24h TTL). Survives crashes via
                                       KeepAlive / Restart=always.
                                   Sets TYTUS_HEADLESS=1 so neither path
                                   opens a browser.

tytus autostart uninstall          Remove both LaunchAgents / user units.

tytus autostart status             Check if the autostart hooks are
                                   installed and loaded.

tytus daemon run                   Run the token-refresh daemon in the
                                   foreground. Called by launchd / systemd.
                                   Listens on /tmp/tytus/daemon.sock for
                                   status + shutdown commands from the CLI
                                   and the tray.

tytus daemon stop                  Send SHUTDOWN to a running daemon.

tytus daemon status                Query the daemon over its Unix socket.

tytus tray install                 macOS only. Creates /Applications/Tytus.app
                                   (a proper LSUIElement=true menu-bar app
                                   bundle with an icon.icns) and registers
                                   a `com.traylinx.tytus.tray` LaunchAgent
                                   so the tray auto-starts at every login.
                                   Also pokes LaunchServices so Spotlight
                                   picks the bundle up immediately.

tytus tray uninstall               Remove /Applications/Tytus.app and the
                                   tray LaunchAgent.

tytus tray status                  Show install / load / running state.

tytus tray start                   Open /Applications/Tytus.app (or fall
                                   back to ~/bin/tytus-tray). Useful from
                                   scripts after a `quit`.

NOTE — TytusOS in-page actions (tray-driven). When the tray is running,
most non-interactive menu items (Run Health Test, Doctor, per-pod
Restart / Uninstall / Revoke / Stop forwarder, Channels catalog, Add
channel, Files, Settings) deep-link the user's browser into TytusOS at
`http://127.0.0.1:<port>/#/<route>` and stream subprocess output there
via SSE. Sudo-bearing commands (`tytus connect`, `tytus tray install`),
browser-auth flows (`tytus login`), and interactive wizards that need a
real TTY may still open a native terminal. TytusOS routes to know:

  #/run/test                       — global health probe
  #/run/doctor                     — daemon/pod diagnostics
  #/channels                       — channel binding UI
  #/files                          — Files app, Tytus Home + pods + shared folders
  #/settings/daemon                — daemon/session status and re-auth card
  #/settings/sharing               — account-scoped shared folder controls
  #/pod/<NN>                       — Pod Inspector detail tab
  #/pod/<NN>/<action>              — restart | revoke | uninstall |
                                     stop-forwarder, streamed through the
                                     tray backend with one active job per pod.

Legacy Tower may still be present only as rollback when
`TYTUS_ENABLE_LEGACY_TOWER=1`; do not direct users there in normal docs
or support flows.
tytus ui [--pod NN] [-P PORT] [--no-open]
                                   Start a 127.0.0.1 → pod agent TCP
                                   forwarder so the browser sees the
                                   OpenClaw / Hermes UI on localhost
                                   (WebCrypto + Service Workers require
                                   HTTPS or localhost). Runs detached
                                   when spawned by the tray — survives
                                   closing any Terminal. Reuses an
                                   existing forwarder if one is already
                                   live for the same pod (marker file
                                   `/tmp/tytus/ui-<pod>.port`). Auto-
                                   swaps the tunnel if it isn't routing
                                   to the requested pod. An internal
                                   5-second upstream probe shuts the
                                   forwarder down after 15s of
                                   unreachable upstream (tunnel dropped).

                                   DEFAULT PORT: 18700 + pod_num, so
                                   pod 01 → http://localhost:18701/,
                                   pod 02 → http://localhost:18702/, …
                                   Bookmarkable and stable. If that
                                   exact port is taken it retries
                                   +100/+200/+300/+400 before falling
                                   back to a kernel-ephemeral port.
                                   Override with `-P <port>` if you
                                   really want a specific value.

tytus ui --stop [--pod NN]         SIGTERM a running UI forwarder.
                                   Without --pod, stops every one.
                                   Cleans stale markers too.

tytus llm-docs                     Print THIS document.
```

**Global flags:**

| Flag | Env var | Effect |
|---|---|---|
| `--json` | — | Machine-readable JSON output on all commands |
| `--headless` | `TYTUS_HEADLESS=1` | Force non-interactive mode. Disables browser device-auth, logs diagnostics to `/tmp/tytus/autostart.log`. Use in LaunchAgents, cron, CI. |

### 6b. Local Cortex (opt-in, current public beta)

By default, chat with a pod routes through the cloud Cortex on Strato. Users
who want chat memory to stay on their Mac can run Cortex locally instead.
Cloud remains the default for everyone; nothing flips until the user
explicitly switches in Settings → AI or via `tytus cortex up`.

All commands below speak the same `--json` and `--headless` conventions.
None of them call out to the network on their own — they shell out to
Docker on the user's machine, plus HTTP to `127.0.0.1:8098`.

```bash
tytus cortex up [--port N] [--pin TAG]    # install + start the local stack
tytus cortex down [--purge]               # stop containers; --purge wipes volumes
tytus cortex status [--json]              # state + container + /health/live probe
tytus cortex test [MESSAGE]               # probe message + latency report
tytus cortex reset --yes                  # full factory reset (destructive)
tytus cortex token rotate                 # mint a new per-user ctx_* token
tytus cortex token show                   # token presence + prefix (never the body)
tytus cortex logs [--tail N] [--follow]   # `docker compose logs` tail
tytus cortex upgrade                      # pull pinned image + run migrations
tytus cortex version                      # show pinned image tag
```

**Two-token model — don't mix them up:**

- `INTERNAL_SERVICE_TOKEN` (state.json::`cortex_internal_service_token`) is
  the service-to-service shared secret between the tray daemon and local
  Cortex. The daemon presents it on `/tytus/chat` calls. Never user-visible.
- `ctx_*` user token (state.json::`cortex_local_token`) is for `/v1/*`
  user-scoped endpoints (memory search, session list, profile). Minted by
  `POST /v1/users` during `tytus cortex up`.

Mixing them is a 401 trap.

**Hard rules for AI agents driving local Cortex:**

- Never run `tytus cortex down --purge` or `tytus cortex reset` without
  explicit user confirmation. Both wipe the user's memory store.
- `tytus cortex up` is idempotent — safe to retry on failure.
- If `tytus cortex status` returns `"docker_status": "Unavailable"`, the
  user has not started Docker Desktop. Tell them, don't retry blindly.
- Cortex has no public memory-write endpoint. Memories are populated
  implicitly via chat. Do not promise users an explicit "save fact" verb.


## Launch user-manual quick answers

Use these answers when a user asks in chat, TytusOS Help, or an AI CLI how to use Tytus.

### Install

macOS/Linux:

```bash
curl -fsSL https://get.traylinx.com/install.sh | bash
```

Windows:

```powershell
powershell -c "irm https://get.traylinx.com/install.ps1 | iex"
```

Homebrew:

```bash
brew install traylinx/tap/tytus
```

### First run

```bash
tytus setup
tytus login
tytus connect
tytus test
tytus os
```

### Stable SDK config

```bash
eval "$(tytus env --export)"
# OPENAI_BASE_URL=http://10.42.42.1:18080/v1
# OPENAI_API_KEY=sk-tytus-user-<32hex>
```

Prefer the stable user key and gateway. Do not ask users to paste raw per-pod keys unless debugging.

### Pods and units

- Included gateway/no-agent pod: 0 units.
- OpenClaw: 1 unit.
- Hermes: 2 units.
- A reserved/free pod is not broken; it is available capacity.
- Custom names come from `/pod/status.display_name`. If names differ between web, tray, and TytusOS, refresh state before changing anything.

### Channels

Current OpenClaw-backed channel flows: Telegram, Discord bot, Slack Socket Mode. Other messengers may require manual/custom bridge work or future support. Do not claim broad native Discord/Slack/Hermes channel automation unless the current UI/release proves it.

### Local Cortex

Cloud Cortex is default. Local Cortex is opt-in via `tytus cortex up` and stores memory in local Docker Postgres/Redis. Before recommending it publicly, verify the GHCR image is public:

```bash
docker manifest inspect ghcr.io/traylinx/tytus-cortex:2026-05-17 >/dev/null
```

### Updates

If the tray shows update available, use **Check for Updates** / **Update Tytus** to start the explicit update flow (not silent auto-install), or:

```bash
tytus update
tytus --version
tytus doctor
```

If old builds lack `tytus update`, reinstall from `https://get.traylinx.com/`.

### Troubleshooting order

1. `tytus status --json`
2. `tytus doctor`
3. `tytus test`
4. TytusOS Settings -> Daemon
5. Pod Inspector readiness
6. Browser console only after daemon/pod state is known

Never fix missing pods by deleting pods first. Never fix session expiry by revoking capacity.

## 7. MCP tools (when the MCP server is wired up)

The `tytus` CLI ships a sister binary `tytus-mcp` that speaks JSON-RPC 2.0
over stdio. It exposes these tools:

| Tool | Args | Returns |
|---|---|---|
| `tytus_status` | none | Login state, plan, pods, units, tunnel state |
| `tytus_env` | `pod_id?` | Stable + raw connection details |
| `tytus_models` | none | Live model list from the pod gateway |
| `tytus_chat` | `model`, `messages` | Chat completion (proxied through pod) |
| `tytus_revoke` | `pod_id` | Free pod units (destructive — confirm) |
| `tytus_setup_guide` | none | What to tell the user when nothing is connected |
| `tytus_docs` | none | This LLM-facing reference |

Always call `tytus_status` first in any new conversation to find out
what the user actually has, then branch based on that.

## 8. Standard recipes

### Recipe A — Make sure the user has a working pod, then chat
```bash
tytus status --json | jq -e '.pods | length > 0' \
    || tytus connect --agent openclaw
tytus test                                              # confirm green
eval "$(tytus env --export)"                            # load stable pair
curl -sS "$OPENAI_BASE_URL/chat/completions" \
    -H "Authorization: Bearer $OPENAI_API_KEY" \
    -H "Content-Type: application/json" \
    -d '{"model":"ail-compound","messages":[{"role":"user","content":"hi"}]}'
```

### Recipe B — Switch a pod from OpenClaw to Hermes
```bash
tytus disconnect --pod 02       # tear down tunnel only (allocation kept)
tytus revoke 02                 # free the units (destroys workspace)
tytus connect --agent hermes    # allocate fresh hermes (2 units)
tytus test
```

### Recipe C — Inspect or edit the agent's overlay config
```bash
# Read the example template
tytus exec --pod 02 "cat /app/workspace/.openclaw/config.user.json.example"

# Write an overlay (deep-merged on top of the base config at restart)
tytus exec --pod 02 "cat > /app/workspace/.openclaw/config.user.json <<JSON
{ \"agents\": { \"defaults\": { \"contextTokens\": 64000, \"timeoutSeconds\": 300 } } }
JSON"

# Apply
tytus restart --pod 02
```

### Recipe D — Use the pod from Cursor / Claude Desktop / OpenCode
```bash
tytus connect                    # one-time
tytus env --export               # see what to paste
# Then in the tool's settings:
#   OPENAI_BASE_URL = http://10.42.42.1:18080/v1
#   OPENAI_API_KEY  = sk-tytus-user-<32hex>
# These never change. Set once, forget forever.
```

### Recipe E — Debug "the tunnel is up but my chat returns 401"
```bash
tytus doctor                                 # quick health check
tytus test                                   # E2E sanity
# If tytus test fails on "AI gateway":
ssh root@<droplet-ip> "tail -20 /var/log/nginx/switchailocal-lb.log"
ssh root@<droplet-ip> "cat /etc/nginx/maps/user-keys.map"
# If the user-keys map doesn't have your stable key:
ssh root@<droplet-ip> "curl -X POST -H 'X-Scalesys-Token: ...' http://localhost:8099/user-keys/sync"
# OR just:
tytus restart                                # triggers DAM sync as a side effect
```

### Recipe F — Call AIL without installing an agent
Users on the free tier (or who just want raw gateway access) don't have
to spend a unit on OpenClaw / Hermes — the default pod covers this.
```bash
tytus login          # provisions the default pod automatically
tytus connect        # no --agent: brings the tunnel up to the default pod
eval "$(tytus env --export)"
curl -sS "$AIL_URL/chat/completions" \
    -H "Authorization: Bearer $AIL_API_KEY" \
    -H "Content-Type: application/json" \
    -d '{"model":"ail-compound","messages":[{"role":"user","content":"hi"}]}'
```
Install an agent later — units are only spent when the user actually
wants one: `tytus agent install openclaw`.

## 9. Error catalog

| Message | Cause | Fix |
|---|---|---|
| `No pods. Run: tytus connect` | No allocation | `tytus connect` (or `tytus setup`) |
| `Tunnel daemon already running` | Stale PID file from previous session | `tytus disconnect` then retry |
| `Pod config not ready` | peer.conf missing on droplet | Backend issue — escalate, do not loop |
| `403 plan_limit_reached` from Scalesys | Unit budget would be exceeded | Tell user to upgrade or revoke an existing pod |
| `401 Invalid API key` from gateway | Stable map sync race; or wrong key; or revoked pod | Wait 2s and retry; check `tytus env`; check `tytus status` |
| `503 no_capacity` from Provider | All droplets full | Backend issue — Scalesys will auto-provision or escalate |
| `Allocation failed` (unspecific) | Network or auth | `tytus doctor` first |
| `Token refresh failed: AuthExpired` | Refresh token expired or revoked | `tytus login` from an interactive terminal |
| `Cannot open browser for login in non-interactive context` | Headless mode blocked device auth | `tytus login` interactively, then `tytus autostart install` |
| `No refresh token available` | Fresh state or state was cleared | `tytus login` from an interactive terminal |
| `Docker CLI not found or not executable` (cortex) | Docker Desktop not installed or not on PATH | Install from docker.com; restart shell; retry |
| `Docker daemon not reachable` (cortex) | Docker Desktop installed but not running | Open Docker Desktop, wait for whale icon, retry |
| `Cortex did not become healthy within 90s` | Image pull slow, or SwitchAILocal unreachable from container | `tytus cortex logs` to investigate; verify host SwitchAILocal binds 127.0.0.1:18080 |
| `cortex_not_local` (HTTP 503) | App called `/api/cortex/memory/*` while profile=cloud | Switch profile to local in Settings → AI, or accept that the API is no-op on cloud |
| `cortex_token_missing` (HTTP 503) | Local stack running but no ctx_* token minted | `tytus cortex token rotate` |
| `cortex_unreachable` (HTTP 502 from tray) | Local Cortex stopped or crashed | `tytus cortex status` then `tytus cortex up` if needed |

## 10. Hard rules for AI agents

1. **Never invent models.** Query the live gateway model list (`/v1/models`, MCP `tytus_models`, or global AIL config). Treat `ail-compound`, `ail-image`, and `ail-embed` as stable aliases, but never hardcode provider-specific model ids in apps.
2. **Never hardcode `10.18.X.Y` IPs.** They change. Use `10.42.42.1`.
3. **Never paste raw per-pod keys into source files.** Read from
   `tytus env` at runtime.
4. **Treat `tytus revoke` and `tytus logout` as destructive.** Always
   confirm with the user before running them.
5. **Never call `sudo` to manipulate the tunnel directly.** `tytus connect`
   handles privilege escalation through its built-in chain.
6. **Read connection vars freshly** at the start of any session — if
   another process revoked or rotated the pod, the cached value is wrong.
7. **`tytus llm-docs` is the source of truth.** When in doubt, re-read it.
8. **Prefer `tytus` commands over raw curl.** The CLI knows the stable
   endpoint, the agent type, and the current state.

## 11. State and storage

- Client state file: `~/Library/Application Support/tytus/state.json`
  (macOS) or `~/.config/tytus/state.json` (Linux). Mode 0600. Contains
  email, refresh_token, access_token, secret_key, agent_user_id,
  organization_id, tier, and the pods array (with stable_user_key).
- Tunnel daemon PIDs: `/tmp/tytus/tunnel-NN.pid`
- Diagnostic log: `/tmp/tytus/autostart.log` (timestamped entries from
  headless mode — token refresh results, startup state, tunnel success/failure)
- OS keychain: refresh_token (cross-tool compatibility)

## 12. What's deliberately NOT exposed

These exist on the backend but are not visible to the user or to you:

- The `SCALESYS_SECRET` shared between control-plane services
- The upstream provider keys (MiniMax, OpenAI)
- The other users' pods, keys, or state
- The droplet's SSH credentials
- The `AIL_POD_KEY_NN` per-pod keys (unless you explicitly ask for
  `--raw`, and even then only your own pod's key)

These are control-plane secrets. Asking for them is a bug.

## 13. End

If you need anything not in this document, run:

```bash
tytus --help
tytus <subcommand> --help
```

The CLI is the source of truth for argument shapes; this document is the
source of truth for product behavior, names, models, and recipes.
## Multi-account Path A (2026-04-30)

Tytus now supports multiple stored Traylinx accounts with one active account at a time:

```bash
tytus account list
tytus account add
tytus account switch work@example.com
tytus account current
tytus account remove work@example.com --force
tytus mcp --account work@example.com --format claude
```

Notes:
- `tytus login` remains an alias for the browser device-auth flow.
- `tytus account remove` is local-only: it deletes local state/keychain entries and never calls Provider revoke. Use `tytus logout` to revoke active account pods server-side.
- Path A is single-active-account. `/tmp/tytus/*`, the daemon socket, tray instance, and tray web port remain singleton resources.
- MCP configs generated with `--account` set `TYTUS_PINNED_ACCOUNT_EMAIL`; the MCP server refuses tool calls if the long-lived process drifts from the pinned account after a switch.

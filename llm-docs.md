# tytus-cli — LLM-facing reference


> You are an AI agent (Claude Code, OpenCode, KiloCode, Gemini CLI, etc.)
> running on a user's machine that has the `tytus` CLI installed. The user
> has a Traylinx subscription with a Tytus private AI pod entitlement. Your
> job is to drive `tytus` so the user can connect to their pod, run agents
> inside it, and call its OpenAI-compatible LLM gateway from local tools.

## 1. What is Tytus

Tytus is a **private AI pod product**. Each subscriber gets one or more
isolated pods that they reach via a **userspace WireGuard tunnel**. Inside
each pod runs an **agent container** (OpenClaw + NemoClaw sandbox, or Hermes
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
| Agent | The AI runtime inside the pod (nemoclaw or hermes) |
| Sidecar | The WireGuard container holding the netns |
| Unit | Resource accounting unit; agents have a unit cost |
| Plan | Subscription tier with a fixed unit budget |
| Stable URL | `http://10.42.42.1:18080` — constant per-droplet endpoint |
| Stable user key | `sk-tytus-user-<32hex>` — per-user, persistent across pods |

## 3. Plans and unit budgets

| Plan | Units |
|---|---|
| Explorer | 1 |
| Creator | 2 |
| Operator | 4 |

Agents cost units when allocated:

| Agent | Image | Cost | Gateway port | Health path |
|---|---|---|---|---|
| nemoclaw | `tytus-nemoclaw:latest` (OpenClaw + NemoClaw blueprint) | 1 unit | 3000 | `/healthz` |
| hermes | `tytus-hermes:latest` (Nous Research) | 2 units | 8642 | `/health` |

`tytus connect --agent <name>` is rejected by the control plane if the
user would exceed their unit budget. The check is atomic in Scalesys
(`BEGIN IMMEDIATE` transaction).

## 4. Models on the SwitchAILocal gateway

| Model id | Backed by | Capabilities |
|---|---|---|
| `ail-compound` | MiniMax M2.7 | text, vision, audio (default chat model) |
| `minimax/ail-compound` | MiniMax M2.7 | text |
| `ail-image` | MiniMax image-01 | image generation |
| `minimax/ail-image` | MiniMax image-01 | image generation |
| `ail-embed` | mistral-embed (via SwitchAI) | embeddings |

These are **all** the models available. There is no `gpt-4`, no `claude-*`,
no `qwen3-8b` — do not invent models.

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
# → TYTUS_AGENT_TYPE=nemoclaw
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

## 6. Full command reference

```text
tytus login                        Browser device-auth via Sentinel.
                                   Stores access_token + refresh_token in
                                   the OS keychain and ~/.config/tytus/state.json.

tytus logout                       Revoke all pods + clear local state +
                                   delete keychain entries.

tytus status [--json]              Plan, pods, units, tunnel state.
                                   Default = human; --json = machine.

tytus doctor                       Full diagnostic: state file,
                                   logged_in, token_valid, subscription,
                                   pods, tunnel, mcp_server. Some checks
                                   may fail before connect — that's normal.

tytus setup                        Interactive wizard: login (if needed),
                                   plan check, agent pick, allocation,
                                   tunnel, sample chat. Use this for
                                   first-run experiences.

tytus connect [--pod NN] [--agent nemoclaw|hermes]
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
                                   Install an agent runtime (nemoclaw,
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

tytus env [--export] [--raw] [--pod NN] [--json]
                                   Default: stable values
                                   (10.42.42.1 + sk-tytus-user-*).
                                   --export: shell-sourceable.
                                   --raw: per-pod legacy values.
                                   --json: full pod state as JSON.

tytus test                         End-to-end health: auth, pod, tunnel,
                                   gateway, sample chat. Print "Everything
                                   is working!" on success.

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
    || tytus connect --agent nemoclaw
tytus test                                              # confirm green
eval "$(tytus env --export)"                            # load stable pair
curl -sS "$OPENAI_BASE_URL/chat/completions" \
    -H "Authorization: Bearer $OPENAI_API_KEY" \
    -H "Content-Type: application/json" \
    -d '{"model":"ail-compound","messages":[{"role":"user","content":"hi"}]}'
```

### Recipe B — Switch a pod from nemoclaw to hermes
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
to spend a unit on NemoClaw / Hermes — the default pod covers this.
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
wants one: `tytus agent install nemoclaw`.

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

## 10. Hard rules for AI agents

1. **Never invent models.** Only `ail-compound`, `ail-image`, `ail-embed`,
   `minimax/ail-compound`, `minimax/ail-image` exist. If the user asks for
   another model, say it's not available on this pod.
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

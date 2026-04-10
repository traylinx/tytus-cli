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
# → OPENAI_BASE_URL=http://10.42.42.1:18080/v1
# → OPENAI_API_KEY=sk-tytus-user-<32hex>
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
`http://10.18.2.1:18080`, key like `<REDACTED_PER_POD_KEY_PREFIX>...`). These change.
**Do not use `--raw` values in user-visible config files** — they break
on the next pod rotation.

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
                                   Allocate (or reuse) a pod, deploy the
                                   agent if needed, elevate (osascript /
                                   sudo -n / interactive sudo), spawn the
                                   tunnel daemon, return immediately. The
                                   daemon writes its PID to
                                   /tmp/tytus/tunnel-NN.pid.

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

tytus llm-docs                     Print THIS document.
```

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

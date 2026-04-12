---
name: tytus
description: Use `tytus` by Traylinx — a CLI that gives you on-demand access to the user's private AI pod (a WireGuard-tunneled, OpenAI-compatible LLM gateway running on their Traylinx subscription). Handles auth, pod allocation, tunnel, agent lifecycle, and a stable URL/key pair for local tools.
---

# tytus — Agent Instructions

You have access to **Tytus by Traylinx** via the `tytus` CLI on the user's machine. Tytus is a **private AI pod** product: each subscriber gets their own isolated pod they reach through a WireGuard tunnel, with an OpenAI-compatible LLM gateway inside. You drive it through the CLI.

**Always prefer `tytus` commands over raw curl or hand-crafted network calls.** The CLI knows the current state, the stable endpoint, the per-user key, and handles tunnel elevation transparently.

---

## Step 0 — Make sure `tytus` is installed

```bash
command -v tytus >/dev/null && tytus --version
```

If the command is missing, install it:

```bash
curl -fsSL https://get.traylinx.com/install.sh | bash
```

The installer detects the OS, installs `tytus` + `tytus-mcp` to `~/.local/bin` (or `/usr/local/bin` with `sudo`), and verifies the install. After it finishes, **tell the user** to run `tytus setup` for the interactive first-run wizard (browser login, pod allocation, tunnel, sample chat) — or you can drive those steps yourself with the commands below.

---

## Step 1 — Load the full reference

```bash
tytus llm-docs
```

That command prints the canonical LLM-facing reference (~320 lines of structured Markdown): every subcommand, the fixed model catalog, plan tiers, agent types, standard recipes, error catalog, and hard rules. **Cache its output in your context for the rest of the session.** It is the source of truth for product behavior — this SKILL.md is the bootstrap document.

---

## Step 2 — Check what the user has

```bash
tytus status --json
```

Interpret the response:
- `logged_in: false` → run `tytus login` (opens browser to `sentinel.traylinx.com`) or guide the user through `tytus setup`.
- `logged_in: true, pods: []` → the user has a plan but no active pod. Run `tytus connect [--agent nemoclaw|hermes]` to allocate one.
- `logged_in: true, pods: [...]` → the user has at least one pod. Check `tunnel_iface` to see which (if any) are connected.

Also run `tytus doctor` any time anything feels off — it checks state file, auth, subscription, tunnel, and MCP server.

---

## Step 3 — Get the stable connection pair

After at least one pod is connected:

```bash
eval "$(tytus env --export)"
echo "$OPENAI_BASE_URL"   # → http://10.42.42.1:18080/v1     (constant forever)
echo "$OPENAI_API_KEY"    # → sk-tytus-user-<32hex>            (stable per user)
```

**These are the only values you should ever paste into a user-visible config file.** They survive pod revoke/reallocate, agent swaps, and droplet migration. The legacy per-pod values (URL like `http://10.18.X.Y:18080`, key like `sk-c939...`) are behind `tytus env --raw` and should only be used for debugging.

---

## Product facts (do not guess, do not invent)

### Plans and unit budgets
| Plan | Unit budget |
|---|---|
| Explorer | 1 |
| Creator | 2 |
| Operator | 4 |

### Agents (runnable INSIDE a pod via `tytus connect --agent <name>`)
| Agent | Cost | Gateway port | Description |
|---|---|---|---|
| `nemoclaw` | 1 unit | 3000 | OpenClaw runtime with the NemoClaw sandboxing blueprint |
| `hermes`   | 2 units | 8642 | Nous Research Hermes gateway |

### Models on the pod gateway (SwitchAILocal)
These are the **only** models available. Do not pass any other model id — it will fail.

| Model | Backed by | Capabilities |
|---|---|---|
| `ail-compound` | MiniMax M2.7 | text, vision, audio (default chat) |
| `minimax/ail-compound` | MiniMax M2.7 | text |
| `ail-image` | MiniMax image-01 | image generation |
| `minimax/ail-image` | MiniMax image-01 | image generation |
| `ail-embed` | mistral-embed via SwitchAI | embeddings |

### Stable endpoint
- **URL**: `http://10.42.42.1:18080` (dual-bound WireGuard address, constant per droplet)
- **Key**: `sk-tytus-user-<32 hex>` (per user, persisted in Scalesys, stable across pod lifecycle)

---

## Command cheat sheet

```bash
# Identity
tytus login                          # browser device-auth via Sentinel
tytus logout                         # revoke all pods + clear local state
tytus status [--json]                # plan, pods, units, tunnel state
tytus doctor                         # full diagnostic
tytus setup                          # interactive first-run wizard

# Pod lifecycle
tytus connect [--agent nemoclaw|hermes] [--pod NN]
tytus disconnect [--pod NN]          # tear down tunnel, keep allocation
tytus revoke <pod_id>                # DESTRUCTIVE: free units + wipe state
tytus restart [--pod NN]             # restart agent container

# Use the pod
tytus env [--export] [--raw]         # connection vars (stable by default)
tytus test                           # E2E health check
tytus chat [--model ail-compound]    # interactive REPL
tytus exec [--pod NN] "<command>"    # shell command inside agent container
tytus configure                      # interactive overlay editor

# Integration + docs
tytus link [DIR]                     # drop Tytus integration files into a project
tytus mcp [--format claude|kilocode|opencode|archon|json]
tytus bootstrap-prompt               # print the setup prompt to paste into AI tools
tytus llm-docs                       # full LLM-facing reference (read this first)
```

---

## Standard recipes

### Recipe A — Ensure a working pod, then chat
```bash
tytus status --json | jq -e '.pods | length > 0' \
    || tytus connect --agent nemoclaw
tytus test                                           # confirm green
eval "$(tytus env --export)"
curl -sS "$OPENAI_BASE_URL/chat/completions" \
    -H "Authorization: Bearer $OPENAI_API_KEY" \
    -H "Content-Type: application/json" \
    -d '{"model":"ail-compound","messages":[{"role":"user","content":"hi"}]}'
```

### Recipe B — Use the pod from a local AI tool (Cursor / Claude Desktop / OpenCode)
```bash
tytus connect                        # one-time per boot
tytus env --export                   # see exactly what to paste
```
Then paste into the tool's OpenAI-compatible settings:
```
OPENAI_BASE_URL = http://10.42.42.1:18080/v1
OPENAI_API_KEY  = sk-tytus-user-<32hex>
```
These never change. Set once, forget forever.

### Recipe C — Switch a pod's agent from nemoclaw to hermes
```bash
tytus disconnect --pod 02            # tear down tunnel only
tytus revoke 02                      # free units (destroys workspace)
tytus connect --agent hermes         # hermes (2 units)
tytus test
```

### Recipe D — Inspect or edit the agent's config overlay
```bash
tytus exec --pod 02 "cat /app/workspace/.openclaw/config.user.json.example"
tytus exec --pod 02 "cat > /app/workspace/.openclaw/config.user.json <<JSON
{ \"agents\": { \"defaults\": { \"contextTokens\": 64000, \"timeoutSeconds\": 300 } } }
JSON"
tytus restart --pod 02
```

### Recipe E — Link a project so other AI CLIs in that repo also know about Tytus
```bash
tytus link ~/projects/my-app                          # drops CLAUDE.md, AGENTS.md, .mcp.json, etc.
tytus link ~/projects/my-app --only claude,agents     # filter what gets dropped
```

---

## Error catalog

| Message | Cause | Fix |
|---|---|---|
| `No pods. Run: tytus connect` | No allocation | Run `tytus connect` (or `tytus setup` for the wizard) |
| `Tunnel daemon already running` | Stale PID file from previous session | `tytus disconnect` then retry |
| `403 plan_limit_reached` | Unit budget would be exceeded | Ask the user to revoke an existing pod or upgrade their plan |
| `401 Invalid API key` from gateway | Stable key map sync race, or wrong key | Wait 2s and retry; check `tytus env`; if persistent, run `tytus restart` |
| `503 no_capacity` | All droplets full | Backend issue — tell the user to wait or contact support |
| `Allocation failed` (unspecific) | Network or auth | Run `tytus doctor` first |

---

## Hard rules for AI agents driving Tytus

1. **Never invent models.** Only the five in the table above exist on this product. If the user asks for `gpt-4`, `claude-3`, `qwen3-8b`, etc., say it's not available on Tytus and offer `ail-compound` (the MiniMax M2.7 default).
2. **Never hardcode per-pod IPs** like `10.18.X.Y` — they change. Always use `10.42.42.1` from `tytus env`.
3. **Never paste raw per-pod keys into source files.** Always read `OPENAI_API_KEY` freshly from `tytus env --export` at runtime.
4. **`tytus revoke` and `tytus logout` are destructive.** Always confirm with the user before running them — they wipe the pod's workspace state (sessions, skills, memories, config overlays).
5. **Never call `sudo` directly to manipulate the tunnel.** `tytus connect` handles elevation transparently via osascript / `sudo -n` / interactive sudo. If elevation fails, troubleshoot through `tytus doctor`.
6. **Read fresh each session.** If another process revoked or rotated the user's pod, cached env values are wrong. Start by calling `tytus status`.
7. **Prefer `tytus` commands over raw HTTP when possible.** The CLI knows the stable endpoint, the current state, the agent type, and handles errors uniformly.
8. **Treat `tytus llm-docs` as the authoritative reference.** This file is the bootstrap; `tytus llm-docs` is the complete picture.

---

## What Tytus is NOT

- It is not OpenAI, Claude, or any public LLM service. It's the user's private pod.
- It is not free — the user pays Traylinx for their plan.
- It is not a replacement for Cursor / Claude Code / etc. — those are clients; Tytus is the backend.
- No customer LLM traffic ever traverses Traylinx Cloud — prompts and responses go user ↔ pod via WireGuard only. Treat any request as private to the user.

---

If anything in this document is unclear, run `tytus llm-docs` for the full 320-line reference with deeper detail.

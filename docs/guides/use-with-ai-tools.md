# Using Tytus with AI Tools

> One pod, every AI tool on your machine.

Tytus gives you a single OpenAI-compatible gateway. Any tool that can talk to OpenAI can talk to your pod — no per-tool configuration, no API key management, no vendor lock-in.

---

## The Stable Connection Pair

After `tytus connect`, you get two values that never change:

| Variable | Value | What it is |
|---|---|---|
| `OPENAI_BASE_URL` | `http://10.42.42.1:18080/v1` | Your pod's gateway endpoint |
| `OPENAI_API_KEY` | `sk-tytus-user-<32hex>` | Your personal API key |

These survive pod rotations, agent swaps, droplet migrations, and reboots. Set them once, forget them.

```bash
# Load them into your shell
eval "$(tytus env --export)"
```

---

## Claude Code

**Option A — Automatic** (recommended):
```bash
tytus link .
claude
```
This drops a `CLAUDE.md`, `.mcp.json`, and a `/tytus` slash command into your project. Claude Code reads them and knows how to drive Tytus natively.

**Option B — Manual**:
```bash
eval "$(tytus env --export)"
claude
```
Claude Code picks up `OPENAI_API_KEY` and `OPENAI_BASE_URL` from the environment.

**Option C — MCP** (deepest integration):
```bash
tytus mcp --format claude
```
Paste the output into your Claude Code MCP config. Claude gets native tools: `tytus_status`, `tytus_env`, `tytus_chat`, etc.

---

## Cursor

```bash
tytus link .
cursor .
```

Or add to Cursor Settings > Models > OpenAI Compatible:
- **Base URL**: `http://10.42.42.1:18080/v1`
- **API Key**: Run `tytus env` to see your key
- **Model**: `ail-compound`

---

## OpenCode

```bash
tytus link . --only opencode
opencode
```

This creates `.kilo/command/tytus.md` and `.kilo/mcp.json` so OpenCode knows about Tytus commands and has MCP tools available.

---

## Gemini CLI

```bash
eval "$(tytus env --export)"
gemini
```

Or inject the documentation:
```bash
tytus link . --only agents
gemini
```
Gemini reads `AGENTS.md` and learns the Tytus commands.

---

## Codex (OpenAI)

```bash
eval "$(tytus env --export)"
codex
```

Codex uses the standard `OPENAI_API_KEY` and `OPENAI_BASE_URL` environment variables.

---

## Aider

```bash
eval "$(tytus env --export)"
aider --model openai/ail-compound
```

Aider needs the `openai/` prefix to route through the OpenAI-compatible endpoint. The env vars handle the rest.

---

## Vibe

```bash
eval "$(tytus env --export)"
vibe
```

---

## Any OpenAI-Compatible Tool

If a tool supports custom OpenAI endpoints, configure it with:

| Setting | Value |
|---|---|
| Base URL / API Base | `http://10.42.42.1:18080/v1` |
| API Key | Your `sk-tytus-user-...` key (run `tytus env`) |
| Model | `ail-compound` |

Or set the environment variables:
```bash
eval "$(tytus env --export)"
your-tool-here
```

---

## Available Models

| Model | What it does | Use for |
|---|---|---|
| `ail-compound` | Text, vision, audio (MiniMax M2.7) | General coding, chat, analysis |
| `ail-image` | Image generation (MiniMax image-01) | Creating images |
| `ail-embed` | Embeddings (mistral-embed) | Vector search, RAG |

---

## The Tray Icon Shortcut

If `tytus-tray` is running in your menu bar:

1. Click the **T** icon
2. Open the **Open in** submenu
3. Pick your CLI (Claude Code, OpenCode, Gemini, etc.)

A new terminal window opens with:
- Environment variables already set
- Tytus documentation injected for that specific CLI
- The CLI running and ready to use

Zero typing, zero configuration.

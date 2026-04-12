# Getting Started with Tytus

> Your private AI pod, running in 2 minutes.

## What You Get

When you subscribe to Tytus, you get your own **private AI pod** — an isolated server with an AI gateway that speaks the OpenAI API format. Your conversations never touch Traylinx Cloud. Everything flows directly between your laptop and your pod through an encrypted WireGuard tunnel.

After setup, you get two values that **never change**:

```
Gateway:  http://10.42.42.1:18080/v1
API Key:  sk-tytus-user-<your-personal-key>
```

Paste these into any OpenAI-compatible tool — Claude Code, Cursor, Aider, OpenCode, VS Code extensions — and they just work. Switch pods, change agents, reboot your laptop — the values stay the same.

---

## Step 1: Install

```bash
curl -sSfL https://raw.githubusercontent.com/traylinx/tytus-cli/main/install.sh | sh
```

This installs `tytus` and `tytus-mcp` into `~/.local/bin` (or `$TYTUS_INSTALL_DIR`).

**What the installer does:**
- Downloads the right binary for your OS (macOS / Linux, Intel / ARM)
- Sets up passwordless sudo so tunnels connect without prompting
- Tells you the next step

**From source** (if you prefer):
```bash
git clone https://github.com/traylinx/tytus-cli.git
cd tytus-cli
cargo install --path cli --bin tytus --bin tytus-mcp
```

---

## Step 2: Setup

```bash
tytus setup
```

The setup wizard walks you through everything:

1. **Sign in** — Opens your browser for secure login (no passwords typed in the terminal)
2. **Plan check** — Shows your subscription tier and available units
3. **Agent pick** — Choose nemoclaw (default, 1 unit) or hermes (2 units)
4. **Connect** — Allocates your pod and opens the WireGuard tunnel
5. **Test** — Sends a sample chat to verify everything works

That's it. You now have a private AI pod running.

---

## Step 3: Use It

### Quick test
```bash
tytus chat
```
Opens an interactive chat with your pod.

### From any AI CLI
```bash
eval "$(tytus env --export)"
claude                    # Claude Code — just works
opencode                  # OpenCode — just works
aider --model openai/ail-compound  # Aider — just works
```

### From curl
```bash
eval "$(tytus env --export)"
curl -sS "$OPENAI_BASE_URL/chat/completions" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"ail-compound","messages":[{"role":"user","content":"hello"}]}'
```

### Using the tray icon
If you have `tytus-tray` installed, click the **T** icon in your menu bar:
- See live connection status
- Open any AI CLI pre-configured with your pod
- Connect / disconnect with one click

---

## What Happens Next?

Your tunnel stays active as long as the daemon is running. If you reboot:

```bash
# Option A: Auto-start (recommended)
tytus autostart install    # Reconnects automatically on every login

# Option B: Manual
tytus connect              # Reconnect after reboot
```

To check if everything is healthy:
```bash
tytus status              # Quick overview
tytus doctor              # Full diagnostic
```

---

## Need Help?

| What you want | Command |
|---|---|
| Check if connected | `tytus status` |
| Full health check | `tytus doctor` |
| See your stable URL + key | `tytus env` |
| Reconnect after reboot | `tytus connect` |
| Something is broken | `tytus doctor` then check the [Troubleshooting Guide](troubleshooting.md) |

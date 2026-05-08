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

Open the public beta page:

```text
https://get.traylinx.com/
```

Pick one file for your computer:

| Platform | What to download | What to do |
|---|---|---|
| macOS Apple Silicon | `Tytus-0.6.14-aarch64-apple-darwin-unsigned-PUBLIC-BETA-UNSIGNED.pkg` | Open the pkg. If macOS blocks it, Control-click the file and choose **Open**. |
| macOS Intel | `Tytus-0.6.14-x86_64-apple-darwin-unsigned-PUBLIC-BETA-UNSIGNED.pkg` | Open the pkg. If macOS blocks it, Control-click the file and choose **Open**. |
| Ubuntu/Debian x86_64 | `Tytus-0.6.14-x86_64-unknown-linux-gnu-unsigned-PUBLIC-BETA-UNSIGNED.deb` | Open with your software installer or run `sudo apt install ./Tytus-0.6.14-x86_64-unknown-linux-gnu-unsigned-PUBLIC-BETA-UNSIGNED.deb`. |
| Windows x86_64 | `tytus-windows-x86_64.zip` | Unzip it and run from PowerShell. MSI and driver packaging are not GA yet. |

This is a **public beta / technical preview**, not production GA. The macOS pkg and Linux deb are unsigned, so OS trust warnings are expected.

Checksum-verified command-line install is also available:

```bash
# macOS / Linux public beta
curl -fsSL https://get.traylinx.com/install.sh | TYTUS_RELEASE_TAG=v0.6.14-beta.7 sh
```

```powershell
# Windows public beta preview
$env:TYTUS_RELEASE_TAG="v0.6.14-beta.7"; irm https://get.traylinx.com/install.ps1 | iex
```

**What the installer does:**
- Installs `tytus`, `tytus-tray`, and `tytus-mcp` where the platform package supports them
- Installs the local Tytus desktop/tray entry where available
- Sets up the narrowly-scoped tunnel permission needed for the private connection
- Opens or points you to the setup wizard

**From source** is developer-only:
```bash
git clone https://github.com/traylinx/tytus-cli.git
cd tytus-cli
cargo install --path cli --bin tytus --bin tytus-mcp
```


## Platform notes

| Platform | v0.6.14-beta.7 status |
|---|---|
| macOS | Public beta unsigned pkg for Apple Silicon and Intel. Full CLI + tray + TytusOS path. |
| Linux | Public beta unsigned deb for Ubuntu/Debian x86_64. CLI, daemon, tunnel, MCP, TytusOS browser path, and desktop entries. |
| Windows | Zip + installer script technical preview for CLI/MCP. Full daemon/tray/tunnel/driver runtime parity is still a GA gate. |

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

### Using TytusOS
Open **Tytus** from your app launcher/menu bar. If you installed with the command-line path instead of the pkg/deb, install the tray first:

```bash
tytus tray install
open -a Tytus  # macOS
```

From TytusOS you can use Pod Inspector, Files, Channels, Settings, Terminal, and app workflows without remembering CLI commands. The tray opens TytusOS by default; legacy Tower is hidden rollback only.

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

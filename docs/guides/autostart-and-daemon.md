# Auto-Start and the Tytus Daemon

> Set it up once, forget about it forever.

## The Problem

You reboot your Mac. You open Claude Code. You start coding. Three minutes later — timeout. The tunnel isn't connected because `tytus connect` didn't run after the reboot.

## The Solution

### Option A: Autostart (Simple)

```bash
tytus autostart install
```

This installs a macOS LaunchAgent (or Linux systemd user service) that runs `tytus connect` every time you log in. Your tunnel is up before you open your first terminal.

**Check if it's installed:**
```bash
tytus autostart status
```

**Remove it:**
```bash
tytus autostart uninstall
```

### Option B: The Daemon (Advanced)

The Tytus daemon is a background process that manages your pod connection:

```bash
# Start in foreground (for launchd/systemd)
tytus daemon run

# Check status
tytus daemon status

# Stop
tytus daemon stop
```

**What the daemon does:**
- Keeps your authentication tokens fresh (refreshes every 5 minutes)
- Monitors connection health
- Provides live status to the tray icon
- Syncs pod state with the server

The daemon does NOT own the tunnel yet (that's coming in a future release). For now, `tytus autostart install` handles tunnel reconnection, and the daemon handles auth.

### Option C: Tray Icon (Visual)

The tray icon (`tytus-tray`) sits in your menu bar and shows:
- Live connection status
- Quick connect/disconnect
- Launch any AI CLI pre-configured
- Start/stop the daemon

```bash
tytus-tray    # Launch the tray icon
```

---

## How They Work Together

```
Boot → LaunchAgent runs "tytus connect"
     → Tunnel comes up automatically
     → Daemon refreshes tokens in background
     → Tray icon shows ● Connected
     → You open Claude Code, everything works
```

**Recommended setup:**
```bash
tytus autostart install    # tunnel reconnects on boot
tytus daemon run &         # background token management (optional)
tytus-tray &               # menu bar icon (optional)
```

---

## Diagnostic Logs

If autostart fails silently, check:

```bash
cat /tmp/tytus/autostart.log
```

This shows timestamped entries for:
- Startup state (email, tokens, pods)
- Token refresh results
- Tunnel activation success/failure
- Why a headless login was blocked

These logs are written automatically when Tytus runs in a non-interactive context (LaunchAgent, cron, pipe).

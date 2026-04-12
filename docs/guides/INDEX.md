# Tytus User Guides

Welcome to Tytus — your private AI pod, driven from any terminal.

## Guides

| Guide | What it covers |
|---|---|
| [Getting Started](getting-started.md) | Install, setup, first connection — 2 minutes to a working pod |
| [Use with AI Tools](use-with-ai-tools.md) | Claude Code, Cursor, OpenCode, Gemini, Aider, Vibe — one pod, every tool |
| [Plans, Agents, and Models](plans-and-agents.md) | Subscription tiers, nemoclaw vs hermes, available models |
| [Auto-Start and Daemon](autostart-and-daemon.md) | Survive reboots, background token refresh, tray icon |
| [Common Use Cases](common-use-cases.md) | Copy-paste recipes for real-world scenarios |
| [Troubleshooting](troubleshooting.md) | Fix common issues in 30 seconds |

## Quick Reference

```bash
tytus setup          # First-time setup wizard
tytus connect        # Connect to your pod
tytus status         # Check connection
tytus chat           # Interactive AI chat
tytus env            # Show your stable URL + key
tytus test           # Health check
tytus doctor         # Full diagnostic
tytus disconnect     # Stop the tunnel
tytus --help         # All commands
```

## The Two Values You Need

After connecting, paste these into any OpenAI-compatible tool:

```
Base URL:  http://10.42.42.1:18080/v1
API Key:   (run: tytus env)
Model:     ail-compound
```

They never change.

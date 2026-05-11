# Tytus User Guides

Welcome to Tytus — your private AI pod, driven from any terminal.

## Guides

| Guide | What it covers |
|---|---|
| [Public Beta Install](public-beta-install.md) | One-file beta installers, OS warnings, and first wizard run |
| [Getting Started](getting-started.md) | Install, setup, first connection — 2 minutes to a working pod |
| [Use with AI Tools](use-with-ai-tools.md) | Claude Code, Cursor, OpenCode, Gemini, Aider, Vibe — one pod, every tool |
| [Atomek Workbench](atomek-workbench.md) | TytusOS editor, chat, artifacts, Resource Fabric cockpit, local app skills |
| [Plans, Agents, and Models](plans-and-agents.md) | Subscription tiers, OpenClaw vs Hermes, available models |
| [Auto-Start and Daemon](autostart-and-daemon.md) | Survive reboots, background token refresh, tray icon |
| [Common Use Cases](common-use-cases.md) | Copy-paste recipes for real-world scenarios |
| [Tytus Ecosystem](tytus-ecosystem.md) | CLI, tray, TytusOS, Files, pods, session state, and install matrix |
| [Troubleshooting](troubleshooting.md) | Fix common issues in 30 seconds |

## Quick Reference

```bash
tytus setup          # First-time setup wizard
tytus os-docs        # TytusOS manual for AI agents
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

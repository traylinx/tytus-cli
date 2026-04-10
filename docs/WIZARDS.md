# Tytus CLI Wizards — "Grandma-Friendly" UX Plan

Goal: anyone can install, set up, and use Tytus without reading docs or touching config files.

## Design Principles

1. **Zero config** — sensible defaults, no manual editing
2. **Zero explanations required** — the wizard tells you what's happening
3. **Reversible** — every step can be redone
4. **Visual feedback** — spinners, progress bars, colored status
5. **Non-interactive fallback** — every wizard has `--yes` flag for CI
6. **Honest errors** — when something fails, say exactly what to do next

## Commands & Wizards

### Core Wizards

| Command | What it does | Wizard? |
|---------|-------------|---------|
| `tytus` (no args) | Welcome + smart default action | ✓ First-run onboarding |
| `tytus setup` | End-to-end first-time setup | ✓ Full wizard |
| `tytus login` | Browser auth | ✓ Animated browser flow |
| `tytus connect` | Allocate pod + tunnel | ✓ Step progress |
| `tytus disconnect` | Stop tunnel daemon | Quick action |
| `tytus revoke <pod>` | Free pod units | ✓ Confirmation prompt |
| `tytus logout` | Sign out everything | ✓ Confirmation prompt |

### Agent Management

| Command | What it does | Wizard? |
|---------|-------------|---------|
| `tytus configure` | Configure agent (OpenClaw/Hermes) | ✓ Interactive wizard |
| `tytus exec <cmd>` | Run command in pod container | No (power tool) |
| `tytus logs` | Stream agent logs | ✓ Live tail with colors |
| `tytus restart` | Restart agent container | ✓ Confirmation + progress |

### Testing & Discovery

| Command | What it does | Wizard? |
|---------|-------------|---------|
| `tytus chat` | Interactive chat with your AI | ✓ Full TUI chat |
| `tytus models` | Browse available models | ✓ Interactive list |
| `tytus doctor` | Run diagnostics | ✓ Animated checks |
| `tytus status` | Show everything at a glance | ✓ Dashboard |
| `tytus test` | Quick "is it working?" test | ✓ Animated test suite |

### Integrations

| Command | What it does | Wizard? |
|---------|-------------|---------|
| `tytus infect [dir]` | Inject AI CLI configs | ✓ Integration picker |
| `tytus env --export` | Shell env vars | No (scripting) |
| `tytus mcp` | Print MCP config | No (scripting) |

## Wizard Components (reusable framework)

### Visual Elements
- **Logo** — Tytus ASCII art banner (shown on welcome screens)
- **Spinner** — for "working..." operations (indicatif)
- **Progress bar** — for multi-step operations
- **Step indicator** — "Step 2/5: Connecting..."
- **Status lines** — green ✓, red ✗, yellow ⚠, blue ℹ
- **Boxes** — bordered info panels for important messages
- **Colored diffs** — for before/after comparisons

### Prompts (via `inquire`)
- Text input (with validation)
- Password input (hidden)
- Select (single choice from list)
- MultiSelect (multiple choices)
- Confirm (yes/no)
- Autocomplete (for model names, paths)

### Animations
- Typing animation for welcome text
- Pulsing spinner during operations
- Progress bar fill for downloads
- Success checkmark animation on completion

## Critical User Journeys

### Journey 1: First-time user
```
1. curl install.sh | sh        ← installs tytus
2. tytus                       ← detects first run, shows welcome
3. [onboarding wizard runs]
   - Welcome screen with logo
   - "Do you have an account?" → opens browser for login
   - "Which agent do you want?" → nemoclaw (simple) or hermes (advanced)
   - "Connecting to your pod..." → animated progress
   - "Configuring OpenClaw..." → runs exec commands
   - "Let's test it!" → runs a sample chat completion
   - "🎉 You're all set!" → shows how to use in their tools
4. tytus chat                  ← starts using it
```

### Journey 2: Returning user
```
1. tytus                       ← shows dashboard
   [Status] Pod 01 nemoclaw — Connected
   AI Gateway: http://10.18.1.1:18080
   [? for help, q to quit]
2. tytus chat                  ← immediate chat
```

### Journey 3: Something broke
```
1. tytus doctor
   [Running diagnostics...]
   ✓ Authentication: OK
   ✓ Subscription: Operator plan
   ✗ Tunnel: Not running
   → Fix: run `tytus connect`
2. tytus connect               ← one command to fix
```

## Implementation Order

1. Wizard framework (primitives)
2. `tytus` default view (dashboard + first-run)
3. `tytus setup` (full wizard)
4. Enhanced `tytus connect` (animated)
5. Enhanced `tytus doctor` (animated)
6. `tytus chat` (built-in chat)
7. `tytus configure` (agent setup)
8. `tytus test` (quick health check with animations)
9. Polish all other commands

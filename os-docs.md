# TytusOS — User Manual (LLM-facing reference)

> You are an AI agent (Claude Code, OpenCode, KiloCode, Gemini, Codex,
> Cursor, Vibe, etc.) running on a user's machine that has TytusOS
> installed. This document is the complete user manual, concatenated
> from the source-of-truth markdown in the tytus-os repository. Read
> it BEFORE answering any question about how TytusOS looks, behaves,
> or what its keyboard shortcuts / settings panels / apps do.
>
> The companion reference for the `tytus` CLI itself is `tytus llm-docs`.
> This document covers the desktop OS surface; `llm-docs` covers the
> command-line tool.

---


<!-- ==== getting-started.md ==== -->

# Getting Started

TytusOS is the desktop for your private AI pods. It runs in a browser tab or installed web app, but the data and control plane come from the local `tytus` tray daemon.

## Install path

Fresh machine flow:

1. Install `tytus` for your OS.
2. Start the tray daemon.
3. Sign in once from the tray or TytusOS login screen.
4. Allocate a pod or use the included All LLM Gateway.
5. Use Files, Channels, Pod Inspector, Terminal, and apps from TytusOS.

Typical commands during development/support:

```bash
tytus setup
tytus tray start
tytus open
```

## First screen

If you are not signed in, TytusOS shows **Sign in to Tytus**. The button opens the normal one-time browser sign-in flow. After approval, the screen refreshes automatically.

If the top bar says **Session expired**, your pods are still running. Open **Settings -> Daemon** and choose **Sign in again**.

## Desktop basics

- Left top bar: Tytus app icon and context-aware app menu.
- Right top bar: daemon/session status, pod count, included gateway, notifications, date/time.
- Left desktop icons: product surfaces such as Pod Inspector, Channels, Settings, Files, Terminal, Browser.
- Bottom dock: pinned apps and running apps.
- Windows: drag, resize, focus, minimize, maximize, close.

## First useful actions

If you are new to the agent-team workflow, read these next: **Tytus Resource Fabric**, **OpenClaw, Hermes, and Local Agents**, **Shared Folders**, and **Tytus Use Cases**. They explain how the local computer, pods, shared folders, local agents, channels, and apps work as one system.


| Need | Open |
|---|---|
| See pods and gateway URLs | Pod Inspector |
| Copy OpenAI-compatible env vars | Pod Inspector -> Copy env |
| Browse local workspace | Files -> Tytus Home |
| Browse a pod workspace | Files -> Pod NN workspace |
| Edit local files and ask AI with file context | Atomek |
| Launch local agents with active file context | Atomek -> Agent Team |
| Coordinate OpenClaw, Hermes, local agents, shared folders, and apps | Atomek -> Start mission |
| Learn workflows inside the product | Help -> Resource Fabric / Agents / Shared Folders / Use Cases, or Atomek -> Docs & Skills |
| Configure Telegram/Discord/Slack/etc. | Channels |
| Fix expired login | Settings -> Daemon |
| Check shared folders | Settings -> Sharing or Files -> Shared |
| Run CLI commands | Terminal |
| Change theme/wallpaper/dock | Settings -> Appearance / Background / Dock |

## Tytus Home

Tytus creates a user workspace at `~/Tytus` on install or first launch. It is the default terminal and file-browser home.

Default structure:

```text
~/Tytus/
├── Downloads/
├── Inbox/
├── Logs/
├── Outbox/
├── Pods/
├── Projects/
├── Shared/
└── README.md
```

## What is real

Real production surfaces:

- Pod Inspector and included All LLM Gateway details
- Pod allocation/install status and readiness checks
- Files over Tytus Home, shared folders, and pod workspaces
- Channels setup
- Terminal backed by the host shell through the tray daemon
- Atomek workbench for local files, chat, artifacts, embedded docs, app skills, and local Resource Fabric cockpit
- Settings for account, daemon, sharing, appearance, dock, language, privacy, updates
- Music Creator and other Tytus apps that use the included gateway

Demo/optional apps may exist behind the demo-app toggle. They must not block core pod workflows.


<!-- ==== resource-fabric.md ==== -->

# Tytus Resource Fabric

The Tytus Resource Fabric is the part of Tytus that turns separate tools into one working team. It connects the local computer, Tytus pods, OpenClaw, Hermes, shared folders, local agent CLIs, app skills, channels, and approval gates around a single user goal.

Use it when work is bigger than one chat answer: repo repair, document production, media generation, research, app automation, or any job where local files and remote pods need to exchange context.

## What the fabric connects

| Resource | What it does | Where the user sees it |
|---|---|---|
| Local computer | Owns real files, terminals, installed apps, browser sessions, and local AI CLIs. | TytusOS Terminal, Files, Atomek, local apps |
| Tytus Home | Default local workspace, logs, downloads, missions, and shared files. | `~/Tytus`, Files, Terminal |
| Shared folders | Exchange layer between local agents, pods, and apps. | Files -> Shared, mission folders, Garage bindings |
| OpenClaw | Fast Tytus pod agent for critique, planning, channel/app workflows, and remote execution. | Pod Inspector, Atomek Agent Team |
| Hermes | Heavier reasoning pod family when allocated. Use for deeper planning, writing, and review. | Pod Inspector, Atomek Agent Team |
| Local agents | Installed tools such as Claude Code, OpenCode, Codex, pi, Kimi, Gemini, Qwen, or Aider. | Terminal, Atomek local jobs |
| App skills | Instructions and drivers for apps such as Atomek, JULI3TA, Blender, Remotion, and future local tools. | App manifests, Atomek Docs & Skills |
| Channels | Telegram, Slack, Discord, LINE-style, and similar communication surfaces. | Channels app |
| AIL routes | Global model routes for remote and local AI. Apps discover these; they do not hardcode model ids. | Settings, Atomek settings, top bar |

## The core loop

1. **Create a mission** in Atomek or a shared folder.
2. **Collect context**: files, selected text, screenshots, links, task notes, resource list.
3. **Choose resources**: OpenClaw, Hermes, local CLI, app skill, shared folder, channel.
4. **Dispatch work** through Tytus host bridges, not raw browser fetches.
5. **Save transcripts and outputs** into the mission folder.
6. **Review changes** as previews, artifacts, or proposals.
7. **Approve and publish** only after the user sees the result.

This keeps autonomous work useful without turning the browser app into an unsafe shell.

## Mission folders

A mission folder is the safest default exchange format. It is ordinary files on disk, so every agent can understand it.

Typical structure:

```text
~/Tytus/Missions/<mission>/
├── MISSION.md        # human goal, rules, context
├── MISSION.json      # machine-readable contract
├── RESOURCES.md      # pods, local tools, folders, apps, skills
├── TASKS.md          # current task graph
├── HANDOFF.md        # copy-paste resume summary
├── INBOX.md          # incoming notes from agents/pods
├── OUTBOX.md         # approved outputs for user handoff
├── AUDIT.jsonl       # append-only events
├── RUNS.jsonl        # reloadable run index
├── runs/             # transcripts
├── outputs/          # generated files and final artifacts
├── proposals/        # patch/write/publish proposals
├── approvals/        # explicit approval records
└── NEXT.md           # next action
```

Agents should write into `runs/`, `outputs/`, or `proposals/` first. They should not blindly edit the source project unless the user approved the exact patch or workflow.

## What makes this useful

The user can run several different agents without losing state:

- OpenClaw writes a critique into the mission folder.
- Hermes writes a deeper plan or final copy.
- OpenCode or Claude Code runs against the local repo and writes a patch proposal.
- Atomek previews the patch and keeps the active files visible.
- JULI3TA, Blender, Remotion, or another app skill consumes the same mission assets.
- Channels notify the user or route handoffs to another workspace.

The important part is not one specific model. The important part is shared context, typed resources, visible outputs, and approval gates.

## Safety rules

- Apps must use the Tytus host/tray bridge for local files, pods, terminals, and model routes.
- Browser apps must not direct-fetch pod or gateway URLs that need same-origin proxying.
- Apps must not hardcode model names. AIL routes define available model aliases globally.
- Local jobs must use allowlisted tools, not arbitrary shell from model text.
- Destructive actions need explicit confirmation.
- Generated edits should become previews or proposals before they touch project files.

## First mission to try

1. Open **Atomek**.
2. Click **Start mission**.
3. Use a goal such as: `Review this repo with OpenClaw and local OpenCode, write findings to the mission folder, then propose a patch for approval.`
4. Attach the active folder or important files.
5. Run the pod/local tasks.
6. Open `runs/` and `proposals/` before applying anything.



<!-- ==== agents.md ==== -->

# OpenClaw, Hermes, and Local Agents

Tytus works best when agents act as a team instead of isolated chat boxes. OpenClaw and Hermes run in Tytus pods. Local agents run on the user's computer. Atomek and shared folders connect both sides.

## Agent families

| Agent | Best for | Typical role |
|---|---|---|
| OpenClaw | Fast critique, planning, tactical execution, channel/app workflows. | reviewer, planner, remote worker |
| Hermes | Deeper reasoning, synthesis, writing, long-form review when allocated. | architect, editor, senior reviewer |
| Local Claude/OpenCode/Codex/pi/Kimi/Gemini/Qwen/Aider | Work against local repos, terminals, installed tools, and user files. | implementer, tester, local operator |

Use the brand names **OpenClaw** and **Hermes** in user-facing docs and UI. Old internal labels should not leak into the product.

## Cortex memory profile (v0.7.0+)

Every chat with OpenClaw or Hermes flows through a Cortex layer that keeps
the agent's memory. There are two profiles:

- **Cloud Cortex** (default). Memory lives on Strato. Routes through
  `tytus.traylinx.com`. Works out of the box, no extra setup.
- **Local Cortex** (opt-in). Memory lives on the user's Mac. Routes
  through `127.0.0.1:8098`. Requires Docker Desktop + one `tytus cortex
  up` step.

For end users, the picker lives in **Settings → AI**. For AI agents
driving TytusOS, the wire is:

- `GET /api/cortex/status` — read profile + reachability.
- `POST /api/cortex/profile` with `{profile:"cloud"|"local"}` — flip.
- `POST /api/cortex/memory/search` — semantic recall (local profile only).

Apps that consume the Cortex surface go through the Host API:

- `host.ai.cortexProfile()` — `{profile, available, port?, cortexVersion?}`.
- `host.ai.cortexSearch({query, limit?, appId?, minSimilarity?})` —
  returns `CortexMemoryHit[]` (empty array on cloud or when unreachable —
  apps don't need to branch on errors).

Chat events now include a leading `{type:'profile', profile, cortexVersion?}`
frame so the UI can label "Cloud Cortex" vs "Local Cortex" beside each
assistant response.

**Memory is NOT writable through an explicit verb.** Cortex consolidates
memories from chats; there is no `cortexRemember()` API. Atomek's existing
"Remember" button writes to a separate workbench-scoped store and is
unchanged.

See `tytus cortex --help` and the user-facing manual page
`services/tytus-os/docs/user-manual/local-cortex.md` for install/operate
details.

## Install and check agents

CLI examples:

```bash
tytus status
tytus agent catalog
tytus agent install openclaw
tytus agent install hermes
tytus agent list
```

TytusOS examples:

1. Open **Pod Inspector**.
2. Check which pods are allocated and ready.
3. Open agent details to see readiness, routes, logs, and environment.
4. Open **Atomek -> Agent Team** to see the same resources in mission context.

Do not treat a pod URL as enough. A pod is useful when its health/readiness checks pass and the Tytus bridge can call it.

## How agents should work together

A strong agent-team workflow:

1. **User defines mission** in Atomek.
2. **OpenClaw** performs fast critique or task decomposition.
3. **Local agent** performs repo/file work on the local machine.
4. **Hermes** reviews architecture, copy, or final plan when available.
5. **Atomek** collects transcripts and patch proposals.
6. **User approves** edits or artifacts.

The mission folder is the shared memory for this work. Every agent should leave enough evidence for the next agent to continue.

## Dispatch surfaces

| Surface | Use it for |
|---|---|
| Pod Inspector | install, inspect, restart, open pod UIs, check readiness |
| Atomek Agent Team | choose resources, create missions, run local jobs, ask pods, inspect transcripts |
| Terminal | supervised shell, project commands, manual local-agent CLI runs |
| Files | browse Tytus Home, Shared, mission folders, pod workspaces |
| Channels | connect agents to Telegram/Slack/Discord/etc. |

Atomek should not replace Claude Code, OpenCode, Codex, or pi. It should orchestrate them with files, context, and approval gates.

## Good use cases

### Repo repair

- Open local repo in Atomek.
- Start mission: `Find root cause, propose patch, run tests, save proof.`
- Ask OpenClaw for critique.
- Run local OpenCode or Claude Code with mission context.
- Ask Hermes or another reviewer for final risk review.
- Apply only approved patches.

### Research and synthesis

- Put source docs in a mission folder.
- Ask OpenClaw to extract claims and risks.
- Ask Hermes for synthesis.
- Ask local agent to format output and update docs.
- Save final summary in `OUTBOX.md`.

### Creative production

- Store brief, references, lyrics, images, and audio in shared/mission folders.
- Use JULI3TA, Blender, Remotion, or other app skills for media steps.
- Use agents for prompts, review, edits, and packaging.
- Keep source assets and final outputs in the same mission folder.

## Troubleshooting

| Problem | Fix |
|---|---|
| OpenClaw or Hermes missing | Install/allocate the agent from Pod Inspector or CLI, then refresh Atomek capabilities. |
| Pod says running but job fails | Check readiness in Pod Inspector and `tytus doctor`. Running is not the same as healthy. |
| Local agent missing | Install the CLI on the computer, confirm it is on `PATH`, then refresh Atomek. |
| Agent output disappears | Look in the mission folder `runs/`, `RUNS.jsonl`, and Atomek Outputs. |
| Wrong model shown | Update global AIL route/model config; do not hardcode model ids in apps. |



<!-- ==== shared-folders.md ==== -->

# Shared Folders

Shared folders are the exchange layer between the local computer, Tytus pods, local agents, app skills, and future remote workspaces. They make the agent team practical because every participant can read and write ordinary files instead of relying on one chat transcript.

## Shared folder types

| Type | Use | Typical path |
|---|---|---|
| Tytus Home Shared | Local drop-zone available from Files and Terminal. | `~/Tytus/Shared` |
| Mission folder | Per-job context, transcripts, outputs, proposals, approvals. | `~/Tytus/Missions/<mission>` |
| Pod workspace | Agent-side working directory. | `/app/workspace` |
| Pod inbox/outbox | File transfer points for pods. | `/app/workspace/inbox`, `/app/workspace/out` |
| Garage/garagetytus binding | Cross-machine or pod-synced folder when configured. | user-chosen local path |

Use **Files** for browsing. Use **Atomek** when the shared folder is part of an agent mission and should be connected to chat, context, and approvals.

## Local to pod exchange

CLI examples:

```bash
tytus push ./brief.md --pod 01 --to /app/workspace/inbox/brief.md
tytus ls /app/workspace/inbox --pod 01
tytus pull /app/workspace/out/result.md --pod 01 --to ~/Tytus/Shared/result.md
```

TytusOS examples:

1. Open **Files**.
2. Browse **Tytus Home**, **Shared**, or a pod workspace.
3. Open **Atomek** when you need editing, context, chat, or mission coordination.
4. Keep final handoffs in the mission folder `OUTBOX.md` or `outputs/`.

## Mission folder exchange

Recommended convention:

```text
INBOX.md          incoming notes, findings, pod outputs
OUTBOX.md         final user-ready handoff
runs/             full transcripts from local or pod jobs
outputs/          generated docs/assets
proposals/        patches or write proposals before approval
approvals/        approve/reject records
```

Agents should append a short note to `INBOX.md` when they leave a finding for another agent. Final artifacts should be copied or summarized into `OUTBOX.md`.

## Garage / garagetytus

When a folder needs to sync across machines or pods, use a Garage-backed binding instead of manual copy/paste. A binding maps a local folder to a shared bucket and keeps it synchronized.

Typical intent:

- share a project handoff with a pod
- share media assets between local apps and remote agents
- keep a mission folder visible on another machine
- let one agent produce files another agent can consume

If no binding exists, the local shared folder `~/Tytus/Shared` is still useful as a same-machine drop-zone.

## Conflict rules

- Prefer one writer per file at a time.
- Use append-only logs (`AUDIT.jsonl`, `RUNS.jsonl`) for events.
- Use separate files in `runs/` for transcripts.
- Put proposed edits in `proposals/` before applying them.
- If two agents produce conflicting outputs, keep both and ask a reviewer agent or the user to choose.

## What not to do

- Do not give agents broad blind write access to the whole home directory.
- Do not use shared folders as a hidden command channel for destructive actions.
- Do not store raw credentials in mission folders.
- Do not assume pod paths and local paths are identical.
- Do not bypass Tytus path guards with `..`, symlink escapes, or encoded traversal.

## First shared-folder workflow

1. Put input files in `~/Tytus/Shared/<project>/` or create an Atomek mission.
2. Ask OpenClaw to review the files and write findings to `INBOX.md`.
3. Ask a local agent to implement or format output and save a proposal.
4. Use Atomek to preview and approve.
5. Move final files to `OUTBOX.md` or `outputs/`.



<!-- ==== use-cases.md ==== -->

# Tytus Use Cases

These workflows show how Tytus, OpenClaw, Hermes, local agents, shared folders, Atomek, and apps fit together.

## 1. Repair a repo with a team

Goal: fix a bug without losing evidence or letting one model blindly edit files.

1. Open the repo folder in **Atomek**.
2. Start a mission: `Find root cause, propose patch, run tests, save proof.`
3. Attach the active file or folder context.
4. Ask **OpenClaw** for a fast critique and task split.
5. Run **OpenCode**, **Claude Code**, **Codex**, or **pi** locally through Terminal or Atomek local job.
6. Save transcript under `runs/` and proposed patch under `proposals/`.
7. Ask **Hermes** or another reviewer for risk review when available.
8. Apply only approved diffs.
9. Put final status in `OUTBOX.md`.

## 2. Build a document package

Goal: produce a client-ready brief from scattered notes.

1. Create a mission folder.
2. Put source notes, PDFs, screenshots, and requirements in `INBOX.md` or `inputs/`.
3. Ask OpenClaw to extract claims, gaps, and risks.
4. Ask Hermes to synthesize structure and tone.
5. Ask a local agent to write markdown files.
6. Review in Atomek markdown preview.
7. Export final files to `outputs/`.

## 3. Creative production across apps

Goal: make a song/video/scene with agents and local apps.

1. Store brief, references, lyrics, audio, and image assets in a mission folder.
2. Use **JULI3TA** for music generation or restyle tasks.
3. Use **Blender** or **Remotion** skills when installed for scenes and renders.
4. Ask agents to generate prompts, check style consistency, and prepare handoffs.
5. Keep every source asset and final render in `outputs/`.

## 4. Shared-folder pod handoff

Goal: let a remote pod consume files from the local computer and return output.

1. Put files into `~/Tytus/Shared/<job>/` or a Garage-bound folder.
2. Push/copy relevant inputs to the pod workspace if needed.
3. Ask OpenClaw or Hermes to process the files.
4. Pull outputs back or let sync return them.
5. Review with Atomek.

## 5. Channel-supervised agent work

Goal: keep the user informed while agents work.

1. Configure Channels for the pod or user account.
2. Create a mission folder with clear rules.
3. Dispatch an agent task.
4. Agent writes status to the mission folder and sends channel updates when configured.
5. User approves proposals from Atomek or the channel workflow.

## 6. Local app automation

Goal: use installed desktop apps without rebuilding them inside Atomek.

1. Install the app and its Tytus/agent skill if available.
2. Refresh capabilities in Atomek.
3. Attach the relevant files or mission folder.
4. Launch the app or local bridge through Tytus host integration.
5. Keep artifacts in the mission folder.

Examples: Blender MCP scene generation, Remotion render recipes, JULI3TA music workflows, browser checks, local terminal tasks.

## 7. Research watch

Goal: monitor a topic and produce a concise report.

1. Start a mission with a topic and sources.
2. Use a pod or AIL route for research.
3. Use a local agent for synthesis and citation cleanup.
4. Store raw findings in `runs/`.
5. Store final report in `OUTBOX.md`.

## Rules that apply to every use case

- Use shared files for context handoff.
- Use Atomek when you need visibility, editing, and approval.
- Use Terminal when you need full manual control.
- Use OpenClaw for fast pod perspective.
- Use Hermes for deep reasoning when allocated.
- Use local agents for work requiring the local filesystem or installed tools.
- Use app skills for app-specific instructions.
- Keep AIL model selection global.
- Never apply destructive output without approval.



<!-- ==== windows.md ==== -->

# Windows

Every app in TytusOS runs in a window. Windows behave like a real OS: drag, resize, focus, minimize, maximize, restore.

## Anatomy

```
┌──────────────────────────────────────┐  ← 1 px border (brighter when focused)
│ 📁 App Name              −  □  ×    │  ← title bar (36 px tall, drag handle)
├──────────────────────────────────────┤
│                                      │
│            App content               │
│                                      │
│                                      │
└──────────────────────────────────────┘  ← rounded 12 px corners (square when maximized)
```

Edges and corners are invisible 6 px / 14 px hit zones — your cursor changes to indicate which direction will resize.

## Move a window

- **Click and drag** the title bar (anywhere except buttons or the icon).
- A grabbing-hand cursor confirms you're dragging.
- Windows can't go above the top panel (28 px) or fully off-screen — at least 100 px stays visible.

## Resize a window

- **Drag any edge or corner**. The cursor shows the direction.
- Minimum size: **320 × 200 px**.
- Edges resize one dimension; corners resize both.
- Maximized windows can't be resized — restore them first.

## Close · Minimize · Maximize

The three buttons on the right side of the title bar:

| Button | Hover effect | Action |
|---|---|---|
| **−** | grey | Hides the window. Click its dock icon to bring it back. |
| **□** | grey | Fills the available space (between top panel and dock — top 28 px + dock space 68 px reserved). Click again to restore. |
| **×** | turns red | Closes the window. State is lost. |

You can also **double-click the title bar** to maximize/restore.

## Focus

Click any window to bring it to the front. The focused window has:
- A brighter border
- A darker title bar background (`#1A1A1A` vs `#141414`)
- The accent-colored dot below its dock icon

## Alt+Tab

Hold **Alt** and tap **Tab** to cycle through open windows. Release Alt to commit. The overlay shows window thumbnails with the currently selected one highlighted.

Minimized windows don't appear in Alt+Tab — click their dock icon to restore.

## Snap to edges

While dragging a window's title bar, drag the cursor near a viewport edge to see a translucent overlay showing the snap target:

- **Left edge** → snap to left half
- **Right edge** → snap to right half
- **Top edge** → maximize

Release the mouse to commit. The unsnapped frame is remembered, so dragging the snapped window away from its half (more than ~24 px) restores the prior size at the cursor position.

Two windows of the same app can be snapped independently — left half + right half work as expected for, e.g., comparing two Files panes.

Snap state persists across reload.

## Keyboard shortcuts for windows

| Combo | Action |
|---|---|
| **Cmd+W** (Ctrl+W) | Close active window. **Does not** close the host browser tab — Tytus intercepts the keypress. |
| **Cmd+Q** (Ctrl+Q) | Close every window of the active app. |
| **Cmd+Space** | Toggle the App Launcher. |
| **Alt+Tab** | Cycle visible windows. |
| **Esc** | Close any open modal or menu. |
| **double-click title** | Maximize / restore. Snap-aware: if snapped, restore returns to the original floating frame, not the half. |

See [keyboard-shortcuts.md](./keyboard-shortcuts.md) for the full list.

## Animations and "Reduce motion"

Open / close / snap-resize transitions are CSS-only and short (120–150 ms). Tytus respects:

- The **system** "Reduce motion" preference (macOS System Settings → Accessibility → Display).
- A **Tytus-level** override at **Settings → Appearance → Reduce motion**.

Either flips animations off live with no reload.

## Restoring a minimized window

Three ways:
1. **Click the dock icon** for the app — restores and focuses
2. Open it via **launcher or desktop icon** — same effect
3. **Alt+Tab** doesn't include minimized windows (intentional)

## Multiple windows of the same app

Open multiple instances by re-launching from the dock or launcher when no window of that app is currently visible. Each gets its own Z-index and its own state.

## Z-index

The most-recently-focused window is on top. Click any other window to bring it forward.

## Edge cases

- **Window stuck off-screen?** Press ⌘+D to minimize all, then click the dock icon to bring it back at default position.
- **Drag won't start?** Make sure you're clicking the title bar background, not the icon, title text, or buttons.
- **Resize jumps?** The minimum size is 320×200 — you can't go smaller.

## Internals

The window chrome lives in `app/src/components/WindowFrame.tsx`. Each window carries `data-app-id`, `data-window-id`, and `data-window-title` attributes so other code (and tests) can find it reliably.

State lives in `useOSStore` under `state.windows`. The reducer actions are `OPEN_WINDOW`, `CLOSE_WINDOW`, `MINIMIZE_WINDOW`, `MAXIMIZE_WINDOW`, `RESTORE_WINDOW`, `FOCUS_WINDOW`, `MOVE_WINDOW`, `RESIZE_WINDOW`.


<!-- ==== desktop.md ==== -->

# Desktop

The desktop is the space below the top menu bar and above the dock. It shows the active wallpaper and product shortcuts.

## Default icons

The default production shortcuts are Tytus surfaces, not fake OS controls:

- **Pod Inspector** — fleet, gateway, readiness, pod actions
- **Channels** — messenger/channel setup
- **System Settings** — account, daemon, sharing, theme, dock, updates
- **Help** — user manual, troubleshooting, diagnostics
- **Chat** — open agent chat surfaces
- **Terminal** — host-backed Tytus terminal
- **Files** — Tytus Home, shared folders, pod workspaces
- **Browser** — registered web/app launchers

## Icon actions

| Action | What happens |
|---|---|
| Double-click | Opens the app |
| Click and drag | Moves the icon on the desktop grid |
| Right-click | Opens the app/context menu when available |
| Single click | Selects the icon |

Icon positions persist in browser storage and survive reloads.

## Empty desktop menu

The desktop context menu should only expose actions backed by real product behavior:

| Item | Action |
|---|---|
| Open Terminal | Opens Terminal in `~/Tytus` |
| Change Background | Opens Settings -> Background |
| Arrange Icons | Aligns icons to the grid |
| Open Help | Opens Help -> Getting Started |

Do not add decorative Display/Wi-Fi/Bluetooth/Printer-style controls unless they are wired to real Tytus behavior.

## Wallpaper

The default wallpaper comes from the TytusOS design pack. Change it from **Settings -> Background**.

## Trash

The trash icon lives at the right end of the dock. Trash/recoverable file operations must be implemented through the Files app and daemon-safe roots; do not simulate destructive behavior.


<!-- ==== dock.md ==== -->

# Dock

The dock is the floating app bar at the bottom-center of the screen. It launches Tytus apps and shows running-window state.

## Default pins

Typical production pins:

1. Show Applications
2. Pod Inspector
3. Channels
4. System Settings
5. Files
6. Terminal
7. Chat
8. Running/unpinned apps
9. Trash

The exact order can change by release; the behavior is stable.

## Active indicator

Open apps show a small dot inside the dock:

- Accent dot: focused app
- Grey/subtle dot: open but not focused
- No dot: no window open

## Click behavior

| State | Click result |
|---|---|
| No window open | Opens the app |
| Window minimized | Restores and focuses it |
| Window already open | Focuses most recent window |
| Multiple windows | Focuses most-recently-active window |

## Customization

Use **Settings -> Dock** for size, position, auto-hide, order reset, and future pinning controls.

## Trash

Trash is a product affordance, not a fake animation. File deletion/recovery belongs to Files and must respect root-anchored source safety.


<!-- ==== launcher.md ==== -->

# App Launcher

A full-screen overlay listing every installed app, organized by category.

## Open it

- **Click the apps-grid button** at the far left of the dock
- **Press ⌘** (Super / Win key)

## Close it

- **Press Esc**
- **Click anywhere outside** the search bar / app grid
- **Click an app** — launcher closes and the app opens

## Layout

```
                      ┌─────────────────────────┐
                      │ 🔍 Type to search…      │
                      └─────────────────────────┘

                      FREQUENTLY USED
                      📦  ⚙️  💬  📁  ✈️  ⌨

  Favorites    All    System    Internet    Productivity    Media    DevTools    Creative    Games

                  🗒️    📅    🧮    ⏰    📊    📝    📄    📑    🎵    🎬    🎮    ♟️    …
```

## Sections

- **Frequently Used** — your 6 pinned dock apps (only shown when search is empty)
- **Categories** — `Favorites`, `All`, `System`, `Internet`, `Productivity`, `Media`, `DevTools`, `Creative`, `Games`
- **App grid** — every app matching the active category and search

## Search

Start typing — the grid filters to apps matching by name *or* description. Search clears the category filter (always searches across all apps).

The **×** button inside the search box clears it.

## Categories at a glance

| Category | Apps |
|---|---|
| **Favorites** | Whatever's pinned to the dock right now |
| **System** | Pod Inspector · Settings · Files · Terminal · Help · System Monitor · Archive Manager |
| **Internet** | Chat · Browser · Channels · Weather · RSS Reader |
| **Productivity** | Notes · Todo · Reminders · Calendar · Calculator · Clock · Spreadsheet · Text Editor · Document Viewer · Markdown Preview |
| **Media** | Image Viewer · Image Gallery · Photo Editor · Music Player · Video Player · Voice Recorder · Screen Recorder · Media Converter |
| **DevTools** | Code Editor · API Tester · JSON Formatter · Regex Tester · Base64 Tool · Color Palette |
| **Creative** | Drawing · Whiteboard · Color Picker · ASCII Art · Matrix Rain |
| **Games** | Minesweeper · Snake · Tetris · Tic-Tac-Toe · 2048 · Sudoku · Chess · Memory · Pong · Solitaire · Flappy Bird |

Full list with one-liners: [apps-catalog.md](apps-catalog.md).

## Keyboard

- **Type** to search
- **Esc** to close

(Arrow-key navigation between cards is on the [roadmap](../development/roadmap.md).)


<!-- ==== keyboard-shortcuts.md ==== -->

# Keyboard Shortcuts

Every shortcut TytusOS responds to today.

> **Note on Cmd vs Ctrl.** Tytus uses a single binding string `Mod+<key>`
> internally — `Mod` resolves to `Cmd` on macOS and `Ctrl` on Linux/Windows.
> The tables below show the macOS form; substitute Ctrl on other platforms.

## System

| Shortcut | Action |
|---|---|
| **⌘+Space** | Toggle App Launcher |
| **Esc** | Close launcher · Close notification center · Close any modal |
| **⌘+D** | Minimize all windows |
| **⌘+Z** | Undo the last reversible file operation (move / copy / delete / rename / paste). |
| **⌘+V** | Paste from the host browser clipboard (image → save / text → toast). Requires a granted clipboard permission. Inside a text input, the browser's native paste runs instead. |

## Windows

| Shortcut | Action |
|---|---|
| **⌘+W** | Close the focused window. **Does not** close the host browser tab — Tytus intercepts the keypress. |
| **⌘+Q** | Close every window of the focused-window's app. |
| **Alt+Tab** (hold Alt, tap Tab) | Cycle through visible windows. |
| **Release Alt** | Commit the Alt+Tab selection. |
| **Double-click title bar** | Maximize / restore. Snap-aware. |

While dragging a window:
- Drag near the **left** / **right** edge → snap to that half on release.
- Drag near the **top** edge → maximize on release.
- Drag a snapped window > 24 px away from its snap → restore prior frame.

## Files (FileManager + Desktop)

| Shortcut | Action |
|---|---|
| **⌘+A** | Select all icons / rows. |
| **Esc** | Clear selection. |
| **Click** | Select one. |
| **⌘+Click** | Toggle selection. |
| **Shift+Click** | Select range. |
| **Drag in empty area** | Lasso-select. |

## Quick launch

| Shortcut | Action |
|---|---|
| **Ctrl+Alt+T** | Open Terminal |
| **⌘+K** (macOS) / **Ctrl+K** (Linux/Windows) | Open the Command Palette — search-as-you-type, **↑ / ↓** to navigate, **Enter** to execute, **Esc** to close |

## In the App Launcher

| Shortcut | Action |
|---|---|
| **Type** | Search |
| **Esc** | Close launcher |

## In Settings, Notes, Todo, etc.

These apps have their own keyboard shortcuts inside their windows — check each app's interface.

## OS reservations

A handful of combos are intercepted before TytusOS sees them:

- `Cmd+Tab` — handled by macOS itself (system app switcher). Tytus ships **Alt+Tab** as the in-OS alternative.
- `Cmd+R` / `F5` — reloads the page (you'll lose unsaved app state).
- `Cmd+T` / `Cmd+N` — open a browser tab / window.

Tytus's shortcut router default-blocks `Cmd+W`/`Cmd+Q`/`Cmd+R`/`Cmd+T`/`Cmd+N` so those host-browser bindings can't kill the WebView from inside the OS, and registers its own handlers for `Cmd+W` and `Cmd+Q` on top.

## Internals

The shortcut router lives at `app/src/lib/shortcuts.ts`. It dispatches by scope priority `text-input > modal > active-app > shell`, so a focused text field always wins for combos like `Cmd+C`/`Cmd+V`/`Cmd+Z`.


<!-- ==== files.md ==== -->

# Files

Files is the Finder-like browser for Tytus. It covers local Tytus Home, shared folders, and pod workspaces. Use Atomek when you need an editor, markdown preview, chat context, patch previews, or local agent work against those files.

## Sources

| Source | Path | Scope |
|---|---|---|
| Tytus Home | `~/Tytus` | local user workspace |
| Inbox | `~/Tytus/Inbox` or `/app/workspace/inbox` on a pod | incoming files/tasks |
| Outbox | `~/Tytus/Outbox` | prepared files for agents/pods |
| Downloads | `~/Tytus/Downloads` or pod downloads | generated/downloaded outputs |
| Shared | configured shared folder / garagetytus binding | account-wide shared storage |
| Pod NN workspace | `/app/workspace` | selected pod filesystem |

## Normal use

- Use **Browse** for Tytus Home and source switching.
- Use **Inbox** and **Downloads** for pod-specific folders.
- Use **Shared** for account-level shared folders. Shared folders are not pod-scoped; pod selection does not change the binding list.
- Use the pod sidebar when you need `/app/workspace` on a specific pod.

## Empty folders

A missing pod inbox or downloads directory should render as a friendly empty state, not raw CLI stderr. If you see `tytus ls: no such path`, report it as a Files empty-state bug.

## Files vs Atomek

| Need | Use |
|---|---|
| Browse or move through Tytus Home, Shared, Inbox, Outbox, Downloads | Files |
| Open a local folder for editing | Atomek |
| Ask AI about the active file | Atomek chat |
| Run tests or local tools with folder context | Atomek -> Agent Team or Terminal |
| Inspect generated artifacts and patch previews | Atomek Outputs |

Files owns broad navigation. Atomek owns editing and agent interaction. Do not duplicate full editor behavior in Files.

For the complete multi-agent exchange model, read **Shared Folders** and **Tytus Resource Fabric** in Help.

## Safety

File operations must be root-anchored to the selected source. Path traversal, symlink escape, null bytes, and double-encoded traversal must be rejected by daemon-side tests before write operations ship broadly.

## Shared folders

Shared folders use the account-level sharing system. Manage global defaults and diagnostics in **Settings -> Sharing**. Use Files -> Shared for browsing/opening the configured source.


<!-- ==== atomek.md ==== -->

# Atomek

Atomek is the TytusOS Resource Fabric cockpit for local files, code, markdown, chat, artifacts, mission folders, shared folders, local agents, OpenClaw/Hermes pods, and app skills. It runs inside TytusOS, but it is published as its own Tytus app so it can move faster than the OS shell.

Use Atomek when you want to open a real folder, inspect or edit files, ask an AI about the active file, preview patches, or coordinate local agents and Tytus pod agents through one shared mission context.

## Open files and folders

Atomek uses the browser File System Access API when the browser supports it.

Normal flow:

1. Open **Atomek** from the launcher or dock.
2. Click **Open Folder** or **Open File**.
3. Pick the local folder or file in the browser permission picker.
4. The Explorer shows the selected tree and recent folders.
5. Click a file to open it in a tab.

Folder rows are clickable. Use the chevron to expand or collapse child folders. Text files open in the Monaco editor. Markdown files can be edited and previewed.

## State restore and recent workspaces

Atomek remembers the last working state in the browser:

- recent files and folders
- persisted browser file/folder handles when Chromium allows it
- active folder and file tree
- open editor tabs and active file
- typed chat input
- selected activity, visible sidebars, and layout

After reload or reopen, Atomek restores the same state and asks the browser for file permission only when needed. If permission expired, use **Open File** or **Open Folder** once to refresh the handle.

Click the active Activity Bar icon again, or use **Cmd/Ctrl+B**, to hide or show the primary side bar.

## Editing

Atomek is a real editor surface, not a static preview.

- Open tabs show the current files.
- Text and markdown files are editable in Monaco.
- Save writes through the browser file handle after the browser has granted permission.
- Dirty files stay in memory until saved.
- Markdown preview is available from the editor surface.
- The layout reflows when the window resizes.

If the editor is blank after a release, hard-refresh TytusOS and reopen the file. A blank editor with a known text file usually means the browser still has an old Atomek bundle cached.

## Chat and context

The right panel is the Atomek chat surface. It is not separate from the workspace.

The chat can attach:

- the active file
- selected/open editor context
- typed prompt text
- generated artifacts
- preview edits
- resolved local app skills

Use the context chips above the input to see what will be sent. Deselect a chip when you do not want that context included.

Chat uses the Tytus host AI bridge. It must not hardcode model IDs in the app. Model/provider selection comes from the global AIL configuration exposed by the host. When global AIL changes, Atomek should pick up the new model list through settings and host state, not through source edits.

## Local AIL and remote AIL

Atomek can route chat through the host AIL settings:

- **Remote AIL**: the Tytus pod/gateway route.
- **Local AIL**: the local/private route exposed by the host if installed and enabled.

The model picker should show models discovered from the selected AIL route. If the picker shows an old model, check the global AIL configuration first. Do not patch Atomek with a hardcoded replacement model.

## Artifacts and patch previews

Atomek should convert AI output into reviewable artifacts, not blind writes.

Expected flow:

1. Ask for an edit or generated file.
2. Atomek stores the answer as an artifact or preview edit.
3. You inspect the diff or generated content.
4. You explicitly save/apply the result.

Use **Outputs** to inspect saved artifacts and agent job output. Code blocks should render as rich output with copy controls.

## Resource Fabric / Agent Team

The **Agent Team** activity is the bridge to the Tytus Resource Fabric. It replaces duplicate extension panels and avoids turning Atomek into another IDE clone.

Atomek coordinates local computer resources, shared folders, OpenClaw/Hermes pods, local agents, AIL routes, channels, and app skills through one mission context.

It discovers capabilities through the Tytus host bridge, for example:

- OpenClaw pod agents
- Hermes pod agents when allocated
- Tytus Terminal
- pi
- OpenCode
- Codex
- Claude Code
- Gemini
- Qwen
- Kimi
- Aider

Only allowlisted tools should launch from Atomek. The browser must not run arbitrary shell commands and must not direct-fetch pod or model endpoints that fail CORS. Local work goes through the same-origin Tytus tray/host bridge.

### Why shared folders matter

Shared folders are the exchange layer between the local computer and Tytus pods. Local agents can write plans, transcripts, patches, screenshots, and artifacts into the shared/mission folder; OpenClaw or Hermes pods can pick that context up and return their own outputs. This is the main Atomek workflow: local resources and remote resources working as one team while the user keeps the files visible and controllable.

### Mission packs

A mission pack is a tray-managed folder under **Tytus Home / Missions**. It gives agents and pods a shared working context without granting blind write access to your project.

A new mission writes a standard pack:

- `MISSION.md` — human-readable goal, context, resources, constraints, next steps
- `MISSION.json` — typed machine contract, selected resources, task graph, approval gates
- `RESOURCES.md` — selected pods, local agents, folders, apps, and skills
- `TASKS.md` — planned task graph
- `HANDOFF.md` — copy-paste summary for another agent/window
- `INBOX.md` / `OUTBOX.md` — lightweight shared-folder exchange points
- `AUDIT.jsonl` — append-only mission events
- `RUNS.jsonl` — reloadable run index for local/pod/app jobs
- `runs/` — transcripts
- `outputs/` — generated artifacts and handoff files
- `proposals/` — patch/write/publish proposals before approval
- `approvals/` — explicit approve/reject records
- `NEXT.md` — immediate next action

The Agent Team board can list and resume existing mission packs through `host.missions.list()`. Resuming a mission restores the mission badge, task graph, and context prompt.

### Team presets

Atomek does not ask the user to manually understand every tool. The front door offers presets generated from the live resource graph:

- **Repo Repair** — local implementer plus independent reviewer.
- **OpenClaw + Local** — OpenClaw/Hermes pod perspective plus local Claude/OpenCode/Codex/pi execution.
- **Creative Production** — app skills such as JULI3TA, Blender, and Remotion plus shared assets.
- **Research Watch** — pod/AIL research, local synthesis, shared-folder handoff, optional channels.

Each preset maps roles to real resources: planner, implementer, reviewer, Team Desk, and app tool when relevant. Missing resources show as setup-needed instead of fake availability.

### Resource graph

The setup view shows resources as a graph: pods, local agents, apps, shared folders, app skills, and the active workspace. Use **Use** to attach a resource to the current mission prompt. Use **Setup** when a missing dependency needs a local install command or app deep link.

### Task graph

The default mission task graph is deliberately small:

1. scope the mission and context with the planner role
2. execute or produce an artifact through the implementer/app role
3. run app-skill work when relevant
4. review, approve, and prepare handoff with the reviewer role

This keeps Atomek useful immediately while leaving room for richer multi-agent orchestration later.


## Docs & Skills inside Atomek

Atomek includes a small **Docs & Skills** area on the Agent Team home screen. Open a guide to load it as a markdown tab, then ask Atomek chat about the active guide or combine it with files from the current workspace.

Built-in guides cover:

- Tytus Resource Fabric
- OpenClaw and Hermes agent teams
- shared folders and mission folders
- practical use cases
- agentic app skills

These docs are bundled with the Atomek app so the user can ask about Tytus workflows directly inside TytusOS, even before opening external documentation.

## Ask pod

**Ask pod** sends the selected mission task to a ready pod agent through `host.daemon.callPodEndpoint()`. Atomek first asks the pod for `/v1/models`, selects the first live model returned by the pod metadata, then sends a non-streaming `/v1/chat/completions` request through the same-origin Tytus bridge. No model id is hardcoded in Atomek.

The pod response is saved like any other run:

- visible in the Runs panel
- captured in Outputs
- written under `runs/`
- indexed in `RUNS.jsonl`

If the pod gateway rejects the request or is unreachable, Atomek writes a failed run transcript instead of silently hiding the error.

## Open in Terminal

**Open in Terminal** launches the TytusOS terminal with the current workspace context. Use it when you want the full interactive shell and can supervise the command.

Good uses:

- run tests
- inspect git state
- launch a local CLI manually
- run a project command that needs a real terminal

The terminal is backed by the Tytus tray PTY bridge and starts in the relevant local workspace when possible.

## Run local job

A local job is for supervised background work by an installed local agent. It receives a selected mission task, the mission folder, selected resources, and Atomek context. Output streams into Atomek, is saved under `runs/`, and is also captured in **Outputs** so patches can become reviewable edit previews.

Atomek also writes `RUNS.jsonl` in the mission folder. That run index stores job id, tool, task, status, exit code, and transcript path so Atomek can reload mission history after a refresh or app restart. Older transcript files still appear as legacy run entries.

Use **Cancel** to stop a running job through the tray job bridge. Canceling sends a safe terminate request to the tracked child process; it does not delete the mission folder, `RUNS.jsonl`, or prior transcript output.

Rules:

- local jobs use allowlisted tools only
- outputs stream back into Atomek
- edits become previews before write/apply
- model selection remains global through AIL
- no arbitrary shell from model text

## Agentic app skills

Tytus apps can expose skills through manifests or sidecar docs. Atomek uses those skills to attach the right instructions and launch the right driver.

Examples:

| Skill | Current meaning |
|---|---|
| Atomek inspect project | Ask an agent to review the active workspace context. |
| Atomek patch preview | Ask for a unified diff or fenced replacement block for Atomek to preview. |
| Local terminal open | Open the TytusOS terminal with context. |
| JULI3TA create song | Hand off a music-generation task to JULI3TA where supported. |
| Blender MCP create scene | Use a Blender MCP/socket bridge when the Blender skill and local server are installed. |

Do not show fake support. If a skill or app driver is not installed, show it as unavailable with the missing dependency.

## Troubleshooting Atomek

| Problem | Fix |
|---|---|
| Old UI or duplicate Agent Team icons | Hard-refresh TytusOS. Confirm Atomek is loaded from `tytus-app-atomek@v0.4.22` or newer. |
| Files are listed but editor is blank | Reopen the file, then hard-refresh. If still broken, report the file type and console error. |
| Folder does not expand/collapse | You are likely on an older bundle. Refresh and check the Atomek version. |
| Chat answer appears only after completion | Streaming path is degraded. Check browser console and host `/v1/chat/completions` proxy errors. |
| Remote pod call gets CORS errors | The app is calling a remote endpoint directly. Route through the Tytus host proxy instead. |
| Local tool missing | Install the CLI/tool, then click **Refresh capabilities** in Agent Team / Setup. |
| Model picker shows an obsolete model | Update global AIL configuration. Do not hardcode the model in Atomek. |

## Contributor rules

- Keep Atomek app code in the standalone `tytus-app-atomek` repo.
- Keep TytusOS as the shell/host, not a forked Atomek implementation.
- Do not change JULI3TA internals while fixing Atomek.
- Do not hardcode AIL model IDs.
- Do not bypass the host bridge for pod, local-tool, or remote-model access.
- User-visible behavior changes must update this manual and regenerate `tytus-cli/os-docs.md`.


<!-- ==== settings.md ==== -->

# System Settings

Open Settings from the dock, desktop icon, app launcher, tray deep-link, or `#/settings/<panel>`.

Settings is split into **Tytus** panels and **System** panels. Production docs should only describe panels that actually exist and work.

## Tytus panels

| Panel | Use it for |
|---|---|
| Account | Signed-in email, plan, sign out |
| Plan & Units | Unit limit, used units, included gateway, upgrade link |
| Pods | Allocated pods, API/UI URLs, keys, status, allocate action |
| Agents | Install OpenClaw/Hermes-style agents into pods and follow install progress |
| Daemon | Local daemon health, session expiry, sign-in recovery, lifecycle buttons, autostart toggles |
| Sharing | Garagetytus/shared-folder bindings, diagnostics, defaults, cache/open-folder actions |

## System panels

| Panel | Use it for |
|---|---|
| Background | Wallpaper selection. Default design pack background is bundled. |
| Appearance | Dark/light mode, accent color, theme tokens, demo-app visibility |
| Dock | Dock size/position/order/visibility |
| Languages | UI language packs |
| Notifications | Recent notifications and notification behavior |
| Privacy | Clipboard reset, lock screen, local privacy notes |
| About | TytusOS version, daemon version/PID/uptime, Tytus Home, update status |

Removed early placeholder panels such as fake Wi-Fi, Bluetooth, printers, mouse, keyboard, display, and battery. Do not reintroduce decorative OS controls unless they are backed by real product behavior.

## Session expired

When the daemon refresh token expires, Settings -> Daemon shows **Session expired** and a **Sign in again** card.

Important behavior:

- Running pods stay online.
- Local files are not deleted.
- The user signs in again through the browser one-time flow.
- After approval, TytusOS should refresh state automatically. If it does not, use **Check session** or reload the page.

## Daemon state labels

| Label | Meaning | User action |
|---|---|---|
| Connected | Daemon authenticated and healthy | none |
| Session expired | Login refresh failed, pods may still run | Sign in again |
| Degraded | Daemon reachable but one subsystem failed | Open Settings -> Daemon, run Doctor/logs |
| Offline | TytusOS cannot reach local daemon | start tray daemon, check localhost port |

## Updates

About should show the installed TytusOS/daemon version and update state when available. Manual update checks belong in About or Daemon, not hidden in Tower.

## Theme rule for contributors

Use shared semantic tokens for foreground/background/border/accent. Never hard-code black icons or text into product UI; it breaks dark mode. New components must be checked in both light and dark mode before shipping.


<!-- ==== apps-catalog.md ==== -->

# Apps Catalog

TytusOS apps fall into two groups:

1. **Product surfaces** — required for daily Tytus usage and backed by the daemon/pods.
2. **Optional/demo utilities** — OS-feel tools or demos that must not block core pod workflows.

## Product surfaces

| App | What it does |
|---|---|
| Pod Inspector | Fleet overview, included gateway, readiness, pod detail tabs, restart/doctor/log/env/copy actions |
| System Settings | Account, Plan & Units, Pods, Agents, Daemon, Sharing, Background, Appearance, Dock, Languages, Notifications, Privacy, About |
| Files | Finder-like browser for `~/Tytus`, Inbox, Outbox, Downloads, Shared, and pod workspaces |
| Channels | Per-pod messenger/channel setup with token-safe flows |
| Terminal | Host-backed shell through the local tray daemon, starting in `~/Tytus` |
| Atomek | Monaco workbench for files, chat, artifacts, AIL routing, Resource Fabric cockpit, and app skills |
| Browser | Registered launchers and safe web/app links |
| Help | Bundled manual, troubleshooting, diagnostic links |
| Chat | Opens agent chat surfaces and pod UIs |
| Music Creator | Tytus music/lyrics workflow using the included gateway |

## Included gateway

The All LLM Gateway is not a normal pod app. It is always included, OpenAI-compatible, and exposed in Pod Inspector with private/public URLs and copy formats.

## Demo and utility apps

Demo utilities can be present behind **Settings -> Appearance -> Show demo apps**. Keep them clearly marked and never document them as required platform capability.

Examples: games, ASCII Art, Matrix Rain, local notes/todos/calculator, API Tester, media viewers.


## Agentic app skills

Some apps publish skills that Atomek and other agent surfaces can attach dynamically. A skill may describe how to inspect a project, generate a patch preview, launch a local tool, drive a media app, or connect to an external app bridge such as Blender MCP.

Rules for production skills:

- declare the dependency honestly
- show unavailable when the dependency is not installed
- run through the Tytus host bridge
- never direct-fetch pod/model endpoints from the browser
- never hardcode AIL model IDs in app code
- return reviewable artifacts or patch previews before writing files

## App documentation rule

If an app appears in the production launcher/dock by default, its manual entry must answer:

- What real backend or local storage it uses
- What user problem it solves
- What is not implemented yet
- Where to troubleshoot it

If the answer is “nothing real”, keep the app hidden behind the demo-app toggle.


<!-- ==== troubleshooting.md ==== -->

# Troubleshooting

## TytusOS says Session expired but the tray says Connected

This means the local daemon is running and pods may still be online, but the browser view has stale auth state.

Fix:

1. Open **Settings -> Daemon**.
2. Click **Sign in again**.
3. Approve the one-time browser login.
4. Click **Check session** or reload TytusOS if the status does not refresh.

Pods are not deleted by this flow.

## Pod shows Not ready but the URL opens

Readiness is stricter than “the browser URL returns something”. TytusOS checks allocation, API health, UI route, shared-storage helper, and smoke/bootstrap status.

If the pod UI opens but status is Not ready:

- Open Pod Inspector -> pod detail.
- Check which readiness row failed.
- Use **Doctor**, **Logs**, **Refresh creds**, or **Restart**.
- If only shared storage is degraded, core chat/API may still work.

## Music Creator or browser fetch gets CORS errors for `/v1/models`

Browser apps should use the Tytus local proxy or included gateway path exposed by the daemon. Direct browser calls to pod public URLs can fail CORS preflight.

Fix for contributors: route browser-side gateway probes through the daemon or same-origin proxy, not directly to `https://pod.tytus.traylinx.com/v1/models`.

## Files shows raw `tytus ls: no such path`

That is a UI bug unless the user explicitly opened a diagnostic log. Missing `Inbox` or `Downloads` should show a friendly empty state with a create/refresh action.

## Terminal output duplicates or breaks after resizing

The terminal must notify the PTY backend of row/column changes and clear/reflow correctly. If output duplicates after resize, test with `stty size`, resize again, and inspect terminal resize events.

## Copy/paste shortcuts do not work in Terminal

Browser shortcuts differ by OS. The terminal should support:

- macOS: Cmd+C/Cmd+V for copy/paste when text selection is active; Ctrl+C goes to the shell process.
- Windows/Linux: Ctrl+Shift+C/Ctrl+Shift+V for terminal copy/paste; Ctrl+C interrupts the shell process.

If this regresses, compare against Ghostty behavior and ensure the terminal app handles platform-specific modifier keys.


## Atomek editor is blank or files are not editable

A blank editor with a normal text file usually means an old Atomek bundle is cached or the file type did not get a text model.

Fix:

1. Hard-refresh TytusOS.
2. Reopen Atomek.
3. Reopen the file from Explorer.
4. Confirm Atomek loads `tytus-app-atomek@v0.4.22` or newer.
5. If still broken, include the file extension and browser console error in the bug report.

## Atomek folder rows do not expand or collapse

Use the folder chevron or click the folder row. If nothing changes, you are likely on an old bundle. Hard-refresh TytusOS and reopen the folder.

## Atomek shows duplicate Agent Team and stale App Skills icons

That was an old app bundle. The current surface has one **Agent Team** activity. Hard-refresh TytusOS and confirm the app comes from `tytus-app-atomek@v0.4.22` or newer.

## Atomek local tools are missing

Open **Atomek -> Agent Team** and click **Refresh capabilities**. If a tool is still missing, install the local CLI first, then refresh again. Atomek only launches allowlisted tools discovered through the host bridge.

## Atomek model list shows an obsolete model

AIL model selection is global. Fix the global AIL configuration or selected route. Do not hardcode a replacement model in Atomek or TytusOS app code.

## `garagetytus-shared` missing inside a pod

Core chat/API may work, but shared S3 bucket access will be degraded.

Fix path:

- Rebuild/restart the pod with the current Tytus agent image.
- Verify the helper exists in the container.
- Re-run Pod Inspector readiness.

## `ail-speech` returns 429

This is provider quota/rate limiting. It is not fixed by restarting TytusOS. Use another model/provider if available or wait for quota recovery.

## Vision works with image URLs but not inline base64

Use public URLs or upload the image first. Inline `data:image/...;base64` may be rejected by the upstream multimodal endpoint.

## Theme looks wrong in dark mode

Hard-coded black icons/text cause this. Capture the screen, identify the component, and replace hard-coded colors with theme tokens.

## Reset local browser state

Only do this when instructed; it resets UI preferences, not your server account:

```js
localStorage.clear()
location.reload()
```

## Support checklist

When reporting a bug, include:

- TytusOS version and daemon version from Settings -> About
- Daemon state from Settings -> Daemon
- Pod Inspector readiness rows
- Browser console error
- Exact route/hash URL
- Whether legacy Tower shows different state

## Shared folder or mission handoff is confusing

Use **Help -> Shared Folders** for the full exchange model. Short version: put raw incoming context in `INBOX.md`, keep transcripts in `runs/`, put generated artifacts in `outputs/`, put proposed edits in `proposals/`, and only move final approved material to `OUTBOX.md`.

## OpenClaw or Hermes does not appear in Atomek

Open **Pod Inspector** first and confirm the agent is allocated and ready. Then return to **Atomek -> Agent Team** and refresh capabilities. Atomek shows real resources only; missing agents are setup work, not UI failure.


<!-- ==== about.md ==== -->

# About TytusOS

TytusOS is the desktop interface for Tytus, the private AI pod product by Traylinx. It turns the local tray daemon, pod fleet, included gateway, files, channels, and app workflows into one browser desktop.

## Names

- **TytusOS** — the desktop UI.
- **Tytus** — the product family and CLI/tray daemon.
- **Tytus Home** — the local workspace at `~/Tytus`.
- **Pod** — a private agent runtime allocated by Tytus.
- **All LLM Gateway** — included OpenAI-compatible gateway, free/included and not counted against pod units.
- **Tower** — legacy web UI. Use only as rollback while cutover finishes.

## What users should open

- Tray menu -> **Open TytusOS**
- `tytus open`
- local URL served by the tray daemon, normally `http://localhost:<tray-port>`

## What belongs in TytusOS

- Pod readiness and install progress
- Gateway URLs and env copy formats
- Files and shared folders
- Channels and messenger setup
- Terminal and streamed command output
- Settings, updates, session expiry, daemon health
- Help/manuals/support flows

## Documentation surfaces

- In-app Help / docs registry
- `tytus os-docs`
- `tytus link [DIR]`
- Central handbook: `~/Documents/TYTUS-OS/`

## Built with

- Vite + React + TypeScript
- Tray daemon HTTP/SSE bridge from `tytus-cli`
- Browser terminal through local daemon shell bridge
- CSS theme tokens for dark/light/accent consistency


---

<!-- ==== troubleshooting/clipboard.md ==== -->

# Host Clipboard — Per-Browser Behaviour

TytusOS's **Cmd+V on the Desktop** (Sprint B Phase 5.4) reads from the
host browser's clipboard. The browser's `navigator.clipboard` API isn't
uniform across vendors — this doc maps what works where and how Tytus
degrades when something doesn't.

## Quick answer

| Browser | Image paste | Text paste | Permission UX |
|---|---|---|---|
| **Chromium / Chrome / Edge** | ✅ via `navigator.clipboard.read()` | ✅ | One-time prompt, cached per origin. |
| **Safari 17+** | ⚠️ per-call permission, may prompt every paste | ✅ | Permission per call; sometimes denied silently. |
| **Safari < 17** | ❌ `read()` unavailable | ✅ via `readText()` | Text-only fallback. |
| **Firefox** | ❌ `read()` unavailable | ✅ via `readText()` | Text-only fallback; may also prompt per call. |
| **Tytus bundled WebView** (tytus-cli) | ✅ — Chromium-based | ✅ | Pre-granted in the bundled profile. |

## How Tytus reads the clipboard

`app/src/lib/hostClipboard.ts` `readClipboard()`:

1. Probe `navigator.clipboard.read` — present on Chromium / recent Edge /
   recent Safari. If yes:
   - Iterate items; **prefer image/* over text/plain** (most users paste
     an image expecting an image, not a hex blob).
   - On any clipboard `image/png|jpeg|gif|webp`, return an `image` payload
     with a generated filename `pasted-YYYYMMDD-HHMMSS.<ext>`.
   - Else look for `text/plain` and return it.
2. Fall back to `navigator.clipboard.readText()` — Firefox + older Safari.
   Always returns a `text` payload or empty.
3. If neither is exposed, return `unavailable` so Tytus can show
   "Clipboard unavailable" in a toast.

## Permission states

Tytus caches the permission decision in `state.clipboardPermission`:
`'granted' | 'denied' | 'prompt'`. On every Cmd+V:

- **'granted'** → call `read()` directly; if the call returns successfully,
  cache stays `granted`.
- **'denied'** → still attempt the read (browser may have re-granted via
  site settings). On `NotAllowedError`, show one toast — no loop.
- **'prompt'** → the browser surfaces its native dialog. The result
  updates the cache immediately.

**Permission recovery**: a successful read always upgrades the cache to
`'granted'`, even if it was `'denied'` before. So a user who reset their
browser permission via the site-info menu doesn't have to also reset
inside Tytus.

## Manually resetting

If Tytus's cache says `'denied'` but you've granted permission at the
browser level:

1. Open **Settings → Privacy → Reset clipboard permission**.
2. The "Will ask" status appears.
3. Press **Cmd+V** again — Tytus re-attempts and the browser will show
   its current verdict.

## Browser-specific quirks

### Chromium / Chrome / Edge

- The first Cmd+V triggers the browser's clipboard prompt.
- After grant, the `read()` call works for image AND text.
- Permission survives reload via the site setting.
- **Bundled tytus-cli WebView** ships with the permission pre-granted in
  its Chromium profile.

### Safari 17+

- `navigator.clipboard.read()` exists but is permissioned **per call** —
  Safari may prompt every paste depending on user setting.
- Sometimes denies silently if not invoked in a clear user-gesture
  context. Tytus binds Cmd+V directly to the keypress (no async
  trampoline) so Safari sees the gesture.

### Safari < 17

- `read()` doesn't exist; only `readText()` works.
- Image paste returns "unavailable" → toast suggests upgrading.

### Firefox

- `read()` is not implemented as of Firefox 124.
- `readText()` works but may prompt per call.
- The `permissions.query({name: 'clipboard-read'})` call rejects
  ("clipboard-read unknown") — Tytus falls back to `'prompt'` and
  attempts the read at gesture time.

## Settings the user can change

| Setting | Where | Effect |
|---|---|---|
| Clipboard permission | Settings → Privacy | Reset cached permission to `'prompt'` |
| Notifications | Settings → Notifications | Mute the post-paste confirmation toast / chime |

## Internals

- `app/src/lib/hostClipboard.ts` — the wrapper + browser detect.
- `app/src/App.tsx` — Cmd+V handler registered via the shortcut router
  at active-app scope.
- 17 unit tests in `lib/hostClipboard.test.ts` cover all permission paths.
- The internal Tytus clipboard (Cmd+C inside the OS) is separate —
  `app/src/lib/clipboard.tsx`. It never touches `navigator.clipboard`.


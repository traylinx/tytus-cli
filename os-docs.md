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

| Need | Open |
|---|---|
| See pods and gateway URLs | Pod Inspector |
| Copy OpenAI-compatible env vars | Pod Inspector -> Copy env |
| Browse local workspace | Files -> Tytus Home |
| Browse a pod workspace | Files -> Pod NN workspace |
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
- Settings for account, daemon, sharing, appearance, dock, language, privacy, updates
- Music Creator and other Tytus apps that use the included gateway

Demo/optional apps may exist behind the demo-app toggle. They must not block core pod workflows.


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

Files is the Finder-like browser for Tytus. It covers local Tytus Home, shared folders, and pod workspaces.

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

## Safety

File operations must be root-anchored to the selected source. Path traversal, symlink escape, null bytes, and double-encoded traversal must be rejected by daemon-side tests before write operations ship broadly.

## Shared folders

Shared folders use the account-level sharing system. Manage global defaults and diagnostics in **Settings -> Sharing**. Use Files -> Shared for browsing/opening the configured source.


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
| Browser | Registered launchers and safe web/app links |
| Help | Bundled manual, troubleshooting, diagnostic links |
| Chat | Opens agent chat surfaces and pod UIs |
| Music Creator | Tytus music/lyrics workflow using the included gateway |

## Included gateway

The All LLM Gateway is not a normal pod app. It is always included, OpenAI-compatible, and exposed in Pod Inspector with private/public URLs and copy formats.

## Demo and utility apps

Demo utilities can be present behind **Settings -> Appearance -> Show demo apps**. Keep them clearly marked and never document them as required platform capability.

Examples: games, ASCII Art, Matrix Rain, local notes/todos/calculator, API Tester, media viewers.

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


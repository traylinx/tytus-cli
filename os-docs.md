# Tytus OS — User Manual (LLM-facing reference)

> You are an AI agent (Claude Code, OpenCode, KiloCode, Gemini, Codex,
> Cursor, Vibe, etc.) running on a user's machine that has Tytus OS
> installed. This document is the complete user manual, concatenated
> from the source-of-truth markdown in the tytus-os repository. Read
> it BEFORE answering any question about how Tytus OS looks, behaves,
> or what its keyboard shortcuts / settings panels / apps do.
>
> The companion reference for the `tytus` CLI itself is `tytus llm-docs`.
> This document covers the desktop OS surface; `llm-docs` covers the
> command-line tool.

---


<!-- ==== getting-started.md ==== -->

# Getting Started

Tytus OS is a web-based desktop. It runs entirely in your browser and looks like a real operating system: boot screen, login, desktop, dock, draggable windows, app launcher, notifications.

## First launch

```bash
cd app
npm install
npm run dev
```

Open **http://localhost:4242** in your browser.

You'll see four phases:

1. **Boot** (~4s) — black screen with the Tytus OS logo (purple/orange/pink dual orb), then a progress bar, then a circle-iris transition reveals the wallpaper.
2. **Login** — blurred wallpaper with a centered card. Click **Unlock** (any password works at this stage) or **Log in as Guest**.
3. **Desktop** — wallpaper, top panel, icons on the left, dock at the bottom.
4. **Apps** — click any icon or use the launcher.

## The desktop in 30 seconds

- **Top panel** (28 px tall) — `Apps` button on the left, clock + date in the middle, system tray on the right (Wi-Fi, volume, battery, power menu). Buttons are 24 px tall so the hover background stays inside the panel.
- **Wallpaper** — replaceable from Settings → Background.
- **Desktop icons** — 8 by default (Pods, Files, Terminal, Settings, Chat, Channels, Browser, Help). Drag to rearrange (snaps to 80×90 grid). Right-click for the context menu.
- **Dock** (bottom-center) — floats 6 px from the viewport with rounded corners on all four sides. Apps grid button on the far left, then 6 pinned apps, then any unpinned-but-open apps, then the trash. Open apps show a small dot near the bottom of their icon (inside the dock).

## Opening an app

Three ways:

1. **Double-click** any desktop icon
2. **Click** any dock icon
3. **Press the Super key** (⌘ on Mac, Win on Windows/Linux) to open the app launcher → search or click

The app opens in a draggable window in the middle of the screen.

## Closing an app

- Click the **×** (close) button in the window's title bar
- Or press **Ctrl+W** while the window is focused

## Keyboard shortcuts (the essentials)

| Shortcut | Action |
|---|---|
| **⌘+Space** | Toggle app launcher |
| **⌘+W** | Close focused window |
| **⌘+Q** | Close every window of the focused app |
| **⌘+Z** | Undo the last file operation |
| **⌘+D** | Minimize all windows |
| **Alt+Tab** | Switch between open windows |
| **Ctrl+Alt+T** | Open Terminal |
| **Esc** | Close launcher / notification center / any modal |

Full reference: [keyboard-shortcuts.md](keyboard-shortcuts.md).

## Personalize Tytus in 90 seconds

Open **Settings** (dock icon or Apps → Settings):

1. **Appearance → Accent color** — click any swatch or pick a custom hex. The change ripples instantly across every app.
2. **Appearance → Text size** — slider from 50% to 150%. Tytus rescales its rem-based layouts live.
3. **Appearance → Light/dark schedule** — Manual / Always light / Always dark / Auto (light 06:00–18:00 local).
4. **Appearance → Reduce motion** — flips off the open / close / snap animations. Tytus also auto-respects your OS preference.
5. **Background → Wallpaper** — pick a bundled preset, upload your own, or set a solid color. Toggle "Match lock screen" if you want the lock/login surface to mirror your desktop.
6. **Dock** — change Position (bottom / left / right), Size (small / medium / large), Auto-hide on/off. Drag-and-reorder dock apps directly in the Dock; **Reset Dock order** restores defaults.
7. **Notifications → System sounds** — turn the chime on or off.
8. **Privacy → Reset clipboard permission** — if Cmd+V isn't working after you previously denied it.

Personalization survives reload (it's persisted via the same hydration/normalizer path as everything else).

## Snap, paste, and drag

- **Drag a window** near the **left** / **right** / **top** edge → translucent overlay shows where it'll snap. Release to commit. Drag away (~24 px) to restore the prior frame.
- **Drag a file from the Files window** OUT to Finder (Chromium build) → file downloads on drop.
- **Cmd+V on the Desktop** with an image on your host clipboard → Tytus saves it as `pasted-YYYYMMDD-HHMMSS.png`. First time prompts for permission; cache after.
- **Drag a JULI3TA track** onto an open MusicPlayer window → playback starts.

## What's not real (yet)

Tytus OS today is a **shell** — it looks and feels like an OS, but most apps are visual placeholders or use fake (browser-local) data. The real magic lands when Tytus OS connects to your private AI pod via the `tytus-cli` daemon. That's the [phase plan](../development/roadmap.md).

What *is* real today:
- All window management (drag, resize, focus, min/max/restore, Alt+Tab)
- Desktop icons + dock + launcher
- Theme (dark/light, accent colors)
- localStorage persistence for desktop layout, notes, todos, calendar events, etc.
- API Tester actually sends real HTTP requests
- Calculator, games, image/video viewers, code editor — all real and functional

What's a placeholder:
- Pod Inspector, Channels, Help (waiting for the pod backend)
- Wi-Fi, Bluetooth toggles in Settings (decorative)
- Terminal commands are simulated (real `tytus exec` arrives in Phase 6)

Continue to [windows.md](windows.md) for window controls, or jump to the [apps catalog](apps-catalog.md) to see what's installed.


<!-- ==== windows.md ==== -->

# Windows

Every app in Tytus OS runs in a window. Windows behave like a real OS: drag, resize, focus, minimize, maximize, restore.

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

The desktop is the area between the top panel and the dock. It hosts the wallpaper and your icons.

## Default icons

8 shortcuts on the left:

- **Pods** → Pod Inspector
- **Files** → File Manager
- **Terminal** → Terminal
- **Settings** → System Settings
- **Chat** → Chat
- **Channels** → Channels
- **Browser** → Browser
- **Help** → Help

## Icon actions

| Action | What happens |
|---|---|
| **Double-click** | Opens the app |
| **Click and drag** | Moves the icon (snaps to 80 × 90 grid) |
| **Right-click** | Context menu (Open / Cut / Copy / Rename / Move to Trash) |
| **Single click** | Selects the icon (purple dashed outline) |

Icon positions persist in `localStorage` under the key `tytus_desktop_icons`. They survive page reload but reset if you clear browser data.

## Right-click on empty desktop

| Item | Action |
|---|---|
| **New Folder** | (placeholder) |
| **New Document** | (placeholder) |
| **Open in Terminal** | Launches Terminal |
| **Change Background** | Launches Settings → Background |
| **Arrange Icons** | Auto-aligns to grid |
| **Display Settings** | Launches Settings → Display |

## Wallpaper

The default wallpaper is `/wallpaper-default.jpg` (a ~2 MB abstract image bundled with the app). Change it from **Settings → Background**.

The wallpaper is rendered as a fixed-position layer behind everything. It scrolls with the desktop and is covered by windows + dock.

## Selection

Click an icon to select it; click empty desktop space to deselect. Selected icons get a translucent purple background and a dashed outline.

Selection is purely visual — there's no multi-select today.

## Trash

The trash icon lives at the right end of the dock (not the desktop). Drag desktop icons there to "trash" them — for now this just removes them from the desktop layout. (Real recoverable trash is part of [the FileManager phase](../development/roadmap.md).)


<!-- ==== dock.md ==== -->

# Dock

The dock is the floating bar at the bottom-center of the screen. It is lifted 6 px from the viewport edge so it visually floats — the rounded corners on all four sides are intentional, not a clip.

```
┌─────────────────────────────────────────────────────┐
│  ⊞ │ 📦 ⚙️ 💬 📁 ✈️ ⌨ │ 🌐 │ 🗑️                    │
└─────────────────────────────────────────────────────┘
   1     2 - 7        8     9
```

1. **Show Applications** — opens the app launcher (same as ⌘ key)
2-7. **Pinned apps** — Pod Inspector, Settings, Chat, Files, Channels, Terminal (default)
8. **Open unpinned apps** — appear here while running, vanish when closed
9. **Trash**

## Active indicator

Open apps show a small dot near the bottom of their icon (inside the dock — never clipped at the viewport edge):
- **Accent-colored dot** (purple by default) when the window is focused
- **Grey dot** when open but not focused
- **No dot** when no window of that app is open

## Click behavior

- **No window open** → opens a new window (icon bounces 400 ms)
- **Window minimized** → restores and focuses it
- **Window already open** → focuses it
- **Multiple windows** → focuses the most-recently-active one

## Hover

Hover any icon to see its name in a tooltip above the dock.

## Trash

Click to open Files (placeholder — will become the recoverable Trash app in a later phase).

## Customizing pins

Today the pinned set is fixed in code (`getDefaultDockApps()` in `app/src/apps/registry.ts`). User-configurable pinning ships in a later phase.

To change the defaults right now, edit:

```ts
// app/src/apps/registry.ts
export const getDefaultDockApps = (): string[] => [
  'pod-inspector',
  'settings',
  'chat',
  'filemanager',
  'channels',
  'terminal',
];
```


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

Every shortcut Tytus OS responds to today.

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

A handful of combos are intercepted before Tytus OS sees them:

- `Cmd+Tab` — handled by macOS itself (system app switcher). Tytus ships **Alt+Tab** as the in-OS alternative.
- `Cmd+R` / `F5` — reloads the page (you'll lose unsaved app state).
- `Cmd+T` / `Cmd+N` — open a browser tab / window.

Tytus's shortcut router default-blocks `Cmd+W`/`Cmd+Q`/`Cmd+R`/`Cmd+T`/`Cmd+N` so those host-browser bindings can't kill the WebView from inside the OS, and registers its own handlers for `Cmd+W` and `Cmd+Q` on top.

## Internals

The shortcut router lives at `app/src/lib/shortcuts.ts`. It dispatches by scope priority `text-input > modal > active-app > shell`, so a focused text field always wins for combos like `Cmd+C`/`Cmd+V`/`Cmd+Z`.


<!-- ==== files.md ==== -->

# Files

Tytus OS has two parallel file backends. Knowing the difference makes the
selection / drag / paste behaviour predictable.

## Two backends

| Backend | Where bytes live | Used by | Lifetime |
|---|---|---|---|
| **vfs** | `localStorage` (`tytus_filesystem`) | Desktop icons, in-OS-only files | Browser-local. Cleared with site data. |
| **daemon** | The Tytus daemon's filesystem (`~/Tytus/...`, shared folders, pod workspaces) | Files window, real OS-level files | Real disk. Survives reload. |

A **FileRef** (in code: `lib/files/fileRef.ts`) is a tagged pointer that says
which backend a file is in plus enough info to reach it. Every file action
(move, copy, delete, paste) flows through this typed reference so a vfs
icon and a daemon row are interchangeable to the user.

## The Files window (daemon-backed)

Open **Files** from the dock or App Launcher. Browse, search, double-click directories, and use right-click for the per-row menu.

### Selection

| Action | What happens |
|---|---|
| Click a row | Select that one row. |
| **Cmd+Click** (Ctrl+Click) | Toggle selection — keep the rest. |
| **Shift+Click** | Select range from the last clicked row. |
| **Cmd+A** | Select every row currently visible (search-respecting). |
| **Esc** | Clear selection. |

Selected rows tint with the accent color. The selection set is component-local — closing the window clears it.

### Drag inside Tytus

- Drag one or more rows to the **Desktop** to create shortcut icons.
- Drag onto the **Trash icon** in the Dock to send to trash.
- Drag a track row from MusicCreator onto **MusicPlayer** to start playback.

The drag ghost shows a small badge with the count when more than one item is selected.

### Drag OUT to the host (Chromium build)

Drag a file row OUT of Tytus onto the macOS Finder, another browser tab,
or a desktop folder. Tytus emits a `DownloadURL` MIME alongside the typed
file payload so the host browser kicks off a download on drop.

- **Chromium / Chrome / Edge**: works.
- **Firefox / Safari < 17**: not supported — use the row's right-click "Download" instead.
- Single-file drags only (the `DownloadURL` envelope can't carry a multi-file download).

## Drop external files INTO Tytus

Drop one or more files from the macOS Finder (or another browser tab)
anywhere in Tytus that accepts files:

- **Desktop** — accepts file drops; the upload pipeline runs and an icon appears.
- **Files window** — accepts native files into the current pane.
- **Dock app icons** — accepts files routed to that app's open-with handler.

Tytus detects external files via `e.dataTransfer.files.length > 0` and
short-circuits the typed-payload path so external drops always win.

## Paste from the host clipboard (Cmd+V)

After copying an image (e.g. macOS Preview → ⌘+C) or text on the host
side, **Cmd+V** anywhere in Tytus that isn't a focused text input
attempts a clipboard read:

1. The first time you press Cmd+V the browser shows a permission prompt.
2. Granted → image becomes a file (`pasted-YYYYMMDD-HHMMSS.png`); text becomes a notification toast.
3. Denied → a single toast with the browser name; no permission re-prompt loop.

The permission state is cached in `state.clipboardPermission`. If you previously denied and want to re-prompt: **Settings → Privacy → Reset clipboard permission**.

Browser support matrix: see [troubleshooting/clipboard.md](../troubleshooting/clipboard.md).

## Conflict resolution

Move / copy / paste operations that hit a name collision present a dialog with four options:

- **Replace** — overwrite the destination.
- **Keep both** — append a deterministic suffix `name (2).ext`. The
  suffix is computed against the destination's actual case-insensitive
  contents (HFS+ semantics) so you never get duplicates.
- **Skip** — leave the destination alone.
- **Cancel all** — abort the entire batch. Items already moved stay.

The dialog has an **Apply to all** checkbox so a 50-item paste with
multiple conflicts only asks once.

## Partial-failure semantics

A multi-item move / copy / delete returns a `PerItemResult[]` array.
The summary toast tells you how many succeeded and surfaces failure
reasons (`not-found`, `read-only`, `quota-exceeded`, `network-error`)
with a per-item Retry button for transient failures.

## Trash

Items deleted from vfs live in localStorage trash; daemon items will
move to `~/Tytus/.Trash/` when the daemon endpoints land (deferred).

- Drag any file or icon onto the **Trash** in the Dock to send it.
- The Trash icon shows a count badge.
- Empty trash from Trash's right-click menu — plays the empty-trash chime.

## Undo

**Cmd+Z** undoes the last reversible file operation. Tytus keeps a 5-deep
ring (most recent on top). The undo entry is per-operation, so a 7-item
move-with-3-failures undo only reverses the 7 that succeeded.

## Internals

- `lib/files/fileRef.ts` — the discriminated union + helpers.
- `lib/files/fileOps.ts` — the unified API (move / copy / delete / rename / paste).
- `lib/files/conflict.ts` — case-insensitive collision detection + name suggestions.
- `lib/repo/trash.ts` — the trash façade.
- `lib/clipboard.tsx` — internal Tytus clipboard.
- `lib/hostClipboard.ts` — host browser clipboard wrapper.
- `lib/undo.ts` — the undo ring.


<!-- ==== settings.md ==== -->

# System Settings

Open from the dock (gear icon), the desktop shortcut, or the launcher → **Settings**. The Settings hash deep-link family (`#/settings/<panel>`) is also addressable from anywhere in the OS — top-panel chips, tray menus, and zero-pods overlays all use them.

## Sidebar layout

The sidebar groups panels in two halves separated by a divider:

- **Tytus** — your private AI pod product configuration. Live, wired to the local daemon.
- **System** — OS-feel preferences. All real and persisted (Sprint A retired the long list of decorative-only panels).

Search the sidebar with the **Search settings…** input at the top — the filter flattens both groups into a single result list. The active panel persists across reloads under `localStorage.tytus_settings_active_category`.

## Tytus panels

| Panel | What it covers |
|---|---|
| **Account** | Email, current tier, units used / units limit. Red **Sign out** button → confirmation modal that warns sign-out revokes every allocated pod. |
| **Plan & Units** | Current tier, unit-budget bar, breakdown by pod (e.g. *Pod 02 · nemoclaw — 1 unit*). External **Upgrade plan** CTA, **Refresh** button. |
| **Pods** | List of allocated pods. Each row exposes copy + reveal affordances on `api_url`, `public_url`, `ui_url` (token-masked), and `user_key` (Secret-masked). Status dot per pod from the lazy `/api/pod/ready` probe. **+ Allocate** jumps to the Agents panel. **Refresh** re-pulls daemon state. |
| **Agents** | Catalog grid with tier + unit-budget gating. Already-running agents show an *"Already running on pod NN"* badge. **Install** opens a wizard modal: confirm → SSE stream pane → success or **Retry**. One-click install via the `#/settings/agents?install=auto` deep link. |
| **Daemon** | Start / Stop / Restart buttons, autostart-tray + autostart-tunnel toggles, status pill (PID, uptime, tunnel state, keychain health). |
| **Sharing** | Garage shared-folder bindings — bind a Mac folder to one or more pods, configure per-binding auto-sync, see live sync indicators. |

## System panels

All System panels are real and persistent. Most preferences flow through the theme normalizer (`lib/theme/normalize.ts`) so old `tytus_settings` blobs upgrade automatically.

| Panel | What it covers |
|---|---|
| **Background** | Wallpaper picker — bundled presets, upload your own, or set a solid color. **Match lock screen** toggle mirrors the lock/login surface to the desktop wallpaper. |
| **Appearance** | Dark mode, accent color (8 presets + custom hex), font scale (50–150%), light/dark schedule, reduce-motion override. |
| **Dock** | Position (bottom / left / right), size (small / medium / large), auto-hide, **Reset Dock order** to restore default pinned apps. |
| **Languages** | Switch UI locale; install / remove official packs from `traylinx/tytus-os-language-index`; sideload third-party packs by URL or file with checksum verification. |
| **Notifications** | Recent notifications list (5 most recent). **System sounds** toggle. |
| **Privacy** | **Reset clipboard permission** escape hatch + privacy-statement copy + **Lock Screen Now** button. |
| **About** | Tytus OS version, daemon PID + version + uptime, update-status banner + link to release notes. |

## Appearance — every control

The most-used System panel. Everything in here re-renders every CSS variable instantly (no reload).

### Dark Mode
Toggle the global theme. Persists to `state.theme.darkMode`.

### Accent Color
- **8 presets**: Purple (default), Blue, Teal, Green, Yellow, Orange, Red, Pink.
- **Custom**: a small color-swatch button next to the presets opens a native color picker. Any 6-digit hex is accepted. The custom value persists separately so you can switch back to a preset and recover the custom one.

The accent ripples through buttons, focus rings, dock indicators, the boot logo, the App Launcher highlight, and the snap overlay during window drags.

### Text Size (font scale)
A slider from **50% to 150%**, step 5%. Applies as a CSS variable on `html` so every rem-based dimension scales live. **Reset** button restores 100%.

### Light / Dark Schedule
Four modes:
- **Manual** — only the Dark Mode toggle changes the theme.
- **Always Light**
- **Always Dark**
- **Auto** — light from 06:00 to 18:00 local time, dark otherwise. The schedule re-evaluates on the minute.

### Reduce Motion
Disables window open / close animations and snap-resize tweens. Tytus also auto-respects `prefers-reduced-motion: reduce` from your OS — the in-Tytus toggle is an explicit override that survives changes to your OS preference.

### Show Demo Apps
Reveals optional OS-feel demo apps (Games + ASCII Art + Matrix Rain) in the App Launcher. Off by default in production builds. Tytus product surfaces stay visible either way. See [apps-catalog.md](apps-catalog.md) for the full demo list.

## Dock panel

| Control | Effect |
|---|---|
| **Position** | bottom (default) / left / right. Re-anchors the dock and rotates the indicators. |
| **Size** | small / medium / large. Changes icon size, padding, and the dock's footprint on the wallpaper. |
| **Auto-hide** | Dock hides off-screen when no window touches it; reveals when the cursor approaches the screen edge it lives on. |
| **Drag-and-reorder** | Click-hold-drag any dock app to reorder. Persists immediately. |
| **Reset Dock order** | Restores the default 6 pinned apps in the original order. |

## Notifications panel

- **Recent notifications** — the 5 most recent toasts, in card form (title, message). Useful when you missed a transient toast.
- **System sounds** — toggle the bundled sound theme (notification chime, error beep, empty-trash whoosh, screenshot click). When OFF, all `playSound(...)` calls are no-ops. The toggle stores in `state.theme.soundEnabled` (default ON unless `prefers-reduced-motion: reduce` *and* the OS has no media output, in which case Tytus boots with sound off).

## Privacy panel

- **Reset clipboard permission** — flips the cached `state.clipboardPermission` back to `'prompt'`. Use this if you previously denied browser-clipboard access via Cmd+V and want Tytus to ask again. Status line shows `Granted`, `Denied`, or `Will ask`.
- **Private AI shell** — informational copy. Tytus OS does not expose browser-side telemetry or fake device controls — daemon and pod state come from the local daemon only.
- **Lock Screen Now** — instantly transitions the OS to the Login phase. Useful when stepping away.

## About panel

- **Tytus OS** version and update status (banner: *up to date* / *update available*).
- **Daemon** PID, version, started-at, uptime.
- Buttons: **Run Doctor** (jumps to Help → Doctor), **Open release notes**, **GitHub**.

## Where settings live on disk

| Key (localStorage) | What it stores |
|---|---|
| `tytus_settings` | The full theme blob (dark mode, accent, custom accent, font scale, schedule, reduce motion, sound). Hydrated through `lib/theme/normalize.ts` so older blobs forward-upgrade. |
| `tytus_settings_active_category` | Last-active panel id |
| `tytus_dock` | Dock position, size, auto-hide, app order |
| `tytus_wallpaper` | Wallpaper kind + value (preset / upload / solid color) |
| `tytus_lock_match_wallpaper` | Lock/login wallpaper match toggle |
| `tytus_clipboard_permission` | Cached clipboard permission verdict (`granted` / `denied` / `prompt`) |
| `tytus_window_snap` | Per-window snap state (current snap kind + frame to restore on un-snap) |
| `tytus_window_geometry` | Per-app reopen geometry (cross-session) |
| `tytus_filesystem` | The vfs file tree (Desktop icons, Notes data, etc.) |
| `tytus_locale` + `tytus_lang_packs` | UI language + installed third-party packs |

Clear browser data → settings reset to factory defaults.

## Hash deep-links

Every panel is addressable directly so other surfaces can link straight to a specific control:

- `#/settings/account`
- `#/settings/plan`
- `#/settings/pods`
- `#/settings/agents`
- `#/settings/agents?install=auto` — open the install wizard immediately
- `#/settings/daemon`
- `#/settings/sharing`
- `#/settings/background`
- `#/settings/appearance`
- `#/settings/dock`
- `#/settings/language`
- `#/settings/notifications`
- `#/settings/privacy`
- `#/settings/about`

Tray menus, top-panel chips, the zero-pods overlay, and the in-OS Help app all use these links to deep-jump.


<!-- ==== apps-catalog.md ==== -->

# Apps Catalog

Every app installed in Tytus OS today — **50 apps in 8 categories**. The Tytus product surfaces (Pod Inspector, Settings, Help, Chat, Files, Channels, Browser) are wired to the daemon — no placeholders. The OS-feel utilities are real and self-contained (Notes / Todo / Calendar all persist to localStorage; API Tester sends real HTTP).

`isDemo: true` apps in `app/src/apps/registry.ts` are gated by **Settings → Appearance → Show demo apps** (off by default in production builds). They're listed below with a *(demo)* tag.

## System (9)

| App | What it does |
|---|---|
| **App Store** | Browse and discover recommended Tytus + community apps; check install status on your machine via the daemon. |
| **Pod Inspector** | Fleet Overview + per-pod tabs. Sort by *Needs attention* or *Pod ID*; search by `pod_id` or `agent_type`. Each row shows a status pill, an **Open agent UI** button, and click-through to the per-pod tab. The per-pod tab shows a status header, URLs grid, **Pin / Unpin**, an action row (**Open / Restart / Doctor / Stop forwarder / Refresh creds**), and a destructive section (**Uninstall… / Revoke…**). Streaming actions render an inline log pane. Pinned pods sort to top. **Restart all** appears when 2+ pods are allocated. |
| **System Settings** | Tytus product config (Account, Plan & Units, Pods, Agents, Daemon, Sharing) plus OS-feel preferences (Background, Appearance, Dock, Languages, Notifications, Privacy, About). See [Settings](settings.md). |
| **Help** | Sidebar tabs: **Doctor**, **Health test**, **Logs**, **About**. Doctor + Test each have a **Run** button, an SSE log pane, and a *"Last run: Xm ago · exit 0"* status line. Logs polls `/api/logs` every 2s with **Pause / Resume** and auto-scroll-when-pinned. About shows daemon PID, formatted uptime, and GitHub links. |
| **Files** | Pods sidebar plus three tabs: **Inbox** (run-streamed `ls-inbox`), **Downloads** (opens `~/Downloads/tytus/pod-NN/` via `postFilesOpenDownloads`), and **Shared** (bind a Mac folder to one or more pods via Garage — folder picker, bucket validation, auto-sync toggle). |
| **Terminal** | Simulated bash today. Real `tytus exec` into pod containers in a later phase. |
| **System Monitor** | CPU, memory, disk, network — host today, pods later. |
| **Archive Manager** | Create and extract ZIP / TAR / 7Z archives. |
| **Channels** | Pods sidebar plus **Available** and **Configured** columns. **Add** opens a modal with a `type=password` input — the token travels in the request body, never in the URL. **Remove** asks for confirmation. |

## Internet (4)

| App | What it does |
|---|---|
| **Chat** | Pods sidebar plus a main pane showing *"Pod NN ready to chat"* and an **Open Pod NN in browser** button (launches the agent UI). Inline chat planned for v1.1. |
| **Browser** | URL bar with scheme validation, registered launchers from `getLaunchers`, plus **Quick Actions** (Tytus dashboard / Provider / GitHub). |
| **Weather** | Forecast with locations. |
| **RSS Reader** | Feed reader with default subscriptions. |

## Productivity (10)

| App | What it does |
|---|---|
| **Notes** | Quick notes with folders. localStorage-backed. |
| **Todo** | Tasks with priorities, projects, due dates. |
| **Reminders** | Time-based reminders + system notification on fire. |
| **Calendar** | Monthly view with events. |
| **Calculator** | Standard 4-function with history. |
| **Clock** | World clock, alarms, timer, stopwatch. |
| **Spreadsheet** | Basic grid with formulas. |
| **Text Editor** | Plain-text editor. Reads/writes the virtual filesystem. |
| **Document Viewer** | PDF and document viewer. |
| **Markdown Preview** | Live markdown rendering with GitHub styling. |

## Media (9)

| App | What it does |
|---|---|
| **Image Viewer** | Single-image view with zoom and slideshow. |
| **Image Gallery** | Browse and organize collections. |
| **Photo Editor** | Crop, filter, adjust. |
| **Music Player** | Audio playback with playlists. Accepts JULI3TA track drops to start playback (Sprint B). |
| **JULI3TA** | "Where songs find their soul." AI lyrics + music creation, powered by your private pod. (Internal id: `musiccreator`.) |
| **Video Player** | Video playback with controls. |
| **Voice Recorder** | Microphone capture, playback, export (wav/mp3). |
| **Screen Recorder** | Browser screen capture (uses native API). |
| **Media Converter** | Format conversion utility. |

## DevTools (6)

| App | What it does |
|---|---|
| **Code Editor** | Syntax-highlighted multi-tab editor. |
| **API Tester** | Postman-clone — real HTTP requests against any URL. Headers, body, params, history, saved endpoints. |
| **JSON Formatter** | Format, validate, beautify, tree view. |
| **Regex Tester** | Test patterns against sample text live. |
| **Base64 Tool** | Encode / decode Base64 + URL strings. |
| **Color Palette** | Generate complementary color schemes. |

## Creative (3)

| App | What it does |
|---|---|
| **Drawing** | Canvas-based drawing app with brushes. |
| **Whiteboard** | Infinite canvas for sketches. |
| **Color Picker** | Pick colors, build palettes. |

## Demo apps (hidden by default)

The 11 Games plus **ASCII Art** and **Matrix Rain** are gated by the **Show demo apps** toggle in **Settings → Appearance** (default OFF — manifest AN8 demo-apps gate). Flip it on to expose:

- **Minesweeper**, **Snake**, **Tetris**, **Tic-Tac-Toe**, **2048**, **Sudoku**, **Chess**, **Memory**, **Pong**, **Solitaire**, **Flappy Bird**
- **ASCII Art** — generate ASCII text art and diagrams
- **Matrix Rain** — animated falling characters (the green movie effect)

## Shell-level surfaces

Beyond windowed apps, Tytus OS now exposes pod state directly from the shell:

- **Top Panel — Daemon status pill** — green / yellow / red / grey. Click opens **Settings → Daemon**.
- **Top Panel — Fleet Health chip** (next to the daemon pill) — pod count + active jobs, color-coded. Click opens **Pod Inspector**.
- **Desktop — Reserved Pods Zone** — top-left 4×2 grid for pinned pods. Click → opens **Pod Inspector** with that pod's tab focused. Right-click → **Unpin**. Stale pins (revoked elsewhere) render at 50% opacity.
- **Desktop — Zero-pods overlay** — appears when `state.agents = []`. CTA jumps to **Settings → Agents**.

## Window state persistence

Open windows persist across reloads — positions, sizes, minimized and maximized state. Focus order and z-index reset on reload.

## Permanently dropped (won't ship)

These were in the original Kimi seed but were cut because they imply product promises we can't keep:

- **Contacts** — fake address book (no real integration)
- **Email** — fake "send mail" creates a real safety risk
- **FtpClient** — fake FTP, dials nothing
- **GitClient** — fake git ops, misleading
- **NetworkTools** — fake ping/traceroute, dangerous
- **PasswordManager** — fake password storage, security misrepresentation

If you need any of these, use the real OS-level tool instead.


<!-- ==== troubleshooting.md ==== -->

# Troubleshooting

Common issues and how to fix them.

## Boot screen never finishes

**Symptom:** the Tytus OS logo + progress bar stays forever.

**Cause:** JavaScript error during boot (rare). The boot phase machine is `off → logo → loading → transition → desktop`; an early error stalls it.

**Fix:**
1. Open browser DevTools → Console. Look for red errors.
2. Reload the page (**Ctrl+R** / **⌘+R**).
3. If it persists: `localStorage.clear()` in the console + reload.
4. If still broken: file an issue with the console output.

## My desktop layout is gone after refresh

Layout persists in `localStorage` under `tytus_desktop_icons`. It resets if:
- You cleared site data
- You're in incognito / private browsing
- A different browser profile

To export your layout: copy the JSON from the `tytus_desktop_icons` key in DevTools → Application → Local Storage.

## A window won't close

**Symptom:** the **×** button doesn't dismiss the window.

**Causes:**
- Modal `<dialog>` open inside the app — close that first
- The app is mid-drag — release the mouse first

**Fix:** Press **Ctrl+W** while the window is focused, or **⌘+D** to minimize all and reopen.

## Can't drag a window

**Symptom:** mouse-down on the title bar does nothing.

**Cause:** another window is on top of yours — its hit zone is intercepting the click. The window you see may not be the one with the highest Z-index.

**Fix:** click anywhere on the body of your window first to focus it, then drag.

## App icon stays "raised" in the dock after closing

**Already fixed** in commit `378ae8d`. If you see this on a deployed build, hard-reload to fetch the latest assets.

## Cmd+V does nothing on the Desktop

**Symptom:** copying an image in macOS Preview, then Cmd+V on the Desktop, doesn't paste anything.

**Cause:** the host browser denied clipboard access, or the API isn't available.

**Fix:**
1. Look for a "Clipboard access denied" notification toast.
2. **Settings → Privacy → Reset clipboard permission**, then Cmd+V again — Tytus will re-prompt.
3. If your browser doesn't expose `navigator.clipboard.read()` (Firefox), only **text** is supported. Use Chrome / Edge / Safari for image paste.
4. Permission was probably granted but the cache is stale: the **Reset** button forces a fresh prompt on the next paste.

## Drag a file out to Finder doesn't download

**Symptom:** dragging a file from the Files window onto Finder leaves no file behind.

**Cause:** the host browser doesn't honour the `DownloadURL` drag MIME (Firefox, Safari prior to 17).

**Fix:** use Chromium / Chrome / Edge / Tytus's bundled WebView. Or right-click the row → Download.

## Window animations look glitchy / jumpy

**Symptom:** open / close / snap transitions stutter on a slow machine.

**Fix:** **Settings → Appearance → Reduce motion** turns off the CSS animations live. Tytus also auto-respects the OS-level "Reduce motion" preference, so toggling that in macOS System Settings has the same effect.

## Notification chime doesn't play

**Symptom:** a notification appears but no sound.

**Cause:** browser autoplay policy blocks audio until the user has interacted with the page.

**Fix:**
1. Click anywhere in the OS once after page load — that registers a user-gesture and unblocks audio.
2. Confirm **Settings → Notifications → System sounds** is on.
3. Confirm OS volume isn't muted.

## A window snapped and won't unsnap

**Symptom:** a window stuck in left-half / right-half mode.

**Fix:** drag its title bar more than ~24 px away from the snap target — Tytus restores the previous floating frame at the cursor. Or double-click the title bar to maximize then double-click again to fully restore.

## Settings don't persist

Settings (theme, accent color, etc.) save to `localStorage` under `tytus_settings`. They survive reload but reset if:
- Browser is in private mode
- Site storage is cleared
- localStorage quota is exceeded (very rare — Tytus uses < 1 MB)

## API Tester says CORS error

The browser blocks cross-origin requests by default. API Tester is a real `fetch()` and is subject to standard CORS rules.

**Fix:**
- Test against APIs that send `Access-Control-Allow-Origin: *`
- Or run a local proxy
- Or enable CORS in your dev server

## Voice / Screen Recorder asks for permission

The browser requires explicit microphone / screen permission. Click **Allow** when prompted. If you said deny, reset the site permission in browser settings.

## Games don't respond to keys

Click into the game window first to focus it. Keyboard events go to the focused window. Some games (Tetris, Snake, Pong) require window focus to receive arrow keys.

## Boot logo says something other than "Tytus OS"

You're on a stale build. HMR usually picks up branding edits, but a hard reload (**⌘+Shift+R** / **Ctrl+Shift+F5**) forces a re-fetch.

## I see a `Maximum update depth exceeded` console error

**Already fixed** in commit `3e81059`. If you still see this, you're on an old build — pull latest + rebuild.

## How do I reset everything?

In DevTools console:

```js
localStorage.clear();
location.reload();
```

This wipes desktop icons, notes, todos, calendar, settings, recordings, all of it. The boot animation will run again.

## Where does Tytus OS store data?

Everything is in your browser's `localStorage`, keyed under `tytus_*`:

| Key | What |
|---|---|
| `tytus_desktop_icons` | Icon positions on the desktop |
| `tytus_settings` | Theme, accent color, panel preferences |
| `tytus_notes` | Notes |
| `tytus_note_folders` | Notes folder structure |
| `tytus_todos` | Tasks |
| `tytus_todo_projects` | Custom Todo projects |
| `tytus_calendar_events` | Calendar events |
| `tytus_chat` | Chat history |
| `tytus_browser_bookmarks` | Browser bookmarks |
| `tytus_filesystem` | Virtual file tree |
| `tytus_archives` | Archive Manager state |
| `tytus_passwords` | (none — PasswordManager was dropped) |
| `tytus_contacts` | (none — Contacts was dropped) |
| ... | (more per app) |

Nothing leaves your machine — Tytus OS does not phone home. The only network requests are:
- Vite dev server (during development)
- Google Fonts CDN (Inter + JetBrains Mono webfonts)
- Whatever you explicitly send via API Tester or Browser

## Reporting bugs

Open an issue at <https://github.com/traylinx/tytus-os/issues> with:
- Browser + version
- Steps to reproduce
- Console output (red errors)
- Screenshot if visual


<!-- ==== about.md ==== -->

# About Tytus OS

Tytus OS is the desktop interface for [**Tytus**](https://tytus.traylinx.com), a private AI pod product by Traylinx. It runs in a browser tab and looks like a real OS — boot, login, desktop, dock, draggable windows.

## What problem it solves

Operating an AI pod today means juggling a tray icon, a hidden web page (Tytus Tower), and a scattered set of CLI commands. Tytus OS unifies all of that into a single coherent surface — the way a Mac unifies "files I can see + apps I can launch + system settings I can change" into one desktop.

## What's in v1

A working web-OS shell with 50 apps:

- **8 Tytus surfaces** — Pod Inspector, Settings, Chat, Files, Channels, Help, Terminal, Browser. Some functional today, some still placeholders.
- **42 OS-feel utilities** — Notes, Todo, Calendar, Calculator, image / video / music players, code editor, API tester, drawing apps, games. All real and working.

The shell itself (window manager, dock, launcher, notifications) is **complete and verified** — every interaction has an automated smoke test.

## What's coming

The roadmap is a 6-phase strangler-port from the legacy "Tytus Tower" web UI bundled in `tytus-cli`:

1. **Foundation cleanup** ✅ shipped
2. **Typed daemon client + auth bridge** — what unblocks real pod data
3. **Settings + Pod Inspector** — first real pod-management surfaces
4. **Chat + streamed runs**
5. **Files + Channels**
6. **Deprecate Tower** — the old page goes away

Full plan: [roadmap.md](../development/roadmap.md). Original architecture decision: [INTEGRATION-DEEPDIVE.md](../../INTEGRATION-DEEPDIVE.md).

## Where it lives

- **Code:** <https://github.com/traylinx/tytus-os> (private)
- **Daemon it talks to:** [tytus-cli](https://github.com/traylinx/tytus-cli)
- **Parent platform:** [Traylinx](https://traylinx.com)
- **Documentation:** this folder

This entire user manual is also bundled into the `tytus` CLI itself — run `tytus os-docs` to print it as a single markdown blob, or `tytus link` to drop it as `.tytus/os-manual.md` in any project so AI CLIs (Claude Code, OpenCode, Gemini, Codex, Cursor, Vibe, Aider) can answer Tytus-OS questions natively. The MCP tool `tytus_os_docs` exposes the same content over MCP.

## Naming

- **Tytus OS** (one word) — this product
- **Tytus** — the AI-pod product family
- **Traylinx** — the company / brand
- **Wannolot** — the internal codename for the pod-orchestration tier (you'll see it in commits)
- **Makakoo** — the parent organization

## Built with

- Vite 7 + React 19 + TypeScript 5.9
- Tailwind CSS 3 + shadcn-style primitives (Radix UI)
- lucide-react icons
- Playwright (smoke + sweep tests)

## License

Private. Not yet open-sourced. (When it is, expected: Apache 2.0.)

## Contact

For now: open issues in the GitHub repo. A user-facing support flow ships with the Help app in Phase 4.


---

<!-- ==== troubleshooting/clipboard.md ==== -->

# Host Clipboard — Per-Browser Behaviour

Tytus OS's **Cmd+V on the Desktop** (Sprint B Phase 5.4) reads from the
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


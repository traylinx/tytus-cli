# Changelog

All notable changes to the `tytus` CLI, `tytus-mcp` server, and
`Tytus.app` menu bar tray. Follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
conventions; versioning is [SemVer](https://semver.org/) — pre-1.0 minor
bumps are allowed to break compat.

## [Unreleased]

## [0.6.0-rc.13] — 2026-04-26

Closes the two remaining Phase B soft gaps from the PHASE-B-RESULTS
"What did NOT ship" list — Channels-tab + Files-tab in-tab pickers
no longer redirect users to the per-pod subpage.

### Phase B — Channels tab in-tab picker

Was: each agent rendered a "Manage channels" link to
`#/pod/NN/channels`. Required a hop into the per-pod subpage just to
add a Telegram bot token.

Now: `__renderChannelsTab` in `tower.js` calls `renderPodChannels`
(the existing per-pod picker — configured chips + add-channel
dropdown + remove flows) directly into the Channels tab, one card
per agent. Same `/api/channels` endpoints, same token-modal flow,
same redeploy-and-poll behavior. Zero new backend code.

The per-pod subpage at `#/pod/NN/channels` keeps working for
deep-links from the tray menu — both surfaces share the same
`renderPodChannels` function so any future channel feature lights
up everywhere without duplicate code.

### Phase B — Files tab in-tab picker

Was: Files tab showed only the Shared Folders details (good) but
nothing per-pod. The "Files tab in-tab picker" was a SPRINT.md soft
gap.

Now: per-pod cards under a new "Per-pod files" section, each with:
- **Browse inbox** — spawns `tytus ls --pod NN` via the existing
  pod run-streamed pipe. Output streams in-place to a `<pre>` log
  region. New whitelist entry `"ls-inbox"` added to `pod_action_argv`.
- **Open downloads folder** — POSTs to new
  `/api/files/open-downloads?pod=NN` endpoint, which `mkdir -p`s
  `~/Downloads/tytus/pod-NN/` then `open`s it in Finder. Mirrors
  the tray menu entry under per-pod Files ▸.

Push (file/folder upload from Mac) stays in the tray menu because
the browser sandbox can't surface a real OS file path. The Files
tab intro tells users where to find Push (`🅣 → Pod → 📁 Files →
Push file…`).

### Backend (Rust)

- New `pod_action_argv` arm: `"ls-inbox"` → `["ls", "--pod", NN]`.
- New endpoint `POST /api/files/open-downloads?pod=NN` →
  `handle_files_open_downloads` (parallels existing
  `/api/shared-folders/open-cache`).
- Whitelist test extended to lockstep-cover `ls-inbox`.

### Not shipped: chooser-as-modal

The original SPRINT.md soft-gap list also flagged "chooser-as-modal"
(wrap the install chooser in a `<dialog>` so it can be invoked from
any tab without a Settings hop). Considered for rc.13 but rejected:

- Existing Settings-tab hop is one click; the chooser is the
  Settings tab's primary content when no install is in progress.
- A real modal would require relocating four sibling sections
  (`#chooser` / `#installing` / `#success` / `#failure`) out of
  their `data-tab="settings"` parent into a `<dialog>` host, OR
  adding a CSS overlay that overrides every tab-visibility rule.
  Both add bug surface for cosmetic-only value.
- Phase B acceptance bar was already met without it.

Documented in PHASE-B-RESULTS.md as design considered, not shipped.

### Tests

- 33 tray unit tests green (whitelist test now covers `ls-inbox`).
- 90+ workspace tests green.
- Manual smoke deferred (requires live tray + browser walk).

## [0.6.0-rc.12] — 2026-04-26

Closes the last open Phase A soft gap: AUDIT.md called out the
`tytus --help` surface as *"30 top-level commands in flat
`--help` output (44 lines, no grouping)"*. rc.12 prepends a
curated grouping above clap's auto-generated alphabetical list so
new users see intent-based buckets first.

### Phase A.5 — `tytus --help` grouping

clap derive doesn't natively support per-subcommand-variant help
headings, so a `before_help` block inside `Cli` injects a curated
TLDR. The output now reads:

```
Most-used:        setup, chat, help, status, test, doctor
Files:            push, pull, ls, rm, transfers
Pod & connection: connect, disconnect, env, capabilities, restart,
                  exec, ui, agent, revoke
Identity:         login, logout, link, mcp, channels, configure
Settings:         autostart, tray, daemon
```

Power users still get the full alphabetical `Commands:` list below
the grouping with every flag clap generated. Hidden commands
(`#[command(hide = true)]` set in rc.8 — `lope`, `bridge`,
`llm-docs`, `bootstrap-prompt`, `tunnel-up`, `tunnel-down`)
remain hidden in both surfaces.

### Tests

- 90+ workspace tests green (`cargo test --workspace`).
- Manual smoke: `tytus --help` shows curated grouping ahead of
  alphabetical list; `tytus help` (the topic command) unchanged.

## [0.6.0-rc.11] — 2026-04-26

Closes two soft Phase C gaps the rc.5 acceptance bar passed but the
SPRINT.md target diagram still showed unfinished:
SPRINT.md C.2 (per-pod submenu reorder around primary verbs) and
SPRINT.md C.3 (daemon controls under a Developer-options toggle).
No actions removed; everything reachable in ≤3 clicks remains so.

### Phase C.2 — per-pod submenu reorder (Chat / Files / Channels first)

Per SPRINT.md §Phase C target diagram, the per-pod expand previously
opened to a 9-item wall (header, Open in Browser, Stop Forwarder,
Copy URL, Restart, Uninstall, Channels▸, Files▸, Revoke). The first
three primary verbs (Chat / Files / Channels) were buried behind
admin verbs.

- `tray::main::build_menu` per-pod block reordered:
  1. Pod header (informational, disabled)
  2. **`💬 Chat now…`** (renamed from "Open in Browser" — vocab
     verdict: same action, plain-language label)
  3. **`📁 Files ▸`** (was 7th; now 2nd primary verb)
  4. **`📨 Channels ▸`** (was 6th; now 3rd primary verb)
  5. **`⚙ Manage ▸`** (NEW — wraps heavy/admin ops)
- `Manage ▸` holds: Copy Public API URL (when public edge is
  populated), Stop Forwarder (when live), Restart Agent, Uninstall
  Agent, separator, Revoke Pod. Destructive Revoke sits under a
  separator so it isn't adjacent to non-destructive items.
- Submenu titles now carry the same emoji as the top-level Chat /
  Files / Channels deep-links so the visual association across
  surfaces is obvious.
- Behavior unchanged: every existing menu ID still fires the same
  handler. The reorder is presentation-only.

### Phase C.3 — Developer options ▸ inside Help

Per SPRINT.md §Phase C target diagram, daemon controls and raw log
tails were always visible at the Help tier, ahead of grandma-facing
items (Documentation, About). Phase C.3 wraps them so the Help
expand reads as: Help… / Documentation / About / Developer options ▸.

- New `Developer options ▸` submenu inside Help, holding:
  - Run diagnostics (advanced)
  - View Daemon Log
  - View Startup Log
  - Start / Stop / Restart Daemon (one of, depending on state)
- Help… stays at the top (single grandma-facing recovery action).
- Documentation + About stay one click deep at the Help tier so
  the Help submenu remains scannable for non-developers.
- All four developer actions remain one click deeper than before.
  Same handlers, same IDs.

### Tests

- 33 tray unit tests green (no behavior changes, unit tests cover
  ID parsing + error friendlify lockstep — both untouched).
- Manual smoke: cold-start menu walk shows Chat/Files/Channels at
  per-pod top; Help reads Help/Docs/About/Dev-options.

## [0.6.0-rc.10] — 2026-04-26

Closes the two strict gaps from earlier rcs that the user audit
flagged: (1) Phase E Tower-side `?` icons (was deferred in rc.5),
(2) Phase F `tray/src/error_ui.rs` Rust port (was JS-only since rc.5).

Both surfaces are now lockstep with each other and with `AUDIT.md` /
`SPRINT.md`.

### Phase E Tower-side — `?` icons + popover

Per SPRINT.md Phase E acceptance bar: *"every form field in Tower
has a `?` within 1 glance"*.

- New `<button class="help-icon" data-help="<key>">?</button>` next
  to non-obvious labels:
  - `share-folder` (next to "+ Share a folder…" button)
  - `bucket-name` (next to the Folder name form-label)
  - `autostart-tunnel` (next to the Settings autostart toggle)
  - `autostart-tray` (next to the launch-tray toggle)
- New `#hdr-help` button in the header bar (next to "Run health test"
  + connection chip): the global "What's all this?" entry.
- New `.help-popover` element (single global node, repositioned per
  open). Contains: title + 1-2 sentence body + "More in Help →" link
  to the relevant tab.
- Popover behavior: click outside dismisses. `Esc` dismisses.
  Hashchange dismisses. Click on the "More" link itself also
  dismisses (covers the same-hash case where hashchange wouldn't
  fire).
- Help-text table keyed by `data-help` attribute lives in `tower.js`
  as `__HELP_TEXT`. 5 entries shipped; trivially extensible by
  adding `data-help="<new-key>"` to any element + an entry in the
  JS table.
- Touch targets are 22×22 (Apple HIG accessibility floor for
  inline-button icons) with 6 px left margin from the labelled text.
  Hover transitions to accent-soft + accent border.

### Phase F Rust port — `tray/src/error_ui.rs`

Per SPRINT.md Phase F: *"`tray/src/error_ui.rs` (new) maps known
error patterns to `{title, body, suggested_action}` triples"*.

- New module `tray/src/error_ui.rs` — Rust mirror of `tower.js`'s
  `__FRIENDLIFY_PATTERNS`. Same 20 patterns, same titles + bodies +
  try-this hints. Lockstep guard test asserts the count matches the
  JS file so future drift surfaces as a test failure.
- Substring-based matching (no regex dep). Each pattern carries an
  OR-list of lowercased needles; we lowercase the input once and
  test contains() across needles. Same semantics as the JS regex
  patterns for every canonical raw error string.
- `pub fn friendlify(raw: &str) -> Option<FriendlyError>` returns
  `None` for unknown errors (caller falls back to the existing
  generic dialog). `pub fn format_for_dialog(err)` formats for
  osascript dialog body. `FriendlyError::one_line()` for tooltip /
  notification surfaces. `raw_disclosure_hint()` keeps the original
  reachable for power users.
- 16 unit tests covering: every primary pattern, unknown returns
  None, empty returns None, raw preservation, one_line ≤ 120 chars,
  case-insensitivity, first-match-wins, format_for_dialog includes
  all fields, UTF-8 char-boundary truncation, unicode inputs don't
  panic. Module-level `#![allow(dead_code)]` because some helpers
  (`format_for_dialog`, `one_line`) are part of the public API for
  future hooks but only `friendlify` has a caller today.
- **First user-visible hook**: `build_diag_summary` (the
  "Copy Diag" flow from the Help menu's "Help… (something's not
  working)" dialog) now appends a friendly-hint section below the
  `connect.log` tail when the log matches a known pattern. Power
  users see the raw log; grandma sees `<title>: <body>. Try this:
  <action>` underneath.

### Workspace test count

12 + 2 + 3 + 31 + 33 + 9 = **90 passing tests** across all packages
(was 74 in rc.9). +16 from the new `error_ui` module.

### Out of scope for this rc

- **SPRINT.md C.2 per-pod submenu reorder** (Chat/Files/Channels
  first, push management actions under "Manage…"). Acceptance bar
  was already met in rc.2 (≤8 top-level items, ≤3 clicks for every
  audit task). The reorder is a layout polish that requires
  invasive refactoring of the per-pod submenu builder; deferred.
- **SPRINT.md C.3 Daemon controls under Developer-options toggle**.
  Daemon controls live under Help (rc.4) which is more discoverable
  than burying behind a Developer-options toggle. Acceptance bar
  not impacted.
- **SPRINT.md B chooser-as-modal**. Chooser stays inline in
  Settings tab (rc.4). Acceptance bar (5 tabs reachable in 1 click)
  was met without it.
- **SPRINT.md B Channels-tab picker promotion + Files-tab Push/Pull
  picker**. Both stubs link to the existing per-pod paths; full
  in-tab pickers are queued for v0.6.x.

### Backwards compat

- **Zero Rust API changes** in core / pods / cli / mcp. New module
  is in `tray/src/` only.
- **Tower additions** are scoped (new selectors, new event handlers,
  new module-level help text table). All existing element IDs +
  handlers continue to work.
- 90/90 workspace tests green. .pkg rebuilds clean for rc.10.

### Files touched

- `tray/src/error_ui.rs` (new) — Rust friendlify port + 16 tests
- `tray/src/main.rs` — `mod error_ui;` declaration + hook into
  `build_diag_summary`
- `tray/web/tower.html` — 5 new help-icon buttons + `#hdr-help`
- `tray/web/assets/tower.css` — `.help-icon` + `.help-popover`
  styles + button-bar variant (~120 LOC)
- `tray/web/assets/tower.js` — `__HELP_TEXT` table + popover
  show/hide/position logic + delegated click handler + Esc handler
  + same-hash dismiss safeguard (~95 LOC)
- `Cargo.toml` — workspace version bump to `0.6.0-rc.10`
- `CHANGELOG.md` — this entry

## [0.6.0-rc.9] — 2026-04-26

**Critical bugfix in tower.js.** Discovered via headless-browser
visual verification (gstack `/browse`): a Temporal Dead Zone
`ReferenceError` had been silently breaking the Phase F friendlify
layer since rc.5, and indirectly preventing the IIFE from completing
its setup. Caught only because `window.__friendlifyError` returned
`undefined` even though the source code declared and exposed it.

### What was broken

`__renderChatTab()` at line ~579 references `budgetState`, but the
`let budgetState = null;` declaration lives at line 1042 — far below.
ES2015 strict-mode `let` doesn't hoist into the Temporal Dead Zone,
so reading `budgetState` before its declaration throws
`ReferenceError`. The throw happened inside the initial
`__applyTabFromHash()` call (line ~376) at script load — which
silently aborted the rest of the IIFE.

Symptoms (subtle — no console error in headless mode):
- `body.dataset.tab` got set to "chat" (the assignment ran first).
- `window.__wizardOpen` got registered (line 497, before the throw).
- `window.__friendlifyError` was **never** registered (line 2648, after).
- Phase F's `showToast(msg, 'err')` wrap was a no-op — error toasts
  showed raw subprocess text instead of friendly hints.
- Phase B chat-tab pod-list rendered correctly because its
  `(budgetState && ...)` check happened to early-return when
  `budgetState` was undefined (TDZ aside, the JS engine evaluated
  the && short-circuit cleanly when `budgetState` was actually
  declared by the time the second pass ran). Real-world impact was
  limited to friendlify being silent — which dogfood walks didn't
  catch because nothing visibly failed.

### Fix

Moved `let budgetState = null;` to the top of the IIFE (right after
`const $ = (id) => ...`). All references in Phase B / G / F code now
hit a proper `null` instead of a TDZ throw. The original
`loadBudget()` initializer keeps populating it later in the lifecycle.

### Verified end-to-end via /browse

Headless visual verification confirmed:
- ✓ All 5 tabs (Chat / Files / Channels / Settings / Help) render
- ✓ Hash routing flips `body.dataset.tab` and CSS visibility on click
- ✓ Phase D Tower-side renames live ("Tytus" not "Tytus Tower",
  "Background service" not "Daemon", "Setup wizard / Run setup
  wizard again" in Help, etc.)
- ✓ Chat tab auto-renders pod list when ≥1 agents installed
  (Sebastian has Pod 02 + Pod 04 with OpenClaw)
- ✓ "Talk to this AI" button loads pod's OpenClaw UI URL via
  iframe — `https://<slug>-p02.tytus.traylinx.com/?token=<gateway-token>`
- ✓ Phase G first-run wizard opens via `__wizardOpen()`, renders
  Step 1 of 4 with Welcome copy + Get started button + Skip link
- ✓ Auto-skip on logged-in + has-agents user advances Step 2 → 3 → 4
  immediately (verified: clicking Get started landed at Step 4
  because Sebastian was already fully set up)
- ✓ Step 4's "Open Chat" closes the wizard, sets
  `localStorage.tytus.wizard.completed = "true"`, navigates to #chat
- ✓ Phase F `friendlifyError("connection refused: 127.0.0.1:18080")`
  now returns the correct `{title: "Tytus isn't running", body:
  "...", try: "Click the menu-bar T → Quick actions → Connect."}`
- ✓ 11 screenshots at /tmp/tytus-screenshots/ documenting each surface

### Why this slipped through

The bug was masked by:
- No console error in headless mode (uncaught script-load
  ReferenceErrors don't always surface to `console`).
- The IIFE failed AFTER setting `body.dataset.tab` and
  `window.__wizardOpen`, so the page rendered with the right tab
  highlighted and the wizard reachable via direct invocation.
- Manual dogfood walks tested visible UI only — not the underlying
  Phase F friendlification path which never had a triggering error
  during testing.
- `node --check` parses fine because TDZ is a runtime, not parse,
  error.

Caught only by an in-browser eval probe that asked
`typeof window.__friendlifyError`.

**Lesson**: visual verification via `/browse` exposes real bugs that
unit tests + manual click-throughs miss. Worth adding to every
release pipeline going forward.

### Files touched

- `tray/web/assets/tower.js` — moved `let budgetState = null;`
  declaration to the top of the IIFE; left a one-line stub at the
  old position.
- `Cargo.toml` — workspace version bump to `0.6.0-rc.9`
- `CHANGELOG.md` — this entry

## [0.6.0-rc.8] — 2026-04-26

`tytus --help` polish — hides four deeply-internal commands that
target AI agents driving Tytus, not human users. The commands still
work; they just don't clutter the top-level help dump.

### Hidden from `tytus --help`

- `lope` — drives a pod-hosted agent as a lope teammate. Audience:
  multi-CLI validator ensembles, not humans. Reachable via
  `tytus lope --help` and `tytus lope ask` etc.
- `bridge` — HarveyBridge reverse channel. Audience: pod-internal
  agents writing to Harvey's brain. Reachable via `tytus bridge run`.
- `llm-docs` — LLM-facing reference. Audience: AI agents that need
  the full driving manual. Reachable via `tytus llm-docs`.
- `bootstrap-prompt` — short setup prompt for pasting into other AI
  CLIs. Audience: AI-tool integrators. Reachable via
  `tytus bootstrap-prompt`.

`tytus --help` now lists 26 user-facing commands (was 30). Backwards
compat: every hidden command is still callable; this is a
presentation-only change via clap's `#[command(hide = true)]`.

### What did NOT change

- All four hidden commands still work, still produce JSON output,
  still get pasted into AI CLI bootstraps. They just don't appear in
  the top-level dump.
- Per-command `tytus <cmd> --help` works for hidden commands too.
- Plain-English `tytus help <topic>` (rc.5) lists the same 12
  user-verb topics; lope / bridge / llm-docs aren't there because
  they aren't user verbs.

### Files touched

- `cli/src/main.rs` — 4 `#[command(hide = true)]` attributes on the
  hidden Commands variants
- `Cargo.toml` — workspace version bump to `0.6.0-rc.8`
- `CHANGELOG.md` — this entry

## [0.6.0-rc.7] — 2026-04-26

Phase H scaffolding — the `.pkg` installer pipeline ships unsigned
in this rc; signing + notarization is a 3-command run for any
maintainer holding the Developer ID Installer cert. Once signed and
uploaded, the only remaining bump is `Cargo.toml` from rc.7 → 0.6.0
+ a CHANGELOG entry promoting the rc to final.

### Phase H — `.pkg` builder + signing scaffolding

New `pkg/` directory:

- **`pkg/build-pkg.sh`** — builds an unsigned
  `target/Tytus-<version>-unsigned.pkg` from `target/release/`
  binaries using `pkgbuild` + `productbuild` (Xcode CLT, included on
  every Mac). Produces a 10 MB universal-binary `.pkg` containing:
  - `/usr/local/bin/tytus`
  - `/usr/local/bin/tytus-tray`
  - `/usr/local/bin/tytus-mcp`
  - Postinstall script (see below)
  - Welcome + conclusion HTML screens for Installer.app
  - Distribution xml with `hostArchitectures="x86_64,arm64"`
  - Strips extended attributes via `xattr -c` so the payload doesn't
    ship AppleDouble resource-fork shadow files.
- **`pkg/scripts/postinstall`** — runs as root on every install:
  1. Identifies the GUI-logged-in user via
     `stat -f "%Su" /dev/console`.
  2. Calls `sudo -u <user> -H tytus tray install` so the `.app`
     bundle and `~/Library/LaunchAgents/com.traylinx.tytus.tray.plist`
     land in the user's home (not `/var/root/`).
  3. Drops `/etc/sudoers.d/tytus` with the same tightly-scoped
     `wg-quick` / `route` / `ifconfig` exemptions `install.sh`
     creates, so the user never types their password to bring the
     tunnel up.
  4. Logs every step to `/tmp/tytus-postinstall.log`.
- **`pkg/SIGNING.md`** — full reference for the maintainer:
  - Prereqs (Xcode CLT, Developer ID Installer cert, app-specific
    password, team ID).
  - One-time `xcrun notarytool store-credentials` setup so future
    notarizations are a single flag.
  - The full pipeline:
    `cargo build --release` → `./pkg/build-pkg.sh` → `productsign`
    → `xcrun notarytool submit --wait` → `xcrun stapler staple`
    → `spctl -a -t install -vv` → `gh release upload`
    → mirror to `tytus.traylinx.com/Tytus.pkg`.
  - Troubleshooting table (signing identity missing, notarization
    rejected, Gatekeeper still blocks, etc.).
  - Optional Mach-O binary code-signing recipe for futureproofing
    against tighter notarization rules.

### What ships unsigned in this rc

`target/Tytus-0.6.0-rc.7-unsigned.pkg` builds locally on demand
(`./pkg/build-pkg.sh`). It's gitignored (`target/` is already in
`.gitignore`) so the artifact never lands in the repo. Maintainers
build, sign, and upload as part of the release process.

The README install command (`curl -fsSL https://get.traylinx.com/install.sh
| bash`) remains the canonical path until the signed `.pkg` is
hosted at `https://tytus.traylinx.com/Tytus.pkg`. Once that lives,
README updates to a "Download for Mac" button.

### Verified locally

```bash
$ ./pkg/build-pkg.sh
✓ Built  target/Tytus-0.6.0-rc.7-unsigned.pkg  (10M, unsigned)

$ pkgutil --expand-full target/Tytus-0.6.0-rc.7-unsigned.pkg /tmp/x
$ ls /tmp/x/Tytus-component.pkg/Payload/usr/local/bin
tytus  tytus-mcp  tytus-tray

$ ls /tmp/x/Tytus-component.pkg/Scripts
postinstall
```

### What still needs Sebastian's hands

`./pkg/build-pkg.sh` produces the unsigned artifact. Three commands
turn it into a notarized, Gatekeeper-accepted installer:

```bash
productsign --sign "Developer ID Installer: <name> (<team>)" \
    target/Tytus-0.6.0-rc.7-unsigned.pkg target/Tytus-0.6.0.pkg

xcrun notarytool submit target/Tytus-0.6.0.pkg \
    --keychain-profile "tytus-notary" --wait

xcrun stapler staple target/Tytus-0.6.0.pkg
```

Total wall-clock: ~5–10 minutes (notarization round-trip dominates).
After that: `gh release upload v0.6.0 target/Tytus-0.6.0.pkg`,
update README "Install" lead to a `[ Download for Mac ]` button,
and re-tag rc.7 contents as `v0.6.0`.

### Backwards compat

- Existing `cargo install --git` source build path unchanged.
- Existing `https://get.traylinx.com/install.sh` one-liner unchanged.
- New `pkg/` directory is build-time only; binaries don't import it.
- Internal Rust unchanged from rc.6.

### Files added

- `pkg/build-pkg.sh`
- `pkg/scripts/postinstall`
- `pkg/SIGNING.md`
- `Cargo.toml` — workspace version bump to `0.6.0-rc.7`
- `CHANGELOG.md` — this entry

## [0.6.0-rc.6] — 2026-04-26

Phase G — first-run wizard inside Tower. Closes the last
implementation phase before v0.6.0 final (rc.6 + Phase H signed `.pkg`
installer = v0.6.0). Phase H needs Sebastian's Apple Developer cert
and interactive notarization, so it ships separately.

### Phase G — First-run wizard

A 4-step overlay shown on the very first Tower load. Auto-skips when
the user is already fully set up (logged_in + ≥1 agents installed).

```
Step 1 of 4 — Welcome to Tytus
Step 2 of 4 — Sign in
Step 3 of 4 — Pick your AI
Step 4 of 4 — Send your first message
```

- **Step 1 — Welcome.** Bullet list ("Talk to your AI from any
  terminal", "Share Mac folders", "Chat from Telegram/Discord/Slack").
  "Get started" advances. "Skip" closes for this session.
- **Step 2 — Sign in.** "Sign in with browser" → POSTs `/api/connect`
  (which kicks `tytus connect` → opens browser auth). Wizard polls
  `/api/state` every 2s; auto-advances when `logged_in` flips to true.
- **Step 3 — Pick your AI.** Hides the overlay so the user can
  interact with the chooser catalog in the Settings tab. Polling
  continues; auto-re-shows the wizard at step 4 when
  `state.agents.length > 0`.
- **Step 4 — Send your first message.** "Open Chat" closes the
  overlay (sets `localStorage.tytus.wizard.completed = 'true'`) and
  sets `location.hash = '#chat'`. Skip is hidden on this step
  because the user is already done.
- **Auto-skip on returning users.** When `state.logged_in` is true
  AND `state.agents.length > 0`, the wizard quietly marks itself
  completed and never shows. New installs go through it; existing
  users never see it.
- **Skip-to-later** closes the overlay for this session without
  marking completed; re-shows next Tower open.
- **Run again.** Help tab → "Run setup wizard again" button clears
  `localStorage` and re-opens. Useful if the user skipped on first
  launch and wants to come back.

### Acceptance bar (from SPRINT.md)

> A user with zero prior context completes all 4 steps in ≤3 minutes
> (timed observation).

Met by structure. Wall-clock observation requires a real grandma sit-
in (per SPRINT "Out of scope") — Sebastian to recruit; the calling-
card video can be filmed against rc.6.

### Backwards compat

- New section `#wizard` and ~140 LOC of JS scoped to a self-
  contained module. Zero edits to existing handlers / sections.
- New CSS scoped to `.wizard-*` selectors.
- localStorage key `tytus.wizard.completed` is namespaced and never
  written under any other path.
- Tower's `/api/state` poll is identical to existing flows (already
  polled by `refreshHeaderConn` every 10s; wizard polls every 2-3s
  during the active step only and stops when the step advances).
- All 74 workspace tests green. Internal Rust unchanged from rc.5.

### Files touched

- `tray/web/tower.html` — wizard `<section>` overlay + Help-tab
  "Run setup wizard again" button row
- `tray/web/assets/tower.css` — wizard overlay/card/dot/spinner
  styles (~120 LOC added)
- `tray/web/assets/tower.js` — wizard state machine + 4 step
  definitions + state-ready hook + DOMContentLoaded wire
  (~165 LOC added)
- `Cargo.toml` — workspace version bump to `0.6.0-rc.6`
- `CHANGELOG.md` — this entry

### Phase H — what's left for v0.6.0 final

- Signed `.pkg` installer via `productbuild` + Apple Developer ID
  signing + notarization round-trip
- Post-install script to run `tytus tray install` automatically
- Hosted at `https://tytus.traylinx.com/Tytus.pkg` + GitHub release
  artifact mirror
- README install section becomes one button: `[ Download for Mac ]`

Needs Sebastian's hands on the Apple Developer cert.

## [0.6.0-rc.5] — 2026-04-26

Phase F (error UX friendlify layer) + Phase E (in-context help — CLI
side). Tower toasts now turn raw subprocess errors into "<title>:
<body>. Try this → <action>" hints. New `tytus help <topic>` command
gives plain-English explanations of every primary user verb without
the clap reference dump.

### Phase F — Error UX friendlify layer

- New `friendlifyError(rawMsg)` helper in `tray/web/assets/tower.js`.
  20 canned patterns mapped to `{title, body, try}` triples covering
  the failure modes from `AUDIT.md`:
  - Keychain dialog pending
  - No workspace yet
  - Sign-in expired (invalid API key / token expired)
  - Connection blocked (other VPN)
  - Tytus already connecting (daemon already running)
  - Workspace not ready
  - Form data went wrong (bad JSON)
  - Tytus not installed (command not found)
  - AI hit an error (HTTP 5xx)
  - macOS permission denied
  - Tytus isn't running (connection refused)
  - Port already taken
  - Computer offline
  - Folder name taken / invalid
  - Disk full
  - WireGuard not installed
  - Workspace would orphan unsaved data
  - Not signed in
- `showToast(msg, 'err')` runs every error toast through
  `friendlifyError`. When a pattern matches, the toast shows
  `"<title>: <body>. Try this → <action>"` for 6.5s (longer than
  the default 2.8s — friendly errors deserve a read).
- `streamGlobalAction`'s SSE `fail` event handler appends the same
  Try-this hint at the bottom of the streamed log when a pattern
  matches the failure data. The full raw error stays above for
  power users.
- Patterns tested in order; first match wins. Unmatched lines fall
  through to the raw text — no false-positive friendlification.
- `window.__friendlifyError` exposed for devtools-side debugging.

### Phase E — `tytus help <topic>` plain-English help

- New CLI subcommand `tytus help` (replaces clap's auto-`help`
  subcommand for grandma-side language). 12 topics:
  - chat — Talk to your AI
  - setup — First-time setup
  - connect — Connect to your AI
  - share — Share a folder with your AI
  - channels — Talk to your AI from Telegram, Discord, or Slack
  - install — Install Tytus on a new Mac
  - sign-in — Sign in to Tytus
  - uninstall — Remove Tytus from this Mac
  - troubleshoot — Something's not working
  - disconnect — Stop the connection
  - doctor — Run diagnostics
  - env — Connect other apps to your AI
- `tytus help` → lists topics with one-line summaries.
- `tytus help <topic>` → 5-line plain-English explanation + a hint
  pointing at `tytus <command> --help` for the technical reference.
- `tytus help bogus` → graceful "No help topic 'bogus'" + topic-list
  hint (no panic, no clap error).
- `--json` mode emits the full topic table as JSON for tooling.
- `disable_help_subcommand = true` on the top-level Cli struct so
  clap's auto-help doesn't shadow the new command. `tytus <cmd>
  --help` still works (clap auto-flag, not subcommand).

### Phase E — Tower side (defer)

The `?` icon + popover system on Tower tabs / form fields is
intentionally deferred to a later rc — Phase E in rc.5 ships the
CLI side which is the higher-signal piece (a non-tech user trying
`tytus help` in Terminal is plausible; clicking `?` icons inside
Tower already lives behind the existing autocomplete-y `settings-
hint` text under most fields).

### Backwards compat

- `tytus <cmd> --help` continues to print the clap reference for
  every subcommand (auto-flag is unchanged).
- `tytus help <command-name>` historically returned the clap help
  for that command (clap's auto-subcommand). v0.6 makes `tytus help
  <topic>` the friendly help instead. Topics overlap with clap
  subcommand names where it makes sense (chat, setup, connect,
  doctor, env, disconnect) and add grandma verbs where clap doesn't
  have one (share, install, sign-in, uninstall, troubleshoot,
  channels). Power users hit `tytus <cmd> --help` for the technical
  flag reference.
- Tower JS additions are scoped — only `showToast` and
  `streamGlobalAction`'s `fail` handler call `friendlifyError`. All
  existing call sites continue to work.
- 74/74 workspace tests green.

### Files touched

- `cli/src/main.rs` — `HelpTopic` variant + `cmd_help_topic` + Cli
  attribute `disable_help_subcommand = true`
- `tray/web/assets/tower.js` — `__FRIENDLIFY_PATTERNS` table,
  `friendlifyError` helper, `showToast` wrap, streamGlobalAction
  `fail` handler hook
- `Cargo.toml` — workspace version bump to `0.6.0-rc.5`
- `CHANGELOG.md` — this entry

## [0.6.0-rc.4] — 2026-04-26

Phase B + Phase D Tower-side. Tower converts from a single scrollable
page with `<details>` collapsibles into a 5-tab SPA with hash-routed
navigation. The tray's rc.2 deep-links (`open_tower_chat` → `#chat`
etc.) now land on the right tab.

### Phase B — 5-tab top-nav

- **5 tabs at the top of Tower**: 💬 Chat / 📁 Files / 📨 Channels /
  ⚙️ Settings / ❓ Help. Each `<a class="tab">` is an `href="#chat"`
  etc.; hashchange triggers the router which sets
  `body[data-tab="<id>"]`. CSS rules show only the active tab's
  `.tab-pane` elements and hide the rest.
- **Default landing is `#chat`** on every cold open. Empty hash + no
  prior tab ⇒ Chat. The router defaults gracefully when the hash is
  unknown (e.g. `#/run/doctor` deep-link) — pod-mode and run-action
  hashes still take priority because the tab router yields when
  `body.pod-mode` is set.
- **Chat tab content (rc.4 minimum)** —
  - **No agents installed**: empty state with a `Settings →` CTA.
  - **≥1 agents**: pod list with "Talk to this AI" buttons that load
    the pod's OpenClaw UI URL (`a.ui_url || a.public_url`) into an
    embedded iframe. Same URL the tray's "Open in Browser" uses, no
    new backend. Per SPRINT.md Phase B anti-goal: iframe embed is
    the SPRINT-blessed path; SSE token streaming is a future rc.
- **Files tab content**: existing Shared Folders details section,
  re-parented into the Files tab. Push/pull (per-pod transfers) defer
  to a later rc.
- **Channels tab content**: stub linking each agent into its existing
  per-pod Channels subpage (`#/pod/NN/channels`). Lift-the-picker-up
  defer.
- **Settings tab content**: existing Settings (autostart toggles,
  Configure your AI, Sign Out) + chooser/installing/success/failure
  install flow (all `data-tab="settings"`). The chooser remains
  inline in this rc — the chooser-as-modal pattern from SPRINT.md is
  defer.
- **Help tab content**: existing Troubleshoot details — Doctor,
  Background service controls, log viewer.
- **Tab nav hides during pod-mode** — `body.pod-mode` is set when
  the user is on a `#/pod/NN/...` deep-link. The pod subpage has its
  own per-pod tabs; the global tab nav disappears via CSS.

### Phase D — Tower-side renames (per locked verdict)

Now that Tower's primary surfaces are stable, applied the locked
HYBRID/C verdict from `verdicts/Q1-VOCAB-LOCKED.md`:

- **Header brand** "Tytus Tower" → "Tytus" (with subtitle change).
- **Subtitle** "Your private AI pods, agents, and connections." →
  "Chat, share files, and manage your AI assistant."
- **Header button** "Run Health Test" → "Run health test"
  (sentence case).
- **Settings tab**:
  - "Configure Agent…" → "Configure your AI…"
  - autostart-tunnel hint "Bring the WireGuard tunnel up
    automatically…" → "Connect to your AI automatically…"
- **Files tab (Shared Folders form)**:
  - "+ Bind a folder…" button → "+ Share a folder…"
  - "Bind a Mac folder" label → "Share a Mac folder"
  - "Bucket name on the cloud" → "Folder name on the cloud"
  - "Bind folder" submit → "Share folder"
  - "Open ~/.cache/garagetytus" button → "Open sync folder"
- **Per-pod Output toolbar**:
  - "Stop forwarder" → "Stop browser shortcut"
  - "Refresh creds" → "Refresh sign-in"
- **Help tab (Troubleshoot)**:
  - "Daemon" label → "Background service"
  - hint "Background token-refresh process. Stop/restart if auth
    is stuck." → "Refreshes your sign-in in the background.
    Stop/restart if you can't connect."
- **Footer About panel**:
  - "Daemon" row label → "Background service"
- **Success-screen**:
  - "Install another agent" button → "Install another AI"

### Backwards compat

- **Every existing element ID is preserved.** `#chooser`, `#installing`,
  `#success`, `#failure`, `#settings`, `#shared-folders`,
  `#troubleshoot`, `#view-pod`, all per-pod IDs. Only `class` and
  `data-tab` attributes added; element identities unchanged.
- **Existing JS handlers** (loadCatalog, view.show, viewPod, etc.) all
  continue to work — they bind to IDs, not classes.
- **CSS additions** are scoped to new selectors (`.tabs`, `.tab`,
  `.tab-pane`, `.chat-*`, `.tab-intro`, `body[data-tab=...]`); zero
  edits to existing rules.
- **Internal Rust** unchanged from rc.3 (web_server.rs untouched —
  Tower assets are baked via `include_bytes!`, just the bytes
  changed).
- 74/74 workspace tests green.

### Files touched

- `tray/web/tower.html` — 5-tab nav + tab-pane wrappers + Chat tab
  + Channels tab stub + Phase D Tower-side renames
- `tray/web/assets/tower.css` — tab nav styles + body[data-tab=]
  visibility rules + Chat/Channels tab pane styles (~150 LOC added)
- `tray/web/assets/tower.js` — tab router IIFE + Chat/Channels
  render functions (~120 LOC added)
- `Cargo.toml` — workspace version bump to `0.6.0-rc.4`
- `CHANGELOG.md` — this entry

## [0.6.0-rc.3] — 2026-04-26

Phase D — vocabulary normalization, partial. Applies the locked
HYBRID/C verdict from `verdicts/Q1-VOCAB-LOCKED.md` to the highest-
traffic CLI + tray surfaces. Tower strings deferred to rc.4 because
Phase B (Tower 5-tab rewrite) introduces fresh strings that get the
rename at write-time.

### Phase D — CLI surfaces (renamed)

- **Welcome banner subtitle** in `cli/src/wizard.rs`:
  `Your private AI pod, tunneled to any terminal`
  → `Your own private AI — talk to it from any terminal`.
- **Welcome info line** in `cli/src/main.rs::cmd_default`:
  `Tytus gives you a private, encrypted AI pod — your own
  OpenAI-compatible gateway.`
  → `Tytus runs your own AI in private. Your messages never leave
  the encrypted line between your computer and your AI.`
- **Dashboard** (`show_dashboard`):
  - `No pods allocated yet` → `No workspaces yet`
  - `Start your pod: tytus connect` → `Set up your workspace: tytus connect`
  - `Your Pods` header → `Your workspaces`
  - `AI Gateway:` → `AI URL:`
  - `Agent API:` → `AI API:`
  - `Tunnel:` → `Connection:`
  - `tytus disconnect — Stop the tunnel` → `tytus disconnect — Disconnect`
  - `tytus connect — Start your tunnel` → `tytus connect — Connect to your AI`
  - `tytus doctor — Diagnose issues` → `tytus doctor — Run diagnostics`
  - `tytus configure — Configure your agent` → `tytus configure — Configure your AI`
- **Recoverable error messages** (user-facing, primary path):
  - `No pods. Run: tytus connect (for AIL) or tytus agent install <name>`
    → `No workspace yet. Run: tytus connect (for direct AI) or
    tytus agent install <name> (to install an AI assistant)`
  - `No pods. Run: tytus connect` → `No workspace yet. Run: tytus connect`
  - `No pod allocated. Run: tytus setup` → `No workspace yet. Run: tytus setup`
  - `No pod allocated` (interactive surfaces) → `No workspace yet`

### Phase D — Tray surfaces (renamed)

- **Status line** in `build_menu`:
  - `No pods allocated — click Connect` → `No workspace yet — click Connect`
  - `Tunnel inactive — click Connect` → `Not connected — click Connect`
- **Tooltip** (`tooltip_for`):
  - `Tytus — Daemon not running` → `Tytus — Background service not running`
- **Quick actions ▸** submenu:
  - `Connect (tunnel)` → `Connect`
- **Settings ▸** submenu:
  - `Configure Agent…` → `Configure your AI…`
- **Help ▸** submenu:
  - `Doctor (advanced)` → `Run diagnostics (advanced)`
- **Pods submenu** (Quick actions ▸ Show all pods ▸):
  - `No pods allocated` → `No workspace yet`

### Phase D — Stayed technical (per locked verdict)

- **CLI subcommand names** — `tytus connect`, `tytus agent install`,
  `tytus daemon`, `tytus exec`, `tytus pods` etc. — all byte-identical
  to v0.5.5. Backwards-compat per locked sprint rule.
- **CLI `--help` clap output** — clap descriptions stay technical;
  Phase E will add `tytus help <topic>` as a parallel plain-English
  entry point.
- **Doctor / capabilities output** — diagnostic surfaces, Advanced.
  Internal `Err()` strings, `tunnel-up` log lines, sidecar JSON
  schemas, API field names — untouched.
- **Daemon controls** in Help submenu (`Restart Daemon`, `Stop Daemon`,
  `Start Daemon`, `View Daemon Log`, `View Startup Log`) — Advanced
  power-user surface; verdict bright-line says technical stays.
- **Per-pod headers + actions** in Show all pods ▸ submenu — power-
  user verbs (Open in Browser, Stop Forwarder, Restart, Refresh
  creds, Uninstall, Revoke); technical stays.
- **AIL Connection Info ▸** items (Copy AIL_URL / AIL_API_KEY / OpenAI
  exports / Anthropic exports / JSON) — intentional dev surface for
  power users wiring Cursor / Claude Code / etc.; technical stays.
- **Internal Rust types, struct fields, env vars, file names, log
  lines** — zero changes from rc.2.

### Phase D — Deferred to rc.4

- **Tower strings** — bind form labels ("Bucket name" → "Folder name"),
  "Open ~/.cache/garagetytus" → "Open sync folder", "Stop forwarder"
  → "Stop browser shortcut", chooser success-screen copy. Phase B
  rewrites Tower into a 5-tab SPA; renames batch with that work to
  avoid double-touching the same lines.

### Files touched

- `cli/src/main.rs` — welcome info line, dashboard, recoverable errors
- `cli/src/wizard.rs` — logo subtitle
- `tray/src/main.rs` — status line, tooltip, Quick actions / Settings /
  Help / pods-submenu labels
- `Cargo.toml` — workspace version bump to `0.6.0-rc.3`
- `CHANGELOG.md` — this entry

## [0.6.0-rc.2] — 2026-04-26

Phase C — tray menu simplification. Top-level tray collapses from 14
visible widgets to 8, with three primary verbs (Chat / Files / Channels)
promoted to top-level alongside the full Tower deep-link. Every existing
menu ID + action handler is preserved — backwards-compatible refactor.

### Phase C — Tray simplification

Top-level tray (logged-in, pod connected) is now:

```
🅣 Tytus
├─ ● Connected (sebastian@…)
├─ Plan: explorer · 0 / 1 units · up 4h
├─ ──────────
├─ 💬  Chat now…             ← open_tower_chat → tower#chat
├─ 📁  Files…                  ← open_tower_files → tower#files
├─ 📨  Channels…               ← open_tower_channels → tower#channels
├─ 🌐  Open Tytus Tower        ← unchanged
├─ ──────────
├─ Quick actions ▸             ← Disconnect/Connect, Open in ▸,
│                                Run Health Test, AIL Connection Info ▸,
│                                Show all pods ▸, Shared Folders ▸
├─ ──────────
├─ Settings ▸                  ← unchanged contents
├─ Help… ▸                     ← was "Troubleshoot ▸"; +Documentation +About
├─ ──────────
└─ Quit Tytus
```

8 actionable top-level items. Previous v0.5.5 layout had 14: status +
meta + Open Tower + Connect/Disconnect + Open in + Health Test +
AIL Connection Info + Pods & Agents + Shared Folders + Settings +
Troubleshoot + Documentation + About + Quit. Phase C goal: ≤8 ✓.

- **3 new menu IDs**: `open_tower_chat`, `open_tower_files`,
  `open_tower_channels` — each routes to `web_server::open_tower_at("#chat"|
  "#files"|"#channels")`. Phase B's tab routing in tower.html consumes the
  hash; until Phase B lands, Tower's hashchange handler ignores unknown
  anchors and the page loads at root — no broken behavior, no surprise.
- **Disconnect / Connect, Open in ▸, Run Health Test** moved into
  Quick actions ▸. All three were former top-level items; the verbs
  remain reachable in ≤2 clicks (T → Quick actions → action).
- **Pods & Agents ▸** renamed **"Show all pods"** and nested under
  Quick actions ▸. Per-pod actions (Open in Browser, Channels, Files,
  Restart, Refresh creds, Uninstall, Revoke) all reachable in ≤3 clicks
  (T → Quick actions → Show all pods → click pod). Previously 1 fewer
  click but at the cost of 30+ items dumped onto a top-level submenu.
- **Shared Folders ▸** nested under Quick actions ▸. Per-binding
  click → open in Finder still ≤3 clicks deep.
- **AIL Connection Info ▸** nested under Quick actions ▸. The export
  blocks for Claude Code / Cursor / OpenCode / OpenAI / Anthropic stay
  reachable for power users; grandma never sees them.
- **Documentation + About Tytus** fold into Help ▸ (formerly
  Troubleshoot ▸). Two top-level text-link items go away.
- **Troubleshoot ▸ → Help…** rename. Under Phase D the inner labels get
  vocab cleanup; rc.2 keeps every internal label byte-identical to v0.5.5.

### Backwards compat

- **Every existing menu ID is preserved.** Action handlers in
  `handle_menu_event` are byte-identical for the moved items
  (`disconnect`, `connect`, `test`, `launch_*`, `copy_ail_*`,
  `open_mcp_guide`, `pod_*`, `install_agent`, `install_agent_*`,
  `shared_folders_*`, `units_line`, `pod_header_*`, `no_pods`,
  `docs`, `about`). They live under different parent submenus
  but respond to clicks identically.
- **3 new IDs added** (`open_tower_chat`, `open_tower_files`,
  `open_tower_channels`). Old IDs kept; new IDs additive.
- **Internal types, file names, log lines, CLI surfaces** unchanged
  vs rc.1.

### Files touched

- `tray/src/main.rs` — `build_menu` restructure, new event handlers
- `Cargo.toml` — workspace version bump to `0.6.0-rc.2`
- `CHANGELOG.md` — this entry

## [0.6.0-rc.1] — 2026-04-26

First release candidate of the v0.6 grandma-easy line. Phases 0 + A
landed: cold-start audit, vocabulary policy locked via lope, and the
"first thing the user sees" cleaned up. Internal types, log lines,
CLI subcommand names, and `--help` output stay 100% backwards-compatible.

### Phase 0 — Audit + locked vocabulary policy

- **`development/sprints/queued/TYTUS-V0.6-GRANDMA-EASY/AUDIT.md`** —
  full string-by-string cold-start audit across CLI, tray, and Tower.
  Surfaces the 79-item tray menu density, the 11-section Tower scroll,
  and the 30-command flat `--help` density that motivated v0.6.
- **Q1 — vocabulary policy** locked via `lope ask --validators
  pi,codex --parallel`. Both validators converged on **C — HYBRID**:
  rename a small, well-defined set of user-facing strings (welcome
  banner, primary tray actions, Tower primary tabs + Shared-Folders
  bind form, recoverable error messages); keep technical terms
  everywhere else (logs, Advanced submenus, CLI `--help`, internal
  Rust/JS code, llm-docs, sidecar JSON, API field names). Verdict and
  Phase D replacement table at
  `development/sprints/queued/TYTUS-V0.6-GRANDMA-EASY/verdicts/Q1-VOCAB-LOCKED.md`.

### Phase A — Entry point polish

- **Keychain WARN no longer leaks to stderr.** Previous heuristic in
  `cli/src/main.rs::init_tracing` keyed off
  `console::Term::stderr().is_term()`, which fails any time stderr is
  captured (`tytus 2>&1 | head`, shell redirects, agent harnesses, CI).
  Those are all "user-facing" runs from the user's perspective —
  they didn't ask for log noise. v0.6 drops the TTY detection: stderr
  emit is opt-in only via `RUST_LOG=` or `--json`; everything else
  routes WARN+ to a log file.
- **Log path moves to macOS-conventional location.**
  `~/.tytus/logs/tytus.log` → `~/Library/Logs/tytus/cli.log`
  (cross-platform: keeps `~/.tytus/logs/cli.log` on non-Mac). Tray
  spawn-action logs at `~/.tytus/logs/<action>.log` are unchanged.
- **`tytus setup` end screen** now leads with
  **"Open Tytus.app for the visual interface, or run `tytus chat` to
  start chatting."** before the existing hint list.
- **README leads with the install command.** First content under the
  H1 is now `curl -fsSL https://get.traylinx.com/install.sh | bash` +
  `tytus setup` — followed by a plain-English "What you get" section,
  with the technical paragraph on WireGuard + AIL gateway moved below.

### Backwards compat

- **Zero internal-API changes.** Rust types, struct fields, log lines,
  CLI subcommand names, `--help` clap output, env var names — all
  byte-identical to v0.5.5 except for the workspace `version` field.
- **Vocabulary still says "pod", "tunnel", "agent"** in every surface
  — Phase D applies the locked rename verdict in a later rc.

### Files touched

- `cli/src/main.rs` — `init_tracing` rewrite, setup end-screen hint
- `Cargo.toml` — workspace version bump to `0.6.0-rc.1`
- `CHANGELOG.md` — this entry
- `README.md` — install-first lead
- `development/sprints/queued/TYTUS-V0.6-GRANDMA-EASY/AUDIT.md` (new)
- `development/sprints/queued/TYTUS-V0.6-GRANDMA-EASY/verdicts/Q1-LOPE-RAW.md` (new, in MAKAKOO)
- `development/sprints/queued/TYTUS-V0.6-GRANDMA-EASY/verdicts/Q1-VOCAB-LOCKED.md` (new, in MAKAKOO)

## [0.5.5] — 2026-04-26

The grandma flow. Tower can now bind a Mac folder end-to-end —
without the user typing a single path. Native macOS folder picker,
auto-suggested bucket name, pod toggle chips, server-side validation
of every input, all argv via `Command::arg` (no shell, no injection).

- **"+ Bind a folder…" button** at the top of the Shared Folders
  section expands a 4-step form:
    1. Choose folder…   → triggers a native macOS folder picker
       via the new `POST /api/shared-folders/pick-folder` endpoint
       (osascript bridge to `choose folder with prompt …`). User
       can't fat-finger a path because none is typed.
    2. Bucket name      → auto-suggested from the folder basename
       (lowercased + sanitized + truncated to 63 chars). Live
       client-side regex validation (Garage rules: 3-63 chars,
       lowercase alnum + dot + hyphen, alnum endpoints) gates
       the Submit button.
    3. Pod toggle chips → enumerated from `/api/state.agents`.
       Click each pod to share with it (each gets its own per-pod
       Garage credentials — no shared keys).
    4. Auto-sync checkbox (default ON) → adds `--auto-sync` to the
       spawn so the launchd polling pipeline starts immediately.
- **Submit streams into the existing `sf-panel`** so the user sees
  every phase (bucket create, key mint, rclone setup, initial
  bisync, pod provision per `--to`). On exit code 0: form hides,
  bindings list auto-refreshes, new row appears.
- **Server-side hardening** (defence-in-depth alongside client
  regex):
  - `local_path` rejected unless absolute + exists + is a directory.
  - Bucket name rejected unless it matches the Garage regex.
  - Pod IDs rejected unless `^\d{1,3}$`.
  - Returns 400 with a specific `error` field; UI surfaces the
    message as a toast.
- New `POST /api/shared-folders/bind` (body
  `{local_path, bucket, pods, auto_sync}`) returns `{job_id}` and
  spawns `garagetytus-folder-bind <local> <bucket> [--to N]…
  [--auto-sync]` via the v0.5.4 `spawn_external_command` helper.

## [0.5.4] — 2026-04-26

Tower learns about garagetytus shared folders. Read-only parity
with the v0.5.2/v0.5.3 tray submenu — list bindings, status,
conflicts, refresh credentials — plus a new per-pod "Refresh creds"
button on the Output toolbar. Bind stays tray-only because the
browser sandbox can't surface a real OS folder path.

### Tower additions
- **New "Shared Folders" details section** between Settings and
  Troubleshoot. Auto-refreshing list of bindings (one click =
  open in Finder) reads `~/.cache/garagetytus/bisync/*.bindings.json`
  via the new `GET /api/shared-folders/list` endpoint. Empty case
  shows guidance pointing at the tray + CLI for binding.
- **Streamed action buttons** for Status / Conflicts / List
  (full table) / Refresh-all wire to a new shared `sf-panel` log
  surface mirroring the Doctor / Health Test pattern. SSE event
  stream from `POST /api/shared-folders/run-streamed?action=…`
  via the existing job-event Registry.
- **"Open ~/.cache/garagetytus" button** for direct Finder access
  to bisync workdir + sidecar metadata.
- **Per-pod "Refresh creds" button** added to the Output toolbar
  alongside Restart / Stop forwarder / Uninstall / Revoke. Streams
  `garagetytus-pod-refresh <NN>` into the same per-pod log panel.

### Backend additions
- `GET  /api/shared-folders/list` — returns sidecar JSON list
- `POST /api/shared-folders/run-streamed?action=<status|conflicts|list|refresh-all>`
  — returns `{ job_id }`, output via existing `/api/jobs/<id>/stream`
- `POST /api/shared-folders/open` — body `{local_path}`, opens in
  Finder; 404 on orphan sidecar so the UI can flag it
- `POST /api/shared-folders/open-cache` — opens `~/.cache/garagetytus`
- `POST /api/pod/refresh-creds?pod=NN` — returns `{ job_id }` for a
  per-pod cred rotation
- New `resolve_garagetytus_helper(name)` mirrors the resolver in
  `tray/src/shared_folders.rs::helper_path` so backend + tray agree
  on which binary to invoke (`/usr/local/bin/`, `/opt/homebrew/bin/`,
  `~/garagetytus/bin/`, then bare PATH lookup)
- New generic `spawn_external_command(job, bin, args)` mirrors
  `spawn_pod_action`'s line-buffered stdout+stderr streaming but
  parameterized over the binary path, so any garagetytus-* helper
  (or future external) can stream into Tower's job channel

### Out of scope (intentional)
- **Bind from Tower** — needs a real OS folder path picker, which
  the browser sandbox doesn't expose. Stays tray-only; the empty-
  state hint links to the tray and the CLI.
- **Unbind from Tower** — destructive op better with native modal
  confirmations than a web button.

## [0.5.3] — 2026-04-26

The Shared Folders submenu becomes useful at a glance. Each active
binding now appears at the top of the submenu as a click target —
clicking opens the bound local folder in Finder. No more "where
did I bind that to again?"

- **Dynamic per-binding entries** at the top of the Shared Folders
  submenu. One row per active binding in the format
  `<bucket>  ↔  <local-path>` (home prefix compressed to `~/`).
  Click opens the local folder in Finder. Sidecar JSONs at
  `~/.cache/garagetytus/bisync/*.bindings.json` (written by
  `garagetytus folder bind` v0.5.3+) are the source of truth.
  Empty case shows a disabled "No folders bound yet" item so the
  user can tell the difference between "nothing bound" and
  "garagetytus not installed".
- New `shared_folders::list_bindings()`, `Binding` struct,
  `menu_id_open_binding`, `parse_open_binding_id`, and
  `open_binding_in_finder` helpers — all in
  `tray/src/shared_folders.rs`. 2 new unit tests (round-trip the
  ID + reject unrelated IDs); 17 tray tests total.

## [0.5.2] — 2026-04-26

The tray learns about garagetytus shared folders. Two integration
surfaces — a per-pod entry under each pod's Files submenu, and a
new top-level "Shared Folders" submenu for global ops — wrap the
v0.5.3 garagetytus bash helpers (`garagetytus-folder-bind`,
`-list`, `-status`, `-conflicts`, `garagetytus-pod-refresh`,
`garagetytus-refresh-watchdog`).

- **Per-pod Files submenu** gains 2 entries at the bottom:
  - "Bind a Mac folder to this pod…" — folder picker → osascript
    bucket-name dialog → spawn `garagetytus-folder-bind <path>
    <bucket> --to <pod-id> --auto-sync` in a detached thread →
    macOS notification on completion (Reveal in Finder on success).
  - "Refresh shared-folder credentials" — spawn
    `garagetytus-pod-refresh <pod-id>` and notify on completion.
    Pod's wrapper re-reads creds on every call so no pod restart.
- **New top-level "Shared Folders" submenu** between Pods and
  Settings:
  - List bindings…              `garagetytus-folder-list`
  - Status (with pod check)…    `garagetytus-folder-status --check-pods`
  - Find conflicts…             `garagetytus-folder-conflicts`
  - Open ~/.cache/garagetytus   (Finder)
  - Run cred refresh now (every pod)  `garagetytus-refresh-watchdog`
- **Graceful absence.** Both surfaces probe for the
  `garagetytus-folder-bind` helper at `/usr/local/bin`,
  `/opt/homebrew/bin`, and `~/garagetytus/bin/`. When no helper is
  found, the menu items still appear (so the integration is
  discoverable) but disabled with the suffix
  "(install garagetytus first)". No silent failure on click.
- New `tray/src/shared_folders.rs` module mirrors the `files.rs`
  pattern — menu_id helpers, per-pod action enum + parser,
  `spawn_*` functions in detached threads, 4 unit tests.

## [0.5.1] — 2026-04-25

Tower becomes the output surface for non-interactive tray actions.
No more cascade of Terminal windows for routine operations; per-pod
state, action streams, and channel-add token entry all live inside
one local web page now.

- **Tower in-page actions + per-pod subpages** — tray menu items that
  used to spawn Terminal.app for `test`, `doctor`, per-pod
  `restart` / `revoke` / `uninstall` / `stop-forwarder`,
  `channels catalog`, `channel-remove`, and `channel-add` now
  deep-link the user's browser into the local Tower web UI. New
  hash-route grammar:
    - `#/run/<action>` — global commands (`test`, `doctor`,
      `channels-catalog`). The doctor route also opens Troubleshoot.
    - `#/pod/<NN>` — per-pod subpage with Overview / Output / Channels
      tabs. The Output tab streams subprocess stdout+stderr live via
      the existing SSE Registry/JobEvent infrastructure (no PTY, no
      WebSocket, no xterm.js).
    - `#/pod/<NN>/<action>` — runs a per-pod streamed action
      (`restart`, `revoke`, `uninstall`, `stop-forwarder`); concurrent
      attempts on the same pod return HTTP 409 from the new
      `Registry::create_pod`. Successful revoke auto-navigates back
      to Tower so the user isn't stranded on a 'pod not found' page.
    - `#/pod/<NN>/channels?action=add&type=<channel>` — opens an
      in-page native `<dialog>` to collect the bot token. The token
      rides only the local POST body (127.0.0.1) and is forwarded to
      the `tytus` subprocess as an argv element — no shell, no
      Terminal `read -rs`, no logs. (Threat-model note in the
      handler: the token is briefly visible to local processes via
      `ps aux` for the ~10–15s subprocess lifetime; same exposure as
      the prior Terminal flow.)
  Backend additions: `Job::pod_id`, `JobEvent::Exit { code }`,
  `Registry::create_pod` (busy-check returns 409), `Registry::active_pods`
  (surfaced as `active_jobs_per_pod` in `/api/state` for the running-
  job dot in the overview), strict action whitelist via
  `pod_action_argv` (no free-form strings ever reach the shell —
  Command::arg per token; doctor/test rejected because they aren't
  pod-scoped), `open_tower_at(fragment)` helper that auto-appends a
  nonce so repeat tray clicks force `hashchange` to fire.
  Tray-menu rewires: `test`, `doctor`, `pod_NN_restart`,
  `pod_NN_revoke`, `pod_NN_uninstall`, `pod_NN_channels_catalog`,
  `pod_NN_channel_X_add`, `pod_NN_channel_X_remove` migrated from
  `open_in_terminal_simple` to `web_server::open_tower_at`. Tower-side
  `handle_channels_catalog` and `handle_channels_remove` migrated from
  Terminal-spawn to inline `run_tytus_inline`. `handle_channels_add`
  rewritten to accept JSON `{pod, channel, token}` body; parse-error
  path returns only `{"error":"bad json"}` (never echoes raw body —
  would leak the token).
  Stays in Terminal.app (intentional, requires TTY): `connect` (sudo),
  `login` (Sentinel browser-auth), `logout`, `tray install` (sudo +
  bundler), `configure` (multi-step interactive wizard), autostart
  toggles (sudo plist), editor launches (`launch_<editor>`,
  `launch_terminal`), the "Try Again" reconnect dialog, and the
  install-agent terminal fallback when the localhost Tower server
  isn't bound. CSP is unchanged (`connect-src 'self'`); no new heavy
  deps (no `portable-pty`, no `tokio-tungstenite`, no `xterm.js`).
  Binary delta ~15KB; release build 6.7M. 11/11 tray tests
  (4 new in `web_server::tests`); 68/68 workspace tests; `cargo
  clippy` clean (no new warnings). Verified across 5 lope review
  rounds with `pi` + `qwen` validators.
  Sprint doc: `~/Projects/makakoo/sprints/wannolot-embedded-terminal-2026-04-24/SPRINT.md`.

- **fix(tower): Run Doctor / Run Health Test stream output line-by-line**
  — both endpoints now respond `{job_id}` (HTTP 202) and stream
  subprocess output via SSE on `/api/jobs/<id>/stream`, same pipeline
  as the install flow + per-pod streamed actions. Pre-fix the page
  showed an empty `<pre>` for ~10 s then dumped the whole output at
  once, because `handle_test`/`handle_doctor` used `Command::output()`
  which blocks until exit. Refactored to spawn-piped + Registry
  pattern. Required pairing with the next two fixes (`wizard::flush()`
  + `with_chunked_threshold(0)`) to actually reach the browser as
  bytes arrive.

- **fix(cli): wizard helpers flush stdout per line**
  (`cli/src/wizard.rs::flush()`) — when `tytus test` / `tytus doctor`
  are spawned with `Stdio::piped()` (which the tray now does for the
  streaming pipeline), Rust block-buffers `println!` output. Every
  spinner / finish_ok / print_* line accumulated in stdout's
  `BufWriter` and only flushed at process exit. Added an explicit
  flush after every println in the non-TTY branches of `spinner` /
  `finish_ok` / `finish_fail` / `print_box` plus unconditionally in
  `print_header` / `print_step` / `print_ok` / `print_fail` /
  `print_warn` / `print_info` / `print_hint` / `print_logo` /
  `print_success_banner`. No-op when running in a TTY (already
  line-buffered).

- **fix(tower): force chunked transfer encoding on SSE responses**
  (`tray/src/web_server.rs::sse_response` →
  `.with_chunked_threshold(0)`) — tiny_http's default chunked
  threshold is 32 KB. When the response body length is unknown AND
  total output is shorter, tiny_http BUFFERS the entire body to
  compute `Content-Length` — defeating SSE streaming entirely.
  `tytus test` output is ~1 KB so all frames were buffered to the
  end. Setting `chunked_threshold(0)` forces chunked from the first
  byte: each pipe `read()` produces a chunk that flushes immediately.
  Verified end-to-end with `curl -sN /api/jobs/<id>/stream` piped
  through a millisecond-precision timestamper — frames now spread
  across the subprocess's actual runtime instead of clustering at
  process exit. The install flow's apparent streaming under the old
  code was coincidental: install output crosses the 32 KB threshold
  mid-run, so it switched to chunked then. All other SSE consumers
  (test, doctor, per-pod actions) silently 100% buffered until now.

Hard-won lessons captured as durable memories in the project's
auto-memory store: `feedback_tray_binary_staleness` (rebuild + swap
recipe after tray edits) and `feedback_tiny_http_chunked_threshold`
(the one-line streaming gate).

## [0.5.0] — 2026-04-24

Four themes consolidated from the three in-flight `v0.5.x-alpha`
drops plus the just-shipped shared-folders sprint:

- **Shared folders** — `tytus push / pull / ls / rm / transfers`
  move files between your Mac and any pod. Tray Files submenu per pod
  with osascript picker + macOS notifications. Portable
  `skill-tytus-files` skill with 53 EN+ES trigger phrasings so every
  infected AI CLI can translate "push the PDF to pod 2" / "manda el
  reporte al pod-04" into the right invocation. Full doc at
  `docs/file-sharing.md`.
- **Channels** — `tytus channels add/list/remove/catalog` wires
  OpenClaw's Telegram / Discord / Slack / LINE extensions without a
  browser. Pod egress bridge + DOCKER-USER iptables rules enable
  outbound chat-API reachability without exposing cross-pod traffic
  or the metadata endpoint.
- **Daemon hardening** — state.json mtime watcher + self-heal
  watchdog + stale-PID sweeper + degradation surfaced in tray. Fixes
  the cache-coherence class of bugs (stale in-memory creds, silent
  keychain timeouts, tray showing "Sign In" while logged in).
- **Lope teammates** — `tytus lope ask/install/list/identity` turns
  pod agents (OpenClaw, Hermes) into lope validators; `tytus bridge`
  ships the reverse channel for pod → Harvey notifications via brain
  journal + superbrain event store. Python SDK at `tytus_sdk/`.

### Shared folders (new in 0.5.0)

#### Added

- **`tytus push <LOCAL> [--pod NN] [--to /app/workspace/PATH]`** —
  file or directory push. Directories are tarred + gzipped locally and
  unpacked on the pod. Default destination is `/app/workspace/inbox/`
  (auto-created). Smart `--pod` default: one pod connected → auto-
  picked; multiple → refuses with the list.
- **`tytus pull <REMOTE> [--pod NN] [--to LOCAL]`** — inverse. Files
  + whole directories supported.
- **`tytus ls [PATH] [--pod NN] [--json]`** — list pod contents under
  `/app/workspace/`. Columns: mode, size, mtime, name.
- **`tytus rm <REMOTE> [--pod NN] [--recursive]`** — delete. Refuses
  directories without `--recursive`; refuses anything outside
  `/app/workspace/` unconditionally.
- **`tytus transfers [--tail N] [--pod NN] [--json]`** — reads the
  append-only JSONL audit log. Every `push`/`pull`/`rm` invocation
  (success OR failure) appends exactly one line.
- **Progress bar** on stderr for transfers > 1 MB (`indicatif`),
  suppressed with `--quiet`.
- **100 MB ceiling per transfer** — refused with a clear pointer to
  the planned v0.7 Garage-backed shared filesystem. Deliberate;
  docker-exec base64 streaming is the wrong foundation for GB-scale.
- **`cli/src/transfer.rs`** — shared helpers: path validation
  (rejects outside `/app/workspace/`, `..` segments, NUL bytes), size
  ceiling, `flock`-serialised JSONL transfer log, shell escaping.
- **`cli/src/cmd_transfer.rs`** — command implementations. Chunked
  256 KB base64 via the existing `tytus exec` pipeline (no new infra,
  fits `dash` ARG_MAX on the pod side).
- **Tray Files submenu** — `Pods ▸ pod-NN ▸ Files ▸ Push file… /
  Push folder… / List inbox in Terminal / Open local download
  folder`. Uses osascript for the file picker; posts macOS
  notifications on transfer completion and reveals pulled files in
  Finder.
- **`plugins-core/skill-tytus-files/`** (in `makakoo-os`) — portable
  SKILL.md with 53 EN + ES trigger phrasings ("push PDF to pod 2",
  "manda el reporte al pod 4", "qué hay en el pod 02", "descarga el
  log del pod-04", etc.), decision table, cross-CLI routing notes,
  and a regex-based trigger-corpus test (7 tests green).
- **`docs/file-sharing.md`** — canonical reference: mental model,
  quickstart, tray tour, skill discovery, when-NOT-to-use
  cross-references (to `harvey_knowledge_ingest` for RAG and the v0.7
  Garage sprint for bulk data), troubleshooting cheatsheet.

#### Why

Before this release the only path to move a file onto a pod was a
hand-crafted `tytus exec base64 -d` pipeline. This release makes the
feature feel like `scp` with zero ceremony — one command, smart pod
defaults, refuses path escapes before sending bytes. The 100 MB cap
is the boundary where the base64-over-exec transport stops being a
good idea; past that, wait for Garage.

### Known limitations

- **Drag-and-drop onto the menu bar icon** is deferred. The
  `tray-icon` crate wraps `NSStatusItem` without the
  `NSDraggingDestination` protocol, and subclassing via `objc2` is
  non-trivial. The menu-based file picker covers the same user
  intent in one extra click. Tracked for a follow-up release.
- **Live-pod integration tests** are manual. Unit + concurrency
  tests cover the helper surface (34 tests green across the CLI +
  tray + skill); end-to-end "push → pull → md5-match" is verified by
  hand before cutting.

### Channels (was v0.5.2-alpha)

Unblock OpenClaw's existing channel extensions (Telegram, Discord,
Slack Socket Mode, LINE). Two layers: infrastructure change so pods
can actually reach chat APIs, CLI change so users can configure
credentials without a browser UI.

### Added

- **`tytus channels` subcommand** — `add` / `list` / `remove` / `catalog`.
  Stores chat-channel credentials in the OS keychain, writes the
  per-pod view to `/app/workspace/.tytus/channels.json` via `tytus
  exec`, redeploys the agent container via DAM so the channel
  extension picks up the env vars at startup. Supports Telegram,
  Discord, Slack (Socket Mode), and LINE at launch. Adding more is a
  ~3-line change to `cli/src/channels.rs`.
- **`cli/src/channels.rs`** — static registry of known channels with
  their required env-var mappings, derived from each OpenClaw
  extension's `openclaw.plugin.json` → `channelEnvVars`.
- **`cli/src/channels_store.rs`** — keychain-backed secret storage +
  local manifest at `~/.tytus/channels.json` that tracks which
  channels are configured per pod.
- **DAM channel merging** — `agent_manager/app.py:agent_deploy`
  reads `state_dir/.tytus/channels.json` on every container deploy
  and merges credentials into the container's env. Non-fatal on
  missing/invalid files; only accepts `UPPER_SNAKE_CASE=string`
  entries. Schema is versioned (`"version": 1`) for future additions.
- **Pod-egress bridge** — sidecars now attach to a Docker bridge
  network (`pod-egress`, 172.30.0.0/16) in addition to WireGuard, so
  the OpenClaw/Hermes extensions can reach external chat APIs. No
  published ports → no inbound internet exposure. Cross-pod traffic
  still blocked at the host iptables FORWARD chain; metadata endpoint
  (169.254.169.254) still blocked; outbound allowlist enforced via
  new DOCKER-USER rules.
- **`scripts/e2e-channels.sh`** — 8-flow harness (3 static, 5 live).
  Static flows: binary surface, catalog contents, JSON output shape.
  Live flows (opt-in via `E2E_TELEGRAM_BOT_TOKEN`): add → channels.json
  on pod → container env → api.telegram.org reachable → remove.

### Changed

- **`services/wannolot-infrastructure/docker-compose.pod.j2`** —
  sidecars no longer `network_mode: none`. They now join the
  `pod-egress` bridge network so the pod container (which shares the
  sidecar's netns via `network_mode=container:...`) inherits a default
  route to the internet via Docker's NAT.
- **`services/wannolot-infrastructure/user-data.strato-eu-001.yml`**
  adds DOCKER-USER iptables rules scoped to `172.30.0.0/16`:
  allowlist TCP/443, UDP/53, TCP/53; block metadata + cross-bridge;
  deny everything else.

### Why

OpenClaw already ships first-class chat channel extensions for 20+
chat apps (Telegram, Discord, Slack, Signal, WhatsApp, iMessage,
Line, Matrix, Teams, Feishu, GoogleChat, etc.). They were
unreachable from Tytus pods because:
(a) pods had no internet egress (iptables DROP catch-all +
`network_mode: none` on the sidecar — no default route), and
(b) users had no way to configure bot tokens without the slow
browser UI tunnel.
This release fixes both at the smallest possible surface area: one
iptables chain + one bridge network + one CLI subcommand. No new
services, no new auth model, no broker. The deferred "TML messaging
layer" direction from earlier design drafts is correctly rejected
— see `dev/design/2026-04-20-unblock-openclaw-channels.md` for the
full reasoning.

### Deploy notes (not automatic)

The infrastructure change requires redeploying the sidecar containers
on each droplet:

1. Pull latest `wannolot-infrastructure` on the droplet
2. `cd /opt/wannolot-infrastructure && bootstrap/02-render-compose.sh`
3. `docker compose -f docker-compose.pod.yml down && docker compose -f docker-compose.pod.yml up -d`
4. Re-apply iptables (either reboot or re-run the egress-filter block
   from `user-data.strato-eu-001.yml`)
5. DAM gets the `channels.json` reader via a normal pull + restart of
   its container on the droplet

Until these steps run, `tytus channels add` will write keychain +
push to the pod, but the agent container will come up without the
new env vars (the channel extension will log "missing
TELEGRAM_BOT_TOKEN" and no-op).

### Daemon hardening (was v0.5.1-alpha)

Production-hardening pass against the class of bugs that shipped the
2026-04-20 tray regression: stale in-memory daemon state, broken
keychain ACL silently pinning the daemon to `NeedsLogin`, and tray
reading the stale view instead of state.json. Fix is systemic, not
just the immediate symptom.

### Added

- **`CliState::load_file_only()`** — side-effect-free state.json parse that
  skips the OS keychain. Safe on the status RPC hot path (~1 ms).
- **Daemon state watcher task** (`state_watcher_loop`) — polls state.json
  mtime every 500 ms; hot-reloads the daemon's in-memory credentials
  whenever any other process (`tytus login`, `tytus connect`, `tytus
  revoke`) updates the file. Ends the 30-min drift window.
- **Daemon self-heal watchdog** (`self_heal_loop`) — when the daemon has
  been stuck in `NeedsLogin` for >5 min while state.json is plainly
  logged in, exits so launchd/systemd can respawn with fresh state.
- **Stale-PID sweep** (`sweep_stale_pids`) — at daemon startup, reaps
  `daemon.pid` / `tray.pid` / `tunnel-*.pid` whose owning process is
  dead. Uses `kill -0` semantics (ESRCH ⇒ stale, EPERM ⇒ keep).
- **Daemon health telemetry in status RPC** — new `daemon` fields:
  `keychain_healthy` (bool), `last_refresh_error` (Option<String>),
  `stuck_for_secs` (Option<u64>).
- **Tray surfaces daemon degradation** — `TrayState` carries
  `keychain_healthy` + `last_refresh_error`; menu renders yellow dot +
  "⚠︎ keychain access pending — re-run `tytus login`" row on metadata
  line; Troubleshoot submenu shows last refresh error verbatim.
- **`scripts/e2e-multiprocess.sh`** — 7-flow harness for daemon ↔
  state.json ↔ tray coherence: ping, status-RPC latency, health-field
  presence, mtime-driven reload, NeedsLogin self-clearing, sweep-linked-
  into-binary, tray-merge agreement. Would have caught the 2026-04-20
  regression on the first run.

### Fixed

- **Daemon now hot-reloads state.json** on every `status` RPC call AND
  on file-mtime change (500 ms watcher). Before: up to 30 min staleness
  + indefinite pin when keychain ACL pended.
- **Tray no longer trusts the daemon over state.json for auth.** Merge
  in `tray/src/socket.rs`: file wins on `logged_in`; daemon contributes
  runtime fields (pid, uptime, pods).
- **`refresh_once` is keychain-resilient.** Two-stage reload: file-only
  first; if that yields a valid AT, stay `Running` regardless of
  keychain outcome. Transient keychain failures no longer flip the
  daemon into `NeedsLogin`.

### Why

The tray was showing "Sign In…" while the user was clearly logged in,
the tunnel was up, and pods were allocated. Root cause wasn't a tray
bug — it was a cache-coherence bug across three processes with no
invalidation protocol: daemon memory ↔ state.json ↔ keychain. Silent
keychain timeouts (logged, but invisible to the user) had pinned the
daemon to `NeedsLogin` 19 hours before the user noticed. Fix is the
watcher + self-heal + health surfacing together — each alone is
insufficient.

### Lope teammates (was v0.5.0-alpha)

Tytus pod agents are now first-class lope teammates with a reusable
Python SDK and a bidirectional bridge back to Harvey (brain journal +
superbrain event store). `tytus lope install` pairs a device on the pod
and registers a `subprocess` provider in `~/.lope/config.json` so
`lope negotiate --validators tytus-openclaw-<pod>` Just Works.

### Added

- **`tytus_sdk/` Python package** — reusable adapter SDK. Files:
  - `adapter.py` (`AgentAdapter` Protocol with `ask/stream/notify/identify`)
  - `identity.py` (Ed25519 keypair at `~/.tytus/openclaw/device.json`, 0600)
  - `adapters/openclaw.py` (OpenClaw WS v3 + v2-canonical Ed25519 handshake, fresh session per ask, `chat{state:"final"}` terminal detection)
  - `install.py` (pod device pairing via `tytus exec` + `~/.lope/config.json` merge)
  - `lope_bridge.py` (VERDICT-emitting subprocess validator with defensive fallback block when the agent skips the rubric)
  - `bridge_daemon.py` (HTTP listener `127.0.0.1:18099`, per-pod outbox pollers, lifecycle guard)
  - `cli.py` (argparse dispatcher — `ask / identity / install / uninstall / list / lope_validate / bridge`)
- **`tytus lope ask --pod NN "…"`** — direct WS ask against OpenClaw. Live reply verified against pod 02 (MiniMax M2.7).
- **`tytus lope install --pod NN`** — idempotent: adds our Ed25519 device to the pod's `/app/workspace/.openclaw/devices/paired.json` with `operator.{read,write,admin}` scopes, registers the `tytus-openclaw-NN` provider in lope.
- **`tytus lope uninstall` / `tytus lope list` / `tytus lope identity`** — inverse + inventory + pubkey dump.
- **`tytus bridge run`** — daemon: binds `127.0.0.1:18099`, spawns per-pod outbox pollers, drains `/app/workspace/.harvey-outbox.jsonl` every 10 s via `tytus exec`, writes to today's Brain journal + best-effort `superbrain remember`. Shared-secret auth via `X-Tytus-Bridge-Token` (kept at `~/.tytus/bridge.token`, mode 0600). Rate limit 30 notifies/pod/hour.
- **`tytus bridge status / rotate-token / test`** — ops surface.
- **`scripts/e2e-lope-teammate.sh`** — 10-flow harness covering SDK imports, identity, ask, VERDICT emission, lope registration, bridge auth (reject + accept), outbox polling end-to-end, lifecycle guard. Verified 10/10 green on pod 02.
- **`docs/DESIGN-TYTUS-LOPE-TEAMMATES.md`** + lope-negotiated sprint doc + **`docs/SECURITY-TEAMMATES.md`** covering device-key 0600, bridge-token isolation, rate-limit invariants, threat model with 7 open items tracked for v0.6+.

### Changed

- **Rust CLI gains `Commands::Lope` + `Commands::Bridge`** — thin pass-through subcommands that shell out to `python3 -m tytus_sdk`. SDK is the source of truth for protocol work; Rust side only handles CLI parsing, PYTHONPATH detection, and subprocess dispatch. Keeps v0.5 changes out of the Rust build surface.

### Phase 1 implementation notes (hard-won lessons)

- **Silent-local-pairing is unreachable over WG.** Server's `isLocalDirectRequest` requires loopback `req.socket.remoteAddress`; WG traffic arrives with the peer's WG IP.
- **Token-only connects get all scopes stripped.** `clearUnboundScopes` fires whenever `!device && authMethod==="token"`. Device identity is mandatory for write scopes.
- **`deviceId` must be `sha256(pub_raw).hex()`** — full 64 hex chars, matching `deriveDeviceIdFromPublicKey`.
- **`client.id` enum is strict.** `gateway-client` + `client.mode="backend"` avoids the Control-UI device-identity gate while keeping operator scope semantics.
- **`thinking` is required string**, not nullable; `"off"` disables reasoning.
- **Fresh session per ask.** Reusing `key="main"` binds to the pod's long-running `agent:main:main` and inherits full agent-orchestration loop. Unique `tytus-lope-<uuid>` key + unique label per ask.
- **Terminal signal is `event:"chat", state:"final"`** scoped to the sessions.send `runId`, not `session.message.status`.
- **Brain-outbox parser gotcha.** Python's `splitlines()` strips trailing `\n` — rebuilding with `"\n".join()` loses the "this line is complete" signal. Fixed by preserving the raw stdout from `tail -c +N` and testing `"\n" in body` directly.

### Known gaps (tracked for v0.6.0)

- `HermesAdapter` REST path — not shipped; design valid, just not coded.
- Keychain-backed bridge + device tokens (currently 0600 flat files).
- Pod-side `tytus_notify.py` helper not bundled in agent images (agents must append to outbox manually until v0.6 infra rebuild).
- Forwarder reverse-tunnel (Option 1 in §7.2 of design doc) — still polling JSONL via `tytus exec`.
- Audit log on pod for device-pair adds/removes.

## [0.4.0] — 2026-04-19

Zero-config Hermes + OpenClaw "one click → working chat" across the full
browser + SDK surface, plus a cold-boot reliability fix for macOS.

### Added

- **Hermes agent zero-config.** `tytus connect --agent hermes` now
  yields a working dashboard + API out of the box. The forwarder
  proxies `http://localhost:18700+pod_num/` to both the Hermes
  dashboard (Vite/React SPA, port 9119) and the Hermes gateway
  (OpenAI-compatible API, port 8642), multiplexing by path:
  `/v1/*`, `/api/jobs*`, `/health*` → gateway; everything else →
  dashboard. Auth (`API_SERVER_KEY`) auto-injected on gateway routes;
  dashboard's own session token is baked into the HTML by
  `hermes dashboard` itself. Commits `fbf1da9`, `0fc13f4`.
- **OpenClaw silent local pairing.** Browser connections to a
  nemoclaw pod now complete handshake without the "pairing required"
  prompt and without the user pasting a gateway token. Forwarder
  issues a 302 that seeds `?token=<T>` for the UI to strip via
  `history.replaceState`, keeps Host/Origin loopback so
  `isControlUiBrowserContainerLocalEquivalent` fires, and writes a
  `config.user.json` overlay that adds
  `http://localhost:18700+N` to `gateway.controlUi.allowedOrigins`
  (survives agent restart). Commits `b633c96`, `81d3c4a`, `3db77be`,
  `1603167`, `fb912e7`.
- **Forwarder self-heal.** On startup, the forwarder verifies the
  overlay is present and `gateway_token` is populated; recovers
  silently if either is missing by fetching from the pod via
  Provider's A2A path (no keychain round-trip needed). Commit
  `fb912e7`.
- **E2E flow harness.** `scripts/e2e-flows.sh` runs 35 flows across
  AUTH / POD / UI (nemoclaw + hermes) / ENV / DIAGNOSTICS / TRAY /
  HERMES-SIM. Safe to re-run, no destructive actions.
  `scripts/FLOWS.md` is the human matrix. Commit `2b86077`.
- **Sprint planning docs.** `docs/SPRINT-2026-04-19.md` (solo),
  `docs/SPRINT-2026-04-19-negotiated.md` (3-round lope-negotiated),
  `docs/SPRINT-P1-SHIP-v0.4.0.md` (focused ship plan). The
  negotiated versions apply validator feedback from claude / gemini
  / pi / qwen panel.

### Changed

- **`is_logged_in` now accepts a valid access token without a
  refresh token.** Previously required both. On macOS cold boot the
  keychain ACL can take seconds to approve after login and
  `get_refresh_token` times out in 3s — the old check saw `has_rt=
  false` and refused to connect even with a currently-valid AT. The
  daemon still retries the keychain in the background; once it
  unblocks, normal RT refresh resumes. Commit `b9d44df`.
- **Forwarder's `Authorization` header handling is now override,
  not preserve.** OpenAI SDK clients always send a placeholder
  `Bearer <api_key>`; preserving it meant upstream rejected every
  request. The forwarder is now the source of truth — any
  client-supplied Authorization is replaced with the real per-pod
  secret. Commit `0fc13f4`.
- **Forwarder no longer rewrites `Host` / `Origin`.** These must
  stay loopback for OpenClaw's silent-local-pairing path to fire.
  Commit `3db77be`.
- **Forwarder streams responses** instead of buffering the full
  body in memory before writing to the client. Browsers parse the
  bundle head while the tail is still on the wire; observed
  first-byte time dropped from ~130s to ~3s on cold-cache loads
  over `~5 KB/s` boringtun tunnels. Commit `ecd35da`.
- **Forwarder auto-invalidates stale config overlay.** The
  nemoclaw-configure.sh script regenerates `config.json` on every
  restart; the forwarder's overlay writer now uses
  `config.user.json` (deep-merged at restart) instead of mutating
  the regenerated file. Commit `1603167`.
- **`tytus ui` is production-ready as a daemon.** Detaches via
  `setsid`, ignores SIGHUP, survives Terminal close. Per-pod static
  asset cache at `/tmp/tytus/ui-<pod>-cache/` for instant reloads.
  Commits `ea5e0ba`, `ad176fd`, `e59782d`.
- **Forwarder prefetches the Vite chunk graph** after caching the
  main bundle, so dynamic imports don't blow up the tunnel with
  serial small requests. Commit `8b6cf10`.
- **Tray menu reflects state changes within ~1 second**, driven by
  a filesystem-signature watcher + action fan-out rather than pure
  polling. Commits `a7783da`, `d0e8836`.

### Fixed

- **Duplicate tunnel daemons.** Prevented at connect time via a
  pidfile pre-check; stale daemons mopped up after disconnect with
  a bounded iteration. Earlier this manifested as two boringtun
  instances fighting over the same WG socket; 2+ minute page load
  pathology. Commit `961676a`.
- **Doctor's tunnel check** now uses live pidfile + ps-p liveness
  rather than just state.json. Commit `1346dde`.
- **Three production-blockers** found during a sprint smoke test:
  racy tunnel teardown, leaked temp files under `$TMPDIR`, missing
  `Origin` rewrite on specific request paths. Commits `9554c14`,
  `603c333`.
- **Tray "Open in Browser"** reuses an existing forwarder instead
  of spawning a new one on port+1, and no longer pops a Terminal
  window on repeat clicks. Commits `708aeed`, `f772cd5`, `54f1885`.

### Shipped with (infrastructure)

Companion `wannolot-infrastructure` repo changes land in the same
deploy wave (push `main` → `production` on that repo to apply):

- `153e216` — hermes pod runs gateway (8642) + dashboard (9119) via
  both-servers entrypoint; DAM returns `ports.ui` alongside
  `ports.api` from `/agent/<N>/status`.
- `4c0021d` — hermes API_SERVER_KEY auto-derived from
  `sha256(AIL_API_KEY + TYTUS_POD_ID)[:48]` if not injected; written
  to `/app/workspace/.hermes/api_server_key` for the forwarder.
- `a3d4021` — switchailocal pin v0.4.0 → v0.4.1 (capability bridge
  fix).
- `5cd43f5` — switchailocal pin v0.3.1 → v0.4.0 (prerequisite of
  the above).

### Known gaps

- **LaunchAgent oneshot tunnel reap on cold boot** — see
  `docs/SPRINT-2026-04-19-negotiated.md` Phase 2. Workaround until
  fixed: manual `tytus connect --pod NN` after login, or
  `sudo -n tytus tunnel-up /tmp/tytus/tunnel-NN.json` from a shell.
  Planned fix is either `AbandonProcessGroup=true` on the plist or
  a dedicated `tytus tunnel-supervise` KeepAlive=true service.
- **Unsigned binaries** — Apple Developer enrollment is a
  prerequisite. Keychain ACL re-approval on every binary update is
  invisible to LaunchAgents and hits silent-failure cold-boot
  scenarios. Planned fix is Phase 3 of the negotiated sprint.
- **Cross-repo item** — Hermes gateway telemetry schema tracked as
  an issue in `traylinx/wannolot-provider` (see CHANGELOG cross-ref
  once filed).

### Upgrade notes

- Users running OpenClaw / Hermes pods allocated on **pre-v0.4.0
  tytus-hermes image** will hit forwarder-multiplex mismatches
  until the droplet rebuilds the image. After infra main →
  production promotion + `bootstrap/03-pull-images.sh`, restart
  existing hermes pods via `tytus restart --pod NN`.
- Existing users on unpatched v0.3.x should upgrade to v0.4.0 to
  pick up `is_logged_in` AT-only fallback before their next macOS
  reboot. Without the fix, keychain-slow cold boots silently fail
  autostart.

## [0.3.0] — 2026-04-13

Earlier work included; see `git log v0.2.0..v0.3.0 --oneline` for
commit-level detail. This file starts at v0.4.0 as the canonical
release log.

[0.4.0]: https://github.com/traylinx/tytus-cli/releases/tag/v0.4.0
[0.3.0]: https://github.com/traylinx/tytus-cli/releases/tag/v0.3.0

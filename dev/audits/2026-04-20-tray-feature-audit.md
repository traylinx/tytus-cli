# Tytus Tray — Feature Audit

**Date:** 2026-04-20
**Trigger:** user feedback — "we fix A and break B, fix B and break A …
the quality is bad … we should list every feature, write use cases, and
verify each one works."

**Approach:** enumerate every tray action. For each:

1. **What it's supposed to do** (one sentence)
2. **Expected user experience** (click → what happens)
3. **Status** — one of: ✅ works · 🟡 broken/fragile · ❓ untested · 🔴 known bad
4. **Evidence** (observation date, commit, what I saw)
5. **Fix path** (what needs to change to get to ✅)

When this doc is fully ✅ column, Tytus tray is ship-ready for non-dev
users. Anything that depends on sudo / elevation / keychain that can
fail silently is held to a higher bar: it must either work or report a
clear human-readable failure.

**Rule:** no new feature work until every row below is ✅. We fix what
we have before building more.

---

## 1 · Top-level menu actions

| # | Action | Status | Expected behavior | Evidence | Fix path |
|---|---|---|---|---|---|
| 1.1 | Status line ("🟢 Connected") | ✅ | Shows dot color (green/yellow/red), email, busy state ("⏳ Connecting… (Ns)") during actions | Verified 2026-04-20 after `90a230c` (busy status) | — |
| 1.2 | Metadata line ("Plan: operator · 2 pods · up 4h") | ✅ | Plan tier, pod count, uptime | Observed working | — |
| 1.3 | Keychain warning ("⚠︎ keychain access pending") | ✅ | Surfaces when daemon can't reach keychain | v0.5.1-alpha fix verified | — |
| 1.4 | **Connect** | 🟡 | Click → sudo/Touch ID prompt → tunnel up → menu flips to Connected. Should take <15s on warm sudo cache. | 2026-04-20 13:39: stuck at "Connecting… (29s)" because `tytus connect` was spawned detached, sudo hung with no TTY. **Fix pushed 2026-04-20:** Terminal window again for sudo, auto-closes on success. Needs re-verification. | Retest now. If Terminal still opens on success, auto-close may not be working. |
| 1.5 | **Disconnect** | ❓ | Click → tunnel drops → menu flips to Disconnected. <2s. | Detached background spawn (no sudo needed). Need to re-verify. | Test: click disconnect → watch menu → confirm 🔴 within 3s |
| 1.6 | **Sign In…** | ❓ | Click → browser opens for Sentinel device auth → on success, menu goes to Connected email | Terminal-based today. Should work. | Test fresh. |
| 1.7 | Open in ▸ (CLI launchers) | ❓ | Lists installed Claude Code / Cursor / OpenCode / Codex; clicking opens that CLI with pod env vars | launcher::detect_installed_clis() — should work for installed CLIs | Test with Claude Code + Terminal |
| 1.8 | Run Health Test | ❓ | Runs `tytus test` in a Terminal, shows pass/fail of tunnel, gateway, chat | Works today; verify output is not cluttered by keychain WARN | Test. Quick. |
| 1.9 | AIL Connection Info ▸ (Copy URL / Key / exports) | ❓ | Copy-to-clipboard works; info rows show the stable endpoint + redacted key | Copy handlers use osascript → pbcopy | Test each of the 7 copy items |

## 2 · Pods & Agents submenu

| # | Action | Status | Expected behavior | Evidence | Fix path |
|---|---|---|---|---|---|
| 2.1 | Default Pod row (info only) | ❓ | Shows "Default Pod NN — AIL only (0 units)" as disabled text | Should work. | Confirm |
| 2.2 | Pod header ("Pod 02 — OpenClaw (1 unit)") | ❓ | Shows pod + agent + unit count. Disabled. | Should work. | Confirm |
| 2.3 | Open in Browser | ❓ | Starts localhost forwarder if needed, opens browser to OpenClaw UI. 3-state label (✓ if already running / plain / "Connect & Open"). | 2026-04-20 observed 689 KB bundle cold load = 3 min over tunnel. Known slow, not broken. | Test. Slowness is expected per perf audit. |
| 2.4 | Stop Forwarder | ❓ | Kills the localhost forwarder, menu removes the "✓" | Should work — same code path as before. | Test. |
| 2.5 | Restart Agent | ❓ | Terminal opens, `tytus restart --pod NN` runs | Terminal-based. Should work. | Test. |
| 2.6 | Uninstall Agent (keeps pod) | ❓ | Confirm dialog → Terminal runs `tytus agent uninstall`. Pod slot stays, container gone. | Confirm dialog + Terminal. Should work. | Test. |
| 2.7 | **Channels ▸ (submenu per pod)** | 🟡 | Lists configured channels, per-channel Remove, "Add Telegram…" / "Add Discord…" / etc, "Browse available channels…". Add opens Terminal with hidden-token prompt. | Added 2026-04-20 commit `39fb170`. Terminal-based add flow verified in earlier screenshot. But relies on pod having egress + DAM reader (needs droplet redeploy). | Test `tytus channels catalog` works first, then wait for droplet deploy before testing live add flow. |
| 2.8 | Revoke Pod | ❓ | Destructive: frees units, wipes pod state. Must have confirm dialog. | `pod_XX_revoke` handler exists. Needs confirm. | Verify confirm dialog fires. |
| 2.9 | Install Agent… | ❓ | Opens browser wizard for catalog-driven install | `web_server::open_wizard()`. Localhost port file at `/tmp/tytus/tray-web.port`. | Test. |
| 2.10 | Install Agent (terminal) ▸ | ❓ | Legacy/fallback submenu with OpenClaw (1 unit) + Hermes (2 units) terminal shortcuts | Terminal-based. Should work. | Test. |
| 2.11 | Units: N / M used | ❓ | Budget line at bottom of Pods & Agents submenu | Should work. | Confirm. |

## 3 · Settings submenu

| # | Action | Status | Expected behavior | Evidence | Fix path |
|---|---|---|---|---|---|
| 3.1 | Configure Agent… | ❓ | Opens Terminal with `tytus configure` (interactive overlay editor) | Terminal-based. Should work when daemon+login are live. | Test. |
| 3.2 | Start Tunnel at Login | ❓ | Toggles `com.traylinx.tytus.plist` LaunchAgent. Shows ✓ when installed. | Toggle action; verify plist is actually created/removed. | Toggle on → confirm file exists → toggle off → confirm gone. |
| 3.3 | Launch Tray at Login | ❓ | Toggles `com.traylinx.tytus.tray.plist`. Shows ✓ when installed. | Same pattern. | Same test. |
| 3.4 | Install Tytus in Applications… | ❓ | Only visible if `/Applications/Tytus.app` doesn't exist. Runs `tytus tray install`. | Conditional menu item. | Test on a machine without the bundle. |
| 3.5 | Sign Out | ❓ | Destructive: revokes pods, clears creds. Confirm dialog required. | Confirm dialog exists. | Verify full flow (revoke + clear + menu reset). |

## 4 · Troubleshoot submenu

| # | Action | Status | Expected behavior | Evidence | Fix path |
|---|---|---|---|---|---|
| 4.1 | Keychain / refresh-error info rows | ✅ | Shows when daemon reports degraded state (v0.5.1-alpha) | Added 2026-04-20 | — |
| 4.2 | Doctor | ❓ | Terminal opens running `tytus doctor` full diagnostic | Should work. | Test. |
| 4.3 | View Daemon Log | ❓ | Opens `/tmp/tytus/daemon.log` in default viewer (Console.app on macOS?) | Need to verify which viewer opens | Test on macOS. |
| 4.4 | View Startup Log | ❓ | Opens `/tmp/tytus/autostart.log` | Same pattern. | Test. |
| 4.5 | Restart Daemon | ❓ | `launchctl kickstart` the daemon. Menu auto-refreshes. | Should work. | Test. |
| 4.6 | Stop Daemon / Start Daemon | ❓ | `launchctl bootout/bootstrap` appropriately. Menu updates. | Should work. | Test. |

## 5 · Bottom-level

| # | Action | Status | Expected behavior | Evidence | Fix path |
|---|---|---|---|---|---|
| 5.1 | Documentation | ❓ | Opens `https://get.traylinx.com` or similar in browser | Link-open via `open` crate. | Verify URL is current. |
| 5.2 | About Tytus | ❓ | Shows version + GitHub link dialog | osascript dialog. Verify version string is accurate. | Test. |
| 5.3 | Quit Tytus | ✅ | Quits the tray process. Only "Quit per-session" if app-bundle + launch-at-login installed; otherwise warn that manual relaunch is needed. | Works. | — |

## 6 · Non-menu behaviors

| # | Behavior | Status | Expected | Evidence | Fix path |
|---|---|---|---|---|---|
| 6.1 | Single-instance guard | ✅ | Second launch detects lock, exits cleanly | `/tmp/tytus/tray.pid` + PID alive check | — |
| 6.2 | Auto-recovery on daemon crash | ✅ | launchd KeepAlive respawns daemon within seconds | Verified | — |
| 6.3 | Auto-refresh after external state change | ✅ | FS watcher on `/tmp/tytus/` wakes poll loop in ~200ms (shell ran `tytus disconnect`) | Condvar + 200ms poll | — |
| 6.4 | Menu stays fresh while open | ✅ | 1.5s steady / 1s when busy. Elapsed counter ticks. | Added 2026-04-20 | — |
| 6.5 | Tooltip | ❓ | Hover menu bar icon shows short status ("Tytus — Connected (email)") | Verify | Test. |
| 6.6 | Notifications | ❓ | macOS notifications fire on connect/disconnect/channel-add outcomes | osascript. Should work; verify not rate-limited. | Test. |
| 6.7 | Icon color changes (green/yellow/red) | ❓ | Icon color reflects HealthDot state | Verify all 3 states render | Test each by simulating disconnect, no-login, connected. |
| 6.8 | Log files at `~/.tytus/logs/` | ✅ | Interactive CLI runs route WARN+ to log file, not stderr | v0.5.2-alpha `a242bfa` | — |

---

## 7 · What we will test, in what order

The point of this doc isn't to gaze at a table. It's to be a checklist
we actually run against a live tray. Order chosen to hit the most
common non-dev user flow first; each step unblocks the next.

### Pass 1: daily-use flow (must all be ✅ before anything else)

1. `1.4 Connect` — click → Terminal opens → sudo prompt → tunnel up → Terminal auto-closes → menu 🟢
2. `1.5 Disconnect` — click → busy status → menu 🔴 → Terminal never opens
3. `1.4 Connect` again — Terminal opens again → …
4. `2.3 Open in Browser` (OpenClaw UI) — click → localhost forwarder starts → browser opens OpenClaw
5. `1.9 AIL Connection Info ▸ Copy export block` — clipboard contains the shell exports

### Pass 2: channel configuration flow

6. `2.7 Channels ▸ Browse available channels…` — Terminal shows catalog
7. `2.7 Channels ▸ Add Telegram…` (with a real bot token) — prompts for token → runs add → shows ✓ → returns
8. Re-open menu → Channels shows "Telegram ✓ (1 secret)"
9. Tell the bot `hello` from your phone → agent replies
10. `2.7 Channels ▸ Remove Telegram` — confirm → runs remove → menu no longer shows Telegram

### Pass 3: admin / recovery

11. `3.5 Sign Out` → confirm → revokes pods → menu returns to "Not logged in"
12. `1.6 Sign In…` → browser auth → menu returns to Connected
13. `4.2 Doctor` — Terminal shows full diagnostic, no errors

### Pass 4: persistence / settings

14. `3.2 Start Tunnel at Login` toggle — on → reboot → tunnel up automatically → off → reboot → tunnel not up
15. `3.3 Launch Tray at Login` toggle — similar
16. `4.5 Restart Daemon` — daemon PID changes, menu doesn't freeze

### Pass 5: edge cases

17. Click Connect twice rapidly — second click no-ops (button disabled while busy) ✅ already wired
18. Quit daemon manually while tunnel is up — tray shows "⚠︎ daemon offline" but dot stays 🟢 because gateway is reachable
19. Kill tunnel manually → tray detects within 2s → dot flips to 🟡

## 8 · Rule for future commits

Every feature fix gets:

1. **A manual test** run from this checklist before commit.
2. **A regression check** — confirm nothing in Pass 1 broke.
3. **An entry in `scripts/e2e-tray.sh`** (doesn't exist yet — build it
   as we go) so the CI-style harness grows with every fix.

No silent sweeps of "we'll verify later."

---

## 9 · Working principle

The tray has ~30 user-facing actions. That's small enough to fully
enumerate, small enough to fully test, small enough to fully document.
We've been pretending it's a feature-rich app that needs to move fast.
It's not — it's a focused status+control surface for one thing
(Tytus pods). Treating it as such is how we get to production.

No new feature work until every row above is ✅.

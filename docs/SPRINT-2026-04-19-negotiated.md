# SPRINT-POST-HERMES-PROD-READINESS

## Origin: user request

Post-Hermes production-readiness sprint: deploy pending commits (infra `main` → `production`, CLI v0.4.0 release, Hermes pod verification), close the cold-boot gap where the LaunchAgent oneshot reaps the tunnel daemon, fix unsigned-binary keychain-ACL invalidation, and add the minimum observability needed to diagnose the above in the field. Covers SPRINT-2026-04-19 items 1, 2, 3, 4, 5, 6, 7, 8, 9, 10. Item 11 (WG rekey watchdog) is explicitly **out of scope** — see Phase 2 notes.

## Phases

### Phase 1: Ship pending work to production (infra + CLI v0.4.0 + docs + provider coordination)

**Goal:** One coordinated release wave: infra 4 commits `main` → `production`, CLI v0.4.0 tag with unsigned artifacts (signing arrives in Phase 3), `llm-docs.md` reconciled with shipped behavior, and provider-repo owner identified for item #9 (cross-repo Hermes telemetry). Covers items 1, 2, 3, 6, 7, 9.

**Criteria:**
- Infra branch promotion: `wannolot-infrastructure/production` fast-forwarded from `main`. Evidence: `git log production..main --oneline` shows exactly the 4 commits `153e216, 4c0021d, a3d4021, 5cd43f5`; `bash bootstrap/99-validate-all.sh` exits 0 on canary droplet before fanout.
- **Hermes dashboard cold-boot (item 6) — investigation first, patch second.** The proposal to add `RUN cd /opt/hermes/web && npm install` is unverified: `hermes/entrypoint.sh` (lines 236–243) invokes `hermes gateway run` and `hermes dashboard --host 0.0.0.0 --port 9119 --insecure` as compiled subcommands with no npm step, so either (a) the dashboard binary bundles static assets, (b) it lazy-builds from `/opt/hermes/web` on first request, or (c) the slowness doesn't exist. Deliverable: 30-min spike on a fresh `tytus-hermes:latest` — `docker run --rm -it --entrypoint sh tytus-hermes:latest -c 'ls /opt/hermes/web 2>/dev/null; which hermes; hermes dashboard --help'` — then time first 9119 response. Only commit a Dockerfile change if the spike confirms the root cause matches the proposed fix. If slowness isn't npm, document the real cause and size the fix; if no slowness, close item 6.
- CHANGELOG (single source of truth, file absent today — `Glob services/tytus-cli/CHANGELOG.md` returns no match) with v0.4.0 entry: forwarder multiplex (ffa33c3, b9d44df), `is_logged_in` AT-only fallback, e2e harness. `RELEASE-NOTES-v0.4.0.md` is a one-line pointer to the CHANGELOG section, not a parallel document.
- `Cargo.toml` workspace version bumped `0.3.0 → 0.4.0` (evidence: current value is `version = "0.3.0"` at `services/tytus-cli/Cargo.toml:14`).
- **Artifact build path:** unsigned v0.4.0 built locally via `cargo build --release` on a macOS + Linux box respectively; `gh release create v0.4.0 --draft` with the two binaries + their shasum. No CI exists; this is manual and documented in a new `docs/RELEASE.md`. Signed artifacts wait for Phase 3.
- `llm-docs.md` updated with Hermes browser UI, forwarder multiplex (`/v1/*` + `/api/jobs/*` → 8642, else → 9119 — evidence: `hermes/entrypoint.sh:225-227`), `config.user.yaml` overlay path, OpenClaw `config.user.json` overlay, `is_logged_in` AT-only path. Hosted `.agents/skills/tytus/SKILL.md` mirrored.
- **Cross-repo coordination (item 9):** Hermes gateway metrics/telemetry live in `traylinx/wannolot-provider`. Before P1 closes, open a tracking issue on that repo with the telemetry schema the CLI expects, confirm a provider-team owner, and record their ETA. If not owned by end-of-P1, escalate — item 9 cannot slip past P1.
- **OpenClaw regression smoke:** forwarder multiplex must not break non-Hermes pods. `tytus connect --agent nemoclaw` + `tytus chat` succeeds on a freshly-allocated OpenClaw pod.
- **Rollback plan (revised):** infra images are local `docker build`s tagged `:latest` only (evidence: `bootstrap/03-pull-images.sh:27` — `docker build -t tytus-hermes:latest …`; no version tags). Plain `git revert` + `docker compose restart` will NOT restore the prior image. Rollback procedure: `git revert` on `production` → re-run `bootstrap/03-pull-images.sh` on every droplet → `docker compose up -d --force-recreate`. Add a preflight `docker tag tytus-hermes:latest tytus-hermes:pre-v0.4.0` snapshot before fanout so per-droplet rollback can pin to the snapshot.

**Files:**
- `services/wannolot-infrastructure/` — branch promotion; Dockerfile.pod change only if spike confirms it.
- `services/wannolot-infrastructure/hermes/docker/Dockerfile.pod` — conditional on spike outcome.
- `services/tytus-cli/Cargo.toml` — bump workspace version to `0.4.0`.
- `services/tytus-cli/CHANGELOG.md` — new; canonical release log.
- `services/tytus-cli/docs/RELEASE-NOTES-v0.4.0.md` — one-line pointer to CHANGELOG.
- `services/tytus-cli/docs/RELEASE.md` — new; manual build/tag/upload procedure.
- `services/tytus-cli/llm-docs.md` — Hermes UI, multiplex, overlays, `is_logged_in`.
- `services/tytus-cli/install.sh` — confirm release-tag fallback resolves v0.4.0 asset names.
- `.agents/skills/tytus/SKILL.md` (hosted mirror).
- Provider-repo tracking issue link recorded in CHANGELOG under "known gaps".

**Tests:**
- `bash services/wannolot-infrastructure/bootstrap/99-validate-all.sh` exits 0 on canary.
- `docker run --rm tytus-hermes:latest` — dashboard responds on 9119 within the time-target the spike establishes (fixed after investigation).
- `scripts/e2e-flows.sh --pod NN` PASS on a Hermes pod AND an OpenClaw pod (regression).
- Clean-machine install: `curl … install.sh | sh` then `tytus --version` → `0.4.0`.
- `tytus llm-docs | grep -E 'multiplex|9119|is_logged_in'` returns hits.
- Rollback drill on canary: snapshot-tag → deploy → revert → re-pull → verify prior `:latest` digest matches snapshot.

### Phase 2: Tunnel survives cold boot (#11 deferred)

**Goal:** Keep the boringtun daemon alive across login after the LaunchAgent oneshot exits. Covers item 4 only.

**Scope exclusion — item 11 (WG rekey watchdog):** The proposed "no successful handshake in 5 min" detector is not feasible against the current architecture without new infrastructure. Evidence: `tunnel/src/monitor.rs` is 36 lines of TCP-connect probing to `gateway_ip:18080` (verified — entire file reviewed) with zero handshake awareness; the boringtun event loop runs in a separate root process spawned by `tytus tunnel-up` and exposes no IPC for peer stats. Adding handshake-level detection requires either in-process boringtun embedding in the CLI or a new status socket alongside `/tmp/tytus/tunnel-NN.pid` — both are their own design exercises. Defer to a follow-up sprint with an explicit "daemon IPC surface" phase. Keep the existing TCP probe as the health signal until then.

**Criteria:**
- **Acceptance gate is a physical reboot of a dev Mac**, not just `launchctl` cycles. Within 60s of login, `tytus status` shows connected with zero terminal interaction; `/tmp/tytus/tunnel-NN.pid` and the `utun*` interface are still alive 30 min post-reboot.
- Attempt 1: add `<key>AbandonProcessGroup</key><true/>` to the `com.traylinx.tytus.plist` template in `cmd_autostart` at `cli/src/main.rs:3227` (plist body approx line 3249+). The root-owned boringtun daemon is spawned by the sudo'd `tytus tunnel-up` inside the oneshot's process group; `AbandonProcessGroup` prevents launchd from SIGTERMing the whole group when the oneshot exits. Ship and stop here if the reboot test passes.
- Attempt 2 (only if attempt 1 fails): hidden `tytus tunnel-supervise` subcommand that owns the boringtun child via `setsid`, written as a `KeepAlive=true` plist. Sudoers stays scoped to `tunnel-up *` and `tunnel-down *` only — supervisor MUST NOT widen the sudoers surface.
- `tytus autostart status` and `tytus doctor` report which model is in use and whether the supervisor is alive.

**Files:**
- `cli/src/main.rs` — `cmd_autostart` at line 3227; plist template ~3249+; `cmd_doctor` reporting. If attempt 2: new `Commands::TunnelSupervise` dispatcher arm.
- `services/tytus-cli/CLAUDE.md` — document the LaunchAgent topology change.

**Tests:**
- Physical reboot; time-to-green from login.
- `launchctl bootout gui/$UID/com.traylinx.tytus && launchctl bootstrap …` — pidfile + utun survive.
- `scripts/e2e-flows.sh --pod NN` 30 min post-reboot, no manual `connect`.
- Supervisor variant only: `kill -9 $(cat /tmp/tytus/tunnel-NN.pid)` → tunnel back within 10s.
- `cargo test -p atomek-tunnel`; `cargo clippy --workspace --all-targets` clean.

### Phase 3: Code-sign + notarize CLI, MCP, tray, Tytus.app

**Goal:** Stop the unsigned-binary keychain-ACL invalidation. Item 5.

**Hard preconditions (block phase start until all three met — not sprint tasks):**
- Apple Developer Program membership active on the team account ($99/yr, 24–48h enrollment). Sprint cannot start P3 until this exists.
- Developer ID Application certificate downloaded into the release machine's login keychain.
- App-specific password for `notarytool` stored via `xcrun notarytool store-credentials`.

**Criteria:**
- `scripts/release-macos.sh` performs **per-binary** `codesign --options runtime --timestamp -s "Developer ID Application: …"` against `target/release/tytus`, `tytus-mcp`, `tytus-tray`, then the `Tytus.app` bundle. No `--deep` (Apple TN3127 — `--deep` skips nested binaries the hardened runtime refuses to load).
- DMG submitted via `xcrun notarytool submit --wait` and stapled (`xcrun stapler staple`); `xcrun stapler validate` passes offline.
- `spctl --assess --type execute` accepted for each binary; `spctl --assess --type install` accepted for the DMG.
- **Keychain migration — empirical, not optimistic.** macOS keychain ACLs bind items to the signing identity of the requesting app; unsigned → signed is an identity change, so the existing ACL for service `com.traylinx.atomek` (verified: `services/tytus-cli/CLAUDE.md` documents this service name as stable) may not match the newly-signed `tytus` binary and users may face a "keychain denied" error rather than a one-time approve prompt. Deliverable: empirical test matrix below, and a documented fallback UX — if the signed binary fails to read the unsigned-owner keychain entry, `tytus login` and `tytus doctor` both detect this and emit `"Keychain entry locked by previous install — run: tytus login"` with a single-command re-auth path that deletes the stale entry before writing the new one.
- `install.sh` keeps `xattr -d com.apple.quarantine` as belt-and-suspenders with a comment explaining it's redundant once notarized but harmless.
- `docs/RELEASE.md` extended with sign+notarize+staple steps; `docs/SECURITY-AUDIT.md` appended with the signing-identity invariant.

**Files:**
- `services/tytus-cli/scripts/release-macos.sh` — new.
- `services/tytus-cli/install.sh` — comment update only.
- `cli/src/main.rs` — verify `Tytus.app` `Info.plist` written by `cmd_tray` has `CFBundleIdentifier=com.traylinx.tytus` for identity match.
- `cli/src/main.rs` — `cmd_login` + `cmd_doctor`: detect keychain-ACL-denied error code, emit the fallback UX.
- `auth/src/keychain.rs` — surface the platform error distinctly so the fallback UX can be triggered without string-matching.
- `tray/src/main.rs` — confirm bundle id matches plist.
- `services/tytus-cli/docs/RELEASE.md`, `docs/RELEASE-NOTES-v0.4.1.md`, `docs/SECURITY-AUDIT.md`.

**Tests:**
- `codesign --verify --strict --verbose=2` valid per-binary.
- `spctl --assess` accepted per binary and for DMG; `xcrun stapler validate` green.
- **Migration matrix on fresh macOS VM, recorded in `docs/RELEASE-NOTES-v0.4.1.md` with actual behavior:**
  1. Install unsigned 0.4.0 → `tytus login` (writes keychain entry under unsigned identity) → upgrade to signed 0.4.1 → run `tytus status`. Record: one prompt / silent re-denial / hard failure.
  2. Signed 0.4.1 fresh install → `tytus login` → `tytus daemon run` → verify daemon reads the signed-owner keychain entry with no prompt.
  3. Unsigned-to-signed where outcome (1) was "failure": verify `tytus doctor` surfaces the fallback UX copy and `tytus login` completes cleanly after it.

### Phase 4: Structured event log on `/tmp/tytus/events.jsonl`

**Goal:** One JSON-line event stream so `tytus doctor --events 1h` answers "what happened" without grepping six logs. Item 8.

**Criteria:**
- `core/src/events.rs` (new): writes `{ts, event, pod_id?, outcome, details}` to `/tmp/tytus/events.jsonl`; rotates at 10 MB; `0o600` perms; per-line atomic write.
- Call sites wired:
  - connect/disconnect/revoke — `cli/src/main.rs` cmd_connect / cmd_disconnect / cmd_revoke handlers.
  - Token refresh outcome — `ensure_token` at `cli/src/main.rs:6052` (verified location).
  - Forwarder start/stop — `cli/src/main.rs` cmd_ui.
  - Pairing auto-approve — `tray/src/web_server.rs`.
  - Upstream health change — `tray/src/gateway_probe.rs`.
  - Tunnel health transitions — `tunnel/src/monitor.rs` (existing TCP probe, still the only signal available until the deferred item 11 lands).
- `tytus doctor --events 1h` renders human timeline; `--events 1h --json` emits raw lines.
- No tokens, keychain values, WireGuard keys in payloads. Droplet IPs + pod octets are fine (already public via `tytus status`).
- No `tracing` subscriber — direct writer only.

**Files:**
- `core/src/events.rs` — new writer + rotation + schema.
- `core/src/lib.rs` — re-export.
- `cli/src/main.rs` — emit at cmd_connect, cmd_disconnect, cmd_revoke, cmd_ui, `ensure_token` (line 6052).
- `auth/src/login.rs` + `auth/src/device_auth.rs` — emit at refresh-network boundary.
- `pods/src/request.rs` + `pods/src/status.rs` — emit allocation + agent-health events.
- `tunnel/src/monitor.rs` — emit health-flap events.
- `tray/src/web_server.rs` + `tray/src/gateway_probe.rs`.
- `cli/src/main.rs` — `cmd_doctor` `--events` flag + timeline renderer.
- `llm-docs.md` — event log location + schema.

**Tests:**
- Unit (`core/src/events.rs`): tempfile round-trip; rotation threshold; 16-thread concurrent write → 16 valid JSON lines, zero partials.
- Integration: connect → chat → disconnect cycle produces expected events in order; `grep -E 'sk-|refresh_token|PrivateKey'` on the log returns zero matches.
- `tytus doctor --events 1h` renders without panic on rotated file.
- `cargo clippy --workspace --all-targets` clean; `cargo audit` clean.

### Phase 5: Actionable `ConnectError` + doctor diagnosis

**Goal:** Replace generic strings in `tytus connect` / `tytus status` with a typed enum that names the next step. Item 10. Depends on Phase 4.

**Criteria:**
- `core/src/error.rs`: `ConnectError` enum — `PROVIDER_DOWN`, `POD_ALLOCATING`, `AGENT_UNHEALTHY`, `TUNNEL_FLAPPING`, `AUTH_EXPIRED`. Each variant has `display()` + `next_step()`.
- `pods/src/request.rs` and `pods/src/status.rs` map upstream HTTP + agent-probe outcomes to variants; `auth/src/device_auth.rs` surfaces `AUTH_EXPIRED` distinctly.
- `tytus connect` + `tytus status` (handlers in `cli/src/main.rs`) render `<message>\nNext: <step>` (human) or `{error, next_step}` (`--json`).
- `tytus doctor` cross-references the last hour of `events.jsonl` (Phase 4) plus a live probe to confirm/refute each variant.
- Copy stability guarded by snapshot test per variant.

**Files:**
- `core/src/error.rs` — `ConnectError` + `display()` + `next_step()`.
- `pods/src/request.rs` + `pods/src/status.rs` — variant mapping.
- `auth/src/device_auth.rs` — `AUTH_EXPIRED` mapping.
- `cli/src/main.rs` — cmd_connect, cmd_status, cmd_doctor consume + render.
- `llm-docs.md` — variant + next-step copy.

**Tests:**
- Unit: each variant renders non-empty message + step; snapshot test per variant.
- Integration: provider 502 → `PROVIDER_DOWN` event written, `tytus doctor` confirms via event log + live probe.
- Integration: `docker kill tytus-NN` mid-session → `tytus status` → `AGENT_UNHEALTHY`.
- Integration: corrupt refresh token in keychain → next `ensure_token` tick → `AUTH_EXPIRED` in `tytus status`.
- `cargo clippy --workspace --all-targets` clean.

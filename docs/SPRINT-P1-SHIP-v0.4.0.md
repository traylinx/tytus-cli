# SPRINT-2026-04-19-v040-ship

## Origin: user request

Ship v0.4.0 as one coordinated wave across `wannolot-infrastructure` (4 commits queued on `main` awaiting promotion to `production` — verified via `git log production..main --oneline`: `153e216` hermes gateway+dashboard, `4c0021d` hermes API_SERVER_KEY auto-derive, `a3d4021` switchailocal v0.4.0→v0.4.1, `5cd43f5` switchailocal v0.3.1→v0.4.0), `tytus-cli` v0.4.0 (manual `cargo build --release` + `gh release create`, no CI), doc reconciliation (`llm-docs.md` is stale — grep confirms no mention of port `9119`, no `is_logged_in`, no `dashboard` path, and hosted `.agents/skills/tytus/SKILL.md` mirrors it via the `raw.githubusercontent.com` URL served by `tytus bootstrap-prompt`), E2E regression gate against *both* Hermes and NemoClaw pods (switchailocal bump touches both LLM paths), and a cross-repo coordination issue in `traylinx/wannolot-provider` for item 9 telemetry schema. Spike-first on Hermes dashboard cold-boot before any Dockerfile change — scope is `/opt/hermes/web` base-image content only (port+host already pinned to `0.0.0.0:9119` by `hermes/entrypoint.sh` in `153e216`). Rollback must account for `:latest`-only infra images — fleet-wide pre-tag before promotion, because `docker compose` reconciles against local tags, not file-stored digests.

Repo layout: `wannolot-infrastructure` lives at `services/wannolot-infrastructure/` (sibling to `services/tytus-cli/`); all `git -C` / `cd` references below assume this.

## Phases

### Phase 1: Fleet-wide preflight retag + Hermes dashboard cold-boot spike

**Goal:** Make rollback physically possible by tagging `:latest` → `:pre-v0.4.0` on *every* production droplet before any promotion, and close the one remaining unknown on `153e216`: does `/opt/hermes/web` ship inside `nousresearch/hermes-agent:latest`, and does `hermes dashboard --host 0.0.0.0 --port 9119` serve index.html on a cold container with no prior session. Dashboard host+port are already fixed by `hermes/entrypoint.sh` in `153e216` — spike is scoped to base-image content + cold-start behavior, not discovery.

**Criteria:**
- Fleet enumeration source of truth: Scalesys `droplets` table (`SELECT droplet_id, ip, octet FROM droplets WHERE revoked_at IS NULL`) — not ansible guesses, not a static list.
- Per-droplet retag loop executes (SSH or Scalesys exec): `for img in tytus-hermes tytus-nemoclaw switchailocal; do id=$(docker image inspect $img:latest --format '{{.Id}}' 2>/dev/null) || continue; docker tag $img:latest $img:pre-v0.4.0 && echo "$img:pre-v0.4.0 = $id" >> /tmp/preflight-v040.log; done`. Results aggregated into `docs/rollback/v0.4.0-preflight.md` keyed by `droplet_id`. Missing `:latest` for an image = record and skip (that image class isn't running on that droplet — expected).
- Digest file alone is NOT sufficient — the `:pre-v0.4.0` *tag* must physically exist on each droplet, because `docker compose up` reconciles against local tags only.
- Abort criterion: if any droplet fails the retag (ssh unreachable, docker daemon down, image present but `docker tag` errors), Phase 2 does not start until the droplet is either fixed or explicitly quarantined out of the rollout with owner + reason logged.
- Spike command (run on scratch droplet or local docker, NEVER inside a live production container):
  ```
  docker run --rm --entrypoint bash nousresearch/hermes-agent:latest \
    -c 'ls -la /opt/hermes/web 2>&1 | head -20; \
        source /opt/hermes/.venv/bin/activate 2>/dev/null && \
        (hermes dashboard --host 0.0.0.0 --port 9119 --insecure --no-open &) && \
        sleep 5 && curl -sS -o /tmp/out.html -w "HTTP=%{http_code} BYTES=%{size_download}\n" http://127.0.0.1:9119/ && \
        head -5 /tmp/out.html'
  ```
- Spike verdict written to `docs/spikes/hermes-dashboard-coldboot.md`: one of `ship-as-is` (base image fine) / `patch-in-this-sprint` (blocker — add to Phase 2 scope) / `defer-to-follow-up` (cosmetic gap, ship anyway).

**Files:**
- `services/wannolot-infrastructure/docs/rollback/v0.4.0-preflight.md` (new) — per-droplet retag log.
- `services/wannolot-infrastructure/docs/spikes/hermes-dashboard-coldboot.md` (new) — base-image investigation + verdict.

**Tests:**
- For every `droplet_id` in Scalesys: `ssh <droplet> "docker image inspect tytus-hermes:pre-v0.4.0 tytus-nemoclaw:pre-v0.4.0 switchailocal:pre-v0.4.0 2>&1 | grep -c Id"` ≥ count of images actually present pre-promotion. 100% coverage required.
- Spike cold-boot returns `HTTP=200` with `BYTES>0` AND index.html head contains a recognizable HTML tag, OR verdict is `patch-in-this-sprint` with owner.

### Phase 2: Infrastructure promotion with canary + rollback rehearsal + fleet-wide health gate

**Goal:** Promote the 4 verified commits `main` → `production`, roll out, rehearse a real rollback on canary (not mental), then gate on *fleet-wide* health — canary-green ≠ fleet-green.

**Criteria:**
- Promotion diff check: `git -C services/wannolot-infrastructure log production..main --oneline` matches exactly the 4 SHAs above with no drift. Fast-forward push `main` → `production`. If non-FF: stop, investigate.
- Pick canary droplet from Scalesys (lowest-traffic by `used_pods` ascending). Let its rolling update complete. Verify: `docker compose ps --format json | jq '.[] | select(.Health != "healthy")'` returns empty; `curl -fsS http://<canary>:18080/v1/models` returns expected model set; for any Hermes pod on canary `curl -fsS http://10.{octet}.{pod}.1:8642/health` AND `http://.../ (port 9119)`; for any NemoClaw pod `curl -fsS http://10.{octet}.{pod}.1:3000/healthz`.
- **Live rollback rehearsal on canary:** while canary is green, retag back: `docker tag tytus-hermes:pre-v0.4.0 tytus-hermes:latest && docker tag switchailocal:pre-v0.4.0 switchailocal:latest && docker compose up -d switchailocal hermes-wannolot-<POD>`. Confirm downgrade took effect (e.g. switchailocal version banner in `/v1/models` response metadata reverts). Then retag forward from digest captured in Phase 1: `docker tag <new-digest-sha> tytus-hermes:latest` etc., restart, confirm recovery. Timing + command log appended to sprint journal.
- Fleet health gate: after canary green + drill complete, allow remaining droplets to roll. Poll Scalesys `/droplets` (or aggregate health endpoint) every 30s until 100% report healthy OR 15-min timeout triggers rollback.
- 5-minute post-promotion log scan per droplet class: aggregated `docker logs` for `switchailocal`, any `tytus-hermes-*`, any `tytus-nemoclaw-*` shows zero new `ERROR` / `FATAL` / stack-trace lines that weren't present pre-promotion.

**Files:**
- No source changes. Branch promotion + rollout only.
- Append to `$MAKAKOO_HOME/data/Brain/journals/2026_04_19.md`: canary `droplet_id`, rollback drill commands + timing, fleet gate wall-clock.

**Tests:**
- `git -C services/wannolot-infrastructure log production..main --oneline` is empty post-push.
- Canary: all curl + `docker compose ps` assertions pass.
- Drill: canary demonstrably served `:pre-v0.4.0` image for ≥1 `/v1/models` request, then recovered to v0.4.1 image.
- Fleet gate: Scalesys shows 100% healthy; zero new ERROR lines in aggregated log scan.

### Phase 3: Docs land first, then tytus-cli v0.4.0 release (gated on docs)

**Goal:** Land doc updates *before* the release commit because `cli/src/main.rs:5646` and `mcp/src/tools.rs:26` both `include_str!("../../llm-docs.md")` — doc drift at release-cut becomes permanently shipped in the binary until the next release. Release is gated: no doc merge → no version bump → no tag.

**Criteria:**
- Doc PR (commit A, merged first):
  - `llm-docs.md` adds: Hermes dashboard on `:9119` (per Phase 1 verdict), forwarder path-based multiplex (`/v1/*` + `/api/jobs*` → gateway `:8642`, else → dashboard `:9119`, per shipped commit `0fc13f4`), `is_logged_in` AT-only fallback (per shipped commit `b9d44df`), overlay formats (`config.user.json` *and* `config.user.yaml`, with precedence order and which one wins on conflict).
  - `.agents/skills/tytus/SKILL.md` updated in lockstep — same commit — so the `raw.githubusercontent.com` URL emitted by `tytus bootstrap-prompt` (verified present in `llm-docs.md:SKILL URL section`) serves identical content.
  - Verification grep must pass on the PR branch: `grep -nE "9119|multiplex|is_logged_in|config\.user\.yaml" llm-docs.md` returns ≥4 matches (currently returns 0 for `9119`, `multiplex`, `is_logged_in`, `config.user.yaml` — confirmed via grep against current `main`).
- `CHANGELOG.md` (new file — does not exist at `services/tytus-cli/CHANGELOG.md` today, confirmed) created in commit A, body enumerates: infra SHAs shipped (`153e216`, `4c0021d`, `a3d4021`, `5cd43f5`), CLI changes since v0.3.0 from `git log v0.3.0..HEAD --oneline`, doc updates in this PR, known gaps.
- Release commit (commit B, only after commit A is on `main` with green CI-equivalents):
  - Bump `services/tytus-cli/Cargo.toml` `[workspace.package] version = "0.3.0"` → `"0.4.0"` (confirmed current value at line 14).
  - Regenerate `Cargo.lock` via `cargo check --workspace` — commit both.
  - Gates: `cargo test --workspace --all-targets` passes; `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo audit` clean.
- Publish: `cargo build --release` produces `target/release/tytus` + `target/release/tytus-mcp`; `git tag v0.4.0 <commit-B-sha> && git push origin v0.4.0`; `gh release create v0.4.0 --notes-file CHANGELOG.md target/release/tytus target/release/tytus-mcp`.
- Install.sh smoke: on a scratch macOS user with no prior state, `curl -fsSL <install.sh URL> | bash` pulls v0.4.0; `tytus --version` prints `0.4.0`; `tytus llm-docs | grep -c 9119` ≥ 1 (proves new doc is bundled in binary).

**Files:**
- `services/tytus-cli/llm-docs.md` — 4 new/updated sections.
- `services/tytus-cli/.agents/skills/tytus/SKILL.md` — mirror.
- `services/tytus-cli/CHANGELOG.md` (new).
- `services/tytus-cli/Cargo.toml` — version bump only (commit B).
- `services/tytus-cli/Cargo.lock` — regenerated (commit B).

**Tests:**
- Commit A: grep invariants on `llm-docs.md` (≥4 matches listed above); `.agents/skills/tytus/SKILL.md` byte-diff against `llm-docs.md` shows only intended deltas.
- Commit B: `cargo test` / `cargo clippy -D warnings` / `cargo audit` all clean.
- Built binary: `./target/release/tytus --version` → `tytus 0.4.0`; `./target/release/tytus llm-docs | grep -c "9119\|is_logged_in"` ≥ 2.
- `gh release view v0.4.0` shows both binaries attached; release-notes body equals `CHANGELOG.md` head section.

### Phase 4: E2E regression gate (Hermes + NemoClaw) + cross-repo coordination

**Goal:** Prove the shipped wave against freshly allocated pods of *both* agent types — switchailocal v0.3.1→v0.4.1 sits on the LLM path for both, Hermes-only testing would hide NemoClaw regressions — and open the item 9 telemetry schema issue in `traylinx/wannolot-provider`.

**Criteria:**
- **Hermes regression:** `tytus logout && tytus login && tytus connect --agent hermes`; `tytus status --json | jq '.pods[0].units_consumed'` = 2; tunnel up; `curl -fsS http://10.42.42.1:8642/health` → 200; `curl -fsS http://10.42.42.1:9119/` returns dashboard HTML through forwarder; `scripts/e2e-flows.sh` (confirmed present in `tytus-cli` repo per commit `2b86077`) exits 0; `tytus chat` with `--model ail-compound` round-trips ≥1 completion.
- **NemoClaw regression** (required — switchailocal bump affects it too): after `tytus revoke <hermes-pod>`, `tytus connect --agent nemoclaw`; units_consumed = 1; `curl -fsS http://10.42.42.1:3000/healthz` → 200; `tytus chat` round-trips ≥1 completion; `scripts/e2e-flows.sh` (nemoclaw path) exits 0.
- Teardown invariant for both: `tytus revoke` succeeds; Scalesys `clients` row has `revoked_at IS NOT NULL`; `tytus status --json | jq '.pods'` = `[]`; units restored to plan cap.
- **Stable-key invariant across revoke/reallocate cycle**: `OPENAI_API_KEY_BEFORE=$(tytus env --export | grep OPENAI_API_KEY)`, then revoke+reallocate, then compare to `OPENAI_API_KEY_AFTER`. Must be byte-identical — confirms `user_stable_keys` table wasn't disturbed by v0.4.0.
- Cross-repo coordination: `gh issue create -R traylinx/wannolot-provider --title "v0.4.0 follow-up: telemetry schema for item 9" --body "<link to this sprint doc + required fields + owner + target sprint>"`. Issue URL captured in Brain journal.
- Brain journal `2026_04_19.md` final entry: infra SHAs, CLI release tag + URL, E2E outcomes for both agents, spike verdict, rollback-drill timing, cross-repo issue URL.

**Files:**
- No source changes.
- `$MAKAKOO_HOME/data/Brain/journals/2026_04_19.md` — sprint summary.

**Tests:**
- `scripts/e2e-flows.sh` exits 0 against Hermes pod.
- `scripts/e2e-flows.sh` exits 0 against NemoClaw pod.
- `tytus chat` smoke completes for both agents.
- `tytus status --json` post-revoke for each: `.pods == []` and units restored.
- `$OPENAI_API_KEY` byte-identical pre/post revoke-reallocate cycle (stable-key invariant).
- `gh issue view <n> -R traylinx/wannolot-provider` returns the created issue.

# Round 1 — drafter
# SPRINT-ROUTE-AWARE-CLI-POD-SELECTORS

## Origin: user request

Fix Tytus CLI/provider pod addressing so duplicate `pod_id` agent routes are safely targetable by `route_id` or `display_name`. Operator (Sebastian) has three agents — Lisa, Claus, Hermie — collapsed under `pod_id=01`, so `tytus exec --pod 01` reaches only one OpenClaw workspace and can silently target the wrong agent. Selectors must resolve by exact `route_id`, then normalized `display_name`, then `pod_id` only when unique; ambiguous `pod_id` must fail closed on both CLI and Provider.

## Phases

### Phase 1: Route-aware pod selector resolution in CLI

**Goal:** Add a single resolver in the CLI that maps a `--pod` argument to a unique `PodEntry` by precedence (exact `route_id` → normalized `display_name` → unique `pod_id`), failing closed with the ambiguity message when a bare `pod_id` matches more than one route.

**Criteria:**
- New resolver function (e.g. `resolve_pod_selector`) accepts the raw `--pod` string and the list of `PodEntry` from `state.rs` and returns the matched entry or a typed error.
- Match order is exact `route_id` first, then case-insensitive/trimmed `display_name`, then `pod_id` only when exactly one entry carries it.
- A `pod_id` matching multiple entries returns an `Ambiguous` error carrying every candidate (display_name, pod_id, route_id, agent label).
- The error renders exactly the brief's ambiguity text: `Ambiguous pod selector "01" matched 3 pods. Use display name or route_id:` followed by one `- <name>: pod_id=.. route_id=.. agent=..` line per candidate.
- Users with a unique `pod_id` keep current behavior (bare `pod_id` still resolves).

**Files:**
- `services/tytus-cli/cli/src/state.rs` — read-only use of existing `PodEntry.route_id` and `PodEntry.display_name`; no schema change.
- `services/tytus-cli/cli/src/main.rs` — add `resolve_pod_selector` helper and the `Ambiguous`/`NotFound` error type plus its `Display` impl.

**Tests:**
- `cargo test -p atomek-cli pod_selector` — `--pod Lisa` resolves to `route_id=0e0ah755r3`; `--pod 0e0ah755r3` resolves to the same entry.
- Ambiguity test: `--pod 01` over the three duplicate entries returns `Ambiguous` and the rendered string lists all three candidates in order.
- Backward-compat test: a single-entry `pod_id=02` fixture resolves `--pod 02` successfully.
- Normalization test: `--pod lisa` and `--pod " Lisa "` resolve via `display_name`.

### Phase 2: Wire route-aware selector through CLI commands and Provider client

**Goal:** Route every command that currently sends a raw `pod_id` (`exec`, `logs`, `restart`, and the transfer commands `ls`/`push`/`pull`/`rm`) through the Phase 1 resolver and send both `pod_id` and `route_id` to the Provider.

**Criteria:**
- `cmd_exec`, `cmd_logs`, `cmd_restart` call `resolve_pod_selector` and pass the resolved `route_id` to the Provider client.
- `resolve_pod` in `transfer.rs` / `cmd_transfer.rs` returns the resolved entry (not a raw string) so `ls`/`push`/`pull`/`rm` reuse route-aware exec internally.
- Provider client in `agent.rs` adds an optional `route_id` field to the exec request body/query; when `Some`, it is included, when `None`, the body is byte-identical to today (backward compatible).
- `tytus exec --pod Lisa` emits Provider body `{"pod_id":"01","route_id":"0e0ah755r3","command":"...","timeout":10}`.
- No changes to `tray/web/os/*` or `tray/src/desktop_llm.rs`; ambiguity surfaces as the Phase 1 error before any network call.

**Files:**
- `services/tytus-cli/cli/src/main.rs` — `cmd_exec`, `cmd_logs`, `cmd_restart` resolve then pass `route_id`.
- `services/tytus-cli/cli/src/transfer.rs` and `services/tytus-cli/cli/src/cmd_transfer.rs` — `resolve_pod` returns resolved `PodEntry`; transfer commands thread `route_id` through internal exec.
- `services/tytus-cli/pods/src/agent.rs` — Provider client gains optional `route_id` on the exec request.

**Tests:**
- `cargo test -p atomek-pods` — exec request serialization includes `route_id` when set and omits it (identical bytes to baseline) when `None`.
- CLI test: `exec --pod Lisa` produces the exact expected JSON body including `route_id=0e0ah755r3`.
- CLI test: a transfer command (`ls --pod Claus`) threads `route_id=eb2qvn3t4s` into its internal exec call.

### Phase 3: Provider ambiguity guard for DAM proxy calls

**Goal:** Make the Provider fail closed when a DAM proxy call carries an ambiguous `pod_id` (multiple matching routes) without a `route_id`, returning 409 without contacting any droplet/DAM, while route-disambiguated calls reach the selected droplet.

**Criteria:**
- In `callDAM(req, podId, ...)`, when `findPod(...)` / `podSelectorFromRequest(req)` resolves more than one route for the given `pod_id` and no `route_id` is supplied, return HTTP 409 with a useful message instead of silently selecting the first.
- The 409 path does not call the droplet/DAM (no proxied side effects).
- When a `route_id` (or unambiguous `pod_id`) is supplied, the existing selected-droplet path is unchanged.
- No secrets from `state.json` or Provider responses are included in the error payload.

**Files:**
- `services/wannolot-provider/src/routes/pod.js` — add the ambiguity guard in/around `callDAM`, reusing existing `podSelectorFromRequest` and `findPod`.

**Tests:**
- Provider Jest: duplicate `pod_id=01` without `route_id` returns 409 and asserts the DAM/droplet call mock was not invoked.
- Provider Jest: duplicate `pod_id=01` with `route_id=0e0ah755r3` calls the selected droplet/DAM exactly once.
- Provider Jest: unique `pod_id=02` without `route_id` still reaches its droplet (backward compatibility).
- Provider Jest: 409 error body contains the disambiguation hint and no fields sourced from `state.json`.

### Phase 4: Verification and diff hygiene

**Goal:** Run the targeted test suites end to end and confirm the working tree contains only the intended files, with no destructive Tytus commands used during verification.

**Criteria:**
- All targeted suites pass: `cargo test -p atomek-pods`, `cargo test -p atomek-cli pod_selector`, and the Provider Jest pod-route tests.
- A non-destructive smoke check confirms `tytus exec --pod Lisa` and `--pod Claus` reach distinct agents and `--pod 01` errors with the ambiguity message (no `revoke`/`logout`/`disconnect` invoked).
- `git diff --name-only` lists only the five intended source files plus their test files; `tray/web/os/*` and `tray/src/desktop_llm.rs` are untouched.

**Files:**
- No production source changes in this phase; verification only across the files touched in Phases 1–3.
- New/updated test fixtures co-located with `atomek-cli`, `atomek-pods`, and `wannolot-provider` test suites.

**Tests:**
- `cargo test -p atomek-pods` passes.
- `cargo test -p atomek-cli pod_selector` passes.
- Provider Jest pod-route suite passes.
- `git diff --name-only` reviewed: only intended files present, confirmed against the constraint list.

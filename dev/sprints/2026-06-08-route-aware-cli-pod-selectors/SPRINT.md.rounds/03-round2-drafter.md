# Round 2 — drafter
# SPRINT-ROUTE-AWARE-CLI-POD-SELECTORS

## Origin: user request

Fix Tytus CLI/provider pod addressing so duplicate `pod_id` agent routes are safely targetable by `route_id` or `display_name`. Operator (Sebastian) has three agents — Lisa, Claus, Hermie — reportedly collapsed under `pod_id=01`, so `tytus exec --pod 01` reaches only one OpenClaw workspace and can silently target the wrong agent. Selectors must resolve by exact `route_id`, then normalized `display_name`, then `pod_id` only when unique; any ambiguous selector must fail closed on both CLI and Provider. Because `CLAUDE.md` documents a one-container-per-pod model, the multi-route-per-pod assumption is verified before any code is written.

## Phases

### Phase 1: Confirm pod-routing data model and reconcile with one-container-per-pod assumption

**Goal:** Before touching code, confirm whether `PodEntry` already carries `route_id` and `display_name` and whether `state.json` actually stores multiple routes under one `pod_id`; reconcile this with the documented "one agent container per pod" model and decide the read-only path versus the schema-change path. This phase gates Phases 2–4.

**Criteria:**
- Audit the `PodEntry` definition and record, yes or no, whether `route_id` and `display_name` fields exist today.
- Audit a real `state.json` (or its serializer) and record whether multiple entries can share `pod_id=01`. If the model is strictly one-container-per-pod, capture that the multi-route shape is a NEW schema requirement and move the field additions into Phase 2 scope rather than treating them as read-only.
- Confirm `desktop_llm.rs` and `tray/web/os/*` reach the model gateway at `10.42.42.1:18080` and do not select by `pod_id`. If any of them do route by `pod_id`, add them to the wiring scope in Phase 3 and remove them from the "untouched" constraint.
- Produce a one-paragraph reconciliation note recording the verified data model and the chosen path (read-only versus schema-add), referenced by every later phase.

**Files:**
- `services/tytus-cli/cli/src/state.rs` — audit `PodEntry` for `route_id` / `display_name` presence.
- `services/wannolot-provider` `state.json` (or its serializer) — audit for multi-route-per-pod shape.
- `services/tytus-cli/cli/src/desktop_llm.rs` and `tray/web/os/*` — audit for any `pod_id`-based selection.

**Tests:**
- Reconciliation note committed stating: route_id/display_name present (yes/no), multi-route-per-pod present (yes/no), schema change required (yes/no).
- If a schema change is required, a recorded checkbox confirming Phase 2 adds the fields plus a serde default so existing single-route `state.json` still deserializes.
- A confirmation line that `desktop_llm.rs` uses gateway `10.42.42.1:18080` and not pod selection — exclusion justified — or, if it does select by `pod_id`, an explicit escalation into Phase 3 scope.

### Phase 2: Route-aware pod selector resolution in the CLI

**Goal:** Add a single `resolve_pod_selector` in the CLI that maps a `--pod` argument to a unique `PodEntry` by precedence (exact `route_id` → normalized `display_name` → unique `pod_id`), failing closed with the ambiguity message whenever a bare `pod_id` OR a normalized `display_name` matches more than one route. If Phase 1 found the fields absent, add them here.

**Criteria:**
- Resolver accepts the raw `--pod` string and the list of `PodEntry` and returns the matched entry or a typed error (`Ambiguous` / `NotFound`).
- Precedence is exact `route_id` first, then trimmed and case-insensitive `display_name`, then `pod_id` only when exactly one entry carries it.
- A `pod_id` matching multiple entries returns `Ambiguous` carrying every candidate (display_name, pod_id, route_id, agent label).
- A normalized `display_name` matching multiple entries ALSO returns `Ambiguous` and never silently picks the first — closing the same mistarget bug for duplicate display names.
- The error renders the brief text exactly: `Ambiguous pod selector "01" matched 3 pods. Use display name or route_id:` followed by one `- <name>: pod_id=<..> route_id=<..> agent=<..>` line per candidate, in input order.
- A single unique `pod_id` keeps current behavior (bare `pod_id` still resolves).
- If Phase 1 determined the fields are absent, add `route_id` and `display_name` to `PodEntry` with `#[serde(default)]` so existing `state.json` deserializes unchanged.

**Files:**
- `services/tytus-cli/cli/src/state.rs` — fields used read-only if present; otherwise add `route_id` / `display_name` with serde defaults.
- `services/tytus-cli/cli/src/main.rs` — add `resolve_pod_selector`, the `Ambiguous` / `NotFound` error type, and its `Display` impl.

**Tests:**
- Fixture set (synthetic, not live): three entries with `pod_id=01` and `route_id` values `route-lisa`, `route-claus`, `route-hermie`, with `display_name` Lisa, Claus, Hermie respectively.
- `--pod Lisa` resolves to `route_id=route-lisa`; `--pod route-lisa` resolves to the same entry.
- Ambiguity test: `--pod 01` over the three duplicate entries returns `Ambiguous` and the rendered string lists all three candidates in input order.
- Duplicate-display_name test: two fixture entries both `display_name="Lisa"` → `--pod Lisa` returns `Ambiguous` and not a first-match.
- Backward-compat test: a single-entry `pod_id=02` fixture resolves `--pod 02` successfully.
- Normalization test: `--pod lisa` and `--pod " Lisa "` resolve via `display_name`.

### Phase 3: Wire the resolver through CLI commands and the Provider client

**Goal:** Route every command that currently sends a raw `pod_id` (`exec`, `logs`, `restart`, and the transfer commands `ls`/`push`/`pull`/`rm`) through the Phase 2 resolver and send both `pod_id` and `route_id` to the Provider.

**Criteria:**
- `cmd_exec`, `cmd_logs`, `cmd_restart` call `resolve_pod_selector` and pass the resolved `route_id` to the Provider client.
- `resolve_pod` in `transfer.rs` / `cmd_transfer.rs` returns the resolved `PodEntry` (not a raw string) so `ls`/`push`/`pull`/`rm` reuse route-aware exec internally.
- The Provider client in `agent.rs` adds an optional `route_id` field to the exec request body; when `Some` it is included, and when `None` the body is byte-identical to today (backward compatible).
- `tytus exec --pod Lisa` emits the Provider body `{"pod_id":"01","route_id":"route-lisa","command":"whoami","timeout":10}`.
- Ambiguity surfaces as the Phase 2 error before any network call; `tray/web/os/*` and `tray/src/desktop_llm.rs` stay untouched unless Phase 1 escalated them.

**Files:**
- `services/tytus-cli/cli/src/main.rs` — `cmd_exec`, `cmd_logs`, `cmd_restart` resolve then pass `route_id`.
- `services/tytus-cli/cli/src/transfer.rs` and `services/tytus-cli/cli/src/cmd_transfer.rs` — `resolve_pod` returns the resolved `PodEntry`; transfer commands thread `route_id` through internal exec.
- `services/tytus-cli/pods/src/agent.rs` — Provider client gains optional `route_id` on the exec request.

**Tests:**
- `cargo test -p atomek-pods` — exec request serialization includes `route_id` when set and omits it (identical bytes to baseline) when `None`.
- CLI test: `exec --pod Lisa` produces the exact expected JSON body, including `route_id=route-lisa`, `command=whoami`, `timeout=10`.
- CLI test: a transfer command (`ls --pod Claus`) threads `route_id=route-claus` into its internal exec call.

### Phase 4: Provider ambiguity guard for DAM proxy calls

**Goal:** Make the Provider fail closed (HTTP 409, no droplet/DAM contact) when a DAM proxy call carries an ambiguous selector — `pod_id` OR `display_name` matching multiple routes — without a `route_id`, while route-disambiguated calls reach the selected droplet. If `findPod` is first-match-only today, add the multiplicity detection that makes ambiguity observable.

**Criteria:**
- In `callDAM`, when `findPod` / `podSelectorFromRequest(req)` resolves more than one route for the given selector and no `route_id` is supplied, return HTTP 409 with a disambiguation hint instead of silently selecting the first.
- If `findPod` currently returns only the first match, add a "find all matches" pass so multiplicity is detectable before the guard fires; the brief assumed multi-match support, which Phase 1 must confirm.
- The 409 path does not call the droplet/DAM (no proxied side effects).
- When a `route_id` (or an unambiguous selector) is supplied, the existing selected-droplet path is unchanged.
- No secrets from `state.json` or Provider responses are included in the error payload.

**Files:**
- `services/wannolot-provider/src/routes/pod.js` — add the ambiguity guard in and around `callDAM`, extending `podSelectorFromRequest` / `findPod` with multiplicity detection if needed.

**Tests:**
- Provider Jest: duplicate `pod_id=01` without `route_id` returns 409 and asserts the DAM/droplet call mock was not invoked.
- Provider Jest: duplicate `pod_id=01` with `route_id=route-lisa` calls the selected droplet/DAM exactly once.
- Provider Jest: duplicate `display_name="Lisa"` without `route_id` returns 409 — display_name ambiguity is guarded as well as pod_id.
- Provider Jest: unique `pod_id=02` without `route_id` still reaches its droplet (backward compatibility).
- Provider Jest: the 409 error body contains the disambiguation hint and no fields sourced from `state.json`.

### Phase 5: Verification and diff hygiene

**Goal:** Run the targeted test suites end to end and confirm the working tree contains only the intended files, with no destructive Tytus commands used during verification.

**Criteria:**
- All targeted suites pass: `cargo test -p atomek-pods`, `cargo test -p atomek-cli pod_selector`, and the Provider Jest pod-route tests.
- A non-destructive smoke check confirms `tytus exec --pod Lisa` and `--pod Claus` reach distinct agents and `--pod 01` errors with the ambiguity message, with no `revoke`, `logout`, or `disconnect` invoked.
- `git diff --name-only` lists only the intended source files (`state.rs`, `main.rs`, `transfer.rs`, `cmd_transfer.rs`, `agent.rs`, `pod.js`) plus their test files; `tray/web/os/*` and `tray/src/desktop_llm.rs` are untouched, or any Phase 1 escalation is explicitly noted.

**Files:**
- No production source changes in this phase; verification only across the files touched in Phases 1–4.
- New or updated test fixtures co-located with the `atomek-cli`, `atomek-pods`, and `wannolot-provider` test suites, using the synthetic `route-lisa` / `route-claus` / `route-hermie` values.

**Tests:**
- `cargo test -p atomek-pods` passes.
- `cargo test -p atomek-cli pod_selector` passes.
- The Provider Jest pod-route suite passes.
- `git diff --name-only` reviewed against the constraint list: only intended files present, with `desktop_llm.rs` and `tray/web/os/*` confirmed untouched (or the Phase 1 escalation recorded).

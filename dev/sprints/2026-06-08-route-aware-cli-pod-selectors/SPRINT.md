# SPRINT-ROUTE-AWARE-CLI-POD-SELECTORS

Date: 2026-06-08
Owner: Sebastian
Operator: Harvey
Lope validator: Claude only, per Sebastian request
Status: COMPLETE — implemented and validated in v0.7.39

## Origin: user request

Fix Tytus CLI/provider agent addressing so duplicate numeric `pod_id` routes are safely targetable by `route_id` or human `display_name`.

Observed production state on Sebastian's machine:

```text
Lisa   pod_id=01 route_id=0e0ah755r3 agent_type=nemoclaw display_name=Lisa
Claus  pod_id=01 route_id=eb2qvn3t4s agent_type=nemoclaw display_name=Claus
Hermie pod_id=01 route_id=12gy79s7g0 agent_type=hermes   display_name=Hermie
None   pod_id=02 route_id=t3n7s69day agent_type=none     display_name=<none>
```

`tytus exec --pod 01` can silently hit the first `01` route. `tytus exec --pod Lisa`, `--pod Claus`, `--pod Hermie`, and `--pod <route_id>` currently fail. That blocks safe shared-folder diagnostics and autonomous marketing-team operation.

## Phase 0 — Verification gate already performed

**Evidence collected before implementation:**

- `cli/src/state.rs::PodEntry` already has `route_id: Option<String>` and `display_name: Option<String>` with serde defaults.
- Real `~/Library/Application Support/tytus/state.json` already contains distinct `route_id` and `display_name` values for Lisa, Claus, and Hermie.
- Provider `services/wannolot-provider/src/routes/pod.js` already has `podSelectorFromRequest(req)` reading `route_id` from body/query/header and `findPod(...)` filtering by route.
- Provider `callDAM(...)` can route correctly if the CLI sends both `pod_id` and `route_id`.

**Chosen path:** read-side/wiring fix. No schema migration needed.

**Abort condition:** if later verification finds `route_id`/`display_name` absent or identical for multiple active routes, stop. That is an allocation/sync bug, not this sprint.

## Phase 1 — Safe pod-route selector resolver in CLI

**Goal:** Add one pure resolver that maps a user `--pod` selector to a unique route-aware target.

**Resolution rules:**

1. Exact `route_id` match wins.
2. If selector is numeric/pod-id-shaped (`^\d+$`), resolve by `pod_id` only:
   - exactly one match → OK
   - multiple matches → typed ambiguity error
   - no matches → not found
3. For non-numeric selectors, resolve by normalized `display_name` (`trim`, case-insensitive):
   - exactly one match → OK
   - multiple matches → typed ambiguity error
   - no matches → not found
4. No fallback from numeric selector to display name. This prevents a pod named `01` bypassing duplicate `pod_id` detection.
5. When no selector is supplied, keep current behavior only when safe: exactly one route or exactly one route with `tunnel_iface`. Otherwise fail ambiguous instead of picking the first route.

**Files:**

- `services/tytus-cli/cli/src/pod_selector.rs` — new pure resolver and tests.
- `services/tytus-cli/cli/src/main.rs` — import resolver.
- `services/tytus-cli/cli/src/lib.rs` — export resolver for tests if needed.

**Tests:**

- Three synthetic routes sharing `pod_id=01`; `Lisa` resolves to Lisa route.
- `0e0ah755r3` resolves to Lisa route.
- `01` returns typed ambiguity with three candidates.
- Duplicate `display_name=Lisa` returns typed ambiguity.
- Numeric display name edge: one route has `display_name=01`, three routes have `pod_id=01`; selector `01` returns pod-id ambiguity, not the display-name route.
- Unique `pod_id=02` resolves.

## Phase 2 — Thread route_id through CLI Provider client calls

**Goal:** When a route-aware target is resolved, Provider client calls include both `pod_id` and `route_id`.

**Files:**

- `services/tytus-cli/pods/src/agent.rs`
  - Add a small `AgentTarget { pod_id, route_id }` helper.
  - Keep existing `exec_in_agent(client, pod_id, ...)` API for backward compatibility.
  - Add `exec_in_agent_target(client, target, ...)` that includes `route_id` in JSON body when present.
  - Add route-aware variants for `restart_agent`, `agent_logs`, and any shared helper needed by CLI commands.
- `services/tytus-cli/cli/src/main.rs`
  - `cmd_exec`, `cmd_logs`, `cmd_restart` resolve selector and pass route-aware target.

**Tests:**

- Serialization/body builder includes `route_id` when set and omits it when absent.
- CLI unit tests cover resolver output; no network call is made on ambiguity.

## Phase 3 — Route-aware transfer commands

**Goal:** `tytus ls/push/pull/rm --pod Lisa` uses the same route-aware selector, because those commands use Provider exec internally.

**Files:**

- `services/tytus-cli/cli/src/transfer.rs`
  - Replace raw-string `resolve_pod` behavior for command execution with route-aware `ResolvedPodTarget`.
- `services/tytus-cli/cli/src/cmd_transfer.rs`
  - Thread `route_id` into internal exec calls.

**Tests:**

- `ls --pod Claus` resolves to `route_id=eb2qvn3t4s` in the internal exec target.
- Ambiguous `--pod 01` aborts before any upload/delete/list command can run.

## Phase 4 — Provider fail-closed ambiguity guard

**Goal:** Provider must not silently select the first route when a DAM proxy call receives duplicate `pod_id` without `route_id`.

**Justification:** This protects old CLI builds, TytusOS/Atomek/Resource Fabric routes, direct support curl, and any caller that still sends only `pod_id` to Provider agent-management endpoints.

**Files:**

- `services/wannolot-provider/src/routes/pod.js`
  - Add `findPodResolution(...)` that can return `{ pod }`, `{ notFound }`, or `{ ambiguous, matches }`.
  - Use it inside `callDAM(...)` so all DAM proxy calls fail closed before droplet/DAM contact when ambiguous.
  - Keep existing `findPod(...)` wrapper for non-DAM call sites unless they are changed in this sprint.

**Provider response:**

- HTTP `409`
- `error: "ambiguous_pod_selector"`
- sanitized candidates: `pod_id`, `route_id`, `agent_type`, `display_name`
- no secrets, keys, droplet IPs, or internal credentials

**Tests:**

- Duplicate `pod_id=01` without route id returns 409; axios/DAM mock not called.
- Duplicate `pod_id=01` with `route_id=0e0ah755r3` calls selected droplet/DAM once.
- Unique `pod_id=02` without route id still calls DAM once.

## Phase 5 — Verification and diff hygiene

**Commands:**

```bash
cargo test -p atomek-pods
cargo test -p atomek-cli pod_selector
cd services/wannolot-provider && npm test -- --runInBand __tests__/routes/pod.test.js
```

**Non-destructive smoke only:**

```bash
tytus exec --pod Lisa -- true
tytus exec --pod Claus -- true
tytus exec --pod Hermie -- true
tytus exec --pod 01 -- true   # must fail ambiguous
```

Do not run `tytus revoke`, `tytus logout`, `tytus disconnect`, or any sudo tunnel command during this sprint.

**Diff hygiene:**

`git diff --name-only` may include only intentional files plus pre-existing unrelated work already present before this sprint. Do not touch `tray/web/os/*` or `tray/src/desktop_llm.rs`.

## Acceptance bar

- Lisa/Claus/Hermie can be targeted by display name and route id.
- Bare duplicate `pod_id=01` fails closed.
- Provider route-aware calls include route id and select the intended route.
- Provider old-style ambiguous calls fail 409 before DAM side effects.
- Tests pass.
- Brain journal updated with the fix and remaining risks.

## Local implementation update — 2026-06-08 11:58 Europe/Berlin

Status: SHIPPED — included in v0.7.39 release commit/tag.

Additional Claude-only Lope check:

- Asked Claude whether the sprint should include local Tytus tray/forwarder surfaces.
- Recommendation: expand boundedly because tray/forwarder had the same duplicate-`pod_id` defect surface.

Extra local-app work completed:

- `tytus ui --pod <selector>` now uses the same route-aware resolver.
- UI/tunnel marker/cache keys use `route_id` when available, avoiding `/tmp/tytus/ui-01.*` collisions between Lisa/Claus/Hermie.
- `tytus connect --pod <selector>` downloads config with `route_id` when available and updates the matching state route instead of first numeric `pod_id`.
- Tray menu actions now pass stable route selectors into CLI file/shared-folder/forwarder commands instead of collapsing back to numeric `pod_id`.

Local install completed:

- Installed `~/bin/tytus` v0.7.38 during local dogfood; release target is v0.7.39.
- Installed `~/bin/tytus-tray` and `/usr/local/bin/tytus-tray` v0.7.38 during local dogfood; release target is v0.7.39.
- Refreshed `/Applications/Tytus.app` via `tytus tray install`.
- Backups stamped `20260608-115155`.

Verification:

```bash
cargo test --workspace --all-targets
cd services/wannolot-provider && npm test -- --runInBand __tests__/routes/pod.test.js
tytus exec --pod 01 -- true          # fails ambiguous
tytus exec --pod Lisa -- 'printf lisa-ok'
tytus exec --pod Claus -- 'printf claus-ok'
tytus exec --pod Hermie -- 'printf hermie-ok'
tytus exec --pod 0e0ah755r3 -- 'printf route-ok'
tytus ui --pod 01 --no-open         # fails ambiguous before forwarder/tunnel side effects
tytus ls /app/workspace/inbox/ --pod 01 # fails ambiguous before remote side effects
```


## Completion — 2026-06-08

Implemented in `v0.7.39`:

- `atomek-cli` resolves pod selectors by route id, display name, or unambiguous pod id. Duplicate pod ids fail closed instead of silently selecting the first route.
- `atomek-pods` forwards `route_id` in DAM exec requests so Provider can target the intended agent route.
- `tytus-tray` shared-folder bind accepts route selectors from TytusOS and validates unknown/injection/ambiguous selectors before provisioning.
- Regression tests passed: `cargo fmt --all -- --check`, targeted pod-selector/shared-folder tests, and `cargo test --workspace`.

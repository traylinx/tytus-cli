# Brief — Route-aware CLI pod selectors

Date: 2026-06-08
Owner: Sebastian
Operator: Harvey
Validator requested: Claude only

## Problem

Sebastian has a Tytus Operator setup with three named agent resources:

- Lisa — OpenClaw — `pod_id=01`, `route_id=0e0ah755r3`
- Claus — OpenClaw — `pod_id=01`, `route_id=eb2qvn3t4s`
- Hermie — Hermes — `pod_id=01`, `route_id=12gy79s7g0`
- included no-agent pod — `pod_id=02`, `route_id=t3n7s69day`

TytusOS Resource API exposes unique route ids, but CLI commands collapse the three agents under the same `pod_id=01`.

Observed failures:

```bash
tytus exec --pod Lisa ...        # 404 not_found
tytus exec --pod Claus ...       # 404 not_found
tytus exec --pod Hermie ...      # 404 not_found
tytus exec --pod 0e0ah755r3 ...  # 404 not_found
```

Only this works:

```bash
tytus exec --pod 01 ...
```

But it reaches only one OpenClaw workspace and can silently target the wrong agent.

## Evidence

`tytus status --json` compact:

```text
Lisa   pod_id=01 route_id=0e0ah755r3 agent_type=nemoclaw
Claus  pod_id=01 route_id=eb2qvn3t4s agent_type=nemoclaw
Hermie pod_id=01 route_id=12gy79s7g0 agent_type=hermes
None   pod_id=02 route_id=t3n7s69day agent_type=none
```

Provider already supports `route_id` selectors through `podSelectorFromRequest(req)` from body/query/header and `findPod(...)`.

Provider `callDAM(req, podId, ...)` currently defaults to route-aware selector but CLI only sends `pod_id`, not `route_id`.

## Constraints

- Be very careful. No broad refactor.
- Do not touch unrelated pre-existing modified assets in `tray/web/os/*` or `tray/src/desktop_llm.rs`.
- No destructive Tytus commands (`revoke`, `logout`, `disconnect`) during verification.
- Do not expose secrets from state.json or Provider responses.
- Preserve backward compatibility for users with unique `pod_id`s.
- When `pod_id` is ambiguous, fail closed with a useful message instead of silently choosing first.

## Target behavior

CLI selectors should resolve by:

1. exact `route_id`
2. exact/normalized `display_name` (e.g. Lisa, Claus, Hermie)
3. `pod_id` only when unique

When `--pod 01` matches several routes, error:

```text
Ambiguous pod selector "01" matched 3 pods. Use display name or route_id:
- Lisa: pod_id=01 route_id=0e0ah755r3 agent=OpenClaw
- Claus: pod_id=01 route_id=eb2qvn3t4s agent=OpenClaw
- Hermie: pod_id=01 route_id=12gy79s7g0 agent=Hermes
```

`tytus exec --pod Lisa ...` should send Provider body:

```json
{"pod_id":"01","route_id":"0e0ah755r3","command":"...","timeout":10}
```

`tytus ls/push/pull/rm --pod Lisa ...` should also use route-aware exec internally.

Provider should reject ambiguous DAM proxy calls without `route_id` rather than silently choosing first when duplicate `pod_id`s exist.

## Expected code areas

- `services/tytus-cli/cli/src/state.rs` — PodEntry has `route_id` and `display_name` already.
- `services/tytus-cli/cli/src/main.rs` — `cmd_exec`, `cmd_logs`, `cmd_restart` currently use raw pod_id.
- `services/tytus-cli/cli/src/transfer.rs` / `cmd_transfer.rs` — `resolve_pod` returns raw string; transfer commands use exec internally.
- `services/tytus-cli/pods/src/agent.rs` — Provider client sends only `pod_id`; add optional `route_id` support safely.
- `services/wannolot-provider/src/routes/pod.js` — Provider has route selectors; add ambiguity guard/tests for DAM calls.

## Acceptance gates

- Unit tests prove duplicate `pod_id=01` resolves by `Lisa` and `route_id` but rejects raw `01` as ambiguous.
- Unit tests prove route id is included in Provider exec body/query where relevant.
- Provider tests prove duplicate `pod_id` without route id returns 409 and does not call DAM.
- Provider tests prove duplicate `pod_id` with route id calls the selected droplet/DAM.
- Targeted tests pass:
  - `cargo test -p atomek-pods`
  - `cargo test -p atomek-cli pod_selector`
  - relevant Provider Jest tests for pod routes
- `git diff` contains only intentional files.

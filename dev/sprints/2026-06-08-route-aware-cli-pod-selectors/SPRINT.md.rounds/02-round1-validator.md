# Round 1 — validator

- status: NEEDS_FIX
- confidence: 0.82
Reviewed. Design sound, well-scoped, good fail-closed instinct and byte-identical backward-compat discipline. But rests on several unverified data-model claims, and one real edge case is missing.

**Evidence gaps (no in-prompt support):**

1. **Core assumption unverified — `PodEntry.route_id` + `PodEntry.display_name` exist.** Entire sprint depends on it. Proposal asserts "no schema change" with zero evidence the fields are there. Worse: CLAUDE.md model says *"Pod = one agent container"* (nemoclaw/hermes, one per pod). Three OpenClaw agents under `pod_id=01` contradicts the documented single-container-per-pod model. If `state.json`/`PodEntry` has no per-route structure, Phase 1 is a schema change, not a read-only resolver. Verify first.

2. **Hardcoded route_ids have no provenance.** `Lisa=0e0ah755r3`, `Claus=eb2qvn3t4s` asserted as exact test expectations. Hermie's route_id never given (yet "lists all three candidates in order"). Specify these as explicit test fixtures; don't bake unverified live values into assertions.

3. **Provider `findPod`/`podSelectorFromRequest` current behavior unverified.** Phase 3 needs multiplicity detection, but if `findPod` returns first-match-only today (the bug being fixed), "reuse existing" understates the work — need a multi-match return path. Confirm.

**Missing edge case:**

4. **Ambiguous `display_name` not handled.** Only ambiguous `pod_id` fails closed. Two agents with same normalized `display_name` would silently resolve to first match — the exact silent-mistarget bug this sprint exists to kill. Must fail closed on ambiguous display_name too. Add resolver case + test.

**Scope/exclusion to justify:**

5. **`desktop_llm.rs` exclusion unjustified.** Why name it as untouched? If it routes by `pod_id` (exec-style), excluding it leaves the hole open. Gateway model calls go via `10.42.42.1:18080` (no pod selection), so likely safe — but state that explicitly rather than assert the exclusion.

**Non-blocking:**
- Package names `atomek-cli`/`atomek-pods` vs dir `services/tytus-cli/*` — confirm Cargo names match (plausible, just flag).
- No NotFound render test (Phase 1 defines the type, never asserts its output).
- File count holds: 5 modified (main/transfer/cmd_transfer/agent/pod.js) + state.rs read-only. Confirm state.rs truly unchanged in diff.

---VERDICT---
status: NEEDS_FIX
confidence: 0.82
rationale: Design and fail-closed approach correct, but four claims (PodEntry schema, route_id values, findPod multiplicity behavior) lack in-prompt evidence and contradict the documented one-container-per-pod model; ambiguous display_name edge case missing.
required_fixes:
  - Verify PodEntry already carries route_id + display_name and state.json has multi-route-per-pod structure; reconcile with CLAUDE.md "one agent container per pod" model before building. If absent, this is a schema change, not read-only.
  - Define route_ids (Lisa/Claus/Hermie) as explicit test fixtures; supply Hermie's; remove unverified live values from assertions.
  - Confirm Provider findPod/podSelectorFromRequest can return multiple matches today; if first-match-only, add multiplicity-detection work to Phase 3 scope.
  - Add ambiguous-display_name fail-closed case + test — current proposal only guards ambiguous pod_id, leaving the same silent-mistarget bug for duplicate display names.
  - Justify desktop_llm.rs exclusion (confirm it doesn't route by pod_id; gateway path uses 10.42.42.1:18080, not pod selection).
---END---

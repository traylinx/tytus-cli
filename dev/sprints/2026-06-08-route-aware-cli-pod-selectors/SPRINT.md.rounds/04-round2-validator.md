# Round 2 — validator

- status: NEEDS_FIX
- confidence: 0.82
Reviewed. Structure sound — Phase 1 gating before code is the right call given CLAUDE.md documents one-container-per-pod. But several gaps, one of which means the sprint as written may not fix the reported bug.

## Blocking

**1. No write/population path for `route_id`/`display_name`.** Schema-change path adds fields with `#[serde(default)]` → existing collapsed entries deserialize with EMPTY route_id/display_name. The actual Lisa/Claus/Hermie entries under `pod_id=01` then still can't be distinguished — resolver has nothing to match. Phases 2-4 build read-side resolution; nothing writes the distinct values. Feature is inert for the exact case it targets. Need a phase/criterion: where do distinct route_id/display_name come from (allocation time? migration of existing state.json?).

**2. Phase 1 has no ABORT branch.** It branches read-only vs schema-add, never STOP. Operator = 4 units (per CLAUDE.md tiers). Plausible real root cause: 3 agents mis-allocated under one pod_id when they should be separate pods (02/03/04) — an *allocation* bug, not an *addressing* bug. If so, `resolve_pod_selector` is dead code for an impossible/wrong model. Add third branch: escalate to operator if fix belongs at allocation layer.

**3. Premise unverified ("reportedly collapsed").** Phase 1 must confirm reproduction — a real `state.json` showing ≥2 routes under `pod_id=01` — with hard stop before Phases 2-5 if not reproduced. Currently Phase 1 records yes/no but doesn't gate on repro existing.

## Should fix

**4. File paths / crate names unverified in-prompt.** `atomek-cli`/`atomek-pods` crates vs `services/tytus-cli/` dir = naming mismatch (plausible but unconfirmed). `wannolot-provider/src/routes/pod.js`, `callDAM`, `findPod`, `podSelectorFromRequest`, `desktop_llm.rs` — none evidenced in prompt context. Phase 1 audit must confirm these exist before scoping; flag mismatch explicitly.

**5. Edge case: display_name numerically equal to a pod_id.** Precedence is display_name BEFORE pod_id. `--pod 01` where some entry has `display_name="01"` resolves via display_name and skips pod_id duplicate detection → potential mistarget, the exact bug class. Define behavior.

**6. Terminology pod vs route inconsistent** — incl. mandated error copy `matched 3 pods` (they're routes). The whole bug is pod≠route; sloppy copy undercuts it. Align, and don't pin exact human copy in tests (brittle) — assert on the typed `Ambiguous` variant + candidate list instead.

**7. Justify or cut Phase 4.** If only the CLI selects by pod_id against the Provider DAM proxy (Phase 1 confirms tray/web/os/desktop_llm use gateway `10.42.42.1:18080`, not pod selection), the Provider guard is defense-in-depth, not required. Per CLAUDE.md the curl recipes hit the gateway directly, not provider pod selection. State the non-CLI threat that justifies it, or cut to keep scope tight.

---VERDICT---
status: NEEDS_FIX
confidence: 0.82
rationale: Gating structure is correct, but as written the feature likely won't fix the reported bug — new defaulted route_id/display_name leave existing collapsed entries indistinguishable, and there's no write/population path. Phase 1 also lacks an abort branch for the plausible allocation-bug root cause.
required_fixes:
  - Add write/population path for route_id/display_name (allocation + migration of existing state.json); without it resolver has nothing to match for Lisa/Claus/Hermie
  - Add Phase 1 ABORT/escalate branch — if root cause is mis-allocation (3 agents should be separate pods on Operator's 4 units), addressing fix is the wrong layer
  - Phase 1 must gate on actual reproduction (real state.json with ≥2 routes under pod_id=01); hard-stop Phases 2-5 if not reproduced
  - Confirm file paths/crate names in Phase 1 (atomek-cli/atomek-pods vs tytus-cli dir mismatch; pod.js/callDAM/findPod unverified in-prompt)
  - Define precedence edge case where display_name is numerically equal to a pod_id (currently bypasses pod_id duplicate detection)
  - Fix pod/route terminology incl. error copy ("matched 3 pods"→routes); assert tests on typed Ambiguous variant, not exact human string
  - Justify Phase 4 with a concrete non-CLI caller that selects by pod_id against the Provider, or cut it
---END---

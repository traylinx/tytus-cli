# Phase 1 execution boundary

**Status:** Accepted for Phase 0.
**Date:** 2026-05-06.
**Sprint:** TYTUS-OS-MULTIPLATFORM.

## Decision

Phase 1 is a truth-building and release-pipeline phase. It does not implement the full platform substrate, tunnel parity, wizard, installers, or update apply path.

Phase 1 starts only after Phase 0 docs are committed. Phase 1 ends when CI and local evidence expose the real build/test state for all target platforms and the TytusOS web bundle can no longer silently drift from the tray artifact.

## Phase 1 deliverables

Phase 1 owns exactly these deliverables:

1. CI matrix for macOS, Ubuntu, and Windows.
2. Build/check/test commands for all Rust workspace packages on each CI platform.
3. TytusOS app build/typecheck/test in CI.
4. Deterministic TytusOS dist generation and embedding into `tytus-tray` release artifacts.
5. Replacement or tightening of `scripts/check-tytus-os-dist.sh` so dist drift is a release blocker.
6. Current Windows compile/stub inventory.
7. Current Linux tray/dependency inventory for Ubuntu 22.04/24.04.
8. Boringtun/Wintun feasibility spike results captured as evidence, without committing to product tunnel implementation work.
9. Grep/static gates for platform leaks that would block Phase 2 substrate work.

## Explicitly out of Phase 1

- Platform path/IPC/service abstraction implementation.
- Windows daemon lifecycle implementation beyond compile/stub inventory.
- Tunnel implementation or route/firewall mutation.
- First-run wizard implementation.
- MSI/pkg/deb/rpm production packaging.
- Update download/apply/rollback implementation.
- Public download-page release.

## Required inventories

Phase 1 must produce short evidence files or CI artifacts for:

- Windows `cargo check --workspace --all-targets` status.
- Linux `cargo check --workspace --all-targets` status with tray dependencies installed.
- All Unix-only imports that touch code expected to compile on Windows.
- Current daemon IPC call sites.
- Current path assumptions, especially `/tmp/tytus` and platform home directories.
- Current TytusOS dist hash and embedded tray dist hash.
- Current tray technology/runtime dependencies.
- Current tunnel crate status on Windows, macOS, Ubuntu.

## Phase 1 pass/fail rule

Phase 1 may expose red CI for Linux/Windows at first. That is the point.

Phase 1 passes only when:

- red jobs have exact file/error evidence
- dist drift cannot silently ship
- Phase 2 worklist is generated from evidence, not guesses
- no new platform abstraction code is merged without the Phase 2 substrate plan

After Phase 2 starts, red platform CI is no longer allowed as an indefinite baseline.

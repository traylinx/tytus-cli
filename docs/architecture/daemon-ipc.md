# ADR: Tytus daemon IPC contract

**Status:** Accepted for Phase 0.
**Date:** 2026-05-06.
**Sprint:** TYTUS-OS-MULTIPLATFORM.

## Decision

Tytus uses localhost HTTP bound to `127.0.0.1` as the cross-platform daemon/tray/TytusOS control plane.

Authentication uses a high-entropy per-user random token. Every request from tray, CLI helper, and TytusOS web shell to the local daemon must present the token. The token is never sent to Traylinx cloud services and never written into support bundles.

Unix sockets may remain as a macOS/Linux legacy compatibility path during migration, but they are not the production cross-platform contract. Windows named pipes are deferred unless loopback HTTP fails security review or platform smoke.

## Why this decision

The sprint needs one control-plane model that works on macOS, Linux, and Windows without spending Phase 2 on IPC experiments.

Named pipes are viable on Windows, but ACL setup and Rust async ergonomics are a higher implementation risk for this sprint. Unix sockets do not solve Windows. Loopback HTTP is boring, inspectable, easy to test in browsers, and already matches the local web-server shape used by tray/TytusOS.

## Security requirements

- Bind only to `127.0.0.1`; never `0.0.0.0`.
- Generate token with OS CSPRNG, minimum 256 bits.
- Store token in OS credential store when available.
- If runtime-file fallback is required, file must be owner-only permissioned:
  - macOS/Linux: `0600` file inside platform runtime/config dir.
  - Windows: current-user ACL only.
- Rotate token on repair, reinstall, logout, suspected leak, or permission repair.
- Reject requests without token.
- Reject requests with wrong token using constant-time comparison where feasible.
- Redact token from logs, errors, telemetry, support bundles, and panic output.
- CORS allows only the local TytusOS origin and explicit CLI/tray callers.
- WebSocket/SSE channels use the same token.

## Runtime paths

The Phase 2 platform substrate owns exact paths, but the contract is:

- macOS runtime: platform runtime dir, with legacy read compatibility for `/tmp/tytus/tray-web.port` during migration.
- Linux runtime: `$XDG_RUNTIME_DIR/tytus` when available; fallback to owner-only runtime dir under user cache/runtime path.
- Windows runtime: `%LOCALAPPDATA%\Tytus\Runtime` or equivalent current-user local app data path.

No new production code may hard-code `/tmp/tytus` outside migration shims and tests.

## Port and discovery protocol

The daemon uses a dynamic localhost port by default to avoid conflicts. Fixed ports are allowed only in tests or explicit developer overrides.

Discovery is file-based through the Phase 2 platform runtime directory:

- macOS: platform runtime dir with legacy read compatibility for `/tmp/tytus/tray-web.port`.
- Linux: `$XDG_RUNTIME_DIR/tytus/control.json` when available.
- Windows: `%LOCALAPPDATA%\Tytus\Runtime\control.json`.

`control.json` contains:

- `schema_version`
- `pid`
- `port`
- `started_at`
- `token_ref`, never the raw token when OS credential storage is available
- `token_file`, only for owner-only fallback mode

The discovery file must be owner-only permissioned and recreated on daemon start. Clients verify daemon liveness before trusting stale discovery data.

The raw IPC token is never placed in URLs. Browser bootstrap uses same-origin session material issued by the local daemon, not query parameters.

## API shape

The daemon exposes local endpoints for:

- health and version
- auth/session state
- pod allocation/selection state
- tunnel status and repair action
- credential backend doctor
- update status/apply
- support bundle creation
- tray lifecycle status

Every state-changing endpoint requires token auth. Destructive or privileged repair actions require both token auth and an explicit user gesture from tray/TytusOS.

## Migration plan

1. Introduce shared IPC client/server module.
2. Keep existing Unix socket reads as compatibility shims on macOS/Linux.
3. Move tray and CLI calls to localhost HTTP token contract.
4. Add Windows daemon implementation using the same HTTP/token contract.
5. Add tests for token generation, token storage permissions, missing-token rejection, wrong-token rejection, and support-bundle redaction.
6. Remove legacy `/tmp/tytus` dependency after release migration window.

## Acceptance evidence

Phase 2 cannot pass until these are true:

- macOS, Linux, Windows unit tests prove path and token behavior.
- No unguarded Unix-only import remains in code that must compile on Windows.
- Local daemon refuses unauthenticated requests.
- Support bundle redacts token.
- `cargo check --workspace --all-targets` passes on all CI platforms or fails only on documented pre-Phase2 platform gaps.

## Rejected options

| Option | Reason rejected for this sprint |
|---|---|
| Unix socket everywhere | No Windows parity. |
| Windows named pipe primary | More ACL and async implementation risk than needed for Phase 2. |
| Unauthenticated localhost HTTP | Unsafe; browser-origin and local malware risks are unacceptable. |
| Cloud-mediated control plane | Breaks offline/local-first tray flow and adds avoidable cloud dependency. |

# Tytus CLI — Security Model

**Last updated:** 2026-04-12
**Status:** Launch-ready after E2–E5 and H1 fixes.

This document describes the threat model, security invariants, and intentional
design decisions. It is kept deliberately short. If you are looking for the
raw audit trail, see `docs/SECURITY-DEEP-AUDIT-2026-04-12.md` and
`docs/PENTEST-RESULTS-2026-04-12.md`.

## Threat model

We protect against the following attackers:

| Attacker                                                   | Protected against |
|------------------------------------------------------------|-------------------|
| **Same-host user-level process** (malware, sandboxed app)  | Yes               |
| **Same-host malicious AI agent** (MCP client, npm postinstall) | Yes           |
| **Passive network observer on the LAN/ISP path**           | Yes               |
| **Active network MITM with a rogue CA**                    | Yes (rustls + WebPKI)    |
| **Someone who gets physical root on the user's machine**   | No (out of scope)         |
| **Rails/Sentinel backend compromise**                      | No (out of scope)         |

## Key invariants

1. **Refresh tokens live in the OS keychain only**. State files never contain
   `refresh_token`. See `cli/src/state.rs::load()` for the migration path from
   legacy state files. Enforced via `#[serde(skip_serializing)]` on the field.

2. **State file mode is 0600**. Enforced at every write via `save()` and
   `save_critical()`. Verified by tests.

3. **`/tmp/tytus/` is 0700 and every file in it is 0600**. Enforced via
   `secure_tytus_tmp_dir()` + `secure_chmod_600()` helpers called at every
   write site (CLI, tray, daemon, tunnel helper).

4. **WireGuard private keys never touch disk**. The tunnel config is parsed
   into an in-memory `TunnelConfig` struct and handed to boringtun directly.
   `WireGuardConfig` and `WannolotPassResponse` implement `Zeroize`.

5. **Sudoers is tightly scoped**. The entry grants exactly two commands:
   ```
   /Users/USER/bin/tytus tunnel-up /tmp/tytus/tunnel-*.json
   /Users/USER/bin/tytus tunnel-down *
   ```
   The `tunnel-down` helper validates the target PID against
   `/tmp/tytus/tunnel-*.pid` files before signalling, so it cannot be used as
   an arbitrary `kill` primitive. The `tunnel-up` path pattern prevents
   pointing the helper at `/etc/shadow` or an attacker-controlled config.

6. **TLS is rustls + WebPKI roots, no `native-tls`, no plaintext fallback**.
   Every `reqwest::Client` in the tree goes through `atomek-core::HttpClient`
   or is audited for the same TLS config.

7. **MCP tools return stable values only by default**. `tytus_env`,
   `tytus_status`, and the daemon socket all emit
   `stable_ai_endpoint` (`http://10.42.42.1:18080`) and
   `stable_user_key` (`sk-tytus-user-<32hex>`) by default. Internal pod IPs
   and per-pod ephemeral keys are opt-in via `--raw` / `raw=true`.

## Intentional design decisions (with threat model)

### The hardcoded `Api-Key` is a public client identifier, not a secret

`auth/src/login.rs` and `auth/src/sentinel.rs` both contain:

```rust
const PUBLIC_CLIENT_API_KEY: &str = "2qQaEiyjeqd0F141C6cFeqpJ353Y7USl";
```

This is the Rails `Api-Key` header value. It is **intentionally public** and
is used to identify "this request is coming from the Tytus CLI" for
telemetry, per-client rate limiting, and feature flagging. It is shipped in
every public binary, exactly like:

- Firebase Web SDK API keys (hardcoded into every web app)
- Auth0 `client_id` values (public JavaScript config)
- Stripe publishable keys (`pk_live_*` — in every e-commerce frontend)

**Why this is safe**: every endpoint that consumes this value also requires
user credentials on top of it:

| Endpoint                                  | Additional required credential  |
|-------------------------------------------|---------------------------------|
| `/ma-authentication-ms/v1/api/auth/login` | email + password in body        |
| `/ma-authentication-ms/v1/api/auth/refresh` | refresh_token in body         |
| `/ma-metrics-wsp-ms/v1/api/me/wannolot-pass` | user OAuth Bearer in header  |

An attacker who extracts this key from the binary gains exactly the same
access surface as a user who downloads the CLI: none, until they supply their
own credentials. The key is metadata, not a gatekeeper.

**Invariant this depends on**: the Rails API must never add an endpoint that
treats `Api-Key` as a standalone credential. If it does, this value becomes
a leaked secret, not a public client ID. That would be a Rails-side
regression — catch it during Rails code review, not CLI review.

**Not rotatable without breaking every installed binary.** If we ever need to
rotate it, we must coordinate a forced upgrade of every deployed client, and
the old value must remain valid for the full deprecation window.

### Root daemon runs for the full session

The `tunnel-up` helper runs as root for the lifetime of the tunnel (hours to
days). It needs root only briefly: TUN device creation + route setup. In
principle it should drop privileges after that. We currently don't. The
attack surface is limited because:

- The binary is tightly scoped (no shell, no file writes outside `/tmp/tytus`)
- The sudoers entry is wildcard-free (`tunnel-up /tmp/tytus/tunnel-*.json`)
- PID validation prevents misuse of `tunnel-down`

Lowering the privilege drop is tracked as M3 in the deep audit; it is
post-launch work.

## Install security

The one-liner install flow (`curl -fsSL https://tytus.traylinx.com/install.sh | bash`)
is safe to post publicly because:

- **SHA256 verification is mandatory.** The installer downloads `SHA256SUMS`
  from the release and refuses to install if any binary's hash doesn't match.
  Escape hatch: `TYTUS_SKIP_CHECKSUM=1` (not recommended).

- **The GitHub release workflow emits `SHA256SUMS` for every artifact.** See
  `.github/workflows/release.yml`.

- **Homebrew, Windows PowerShell, and direct-curl paths all verify.**

What this does NOT protect against:

- Compromise of the GitHub account publishing releases. (Mitigation: protected
  branch rules + required reviews on the release workflow + hardware MFA.)
- Compromise of the Cloudflare Pages static host serving the landing page.
  (Mitigation: install script is also mirrored on `raw.githubusercontent.com`.)

A future version will add cosign signing of the SHA256SUMS file + keyless
verification in the installer; this is tracked as post-launch hardening.

## Reporting a vulnerability

Email `security@traylinx.com`. Please do not open public GitHub issues for
security findings.

## Audit history

- `docs/DEEP-AUDIT-2026-04-03.md` — first audit (pre-CLI pivot)
- `docs/SECURITY-HARDENING-2026-04-12.md` — network/infra sweep + CLI output redaction
- `docs/SECURITY-DEEP-AUDIT-2026-04-12.md` — 34 findings, 1 CRITICAL, 5 HIGH
- `docs/PENTEST-RESULTS-2026-04-12.md` — red team exploitation proof
- `docs/SECURITY.md` (this file) — steady-state model

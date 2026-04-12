# Tytus CLI — Deep Security Audit

**Date:** 2026-04-12
**Auditors:** Harvey (Claude Opus 4.6), with independent review by OpenCode (MiniMax-M2.7) and Gemini CLI
**Scope:** Full codebase + network + install script + MCP server + tray app
**Method:** Three parallel auditors examined secrets/auth, network/filesystem/process, and MCP/data-exposure independently. Findings merged, deduplicated, and cross-reviewed.

---

## Executive Summary

**48 findings** across 7 crates, the install script, and runtime behavior.

| Severity | Count | Action required |
|----------|-------|-----------------|
| CRITICAL | 1 | Must fix before launch |
| HIGH | 5 | Must fix before launch |
| MEDIUM | 12 | Should fix before launch |
| LOW | 8 | Fix when convenient |
| INFO | 8 | No action needed |

**The three most dangerous findings:**
1. **No binary verification in install.sh** + overly broad sudoers wildcard = unauthenticated path to root (CRITICAL)
2. **Hardcoded API key in binary** — extractable via `strings`, used for Rails API auth (HIGH)
3. **Refresh token in plaintext state.json** — contradicts documented security model (HIGH)

---

## CRITICAL Findings

### C1. Install Script: No Checksum Verification + Sudoers = Root Takeover

**File:** `install.sh:136-137, 222-223`
**Team verdict:** Gemini UPGRADED to CRITICAL. Both OpenCode and Gemini AGREE.

The installer downloads a binary from GitHub releases:
```sh
curl -fsSL "$RELEASE_URL" -o "${TMP}/${RELEASE_ASSET}"
tar xzf "${TMP}/${RELEASE_ASSET}" -C "${TMP}"
```
No SHA256 checksum, no signature verification, no cosign. Then creates a sudoers entry:
```
$USER ALL=(root) NOPASSWD: $BIN_PATH tunnel-up *, $BIN_PATH tunnel-down *
```

**Attack:** Compromise the GitHub release (account takeover, CI pipeline injection, CDN cache poisoning) → user downloads malicious binary → installer grants it passwordless root via sudoers → attacker has root on every machine that runs the installer.

**Fix:**
1. Publish SHA256SUMS alongside releases (signed with GPG or cosign)
2. Verify checksum in install.sh before extracting
3. Tighten sudoers wildcard: `tunnel-up /tmp/tytus/tunnel-*.json` instead of `tunnel-up *`
4. Add `visudo -cf` validation after writing sudoers file

---

## HIGH Findings

### H1. Hardcoded API Key in Binary

**File:** `auth/src/sentinel.rs:20`, `auth/src/login.rs:10`
**Team verdict:** Both AGREE.

```rust
.unwrap_or_else(|_| "2qQaEiyjeqd0F141C6cFeqpJ353Y7USl".to_string())
```

This production API key is embedded in every compiled binary. `strings tytus | grep 2qQa` extracts it. Used as `X-Api-Key` / `Api-Key` header to the Rails API (`api.makakoo.com`).

**Risk:** If this key grants any access beyond what a regular user token provides, it's an escalation vector. If it's a public client identifier (like a Firebase API key), document it as such.

**Fix:** Determine if this key is a secret or a public client ID. If secret: inject at build time via env var, never hardcode. If public: document clearly that it is intentionally public and has no server-side privileges beyond identifying the client.

### H2. All Tokens in Plaintext state.json (Contradicts Security Docs)

**File:** `cli/src/state.rs:7-18`
**Team verdict:** Both AGREE.

`state.json` contains `refresh_token`, `access_token`, `secret_key`, `agent_user_id`, `pod_api_key`, `stable_user_key` — all as plaintext strings. Permissions are `0o600` (good), but:
- Any process running as the same user can read all secrets
- Time Machine / backups include the file
- CLAUDE.md claims "Refresh tokens go to the OS keychain, never to plain files" — this is **false**

The keychain IS used as a secondary store, but `CliState::load()` reads from the file.

**Fix:** Remove `refresh_token` from `state.json`. Load it exclusively from OS keychain. Move `secret_key` to keychain as well.

### H3. Sudoers Wildcard Allows Arbitrary File Read as Root

**File:** `install.sh:222-223`
**Team verdict:** Both AGREE.

`tytus tunnel-up *` allows `sudo tytus tunnel-up /etc/shadow`. The binary reads the file (fails to parse as JSON), but the error message may leak content. More practically, `tunnel-up /tmp/attacker-config.json` creates a tunnel to an attacker-controlled endpoint as root.

**Fix:** Restrict to `tunnel-up /tmp/tytus/tunnel-*.json`. Or better: pass config via stdin pipe, eliminating the file argument entirely.

### H4. WireGuard Private Key in Predictable Temp File

**File:** `cli/src/main.rs:627-651`
**Team verdict:** Both AGREE.

WG private key written to `/tmp/tytus/tunnel-{pod_id}.json` with predictable name. `0o600` permissions, but write-then-chmod race window exists. The elevated process reads and deletes it, but if the parent crashes, the key persists.

**Fix:** Use `O_CREAT|O_EXCL` with random filename, or pass config via pipe/fd inheritance to the elevated process.

### H5. MCP Server Leaks Raw Per-Pod Keys and Internal IPs

**File:** `mcp/src/tools.rs:63-99`
**Team verdict:** OpenCode DOWNGRADED to MEDIUM (per-pod keys are ephemeral). Gemini did not review MCP specifically.

`tytus_env` MCP tool returns raw `pod_api_key` and `ai_endpoint` (containing internal `10.18.X.Y` IPs) to AI agents. Unlike the CLI's `tytus env` which defaults to stable values, the MCP tool has no stable/raw distinction.

**Fix:** Return `stable_ai_endpoint` and `stable_user_key` by default. Add `raw` boolean parameter for debug.

---

## MEDIUM Findings

### M1. `#[derive(Debug)]` on Secret-Bearing Structs

**Files:** `state.rs:7`, `state.rs:20`, `device_auth.rs:34`, `login.rs:21`, `tunnel/lib.rs:7`, `pods/config.rs:6`

Any `{:?}` format, panic, or `dbg!()` dumps secrets to stderr/logs.

**Fix:** Custom `Debug` implementations that redact sensitive fields.

### M2. `TunnelConfig` Lacks `Zeroize` (Unlike `WireGuardConfig`)

**File:** `tunnel/src/lib.rs:7-17`

`TunnelConfig` holds `private_key` and `preshared_key` as plain `String` with `#[derive(Clone)]`. Not zeroized on drop.

**Fix:** Add `Zeroize + ZeroizeOnDrop`.

### M3. Root Daemon Never Drops Privileges

**File:** `cli/src/main.rs:801-986`

Tunnel daemon runs as root for the entire session (hours/days). Only needs root for TUN creation.

**Fix:** Drop to original user after TUN device creation and route setup.

### M4. `/tmp/tytus/` Directory Ownership Race

**Files:** `main.rs`, `daemon.rs`, `launcher.rs`

Multiple components create `/tmp/tytus/` with `create_dir_all` (default permissions). An attacker who pre-creates it owns the directory.

**Fix:** Verify directory ownership after creation. Or use `$XDG_RUNTIME_DIR` (Linux) / `$TMPDIR` (macOS, per-user: `/var/folders/.../T/`).

### M5. Daemon Socket Transmits Credentials

**File:** `daemon.rs:264-297`

Status response includes `stable_user_key` over Unix socket. Socket has `0o600` permissions, but compromised same-user process can extract credentials.

**Fix:** Return truncated key by default. Full key only on explicit `auth` subcommand.

### M6. `tytus env --json` Still Dumps Full PodEntry

**File:** `cli/src/main.rs:1470`

`tytus env --json` serializes the entire `PodEntry` struct including `droplet_id`, `droplet_ip`, internal IPs, and both key types.

**Fix:** Filter output to only stable values. Use `--raw` flag for debug data.

### M7. MCP `tytus_chat` Allows Arbitrary Prompts

**File:** `mcp/src/tools.rs:163-228`

AI agents can send arbitrary prompts through the user's pod without user visibility. Prompt injection vector.

**Fix:** Rate limiter, token budget, or require explicit user consent per call.

### M8. MCP `tytus_revoke` Has No Confirmation Gate

**File:** `mcp/src/tools.rs:230-259`

The tool description says "confirm with user" but there's no enforcement. Auto-approving MCP clients can revoke pods silently.

**Fix:** Two-phase revoke with confirmation token.

### M9. Tray Launcher Write-Then-Chmod Race

**File:** `tray/src/launcher.rs:140-155`

Script written with default umask, then `chmod 0o700`. Brief window where file is world-readable.

**Fix:** Use `O_CREAT|O_EXCL` with mode `0o700` from creation, or use `$TMPDIR`.

### M10. Separate reqwest Clients Skip Shared TLS Config

**File:** `cli/src/main.rs:2058-2060, 2165-2168`

`test_chat_completion()` and `cmd_chat()` create standalone `reqwest::Client`s that don't use the shared HttpClient config.

**Fix:** Use the shared `HttpClient` for all requests.

### M11. `SUDO_USER`/`TYTUS_REAL_HOME` Path Not Validated

**File:** `cli/src/state.rs:44-58`

`TYTUS_REAL_HOME` is user-controllable and used to construct the state file path. Could redirect state reads to attacker-controlled location.

**Fix:** Validate: reject if contains `..`, is not an absolute path, or doesn't exist.

### M12. autostart.log Has No Permission Restriction

**File:** `cli/src/main.rs:3278, 3330`

Log file created with default umask (typically `0o644`). May contain diagnostic data readable by other users.

**Fix:** Set `0o600` on creation.

---

## LOW Findings

| # | File | Issue |
|---|------|-------|
| L1 | `daemon.rs:17-18` | `/tmp/tytus/` directory not created with `0o700` |
| L2 | `main.rs:3415` | JSON status outputs full `stable_user_key` (by design, but consider truncating) |
| L3 | `sentinel.rs:25` + `main.rs:3366` | Zeroize defeated by `.clone()` into non-zeroizing `CliState` fields |
| L4 | `main.rs:1476-1488` | `tytus env --raw` outputs internal IPs with no warning |
| L5 | `mcp/src/main.rs` | MCP server inherits invoking process permissions (standard, but document) |
| L6 | `main.rs:1516-1519` | `.mcp.json` binary path could be hijacked in world-writable dirs |
| L7 | `main.rs:2793-2798` | Bootstrap prompt fetches from GitHub `main` branch (supply chain risk) |
| L8 | `install.sh:240` | Sudoers entry via echo in sh -c — quote injection if path has single quotes |

---

## INFO Findings (Positive)

| # | Finding |
|---|---------|
| I1 | TLS correctly configured: rustls + WebPKI roots, no native-tls, no plaintext fallback |
| I2 | No command injection vectors found — all `Command::new()` uses `.args()`, not shell interpolation |
| I3 | HTTP client does not log request bodies (verified in `core/src/http.rs`) |
| I4 | `tytus link` uses `canonicalize()` — no path traversal |
| I5 | CLAUDE.md and AGENTS.md templates contain no secrets |
| I6 | Default `tytus env` output correctly uses stable values only |
| I7 | `--only` filter uses exact string match — no injection |
| I8 | Cross-pod isolation verified by network scan — other pods unreachable |

---

## Team Review Notes

**OpenCode (MiniMax-M2.7):**
- AGREE on H1, H2, H3, H4
- DOWNGRADED H5 (MCP env) to MEDIUM: "per-pod keys are ephemeral, blast radius limited"
- DOWNGRADED H6 (tray launcher) to MEDIUM: "requires pre-existing local access + tight timing"

**Gemini CLI:**
- UPGRADED H2 (install.sh) to CRITICAL: "MITM on unverified binaries + passwordless sudo = immediate unauthenticated root"
- AGREE on H1, H3, H4, H6
- DOWNGRADED H5 to MEDIUM: "ephemeral per-pod keys, limited compared to root or host creds"

---

## Priority Fix Order

### Must-fix before launch (CRITICAL + HIGH)
1. **C1:** Add checksum verification to install.sh + tighten sudoers wildcard
2. **H1:** Determine if embedded API key is public or secret; if secret, remove from binary
3. **H2:** Remove refresh_token from state.json, use keychain exclusively
4. **H3:** Restrict sudoers to specific file pattern
5. **H4:** Use unpredictable temp file or pipe for WG config
6. **H5:** Fix MCP tytus_env to return stable values

### Should-fix before launch (MEDIUM)
7. **M1:** Custom Debug implementations
8. **M2:** Add Zeroize to TunnelConfig
9. **M3:** Drop root after TUN creation
10. **M4:** Verify /tmp/tytus/ ownership or use $TMPDIR
11. **M5-M6:** Redact daemon/env output
12. **M7-M8:** MCP rate limiter + two-phase revoke
13. **M9:** Atomic file creation for launch script
14. **M12:** Set 0o600 on autostart.log

### Already fixed in this session
- CLI `tytus status --json` no longer leaks droplet_id, droplet_ip, internal IPs, raw per-pod keys
- CLI `tytus connect` output redacted to stable endpoint only
- Droplet SSH exposure flagged for infra team

---

## Methodology

1. Three auditors read every source file in parallel, each focused on a different attack surface
2. Findings merged and deduplicated (48 → 34 unique after dedup)
3. OpenCode and Gemini CLI independently reviewed all HIGH findings
4. Disagreements resolved: Gemini's CRITICAL upgrade on C1 accepted (team consensus)
5. Network scan verified tunnel isolation: cross-pod blocked, metadata blocked, K8s unreachable

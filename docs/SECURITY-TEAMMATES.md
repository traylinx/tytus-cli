# Tytus Lope Teammates — Security Audit (v0.5.0-alpha)

**Scope:** the Python SDK at `tytus_sdk/` plus the Rust shims
`tytus lope` and `tytus bridge`. Audited against the design in
`docs/DESIGN-TYTUS-LOPE-TEAMMATES.md` §8.

**Audit status:** v0.5.0-alpha. Fine for solo/dev use on a Traylinx-owned
machine. Review before shipping to external users — several items
(keychain migration, device-token rotation UX, audit log) are open.

---

## 1 · Threat model (what we're defending against)

| # | Threat | Severity |
|---|---|---|
| T1 | Malicious process on the Harvey-side machine injects fake pod→Harvey messages into the brain journal | High |
| T2 | Compromised pod writes malicious lope VERDICT blocks that subvert sprint outcomes | Medium |
| T3 | Rate-limit abuse — pod outbox floods brain journal | Medium |
| T4 | Device private-key theft — attacker impersonates our SDK from anywhere | High |
| T5 | Stale device entry remains in pod `paired.json` after uninstall | Low |
| T6 | Lope config tampering — malicious package adds a rogue `tytus-*` provider | Medium |
| T7 | Replay of a captured VERDICT block across negotiations | Low |

---

## 2 · Controls in place

### 2.1 Device identity (T4)

- **Keypair generation:** `cryptography.hazmat.primitives.asymmetric.ed25519.Ed25519PrivateKey.generate()` — CSPRNG-backed, no user input.
- **Storage:** `~/.tytus/openclaw/device.json`, file mode `0600`, parent dir `0700`. Enforced by `os.chmod(path, stat.S_IRUSR | stat.S_IWUSR)` in `identity.py:_create`.
- **Format:** JSON with `private_key_seed` (base64url-encoded raw 32-byte seed) + `public_key` + derived `device_id`. The seed is never logged, never transmitted, never printed by `identity` subcommand (only the public half).
- **device_id binding:** computed as `sha256(pub_raw).hex()` — matches server-side `deriveDeviceIdFromPublicKey` verbatim, so tampering the JSON to swap pubkey without updating `device_id` is rejected at the gateway.
- **Known gap:** Keychain migration still TODO (auto-memory item: "migrate bridge + device key to `keychain` crate"). v0.6 will move both to `security.framework`/`libsecret` via the existing `auth::keychain::Keychain` trait.

### 2.2 Gateway token (T1, T2, T4)

- Bootstrapped per-session from the Tytus forwarder's `302 Location: /?token=<T>` redirect. Never written to disk on the SDK side — lives only in the `OpenClawAdapter._conn.gateway_token` field.
- Rotating the pod's `gateway.auth.token` (via `tytus restart --pod NN`) immediately invalidates all in-flight SDK connections; the next adapter call re-fetches through the forwarder.
- Token is set on `auth.token` in the connect request; forwarder also injects `Authorization: Bearer <T>` on the WS upgrade HTTP request, so clients that don't pass token in connect still authenticate.

### 2.3 Device pre-pairing (T2, T4)

- Only a Tytus operator with `tytus exec` permission can add a device to `/app/workspace/.openclaw/devices/paired.json`. That path is already behind `tytus` → provider → A2A auth (`SCALESYS_SECRET` + Sentinel token on the provider side).
- Approved scopes are explicit: `["operator.read", "operator.write", "operator.admin"]`. Not `operator.pairing` (we don't need to pair other devices) and not `operator.approvals` (we don't bypass approvals).
- `install.py:_write_paired_json` writes atomically via base64-encoded heredoc + `mv` — no partial writes, no shell-injection surface (content is base64 which is safe inside single-quoted shell literals).
- **Known gap:** there is no audit log on the pod side recording which device was added/removed by which operator at which time. `install.py` should append to `/app/workspace/.openclaw/devices/audit.jsonl` in v0.6.

### 2.4 HarveyBridge token (T1)

- `~/.tytus/bridge.token` — `secrets.token_urlsafe(32)` (≈ 256 bits), mode `0600`, parent `0700`.
- Constant-time compare via `hmac.compare_digest` in `bridge_daemon._InboxHandler.do_POST` — no early-exit timing leak.
- Token is sent in header `X-Tytus-Bridge-Token`, never in URL/query string (so it doesn't land in shell history, browser history, or server access logs that record URLs).
- **Known gap:** flat-file storage. Keychain migration is the same v0.6 task as 2.1.

### 2.5 Network surface

- **HTTP listener:** binds to `127.0.0.1:18099` only. `BRIDGE_HOST = "127.0.0.1"` is not configurable without code change. No remote attack surface.
- **No TLS:** deliberate. Traffic is loopback only; TLS on loopback is theater and would add certificate management burden.
- **WireGuard tunnel:** already encrypted end-to-end (user ↔ pod). The SDK's WS connection runs over the existing tunnel; no plaintext pod traffic traverses any shared network.

### 2.6 Input validation

- `bridge_daemon` rejects non-JSON bodies (400), oversized bodies (413, 64 KiB limit), missing/mismatched token (401 via constant-time compare), unknown routes (404).
- `install.py` parses `paired.json` strictly — JSON decode failures raise `RuntimeError` before any write, preventing silent corruption of pod state.
- `lope_bridge.py` validates the VERDICT block structure (`_validate_verdict_body`): status enum, confidence float range [0,1], non-empty rationale. Malformed blocks are replaced with a defensive fallback so lope doesn't hang on a nonsense reply (T7 defense — a replay wouldn't carry a valid run's framing anyway).

### 2.7 Rate limits (T3)

- `_PodRateLimit`: 30 notifies / pod / hour, sliding window. Refills as old timestamps age out.
- Lope's own timeout + 3-round escalation caps runaway validator behaviour (T2 defense).
- **Known gap:** no rate-limit on the SDK → pod direction. A buggy caller can hammer `sessions.create` until OpenClaw's server-side limits kick in. Acceptable for solo use; should add client-side throttle for multi-tenant deployments.

### 2.8 Lifecycle guard (T5)

- `poll_pod_outbox` terminates whenever `_pod_is_connected` returns false (either the forwarder port stops accepting or `tytus status` drops the pod). Prevents zombie pollers leaking shell commands to a reallocated pod.
- `tytus lope uninstall` is idempotent and never fails hard when the pod is unreachable — just warns and still removes the lope config entry so lope stops routing there.

### 2.9 Lope config (T6)

- `~/.lope/config.json` — owned and written by the user; no SUID/SGID surface. Tytus entries are all in one list and clearly named `tytus-<agent>-<pod>`.
- `tytus lope list` surfaces exactly what's registered so the user can audit at any time.
- **Known gap:** no signature on the config. A malicious process that can write `~/.lope/config.json` could add a rogue provider. Defence: file permissions (0644 by default) + `chmod 600` if hosting the config on a shared machine.

---

## 3 · Residual risks / open items

| # | Item | Target |
|---|---|---|
| O1 | Bridge + device key not in OS keychain yet | v0.6.0 |
| O2 | No audit log for device pair add/remove on the pod | v0.6.0 |
| O3 | No rate limit on SDK→pod `sessions.send` | v0.7.0 |
| O4 | Pod-side `tytus_notify.py` helper not bundled in agent images | v0.6.0 (part of infra rebuild) |
| O5 | No signed-config check for `~/.lope/config.json` | v0.7.0 |
| O6 | Forwarder reverse-tunnel replaces outbox polling | v0.6.0 |

---

## 4 · Operator checklist

Before enabling a new teammate:

```bash
# 1. Confirm device identity is yours
tytus lope identity

# 2. Pair + register
tytus lope install --pod NN

# 3. Verify
tytus lope list                       # shows tytus-openclaw-NN
tytus lope ask --pod NN "hello"       # round-trip smoke test

# 4. Verify the pod's devices/paired.json reflects only your operators
tytus exec --pod NN "cat /app/workspace/.openclaw/devices/paired.json" \
    | jq 'keys'
```

Before shipping to a new user account:

- `ls -l ~/.tytus/openclaw/device.json` → must be `-rw-------`
- `ls -l ~/.tytus/bridge.token` → must be `-rw-------`
- `curl -sS http://127.0.0.1:18099/health` → must be 200 only from this host
- Access attempt with wrong token → must be 401

---

## 5 · Changelog

- **2026-04-20** — First edition. Audit covers Phase 1–4 implementation.

# SPRINT-TYTUS-LOPE-TEAMMATES-V0.5

## Origin: user request

### Phase 1: Ed25519 Identity & Secure Persistence
**Goal:** Implement Ed25519 device-key management for OpenClaw with secure file-system persistence.
**Criteria:**
*   `auth/src/device_identity.rs` implements Ed25519 key generation and signing for the OpenClaw protocol.
*   Keypair is persisted to `~/.tytus/openclaw/device.json` with strict `0600` permissions (Fix: Phase 1 requirement).
*   CLI provides `tytus auth device-key` to rotate or view the public identity without exposing secrets.
**Files:**
*   `auth/src/device_identity.rs` (New: Ed25519 logic and 0600 file I/O)
*   `auth/src/lib.rs` (Export device_identity)
*   `cli/src/main.rs` (Wire up `auth device-key` subcommand)
**Tests:**
*   `auth/tests/identity_persistence.rs`: Verify that `device.json` is created with `0600` and contains a valid Ed25519 seed.
*   **Verification Evidence:** `auth/src/sentinel.rs` already uses `fs::set_permissions` for token files; this pattern will be replicated for `device.json`.

### Phase 2: HarveyBridge Daemon & Keychain Auth
**Goal:** Implement the HarveyBridge listener on port 18099 with OS-keychain backed shared-secret authentication.
**Criteria:**
*   `tytus bridge run` starts a `tokio` HTTP server on `127.0.0.1:18099`.
*   The `bridge_token` is generated on first run and stored via `auth::keychain::Keychain` (Fix: Phase 2 requirement).
*   Incoming requests must provide `X-Tytus-Bridge-Token` matching the keychain value.
**Files:**
*   `cli/src/bridge.rs` (New: Bridge daemon and request handlers)
*   `cli/src/main.rs` (Add `bridge run` and `bridge token` subcommands)
**Tests:**
*   `cli/tests/bridge_auth.rs`: Verify 401 Unauthorized for missing/wrong token and 200 OK for keychain-matched token.
*   **Verification Evidence:** `auth/src/keychain.rs:18` provides the `SecureStorage` trait which abstracts `security.framework` on macOS and `libsecret` on Linux, ensuring process isolation for the bridge secret.

### Phase 3: Outbox Polling with Lifecycle Guard
**Goal:** Implement the JSONL polling loop for pod outboxes with automatic termination based on tunnel state.
**Criteria:**
*   Bridge daemon spawns a background task per pod to poll `/app/workspace/outbox.jsonl` using `exec` calls.
*   Polling loop checks `state.json` every iteration; if `tunnel_iface` is `null` or the pod entry is deleted, the task terminates (Fix: Phase 3 requirement).
*   Parsed JSONL lines are buffered and exposed via a local `/messages` GET endpoint for Harvey.
**Files:**
*   `cli/src/bridge.rs` (Implementation of `PollWorker` with lifecycle guard)
*   `cli/src/state.rs` (Integration with `PodState` watchers)
*   `pods/src/agent.rs` (Tail-optimized `exec` commands for outbox.jsonl)
**Tests:**
*   `cli/tests/polling_lifecycle.rs`: Verify that setting `tunnel_iface: null` in `state.json` immediately drops the polling thread.
*   **Verification Evidence:** `cli/src/state.rs:45` defines the `Pod` struct; the `PollWorker` will take a `Watch<State>` to react to `Pod` removals or interface drops.

### Phase 4: Lope Integration & Security Audit
**Goal:** Execute E2E validation with `lope` and verify the v0.5 security posture.
**Criteria:**
*   `lope negotiate` successfully uses a Tytus pod as a validator via the `tytus bridge`.
*   `docs/SECURITY-TEAMMATES.md` documents the device-key 0600 logic and keychain isolation.
*   Final audit confirms no "brain injection" vulnerabilities via the bridge-token path.
**Files:**
*   `docs/SECURITY-TEAMMATES.md` (New: v0.5 Security Audit)
*   `docs/DESIGN-TYTUS-LOPE-TEAMMATES.md` (Update status to IMPLEMENTED for v0.5 sections)
**Tests:**
*   `scripts/e2e-lope-teammate.sh`: Integration script simulating a full teammate negotiation loop.
*   **Verification Evidence:** `scripts/e2e-flows.sh` provides the template for multi-component testing; this will be extended to include `lope` CLI mocking.
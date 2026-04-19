# Tytus Agents as Lope Teammates — Full Design

**Status:** Phase 1 IMPLEMENTED (2026-04-20). Phases 2–5 IMPLEMENTED in v0.5.0-alpha.
**Owner:** Sebastian (via Harvey).
**Target:** v0.5.0 (Tytus CLI) + `lope v0.3.x` validator pool entry.
**Dependencies:** Tytus v0.4.0 (shipped), Lope `validators.py` contract (see §3), OpenClaw WS protocol v3 (`.specs/apps/openclaw/`), Hermes gateway v0.4.0.

## Phase 1 implementation notes (2026-04-20)

Several assumptions in the original design were wrong. What we actually
learned by reading the OpenClaw source and running against pod 02:

1. **Silent-local-pairing is unreachable over WireGuard.** The server's
   `isLocalDirectRequest` (src/gateway/auth.ts:115) requires
   `req.socket.remoteAddress` to be loopback. Packets arriving via the WG
   tunnel have the peer's WG IP (10.42.42.x), so `isLocalClient = false`
   and the silent-pair branch never fires. The browser works only because
   it was paired on a prior connect — not because of loopback semantics.

2. **Token-only connects get all scopes stripped.** In
   `server/ws-connection/message-handler.ts:547`, `clearUnboundScopes`
   fires whenever a connect has `!device && authMethod === "token"`.
   Result: `sessions.create` fails with `missing scope: operator.write`.
   **Device identity is mandatory** for any write scope.

3. **Device pre-pairing via `tytus exec` is the only path.** The SDK's
   Ed25519 public key gets injected into the pod's
   `/app/workspace/.openclaw/devices/paired.json` with approved
   `operator.{read,write,admin}` scopes, using the `tytus lope install`
   command (Phase 2).

4. **`deviceId` must be `sha256(pub_raw).hex()`** — full 64 hex chars.
   Truncating to 32 triggers `DEVICE_AUTH_DEVICE_ID_MISMATCH`. Matches
   `deriveDeviceIdFromPublicKey` in `src/infra/device-identity.ts:146`.

5. **`client.id` must be in the enum** (see `src/gateway/protocol/
   client-info.ts:1`). `gateway-client` + `client.mode="backend"` is the
   right match for a non-browser, non-native-app SDK client.

6. **`thinking` is a required string field** on `sessions.send`, not
   nullable. `"off"` is the value for disable-reasoning-mode.

7. **Fresh session per ask.** Reusing `key="main"` binds to the pod's
   long-running `agent:main:main` with the full agent-orchestration loop
   (tool-calls, skills, memory). For validator use we want a clean slate —
   unique `tytus-lope-<uuid>` per ask. Label must also be unique (server
   enforces `label already in use`).

8. **Terminal signal is `event:"chat"` with `state:"final"`** for our
   `runId`, NOT `session.message.status`. The `session.message` events
   carry content deltas; `chat{state:final}` is the "this turn is over"
   marker.

The design below has been left intact. The "Open questions" in §12 are
now mostly answered by the implementation; a §14 Implementation Reality
below records the operational surface actually shipped.

---

## 1 · Goal

Make every agent running in a Tytus pod (today: OpenClaw, Hermes; tomorrow: anything) a first-class, plug-and-play teammate in Lope's negotiation pool — with **bidirectional** communication to Harvey (or any local AI), not just a one-way RPC.

A "teammate" means:
1. Lope can pick it as drafter OR validator in `lope negotiate` runs.
2. The agent can *proactively* message Harvey (journal entries, brain events, Telegram nudges) from inside its pod.
3. New agent types plug in without patching Lope, Tytus, or Harvey.

Non-goals (explicit):
- Not a replacement for `tytus chat` / `tytus env` / MCP tools — those stay.
- Not a general-purpose multi-agent framework (no Swarm orchestration, no A2A protocol beyond what exists in Tytus). This is *specifically* lope + Harvey integration.
- Not cross-user. One Tytus account = one pool.

---

## 2 · Architecture

Three layers, each independently useful.

```
┌─────────────────────────────────────────────────────────────────┐
│ Lope (~/.lope)                       Harvey (Claude Code etc.)  │
│  ├─ cli.py                            ├─ harvey-mcp tools       │
│  ├─ validators.py                     └─ journal / superbrain   │
│  └─ config.json ─┐                        ▲                      │
│                  │                        │                      │
│  ┌───────────────┴─────────────┐   ┌──────┴────────────────┐    │
│  │ LAYER 3: LopeBridge          │   │ LAYER 3: HarveyBridge │    │
│  │ (~/.lope/validators/         │   │ (local HTTP daemon,   │    │
│  │  tytus_validator.py)         │   │  127.0.0.1:18099)     │    │
│  │  implements lope.Validator   │   │  POST /inbox → brain  │    │
│  └──────────────┬───────────────┘   └──────────┬────────────┘    │
│                 │                               │                │
│  ┌──────────────┴───────────────────────────────┴────────────┐  │
│  │ LAYER 2: TytusAgentSDK (Python + Rust)                    │  │
│  │  AgentAdapter trait/protocol:                              │  │
│  │    ask(prompt) -> str                                      │  │
│  │    stream(prompt) -> Iterator[chunk]                       │  │
│  │    identify() -> AgentIdentity                             │  │
│  │    notify(message, typ) -> None   # agent → human          │  │
│  │  Registry: scans ~/.tytus/agents/*.toml + tytus status     │  │
│  └──────────────┬────────────────────────────────────────────┘  │
│                 │                                                │
│  ┌──────────────┴────────────────────────────────────────────┐  │
│  │ LAYER 1: Protocol adapters (per-agent)                    │  │
│  │   OpenClawAdapter  — WS v3 + Ed25519 (localhost:18700+N)  │  │
│  │   HermesAdapter    — REST + SSE (localhost:18700+N/v1)    │  │
│  │   SwitchAIAdapter  — OpenAI REST (10.42.42.1:18080/v1)    │  │
│  │   CustomAdapter    — shell hook, loaded from manifest      │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Why three layers

- **Layer 1 (adapters)** is where each agent's weird protocol lives — WS Ed25519, SSE, OpenAI REST. Isolated here so adding a new agent means one file.
- **Layer 2 (SDK)** is the reusable kernel — discovery, lifecycle, a uniform `ask`/`stream`/`notify` surface. Any consumer (lope, harvey, future cursor plugin, CI gate) uses the same API.
- **Layer 3 (bridges)** is the thin glue to a specific external system. `LopeBridge` renders adapter responses as VERDICT blocks; `HarveyBridge` exposes a POST endpoint and writes to the brain. Trivial once Layer 2 exists.

---

## 3 · Lope contract recap (from investigation)

```python
# ~/.lope/lope/validators.py:168–221
class Validator(ABC):
    @property
    @abstractmethod
    def name(self) -> str: ...

    @abstractmethod
    def validate(self, prompt: str, timeout: int = 180) -> ValidatorResult: ...

    def available(self) -> bool:
        return True

    def generate(self, prompt: str, timeout: int = 180) -> str:
        raise NotImplementedError
```

`ValidatorResult` must be either a parsed `---VERDICT---{...}---END---` JSON block (schema: `status`, `confidence`, `rationale`, `required_fixes`, `nice_to_have`) or an `INFRA_ERROR` fallback.

Config at `~/.lope/config.json`:
```json
{
  "validators": ["claude", "gemini", "pi", "tytus-openclaw-02", "tytus-hermes-03"],
  "primary": "claude",
  "providers": [
    {
      "name": "tytus-openclaw-02",
      "type": "subprocess",
      "command": ["python3", "-m", "tytus_sdk.lope_bridge", "--pod", "02"]
    }
  ]
}
```

`GenericSubprocessValidator` already exists — our LopeBridge just needs to emit VERDICT blocks on stdout.

---

## 4 · Layer 1 — Protocol adapters (Python)

### 4.1 `AgentAdapter` protocol

```python
# tytus_sdk/adapter.py
from dataclasses import dataclass
from typing import Protocol, Iterator, Literal

@dataclass
class AgentIdentity:
    pod_id: str                  # "02"
    agent_type: str              # "openclaw" | "hermes" | "switchai"
    display_name: str            # "TytusOpenClaw (pod 02)"
    capabilities: list[str]      # ["chat", "tools", "stream", "notify_back"]
    model: str | None            # best-effort introspection
    stable_endpoint: str         # "http://10.42.42.1:18080/v1" or "ws://..."

@dataclass
class AgentMessage:
    role: Literal["system", "user", "assistant"]
    content: str

class AgentAdapter(Protocol):
    def identify(self) -> AgentIdentity: ...

    def ask(
        self,
        messages: list[AgentMessage],
        timeout_s: int = 120,
    ) -> str:
        """Blocking single-turn call. Returns full assistant content."""

    def stream(
        self,
        messages: list[AgentMessage],
        timeout_s: int = 300,
    ) -> Iterator[str]:
        """Token-chunk stream. Default impl: call ask() and yield once."""

    def notify(
        self,
        message: str,
        typ: Literal["info", "warn", "decision", "ask"] = "info",
        details: dict | None = None,
    ) -> None:
        """
        Reverse channel: agent → human. Delivered via HarveyBridge.
        Non-blocking fire-and-forget.
        """

    def close(self) -> None: ...
```

### 4.2 `OpenClawAdapter`

Protocol: WebSocket v3 to `ws://localhost:18700+N/` (existing forwarder).

Device identity: reuse `~/.tytus/openclaw/device.json` (generated once on first `tytus lope install --agent openclaw`; same keypair across pods is fine — pods are trust-equivalent within one Tytus account).

Flow (from `.specs/apps/openclaw/src/gateway/server.auth.shared.ts`):

```python
# tytus_sdk/adapters/openclaw.py (sketch)
class OpenClawAdapter:
    def __init__(self, pod_id: str, port: int, device_key: Ed25519PrivateKey, gateway_token: str):
        self.ws_url = f"ws://localhost:{port}/"
        self.pod_id = pod_id
        self.device_key = device_key
        self.gateway_token = gateway_token   # injected by Tytus forwarder
        self._session_key = "tytus-lope"
        self._ws = None

    async def _connect(self):
        self._ws = await websockets.connect(self.ws_url)
        # wait for connect.challenge event
        frame = json.loads(await self._ws.recv())
        assert frame["type"] == "event" and frame["event"] == "connect.challenge"
        nonce = frame["payload"]["nonce"]
        # build v2-canonical payload
        signed_at = int(time.time() * 1000)
        payload = "|".join([
            "v2",
            self.device_id, "tytus-lope", "node",
            "operator", "operator.admin,operator.read,operator.write",
            str(signed_at), self.gateway_token, nonce,
        ])
        sig = self.device_key.sign(payload.encode())
        await self._ws.send(json.dumps({
            "type": "req", "id": uuid4().hex, "method": "connect",
            "params": {
                "minProtocol": 3, "maxProtocol": 3,
                "client": {"id": "tytus-lope", "version": __version__, "platform": "node", "mode": "test"},
                "role": "operator",
                "scopes": ["operator.admin", "operator.read", "operator.write"],
                "device": {
                    "id": self.device_id,
                    "publicKey": b64url(self.device_key.public_key().public_bytes_raw()),
                    "signature": b64url(sig),
                    "signedAt": signed_at,
                    "nonce": nonce,
                },
                "auth": {"token": self.gateway_token, "deviceToken": None},
                "caps": ["tool-events"],
                "locale": "en",
                "userAgent": f"tytus-sdk/{__version__}",
            },
        }))
        # await res, create session, subscribe, ready
        ...

    async def ask_async(self, messages, timeout_s):
        if not self._ws:
            await self._connect()
            await self._ensure_session()
        # sessions.send, collect all delta events, stop on state=final
        ...
```

Notes:
- Device keypair is generated ONCE, stored at `~/.tytus/openclaw/device.json` with 0600. Identity persists across pod revoke/reallocate so the Lope validator's track record in Brain is stable.
- The gateway token comes from Tytus `state.json` (the forwarder already has it — SDK reads the same state).
- `notify()` for OpenClaw: since this adapter runs on Harvey's machine, `notify` is cheap — directly POST to HarveyBridge without going through the pod.

### 4.3 `HermesAdapter`

Protocol: OpenAI-compatible REST at `http://localhost:18700+N/v1/chat/completions`. Forwarder already injects `API_SERVER_KEY`.

```python
class HermesAdapter:
    def __init__(self, pod_id: str, port: int):
        self.base = f"http://localhost:{port}/v1"

    def ask(self, messages, timeout_s):
        r = httpx.post(
            f"{self.base}/chat/completions",
            json={"model": "hermes-default", "messages": [asdict(m) for m in messages]},
            timeout=timeout_s,
        )
        r.raise_for_status()
        return r.json()["choices"][0]["message"]["content"]
```

Trivial. Streaming = SSE + `stream: true`.

### 4.4 `SwitchAIAdapter` (baseline — always available)

Fallback that works without an agent container: talks straight to the per-droplet OpenAI-compat gateway at `http://10.42.42.1:18080/v1`. Identity: `TytusRaw (pod NN)`. Capabilities: `["chat"]`, not `["tools", "notify_back"]`. Useful as a sanity check or when the agent is down.

### 4.5 Third-party adapters via TOML manifest

A user or org can plug a custom agent without writing Python:

```toml
# ~/.tytus/agents/my-custom.toml
name = "my-rag-agent"
display_name = "Custom RAG on Pod 04"
pod_id = "04"
protocol = "http"           # http | openai_rest | websocket_jsonrpc | shell
endpoint = "http://localhost:18704/ask"
method = "POST"
auth.type = "bearer"
auth.token_from_state = true    # read from tytus state.json
request_template = '{"question": "{{prompt}}"}'
response_jsonpath = "$.answer"
capabilities = ["chat"]
```

Loader: `tytus_sdk.adapters.manifest.load_manifest(path) -> AgentAdapter`. Generic HTTP adapter implements the Protocol using the template + jsonpath. Shell adapter just pipes stdin/stdout.

---

## 5 · Layer 2 — TytusAgentSDK (discovery + registry)

### 5.1 Registry

```python
# tytus_sdk/registry.py
def discover() -> list[AgentAdapter]:
    """
    Discovery order:
      1. Parse `tytus status --json` → one adapter per (pod, agent_type)
      2. Scan ~/.tytus/agents/*.toml for custom manifests
      3. Merge, dedupe by (pod_id, agent_type)
    Returns only adapters whose pod is currently connected
    (tunnel_iface != null) unless include_offline=True.
    """
```

Runs `tytus status --json` via subprocess — no new dependency on internal Tytus APIs. If `tytus` binary is missing, registry returns empty list with a one-line warning.

### 5.2 Lifecycle

Adapters are **lazy**: created when first used, not at discovery. `OpenClawAdapter._connect()` runs on first `ask()`. The registry just holds factories.

Adapter connections are pooled — subsequent `ask()` calls reuse the WS. A background keepalive (ping every 20s) prevents the forwarder's transient-probe log from firing. On any unrecoverable error, the adapter is marked dead and next `ask()` reconnects from scratch.

### 5.3 CLI

```
tytus lope list                     # what's discoverable
tytus lope install [--agent X]      # register adapters in ~/.lope/config.json
tytus lope uninstall
tytus lope ask --agent openclaw "..."  # manual single-shot (debug)
tytus lope doctor                   # end-to-end reachability probe
```

Implemented in Rust (`tytus-cli/cli/src/lope.rs`) but delegates actual asking to the Python SDK via subprocess — so the SDK stays the source of truth for protocol work.

---

## 6 · Layer 3a — LopeBridge (adapter → lope validator)

```python
# tytus_sdk/lope_bridge.py
import sys, json
from tytus_sdk.registry import discover

def main(argv):
    # argv: --pod 02 [--agent openclaw]
    # stdin: the lope review prompt
    # stdout: raw assistant reply INCLUDING a ---VERDICT---...---END--- block
    pod, agent = parse_args(argv)
    adapter = pick(discover(), pod, agent)
    prompt = sys.stdin.read()
    sys_prompt = VERDICT_SYSTEM_PROMPT     # demands JSON verdict block
    reply = adapter.ask([
        AgentMessage("system", sys_prompt),
        AgentMessage("user", prompt),
    ], timeout_s=int(os.environ.get("LOPE_TIMEOUT", 180)))
    # validate that the reply contains a VERDICT block, else wrap
    if "---VERDICT---" not in reply:
        reply = wrap_in_fallback_verdict(reply)
    sys.stdout.write(reply)
```

Registered as a `subprocess` provider in `~/.lope/config.json`. That's it — Lope already handles the rest (VERDICT parsing, confidence gating, pool routing).

### 6.1 System prompt that demands VERDICT

Identical to what Lope injects for hardcoded validators (Claude/Gemini), so Tytus agents validate on the same rubric:

> You are a code reviewer for an engineering sprint. Read the artifacts listed below. Evaluate against each criterion. Return your final decision in a `---VERDICT---\n{...json...}\n---END---` block with fields: `status` (PASS|NEEDS_FIX|FAIL), `confidence` (0.0-1.0), `rationale`, `required_fixes`, `nice_to_have`.

Copy verbatim from `~/.lope/lope/executor.py:288` so rubrics stay in lockstep.

### 6.2 Generate (drafting) mode

OpenClaw can draft too — it has full chat capability. `generate()` is just `ask()` without the VERDICT wrapper and with a drafter-oriented system prompt. Hermes can draft. SwitchAI can draft (it's literally an LLM). So all three can be `"primary"` in Lope.

---

## 7 · Layer 3b — HarveyBridge (agent → human)

This is the **bidirectional** piece. An agent inside a pod (OpenClaw tool, Hermes tool, arbitrary script) needs a way to reach Harvey.

### 7.1 Bridge daemon

New Rust subcommand: `tytus bridge run` (runs as user launchd service).

- Binds `127.0.0.1:18099` (config: `TYTUS_BRIDGE_PORT`).
- Accepts `POST /inbox` with shared-secret `X-Tytus-Bridge-Token` header.
- Writes entries to:
  - Today's brain journal: `$MAKAKOO_HOME/data/Brain/journals/YYYY_MM_DD.md` (one line per notify)
  - Superbrain event store: invokes `$MAKAKOO_HOME/harvey-os/core/superbrain/ingest.py`
  - Optional: fires a macOS notification via `terminal-notifier` for `typ=ask`
- Rate-limited: 30 notifies / pod / hour. Config-overridable.

Shared secret generated on install, written to both `~/.tytus/bridge.token` (local, 0600) and the pod's agent workspace (`/app/workspace/.tytus/bridge.token` via `tytus exec`). Pod code reads the token + bridge URL from that file.

### 7.2 Pod-side client

Because pods are network-isolated from Harvey's machine (WireGuard is one-way for our purposes: Harvey → pod), the bridge isn't directly reachable from inside the pod. Three options, ranked:

**Option 1 (chosen): Forwarder reverse tunnel.** The existing Tytus forwarder already bridges localhost:18700+N → pod:3000. We add a second route in the *opposite* direction: pod-origin requests to `http://tytus-host.local:18099` are funneled through the WS tunnel back to Harvey's local bridge. Implementation: the forwarder's pod-side counterpart listens on `10.X.Y.1:18099` inside the pod; WS frames tagged `direction=reverse` carry HTTP-over-WS back to the forwarder, which dispatches to the local bridge daemon.

**Option 2:** Pod writes to a file that Harvey polls via `tytus exec --pod NN "cat /app/workspace/.harvey-outbox.jsonl"`. Simpler but lossy (requires polling). Good as a fallback.

**Option 3:** Pod calls out to Scalesys/Provider which forwards via webhook to Harvey. Requires public webhook URL for Harvey. Ruled out for v1 — privacy + NAT friction.

Decision: **ship Option 2 in v0.5.0** (minimal, robust, no forwarder changes), **promote to Option 1 in v0.6.0** after the reverse-tunnel pattern lands for other features (config sync, tool-call streams).

### 7.3 Pod SDK snippet (in OpenClaw tool)

```python
# pod-side helper (bundled with tytus image)
# /app/workspace/tytus_notify.py
def notify_harvey(message, typ="info", details=None):
    import json, pathlib, datetime
    path = pathlib.Path("/app/workspace/.harvey-outbox.jsonl")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.open("a").write(json.dumps({
        "ts": datetime.datetime.utcnow().isoformat() + "Z",
        "pod_id": os.environ["TYTUS_POD_ID"],
        "agent": os.environ.get("TYTUS_AGENT_TYPE", "unknown"),
        "typ": typ,
        "message": message,
        "details": details or {},
    }) + "\n")
```

Harvey-side cron-like watcher inside `tytus bridge run` drains the file every 10s via `tytus exec` (or direct WS read in v0.6).

---

## 8 · Security model

| Concern | Mitigation |
|---|---|
| Arbitrary pod injects into Harvey's brain | Shared-secret token in `X-Tytus-Bridge-Token`, rotated by `tytus lope reset-token` |
| Malicious pod drafts a lope PASS with harmful `required_fixes` | VERDICT block is parsed but actions always confirmed by Harvey before execution (same as other lope validators) |
| Rate-limit abuse | 30 notifies/pod/hour, 10 asks/pod/min (configurable) |
| Device key compromise | Keypair stored 0600 under `~/.tytus/openclaw/`. Rotation: `tytus lope reset-keys` reissues + wipes OpenClaw's known_devices. |
| Cleartext over WG tunnel | WG is already encrypted; localhost forwarder is loopback-only. No TLS needed. |
| DoS from runaway validator loop | Lope's 3-round max + timeout kills unresponsive adapters |
| Adapter reads Harvey's journal (leak) | No — adapters are OUT-bound only. Pods cannot read Harvey's brain. |
| Pod exec hook abused as RCE | `tytus exec` already requires Tytus auth + A2A; bridge inherits that. |

---

## 9 · Test strategy

Four layers of tests. All hermetic where possible.

### 9.1 Unit
- `tests/adapters/test_openclaw_adapter.py` — mock WS server returns canned challenge, canned session replies. Verifies v2-canonical payload formation byte-for-byte.
- `tests/adapters/test_hermes_adapter.py` — httpx `respx` mock.
- `tests/adapters/test_manifest.py` — round-trip TOML.

### 9.2 Integration (requires live pod)
- `scripts/lope-e2e.sh` — spins up a lope negotiation run with `validators=["claude", "tytus-openclaw-02"]` and asserts:
  - Both validators emit valid VERDICT blocks within timeout
  - Disagreement triggers a round-2 redraft
  - 3-round escalation fires if consensus doesn't emerge

### 9.3 Bridge (requires pod + Harvey box)
- `scripts/bridge-e2e.sh` — pod writes a synthetic notify, Harvey-side watcher picks it up within 30s, it appears in today's journal + in `superbrain search`.

### 9.4 Simulation (no pod needed)
- New flag `TYTUS_SDK_FAKE=1` makes `registry.discover()` return a deterministic `FakeAdapter` that replays canned responses. Lets lope devs test pool logic without Tytus installed.

Acceptance bar: E2E suite green in `scripts/e2e-flows.sh` (extending the existing 35-flow harness to 40 flows covering the LOPE/HARVEYBRIDGE categories).

---

## 10 · Rollout phases

### Phase 1 — v0.5.0-alpha (week 1)
- Layer 1: OpenClawAdapter (WS+Ed25519), HermesAdapter (REST), SwitchAIAdapter (REST).
- Layer 2: Python SDK scaffold, `discover()`, `tytus lope list`, `tytus lope ask`.
- Layer 3a: LopeBridge subprocess entrypoint.
- Manual `~/.lope/config.json` edit to register. One-line install tested.

### Phase 2 — v0.5.0-beta (week 2)
- `tytus lope install` writes `~/.lope/config.json` automatically.
- Device key generation + rotation.
- Custom TOML manifests working end-to-end.
- Unit test coverage ≥ 80% on SDK.

### Phase 3 — v0.5.0 (week 3)
- HarveyBridge daemon (`tytus bridge run`).
- Outbox polling (Option 2).
- Journal + superbrain write-through.
- `tytus bridge doctor` diagnostic.
- E2E suite expanded.

### Phase 4 — v0.6.0 (+2 weeks)
- Reverse-tunnel forwarder support (Option 1 for HarveyBridge).
- Streaming `stream()` end-to-end through lope (show live drafting in lope logs).
- Claude Code / Cursor plugins that consume the same SDK.

### Phase 5 — v0.7.0 (+month)
- Multi-user pools: when an org shares a pod, multiple humans see the same TytusOpenClaw in their respective lope setups. Requires Scalesys user-scoped bridge tokens.
- Per-domain rubrics: TytusOpenClaw specialised for engineering, another OpenClaw instance for business — surfaced as distinct validators.

---

## 11 · File layout

```
services/tytus-cli/
├── cli/src/lope.rs            # new: `tytus lope` subcommands
├── cli/src/bridge.rs          # new: `tytus bridge run`
├── tytus_sdk/                 # new: Python SDK (distributed via pip or bundled in install.sh)
│   ├── __init__.py
│   ├── adapter.py             # Protocol + dataclasses
│   ├── registry.py
│   ├── lope_bridge.py
│   ├── harvey_bridge.py       # HarveyBridge client
│   └── adapters/
│       ├── openclaw.py
│       ├── hermes.py
│       ├── switchai.py
│       └── manifest.py        # generic TOML-driven
├── tytus_sdk/tests/
└── docs/
    ├── DESIGN-TYTUS-LOPE-TEAMMATES.md   ← this file
    └── LOPE-BRIDGE-RUNBOOK.md           # ops + troubleshooting
```

Python SDK can live inside the tytus-cli repo (keeps versions locked) and publish to PyPI as `tytus-sdk`. Rust side only calls it via subprocess for now — avoids Python↔Rust FFI churn.

---

## 12 · Open questions (for the lope-negotiate pass)

1. **Should the SDK be Rust-native eventually?** Lope is Python, so subprocess is fine. But a Claude Code MCP extension using the same SDK would want Rust-native. Propose: keep Python as SoT for v0.5–0.6, add Rust crate via `pyo3` wrapper in v0.7.
2. **Device key per-pod or per-account?** Current design: per-account. Per-pod is more granular but bloats `known_devices`. Stay per-account unless OpenClaw upstream forces otherwise.
3. **Do Hermes dashboards also want WS protocol support, or is REST enough?** REST is enough for `ask`/`stream`; dashboards don't issue validator verdicts. Skip WS for Hermes in v0.5.
4. **Outbox format**: JSONL vs SQLite vs protobuf? JSONL is grep-able and trivial. Stick with JSONL.
5. **Should HarveyBridge also accept requests from non-Tytus sources** (e.g., Telegram → Harvey)? Out of scope here, but the `POST /inbox` contract is designed to allow it.

---

## 13 · Done-done definition

- `lope negotiate --validators claude,tytus-openclaw-02 "design a Kafka retry"` produces a valid 3-round sprint doc.
- OpenClaw-in-pod writes `notify_harvey("sprint ready for review")` and the line appears in today's brain journal + `superbrain search "sprint ready"` returns it.
- Fresh laptop: `curl ... | bash && tytus setup && tytus lope install` produces a working TytusOpenClaw teammate in under 5 minutes.
- `scripts/e2e-flows.sh` passes 40/40 including 5 new LOPE/BRIDGE flows.

---

**Next step after this doc:** run `/lope-negotiate "Tytus agents as lope teammates per docs/DESIGN-TYTUS-LOPE-TEAMMATES.md"` to get drafter + 3 validators to critique this plan before any Rust/Python lands.

---

## 14 · Implementation Reality (shipped in v0.5.0-alpha)

### Shipped command surface

```bash
tytus lope ask --pod 02 "prompt"           # direct WS ask against OpenClaw
tytus lope install --pod 02 [--agent X]    # pair device + register lope provider
tytus lope uninstall --pod 02
tytus lope list                            # show registered tytus-* validators
tytus lope identity                        # print Ed25519 device id + pubkey
tytus lope lope_validate --pod 02          # VERDICT-emitting subprocess (for lope)

tytus bridge run                           # HTTP listener + per-pod pollers
tytus bridge status                        # health check
tytus bridge rotate-token
tytus bridge test --pod 02 "message"       # synthetic notify → journal
```

### Files actually shipped

```
services/tytus-cli/tytus_sdk/
├── __init__.py              version + surface
├── __main__.py              → cli.main()
├── adapter.py               AgentAdapter protocol + dataclasses
├── identity.py              Ed25519 keypair, 0600 persistence
├── install.py               device pairing on pod + ~/.lope/config.json
├── lope_bridge.py           VERDICT-wrapper + fallback block
├── bridge_daemon.py         HTTP inbox + outbox pollers + rate limit
├── cli.py                   argparse dispatcher
└── adapters/
    ├── __init__.py
    └── openclaw.py          WS v3 + Ed25519 + fresh-session-per-ask

services/tytus-cli/cli/src/main.rs
  Commands::Lope   — pass-through to `python3 -m tytus_sdk`
  Commands::Bridge — pass-through to `python3 -m tytus_sdk bridge`
```

### Lope provider format (as actually written)

```json
{
  "providers": [
    {
      "name": "tytus-openclaw-02",
      "type": "subprocess",
      "command": [
        "python3", "-m", "tytus_sdk", "lope_validate",
        "--pod", "02", "--agent", "openclaw", "{prompt}"
      ]
    }
  ],
  "validators": ["claude", "gemini", "pi", "qwen", "tytus-openclaw-02"]
}
```

### Pod-side outbox contract (what agents write)

```jsonl
{"ts":"2026-04-20T00:15:00Z","pod_id":"02","agent":"openclaw-agent","typ":"info","message":"human-readable text","details":{...}}
```

Path on the pod: `/app/workspace/.harvey-outbox.jsonl`. Append-only, newline-
delimited JSON. The HarveyBridge poller tails the file via `tail -c +N`
over `tytus exec` every 10 s and persists the drained byte-offset at
`~/.tytus/bridge/outbox-<pod>.state`.

### Lifecycle guard

Pollers terminate cleanly when:
- the pod's forwarder port (18700+N) stops accepting TCP (`_pod_is_connected`)
- `tytus status` removes the pod entirely
- the bridge daemon is shut down (SIGINT)

### Known gaps (deferred to v0.6.0)

- `HermesAdapter` REST path — not shipped; design still valid, just not coded.
- Forwarder reverse-tunnel (Option 1 in §7.2) — still using JSONL outbox
  polling. Upgrade when other features need a reverse tunnel anyway.
- Keychain-backed bridge token — still flat file at `~/.tytus/bridge.token`
  (0600). Rust `keychain` crate migration planned alongside `tytus auth
  device-key` rotation command.
- Custom TOML manifest adapters — not implemented; `manifest.py` is a stub.
- `notify()` method on `OpenClawAdapter` — stubbed no-op. Pod-side agents
  must call the outbox helper directly (pod-side SDK bundle is out-of-scope).

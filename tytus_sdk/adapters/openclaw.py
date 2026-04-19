"""OpenClaw adapter — speaks the gateway WS v3 protocol via the local Tytus
forwarder (ws://localhost:18700+N/).

Handshake (v2-canonical Ed25519):
  1. HTTP GET http://localhost:{port}/  → 302 Location: /?token=<T>   (forwarder
     emits this on every pairing; we grab T as the gateway token)
  2. WS upgrade to ws://localhost:{port}/
  3. Receive event `connect.challenge` with `payload.nonce`
  4. Build canonical payload:
        v2|deviceId|clientId|clientMode|role|scopes.join(',')|signedAtMs|token|nonce
     Sign with Ed25519; send `connect` req with device fields + auth.token=T
  5. Receive `res` with ok:true → connection is authenticated
  6. sessions.create + sessions.messages.subscribe + sessions.send
  7. Collect `session.message` events until the final one arrives, then return.

Minimal, blocking, single-turn. Streaming and tool events come in Phase 4.
"""

from __future__ import annotations

import asyncio
import json
import logging
import re
import time
import urllib.parse
import uuid
from dataclasses import dataclass
from typing import Iterator

import httpx
import websockets

from tytus_sdk.adapter import AgentIdentity, AgentMessage
from tytus_sdk.identity import DeviceIdentity, load_or_create_identity

log = logging.getLogger(__name__)

PROTOCOL_VERSION = 3
OPERATOR_SCOPES = ["operator.admin", "operator.read", "operator.write"]
# We identify as the control UI (same category the browser uses) because
# only that id + loopback Origin/Host + valid gateway token triggers silent-
# local-pairing on the server (shouldSkipControlUiPairing in
# server/ws-connection/message-handler.ts). Any other client id lands us in
# the NOT_PAIRED branch and requires explicit user-side pair approval.
CLIENT_ID = "gateway-client"  # generic non-control-UI client with device identity
CLIENT_VERSION = "0.1.0"
CLIENT_PLATFORM = "linux"
CLIENT_MODE = "backend"
# Session key prefix — one fresh session per ask() so we never inherit
# a prior conversation's agent state (observed: `main` maps to `agent:main:main`
# which carries full agent history + tool-call loops).
SESSION_KEY_PREFIX = "tytus-lope"


@dataclass
class _ConnectInfo:
    ws: websockets.WebSocketClientProtocol
    gateway_token: str
    canonical_session_key: str


class OpenClawAdapter:
    """Single-pod OpenClaw client. One instance == one live WS connection."""

    def __init__(self, pod_id: str, forwarder_port: int, identity: DeviceIdentity | None = None):
        self.pod_id = pod_id
        self.port = forwarder_port
        self.identity = identity or load_or_create_identity()
        self._conn: _ConnectInfo | None = None
        self._loop: asyncio.AbstractEventLoop | None = None

    # ── Public surface ─────────────────────────────────────────────────

    def identify(self) -> AgentIdentity:
        return AgentIdentity(
            pod_id=self.pod_id,
            agent_type="openclaw",
            display_name=f"TytusOpenClaw (pod {self.pod_id})",
            capabilities=["chat", "tools"],
            model=None,
            stable_endpoint=f"ws://localhost:{self.port}/",
        )

    def ask(self, messages: list[AgentMessage], timeout_s: int = 120) -> str:
        return self._run(self._ask_async(messages, timeout_s))

    def stream(self, messages: list[AgentMessage], timeout_s: int = 300) -> Iterator[str]:
        yield self.ask(messages, timeout_s=timeout_s)

    def close(self) -> None:
        if self._conn and self._loop:
            try:
                self._run(self._conn.ws.close())
            except Exception:
                pass
        self._conn = None

    # ── Internals ──────────────────────────────────────────────────────

    def _run(self, coro):
        if self._loop is None:
            self._loop = asyncio.new_event_loop()
        return self._loop.run_until_complete(coro)

    async def _ask_async(self, messages: list[AgentMessage], timeout_s: int) -> str:
        if self._conn is None:
            await self._connect(timeout_s=timeout_s)

        assert self._conn is not None
        conn = self._conn

        # Fresh session per ask — avoids inheriting the pod's long-running
        # agent:main:main conversation state (observed 125-entry transcript).
        canonical = await self._create_fresh_session(conn.ws)
        conn.canonical_session_key = canonical

        # Flatten messages to a single prompt (OpenClaw sessions.send takes one
        # message string). System prompt is prepended with a divider; prior
        # assistant/user pairs become labeled lines. This is good enough for
        # v0.5 single-turn validation; v0.6 will use proper session state.
        flattened = _flatten_messages(messages)

        # Send + await. sessions.send RPC response returns when the run starts;
        # the actual assistant output arrives as session.message events.
        send_id = uuid.uuid4().hex
        await conn.ws.send(json.dumps({
            "type": "req",
            "id": send_id,
            "method": "sessions.send",
            "params": {
                "key": conn.canonical_session_key,
                "message": flattened,
                # thinking is `Type.String()` (not optional) — "off" disables
                # extended-thinking/reasoning mode. Valid: "off" | "low" | "medium" | "high"
                "thinking": "off",
                "attachments": [],
                "idempotencyKey": send_id,
            },
        }))

        return await asyncio.wait_for(
            _collect_assistant_reply(conn.ws, conn.canonical_session_key, send_id),
            timeout=timeout_s,
        )

    async def _connect(self, timeout_s: int) -> None:
        gateway_token = await self._bootstrap_token()
        log.debug("gateway token bootstrapped: %s…", gateway_token[:8])

        # Silent-local-pairing needs loopback Origin; python's websockets
        # omits Origin by default for ws:// schemes, so we set it explicitly.
        origin = f"http://localhost:{self.port}"
        ws = await websockets.connect(
            f"ws://localhost:{self.port}/",
            open_timeout=10,
            origin=origin,
        )
        try:
            # Step 1: wait for connect.challenge event
            nonce = await _await_event(ws, "connect.challenge", timeout_s=10)
            signed_at = int(time.time() * 1000)
            payload = "|".join([
                "v2",
                self.identity.device_id,
                CLIENT_ID,
                CLIENT_MODE,
                "operator",
                ",".join(OPERATOR_SCOPES),
                str(signed_at),
                gateway_token,
                nonce,
            ])
            signature = self.identity.sign_b64url(payload)

            connect_id = uuid.uuid4().hex
            # Device identity is REQUIRED for write scopes. Token-only connects
            # get all scopes stripped (message-handler.ts:547 `clearUnboundScopes`).
            # Our device must be pre-paired on the pod via `tytus lope install`
            # which writes our public key into /app/workspace/.openclaw/devices/
            # paired.json with approved operator.{read,write,admin} scopes.
            connect_params = {
                "minProtocol": PROTOCOL_VERSION,
                "maxProtocol": PROTOCOL_VERSION,
                "client": {
                    "id": CLIENT_ID,
                    "version": CLIENT_VERSION,
                    "platform": CLIENT_PLATFORM,
                    "mode": CLIENT_MODE,
                },
                "role": "operator",
                "scopes": OPERATOR_SCOPES,
                "device": {
                    "id": self.identity.device_id,
                    "publicKey": self.identity.public_key_b64url,
                    "signature": signature,
                    "signedAt": signed_at,
                    "nonce": nonce,
                },
                "auth": {"token": gateway_token},
                "caps": ["tool-events"],
                "locale": "en",
                "userAgent": f"tytus-sdk/{CLIENT_VERSION}",
            }
            await ws.send(json.dumps({
                "type": "req",
                "id": connect_id,
                "method": "connect",
                "params": connect_params,
            }))

            connect_res = await _await_res(ws, connect_id, timeout_s=15)
            if not connect_res.get("ok"):
                raise ConnectError(f"OpenClaw connect rejected: {connect_res.get('error')}")

            self._conn = _ConnectInfo(
                ws=ws,
                gateway_token=gateway_token,
                canonical_session_key="",  # set by _create_fresh_session per ask
            )
        except Exception:
            await ws.close()
            raise

    async def _create_fresh_session(self, ws) -> str:
        suffix = uuid.uuid4().hex[:8]
        key = f"{SESSION_KEY_PREFIX}-{suffix}"
        create_id = uuid.uuid4().hex
        await ws.send(json.dumps({
            "type": "req",
            "id": create_id,
            "method": "sessions.create",
            # Label must be unique across active sessions too — appending the
            # same suffix keeps it human-readable AND distinct.
            "params": {"key": key, "label": f"Tytus Lope {suffix}"},
        }))
        res = await _await_res(ws, create_id, timeout_s=15)
        if not res.get("ok"):
            raise ConnectError(f"sessions.create failed: {res.get('error')}")
        canonical = _extract_session_key(res["payload"]) or key
        sub_id = uuid.uuid4().hex
        await ws.send(json.dumps({
            "type": "req",
            "id": sub_id,
            "method": "sessions.messages.subscribe",
            "params": {"key": canonical},
        }))
        sub_res = await _await_res(ws, sub_id, timeout_s=15)
        if not sub_res.get("ok"):
            raise ConnectError(f"messages.subscribe failed: {sub_res.get('error')}")
        return canonical

    async def _bootstrap_token(self) -> str:
        """Hit the forwarder once; the 302 redirect carries the gateway token.

        The forwarder writes the same token into `config.user.json.gateway.controlUi.allowedOrigins`
        and redirects every browser with `?token=T` for silent local pairing.
        We harvest the same token the same way.
        """
        url = f"http://localhost:{self.port}/"
        async with httpx.AsyncClient(follow_redirects=False, timeout=5.0) as client:
            resp = await client.get(url)
        if resp.status_code not in (302, 303):
            # Some forwarder paths return 200 with the token baked into HTML.
            # Not our case today, but keep a clear error.
            raise ConnectError(
                f"Forwarder on port {self.port} didn't emit a pairing redirect "
                f"(status={resp.status_code}). Is the pod connected? Try `tytus ui --pod {self.pod_id}`."
            )
        loc = resp.headers.get("location") or resp.headers.get("Location") or ""
        q = urllib.parse.urlparse(loc).query
        token = urllib.parse.parse_qs(q).get("token", [None])[0]
        if not token:
            raise ConnectError(f"Forwarder redirect has no token: {loc!r}")
        return token


# ── Module-level helpers ────────────────────────────────────────────────


class ConnectError(RuntimeError):
    pass


async def _await_event(ws, event_name: str, timeout_s: int) -> str:
    """Wait for a specific event frame, return its payload.nonce (for challenge)."""
    deadline = time.time() + timeout_s
    while True:
        remaining = deadline - time.time()
        if remaining <= 0:
            raise ConnectError(f"Timed out waiting for event {event_name!r}")
        raw = await asyncio.wait_for(ws.recv(), timeout=remaining)
        frame = json.loads(raw)
        if frame.get("type") == "event" and frame.get("event") == event_name:
            return frame.get("payload", {}).get("nonce", "")


async def _await_res(ws, req_id: str, timeout_s: int) -> dict:
    """Wait for the response frame matching req_id."""
    deadline = time.time() + timeout_s
    while True:
        remaining = deadline - time.time()
        if remaining <= 0:
            raise ConnectError(f"Timed out waiting for res to {req_id}")
        raw = await asyncio.wait_for(ws.recv(), timeout=remaining)
        frame = json.loads(raw)
        if frame.get("type") == "res" and frame.get("id") == req_id:
            return frame


async def _collect_assistant_reply(ws, session_key: str, run_req_id: str) -> str:
    """Read frames until this run produces its final assistant message.

    Terminal signal we rely on: a `chat` event frame with `state: "final"` for
    *our* runId (observed in live traffic). session.message events carry the
    actual content — we keep the latest assistant-role message whose runId
    matches ours. We ignore prior-turn replay (the pod may echo older history
    on subscribe, but session.message.payload.message.runId on the assistant
    entries scopes them to runs, so we filter by our runId).
    """
    assistant_text = ""
    run_id: str | None = None
    seen_any_assistant = False

    while True:
        raw = await ws.recv()
        frame = json.loads(raw)

        # sessions.send RPC response — gives us the runId for this turn.
        if frame.get("type") == "res" and frame.get("id") == run_req_id:
            if frame.get("ok") is False:
                raise ConnectError(f"sessions.send failed: {frame.get('error')}")
            payload = frame.get("payload") or {}
            run_id = payload.get("runId") or payload.get("id") or run_id
            continue

        if frame.get("type") != "event":
            continue

        event = frame.get("event")
        payload = frame.get("payload") or {}

        # Terminal signal: chat event for our run hitting state=final.
        if event == "chat" and payload.get("sessionKey") == session_key:
            state = payload.get("state")
            matches_run = (run_id is None) or payload.get("runId") == run_id
            if state == "final" and matches_run and seen_any_assistant:
                break
            if state in {"error", "aborted"}:
                raise ConnectError(f"OpenClaw run ended in state {state!r}: {payload}")
            continue

        if event == "session.message" and payload.get("sessionKey") == session_key:
            msg = payload.get("message") or {}
            role = msg.get("role") if isinstance(msg, dict) else None
            if role != "assistant":
                continue
            # Filter to this run only (the subscribe replay may include
            # previous assistant turns with different runIds).
            msg_run = msg.get("runId") if isinstance(msg, dict) else None
            if run_id and msg_run and msg_run != run_id:
                continue
            content = _extract_message_content(payload)
            if content:
                assistant_text = content
                seen_any_assistant = True
            continue

        if event in {"session.error", "run.error", "chat.error"}:
            raise ConnectError(f"OpenClaw stream error: {payload}")

    return assistant_text or "(no assistant text — agent may have used tools only)"


def _extract_message_content(payload: dict) -> str:
    """Pull text content out of the many shapes OpenClaw might emit."""
    msg = payload.get("message")
    if msg is None:
        return ""
    if isinstance(msg, str):
        return msg
    if isinstance(msg, dict):
        content = msg.get("content")
        if isinstance(content, str):
            return content
        if isinstance(content, list):
            parts: list[str] = []
            for item in content:
                if isinstance(item, str):
                    parts.append(item)
                elif isinstance(item, dict):
                    t = item.get("text") or item.get("content") or ""
                    if isinstance(t, str):
                        parts.append(t)
            return "".join(parts)
        # Pi-style transcripts: {role, parts: [{kind, text}]}
        if "parts" in msg and isinstance(msg["parts"], list):
            return "".join(
                p.get("text", "") for p in msg["parts"] if isinstance(p, dict)
            )
    return ""


def _extract_role(payload: dict) -> str | None:
    msg = payload.get("message")
    if isinstance(msg, dict):
        return msg.get("role")
    return None


def _extract_status(payload: dict) -> str | None:
    msg = payload.get("message")
    if isinstance(msg, dict):
        return msg.get("status")
    return None


def _extract_session_key(payload) -> str | None:
    if isinstance(payload, dict):
        # Common shapes: {"key": "..."} or {"session": {"key": "..."}}
        if isinstance(payload.get("key"), str):
            return payload["key"]
        sess = payload.get("session")
        if isinstance(sess, dict) and isinstance(sess.get("key"), str):
            return sess["key"]
        # sessions.create sometimes returns {sessions: [{key:...}]}
        ss = payload.get("sessions")
        if isinstance(ss, list) and ss and isinstance(ss[0], dict):
            k = ss[0].get("key")
            if isinstance(k, str):
                return k
    return None


_DIVIDER = "\n---\n"


def _flatten_messages(messages: list[AgentMessage]) -> str:
    """Render [system, user, assistant, user, …] as a single string
    OpenClaw's sessions.send can accept. System prompt gets a clear preamble;
    prior turns are labeled so the agent can reason over history.

    For Phase 1 we only exercise [system?, user] pairs (the lope validator case),
    so this is a correctness-preserving flattening — not a session reconstruction.
    """
    if not messages:
        return ""
    parts: list[str] = []
    for m in messages:
        if m.role == "system":
            parts.append(f"[[SYSTEM]]\n{m.content}")
        elif m.role == "assistant":
            parts.append(f"[[ASSISTANT]]\n{m.content}")
        else:
            parts.append(m.content if len(messages) == 1 else f"[[USER]]\n{m.content}")
    return _DIVIDER.join(parts)

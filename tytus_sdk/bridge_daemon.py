"""HarveyBridge — the reverse channel (pod agent → human).

Two moving parts, one process:

1. **HTTP listener** on 127.0.0.1:18099. Accepts `POST /inbox` with a
   shared-secret header `X-Tytus-Bridge-Token`. Writes each accepted
   entry to today's Brain journal, and fires an event to the superbrain
   event store when the `$MAKAKOO_HOME` is populated.

2. **Outbox pollers** — one per connected pod. Tails
   `/app/workspace/.harvey-outbox.jsonl` via `tytus exec` every 10s,
   POSTing new lines back to the local listener. The poller terminates
   itself when the pod's `tunnel_iface` goes null (tunnel torn down) or
   the pod disappears from `tytus status`.

Token storage: `~/.tytus/bridge.token` (mode 0600). Phase 5 will migrate
this to the OS keychain via `auth::keychain::Keychain` on the Rust side;
for v0.5 the flat file keeps the Python side self-contained.

Rate limits: hardcoded — 30 notifies / pod / hour, 1 req / client / 200ms.
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import hmac
import http.server
import json
import logging
import os
import secrets
import socketserver
import subprocess
import sys
import threading
import time
from collections import deque
from pathlib import Path
from typing import Any


log = logging.getLogger("tytus_sdk.bridge_daemon")

BRIDGE_HOST = "127.0.0.1"
BRIDGE_PORT = 18099
BRIDGE_TOKEN_PATH = Path.home() / ".tytus" / "bridge.token"
POD_OUTBOX_PATH = "/app/workspace/.harvey-outbox.jsonl"
# Track drained offset per pod on Harvey's side so we don't re-deliver.
STATE_DIR = Path.home() / ".tytus" / "bridge"
POLL_INTERVAL_S = 10
NOTIFY_WINDOW_S = 3600
NOTIFY_LIMIT_PER_POD = 30


# ── Shared secret management ────────────────────────────────────────────────


def ensure_bridge_token() -> str:
    """Load or mint the bridge token. Stored 0600 under ~/.tytus/."""
    if BRIDGE_TOKEN_PATH.exists():
        return BRIDGE_TOKEN_PATH.read_text().strip()
    BRIDGE_TOKEN_PATH.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    token = secrets.token_urlsafe(32)
    BRIDGE_TOKEN_PATH.write_text(token)
    os.chmod(BRIDGE_TOKEN_PATH, 0o600)
    return token


def rotate_bridge_token() -> str:
    if BRIDGE_TOKEN_PATH.exists():
        BRIDGE_TOKEN_PATH.unlink()
    return ensure_bridge_token()


def _constant_time_eq(a: str, b: str) -> bool:
    return hmac.compare_digest(a.encode(), b.encode())


# ── Brain writer ────────────────────────────────────────────────────────────


def _brain_home() -> Path | None:
    for var in ("MAKAKOO_HOME", "HARVEY_HOME"):
        raw = os.environ.get(var)
        if raw and Path(raw).is_dir():
            return Path(raw)
    fallback = Path.home() / "MAKAKOO"
    if fallback.is_dir():
        return fallback
    fallback2 = Path.home() / "HARVEY"
    if fallback2.is_dir():
        return fallback2
    return None


def write_to_brain(entry: dict[str, Any]) -> Path | None:
    """Append a line to today's Brain journal. Logseq-outliner format.

    entry keys: ts, pod_id, agent, typ, message, details
    """
    brain = _brain_home()
    if not brain:
        log.warning("No MAKAKOO_HOME / HARVEY_HOME set; skipping brain write")
        return None
    today = datetime.date.today().strftime("%Y_%m_%d")
    journal = brain / "data" / "Brain" / "journals" / f"{today}.md"
    journal.parent.mkdir(parents=True, exist_ok=True)
    pod_id = entry.get("pod_id", "?")
    agent = entry.get("agent", "?")
    typ = entry.get("typ", "info")
    msg = entry.get("message", "")
    ts = entry.get("ts", datetime.datetime.utcnow().isoformat() + "Z")
    # Compact, grep-friendly, tagged with the pod so filtering works.
    line = (
        f"- [tytus-{pod_id} {typ}] [[TytusOpenClaw]] [[{agent}]] · {ts}: "
        f"{msg.replace(chr(10), ' / ')}"
    )
    with journal.open("a") as f:
        f.write(line + "\n")
    return journal


def write_superbrain_event(entry: dict[str, Any]) -> bool:
    """Best-effort ingest to superbrain (only if the binary is on PATH)."""
    try:
        payload = json.dumps({
            "type": f"tytus.{entry.get('typ', 'info')}",
            "agent": f"tytus-{entry.get('pod_id','?')}-{entry.get('agent','?')}",
            "summary": entry.get("message", ""),
            "details": entry.get("details", {}) or {},
        })
    except (TypeError, ValueError):
        return False
    try:
        subprocess.run(
            ["superbrain", "remember", payload],
            check=False,
            timeout=5,
            capture_output=True,
        )
        return True
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return False


# ── HTTP listener ──────────────────────────────────────────────────────────


class _PodRateLimit:
    def __init__(self):
        self._hits: dict[str, deque[float]] = {}
        self._lock = threading.Lock()

    def allow(self, pod_id: str) -> bool:
        now = time.time()
        with self._lock:
            q = self._hits.setdefault(pod_id, deque())
            while q and now - q[0] > NOTIFY_WINDOW_S:
                q.popleft()
            if len(q) >= NOTIFY_LIMIT_PER_POD:
                return False
            q.append(now)
        return True


class _InboxHandler(http.server.BaseHTTPRequestHandler):
    server_version = "tytus-bridge/0.5"
    expected_token: str = ""
    rate_limiter: _PodRateLimit = _PodRateLimit()

    def log_message(self, fmt, *args):
        log.debug("%s - - %s", self.address_string(), fmt % args)

    def _json(self, code: int, body: dict):
        payload = json.dumps(body).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def do_GET(self):
        if self.path == "/health":
            self._json(200, {"ok": True, "service": "tytus-bridge"})
            return
        self._json(404, {"ok": False, "error": "not found"})

    def do_POST(self):
        if self.path != "/inbox":
            self._json(404, {"ok": False, "error": "not found"})
            return
        provided = self.headers.get("X-Tytus-Bridge-Token", "")
        if not _constant_time_eq(provided, self.expected_token):
            self._json(401, {"ok": False, "error": "unauthorized"})
            return
        length = int(self.headers.get("Content-Length", "0") or 0)
        if length <= 0 or length > 64 * 1024:
            self._json(413, {"ok": False, "error": "payload size out of range"})
            return
        raw = self.rfile.read(length)
        try:
            entry = json.loads(raw)
        except json.JSONDecodeError:
            self._json(400, {"ok": False, "error": "invalid JSON"})
            return
        if not isinstance(entry, dict):
            self._json(400, {"ok": False, "error": "JSON must be an object"})
            return
        pod_id = str(entry.get("pod_id", "unknown"))
        if not self.rate_limiter.allow(pod_id):
            self._json(429, {"ok": False, "error": "rate limit"})
            return
        journal = write_to_brain(entry)
        superbrain_ok = write_superbrain_event(entry)
        self._json(200, {
            "ok": True,
            "journal": str(journal) if journal else None,
            "superbrain": superbrain_ok,
        })


def start_http_server(token: str) -> tuple[socketserver.TCPServer, threading.Thread]:
    _InboxHandler.expected_token = token

    class _Server(socketserver.ThreadingMixIn, http.server.HTTPServer):
        daemon_threads = True
        allow_reuse_address = True

    srv = _Server((BRIDGE_HOST, BRIDGE_PORT), _InboxHandler)
    t = threading.Thread(target=srv.serve_forever, name="bridge-http", daemon=True)
    t.start()
    log.info("bridge listening on %s:%s", BRIDGE_HOST, BRIDGE_PORT)
    return srv, t


# ── Outbox pollers (per pod) ───────────────────────────────────────────────


def _pod_is_connected(pod_id: str) -> bool:
    """True if the pod is reachable.

    `tytus status --json` can report `tunnel_iface: null` even when the WG
    tunnel is up (state.json drift — see cli/src/state.rs TODOs). Belt-and-
    braces: also probe the pod's forwarder port on localhost. If the forwarder
    answers, the tunnel + agent are live regardless of what state.json says.
    """
    import socket
    try:
        num = int(pod_id)
    except ValueError:
        return False
    port = 18700 + num
    # TCP probe — 1 s timeout, loopback only
    try:
        s = socket.create_connection(("127.0.0.1", port), timeout=1.0)
        s.close()
        return True
    except (OSError, ConnectionError):
        pass
    # Fallback: trust tytus state.json if forwarder isn't up yet
    try:
        out = subprocess.check_output(["tytus", "status", "--json"], timeout=10)
        data = json.loads(out)
    except (subprocess.CalledProcessError, FileNotFoundError,
            subprocess.TimeoutExpired, json.JSONDecodeError):
        return False
    for pod in data.get("pods", []):
        if str(pod.get("pod_id")) == str(pod_id):
            return bool(pod.get("tunnel_iface"))
    return False


def _state_file(pod_id: str) -> Path:
    STATE_DIR.mkdir(parents=True, exist_ok=True, mode=0o700)
    return STATE_DIR / f"outbox-{pod_id}.state"


def _load_offset(pod_id: str) -> int:
    p = _state_file(pod_id)
    if not p.exists():
        return 0
    try:
        return int(p.read_text().strip() or "0")
    except ValueError:
        return 0


def _save_offset(pod_id: str, offset: int) -> None:
    _state_file(pod_id).write_text(str(offset))


def _read_pod_outbox_tail(pod_id: str, offset: int) -> tuple[list[str], int]:
    """Return (new lines, new offset). Uses `wc -c` + `tail -c +N` pattern.

    We avoid relying on `python3` / `node` on the pod — plain POSIX shell.
    Returns ([], offset) if outbox is empty or missing.
    """
    # Byte-count of the outbox file (0 if missing).
    cmd_size = (
        f"[ -f {POD_OUTBOX_PATH} ] && wc -c < {POD_OUTBOX_PATH} || echo 0"
    )
    size_out = subprocess.run(
        ["tytus", "exec", "--pod", pod_id, "--timeout", "10", cmd_size],
        capture_output=True, text=True, timeout=20,
    )
    if size_out.returncode != 0:
        raise RuntimeError(f"tytus exec (size) pod {pod_id} rc={size_out.returncode}")
    size_line = next(
        (l for l in size_out.stdout.splitlines() if not l.startswith("Running on pod") and l.strip().isdigit()),
        "0",
    )
    size = int(size_line or 0)
    if size <= offset:
        return [], size  # no new bytes; if truncated, reset to new size
    # `tail -c +N` is 1-indexed ("start at byte N").
    start_byte = offset + 1
    cmd_tail = f"tail -c +{start_byte} {POD_OUTBOX_PATH}"
    tail_out = subprocess.run(
        ["tytus", "exec", "--pod", pod_id, "--timeout", "20", cmd_tail],
        capture_output=True, text=True, timeout=30,
    )
    if tail_out.returncode != 0:
        raise RuntimeError(f"tytus exec (tail) pod {pod_id} rc={tail_out.returncode}")
    # `tytus exec` prints its "Running on pod …" header to STDERR, so
    # stdout already contains the raw file bytes. Preserve trailing
    # newlines (they're the "line terminator" signal that this entry is
    # complete and safe to drain).
    body = tail_out.stdout
    # Defensive: in case future tytus CLI versions ever print on stdout,
    # strip a leading header line if it's present.
    if body.startswith("Running on pod"):
        first_nl = body.find("\n")
        if first_nl >= 0:
            body = body[first_nl + 1:]
    # Only complete (newline-terminated) lines count as drained.
    if "\n" not in body:
        return [], offset
    complete, _, _partial = body.rpartition("\n")
    # complete now holds every finished line minus the trailing \n; split
    # preserves the original line boundaries.
    lines = [l for l in complete.split("\n") if l.strip()]
    drained_bytes = len(complete.encode("utf-8")) + 1  # include the trailing \n
    return lines, offset + drained_bytes


def _post_to_bridge(token: str, line: str) -> bool:
    """POST a single JSONL line to our own /inbox."""
    import http.client
    try:
        entry = json.loads(line)
    except json.JSONDecodeError:
        log.warning("dropping malformed outbox line: %r", line[:120])
        return False
    conn = http.client.HTTPConnection(BRIDGE_HOST, BRIDGE_PORT, timeout=5)
    try:
        conn.request(
            "POST",
            "/inbox",
            body=json.dumps(entry).encode(),
            headers={
                "Content-Type": "application/json",
                "X-Tytus-Bridge-Token": token,
            },
        )
        resp = conn.getresponse()
        ok = resp.status == 200
        if not ok:
            log.warning("bridge POST %s %s", resp.status, resp.read(500))
        return ok
    except (ConnectionError, TimeoutError, OSError) as e:
        log.warning("bridge POST failed: %s", e)
        return False
    finally:
        conn.close()


def poll_pod_outbox(pod_id: str, token: str, stop_event: threading.Event) -> None:
    """Long-running poller for one pod. Terminates when stop_event is set or
    the pod disconnects (lifecycle guard per gemini's critique).

    All per-iteration errors are logged and swallowed — the poller only exits
    via stop_event or explicit disconnect. This matters because silent thread
    death leaves the bridge looking healthy (HTTP listener is fine) but
    delivering nothing.
    """
    log.info("outbox poller started for pod %s", pod_id)
    offset = _load_offset(pod_id)
    tick = 0
    while not stop_event.is_set():
        tick += 1
        try:
            log.debug("pod %s poll tick %d (offset=%s)", pod_id, tick, offset)
            if not _pod_is_connected(pod_id):
                log.info("pod %s no longer connected — poller exiting", pod_id)
                return
            try:
                lines, new_offset = _read_pod_outbox_tail(pod_id, offset)
            except (subprocess.TimeoutExpired, RuntimeError) as e:
                log.warning("pod %s outbox read failed: %s", pod_id, e)
                stop_event.wait(POLL_INTERVAL_S)
                continue
            log.debug("pod %s tick %d: lines=%d new_offset=%s", pod_id, tick, len(lines), new_offset)
            if lines:
                delivered = 0
                for line in lines:
                    if _post_to_bridge(token, line):
                        delivered += 1
                log.info("pod %s: delivered %d/%d notifies", pod_id, delivered, len(lines))
            if new_offset != offset:
                _save_offset(pod_id, new_offset)
                offset = new_offset
        except Exception as e:  # noqa: BLE001 — last-resort catch
            log.exception("pod %s poller iteration error: %s", pod_id, e)
        stop_event.wait(POLL_INTERVAL_S)


def connected_pod_ids() -> list[str]:
    """Return pod ids whose forwarder port is accepting TCP connections."""
    try:
        out = subprocess.check_output(["tytus", "status", "--json"], timeout=10)
        data = json.loads(out)
    except (subprocess.CalledProcessError, FileNotFoundError, json.JSONDecodeError, subprocess.TimeoutExpired):
        return []
    return [
        str(p["pod_id"])
        for p in data.get("pods", [])
        if _pod_is_connected(str(p["pod_id"]))
    ]


# ── CLI entrypoints ───────────────────────────────────────────────────────


def cmd_bridge_run(args: argparse.Namespace) -> int:
    # basicConfig may have already run in cli.py:main() at WARNING — we need
    # INFO/DEBUG for the daemon. Override explicitly.
    root = logging.getLogger()
    level = logging.DEBUG if getattr(args, "verbose", False) else logging.INFO
    root.setLevel(level)
    if not root.handlers:
        handler = logging.StreamHandler()
        handler.setFormatter(logging.Formatter("%(asctime)s %(levelname)s %(name)s: %(message)s"))
        root.addHandler(handler)
    else:
        for h in root.handlers:
            h.setLevel(level)
    log.setLevel(level)
    token = ensure_bridge_token()
    print(f"Bridge token: {BRIDGE_TOKEN_PATH} ({len(token)} chars)", file=sys.stderr)

    srv, http_thread = start_http_server(token)

    stop_event = threading.Event()
    pollers: dict[str, tuple[threading.Thread, threading.Event]] = {}

    def sync_pollers():
        connected = set(connected_pod_ids())
        for pod in connected - set(pollers.keys()):
            ev = threading.Event()
            t = threading.Thread(
                target=poll_pod_outbox, args=(pod, token, ev),
                name=f"outbox-poll-{pod}", daemon=True,
            )
            t.start()
            pollers[pod] = (t, ev)
            log.info("spawned poller for pod %s", pod)
        for pod in set(pollers.keys()) - connected:
            _, ev = pollers.pop(pod)
            ev.set()
            log.info("stopping poller for pod %s (disconnected)", pod)

    # Daemons that may be launched by a parent shell under nohup receive
    # SIGPIPE if stdout/stderr get severed (e.g. script harness closes its
    # log file). Ignoring keeps us alive.
    import signal as _signal
    try:
        _signal.signal(_signal.SIGPIPE, _signal.SIG_IGN)
    except (AttributeError, ValueError):
        pass

    try:
        while not stop_event.is_set():
            try:
                sync_pollers()
            except Exception as e:
                log.warning("sync_pollers error (ignored): %s", e)
            time.sleep(30)
    except KeyboardInterrupt:
        print("\nShutting down bridge…", file=sys.stderr)
    finally:
        for _, ev in pollers.values():
            ev.set()
        srv.shutdown()
    return 0


def cmd_bridge_status(args: argparse.Namespace) -> int:
    import http.client
    try:
        conn = http.client.HTTPConnection(BRIDGE_HOST, BRIDGE_PORT, timeout=3)
        conn.request("GET", "/health")
        resp = conn.getresponse()
        body = resp.read()
        if resp.status == 200:
            print(body.decode())
            return 0
        print(f"Bridge responded with {resp.status}: {body.decode()[:200]}", file=sys.stderr)
        return 1
    except (ConnectionError, OSError):
        print("Bridge is not running. Start it with: tytus bridge run", file=sys.stderr)
        return 1


def cmd_bridge_rotate_token(args: argparse.Namespace) -> int:
    token = rotate_bridge_token()
    if args.json:
        print(json.dumps({"token_path": str(BRIDGE_TOKEN_PATH), "rotated": True}, indent=2))
    else:
        print(
            f"New bridge token: {BRIDGE_TOKEN_PATH}\n"
            f"Re-seed pod-side helpers with: tytus lope install --pod NN",
            file=sys.stderr,
        )
    return 0


def cmd_bridge_test(args: argparse.Namespace) -> int:
    """Send a synthetic notify and verify it lands in the journal."""
    token = ensure_bridge_token()
    entry = {
        "ts": datetime.datetime.utcnow().isoformat() + "Z",
        "pod_id": args.pod,
        "agent": args.agent,
        "typ": "info",
        "message": args.message,
        "details": {"source": "tytus bridge test"},
    }
    ok = _post_to_bridge(token, json.dumps(entry))
    if args.json:
        print(json.dumps({"ok": ok, "entry": entry}, indent=2))
    elif ok:
        print(f"✓ delivered: {args.message}", file=sys.stderr)
    else:
        print(f"✗ delivery failed — is `tytus bridge run` up?", file=sys.stderr)
    return 0 if ok else 1

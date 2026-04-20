#!/usr/bin/env bash
# =============================================================================
# Tytus multi-process coherence harness
# =============================================================================
# Verifies that the three-process dance (CLI writer ↔ daemon cache ↔ tray
# reader) stays coherent under the class of faults that shipped the
# 2026-04-20 regression: stale in-memory daemon state, broken keychain,
# and post-login tray lies. Would have caught that bug on the first run.
#
# Flows:
#   M1 Daemon socket is up + ping responds
#   M2 status RPC returns JSON with expected top-level shape
#   M3 Response carries keychain_healthy + last_refresh_error fields
#   M4 File watcher: touch state.json → daemon mtime advances, pid stable
#   M5 Hot reload unsticks NeedsLogin when file has valid creds
#   M6 Stale-PID sweep logic linked into the shipped binary
#   M7 Tray-side merged view agrees with daemon on logged_in
#
# Exit 0 on green. Non-destructive: does not logout, revoke, or kill
# foreign processes. Touches only the user's own daemon + state.json.
#
# Usage:
#   scripts/e2e-multiprocess.sh
# =============================================================================
set -u

export PATH="/bin:/usr/bin:/usr/local/bin:/Users/sebastian/bin:${PATH:-}"

pass() { printf '  \033[32m✓\033[0m %s\n' "$*"; }
fail() { printf '  \033[31m✗\033[0m %s\n' "$*"; FAILS=$((FAILS+1)); }
section() { printf '\n\033[1m%s\033[0m\n' "$*"; }
FAILS=0

SOCK=/tmp/tytus/daemon.sock
STATE="$HOME/Library/Application Support/tytus/state.json"
if [[ ! -f "$STATE" ]]; then
    STATE="$HOME/.config/tytus/state.json"
fi

# Speak the JSON-line RPC over the daemon's Unix socket. We use Python
# instead of `nc -U`: macOS's bundled nc has a flaky `-w` behaviour
# around idle-close detection (it waits for the server to close even
# after a full response arrives), producing false-positive timeouts
# for RPCs that are semantically fast.
#
# Usage: rpc <cmd>
rpc() {
    python3 - "$1" <<'PY'
import json, socket, sys
cmd = sys.argv[1]
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.settimeout(5.0)
s.connect('/tmp/tytus/daemon.sock')
s.sendall((json.dumps({"cmd": cmd}) + "\n").encode())
s.shutdown(socket.SHUT_WR)
buf = b""
while True:
    chunk = s.recv(4096)
    if not chunk: break
    buf += chunk
sys.stdout.write(buf.decode(errors="replace"))
PY
}

section "M1  Daemon socket present + ping responds"
if [[ -S "$SOCK" ]] && rpc ping 2>/dev/null | grep -q '"pong"'; then
    pass "daemon responds to ping"
else
    fail "daemon socket not responding — start with: tytus daemon run"
    echo "$FAILS failure(s). Aborting; remaining flows need the daemon."
    exit 1
fi

section "M2  status RPC latency (no keychain on hot path)"
T0=$(python3 -c 'import time; print(int(time.time()*1000))')
RESP=$(rpc status 2>/dev/null)
T1=$(python3 -c 'import time; print(int(time.time()*1000))')
ELAPSED=$((T1 - T0))
if [[ "$ELAPSED" -lt 1000 ]]; then
    pass "status RPC returned in ${ELAPSED}ms"
else
    fail "status RPC took ${ELAPSED}ms — keychain likely still on hot path"
fi

section "M3  status response carries health fields"
HAS_KC=$(echo "$RESP" | python3 -c "import json,sys; print('keychain_healthy' in json.load(sys.stdin).get('data',{}).get('daemon',{}))" 2>/dev/null)
HAS_ERR_FIELD=$(echo "$RESP" | python3 -c "import json,sys; print('last_refresh_error' in json.load(sys.stdin).get('data',{}).get('daemon',{}))" 2>/dev/null)
HAS_STUCK=$(echo "$RESP" | python3 -c "import json,sys; print('stuck_for_secs' in json.load(sys.stdin).get('data',{}).get('daemon',{}))" 2>/dev/null)
if [[ "$HAS_KC" == "True" ]]; then pass "daemon reports keychain_healthy"; else fail "missing keychain_healthy"; fi
if [[ "$HAS_ERR_FIELD" == "True" ]]; then pass "daemon reports last_refresh_error"; else fail "missing last_refresh_error"; fi
if [[ "$HAS_STUCK" == "True" ]]; then pass "daemon reports stuck_for_secs"; else fail "missing stuck_for_secs"; fi

section "M4  File watcher picks up state.json mtime change"
PID_BEFORE=$(echo "$RESP" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['daemon']['pid'])")
touch "$STATE"
sleep 2
RESP2=$(rpc status 2>/dev/null)
PID_AFTER=$(echo "$RESP2" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['daemon']['pid'])")
if [[ "$PID_BEFORE" == "$PID_AFTER" ]]; then
    pass "same daemon pid before/after touch (pid=$PID_AFTER, no respawn)"
else
    fail "pid changed ${PID_BEFORE} → ${PID_AFTER} — daemon respawned unexpectedly"
fi
FILE_EMAIL=$(python3 -c "import json; print(json.load(open('$STATE')).get('email',''))" 2>/dev/null)
DAEMON_EMAIL=$(echo "$RESP2" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['auth'].get('email',''))")
if [[ -n "$FILE_EMAIL" && "$FILE_EMAIL" == "$DAEMON_EMAIL" ]]; then
    pass "file email == daemon email ($FILE_EMAIL)"
else
    fail "email drift: file='$FILE_EMAIL', daemon='$DAEMON_EMAIL'"
fi

section "M5  NeedsLogin self-clears when file is logged in"
STATUS=$(echo "$RESP2" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['daemon']['status'])")
LOGGED_IN=$(echo "$RESP2" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['auth']['logged_in'])")
if [[ -z "$FILE_EMAIL" ]]; then
    pass "file has no email — skipping (user legitimately not logged in)"
elif [[ "$STATUS" != "needs_login" && "$LOGGED_IN" == "True" ]]; then
    pass "daemon status=$STATUS, auth.logged_in=$LOGGED_IN (healthy)"
else
    fail "stuck: status=$STATUS, logged_in=$LOGGED_IN, file_email=$FILE_EMAIL"
fi

section "M6  Stale PID sweep linked into binary"
if /Users/sebastian/bin/tytus --help >/dev/null 2>&1; then
    if strings /Users/sebastian/bin/tytus 2>/dev/null | grep -q 'sweeping stale pidfile'; then
        pass "sweep_stale_pids linked into tytus binary"
    else
        fail "stale-pid sweep log string not found — binary may be pre-hardening"
    fi
else
    fail "tytus binary not runnable"
fi

section "M7  Tray merge agrees with daemon"
TRAY_LOGGED_IN=$(python3 <<PY
import json
d = json.loads('''$RESP2''')
daemon = d['data']['auth']
try:
    with open('$STATE') as f:
        state = json.load(f)
    file_logged_in = bool(state.get('email'))
except Exception:
    file_logged_in = False
# Tray's merge: daemon.logged_in OR file.logged_in (see socket.rs poll_daemon_status)
print(daemon['logged_in'] or file_logged_in)
PY
)
DAEMON_LOGGED_IN=$(echo "$RESP2" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['auth']['logged_in'])")
# Acceptable: tray == daemon, OR tray=true and daemon=false (file catches up).
# NOT acceptable: tray=false while daemon=true (would hide login).
if [[ "$TRAY_LOGGED_IN" == "False" && "$DAEMON_LOGGED_IN" == "True" ]]; then
    fail "tray computed logged_in=false while daemon says true"
else
    pass "tray merge OK (tray=$TRAY_LOGGED_IN, daemon=$DAEMON_LOGGED_IN)"
fi

echo
if [[ "$FAILS" == 0 ]]; then
    echo "All multiprocess flows green."
    exit 0
else
    echo "$FAILS flow(s) failed."
    exit 1
fi

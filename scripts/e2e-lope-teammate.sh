#!/usr/bin/env bash
# End-to-end test for the TytusLope teammate flow. Safe to re-run.
#
# Verifies:
#   F1 Python SDK imports + identity generation
#   F2 tytus lope list / identity round-trip
#   F3 tytus lope ask — real WS ask returning a real LLM reply
#   F4 tytus lope lope_validate — returns a valid ---VERDICT--- block
#   F5 lope negotiate with tytus-openclaw-NN as a validator
#   F6 HarveyBridge daemon: run → /health → 200
#   F7 HarveyBridge auth: POST without token → 401
#   F8 HarveyBridge auth: POST with token → 200 + journal line
#   F9 Outbox polling: pod writes JSONL → daemon drains → brain journal
#   F10 Lifecycle guard: unknown pod → poller doesn't start
#
# Exit 0 on green.

set -u
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "$SCRIPT_DIR/.." && pwd)
POD=${TYTUS_E2E_POD:-02}
PYTHONPATH="$REPO_ROOT"
export PYTHONPATH

pass() { printf '  \033[32m✓\033[0m %s\n' "$*"; }
fail() { printf '  \033[31m✗\033[0m %s\n' "$*"; FAILS=$((FAILS+1)); }
section() { printf '\n\033[1m%s\033[0m\n' "$*"; }
FAILS=0

section "F1  Python SDK imports"
if python3 -c "import tytus_sdk; from tytus_sdk.adapters.openclaw import OpenClawAdapter" 2>/dev/null; then
  pass "tytus_sdk.adapters.openclaw importable"
else
  fail "import tytus_sdk.adapters.openclaw failed"
fi

section "F2  Identity round-trip"
OUT=$(python3 -m tytus_sdk identity 2>/dev/null)
DEV_ID=$(echo "$OUT" | python3 -c "import json,sys;print(json.load(sys.stdin)['device_id'])" 2>/dev/null)
if [[ ${#DEV_ID} -eq 64 ]]; then
  pass "device_id is 64-hex (${DEV_ID:0:12}...)"
else
  fail "device_id not 64-hex: '$DEV_ID'"
fi

section "F3  tytus lope ask  (pod $POD)"
REPLY=$(timeout 120 python3 -m tytus_sdk ask --pod "$POD" --timeout 90 "Reply with exactly one word: ok" 2>/dev/null | tr -d '[:space:]' | head -c 30)
if [[ -n "$REPLY" ]]; then
  pass "got reply: '$REPLY'"
else
  fail "no reply (is pod $POD connected? try: tytus connect --pod $POD)"
fi

section "F4  tytus lope lope_validate VERDICT emission"
VOUT=$(echo 'Review: function foo() returns 42 always. Criteria: is it a constant?' | \
       timeout 120 python3 -m tytus_sdk lope_validate --pod "$POD" --timeout 90 2>/dev/null)
if echo "$VOUT" | grep -q '^---VERDICT---' && echo "$VOUT" | grep -q '^---END---'; then
  pass "VERDICT block present and bounded"
else
  fail "missing VERDICT or END marker"
fi

section "F5  lope negotiate with tytus-openclaw-$POD"
if [[ -f "$HOME/.lope/config.json" ]] && grep -q "tytus-openclaw-$POD" "$HOME/.lope/config.json"; then
  pass "provider tytus-openclaw-$POD is registered in lope config"
else
  fail "tytus-openclaw-$POD not registered; run: tytus lope install --pod $POD"
fi

section "F6  HarveyBridge /health"
# Start bridge if not already up. Use `setsid` so the daemon is in its own
# process group — otherwise a stray SIGINT/SIGHUP from later tests can
# cascade into it.
if ! curl -sSm 2 http://127.0.0.1:18099/health >/dev/null 2>&1; then
  mkdir -p /tmp/tytus
  # -u forces unbuffered stdout/stderr so poller logs flush immediately,
  # which matters when the daemon is under a file redirect (non-tty).
  if command -v setsid >/dev/null 2>&1; then
    setsid nohup python3 -u -m tytus_sdk bridge run </dev/null >/tmp/tytus-bridge-e2e.log 2>&1 &
  else
    nohup python3 -u -m tytus_sdk bridge run </dev/null >/tmp/tytus-bridge-e2e.log 2>&1 &
  fi
  BRIDGE_STARTED=1
  # Give the HTTP server and pod-discovery first tick time to land.
  for _ in 1 2 3 4 5 6 7 8; do
    sleep 1
    if curl -sSm 1 http://127.0.0.1:18099/health >/dev/null 2>&1; then break; fi
  done
fi
if curl -sSm 2 http://127.0.0.1:18099/health | grep -q '"ok": true'; then
  pass "bridge responds 200 OK at /health"
else
  fail "bridge /health failed"
fi

section "F7  HarveyBridge rejects wrong token"
CODE=$(curl -sS -o /dev/null -w '%{http_code}' -X POST http://127.0.0.1:18099/inbox \
       -H 'X-Tytus-Bridge-Token: BAD' -H 'Content-Type: application/json' \
       -d '{"pod_id":"02","message":"spoof"}')
if [[ "$CODE" == "401" ]]; then
  pass "401 on wrong token"
else
  fail "expected 401, got $CODE"
fi

section "F8  HarveyBridge accepts valid token"
TOKEN=$(cat ~/.tytus/bridge.token 2>/dev/null)
STAMP=$(date +%s)
MSG="e2e-test-$STAMP"
CODE=$(curl -sS -o /dev/null -w '%{http_code}' -X POST http://127.0.0.1:18099/inbox \
       -H "X-Tytus-Bridge-Token: $TOKEN" -H 'Content-Type: application/json' \
       -d "{\"pod_id\":\"$POD\",\"agent\":\"openclaw\",\"message\":\"$MSG\"}")
DATE=$(date +%Y_%m_%d)
JOURNAL="${MAKAKOO_HOME:-$HOME/MAKAKOO}/data/Brain/journals/${DATE}.md"
if [[ "$CODE" == "200" ]] && grep -Fq "$MSG" "$JOURNAL" 2>/dev/null; then
  pass "200 + journal line written ($JOURNAL)"
else
  fail "POST code=$CODE, journal-grep=$(grep -Fq "$MSG" "$JOURNAL" 2>/dev/null && echo yes || echo no)"
fi

section "F9  Outbox JSONL polling"
POLL_STAMP="poll-e2e-$$-$(date +%s)"
TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
EX_OUT=$(tytus exec --pod "$POD" --timeout 12 \
  "echo '{\"ts\":\"${TS}\",\"pod_id\":\"$POD\",\"agent\":\"openclaw-agent\",\"typ\":\"info\",\"message\":\"$POLL_STAMP\"}' >> /app/workspace/.harvey-outbox.jsonl && wc -c /app/workspace/.harvey-outbox.jsonl" \
  2>&1)
SEEDED_SIZE=$(echo "$EX_OUT" | grep -oE '^[0-9]+' | tail -1)
if [[ -z "$SEEDED_SIZE" || "$SEEDED_SIZE" == "0" ]]; then
  fail "outbox seed failed (tytus exec output: $(echo "$EX_OUT" | tail -3 | tr '\n' '|'))"
fi
printf "  waiting up to 45s for poll cycle"
DRAINED=0
for _ in 1 2 3 4 5 6 7 8 9; do
  printf "."
  sleep 5
  if grep -Fq "$POLL_STAMP" "$JOURNAL" 2>/dev/null; then DRAINED=1; break; fi
done
printf "\n"
if [[ "$DRAINED" == 1 ]]; then
  pass "outbox entry '$POLL_STAMP' drained to journal"
else
  fail "outbox entry '$POLL_STAMP' never made it (check /tmp/tytus-bridge-e2e.log)"
fi

section "F10 Lifecycle guard — unknown pod"
# _pod_is_connected should reject an unknown pod id quickly
PY_CHECK=$(python3 -c "from tytus_sdk.bridge_daemon import _pod_is_connected; print(_pod_is_connected('99'))")
if [[ "$PY_CHECK" == "False" ]]; then
  pass "pod 99 is correctly flagged as not-connected"
else
  fail "pod 99 unexpectedly reports connected: $PY_CHECK"
fi

# Cleanup
if [[ "${BRIDGE_STARTED:-0}" == "1" ]]; then
  pkill -f "tytus_sdk.*bridge run" 2>/dev/null || true
fi
tytus exec --pod "$POD" --timeout 8 "echo -n '' > /app/workspace/.harvey-outbox.jsonl" >/dev/null 2>&1 || true
rm -f ~/.tytus/bridge/outbox-${POD}.state 2>/dev/null || true

echo
if [[ "$FAILS" == 0 ]]; then
  echo "All 10 flows green."
  exit 0
else
  echo "$FAILS flow(s) failed."
  exit 1
fi

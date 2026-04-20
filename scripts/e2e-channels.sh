#!/usr/bin/env bash
# =============================================================================
# Tytus channels E2E harness
# =============================================================================
# Exercises `tytus channels add/list/remove` against a live pod and confirms
# the pod's agent container ends up with the expected env vars.
#
# Two modes:
#   - Static: CLI-only flows + static binary checks (works anywhere).
#   - Live:   end-to-end against a real pod with TELEGRAM_BOT_TOKEN.
#             Set E2E_TELEGRAM_BOT_TOKEN=... to enable.
#
# Requires on the droplet:
#   - pod-egress bridge applied (docker-compose.pod.j2 updated 2026-04-20)
#   - DAM's agent_deploy reads channels.json from state volume
#   - Outbound HTTPS allowlisted (user-data.*.yml DOCKER-USER rules)
#
# Flows:
#   C1  Binary exposes `channels catalog` + `channels list` + `channels add`/`remove`
#   C2  Catalog lists telegram, discord, slack, line (minimum set)
#   C3  `channels list --pod N --json` parses cleanly before any add
#   C4  (LIVE) `channels add --pod N --type telegram --token ...` returns ok
#   C5  (LIVE) Pod's `/app/workspace/.tytus/channels.json` contains TELEGRAM_BOT_TOKEN
#   C6  (LIVE) Agent container's env contains TELEGRAM_BOT_TOKEN after redeploy
#   C7  (LIVE) Outbound HTTPS reaches api.telegram.org from within the pod
#   C8  (LIVE) `channels remove` clears credentials + redeploys
#
# Exit 0 on green.
# =============================================================================
set -u

export PATH="/bin:/usr/bin:/usr/local/bin:/Users/sebastian/bin:${PATH:-}"

POD=${TYTUS_E2E_POD:-02}
LIVE="${E2E_TELEGRAM_BOT_TOKEN:-}"
FAILS=0

pass() { printf '  \033[32m✓\033[0m %s\n' "$*"; }
fail() { printf '  \033[31m✗\033[0m %s\n' "$*"; FAILS=$((FAILS+1)); }
skip() { printf '  \033[33m⊘\033[0m %s\n' "$*"; }
section() { printf '\n\033[1m%s\033[0m\n' "$*"; }

section "C1  Binary surface"
if /Users/sebastian/bin/tytus channels --help >/dev/null 2>&1; then
    pass "tytus channels subcommand present"
else
    fail "tytus channels missing — binary out of date"
    exit 1
fi
for verb in catalog list add remove; do
    if /Users/sebastian/bin/tytus channels $verb --help >/dev/null 2>&1; then
        pass "subcommand $verb present"
    else
        fail "subcommand $verb missing"
    fi
done

section "C2  Catalog contains MVP channels"
CATALOG=$(/Users/sebastian/bin/tytus channels catalog --json 2>/dev/null || /Users/sebastian/bin/tytus --json channels catalog 2>/dev/null)
# Fall back to plain text grep if JSON mode not wired for catalog
if [[ -z "$CATALOG" ]]; then
    CATALOG=$(/Users/sebastian/bin/tytus channels catalog 2>/dev/null)
    mode=text
else
    mode=json
fi
for ch in telegram discord slack line; do
    if echo "$CATALOG" | grep -q "\"name\": \"$ch\"\|  $ch —"; then
        pass "channel '$ch' in catalog"
    else
        fail "channel '$ch' missing from catalog"
    fi
done

section "C3  Empty list prints cleanly"
OUT=$(/Users/sebastian/bin/tytus --json channels list --pod "$POD" 2>&1 || true)
if echo "$OUT" | python3 -c "import json,sys; d=json.load(sys.stdin); assert 'pod' in d and 'channels' in d" 2>/dev/null; then
    pass "channels list --json returns parseable JSON"
else
    fail "channels list --json output is not valid JSON: $OUT"
fi

if [[ -z "$LIVE" ]]; then
    section "Live flows (C4–C8) skipped"
    skip "Set E2E_TELEGRAM_BOT_TOKEN=<bot token> to run live end-to-end against pod $POD"
    echo
    if [[ "$FAILS" == 0 ]]; then
        echo "Static flows green ($FAILS failure(s))."
        exit 0
    else
        echo "$FAILS static flow(s) failed."
        exit 1
    fi
fi

section "C4  Live: tytus channels add telegram"
if /Users/sebastian/bin/tytus channels add --pod "$POD" --type telegram --token "$LIVE" 2>&1 | tee /tmp/tytus-channels-add.log | grep -q 'configured\|"ok":'; then
    pass "add command reports success"
else
    fail "add command did not report success — see /tmp/tytus-channels-add.log"
fi

section "C5  Live: channels.json on pod"
SLEEP_MAX=15
for _ in $(seq 1 $SLEEP_MAX); do
    if /Users/sebastian/bin/tytus exec --pod "$POD" --timeout 8 "cat /app/workspace/.tytus/channels.json 2>/dev/null" | grep -q TELEGRAM_BOT_TOKEN; then
        pass "channels.json on pod contains TELEGRAM_BOT_TOKEN"
        break
    fi
    sleep 1
done
if ! /Users/sebastian/bin/tytus exec --pod "$POD" --timeout 8 "cat /app/workspace/.tytus/channels.json 2>/dev/null" | grep -q TELEGRAM_BOT_TOKEN; then
    fail "TELEGRAM_BOT_TOKEN never appeared in pod's channels.json"
fi

section "C6  Live: container env has TELEGRAM_BOT_TOKEN"
if /Users/sebastian/bin/tytus exec --pod "$POD" --timeout 8 "env | grep -c '^TELEGRAM_BOT_TOKEN='" | grep -q '^1'; then
    pass "container env includes TELEGRAM_BOT_TOKEN"
else
    fail "TELEGRAM_BOT_TOKEN not in container env — DAM merge may have failed"
fi

section "C7  Live: pod reaches api.telegram.org over HTTPS"
CODE=$(/Users/sebastian/bin/tytus exec --pod "$POD" --timeout 10 \
    "curl -sSm 5 -o /dev/null -w '%{http_code}' https://api.telegram.org/bot$LIVE/getMe 2>&1" | tr -d '\r\n ')
if [[ "$CODE" == "200" ]]; then
    pass "pod can reach api.telegram.org (HTTP 200)"
else
    fail "pod couldn't reach api.telegram.org (got '$CODE' — is the pod-egress bridge deployed?)"
fi

section "C8  Live: channels remove clears everything"
if /Users/sebastian/bin/tytus channels remove --pod "$POD" --type telegram 2>&1 | grep -q 'removed\|"ok":'; then
    pass "remove command reports success"
else
    fail "remove command did not report success"
fi
sleep 5
if ! /Users/sebastian/bin/tytus exec --pod "$POD" --timeout 8 "env | grep '^TELEGRAM_BOT_TOKEN='" | grep -q TELEGRAM; then
    pass "TELEGRAM_BOT_TOKEN no longer in container env"
else
    fail "TELEGRAM_BOT_TOKEN still present after remove"
fi

echo
if [[ "$FAILS" == 0 ]]; then
    echo "All flows green."
    exit 0
else
    echo "$FAILS flow(s) failed."
    exit 1
fi

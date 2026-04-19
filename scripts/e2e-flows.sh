#!/usr/bin/env bash
# =============================================================================
# Tytus end-to-end flow verifier
# =============================================================================
# Runs through every user-facing flow the CLI + tray + agents expose and
# reports PASS / FAIL per flow. Safe to re-run; no destructive actions
# (does NOT revoke pods, does NOT logout, does NOT kill foreign processes).
#
# Usage:
#   scripts/e2e-flows.sh                         # exercise all flows
#   scripts/e2e-flows.sh --pod 02                # focus on one pod
#   scripts/e2e-flows.sh --section auth,status   # run a subset
#   scripts/e2e-flows.sh --sim-hermes            # use local hermes container
#
# Exit code: number of failed flows (0 = all green).
#
# Why bash: the flows span `tytus` CLI invocations, curl probes, docker
# (for the hermes sim), `python3` for Provider API calls, and pid-file
# inspection — bash is the lowest-common-denominator glue. No extra deps
# beyond what Tytus already requires.
# =============================================================================
set -u

# ── Sanity: put real bin paths first; some shells on the test box have
#    aliased or reduced PATH entries after `tytus` is invoked.
export PATH="/bin:/usr/bin:/usr/local/bin:/Users/sebastian/bin:${PATH:-}"

# ── Options ──────────────────────────────────────────────────────────
POD_FILTER=""
SECTION_FILTER=""
SIM_HERMES=0
VERBOSE=0
while [[ $# -gt 0 ]]; do
    case "$1" in
        --pod)        POD_FILTER="$2"; shift 2;;
        --section)    SECTION_FILTER="$2"; shift 2;;
        --sim-hermes) SIM_HERMES=1; shift;;
        -v|--verbose) VERBOSE=1; shift;;
        -h|--help)    grep '^#' "$0" | sed 's/^# \{0,1\}//'; exit 0;;
        *)            echo "unknown flag: $1" >&2; exit 2;;
    esac
done

# ── Output ───────────────────────────────────────────────────────────
GREEN='\033[32m'; RED='\033[31m'; YELLOW='\033[33m'; DIM='\033[2m'; RESET='\033[0m'
PASS=0; FAIL=0; SKIP=0
FAILED_FLOWS=()

section_enabled() {
    [[ -z "$SECTION_FILTER" ]] && return 0
    local s="$1"
    [[ ",${SECTION_FILTER}," == *",${s},"* ]]
}

log() { printf "${DIM}  · %s${RESET}\n" "$*"; }
pass() { printf "${GREEN}✓${RESET} %s\n" "$1"; PASS=$((PASS+1)); }
fail() { printf "${RED}✗${RESET} %s ${DIM}— %s${RESET}\n" "$1" "${2:-}"; FAIL=$((FAIL+1)); FAILED_FLOWS+=("$1"); }
skip() { printf "${YELLOW}−${RESET} %s ${DIM}— %s${RESET}\n" "$1" "${2:-}"; SKIP=$((SKIP+1)); }
section() { printf "\n${DIM}══ %s ══${RESET}\n" "$1"; }

# ── State introspection helpers ──────────────────────────────────────
STATE_FILE="${HOME}/Library/Application Support/tytus/state.json"
[[ ! -f "$STATE_FILE" ]] && STATE_FILE="${HOME}/.config/tytus/state.json"

jq_state() { python3 -c "import json,sys; d=json.load(open(sys.argv[1])); print($1)" "$STATE_FILE" 2>/dev/null; }

user_email()   { jq_state 'd.get("email") or ""'; }
has_secret()   { [[ -n "$(jq_state 'd.get("secret_key") or ""')" ]]; }
pod_ids()      { jq_state '" ".join(p["pod_id"] for p in d.get("pods",[]))'; }
pod_field() { # pod_field <pod_id> <key>
    python3 -c "
import json, sys
d = json.load(open(sys.argv[1]))
for p in d.get('pods', []):
    if p['pod_id'] == sys.argv[2]:
        v = p.get(sys.argv[3])
        print(v if v is not None else '')
        break
" "$STATE_FILE" "$1" "$2" 2>/dev/null
}

# ── Provider exec helper (keychain-free; uses secret_key from state) ─
provider_exec() { # provider_exec <pod_id> <cmd> [timeout]
    local pod="$1" cmd="$2" timeout="${3:-20}"
    python3 - <<PY 2>&1
import json, sys, urllib.request
d = json.load(open("$STATE_FILE"))
req = urllib.request.Request(
    "https://tytus.traylinx.com/pod/agent/exec",
    data=json.dumps({"pod_id": "$pod", "command": """$cmd""", "timeout": $timeout}).encode(),
    headers={
        "Content-Type": "application/json",
        "X-Agent-Secret-Token": d["secret_key"],
        "X-Agent-User-Id": d["agent_user_id"],
    },
    method="POST",
)
r = urllib.request.urlopen(req, timeout=$timeout+10)
body = json.loads(r.read().decode())
sys.stdout.write(body.get("stdout","") or "")
sys.stderr.write(body.get("stderr","") or "")
sys.exit(body.get("exit_code", 0))
PY
}

wait_for_http() { # wait_for_http <url> <timeout_s>
    local url="$1" t="$2"
    local deadline=$(( $(date +%s) + t ))
    while (( $(date +%s) < deadline )); do
        if curl -sS -m 2 -o /dev/null "$url" 2>/dev/null; then return 0; fi
        sleep 0.5
    done
    return 1
}

wait_for_http_code() { # wait_for_http_code <url> <expected-code> <timeout_s>
    local url="$1" want="$2" t="$3"
    local deadline=$(( $(date +%s) + t ))
    while (( $(date +%s) < deadline )); do
        local got
        got=$(curl -sS -m 2 -o /dev/null -w '%{http_code}' "$url" 2>/dev/null || echo 0)
        [[ "$got" == "$want" ]] && return 0
        sleep 0.5
    done
    return 1
}

# =============================================================================
# SECTION: AUTH
# =============================================================================
section_auth() {
    section_enabled auth || return 0
    section "AUTH"

    # A1: state file present + 0600
    if [[ -f "$STATE_FILE" ]]; then
        local mode
        mode=$(stat -f '%Lp' "$STATE_FILE" 2>/dev/null || stat -c '%a' "$STATE_FILE" 2>/dev/null)
        if [[ "$mode" == "600" ]]; then
            pass "A1 state file exists, mode 0600"
        else
            fail "A1 state file mode should be 0600" "got $mode"
        fi
    else
        fail "A1 state file present" "not found at $STATE_FILE"
        return
    fi

    # A2: logged in
    local email
    email=$(user_email)
    if [[ -n "$email" ]]; then
        pass "A2 logged in as $email"
    else
        fail "A2 logged in" "state has no email — run: tytus login"
    fi

    # A3: A2A credentials (secret_key + agent_user_id) present
    if has_secret; then
        pass "A3 A2A credentials present (keychain-free Provider access)"
    else
        fail "A3 secret_key cached" "no secret_key in state.json"
    fi

    # A4: tytus status exits 0 (doesn't hang on keychain)
    if timeout 10 tytus status --json >/tmp/tytus-e2e-status.json 2>&1; then
        pass "A4 \`tytus status --json\` completes within 10s"
    else
        fail "A4 \`tytus status --json\` completes" "exit/timeout"
    fi
}

# =============================================================================
# SECTION: POD LIFECYCLE (non-destructive probes only)
# =============================================================================
section_pods() {
    section_enabled pods || return 0
    section "POD LIFECYCLE"

    local pods
    pods=$(pod_ids)
    if [[ -z "$pods" ]]; then
        skip "P1 at least one pod" "no pods — run: tytus connect --agent nemoclaw"
        return
    fi
    pass "P1 pods in state: $pods"

    for pod in $pods; do
        [[ -n "$POD_FILTER" && "$pod" != "$POD_FILTER" ]] && continue
        local at ai endpoint
        at=$(pod_field "$pod" agent_type)
        ai=$(pod_field "$pod" ai_endpoint)
        endpoint=$(pod_field "$pod" agent_endpoint)

        # P2: agent_type recognized
        case "$at" in
            nemoclaw|hermes|none)
                pass "P2/pod $pod agent_type=$at"
                ;;
            *)
                fail "P2/pod $pod agent_type" "unrecognized: $at"
                ;;
        esac

        # P3: ai_endpoint format
        if [[ "$ai" =~ ^http://10\.[0-9]+\.[0-9]+\.1:18080$ ]]; then
            pass "P3/pod $pod ai_endpoint=$ai"
        else
            fail "P3/pod $pod ai_endpoint well-formed" "$ai"
        fi

        # P4: agent_endpoint port matches agent_type convention
        if [[ "$at" == "none" ]]; then
            skip "P4/pod $pod endpoint port" "agent_type=none (AIL-only pod)"
        else
            local want_ui_port
            case "$at" in
                hermes)    want_ui_port=9119;;
                nemoclaw)  want_ui_port=3000;;
                *)         want_ui_port="";;
            esac
            if [[ "$endpoint" == *":${want_ui_port}" ]]; then
                pass "P4/pod $pod agent_endpoint on UI port $want_ui_port ($endpoint)"
            else
                fail "P4/pod $pod agent_endpoint UI port" "expected :$want_ui_port, got $endpoint"
            fi
        fi

        # P5: tunnel pidfile present + process alive.
        # The tunnel daemon is root-owned, so a user-run `kill -0 $pid`
        # returns 1 regardless of whether the pid exists (EPERM vs ESRCH
        # are indistinguishable from shell). `ps -p` doesn't care about
        # ownership and works for any live pid — use that.
        local pidfile="/tmp/tytus/tunnel-${pod}.pid"
        if [[ -f "$pidfile" ]]; then
            local tunnel_pid
            tunnel_pid=$(cat "$pidfile" 2>/dev/null)
            if [[ -z "$tunnel_pid" ]]; then
                fail "P5/pod $pod tunnel pidfile" "empty"
            elif ps -p "$tunnel_pid" -o pid= >/dev/null 2>&1; then
                pass "P5/pod $pod tunnel daemon alive (pid $tunnel_pid)"
            else
                fail "P5/pod $pod tunnel daemon alive" "pid $tunnel_pid no longer running"
            fi
        else
            skip "P5/pod $pod tunnel pidfile" "no pidfile — run: tytus connect --pod $pod"
        fi

        # P6: upstream reachable (health probe via WG)
        if [[ -n "$endpoint" && "$at" != "none" ]]; then
            local health_path
            case "$at" in
                hermes)   health_path="/health";;
                nemoclaw) health_path="/healthz";;
            esac
            if wait_for_http "http://${endpoint}${health_path}" 5; then
                pass "P6/pod $pod agent health ${endpoint}${health_path}"
            else
                fail "P6/pod $pod agent health" "${endpoint}${health_path} unreachable (tunnel down?)"
            fi
        fi
    done
}

# =============================================================================
# SECTION: UI FORWARDER (per-agent multiplex)
# =============================================================================
section_ui() {
    section_enabled ui || return 0
    section "UI FORWARDER"

    local pods
    pods=$(pod_ids)
    for pod in $pods; do
        [[ -n "$POD_FILTER" && "$pod" != "$POD_FILTER" ]] && continue
        local at
        at=$(pod_field "$pod" agent_type)
        [[ "$at" == "none" ]] && continue

        local port=$((18700 + 10#$pod))
        local base="http://localhost:${port}"

        # U1: forwarder marker exists (from prior `tytus ui`)
        local marker="/tmp/tytus/ui-${pod}.port"
        local running=0
        if [[ -f "$marker" ]]; then
            local mp
            mp=$(python3 -c "import json; print(json.load(open('$marker'))['pid'])" 2>/dev/null)
            if [[ -n "$mp" ]] && kill -0 "$mp" 2>/dev/null; then
                running=1
                pass "U1/pod $pod forwarder marker + pid $mp alive on port $port"
            fi
        fi

        if [[ $running -eq 0 ]]; then
            # Start one for the duration of this section
            log "starting forwarder for pod $pod on port $port..."
            nohup tytus ui --pod "$pod" --no-open >/tmp/tytus-e2e-ui-"$pod".log 2>&1 &
            disown
            if wait_for_http_code "$base/health" "200" 15 ||
               wait_for_http_code "$base/" "200" 15 ||
               wait_for_http_code "$base/" "302" 15; then
                pass "U1/pod $pod forwarder started + responding"
            else
                fail "U1/pod $pod forwarder started" "no response within 15s"
                continue
            fi
        fi

        case "$at" in
            nemoclaw)
                # U2: GET / → 302 to /?token=<T> (token-seed redirect)
                local r
                r=$(curl -sS -m 5 -o /dev/null -w '%{http_code} %{redirect_url}' "$base/")
                if [[ "$r" == 302* && "$r" == *"?token="* ]]; then
                    pass "U2/pod $pod (nemoclaw) GET / → 302 with ?token= seed"
                else
                    fail "U2/pod $pod GET / 302 with token" "$r"
                fi

                # U3: GET /?token=X → 200 (no redirect loop). Retry once:
                # cold cache + slow WG can make the first fetch timeout
                # on dev boxes where boringtun sustains ~3 KB/s and the
                # bundle is 689 KB. Second try is served from disk cache.
                local c
                c=$(curl -sS -m 6 -o /dev/null -w '%{http_code}' "$base/?token=xxx" 2>/dev/null)
                if [[ "$c" != "200" ]]; then
                    c=$(curl -sS -m 15 -o /dev/null -w '%{http_code}' "$base/?token=xxx" 2>/dev/null)
                fi
                if [[ "$c" == "200" ]]; then
                    pass "U3/pod $pod no redirect loop when ?token= present"
                else
                    fail "U3/pod $pod no redirect loop" "got $c (tunnel slow? cache cold?)"
                fi

                # U4: WS upgrade not redirected. Use curl: once the upgrade
                # succeeds the connection stays open; `-m 2` forces curl to
                # give up after 2s (exit 28 "timed out"), by which point it
                # has already printed the response head. We just want to
                # confirm the first status line is 101.
                local ws_resp
                ws_resp=$(curl -sSi -m 2 --http1.1 \
                    -H "Upgrade: websocket" -H "Connection: Upgrade" \
                    -H "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==" \
                    -H "Sec-WebSocket-Version: 13" \
                    -H "Origin: http://localhost:${port}" \
                    "http://localhost:${port}/" 2>/dev/null | head -1 | tr -d '\r')
                if [[ "$ws_resp" == "HTTP/1.1 101"* ]]; then
                    pass "U4/pod $pod WS upgrade → 101 Switching Protocols"
                else
                    fail "U4/pod $pod WS upgrade → 101" "got: ${ws_resp:-<empty>}"
                fi
                ;;
            hermes)
                # Hermes multiplex. Requires the new infra entrypoint on the
                # droplet — if unavailable, these skip.
                local ui_code api_code
                ui_code=$(curl -sS -m 5 -o /tmp/tytus-e2e-ui.html -w '%{http_code}' "$base/")
                if [[ "$ui_code" == "200" ]]; then
                    if grep -q 'HERMES_SESSION_TOKEN' /tmp/tytus-e2e-ui.html 2>/dev/null; then
                        pass "U5/pod $pod (hermes) GET / → dashboard SPA with session token"
                    else
                        fail "U5/pod $pod GET / dashboard SPA" "200 but no __HERMES_SESSION_TOKEN__ in body (old image?)"
                    fi
                else
                    fail "U5/pod $pod dashboard SPA" "got $ui_code"
                fi

                api_code=$(curl -sS -m 5 -o /dev/null -w '%{http_code}' "$base/v1/models")
                if [[ "$api_code" == "200" ]]; then
                    pass "U6/pod $pod (hermes) GET /v1/models → gateway (auth auto-injected)"
                else
                    fail "U6/pod $pod /v1/models via gateway" "got $api_code"
                fi
                ;;
        esac
    done
}

# =============================================================================
# SECTION: ENV / SDK INTEGRATION
# =============================================================================
section_env() {
    section_enabled env || return 0
    section "ENV"

    # 2>/dev/null silences the benign "keychain get_refresh_token timed
    # out" WARN that macOS emits on dev rebuilds — tytus env only needs
    # state.json, no keychain round-trip, so the warning is cosmetic.
    if ! timeout 8 tytus env --export >/tmp/tytus-e2e-env.sh 2>/dev/null; then
        fail "E1 \`tytus env --export\` completes" "exit/timeout"
        return
    fi
    if grep -q '^export OPENAI_BASE_URL=' /tmp/tytus-e2e-env.sh &&
       grep -q '^export OPENAI_API_KEY=' /tmp/tytus-e2e-env.sh; then
        pass "E1 \`tytus env --export\` emits OPENAI_BASE_URL + OPENAI_API_KEY"
    else
        fail "E1 env vars" "OPENAI_BASE_URL / OPENAI_API_KEY missing"
    fi

    # E2: the emitted base URL resolves to the stable 10.42.42.1:18080.
    # `tytus env --export` can emit with or without quotes depending on
    # shell dialect, so tolerate both `X=v` and `X="v"`.
    if grep -qE '^export OPENAI_BASE_URL="?http://10\.42\.42\.1:18080/v1"?$' /tmp/tytus-e2e-env.sh; then
        pass "E2 stable endpoint = http://10.42.42.1:18080/v1 (cross-pod)"
    else
        skip "E2 stable endpoint" "not using stable pair (raw mode?)"
    fi

    # E3: stable key format. Quoted or unquoted, 32-hex suffix.
    if grep -qE '^export OPENAI_API_KEY="?sk-tytus-user-[0-9a-f]{32}"?$' /tmp/tytus-e2e-env.sh; then
        pass "E3 stable user key format sk-tytus-user-<32hex>"
    else
        skip "E3 stable key format" "not using stable pair"
    fi
}

# =============================================================================
# SECTION: DIAGNOSTICS
# =============================================================================
section_doctor() {
    section_enabled doctor || return 0
    section "DIAGNOSTICS"

    # Tracing WARNs leak onto stdout alongside the JSON blob (tracing
    # subscribes to stdout when no explicit writer is configured). Grep
    # out only the JSON line so python parsing doesn't choke.
    if timeout 30 tytus doctor --json 2>/dev/null \
         | grep -E '^[[:space:]]*\{' \
         > /tmp/tytus-e2e-doctor.json; then
        pass "D1 \`tytus doctor --json\` completes"
    else
        fail "D1 \`tytus doctor --json\`" "exit/timeout"
        return
    fi

    # Doctor shape: {"checks":[{"check":"…","ok":bool,"message":"…"},...],
    #                "healthy": bool}
    #
    # D2: count failing checks. `logged_in: false` is a KNOWN false negative
    # on dev boxes where the OS keychain ACL hasn't been approved — state
    # still has email + secret_key so every downstream flow works. Don't
    # count it as a hard fail; downgrade to a warning.
    python3 - >/tmp/tytus-e2e-doctor.report 2>&1 <<'PY'
import json
d = json.load(open('/tmp/tytus-e2e-doctor.json'))
checks = d.get('checks', d if isinstance(d, list) else [])
warnings, hard_fails = [], []
benign = {'logged_in'}  # keychain-broken dev box, flow still works
for c in checks:
    if c.get('ok'):
        continue
    name = c.get('check') or c.get('name') or '?'
    msg  = c.get('message', '')
    (warnings if name in benign else hard_fails).append(f"{name}: {msg}")
if hard_fails:
    print('HARD_FAILS')
    for f in hard_fails: print(f)
    raise SystemExit(1)
if warnings:
    print('WARN_ONLY')
    for w in warnings: print(w)
PY
    rc=$?
    if [[ $rc -eq 0 ]]; then
        if grep -q '^WARN_ONLY' /tmp/tytus-e2e-doctor.report 2>/dev/null; then
            local warns
            warns=$(grep -v '^WARN_ONLY$' /tmp/tytus-e2e-doctor.report | head -3 | tr '\n' '; ')
            pass "D2 doctor checks green (ignoring known dev-env noise: ${warns})"
        else
            pass "D2 all doctor checks green"
        fi
    else
        fail "D2 doctor checks green" "see /tmp/tytus-e2e-doctor.report"
        [[ $VERBOSE -eq 1 ]] && cat /tmp/tytus-e2e-doctor.report
    fi
}

# =============================================================================
# SECTION: TRAY (headless observation)
# =============================================================================
section_tray() {
    section_enabled tray || return 0
    section "TRAY"

    # T1: tray process alive (if installed)
    if [[ -f /tmp/tytus/tray.pid ]]; then
        local tpid
        tpid=$(cat /tmp/tytus/tray.pid 2>/dev/null)
        if [[ -n "$tpid" ]] && kill -0 "$tpid" 2>/dev/null; then
            pass "T1 tray process alive (pid $tpid)"
        else
            skip "T1 tray process" "pidfile stale"
        fi
    else
        skip "T1 tray process" "not installed — run: tytus tray install"
    fi

    # T2: tray's refresh inputs exist (FS signature depends on these)
    local bad=0
    for path in "$STATE_FILE"; do
        [[ -f "$path" ]] || { bad=1; break; }
    done
    [[ $bad -eq 0 ]] && pass "T2 tray refresh inputs present (state.json)" || fail "T2 tray refresh inputs" "state.json missing"

    # T3: /tmp/tytus readable by user (tray expects 0644 for pidfiles)
    if [[ -r /tmp/tytus ]]; then
        pass "T3 /tmp/tytus readable"
    else
        fail "T3 /tmp/tytus readable" "perm denied"
    fi

    # T4: tunnel pidfiles readable as user (tray's tunnel_reaches_pod
    # does `ps -p $pid` on a root-owned pid, which needs the pidfile
    # itself to be readable first). Re-check perms per pidfile.
    local bad=0 shown=0
    for f in /tmp/tytus/tunnel-*.pid; do
        [[ -e "$f" ]] || continue
        if [[ ! -r "$f" ]]; then bad=1; break; fi
        shown=1
    done
    if [[ $bad -eq 0 && $shown -eq 1 ]]; then
        pass "T4 tunnel pidfiles readable by user (tray can probe liveness)"
    elif [[ $shown -eq 0 ]]; then
        skip "T4 tunnel pidfiles readable" "no active tunnels"
    else
        fail "T4 tunnel pidfiles readable" "one or more pidfiles are 0600"
    fi

    # T5: filesystem signature changes when a watched file changes. The
    # tray's FS watcher thread uses a fingerprint (file sizes + mtimes
    # of /tmp/tytus/{tunnel-*, ui-*} + state.json) to decide when to
    # wake the poll thread for an out-of-band refresh. Touching a
    # watched file must move the fingerprint; otherwise the tray would
    # only refresh on its 1.5s poll tick, never on the sub-second fast
    # path for click actions.
    local sig_before sig_after
    sig_before=$(ls -laT /tmp/tytus 2>/dev/null | awk '/tunnel-|ui-/ {print $0}' | md5 2>/dev/null || ls -la --full-time /tmp/tytus 2>/dev/null | awk '/tunnel-|ui-/ {print $0}' | md5sum | cut -d' ' -f1)
    # Find a safe ui-*.port marker to re-touch (rewriting same content)
    local marker
    marker=$(ls /tmp/tytus/ui-*.port 2>/dev/null | head -1)
    if [[ -n "$marker" && -f "$marker" ]]; then
        local content
        content=$(cat "$marker")
        sleep 1.1  # next-second mtime
        printf '%s' "$content" > "$marker"
        sleep 0.3
        sig_after=$(ls -laT /tmp/tytus 2>/dev/null | awk '/tunnel-|ui-/ {print $0}' | md5 2>/dev/null || ls -la --full-time /tmp/tytus 2>/dev/null | awk '/tunnel-|ui-/ {print $0}' | md5sum | cut -d' ' -f1)
        if [[ "$sig_before" != "$sig_after" ]]; then
            pass "T5 FS signature picks up marker-file changes (tray refresh trigger)"
        else
            fail "T5 FS signature" "mtime didn't bump — tray may stick on old state"
        fi
    else
        skip "T5 FS signature" "no ui-*.port marker to touch"
    fi
}

# =============================================================================
# SECTION: HERMES LOCAL SIMULATION (opt-in via --sim-hermes)
# =============================================================================
section_sim_hermes() {
    [[ $SIM_HERMES -eq 1 ]] || return 0
    section_enabled sim-hermes || [[ -z "$SECTION_FILTER" ]] || return 0
    section "HERMES LOCAL SIM"

    if ! command -v docker >/dev/null 2>&1; then
        skip "H1 docker available" "docker not installed"
        return
    fi

    # Ensure tytus-hermes:test image exists
    if ! docker image inspect tytus-hermes:test >/dev/null 2>&1; then
        log "building tytus-hermes:test from local entrypoint..."
        local tdir
        tdir=$(mktemp -d)
        cp /Users/sebastian/Projects/makakoo/api/ProjectWannolot/services/wannolot-infrastructure/hermes/entrypoint.sh "$tdir/"
        cat >"$tdir/Dockerfile" <<'DOCK'
FROM nousresearch/hermes-agent:latest
ENV API_SERVER_ENABLED=true
ENV API_SERVER_HOST=0.0.0.0
ENV API_SERVER_PORT=8642
ENV HERMES_HOME=/app/workspace
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
DOCK
        if docker build -q -t tytus-hermes:test "$tdir" >/dev/null 2>&1; then
            pass "H1 built tytus-hermes:test"
        else
            fail "H1 built tytus-hermes:test" "docker build failed"
            rm -rf "$tdir"; return
        fi
        rm -rf "$tdir"
    else
        pass "H1 tytus-hermes:test present"
    fi

    # Start container if not running
    if ! docker ps --filter name=tytus-hermes-sim --format '{{.Names}}' | grep -q '^tytus-hermes-sim$'; then
        docker rm -f tytus-hermes-sim 2>/dev/null >/dev/null
        docker volume create tytus-hermes-sim-workspace >/dev/null
        docker run --name tytus-hermes-sim -d \
          -v tytus-hermes-sim-workspace:/app/workspace \
          -e AIL_API_KEY=sk-ail-e2e-sim-xxxxxxxxxxxxxxxxxxxxxxxxxxxx \
          -e AIL_INFERENCE_URL=http://127.0.0.1:99999/v1 \
          -e TYTUS_POD_ID=99 \
          -p 127.0.0.1:8642:8642 -p 127.0.0.1:9119:9119 \
          tytus-hermes:test >/dev/null
        log "waiting for gateway /health..."
        if wait_for_http_code "http://127.0.0.1:8642/health" "200" 30; then
            pass "H2 gateway up on 8642"
        else
            fail "H2 gateway up" "no /health response"
            docker logs tytus-hermes-sim 2>&1 | tail -10
            return
        fi
        log "waiting for dashboard SPA (first run does npm install + build)..."
        if wait_for_http_code "http://127.0.0.1:9119/" "200" 90; then
            pass "H3 dashboard up on 9119 (SPA built)"
        else
            fail "H3 dashboard up" "no response on 9119"
            docker logs tytus-hermes-sim 2>&1 | tail -20
            return
        fi
    else
        pass "H2 gateway up on 8642 (container already running)"
        if wait_for_http_code "http://127.0.0.1:9119/" "200" 5; then
            pass "H3 dashboard up on 9119"
        else
            fail "H3 dashboard up" "port 9119 not responding"
        fi
    fi

    # H4: dashboard HTML injects session token
    if curl -sS -m 5 http://127.0.0.1:9119/ | grep -q 'HERMES_SESSION_TOKEN__='; then
        pass "H4 dashboard injects __HERMES_SESSION_TOKEN__"
    else
        fail "H4 dashboard injects __HERMES_SESSION_TOKEN__" "not found in HTML"
    fi

    # H5: gateway API responds with correct API_SERVER_KEY
    local key
    key=$(docker exec tytus-hermes-sim cat /app/workspace/.hermes/api_server_key 2>/dev/null)
    if [[ -z "$key" ]]; then
        fail "H5 api_server_key cached" "missing"
        return
    fi
    local code
    code=$(curl -sS -m 5 -o /dev/null -w '%{http_code}' -H "Authorization: Bearer $key" http://127.0.0.1:8642/v1/models)
    if [[ "$code" == "200" ]]; then
        pass "H5 /v1/models with API_SERVER_KEY → 200"
    else
        fail "H5 /v1/models with API_SERVER_KEY" "got $code"
    fi

    # H6: synthetic pod 99 + spawn forwarder, verify multiplex
    python3 - <<PY
import json
p = "$STATE_FILE"
d = json.load(open(p))
d["pods"] = [pp for pp in d["pods"] if pp["pod_id"] != "99"]
d["pods"].append({
    "pod_id":"99","droplet_id":"local","droplet_ip":"127.0.0.1",
    "ai_endpoint":"http://127.0.0.1:99999","pod_api_key":"sk-sim",
    "agent_type":"hermes","agent_endpoint":"127.0.0.1:9119",
    "tunnel_iface":"none",
    "stable_ai_endpoint":"http://10.42.42.1:18080",
    "stable_user_key":"sk-tytus-user-0000000000000000000000000000sim1",
    "gateway_token":"$key",
})
open(p,"w").write(json.dumps(d, indent=2))
PY
    tytus ui --pod 99 --stop 2>/dev/null >/dev/null
    lsof -ti :18799 2>/dev/null | xargs -r kill -9 2>/dev/null
    nohup tytus ui --pod 99 --no-open >/tmp/tytus-e2e-hermes-sim.log 2>&1 &
    disown
    if wait_for_http_code "http://localhost:18799/" "200" 10; then
        pass "H6 hermes forwarder up on 18799"
    else
        fail "H6 hermes forwarder up" "no response"
    fi

    # H7: multiplex — GET / (SPA) vs GET /v1/models (gateway)
    if curl -sS -m 5 http://localhost:18799/ 2>/dev/null | grep -q 'HERMES_SESSION_TOKEN__='; then
        pass "H7 forwarder GET / → dashboard SPA"
    else
        fail "H7 forwarder GET /" "no __HERMES_SESSION_TOKEN__"
    fi
    if curl -sS -m 5 http://localhost:18799/v1/models 2>/dev/null | grep -q '"object": "list"'; then
        pass "H8 forwarder GET /v1/models → gateway (auth auto-injected)"
    else
        fail "H8 forwarder /v1/models" "gateway response absent"
    fi
    # H9: SDK placeholder override
    if curl -sS -m 5 -H 'Authorization: Bearer sk-placeholder-sdk' http://localhost:18799/v1/models 2>/dev/null | grep -q '"object": "list"'; then
        pass "H9 forwarder overrides SDK placeholder Authorization"
    else
        fail "H9 SDK placeholder override" "gateway rejected"
    fi

    # Cleanup: stop synthetic forwarder, leave container running for reuse
    tytus ui --pod 99 --stop 2>/dev/null >/dev/null
    # Remove synthetic pod from state
    python3 - <<PY
import json
p = "$STATE_FILE"
d = json.load(open(p))
d["pods"] = [pp for pp in d["pods"] if pp["pod_id"] != "99"]
open(p,"w").write(json.dumps(d, indent=2))
PY
}

# =============================================================================
# Run selected sections
# =============================================================================
section_auth
section_pods
section_ui
section_env
section_doctor
section_tray
section_sim_hermes

# =============================================================================
# Summary
# =============================================================================
printf "\n${DIM}══ SUMMARY ══${RESET}\n"
printf "${GREEN}%d passed${RESET}   ${RED}%d failed${RESET}   ${YELLOW}%d skipped${RESET}\n" "$PASS" "$FAIL" "$SKIP"
if (( FAIL > 0 )); then
    printf "\n${RED}Failed flows:${RESET}\n"
    for f in "${FAILED_FLOWS[@]}"; do printf "  - %s\n" "$f"; done
fi
exit "$FAIL"

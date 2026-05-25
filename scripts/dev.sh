#!/usr/bin/env bash
# dev.sh — flip into local-development mode.
#
# In dev mode:
#   • Production LaunchAgent tray is stopped.
#   • A new tray instance is started with TYTUS_TRAY_PORT=4343 so it
#     binds the sidecar port and writes 4343 into /tmp/tytus/tray-web.port.
#   • Vite dev server starts on http://localhost:4242 — the canonical
#     Tytus OS URL. Its built-in proxy forwards /api/* to the sidecar
#     tray via the tray-web.port file.
#
# Result: open http://localhost:4242/ in the browser and you get HMR
# for every TypeScript/React edit while the daemon/tray keep serving
# real /api/* responses behind the scenes.
#
# Usage:
#   scripts/dev.sh start   # default — bring dev mode up
#   scripts/dev.sh stop    # tear it down, restore the production tray
#   scripts/dev.sh status  # show what's running
#
# Rust changes still need scripts/dev-rebuild.sh — that script copies
# fresh binaries to /usr/local/bin/ and reinstalls the tray. After
# dev-rebuild.sh, re-run this script if you want dev mode back.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TYTUS_OS_APP_DIR="$REPO_ROOT/../tytus-os/app"

DEV_TRAY_PORT="${TYTUS_TRAY_PORT:-4343}"
DEV_VITE_PORT=4242

DEV_DIR="/tmp/tytus-dev"
DEV_TRAY_PID_FILE="$DEV_DIR/tray.pid"
DEV_VITE_PID_FILE="$DEV_DIR/vite.pid"
DEV_DAEMON_PID_FILE="$DEV_DIR/daemon.pid"
DEV_TRAY_LOG="$DEV_DIR/tray.log"
DEV_VITE_LOG="$DEV_DIR/vite.log"
DEV_DAEMON_LOG="$DEV_DIR/daemon.log"

PROD_TRAY_PLIST="$HOME/Library/LaunchAgents/com.traylinx.tytus.tray.plist"
PROD_DAEMON_PLIST="$HOME/Library/LaunchAgents/com.traylinx.tytus.daemon.plist"

mkdir -p "$DEV_DIR"

step() { printf "\n\033[1;36m▸ %s\033[0m\n" "$*"; }
ok()   { printf "  \033[1;32m✓\033[0m %s\n" "$*"; }
warn() { printf "  \033[1;33m!\033[0m %s\n" "$*"; }

is_running() {
  local pidfile="$1"
  [[ -f "$pidfile" ]] && kill -0 "$(cat "$pidfile" 2>/dev/null)" 2>/dev/null
}

stop_pidfile() {
  local pidfile="$1"
  local label="$2"
  if is_running "$pidfile"; then
    local pid
    pid="$(cat "$pidfile")"
    kill "$pid" 2>/dev/null || true
    # Wait up to 5s for graceful shutdown.
    for _ in $(seq 1 10); do
      if ! kill -0 "$pid" 2>/dev/null; then break; fi
      sleep 0.5
    done
    if kill -0 "$pid" 2>/dev/null; then
      kill -9 "$pid" 2>/dev/null || true
    fi
    rm -f "$pidfile"
    ok "$label stopped (pid $pid)"
  else
    rm -f "$pidfile" 2>/dev/null || true
  fi
}

cmd_start() {
  step "Stopping production tray + daemon LaunchAgents"
  if [[ -f "$PROD_TRAY_PLIST" ]]; then
    launchctl bootout "gui/$(id -u)/com.traylinx.tytus.tray" 2>/dev/null || true
    ok "production tray LaunchAgent unloaded"
  fi
  if [[ -f "$PROD_DAEMON_PLIST" ]]; then
    launchctl bootout "gui/$(id -u)/com.traylinx.tytus.daemon" 2>/dev/null || true
    ok "production daemon LaunchAgent unloaded"
  fi
  # Belt-and-braces: kill any straggler tray bound to 4242 or 4343.
  pkill -f '/Applications/Tytus.app/Contents/MacOS/Tytus' 2>/dev/null || true
  pkill -f '^/usr/local/bin/tytus-tray' 2>/dev/null || true
  # The daemon plist's KeepAlive was triggering on every test run —
  # leave it loaded but stopped so a `tytus daemon run` we start
  # ourselves doesn't fight launchd.
  tytus daemon stop 2>/dev/null | tail -1 || true
  sleep 1

  step "Starting dev daemon (token refresh + pod status sync)"
  TYTUS_HEADLESS=1 \
    nohup /usr/local/bin/tytus daemon run >"$DEV_DAEMON_LOG" 2>&1 &
  echo $! >"$DEV_DAEMON_PID_FILE"
  # Wait for the daemon socket to appear so the tray's first poll succeeds.
  for _ in $(seq 1 20); do
    if [[ -S /tmp/tytus/daemon.sock ]]; then break; fi
    sleep 0.5
  done
  if [[ -S /tmp/tytus/daemon.sock ]]; then
    ok "daemon socket up (pid $(cat "$DEV_DAEMON_PID_FILE"))"
  else
    warn "daemon socket missing — token refresh and pod status won't work"
  fi

  step "Starting dev tray on TYTUS_TRAY_PORT=$DEV_TRAY_PORT"
  TYTUS_TRAY_PORT="$DEV_TRAY_PORT" \
  TYTUS_HEADLESS=1 \
    nohup /usr/local/bin/tytus-tray >"$DEV_TRAY_LOG" 2>&1 &
  echo $! >"$DEV_TRAY_PID_FILE"
  # Wait for the tray to bind and write the port file.
  for _ in $(seq 1 20); do
    if [[ -f /tmp/tytus/tray-web.port ]]; then
      local p
      p="$(cat /tmp/tytus/tray-web.port 2>/dev/null || true)"
      if [[ "$p" == "$DEV_TRAY_PORT" ]]; then break; fi
    fi
    sleep 0.5
  done
  if curl -fsS --max-time 1 "http://127.0.0.1:$DEV_TRAY_PORT/api/state" >/dev/null 2>&1; then
    ok "dev tray responding on :$DEV_TRAY_PORT (pid $(cat "$DEV_TRAY_PID_FILE"))"
  else
    warn "dev tray didn't respond on :$DEV_TRAY_PORT — check $DEV_TRAY_LOG"
  fi

  step "Starting Vite dev server on http://localhost:$DEV_VITE_PORT"
  (
    cd "$TYTUS_OS_APP_DIR"
    nohup npm run dev >"$DEV_VITE_LOG" 2>&1 &
    echo $! >"$DEV_VITE_PID_FILE"
  )
  for _ in $(seq 1 60); do
    if curl -fsS --max-time 1 "http://localhost:$DEV_VITE_PORT/" >/dev/null 2>&1; then
      ok "vite ready on http://localhost:$DEV_VITE_PORT (pid $(cat "$DEV_VITE_PID_FILE"))"
      break
    fi
    sleep 0.5
  done

  # Sanity-check the /api proxy.
  local proxy_status
  proxy_status="$(curl -fsS -o /dev/null -w '%{http_code}' \
    -H 'Sec-Fetch-Site: same-origin' \
    "http://localhost:$DEV_VITE_PORT/api/state" || true)"
  if [[ "$proxy_status" == "200" ]]; then
    ok "vite → tray proxy verified (/api/state = 200)"
  else
    warn "vite → tray proxy returned HTTP $proxy_status — check $DEV_VITE_LOG"
  fi

  cat <<NEXT

────────────────────────────────────────────────────────────────────
✓ Dev mode is live.

  Open in browser:   http://localhost:$DEV_VITE_PORT/
  Tray sidecar:      http://localhost:$DEV_TRAY_PORT/ (debug only)
  Vite log:          tail -f $DEV_VITE_LOG
  Tray log:          tail -f $DEV_TRAY_LOG

Frontend edits in services/tytus-os/app/src/ auto-reload via HMR.
For Rust changes:        scripts/dev-rebuild.sh
To leave dev mode:       scripts/dev.sh stop
────────────────────────────────────────────────────────────────────
NEXT
}

cmd_stop() {
  step "Stopping dev mode"
  stop_pidfile "$DEV_VITE_PID_FILE" "vite"
  stop_pidfile "$DEV_TRAY_PID_FILE" "dev tray"
  stop_pidfile "$DEV_DAEMON_PID_FILE" "dev daemon"
  # Belt-and-braces: kill any other vite or alt-port tray.
  pkill -f 'vite' 2>/dev/null || true
  pkill -f 'TYTUS_TRAY_PORT=' 2>/dev/null || true
  # Also kill a foreground `tytus daemon run` we spawned. The LaunchAgent
  # version (if running) will be re-bootstrapped below.
  tytus daemon stop 2>/dev/null | tail -1 || true

  step "Restoring production tray + daemon"
  if [[ -f "$PROD_DAEMON_PLIST" ]]; then
    launchctl bootstrap "gui/$(id -u)" "$PROD_DAEMON_PLIST" 2>/dev/null \
      || launchctl kickstart -k "gui/$(id -u)/com.traylinx.tytus.daemon" 2>/dev/null \
      || true
    ok "production daemon LaunchAgent reloaded"
  fi
  if [[ -f "$PROD_TRAY_PLIST" ]]; then
    launchctl bootstrap "gui/$(id -u)" "$PROD_TRAY_PLIST" 2>/dev/null \
      || launchctl kickstart -k "gui/$(id -u)/com.traylinx.tytus.tray" 2>/dev/null \
      || true
    ok "production tray LaunchAgent reloaded"
  fi
  sleep 2
  if curl -fsS --max-time 1 "http://127.0.0.1:4242/api/state" >/dev/null 2>&1; then
    ok "production tray responding on :4242"
  else
    warn "production tray not yet responding — run \`tytus tray install\` if it doesn't come back"
  fi
}

cmd_status() {
  step "Dev mode status"
  if is_running "$DEV_VITE_PID_FILE"; then
    ok "vite running (pid $(cat "$DEV_VITE_PID_FILE")) → http://localhost:$DEV_VITE_PORT"
  else
    warn "vite not running"
  fi
  if is_running "$DEV_TRAY_PID_FILE"; then
    ok "dev tray running (pid $(cat "$DEV_TRAY_PID_FILE")) → http://localhost:$DEV_TRAY_PORT"
  else
    warn "dev tray not running"
  fi
  if is_running "$DEV_DAEMON_PID_FILE"; then
    ok "dev daemon running (pid $(cat "$DEV_DAEMON_PID_FILE")) → /tmp/tytus/daemon.sock"
  else
    warn "dev daemon not running — token will not refresh"
  fi
  if curl -fsS --max-time 1 "http://localhost:$DEV_VITE_PORT/api/state" -H 'Sec-Fetch-Site: same-origin' >/dev/null 2>&1; then
    ok "vite → tray proxy responding"
  else
    warn "vite → tray proxy NOT responding"
  fi
}

case "${1:-start}" in
  start) cmd_start ;;
  stop)  cmd_stop ;;
  status) cmd_status ;;
  *)
    echo "usage: $0 [start|stop|status]" >&2
    exit 2
    ;;
esac

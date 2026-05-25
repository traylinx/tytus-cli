#!/usr/bin/env bash
# dev-rebuild.sh — rebuild tytus + tytus-tray + tytus-mcp from local source
# and hot-swap the installed binaries WITHOUT going through the release
# pipeline. Use this when you've changed Rust code and want it live on
# your own machine.
#
# Frontend changes (TypeScript / React) do NOT need this script — run
# `npm run dev` in services/tytus-os/app/ on port 5173 for HMR.
#
# Sebastian's normal dogfood loop:
#   1. Make Rust changes in services/tytus-cli/
#   2. cd services/tytus-cli && scripts/dev-rebuild.sh
#   3. The tray respawns automatically with the new binary.
#
# Idempotent — safe to run repeatedly.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TYTUS_OS_APP_DIR="$REPO_ROOT/../tytus-os/app"
TYTUS_OS_DIST_DIR="$TYTUS_OS_APP_DIR/dist"
TYTUS_OS_VENDOR_DIR="$REPO_ROOT/tray/web/os"
TARGET_DIR="$REPO_ROOT/target/release"

INSTALL_DIR="/usr/local/bin"
APP_BUNDLE_MACOS="/Applications/Tytus.app/Contents/MacOS"

step() { printf "\n\033[1;36m▸ %s\033[0m\n" "$*"; }
ok()   { printf "  \033[1;32m✓\033[0m %s\n" "$*"; }
warn() { printf "  \033[1;33m!\033[0m %s\n" "$*"; }

# ── Optional frontend rebuild ────────────────────────────────────
# Pass --with-os to also rebuild the embedded TytusOS bundle.
# Skip this if you're using `npm run dev` for HMR on 5173 — the
# baked-in bundle isn't what your browser is seeing in dev mode.
REBUILD_OS=false
if [[ "${1:-}" == "--with-os" ]]; then
  REBUILD_OS=true
fi

if $REBUILD_OS; then
  step "Building tytus-os frontend ($TYTUS_OS_APP_DIR)"
  ( cd "$TYTUS_OS_APP_DIR" && npm run build )
  ok "vite build complete"

  step "Syncing dist → $TYTUS_OS_VENDOR_DIR (the bundle baked into the Rust binary)"
  rm -rf "$TYTUS_OS_VENDOR_DIR"
  mkdir -p "$TYTUS_OS_VENDOR_DIR"
  cp -R "$TYTUS_OS_DIST_DIR"/* "$TYTUS_OS_VENDOR_DIR/"
  ok "vendored bundle in place"
fi

# ── Rust build ─────────────────────────────────────────────────────
step "cargo build --release (tytus, tytus-tray, tytus-mcp)"
( cd "$REPO_ROOT" && cargo build --release --bin tytus --bin tytus-tray --bin tytus-mcp )
ok "release binaries built in $TARGET_DIR"

# ── Stop running tray + daemon so we can replace binaries ─────────
step "Stopping running tray + daemon"
tytus tray uninstall 2>/dev/null | tail -3 || warn "tytus tray uninstall returned non-zero (continuing)"
tytus daemon stop 2>/dev/null | tail -1 || true
# The installed tray runs as /Applications/Tytus.app/Contents/MacOS/Tytus
# (the bundle's executable is named "Tytus", not "tytus-tray"), so
# `pkill -f tytus-tray` doesn't match. Kill by app-bundle path AND by
# binary name to cover both the installed bundle and any dev-from-cargo
# invocation.
pkill -f '/Applications/Tytus.app/Contents/MacOS/Tytus' 2>/dev/null || true
pkill -f 'tytus-tray' 2>/dev/null || true
# Give launchd a moment to release the LaunchAgent slot.
sleep 1
ok "tray + daemon stopped"

# ── Install new binaries ──────────────────────────────────────────
step "Installing fresh binaries → $INSTALL_DIR"
# /usr/local/bin is typically root-writable. Use sudo non-interactively
# if available; otherwise fall back to cp and let the OS prompt.
if [[ -w "$INSTALL_DIR" ]]; then
  cp "$TARGET_DIR/tytus"      "$INSTALL_DIR/tytus"
  cp "$TARGET_DIR/tytus-tray" "$INSTALL_DIR/tytus-tray"
  cp "$TARGET_DIR/tytus-mcp"  "$INSTALL_DIR/tytus-mcp"
else
  sudo cp "$TARGET_DIR/tytus"      "$INSTALL_DIR/tytus"
  sudo cp "$TARGET_DIR/tytus-tray" "$INSTALL_DIR/tytus-tray"
  sudo cp "$TARGET_DIR/tytus-mcp"  "$INSTALL_DIR/tytus-mcp"
fi
ok "binaries copied"

# ── Reinstall tray (recreates /Applications/Tytus.app + LaunchAgent) ──
step "Reinstalling tray (tytus tray install)"
tytus tray install
ok "tray installed"

# ── Verify ────────────────────────────────────────────────────────
step "Verification"
INSTALLED_VERSION="$(tytus --version | awk '{print $2}')"
ok "installed version: $INSTALLED_VERSION"

# Wait for the LaunchAgent to spawn the daemon (KeepAlive normally
# fires within a couple seconds when RunAtLoad=true).
for _ in $(seq 1 10); do
  if curl -fsS --max-time 1 http://127.0.0.1:4242/api/state >/dev/null 2>&1; then
    ok "tray HTTP server responding on :4242"
    break
  fi
  sleep 0.5
done

cat <<'NEXT'

────────────────────────────────────────────────────────────────────
✓ Rust rebuild complete. The tray is now running your local code.

Frontend dev loop (separate terminal):
    cd services/tytus-os/app
    npm run dev
    # opens http://localhost:5173 with HMR
    # proxies /api/* to the tray on :4242

Re-run this script any time you change Rust code in tytus-cli/.
Pass --with-os to also refresh the bundled TytusOS dist (release flow).
────────────────────────────────────────────────────────────────────
NEXT

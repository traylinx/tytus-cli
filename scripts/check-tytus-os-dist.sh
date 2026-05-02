#!/usr/bin/env bash
set -euo pipefail

# Verifies the vendored TytusOS SPA bundle embedded by tytus-tray is byte-for-byte
# in sync with services/tytus-os/app/dist. Run after `npm run build` in tytus-os.

SERVICES_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OS_DIST="${SERVICES_ROOT}/tytus-os/app/dist"
TRAY_OS="${SERVICES_ROOT}/tytus-cli/tray/web/os"

if [ ! -d "${SERVICES_ROOT}/tytus-os" ]; then
  echo "TytusOS source tree not present — skipping monorepo dist freshness check"
  exit 0
fi

if [ ! -f "${OS_DIST}/index.html" ]; then
  echo "missing ${OS_DIST}/index.html — run npm run build in services/tytus-os/app" >&2
  exit 1
fi

if [ ! -f "${TRAY_OS}/index.html" ]; then
  echo "missing ${TRAY_OS}/index.html — copy TytusOS dist into tray/web/os" >&2
  exit 1
fi

tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

diff -qr "${OS_DIST}" "${TRAY_OS}" >"${tmp}/diff.txt" || {
  echo "vendored TytusOS bundle is stale. Refresh with:" >&2
  echo "  rm -rf services/tytus-cli/tray/web/os" >&2
  echo "  mkdir -p services/tytus-cli/tray/web/os" >&2
  echo "  cp -R services/tytus-os/app/dist/. services/tytus-cli/tray/web/os/" >&2
  echo >&2
  cat "${tmp}/diff.txt" >&2
  exit 1
}

echo "TytusOS vendored bundle OK — tray/web/os matches tytus-os/app/dist"

#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/sync-tytus-os-dist.sh [--check] [--sync] [--allow-missing-source]

Build system helper for deterministic TytusOS embedding.

Defaults:
  --sync   Copy the built TytusOS dist into tray/web/os and write tray/web/os.dist.sha256.

Source lookup:
  TYTUS_OS_SOURCE may point at either the tytus-os repo root or app dir.
  Without TYTUS_OS_SOURCE, the script looks for ../tytus-os, then ./tytus-os-source.
EOF
}

mode="sync"
allow_missing="0"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --check) mode="check" ;;
    --sync) mode="sync" ;;
    --allow-missing-source) allow_missing="1" ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown argument: $1" >&2; usage >&2; exit 2 ;;
  esac
  shift
done

CLI_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVICES_ROOT="$(cd "${CLI_ROOT}/.." && pwd)"

if [[ -n "${TYTUS_OS_SOURCE:-}" ]]; then
  OS_SOURCE="${TYTUS_OS_SOURCE}"
elif [[ -d "${SERVICES_ROOT}/tytus-os" ]]; then
  OS_SOURCE="${SERVICES_ROOT}/tytus-os"
elif [[ -d "${CLI_ROOT}/tytus-os-source" ]]; then
  OS_SOURCE="${CLI_ROOT}/tytus-os-source"
else
  OS_SOURCE=""
fi

if [[ -z "${OS_SOURCE}" || ! -d "${OS_SOURCE}" ]]; then
  if [[ "${allow_missing}" == "1" ]]; then
    echo "TytusOS source tree not present — skipping dist sync/check"
    exit 0
  fi
  echo "missing TytusOS source tree. Set TYTUS_OS_SOURCE or checkout ../tytus-os" >&2
  exit 1
fi

if [[ -d "${OS_SOURCE}/app" ]]; then
  OS_APP="${OS_SOURCE}/app"
else
  OS_APP="${OS_SOURCE}"
fi
OS_DIST="${OS_APP}/dist"
TRAY_OS="${CLI_ROOT}/tray/web/os"
SHA_FILE="${CLI_ROOT}/tray/web/os.dist.sha256"

if [[ ! -f "${OS_DIST}/index.html" ]]; then
  echo "missing ${OS_DIST}/index.html — run npm run build in ${OS_APP}" >&2
  exit 1
fi

hash_dir() {
  local dir="$1"
  (cd "${dir}" && find . -type f -print0 | LC_ALL=C sort -z | xargs -0 shasum -a 256 | shasum -a 256 | awk '{print $1}')
}

src_sha="$(hash_dir "${OS_DIST}")"

if [[ "${mode}" == "sync" ]]; then
  rm -rf "${TRAY_OS}"
  mkdir -p "${TRAY_OS}"
  cp -R "${OS_DIST}/." "${TRAY_OS}/"
  printf '%s  tray/web/os\n' "${src_sha}" > "${SHA_FILE}"
  echo "TytusOS dist synced into tray/web/os"
  echo "sha256-tree=${src_sha}"
  exit 0
fi

if [[ ! -f "${TRAY_OS}/index.html" ]]; then
  echo "missing ${TRAY_OS}/index.html — run scripts/sync-tytus-os-dist.sh --sync" >&2
  exit 1
fi
tray_sha="$(hash_dir "${TRAY_OS}")"
expected_sha=""
if [[ -f "${SHA_FILE}" ]]; then
  expected_sha="$(awk '{print $1}' "${SHA_FILE}" | head -1)"
fi

tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

diff -qr "${OS_DIST}" "${TRAY_OS}" >"${tmp}/diff.txt" || {
  echo "vendored TytusOS bundle is stale. Refresh with:" >&2
  echo "  TYTUS_OS_SOURCE=${OS_SOURCE} scripts/sync-tytus-os-dist.sh --sync" >&2
  echo >&2
  cat "${tmp}/diff.txt" >&2
  exit 1
}

if [[ -n "${expected_sha}" && "${expected_sha}" != "${tray_sha}" ]]; then
  echo "${SHA_FILE} is stale: expected ${tray_sha}, found ${expected_sha}" >&2
  exit 1
fi

echo "TytusOS vendored bundle OK — tray/web/os matches ${OS_DIST}"
echo "sha256-tree=${tray_sha}"

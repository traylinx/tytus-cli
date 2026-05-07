#!/usr/bin/env bash
set -euo pipefail

# Compatibility wrapper. Prefer scripts/sync-tytus-os-dist.sh directly.
exec "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/sync-tytus-os-dist.sh" --check --allow-missing-source "$@"

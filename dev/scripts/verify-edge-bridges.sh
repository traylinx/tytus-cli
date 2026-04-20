#!/usr/bin/env bash
# verify-edge-bridges.sh — Phase 2.5 E2E gate.
#
# For every active pod allocation in Scalesys, hit the public edge URL
# https://<slug>.tytus.traylinx.com/p/<N>/healthz with the owner's
# bearer (resolved against --resolve so this works before DNS lands).
# Expects 200 from each.
#
# Requires kubectl context pointing at k8s-blox/lon1 (Scalesys + edge LB).
# Pulls slugs and bearers directly from Scalesys SQLite via kubectl exec.
#
# Usage:
#   ./verify-edge-bridges.sh [--lb-ip 143.198.241.20]
#
# Exit code: 0 if all 200, non-zero on first failure.
set -euo pipefail

LB_IP="${1:-143.198.241.20}"
DOMAIN="tytus.traylinx.com"

echo "Verifying edge bridges via LB ${LB_IP}"

# Pull (slug, pod_id, bearer) tuples for every active allocation.
ROWS=$(kubectl -n production exec deploy/scalesys-controller -- python3 -c "
import sqlite3
c = sqlite3.connect('/data/fleet.db')
rows = c.execute('''
  SELECT s.slug, cl.pod_id, k.stable_key
  FROM clients cl
  JOIN user_slugs s ON s.client_id = cl.client_id
  JOIN user_stable_keys k ON k.client_id = cl.client_id
  WHERE cl.revoked_at IS NULL
  ORDER BY cl.droplet_id, cl.pod_id
''').fetchall()
for slug, pod_id, key in rows:
    print(f'{slug}|{pod_id}|{key}')
" 2>/dev/null)

if [ -z "${ROWS}" ]; then
    echo "No active allocations to verify."
    exit 0
fi

FAILS=0
TOTAL=0
while IFS='|' read -r slug pod_id bearer; do
    [ -z "${slug}" ] && continue
    TOTAL=$((TOTAL + 1))
    host="${slug}.${DOMAIN}"
    code=$(curl -sk --max-time 10 \
        --resolve "${host}:443:${LB_IP}" \
        -H "Authorization: Bearer ${bearer}" \
        -o /dev/null -w "%{http_code}" \
        "https://${host}/p/${pod_id}/healthz" || echo "000")
    if [ "${code}" = "200" ]; then
        printf "  OK    %s /p/%s/healthz (%s)\n" "${slug}" "${pod_id}" "${code}"
    else
        printf "  FAIL  %s /p/%s/healthz (%s)\n" "${slug}" "${pod_id}" "${code}"
        FAILS=$((FAILS + 1))
    fi
done <<EOF
${ROWS}
EOF

echo "${TOTAL} checked, ${FAILS} failed."
[ "${FAILS}" -eq 0 ]

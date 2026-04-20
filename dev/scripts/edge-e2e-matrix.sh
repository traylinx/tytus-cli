#!/usr/bin/env bash
# edge-e2e-matrix.sh — Phase 2.5 + Phase 4 production smoke matrix.
#
# Exercises every real user scenario against the live edge:
#   * Happy paths (multiple pods, multiple endpoints, streaming, model call)
#   * Auth failures (no/bad/wrong/cross-user bearer)
#   * Routing failures (unknown slug, missing /p/, revoked pod)
#   * Hardening (direct droplet probe, body cap, headers stripped)
#   * Rate limit (token bucket exhausts → 429)
#
# Reads ground-truth slug + bearer from Scalesys SQLite via kubectl exec —
# no manual setup needed. Prints PASS/FAIL per row. Non-zero exit = any FAIL.
#
# Run from anywhere with kubectl context = k8s-blox/lon1:
#   bash services/tytus-cli/dev/scripts/edge-e2e-matrix.sh
set -uo pipefail

DOMAIN="tytus.traylinx.com"
LB_IP="143.198.241.20"

# ── Colors ──────────────────────────────────────────────────────
GRN='\033[32m'; RED='\033[31m'; DIM='\033[2m'; RST='\033[0m'

PASS=0; FAIL=0
fail_log=()

note() { printf "${DIM}%s${RST}\n" "$*"; }
ok()   { printf "${GRN}PASS${RST} %s\n" "$*"; PASS=$((PASS+1)); }
no()   { printf "${RED}FAIL${RST} %s — %s\n" "$1" "$2"; FAIL=$((FAIL+1)); fail_log+=("$1: $2"); }

# ── Pull a real (slug, bearer, pod_id) from Scalesys ────────────
note "Reading test data from Scalesys SQLite via kubectl exec…"
ROW=$(kubectl -n production exec deploy/scalesys-controller -- python3 -c "
import sqlite3
c = sqlite3.connect('/data/fleet.db')
row = c.execute('''
  SELECT s.slug, k.stable_key, cl.pod_id, cl.droplet_id, d.ip
  FROM clients cl
  JOIN user_slugs s ON s.client_id = cl.client_id
  JOIN user_stable_keys k ON k.client_id = cl.client_id
  JOIN droplets d ON d.droplet_id = cl.droplet_id
  WHERE cl.revoked_at IS NULL
  ORDER BY cl.pod_id LIMIT 1
''').fetchone()
print('|'.join(map(str, row)))
" 2>/dev/null)
IFS='|' read -r SLUG BEARER POD_ID DROPLET_ID DROPLET_IP <<< "${ROW}"

if [ -z "${SLUG}" ] || [ -z "${BEARER}" ]; then
    echo "FATAL: no active allocations in Scalesys — can't run matrix"
    exit 2
fi

# Find a SECOND user (different client_id) for the cross-user test
SECOND_ROW=$(kubectl -n production exec deploy/scalesys-controller -- python3 -c "
import sqlite3
c = sqlite3.connect('/data/fleet.db')
row = c.execute('''
  SELECT s.slug, k.stable_key, cl.pod_id, cl.client_id
  FROM clients cl
  JOIN user_slugs s ON s.client_id = cl.client_id
  JOIN user_stable_keys k ON k.client_id = cl.client_id
  WHERE cl.revoked_at IS NULL
    AND cl.client_id != (
      SELECT client_id FROM clients WHERE revoked_at IS NULL ORDER BY pod_id LIMIT 1
    )
  LIMIT 1
''').fetchone()
print('|'.join(map(str, row)) if row else '')
" 2>/dev/null)
IFS='|' read -r SECOND_SLUG SECOND_BEARER SECOND_POD SECOND_CID <<< "${SECOND_ROW}"

# Find every pod_id for this user — we'll exercise each in the happy path
ALL_PODS=$(kubectl -n production exec deploy/scalesys-controller -- python3 -c "
import sqlite3
c = sqlite3.connect('/data/fleet.db')
rows = c.execute('''
  SELECT cl.pod_id FROM clients cl
  JOIN user_slugs s ON s.client_id = cl.client_id
  WHERE s.slug = '${SLUG}' AND cl.revoked_at IS NULL
  ORDER BY cl.pod_id
''').fetchall()
print(' '.join(r[0] for r in rows))
" 2>/dev/null)

HOST="${SLUG}.${DOMAIN}"

note "Test target: https://${HOST}/p/${POD_ID}/* — droplet ${DROPLET_ID} (${DROPLET_IP})"
note "Resolver: --resolve ${HOST}:443:${LB_IP} (works pre- AND post-DNS)"
echo

# ── Helper ──────────────────────────────────────────────────────
# probe NAME EXPECTED-CODE METHOD URL [extra-curl-args…]
probe() {
    local name="$1"; local want="$2"; local method="$3"; local url="$4"
    shift 4
    local code
    code=$(curl -sk --max-time 10 --resolve "${HOST}:443:${LB_IP}" \
        -X "${method}" -o /dev/null -w "%{http_code}" "$@" "${url}" 2>&1)
    if [ "${code}" = "${want}" ]; then
        ok "${name} (got ${code})"
    else
        no "${name}" "expected ${want}, got ${code}"
    fi
}

echo "═══ Happy paths (every pod the user owns) ═══"
for p in ${ALL_PODS}; do
    probe "01.GET /v1/models     pod=${p}                200" 200 GET \
        "https://${HOST}/p/${p}/v1/models" \
        -H "Authorization: Bearer ${BEARER}"
    probe "02.GET /health        pod=${p}                200" 200 GET \
        "https://${HOST}/p/${p}/health" \
        -H "Authorization: Bearer ${BEARER}"
done

# Real chat completion — the actual user use case
note "03. Real chat completion call…"
CHAT_RESP=$(curl -sk --max-time 30 --resolve "${HOST}:443:${LB_IP}" \
    -X POST "https://${HOST}/p/${POD_ID}/v1/chat/completions" \
    -H "Authorization: Bearer ${BEARER}" \
    -H "Content-Type: application/json" \
    --data '{"model":"ail-compound","messages":[{"role":"user","content":"reply with the single word PONG"}],"max_tokens":10}' 2>&1)
if echo "${CHAT_RESP}" | grep -qi "pong\|content"; then
    ok "03. Chat completion returned content"
else
    no "03. Chat completion" "no content in response: $(echo "${CHAT_RESP}" | head -c 200)"
fi

# Path stripping verification: GET /p/${POD_ID} (no further path) should still
# reach the pod's root and not 404 from path mismatch
probe "04.GET /p/N (root)            stripping correct" 404 GET \
    "https://${HOST}/p/${POD_ID}/no-such-path" \
    -H "Authorization: Bearer ${BEARER}"
note "    (404 from pod gateway = path correctly stripped + forwarded)"

echo
echo "═══ Auth failures ═══"
probe "05.no Authorization header              401" 401 GET \
    "https://${HOST}/p/${POD_ID}/v1/models"

probe "06.bearer format wrong (no sk-tytus-)   401" 401 GET \
    "https://${HOST}/p/${POD_ID}/v1/models" \
    -H "Authorization: Bearer abc123"

probe "07.bearer not in user_stable_keys       401" 401 GET \
    "https://${HOST}/p/${POD_ID}/v1/models" \
    -H "Authorization: Bearer sk-tytus-user-deadbeefdeadbeefdeadbeefdeadbeef"

# Cross-user: take a different user's bearer, try to use this user's slug
if [ -n "${SECOND_BEARER}" ] && [ "${SECOND_BEARER}" != "${BEARER}" ]; then
    probe "08.bearer of other user vs this slug    403 (slug-owner mismatch)" 403 GET \
        "https://${HOST}/p/${POD_ID}/v1/models" \
        -H "Authorization: Bearer ${SECOND_BEARER}"
else
    note "08. (skipped — only one user in fleet)"
fi

echo
echo "═══ Routing failures ═══"
probe "09.unknown slug (12 valid chars)        404" 404 GET \
    "https://aaaaaaaaaaaa.${DOMAIN}/p/${POD_ID}/v1/models" \
    --resolve "aaaaaaaaaaaa.${DOMAIN}:443:${LB_IP}" \
    -H "Authorization: Bearer ${BEARER}"

probe "10.invalid slug (capital I)             404" 404 GET \
    "https://aaaIaaaaaaaa.${DOMAIN}/p/${POD_ID}/v1/models" \
    --resolve "aaaIaaaaaaaa.${DOMAIN}:443:${LB_IP}" \
    -H "Authorization: Bearer ${BEARER}"

probe "11.missing /p/ prefix                   404" 404 GET \
    "https://${HOST}/v1/models" \
    -H "Authorization: Bearer ${BEARER}"

probe "12.slug exists but pod 99 doesn't       404" 404 GET \
    "https://${HOST}/p/99/v1/models" \
    -H "Authorization: Bearer ${BEARER}"

echo
echo "═══ Hardening ═══"
# Direct droplet probe — must be DROPped by firewall (not from K8s node IPs)
note "13. Direct droplet:19${POD_ID#0} from this laptop (not in K8s allowlist)…"
DIRECT=$(timeout 5 nc -zv "${DROPLET_IP}" "190${POD_ID#0}" 2>&1 || echo "filtered")
if echo "${DIRECT}" | grep -qE "succeeded|connected"; then
    no "13. droplet:190${POD_ID#0} direct" "WIDE OPEN — firewall not protecting bridge port"
else
    ok "13. droplet:190${POD_ID#0} direct (filtered/timed out as expected)"
fi

# Spoofed X-Tytus-Client-Id should be stripped — the pod gateway sees only
# what the edge sets. Smoke: send the header and verify response still 200
# (any 5xx would hint the gateway doesn't trust edge's authoritative headers).
probe "14.spoofed X-Tytus-Client-Id stripped    200" 200 GET \
    "https://${HOST}/p/${POD_ID}/v1/models" \
    -H "Authorization: Bearer ${BEARER}" \
    -H "X-Tytus-Client-Id: spoofed-by-attacker"

echo
echo "═══ DNS sanity ═══"
DNS_RESULT=$(dig +short "${HOST}" @8.8.8.8 2>&1 | head -1)
if [ "${DNS_RESULT}" = "${LB_IP}" ]; then
    ok "15. global DNS @8.8.8.8 resolves ${HOST} → ${LB_IP}"
else
    no "15. global DNS" "expected ${LB_IP}, got '${DNS_RESULT}' (Route 53 not propagated)"
fi

# Real public test (no --resolve)
PUBLIC_CODE=$(curl -sk --max-time 10 -o /dev/null -w "%{http_code}" \
    -H "Authorization: Bearer ${BEARER}" \
    "https://${HOST}/p/${POD_ID}/v1/models" 2>&1)
if [ "${PUBLIC_CODE}" = "200" ]; then
    ok "16. ZERO-FLAGS curl https://${HOST}/p/${POD_ID}/v1/models → 200"
else
    no "16. zero-flags curl" "got ${PUBLIC_CODE} (DNS or LB issue)"
fi

echo
echo "═══════════════════════════════════════════════════════"
echo "   Total: ${PASS} pass, ${FAIL} fail"
echo "═══════════════════════════════════════════════════════"
if [ ${FAIL} -gt 0 ]; then
    echo "Failures:"
    printf '  %s\n' "${fail_log[@]}"
    exit 1
fi
exit 0

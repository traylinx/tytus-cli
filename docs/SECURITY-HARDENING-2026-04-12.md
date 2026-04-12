# Security Hardening Audit — 2026-04-12

**Status:** CLI fixes applied. Infrastructure fixes flagged for droplet team.

---

## Audit Summary

Full reverse-engineering of tytus-cli security surface: network reachability
through the WireGuard tunnel, CLI information leakage, API endpoint exposure.

### What's Good (verified)

| Check | Result |
|---|---|
| Cross-pod isolation | PASS — pods 1,3,4,5,6,7,8 all unreachable |
| Metadata API (169.254.169.254) | PASS — blocked |
| K8s API (6443) | PASS — not reachable through tunnel |
| DAM (8099) | PASS — not reachable |
| SSH through tunnel | PASS — port 22 closed on pod subnet |
| Tunnel route scoping | PASS — only 10.18.2.0/24 + 10.42.42.1/32 |
| WG private key on disk | PASS — never written, in-memory only |
| State file permissions | PASS — 0600 |
| Token in keychain | PASS — OS keychain, not plain file |

### What Was Fixed (CLI-side, this commit)

| Issue | Severity | Fix |
|---|---|---|
| `tytus status --json` exposed droplet_id, droplet_ip, internal IPs, raw per-pod keys | MEDIUM | Redacted: only pod_id, agent_type, stable_ai_endpoint, stable_user_key, tunnel_iface exposed |
| `tytus connect` printed AI_GATEWAY (internal IP), AGENT_API, API_KEY | MEDIUM | Now prints only ENDPOINT (stable) |
| Human status showed internal IPs and partial raw keys | MEDIUM | Shows only stable endpoint + masked stable key |

### What Needs Infrastructure Fixes (DROPLET TEAM)

| Issue | Severity | Fix | Owner |
|---|---|---|---|
| **Droplet SSH open on public internet** | CRITICAL | `ufw deny 22/tcp` from 0.0.0.0/0. SSH only via WireGuard or jump host. | Infra |
| **`/metrics` returns Go runtime stats with NO auth** | MEDIUM | nginx: `location /metrics { return 404; }` or restrict to 127.0.0.1 | Infra |
| **`/` returns server identity + endpoint listing, no auth** | LOW | nginx: return 404 on / or remove endpoint listing | Infra |
| **`/health` returns status with no auth** | LOW | Acceptable for load balancer probes, but consider auth | Infra |

### Detailed Network Scan Results

**Ports open on own pod (10.18.2.1):**
- 3000 (agent — NemoClaw) — expected, needed for `tytus ui`
- 18080 (SwitchAILocal gateway) — expected

**Ports open on stable endpoint (10.42.42.1):**
- 18080 only — expected

**HTTP paths on gateway (10.42.42.1:18080):**
- `/` → 200, server identity (no auth) — LOW risk
- `/health` → 200, `{"status":"ok"}` (no auth) — LOW risk
- `/metrics` → 200, Go runtime stats (no auth) — **MEDIUM risk: fingerprinting**
- `/v1/models` → 200 (auth required) — correct
- `/v1/chat/completions` → auth required — correct
- All other paths → 404 — correct

**Cross-pod isolation:**
- Pods 1,3,4,5,6,7,8 all unreachable — PASS

**Droplet public IP (redacted for public repo):**
- Port 22 (SSH) → **OPEN from public internet** — CRITICAL
- This IP was previously exposed in `tytus status --json` output

---

## Recommendations for Launch

### Must-fix before launch (CRITICAL)
1. Close SSH on droplet public IP (use WireGuard-only SSH or jump host)
2. ~~Strip infrastructure data from CLI output~~ — DONE

### Should-fix before launch (MEDIUM)
3. Block `/metrics` endpoint on nginx (or require auth)
4. Rate-limit the gateway's auth failure responses (prevent key brute-force)

### Nice-to-have (LOW)
5. Suppress server identity on `/` endpoint
6. Add `X-Content-Type-Options: nosniff` and security headers to gateway responses

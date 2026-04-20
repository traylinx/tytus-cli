Verified each claim. Writing revised doc — pushing back on two (Caddy built-in metrics; 60-bit slug is not "low-entropy").

# SPRINT-PUBLIC-POD-EDGE

## Origin: user request

Public HTTPS access to Tytus pods via per-user random-slug subdomain (`{slug}.tytus.traylinx.com`) fronted by a central edge router; pod selected via path prefix (`/p/{pod_id}/...`). Single wildcard cert, droplets stay private, WireGuard becomes opt-in via `tytus connect --tunnel`. Must scale to ~20K pods without per-pod cert management.

**Threat model.** The slug is a user-visible, non-secret identifier (60 bits of entropy from 12 Crockford base32 chars — plenty to prevent enumeration, but it leaks in URLs / browser history / logs and is therefore treated as public). The bearer token `sk-tytus-user-*` issued by Scalesys (`user_stable_keys.stable_key`, `app.py:595`) is the **sole** authenticator. The slug→owner check at the edge is defense-in-depth only (prevents slug-squatting and surfaces misconfiguration) and is not load-bearing for confidentiality.

## Phases

### Phase 1: Scalesys slug schema, resolve API, perf go/no-go gate

**Goal:** Persist one slug per `client_id` (the only user identifier in this DB — `app.py:121,146`), expose a fast internal lookup the edge will hit on every request, and **measure resolve latency before committing Phase 2**.

**Criteria:**
- New table `user_slugs(client_id TEXT PRIMARY KEY, slug TEXT UNIQUE NOT NULL, created_at REAL)` added inside `init_db()` using the existing `CREATE TABLE IF NOT EXISTS` + try/except `ALTER` idempotent pattern (mirrors `app.py:99–151`). `CREATE UNIQUE INDEX IF NOT EXISTS idx_user_slugs_slug ON user_slugs(slug)`. No `migrations/` folder, no runner, no new dep.
- `ensure_slug(conn, client_id)` helper placed next to `ensure_stable_key` (`app.py:595`) and called from the same three sites (`app.py:387, 458, 515`) so every active user gets a slug on next pod op. One-shot boot backfill: `SELECT DISTINCT client_id FROM clients` + `SELECT client_id FROM user_stable_keys`.
- Slug = 12-char Crockford base32 (no I/L/O/U, upper-case folded to lower on output), source `secrets.token_bytes(8)`, retry on UNIQUE conflict (≤3 attempts then 500).
- `GET /internal/resolve/<slug>/<pod_id>` → `{client_id, droplet_ip, pod_id, status: "active"|"revoked"|"unknown"}`. Requires `X-Scalesys-Token` header validated by the existing `@require_auth` decorator (`app.py:163–170`, `secrets.compare_digest`). 404 unknown slug, 410 revoked pod (edge negative-caches), 401 bad token.
- `GET /internal/client/<client_id>/slug` returns `{slug}` for Provider. Same `@require_auth`.
- **Extend existing `/pod/user-key` response instead of building a parallel endpoint.** `app.py`'s `/api/user-keys/<client_id>` (`app.py:1043`) includes `slug` in its payload; `services/wannolot-provider/src/routes/pod.js:320–340` passes it through in the `/pod/user-key` response. CLI already calls `atomek_pods::get_user_key` at `main.rs:2941–2943` — we just add a `slug` field to `UserKeyResponse` in `pods/src/user_key.rs`. No new route, no new CLI client call.
- Prometheus histogram `scalesys_resolve_duration_seconds` (labels: `status`) on the resolve endpoint using the Flask `prometheus_client` already pulled in elsewhere in this repo (or add `prometheus_client==0.20.*` to `requirements.txt` — one-line change).
- **Go/no-go gate at end of Phase 1 (before Phase 2 work starts):** `scripts/loadtest_resolve.py` runs 1k / 5k / 10k rps against staging Scalesys with seeded dataset (20K slugs, 50K pods). Back-of-envelope: indexed SQLite SELECT on 20K rows with WAL mode is ≈0.1–0.3 ms; Flask+gunicorn single-worker ceiling is ~2–3k rps. **Decision written to `dev/audits/2026-04-2X-scalesys-resolve-perf.md`:** if p99 < 10 ms at 5 k rps → proceed with in-process edge LRU only. Else → add a `caddy-storage-redis`-backed L2 cache to the plugin design in Phase 2 *before* writing it.

**Files:**
- `services/wannolot-infrastructure/scalesys/controller/app.py` (extend `init_db`, add `ensure_slug`, add two `/internal/*` routes, include slug in existing `/api/user-keys/<client_id>` response, add histogram)
- `services/wannolot-infrastructure/scalesys/controller/requirements.txt` (possibly add `prometheus_client`)
- `services/wannolot-infrastructure/scalesys/controller/scripts/loadtest_resolve.py` (new)
- `services/wannolot-provider/src/routes/pod.js` (extend `/pod/user-key` passthrough to include `slug` — the existing file already proxies Scalesys; no new route file, no `src/clients/`)
- `services/tytus-cli/pods/src/user_key.rs` (add `slug: Option<String>` to `UserKeyResponse`, return `(endpoint, key, slug)`)

**Tests:**
- `init_db()` idempotent on populated DB (existing pattern, new coverage for `user_slugs`).
- `ensure_slug` collision: monkey-patch `secrets.token_bytes` to a constant twice → second call hits retry; 4th hit returns 500.
- Resolve matrix: unknown slug → 404, revoked pod → 410, wrong `X-Scalesys-Token` → 401, happy path → correct `droplet_ip` joined from `droplets`.
- Concurrency: 20 threads call `ensure_slug` for one client → exactly one row, same slug returned.
- Backfill pass creates a slug for every distinct `client_id` in `clients` ∪ `user_stable_keys`; re-run is a no-op.
- Load-test artifact committed to `dev/audits/`.

---

### Phase 2: Edge router (Caddy + tiny resolver plugin)

**Goal:** Stateless HTTPS edge terminating `*.tytus.traylinx.com`, mapping `Host`+path prefix to `{droplet_ip}:18080`.

**Criteria:**
- Caddy 2.8+ compiled with three external modules: `caddy-dns/digitalocean` (DNS-01), `mholt/caddy-ratelimit` (token-bucket — verified active community plugin, pin exact tag in builder stage), `caddy-storage-redis` (only used if the Phase 1 gate flips it on).
- Custom Caddy module `tytus_resolver` (Go, ≤200 LOC) implements `caddyhttp.MiddlewareHandler`:
  - Parses slug from `Host` (everything before first `.`), `pod_id` from regex `^/p/(?P<pod>[0-9]{2})(/.*)?$`.
  - Calls Scalesys `/internal/resolve/<slug>/<pod>` with `X-Scalesys-Token: {env.SCALESYS_SECRET}` (not `X-Scalesys-Secret`). Positive cache TTL **2 s**, negative TTL 5 s, in-process LRU sized to current pod count × 1.2.
  - Rewrites the request: strip `/p/{pod}` prefix so `/p/01/v1/chat/completions` → `/v1/chat/completions` (switchAILocal expects `/v1/...`).
  - Validates `Authorization: Bearer sk-tytus-user-<hex>` against Scalesys via `POST /internal/auth` returning `client_id` (this endpoint added in Phase 1 if not already present — SELECT on `user_stable_keys.stable_key`). Reject when `auth.client_id != resolve.client_id` (slug-owner defense-in-depth). Positive auth cache 30 s keyed on `sha256(key)`.
  - Sets placeholders `{tytus.client_id}` and `{tytus.droplet_ip}` for downstream directives.
  - Strips inbound `X-Tytus-*` and `X-Forwarded-*` headers before adding our own.
- **Caddyfile explicit directive order (global block):** `{ order tytus_resolver before rate_limit; order rate_limit before reverse_proxy }` so `{tytus.client_id}` is populated before `rate_limit` reads it. Without this, Caddy's default ordering will run `rate_limit` first and the placeholder will be empty.
- `reverse_proxy {tytus.droplet_ip}:18080` with active health check on `/health`, 65 s idle, 10 MB body limit, 60 rps / burst 120 keyed on `{tytus.client_id}`.
- **Rate-limit semantics documented explicitly:** `caddy-ratelimit` is **per-instance**. With N Caddy replicas, effective user-visible ceiling is N × configured rate. v1 ships per-instance with the configured value set to 1/N of the abuse threshold (with N = current replica count). Distributed rate-limiting via a shared backend is explicit non-goal for v1; re-evaluate if abuse is observed (tracked in runbook).
- **Metrics:** Caddy's built-in Prometheus exporter **is** the source — enabled by the global `servers { metrics }` directive (available since Caddy v2.4, see https://caddyserver.com/docs/metrics). It exposes `/metrics` on the admin endpoint in Prometheus format. No sidecar, no caddy-exporter fork. Custom plugin counters (`tytus_resolver_cache_hits_total`, `tytus_resolver_auth_failures_total`, `tytus_resolver_slug_owner_mismatches_total`) are registered with the same `prometheus.DefaultRegisterer` and surface via the same `/metrics` endpoint. *Pushing back on validator claim — Caddy's built-in metrics exporter is Prometheus-format and sufficient; verified in docs.*
- `/healthz` (always 200), `/readyz` (probes Scalesys `/health`).

**Files:**
- `services/wannolot-edge/Caddyfile` (new)
- `services/wannolot-edge/plugin/tytus_resolver.go` (new)
- `services/wannolot-edge/plugin/cache.go` (new, LRU)
- `services/wannolot-edge/Dockerfile` (new — `caddy:builder` stage, pinned plugin tags)
- `services/wannolot-edge/README.md` (new — runbook incl. rate-limit semantics)

**Tests:**
- Plugin unit: slug+pod extraction, prefix strip, header sanitization, slug-owner mismatch returns 403.
- Integration with stubbed Scalesys (`httptest`): cache hit avoids second call, 410 negative-cached for 5 s, `X-Scalesys-Token` sent verbatim.
- Integration with real switchAILocal in `docker-compose`: `/v1/models`, `/v1/chat/completions`, `/v1/embeddings` all succeed after `/p/{pod}` strip.
- `caddy validate` passes in CI; directive order verified by inspecting `caddy adapt` output.
- Adversarial: inbound `X-Tytus-Client-Id: someone-else` is stripped; `X-Forwarded-For` spoof is stripped.
- Rate-limit verified per-instance with a 2-replica compose; documented effective ceiling.

---

### Phase 3: Wildcard cert (cert-manager), egress via NAT droplet, droplet firewall

**Goal:** Live edge with a wildcard cert that works across HPA replicas, a stable egress IP we control, and a droplet firewall that opens `:18080` only to that IP. Explicitly document the net-new attack surface.

**Criteria:**
- **Cert storage solved via cert-manager, not Caddy-managed storage.** The original plan (Caddy-managed ACME + RWO PVC) does not work with 3–10 HPA replicas — the PVC can only mount to one pod and Caddy's filesystem cert storage is not cluster-safe. Fix:
  - `cert-manager` with `Issuer` using DigitalOcean DNS-01 issues `*.tytus.traylinx.com` into a K8s `Secret` (`tytus-wildcard-tls`).
  - All Caddy pods mount the Secret read-only at `/etc/caddy/tls/`.
  - Caddy's `tls /etc/caddy/tls/tls.crt /etc/caddy/tls/tls.key` directive loads the cert; `reloader.stakater.com/auto: "true"` on the Deployment triggers rollout when cert-manager rotates the Secret.
  - Caddy's ACME code path is fully disabled; the edge is a pure consumer of the Secret.
- DNS: `*.tytus.traylinx.com` CNAME to the K8s `Service` external hostname.
- `Deployment replicas: 3`, PDB `minAvailable: 2`, HPA 3–10 on CPU 70 %, readiness `/readyz`.
- **Egress via dedicated NAT gateway droplet, not "Reserved IP attached to the worker pool" (which is not a real DO feature).** DO Reserved IPs attach to single droplets, not K8s node pools. Fix:
  - Provision one small NAT droplet `edge-nat-01` with a DO **Reserved IP**. MASQUERADE `iptables` rule on that droplet.
  - Cilium egress gateway (already installable as a Helm chart on DOKS) pins egress for pods with label `role=edge` through `edge-nat-01`. Alternative if Cilium is not in use: run Caddy on a dedicated node pool tainted + tolerated, and set the node's default route via the NAT droplet using a CNI-agnostic static route + per-pod routing policy. Concrete choice recorded in `deploy/edge/EGRESS.md`.
  - The Reserved IP of `edge-nat-01` is the one value the droplet firewall allowlists.
- **Droplet firewall change — net-new attack surface, documented.** Current `cloud-init/user-data.yml:105–107` allows only SSH (22) and WireGuard UDP (`${WIREGUARD_BASE_PORT}..+TYTUS_MAX-1`). Before this sprint, port 18080 is reachable only via the in-droplet WireGuard sidecars — zero public exposure. Adding `ufw allow from {edge_nat_reserved_ip} to any port 18080 proto tcp comment 'edge-ingress'` opens port 18080 to one specific external IP. Threat analysis written in `docs/SECURITY-AUDIT.md` addendum:
  - Attacker who compromises `edge-nat-01` gains network-level reach to `:18080` on every droplet (but still needs a valid `sk-tytus-user-*` token — switchAILocal requires bearer auth).
  - Attacker who compromises the edge pods bypasses resolver/owner checks but is still bounded by the switchAILocal authenticator.
  - The WireGuard + netns egress firewall remains unchanged (pod egress allowlist: switchAILocal + DNS; `iptables FORWARD` rules preserved per `user-data.yml:11,318`).
  - Mitigations: `edge-nat-01` runs nothing except MASQUERADE; SSH closed except from bastion; reserved IP rotated if compromise suspected.
- Rollback: tunnel path remains live; flipping CLI flag is a config change (Phase 5).

**Files:**
- `deploy/edge/deployment.yaml` (new — Caddy with Secret mount + reloader annotation)
- `deploy/edge/service.yaml` (new — `LoadBalancer`)
- `deploy/edge/hpa.yaml` (new)
- `deploy/edge/certificate.yaml` (new — cert-manager `Certificate` for `*.tytus.traylinx.com`)
- `deploy/edge/issuer.yaml` (new — DO DNS-01 `Issuer`, references existing `DO_API_TOKEN` Secret)
- `deploy/edge/ciliumegressgatewaypolicy.yaml` (new — or node-pool static route, per `EGRESS.md` decision)
- `deploy/edge/EGRESS.md` (new — records the concrete egress mechanism)
- `services/wannolot-infrastructure/cloud-init/user-data.yml` (edit — add one `ufw allow from <RESERVED_IP> to any port 18080 proto tcp`)
- `docs/SECURITY-AUDIT.md` (edit — addendum documenting the new ingress surface)
- `.github/workflows/edge-deploy.yml` (new)

**Tests:**
- Staging: `curl -v https://<slug>.tytus.traylinx.com/p/01/v1/models` from public internet returns models JSON; cert chain validates with public roots.
- From a non-edge host: `curl http://<droplet_ip>:18080/v1/models` is rejected (timeout).
- From `edge-nat-01` itself: same curl succeeds — confirms the allowlist is narrow.
- Kill one edge pod under sustained 1 k rps → zero dropped requests (PDB + rolling update + Secret mount).
- Cert rotation drill: force cert-manager renewal → reloader annotation triggers Caddy rolling restart → no user-visible downtime.

---

### Phase 4: CLI dual-path (`--tunnel` opt-in, public default)

**Goal:** `tytus connect` defaults to public edge (no sudo, no `utun`); `tytus connect --tunnel` keeps existing boringtun behavior.

**Criteria:**
- Edits live in the monolithic `cli/src/main.rs` (7141 lines) — there is no `src/commands/*.rs` module tree.
- `cmd_connect` (`main.rs:749`) gains `--tunnel` clap flag. Default branch: allocate pod via Provider, call `get_user_key` (`pods/src/user_key.rs:20`) which now returns `(endpoint, key, slug)`, persist `mode = "edge"` on the `PodState` record, print `https://{slug}.tytus.traylinx.com/p/{pod_id}`. No tunnel daemon spawn, no boringtun.
- `--tunnel` branch: existing flow unchanged (guarded behind the flag).
- `cmd_env` (`main.rs:2920`): when `mode == "edge"` emit `OPENAI_BASE_URL=https://{slug}.tytus.traylinx.com/p/{pod_id}/v1`; when `mode == "tunnel"` emit `http://10.42.42.1:18080/v1` (current behavior). `OPENAI_API_KEY` is unchanged. The existing fallback at `main.rs:2941–2950` that calls `get_user_key` on missing cache is preserved and now also populates `slug`.
- `cmd_status` (`main.rs:723`): add mode column.
- `cmd_test` (`main.rs:3493`): dispatch probe target by mode.
- `cmd_doctor` (`main.rs:6082`): for edge pods, probe DNS of `{slug}.tytus.traylinx.com`, TLS handshake, `/readyz` 200.
- `cli/src/state.rs`: add `mode: String` to the per-pod record, `#[serde(default = "mode_default")]` returning `"tunnel"` so existing state files decode unchanged.
- Provider: no new route. The existing `router.get('/user-key', …)` at `src/routes/pod.js:324` is extended to include `slug` from the Scalesys response (which already carries it after Phase 1). No new client file — `src/services/scalesys-client.js` is untouched (it does not call `/api/user-keys/*`; `pod.js` calls it directly via `axios`, `pod.js:326–331`).

**Files:**
- `services/tytus-cli/cli/src/main.rs` (edit at the line references above, add clap `--tunnel`)
- `services/tytus-cli/cli/src/state.rs` (add `mode` with `serde(default)`)
- `services/tytus-cli/pods/src/user_key.rs` (adjusted in Phase 1; mentioned here for clarity)
- `services/wannolot-provider/src/routes/pod.js` (extend `/user-key` response to include `slug` — already scheduled in Phase 1)

**Tests:**
- Rust unit: `cmd_env` emits correct base URL for each mode; state file without `mode` round-trips as `"tunnel"`.
- Integration macOS: `tytus connect` (no flag) creates **no** `utun*` interface (`ifconfig -l` diff before/after).
- Backward compat: load an existing Phase-1-era state file → `status` shows mode `tunnel`, behavior identical.
- E2E against staging edge: `tytus connect && tytus test` succeeds without sudo.
- Provider `/pod/user-key`: 401 without A2A; includes `slug` field; returns same slug across calls.

---

### Phase 5: Flag-gated cutover, observability, kill-switch

**Goal:** Roll out behind a flag, monitor, keep rollback fast. Perf gate moved to Phase 1; this phase is cutover only.

**Criteria:**
- Provider feature flag `EDGE_PATH_ENABLED` (env var). When `false`: `/pod/user-key` still returns `stable_ai_endpoint` + `stable_user_key` but strips `slug` from the response so the CLI falls through to tunnel mode.
- Grafana dashboard: edge RPS/latency/errors by status, resolver cache hit ratio, rate-limit trips by `client_id`, Scalesys resolve latency (histogram from Phase 1), cert days-remaining (from cert-manager metrics, not Caddy — cert-manager owns the cert now).
- Alerts: edge 5xx > 1 % / 5 min, resolve p99 > 20 ms / 5 min, cert < 14 days.
- Runbook `docs/runbooks/edge-router.md`: rotate cert (cert-manager `cmctl renew`), force-purge a slug, flip kill-switch, debug a stuck slug-owner mismatch, investigate rate-limit per-instance skew.
- Canary: 10 internal users for 48 h, then 100 % (still gated by flag for instant rollback).
- Chaos test in staging: kill Scalesys for 30 s during load → edge serves cached resolves until TTL expires, 503s, recovers cleanly.

**Files:**
- `services/wannolot-provider/src/routes/pod.js` (flag guard on `slug` field)
- `services/wannolot-provider/src/config.js` (read `EDGE_PATH_ENABLED`)
- `docs/runbooks/edge-router.md` (new)
- `deploy/edge/dashboards/edge-router.json` (new)
- `deploy/edge/alerts.yaml` (new)

**Tests:**
- Kill-switch: flip `EDGE_PATH_ENABLED=false` → new `tytus connect` falls back to tunnel messaging within one invocation; existing edge sessions drain naturally (no forced disconnect).
- Each alert threshold synthetically tripped end-to-end in staging.
- Cert rotation drill (already in Phase 3) re-verified under live traffic with zero error-rate spike on the dashboard.

# SPRINT-PUBLIC-POD-EDGE-PHASE-2.5

## Origin: user request

Phase 2 deploy (2026-04-20) proved the edge (`wannolot-edge`, LB `143.198.241.20`) resolves + authenticates but cannot dial any pod: pod gateways bind `10.X.X.1:18080` inside the WireGuard sidecar netns (`wannolot-XX`, `network_mode: none`), never on the droplet's public IP. Phases 1 + 2 (Scalesys `/internal/resolve` + auth, Caddy + `tytus_resolver`, LE wildcard) are green. Phase 2.5 ships the droplet-side public bridge so the edge path closes.

**Architecture** (locked; not up for rediscussion): per-pod public port `19000+pod_id` on each droplet, source-IP whitelisted to the `k8s-blox/lon1` worker node public IPs. Forwarder = `socat` managed by DAM via a per-pod systemd unit (`tytus-bridge@<N>.service`) that `nsenter`s into the sidecar PID — same pattern `wannolot-network.service` uses for the existing pod → switchAILocal hop (verified: `bootstrap/05-provision-switchailocal.sh:300-301` shows `nsenter -t $PID -n socat TCP4-LISTEN:18080,fork,reuseaddr EXEC:"nsenter -t 1 -n socat STDIO TCP4\:127.0.0.1\:18090"`). Rationale: (a) no second auth layer — edge already enforces bearer + slug ownership; (b) bridge stays a dumb TCP forwarder; (c) `expose`/`unexpose` are siblings of DAM's existing `/agent/<N>/deploy|stop` lifecycle.

**Port range.** Bridges land at **19000–19099**, not 18000–18099. `docker-compose.switchailocal.j2:35` runs switchAILocal with `network_mode: host` on `18080 + inst` (18080–18099 at TYTUS_MAX=80). Overlap would clash at bind time. 19000–19099 is free; a pre-merge `ss -lntp` snapshot on `strato-eu-001` is captured in the audit doc to prove it stays free (no Docker daemon, DO agent, fail2ban, or sshd listener in that range).

**Threat model.** Bridge ports are TCP-reachable at L3 from the public internet until the firewall drops. Only `INPUT -s <k8s_node_ip>/32 -j ACCEPT` above a `-p tcp --dport 19000:19099 -j DROP` baseline keeps them private. DAM installs the DROP baseline first, verifies it second, opens ACCEPTs third; fail-closed on any step. Any IP-filter bypass lands on a forwarder that does no auth, so the firewall is load-bearing and every apply is read-back verified.

**Two firewall layers.** Production droplets on DigitalOcean may be attached to a DO cloud firewall that drops *before* packets reach the host. Phase 2 includes an audit of `doctl compute firewall list --format Name,DropletIDs` against current fleet; cloud firewall rules for `tcp:19000-19099` with source = K8s node IPs are added if one is attached, or the absence is documented. Host `iptables` (via `iptables-nft` on Ubuntu 22.04) remains the authoritative layer and ships regardless.

**K8s allowlist source.** `k8s-blox/lon1` has 3 workers, no autoscaling. One-shot discovery: `kubectl get nodes -o wide -l doks.digitalocean.com/node-pool=<pool> -o jsonpath='{.items[*].status.addresses[?(@.type=="ExternalIP")].address}'`. Commit the actual three IPs to `services/wannolot-infrastructure/k8s-node-allowlist.txt` before merge. `143.198.241.20` is the LB VIP, not a node IP, and MUST NOT be in this list. GitOps renders it to `/etc/wannolot/k8s-allowlist.txt` on every tick. Dynamic reconciliation (DO API poll) is a follow-up ticket.

## Phases

### Phase 1: Scalesys resolve + active-pods contract

**Goal:** Scalesys returns `pod_port` alongside `droplet_ip` in resolve, and exposes a droplet-scoped active-pods list so DAM can reconcile bridges on boot. Additive only; back-compat preserved for the already-deployed edge build.

**Criteria:**
- New `services/wannolot-infrastructure/scalesys/controller/constants.py` exports `PUBLIC_PORT_BASE = 19000`. A `conftest.py` under `scalesys/tests/` adds `scalesys/controller` to `sys.path` if the existing layout doesn't already (verify before commit).
- `GET /internal/resolve/<slug>/<pod_id>` response gains `pod_port: int = PUBLIC_PORT_BASE + int(pod_id)`. 404/410 negative-cache contract unchanged.
- New `GET /internal/droplet/<droplet_id>/active-pods` → `{pods: [{pod_id, client_id, slug}, ...]}`, protected by the existing `require_auth` decorator (header `X-Scalesys-Token` validated via `secrets.compare_digest(token, SCALESYS_SECRET)` — verified in `agent-manager/app.py:88-97`, same idiom in Scalesys). Filters `WHERE revoked_at IS NULL AND droplet_id = ?`.
- DAM carries its own copy at `services/wannolot-infrastructure/agent-manager/constants.py` (no runtime fetch — DAM must not crash when Scalesys is down). Drift guard = a `make check-constants` target (bash grep for `PUBLIC_PORT_BASE = 19000` in both files, fail on mismatch), wired into CI. **Not** a cross-package pytest — DAM and Scalesys run in separate venvs and cross-import is brittle.
- Old edge build (no `pod_port` read) continues to work — field is additive.

**Files:**
- `services/wannolot-infrastructure/scalesys/controller/app.py`
- `services/wannolot-infrastructure/scalesys/controller/constants.py` (new)
- `services/wannolot-infrastructure/agent-manager/constants.py` (new, copy)
- `services/wannolot-infrastructure/Makefile` (add `check-constants` target)
- `.github/workflows/ci.yml` (invoke `make check-constants`)

**Tests:**
- `pytest scalesys/tests/test_resolve.py` — pin `pod_port == 19000 + pod_id` for pod_ids 1, 2, 80.
- `pytest scalesys/tests/test_active_pods.py` — returns only non-revoked rows for the given droplet; other droplets' allocations excluded; `X-Scalesys-Token` missing/wrong → 401.
- `make check-constants` passes on `main`; hand-edit one constant on a scratch branch and confirm CI fails red.

### Phase 2: DAM bridge lifecycle + fail-closed bootstrap

**Goal:** DAM can expose/unexpose a pod's sidecar-internal `10.X.X.1:18080` on `<DROPLET_PUBLIC_IP>:19000+pod_id`, firewalled to K8s node IPs. Idempotent, reversible, self-healing on DAM restart, no callers yet. Architecture mirrors `wannolot-network.service` — systemd unit + `nsenter` into sidecar PID, not a Docker container.

**Criteria:**
- New systemd template `services/wannolot-infrastructure/systemd/tytus-bridge@.service`. ExecStart shape (mirrors `bootstrap/05-provision-switchailocal.sh:300-301`):
  ```
  ExecStart=/bin/bash -c 'PID=$(docker inspect -f "{{.State.Pid}}" wannolot-%i); \
    exec socat TCP4-LISTEN:$((19000 + 10#%i)),bind=${DROPLET_PUBLIC_IP},fork,reuseaddr \
      EXEC:"nsenter -t $PID -n socat STDIO TCP4\\:10.${DROPLET_OCTET}.$((10#%i)).1\\:18080"'
  ```
  `Restart=on-failure`, `After=docker.service`. Environment file `/etc/wannolot/bridge.env` provides `DROPLET_PUBLIC_IP` and `DROPLET_OCTET`.
- DAM gains endpoints in `agent-manager/app.py`, all `@require_auth`:
  - `POST /pod/<N>/expose` → (a) refuse with 503 `fail_closed_no_allowlist` if `K8S_NODE_IPS` is empty; (b) ensure DROP baseline `iptables -C INPUT -p tcp --dport 19000:19099 -j DROP` exists, installing it with `iptables -A INPUT` (append, so ACCEPTs inserted via `-I INPUT 1` always sit above it); (c) for each IP in `K8S_NODE_IPS`, `iptables -I INPUT 1 -p tcp --dport $((19000+N)) -s <ip>/32 -j ACCEPT -m comment --comment "tytus-bridge-<N>"`; (d) verify via `iptables-save -t filter | grep -c -- '--comment "tytus-bridge-<N>"'` equals `len(K8S_NODE_IPS)` — on mismatch, roll back inserts via the per-rule unexpose path and return 500; (e) `systemctl enable --now tytus-bridge@<N>.service`; (f) post-start health = `systemctl is-active tytus-bridge@<N>` **plus** `ss -H -lnt "sport = :$((19000+N))"` must return a listener bound to `${DROPLET_PUBLIC_IP}`. **Loopback probe removed** — socat binds only to the public IP, so `nc -z 127.0.0.1` is guaranteed to fail; `ss` reads kernel state directly. Returns `{port, status: "active"|"already_active", health: "ok"|"degraded"}`.
  - `POST /pod/<N>/unexpose` → `systemctl disable --now tytus-bridge@<N>`; remove firewall rules with an **explicit per-rule delete loop**: parse `iptables-save -t filter`, for each line containing `--comment "tytus-bridge-<N>"` translate the `-A` spec into an `iptables -D INPUT <spec>` call. **Never use `iptables-save | grep -v | iptables-restore`** — whole-table restore races with concurrent writers (DO agent, fail2ban, Docker) and silently drops their rules. No error if no rules match. Returns `{status: "removed"|"not_found"}`.
- `POST /pod/all/unexpose-stale` → intersect `systemctl list-units 'tytus-bridge@*.service'` and comment-tags from `iptables-save` against Scalesys `active-pods`; unexpose any pod not in the active list.
- `K8S_NODE_IPS` env = comma-separated, injected into the DAM systemd unit (`EnvironmentFile=/etc/wannolot/bridge.env`) from `/etc/wannolot/k8s-allowlist.txt`. `cloud-init/user-data.yml` writes the allowlist file from the committed `k8s-node-allowlist.txt`; a GitOps tick re-renders it. Empty file → DAM refuses to expose (fail-closed); logs `FATAL: k8s allowlist empty, bridge disabled`.
- Reconciliation runs in DAM's existing boot path — append to `_start_background_workers()` in `agent-manager/app.py:909` (verified as the real entry point; called at line 922). Runs synchronously on that thread **before** the health monitor starts, so backfill completes before Provider's first allocate can race it. On Scalesys unreachable, retry 3× with 2s backoff, then log and continue (bridges come up lazily on the next allocate).
- DO cloud firewall audit: `doctl compute firewall list --format Name,DropletIDs,InboundRules` run against current fleet; either add inbound rule `tcp:19000-19099 source=<K8s node IPs>` or record absence in the audit doc. Host iptables ships regardless.
- Port-range pre-flight: `ss -lntp | awk '$4 ~ /:(19[0-9]{3})$/'` on every active droplet must be empty before merge; snapshot captured in the audit doc.

**Files:**
- `services/wannolot-infrastructure/agent-manager/app.py` (endpoints + reconcile call in `_start_background_workers`)
- `services/wannolot-infrastructure/agent-manager/bridge.py` (new — `expose_pod`, `unexpose_pod`, `ensure_drop_baseline`, `verify_rules`, `parse_and_delete_comment_rules`)
- `services/wannolot-infrastructure/systemd/tytus-bridge@.service` (new template unit)
- `services/wannolot-infrastructure/bootstrap/09-install-tytus-bridge.sh` (new — matches `08-install-netns-watchdog.sh` shape: copy unit + daemon-reload)
- `services/wannolot-infrastructure/cloud-init/user-data.yml` (writes `/etc/wannolot/k8s-allowlist.txt` and `/etc/wannolot/bridge.env`)
- `services/wannolot-infrastructure/k8s-node-allowlist.txt` (new — commit the three real `kubectl get nodes -o wide` ExternalIPs for `k8s-blox/lon1`)

**Tests:**
- `pytest agent-manager/tests/test_bridge.py` (subprocess stubbed):
  - `expose` twice → systemd unit enabled once, rules present once; second call returns `already_active`.
  - `unexpose` on non-existent pod returns `not_found`, no exception; iptables delete loop called with zero matches is a no-op.
  - Missing DROP baseline on boot → baseline appended (not inserted), ordering check confirms ACCEPTs land at position 1 and DROP at the tail.
  - `iptables-save` verify step catches a rule that failed to insert (stub returns wrong count) → expose rolls back via the per-rule delete path and returns 500.
  - Unexpose rule parser: given a stubbed `iptables-save` output with three tytus-bridge-02 rules plus unrelated DO/fail2ban rules, only the three tagged rules are deleted; unrelated rules survive.
  - `unexpose-stale` removes bridges with no Scalesys entry; leaves valid ones alone.
  - Empty `K8S_NODE_IPS` env → expose returns 503 `fail_closed_no_allowlist`; no systemd or iptables calls made.
  - Health probe uses `ss` + `systemctl is-active`; stubbed `ss` returning no listener → `health: "degraded"`.
- Manual on one staging droplet: from a non-allowlisted host, `nc -zv <droplet_ip> 19001` returns connection refused/timeout; from a K8s worker node, the same succeeds.

### Phase 3: Edge + Provider cutover + backfill verification

**Goal:** Edge dials `pod_port` from the resolve payload. Provider wires `expose` into allocate and `unexpose` into revoke so the bridge tracks pod lifecycle. Currently-active allocations on `strato-eu-001` reach via edge end-to-end.

**Verified call-site map** (re-checked against `services/wannolot-provider/src/routes/pod.js` at sprint drafting):
  - `pod.js:131-139` — `GET /config/pod-key/<N>` (read). **No expose.**
  - `pod.js:157-170` — `POST /agent/<N>/deploy` during allocate, fire-and-forget (`damIp` at :157, deploy at :162). **Expose needed (blocking — see behavior change).**
  - `pod.js:387` — `POST /pod/revoke`; `cleanupAgentOnRevoke` at `:415`. **Unexpose needed alongside cleanup.**
  - `pod.js:456` — `callDAM` helper (proxied user-triggered ops).
  - `pod.js:477-479` — `GET /agent/<N>/status`. **No expose.**
  - `pod.js:490-500` — `POST /agent/<N>/deploy` (user switches agent type). **Expose idempotent self-heal** — fire-and-forget, log failures.
  - `pod.js:532-534` — `POST /agent/<N>/restart`. **No expose** (reconciled by DAM boot loop).
  - `pod.js:543-553` — `POST /agent/<N>/exec`. **No expose.**
  - `pod.js:562-564` — `POST /agent/<N>/stop`. **No unexpose** — unexpose fires only on revoke.

**Behavior change (requires product sign-off before merge):** today's allocate calls DAM `/agent/<N>/deploy` fire-and-forget and returns to the user without waiting. After this sprint, `expose` becomes **blocking** with a 45s timeout inside `/pod/request`; expose 5xx returns 502 to the client, because handing back a pod URL that can't route is worse than a slower allocate. Agent deploy stays fire-and-forget.

**Criteria:**
- `services/wannolot-edge/plugin/tytusresolver/tytus_resolver.go`:
  - Add `PodPort int \`json:"pod_port"\`` field to the `resolveResult` struct, populate from the Scalesys response.
  - **Always** set the `tytus.pod_port` Caddy placeholder before returning — default to `"18080"` when the field is absent or zero (old Scalesys during staging rollout). Never leave the placeholder unset; if unset, `reverse_proxy {tytus.droplet_ip}:{tytus.pod_port}` dials `X.Y.Z.W:` and 502s.
  - Log `pod_port_missing` at WARN once per resolve when the field is absent so the staging-drift window is observable.
- `services/wannolot-edge/Caddyfile`: `reverse_proxy {tytus.droplet_ip}:{tytus.pod_port}` — the literal `:18080` is dropped.
- `services/wannolot-provider/src/routes/pod.js`:
  - `/pod/request` happy path: after Scalesys assign succeeds, `POST http://<droplet_ip>:8099/pod/<N>/expose` with header `X-Scalesys-Token: ${scalesysConfig.token}` (matches the idiom at `pod.js:164`; DAM validates via `secrets.compare_digest` at `agent-manager/app.py:91-92`). No `DAM_TOKEN`, no `Authorization: Bearer`.
  - Expose 5xx or timeout → Provider returns 502 `pod_expose_failed`; pod is not handed back. Scalesys allocation stays (reconciled by DAM's next `unexpose-stale` or a follow-up user revoke — document this trade-off).
  - `/pod/agent/deploy` (`:500`): after `callDAM` deploy succeeds, fire-and-forget `POST /pod/<N>/expose` (idempotent self-heal, failure logged only).
  - `/pod/revoke` cleanup (`:415`): in `cleanupAgentOnRevoke`, add `POST /pod/<N>/unexpose` alongside the existing agent-stop. Log-only on 5xx.
- `dev/scripts/verify-edge-bridges.sh` — lists all active allocations from Scalesys, curls `https://<slug>.tytus.traylinx.com/p/<N>/healthz` with the owner's bearer against `--resolve …:443:143.198.241.20`, expects 200 for every row. Non-zero exit on any failure. Output committed to `dev/audits/2026-04-2X-edge-e2e.md`.

**Files:**
- `services/wannolot-edge/plugin/tytusresolver/tytus_resolver.go`
- `services/wannolot-edge/Caddyfile`
- `services/wannolot-provider/src/routes/pod.js` (allocate, switch-agent, revoke)
- `dev/scripts/verify-edge-bridges.sh` (new)
- `dev/audits/2026-04-2X-edge-e2e.md` (new — curl output, external-scan evidence, DO firewall audit, port-range pre-flight snapshot)

**Tests:**
- Go plugin unit: resolve mock returns `{droplet_ip:"X", pod_port:19002}` → placeholder resolves to `"19002"`; mock without the field → placeholder resolves to `"18080"` + one WARN log asserted.
- Provider unit (jest mocks `axios`):
  - Allocate happy path: DAM expose called exactly once with URL `…:8099/pod/<N>/expose` and header `X-Scalesys-Token: <cfg>`; port math `19000 + parseInt(pod_id)`.
  - DAM expose 500 → `/pod/request` returns 502 `pod_expose_failed`, no pod body; Scalesys revoke is **not** called (reconciled, not unwound — documented).
  - Revoke path: unexpose called; 5xx from DAM → revoke still returns 200 with a log line.
- E2E after deploy (captured in audit doc):
  - `curl --resolve <slug>.tytus.traylinx.com:443:143.198.241.20 -H 'Authorization: Bearer sk-tytus-user-…' https://<slug>.tytus.traylinx.com/p/02/v1/models` returns 200 + model list for both currently-active pods on `strato-eu-001`.
  - External scan from a non-allowlisted host: `nmap -p 19000-19099 <droplet_ip>` returns all filtered/closed.
  - DO cloud firewall state + `ss -lntp` port-range snapshot attached.
- Rollback drill (scratch branch, not merged): revert Provider's expose call, run `POST /pod/all/unexpose-stale` on a staging droplet, confirm edge returns 502 and no public listener remains on 19000–19099. Documented in the audit.

## Exit criteria

1. `verify-edge-bridges.sh` green for every active allocation on `strato-eu-001`.
2. `curl https://<slug>.tytus.traylinx.com/p/<N>/v1/models` → 200 + model list for both active pods, saved in `dev/audits/2026-04-2X-edge-e2e.md`.
3. External scan from a non-allowlisted IP confirms no listener in 19000–19099 accepts — captured in the audit.
4. DO cloud firewall state documented (rules added or absence recorded) in the audit.
5. `make check-constants` green on `main`.
6. Phase 4 (CLI dual-path) unblocked.

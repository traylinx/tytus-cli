# Sprint — Per-Pod Subdomain (2026-04-23)

> Status: **NOT STARTED**. This doc is the handoff from the planning session
> on 2026-04-23; everything a fresh context needs to execute is embedded
> below. No external link-chasing required.

---

## TL;DR

Replace every pod URL of shape `{slug}.tytus.traylinx.com/p/{NN}/…` with
`{slug}-p{NN}.tytus.traylinx.com/…` so each pod is its own **browser
origin**. Fixes the bug where opening pod 02 and pod 04 in the same
browser made them fight over the same `localStorage` keys (the
OpenClaw Control SPA stores `gatewayToken` + WebSocket URL by origin,
not by path → second pod's token clobbers the first's → auth mismatch
→ pod rate-limits you out).

The public DNS wildcard `*.tytus.traylinx.com` and the Let's Encrypt
wildcard cert already cover this one level deep. No DNS work, no cert
work. Just code.

---

## Why this is the right fix

Alternatives considered and rejected:

| Option | Why rejected |
|---|---|
| Open each pod in a Chrome profile / incognito | Manual, brittle, user gets confused |
| Tray opens each pod with `--incognito` | Windows close = state gone; stop-gap only |
| Path-scoped `localStorage` in OpenClaw Control | Upstream PR; out of our control, slow |
| Use the localhost forwarder (each pod already a distinct port = origin) | Caps bandwidth at ~4 KB/s over WireGuard; not a real product path |
| **Per-pod subdomain** | One infra change, fixes it forever, scales to N pods | ← this sprint |

---

## Current architecture (read before editing)

### Edge (the linchpin)

* **Repo:** `traylinx/wannolot-edge` — checked out locally at
  `services/wannolot-edge/`.
* **Runtime:** Caddy 2.8 + custom Go plugin `tytusresolver`.
* **Deploy:** K8s LoadBalancer `wannolot-edge` in the `production`
  namespace, IP `143.198.241.20`. Autoscales 3-10 replicas. CI on
  push to `production` branch rebuilds + rolls the Deployment.
* **Caddyfile** (`services/wannolot-edge/Caddyfile`):
  * Wildcard site matcher: `*.{$EDGE_BASE:tytus.traylinx.com}` — this
    already accepts `{slug}-p{NN}.tytus.traylinx.com` because it's one
    level deep. **No Caddyfile change needed.**
  * `protocols h1` only — WebSocket needs extended-CONNECT, Caddy
    doesn't wire it through h2/h3 yet. Don't touch this.
* **Plugin ServeHTTP pipeline** (in
  `services/wannolot-edge/plugin/tytusresolver/tytus_resolver.go`):
  1. `host := req.Host; slug = host[:firstDot]` — slug from the
     subdomain (today: always the full prefix).
  2. `m := podPathRE.FindStringSubmatch(req.URL.Path)` — pod from
     the path regex `^/p/([0-9]{1,4})(/.*)?$`. Rejects with 404 if
     no match.
  3. `tail := m[2]` — path with `/p/NN` stripped; `""` → `/`.
  4. `r.resolve(ctx, slug, podID)` → Scalesys
     `GET /internal/resolve/<slug>/<pod>`. Cached LRU.
  5. Route class: `isLLMRoute = (tail == "/v1" || tail.startsWith("/v1/"))`.
     LLM routes require Bearer. UI routes accept Bearer, `?token=`,
     or the `tytus_pod_token` cookie.
  6. Cookie issuance on first `?token=` hit — `Path=/p/<NN>` (no
     trailing slash — RFC 6265 quirk; don't "fix" it).
  7. `{tytus.upstream_port}` populated: LLM → pod_port (= `PUBLIC_PORT_BASE+pod = 19000+pod`),
     UI → ui_port (= `OPENCLAW_PORT_BASE+pod = 21000+pod`). Both come
     back in the resolve response.

### Scalesys

* **Handler:** `services/wannolot-infrastructure/scalesys/controller/app.py`.
* **Slug table:** `user_slugs(client_id, slug)` — per-user 60-bit
  Crockford base32, populated lazily by `ensure_slug(conn, client_id)`.
  Every active user has one after their first pod op.
* **User-key endpoint:** `@app.route("/api/user-keys/<client_id>")`
  at `app.py:1182`. Today returns:
  ```json
  { "client_id": "...", "stable_key": "...", "stable_ai_endpoint":
    "http://10.42.42.1:18080", "slug": "njc9ctj3zgkn" }
  ```
  **Does NOT currently return a URL — Provider builds the URL from the
  slug.** This is why we need to change the URL-shape decision at
  Provider, or teach Scalesys to emit a template string.
* **Resolve endpoint:** `/internal/resolve/<slug>/<pod_id>` at
  `app.py:1317`. Returns `{client_id, droplet_ip, pod_port, ui_port, status}`.
  No change needed unless the plugin sends the pod via a different
  shape — we'll pass pod_id explicitly either way.

### Provider (Node.js / Express)

* **Repo:** `traylinx/wannolot-provider`. Local checkout at
  `services/wannolot-provider/`.
* **`/pod/user-key` handler:** `src/routes/pod.js:351`. Proxies to
  Scalesys `/api/user-keys/<client_id>`, then builds the response
  delivered to the CLI:
  ```js
  res.json({
    stable_ai_endpoint: "...",
    stable_user_key: resp.data?.stable_key,
    slug: resp.data?.slug,
    public_url: slug ? `https://${slug}.${EDGE_BASE}` : null,
  })
  ```
* **`EDGE_BASE`** env → `tytus.traylinx.com`.
* Provider is the one that builds `public_url` today. **This is where
  we introduce the new shape.**

### CLI / pods crate

* `services/tytus-cli/pods/src/user_key.rs` — parses the response:
  ```rust
  struct UserKey { endpoint, key, slug, public_url }
  ```
* `services/tytus-cli/cli/src/main.rs`:
  * Line 3030 and 3105 build per-pod URL as `format!("{}/p/{}", public.trim_end_matches('/'), pod.pod_id)`.
  * Stores `edge_public_url` (BASE, no pod path) and `edge_slug` in
    state.json per pod.
* `services/tytus-cli/tray/src/main.rs`:
  * `PodInfo::public_pod_url()` at line 323 does the same
    `format!("{}/p/{}", …)` composition.
  * `public_ui_url()` at line 333 appends `/?token={gateway_token}`.
  * `impl PodInfo` at line 319.
* `services/tytus-cli/tray/src/web_server.rs` (wizard):
  * Already computes `public_url = format!("{}/p/{}", …)` in
    `compute_state_snapshot()`. Same composition.
  * Ships `derive_gateway_token()` using `sha256(pod_api_key || pod_id)[:48]`
    because freshly-installed pods have `gateway_token: null` in state.json
    until `tytus env` runs. **This derivation stays post-sprint as a
    safety net; post-sprint it should still match the pod's actual
    token (verified — pod 02 stored token byte-matches derivation).**
* `services/tytus-cli/tray/src/socket.rs`:
  * `build_pod_info()` applies the same `derive_gateway_token` +
    `shared_edge_base` fallbacks used by the wizard. Introduced in
    the 2026-04-22 session.

### State files + runtime

* State.json: `~/Library/Application Support/tytus/state.json` on macOS.
  Per-pod fields that matter: `pod_id`, `pod_api_key`, `stable_user_key`,
  `edge_slug`, `edge_public_url`, `gateway_token`.
* `pod_api_key` is **always** present in state.json after install; the
  daemon socket (`/tmp/tytus/daemon.sock`) redacts it (secret hygiene)
  so the tray cross-refs state.json via `load_api_keys_from_state()` in
  `socket.rs`.

---

## Target architecture

### URL shape

```
{slug}-p{NN}.tytus.traylinx.com
```

Example: `njc9ctj3zgkn-p02.tytus.traylinx.com`.

* Pod id is the two-digit zero-padded form (`02`, `04`, `12`). Keep
  parsing tolerant: `p4`, `p04`, `p004` all resolve the same way
  (Scalesys's `clients.pod_id` column is stored `"02"`-style — pad to
  2 digits on ingress).
* Slug pattern is unchanged (60-bit Crockford base32, validated by
  `validSlug()` in the edge plugin).

### Why `-p{NN}` and not alternatives

| Candidate | Verdict |
|---|---|
| `njc9ctj3zgkn-p02.tytus.traylinx.com` | **Chosen.** Unambiguous, fits 1-level wildcard, ~20 chars |
| `njc9ctj3zgkn.p02.tytus.traylinx.com` | Needs 2-level wildcard cert — reject |
| `p02-njc9ctj3zgkn.tytus.traylinx.com` | Pod id first hurts eyeball grouping — reject |
| `njc9ctj3zgkn-02.tytus.traylinx.com` | Ambiguous if slugs ever end with digits — reject |

### Auth semantics (unchanged)

* LLM route (`/v1/*` at the origin root): `Authorization: Bearer sk-tytus-user-<32hex>`.
* UI route (everything else): Bearer, `?token=<gateway_token>`, or `tytus_pod_token`
  cookie.
* Cookie scope now `Path=/` (the origin IS per-pod — the whole host
  is the pod's world).

---

## Implementation plan

Zero-downtime rollout. Old URL path keeps working at every phase
until the last one rips it out.

### Phase 1 — Edge plugin accepts BOTH host formats

**File:** `services/wannolot-edge/plugin/tytusresolver/tytus_resolver.go`

**Change:** at the start of `ServeHTTP` (line ~166), parse host into
optional `(slug, pod_id_from_host)`. If pod_id_from_host is
populated, skip the `podPathRE` check and set `tail = req.URL.Path`
unchanged.

Pseudo-diff:

```go
// Regex for the new hostname shape. Slug is lowercase alphanum only
// (matches existing validSlug). pod is 1–4 digits, zero-padded later.
var hostPodRE = regexp.MustCompile(`^([a-z0-9]+)-p([0-9]{1,4})$`)

// Inside ServeHTTP, after host trim:
var slug string
var podFromHost string
if m := hostPodRE.FindStringSubmatch(host[:firstDot]); m != nil {
    slug = m[1]
    podFromHost = m[2]
} else {
    slug = strings.ToLower(host[:firstDot])
}

// Pod resolution:
var podID string
var tail string
if podFromHost != "" {
    // New per-pod origin. Path is passed through.
    podID = podFromHost
    tail = req.URL.Path
    if tail == "" { tail = "/" }
} else {
    // Legacy /p/NN/ path routing.
    m := podPathRE.FindStringSubmatch(req.URL.Path)
    if m == nil { … 404 … }
    podID = m[1]
    tail = m[2]
    if tail == "" { tail = "/" }
}

// Zero-pad to 2 digits as before.
if len(podID) < 2 { podID = strings.Repeat("0", 2-len(podID)) + podID }
```

**Cookie path (line ~319):** when serving a per-pod-origin request,
issue the cookie at `Path=/`. When serving a legacy path, keep
`Path=/p/<NN>`. Use `podFromHost != ""` as the toggle.

**Path rewrite (line ~350-ish, before `next.ServeHTTP`):** only
strip `/p/NN` prefix in the legacy branch. On the new branch, leave
`req.URL.Path` untouched.

**Tests** in `tytus_resolver_test.go`:

```go
func TestHostParsing(t *testing.T) {
    cases := []struct {
        host, wantSlug, wantPod string
    }{
        {"njc9ctj3zgkn.tytus.traylinx.com", "njc9ctj3zgkn", ""},          // legacy
        {"njc9ctj3zgkn-p02.tytus.traylinx.com", "njc9ctj3zgkn", "02"},    // new
        {"njc9ctj3zgkn-p4.tytus.traylinx.com", "njc9ctj3zgkn", "04"},     // padded
        {"NJC9CTJ3ZGKN-P02.tytus.traylinx.com", "njc9ctj3zgkn", "02"},    // case-fold
        {"badslug_.tytus.traylinx.com", "", ""},                          // reject
    }
    // ...
}
```

Run: `cd services/wannolot-edge && go test ./...`

**Gotcha from Phase 2.5 memory (lesson #8):** browsers don't send
Bearer on a URL click. The `?token=` → session-cookie handoff is what
makes the UI reachable from a shared link. Preserve it exactly.

### Phase 2 — Scalesys + Provider emit new URL

**File:** `services/wannolot-infrastructure/scalesys/controller/app.py`

* Add constant: `EDGE_BASE = os.environ.get("EDGE_BASE", "tytus.traylinx.com")`.
* Add helper:
  ```python
  def _pod_public_url(slug, pod_id):
      pod2 = str(pod_id).zfill(2)
      return f"https://{slug}-p{pod2}.{EDGE_BASE}"
  ```
* Extend `get_user_key()` response (line 1182) with:
  * `pod_public_url_template`: `f"https://{slug}-p{{pod_id}}.{EDGE_BASE}"`
    (with the literal `{pod_id}` substring for the caller to format
    per-pod).

Alternative if we don't want templating in the response: extend
`request_pod` + `request_default_pod` to include `pod_public_url` in
their response. That's cleaner because those responses are per-pod.

Look at `app.py` search for `stable_ai_endpoint` — the two endpoints
that build responses are around line 443 and line 556 (`request_pod`
and `request_default_pod`). Add `pod_public_url` to both.

**File:** `services/wannolot-provider/src/routes/pod.js`

* Line 351's `/pod/user-key` handler: include `pod_public_url_template`
  in the response when slug is present:
  ```js
  res.json({
    ...existing...,
    slug,
    public_url: slug ? `https://${slug}.${EDGE_BASE}` : null,   // keep for back-compat
    pod_public_url_template: slug ? `https://${slug}-p{pod_id}.${EDGE_BASE}` : null,
  })
  ```
* Lines 208 and 288 (`request_pod` and `request_default_pod`
  passthroughs): pass any `pod_public_url` from Scalesys straight
  through to the CLI.

### Phase 3 — CLI + tray prefer the per-pod URL

**File:** `services/tytus-cli/pods/src/user_key.rs`

* Add `pod_public_url_template: Option<String>` to `UserKeyResponse`
  and `UserKey`.

**File:** `services/tytus-cli/pods/src/request.rs` (and `default_pod.rs`
if present)

* Add `pod_public_url: Option<String>` to the allocation response
  struct.

**File:** `services/tytus-cli/cli/src/state.rs`

* Add `pod_public_url: Option<String>` to `PodEntry`.

**File:** `services/tytus-cli/cli/src/main.rs`

* At install time (line ~1798 `state.pods.push(PodEntry { … })`),
  populate `pod_public_url` from the allocation response.
* `get_user_key_full`-driven backfill (line 2972+ and 3079+) also
  fills `pod_public_url` from the template + this pod's id.
* Lines 3030 and 3105 — replace the `format!("{}/p/{}", public, pod_id)`
  construction with:
  ```rust
  // Prefer per-pod origin URL; fall back to legacy path composition
  // for state entries written before this sprint.
  pod.pod_public_url.clone()
      .unwrap_or_else(|| format!("{}/p/{}", public.trim_end_matches('/'), pod.pod_id))
  ```

**File:** `services/tytus-cli/tray/src/main.rs`

* Add `pub pod_public_url: Option<String>` to `PodInfo` (line 297).
* Rewrite `public_pod_url()` (line 323):
  ```rust
  pub fn public_pod_url(&self) -> Option<String> {
      if let Some(u) = &self.pod_public_url {
          return Some(u.clone());
      }
      self.edge_public_url.as_ref()
          .map(|u| format!("{}/p/{}", u.trim_end_matches('/'), self.pod_id))
  }
  ```
* `public_ui_url()` (line 333) keeps working because it already composes
  off `public_pod_url()`.

**File:** `services/tytus-cli/tray/src/socket.rs`

* Extend `build_pod_info()` to read `pod_public_url` from the input
  JSON + state.json. Fallback chain:
  1. Input JSON's `pod_public_url`
  2. Shared base + compose `/p/NN` (legacy)
  3. state.json lookup for same pod
* Extend `daemon_status` schema checks (line 145+) to pass
  `pod_public_url` through.

**File:** `services/tytus-cli/tray/src/web_server.rs`

* `compute_state_snapshot()`: same update to `AgentSlot.public_url` —
  read pod's `pod_public_url` first, fall back to composed path.
* `derive_gateway_token()` and `load_api_keys_from_state()` stay —
  they're still correct fallbacks for pods whose backfill hasn't
  reached state.json.

### Phase 4 — deprecate `/p/NN` path routing

After 7 days with no state.json files in the wild carrying the old
URL form (verify via a grep in any support-case dump or a tracked
metric on the old-format route):

* Delete the legacy branch in the edge plugin.
* Delete the `format!("{}/p/{}", …)` fallbacks in CLI/tray.
* Delete `public_url` base-form from Provider response (keep `pod_public_url`).
* Drop the `podPathRE` + cookie path `/p/<NN>` handling.

Tracking: add a Prometheus counter to the edge plugin that increments
on the legacy branch. When it's 0 for 7 days, safe to pull the
trigger.

---

## Lessons from prior sprints (don't repeat)

From `memory/project_phase25_complete.md`:

1. `cp -r src/* dst/` aborts under `set -e` when `src == dst` (installer
   gotcha on droplets where state gets cloned to `/opt/wannolot-infrastructure`).
2. `gitops/pull-and-apply.sh` DAM source-hash drift: `bootstrap/` changes
   alone don't trigger a DAM reinstall — bump `_SCHEMA_VERSION` in
   `agent-manager/constants.py` to force it.
3. Caddy `{tytus.droplet_ip}` is a per-request placeholder — can't be
   used in active health checks (resolves once at config load).
4. Provider env flags accept `1`/`true`/`yes`/`on` — match all four in
   code, K8s manifests use `"1"`, `.env` files use `"true"`.
5. Pod gateway health path is `/health`, not `/healthz` (which is a
   DAM path).
6. OpenClaw `config.user.json` overlay schema is fragile — rejects
   `id`/`output[]`/`tools` and top-level `memoryEmbedding`. Use `name`.
7. socat binds to public IP only — loopback `nc 127.0.0.1` always fails.
   Probe via `ss -H -lnt "sport = :PORT"` on kernel state.
8. **Edge URL is API-only — browsers can't open it directly.** Browsers
   don't send `Authorization: Bearer` from a URL click. The UI path
   needs the `?token=` → cookie handoff that the plugin does today.
   Don't remove it.

From `memory/project_phase26_ui_fast_path.md` (allowlist set-equality
fix, keep in mind when touching edge iptables):

* The droplet's K8s-node allowlist must be enforced by **set
  equality**, not superset — otherwise rotating an edge node that
  didn't exist at boot silently gets blocked.

From 2026-04-22 session (this repo, this week):

* `gateway_token` derivation is `sha256(pod_api_key || pod_id)[:48]`.
  Verified byte-match against pod 02's stored token.
  Derivation lives in TWO places today:
  - `services/tytus-cli/tray/src/web_server.rs` (`derive_gateway_token`)
  - `services/tytus-cli/tray/src/socket.rs` (`derive_gateway_token`)
  Keep both as fallbacks post-sprint — new state.json may not have
  `gateway_token` filled until `tytus env` runs.
* `zero-config` hook (`configure_nemoclaw_for_zero_auth` in
  `cli/src/main.rs:1960`) silently fails during install if the pod
  container isn't ready yet (the `exec_in_agent` times out). This
  leaves `gateway_token: null` in state.json. The tray's derivation
  is the safety net. Sprint doesn't fix this — separate ticket.
* Pod 04 rate-limited itself on 2026-04-22 after the wizard fed it
  pod-02's cached localStorage token. **This entire sprint exists to
  prevent that localStorage bleed.**

---

## Test matrix (must all pass before declaring done)

| # | Scenario | How |
|---|---|---|
| 1 | New URL shape resolves via edge plugin | `curl -H "Authorization: Bearer sk-tytus-user-…" https://{slug}-p02.tytus.traylinx.com/v1/models` → 200 |
| 2 | Legacy URL still resolves (back-compat) | Same curl against `{slug}.tytus.traylinx.com/p/02/v1/models` → 200 |
| 3 | `?token=` → cookie handoff works at new origin | `curl -c /tmp/c.txt "https://{slug}-p02.tytus.traylinx.com/?token=X"` then `curl -b /tmp/c.txt "https://{slug}-p02.tytus.traylinx.com/assets/foo.js"` → 200 |
| 4 | Cookie scope Path=/ (only issued on new origin) | inspect `Set-Cookie` — must be `Path=/`, not `/p/NN` |
| 5 | Slug-owner mismatch still blocks | Bearer from user A against user B's slug → 403 |
| 6 | `validSlug` still rejects non-conformant slugs | `fo_obar-p02.tytus.traylinx.com` → 404 (underscore not in Crockford) |
| 7 | Pod 02 and pod 04 open simultaneously in same browser | two tabs, each loads OpenClaw Control, each stays connected — no "token mismatch" |
| 8 | Fresh install → new pod immediately reachable at `-p{NN}` URL | install, no wait; `curl` the new URL → 200 |
| 9 | Tray menu "Open in Browser (fast)" uses new URL | click, check browser's URL bar |
| 10 | Wizard "Already running" panel's API URL field shows new shape | state dump |
| 11 | Install wizard success-screen URLs are new shape | inspect panel |
| 12 | `tytus env --export` emits new URL | run, check output |
| 13 | `tytus env --raw` still emits WG path (debug flag) | keeps working |
| 14 | Revoke clears the pod's subdomain route | `curl` after revoke → 404 from plugin |
| 15 | E2E matrix `dev/scripts/edge-e2e-matrix.sh` passes | re-run the 17-case script |

---

## Verification / proof so far (carry forward)

These were proven in the 2026-04-22 session on a live pod 04 that
was stuck at the old URL shape:

* `pod_api_key` for pod 04 from state.json → deterministic `sha256(pod_api_key + "04")[:48] = 77a7c74a0e05ef9f3364e66d36d1b62098a6977efde030a6`. Matched the
  pod's config.json byte-for-byte.
* `curl -L "https://njc9ctj3zgkn.tytus.traylinx.com/p/04/?token=77a7c74a…"` → HTTP 200 + OpenClaw Control HTML.
* Same URL without `?token=` → HTTP 401 (edge bouncer).
* Bearer-auth'd probe to `/v1/models` → 200 with model list — pod's
  gateway is live.

The pod answers. The new subdomain just changes the *host* part
(and stops the `/p/NN` path strip). Upstream routing on the droplet
side is unchanged.

---

## What I can do vs what requires a push

**Can do locally (no deploy):**

* All code changes listed above.
* Compile + unit tests (`go test`, `cargo test`, Python syntax check).
* Local wizard smoke (build tray, install over `/Applications/Tytus.app`,
  restart via `launchctl kickstart`).
* Curl tests against the EXISTING live production (validates
  back-compat regression on nothing we changed).

**Requires you to push a `production` branch** (deployment is gated by
CI — I won't push without being asked):

* Edge plugin → `traylinx/wannolot-edge`.
* Scalesys → `traylinx/wannolot-infrastructure`.
* Provider → `traylinx/wannolot-provider`.

All three branches auto-deploy from `production` via Netlify/K8s CI.
Roll out in order (edge → Scalesys+Provider → tray rebuild/install).

---

## Commands ready to run

```bash
# From repo root. Each one should run clean before moving on.

# Phase 1 build + test
cd services/wannolot-edge
go build ./...
go test ./...

# Phase 2 lint (Python + JS)
python3 -m py_compile services/wannolot-infrastructure/scalesys/controller/app.py
cd services/wannolot-provider && node --check src/routes/pod.js

# Phase 3 build + test + install
cd services/tytus-cli
cargo build --release -p tytus-tray
cargo test --workspace
cp target/release/tytus-tray /Applications/Tytus.app/Contents/MacOS/Tytus
PID=$(cat /tmp/tytus/tray.pid); kill "$PID" 2>/dev/null
launchctl kickstart -k "gui/$(id -u)/com.traylinx.tytus.tray"

# Wizard smoke
PORT=$(cat /tmp/tytus/tray-web.port)
curl -s "http://127.0.0.1:$PORT/api/state" | jq '.agents[] | {pod_id, public_url, api_url, ui_url}'

# Live verify the new URL shape returns 200 (after production deploy)
curl -sI "https://$(cat state.json | jq -r '.pods[] | select(.pod_id=="02") | .pod_public_url')/v1/models" \
  -H "Authorization: Bearer $(cat state.json | jq -r '.pods[0].stable_user_key')"
```

---

## Files the fresh context should read BEFORE starting

In order of importance:

1. **This sprint doc** (you're already here).
2. `memory/project_phase25_complete.md` — the full public-HTTPS-per-pod
   shipping notes. Lessons 1-8 apply.
3. `memory/reference_wannolot_architecture.md` — service map, repos,
   deploy targets.
4. `memory/project_phase26_ui_fast_path.md` — the `?token=` → cookie
   handoff mechanism this sprint must preserve.
5. `services/wannolot-edge/Caddyfile` (~120 lines) and
   `services/wannolot-edge/plugin/tytusresolver/tytus_resolver.go`
   (~500 lines).
6. `services/wannolot-infrastructure/scalesys/controller/app.py`
   around line 1182 (user-key handler), 1317 (internal resolve),
   685-715 (slug generator).
7. `services/tytus-cli/pods/src/user_key.rs` (~90 lines).
8. `services/tytus-cli/tray/src/main.rs` lines 290-370
   (`PodInfo` + its methods).
9. `services/tytus-cli/tray/src/web_server.rs`
   `compute_state_snapshot()` + `derive_gateway_token()`.
10. `services/tytus-cli/tray/src/socket.rs`
    `build_pod_info()`.

---

## Explicit non-goals

* **Not** fixing the `configure_nemoclaw_for_zero_auth` post-install
  retry problem (separate ticket).
* **Not** changing the WebSocket `protocols h1` constraint in the
  Caddyfile.
* **Not** moving to 2-level wildcard certs.
* **Not** touching DAM / droplet bridges — they already key on pod
  ports, not URLs.
* **Not** re-architecting the slug allocation scheme — slugs stay
  per-user, not per-pod.

---

## Acceptance criteria

* All 15 scenarios in the test matrix pass.
* A single browser with pod 02 + pod 04 tabs open stays healthy for
  10 minutes without any "token mismatch" error.
* Legacy URL form still returns 200 (back-compat).
* Edge plugin's legacy-branch metric counter exists and can be
  watched in Prometheus.
* Sprint memory written to `~/.claude/projects/…/memory/` summarizing
  what shipped + any surprises for the next session.

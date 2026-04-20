# Tytus Performance Audit — 2026-04-20

**Trigger:** user report that the OpenClaw web UI at
`http://localhost:18702/chat` is slow on page reload. "Tytus is
absolutely not production ready."

**Scope:** end-to-end request path from browser → tytus forwarder → WG
tunnel → droplet → OpenClaw → SwitchAILocal. Find root causes,
distinguish bugs from architectural ceilings, inventory the
production-readiness gap. **No implementation in this pass.**

**Conclusion up front:** the slowness is real, it's measurable, and
it's not a bug. It's a ceiling of the current architecture —
specifically userspace WireGuard on macOS combined with all traffic
(static assets + LLM/API) sharing the same 4 KB/s tunnel.

---

## 1 · Hard numbers (measured 2026-04-20)

Measurements taken on user's MacBook with pod 02 (nemoclaw/OpenClaw)
allocated and tunnel active on `utun4 → 10.18.2.1`.

| Measurement | Result | Verdict |
|---|---|---|
| WG tunnel ping (10.18.2.1) | 24 ms RTT, 0% loss | fine |
| Forwarder localhost overhead | ~500 μs | fine |
| LLM gateway `/v1/models` (2.5 KB) — cold | 3.8 s TTFB | rekey stall |
| LLM gateway `/v1/models` — warm | 127 ms | fine |
| Small cached JS assets (1–46 KB) | 1–4 ms (disk hit) | fine |
| CSS bundle (214 KB) — cache hit | 1.8 ms | fine |
| **OpenClaw main JS bundle (689 KB) — cold fetch** | **171.5 s** (4,022 B/s) | **BROKEN UX** |
| Same bundle — cache hit | 3.4 ms (200 MB/s local read) | fine |

**Translation:** the tunnel's sustained throughput is ~4 KB/s. With a
689 KB JS bundle that OpenClaw ships and never lazy-loads, every cold
page load costs ~3 minutes. Once cached, everything is instant.

---

## 2 · Root cause

### 2.1 · Userspace WireGuard throughput ceiling

`tunnel/src/wireguard.rs` uses **boringtun** (Cloudflare's pure-Rust
userspace WireGuard). On macOS this tops out at ~3–5 KB/s sustained
because:

- X25519 + ChaCha20-Poly1305 runs **synchronously** on the packet loop
  — no SIMD, no AES-NI, no async crypto pipeline.
- TUN I/O goes through macOS's utun userspace API (extra memory copies
  per packet).
- Single `Tunn` Mutex means one-at-a-time packet processing.

The codebase already admits this at `cli/src/main.rs:4425–4428`:

> "Throughput over userspace WireGuard on macOS is ~3 KB/s sustained
> for the 689 KB bundle OpenClaw ships — that's ~4 minutes for the
> initial paint, every single tab reload."

**Kernel WireGuard would be 20–100× faster.** macOS does not ship one.
Getting kernel-speed WG on macOS requires either:

- A signed System Extension (Apple Developer Program + notarization +
  user "Allow in Privacy & Security" approval), or
- Shipping a second VPN-style install flow (pfSense-style, adds a
  system-profile-level complexity).

Both are big decisions. Neither is a quick fix.

### 2.2 · All traffic through one tunnel

Static assets (OpenClaw JS/CSS), control-plane traffic (WebSocket
agent loop), and inference traffic (LLM tokens) all multiplex through
the same 4 KB/s pipe. A 689 KB asset fetch starves every other call
for 3 minutes.

Data plane and control plane are not separated. There is no CDN. The
tunnel is the only path in.

### 2.3 · Secondary: cache design makes the cliff feel random

1. Cache lives in `/tmp/tytus/ui-02-cache/` — wiped on reboot and by
   macOS's periodic `/tmp` sweep. Users hit cold loads on "random"
   days.
2. Cache key is the hashed filename (`index-Dts6VHgr.js`). Every
   OpenClaw version bump changes the hash → full cold load.
3. No compression passthrough. Assets travel uncompressed through WG.
   A gzip'd 689 KB bundle would be ~180 KB → 45 s instead of 172 s.
4. 3.8 s rekey stall on first request after idle (boringtun session
   recovery). Users come back to a tab after lunch → first click is
   slow for no obvious reason.
5. Existing asset prefetcher (`cli/src/main.rs:4944`) warms cache for
   *next* load — can't help the first visit.

---

## 3 · Architectural limits (ceilings, not bugs)

These aren't things we can fix with a two-line patch. They're physics
of what we built.

| Limit | User-visible consequence | Why it's a ceiling |
|---|---|---|
| boringtun userspace crypto on macOS | ~4 KB/s throughput; 3 min cold load for 689 KB bundle | Kernel WG needs a signed System Extension |
| All traffic through one tunnel | Static assets starve inference traffic | Data plane + control plane unseparated |
| No region awareness | EU/APAC users pay trans-Atlantic on top of throughput ceiling | No geo-routing at allocation time |
| Single tunnel per user | Browser connection pooling can't parallelize across pods | One utun; all pods share it |
| Ephemeral `/tmp` cache | Cold-load roulette every few days | Historical choice — zero setup |
| No observability below TCP probe | "UI slow" has no drill-down (rekey? throughput? pod CPU?) | Boringtun exposes no IPC; tunnel is a separate root process |
| Forwarder is HTTP/1.1 | Per-request TCP handshake; no H2 multiplexing | `copy_bidirectional` on each request |
| 689 KB JS bundle is OpenClaw-normal | Floor on cold-load time | We don't own OpenClaw's build |

---

## 4 · What is NOT wrong (so we don't chase ghosts)

- Tunnel is alive (24 ms ping, no packet loss)
- Forwarder is not the bottleneck (~500 μs localhost overhead)
- Auth / token injection is fast (1–3 ms on cached paths)
- LLM gateway response latency is fine (127 ms warm)
- Cache hit path is fast (disk speed)
- Daemon / tray are no longer drifting (v0.5.1-alpha fix)

If the next symptom looks like any of these, it's probably a
different bug.

---

## 5 · Production-readiness gap inventory

What "production ready" demands we don't currently have, ordered by
leverage:

1. **Data plane / control plane split** — OpenClaw static assets via
   signed-URL CDN; tunnel reserved for API/inference. Removes ~90% of
   user-visible pain in one architectural move.

2. **Persistent asset cache** (`~/Library/Caches/tytus/ui-<pod>/`) plus
   a pre-seed step during `tytus connect`. Low effort, eliminates
   the "why is it slow *today*" class of complaints.

3. **Kernel WG on macOS** — System Extension. The only way to *actually
   remove* the ceiling rather than route around it. High cost:
   Apple Developer account, notarization workflow, user-facing
   "Allow" dialog, ongoing signing chores.

4. **Regional droplet allocation** — geo-route at pod-request time.
   High leverage for EU/APAC users, no effect for US.

5. **Per-hop telemetry** (`/tmp/tytus/events.jsonl`) so "UI slow for
   user X" is answerable from logs. Already scoped as deferred
   Phase 4 in `docs/SPRINT-2026-04-19-negotiated.md`.

6. **HTTP/2 on forwarder + upstream** — lower priority because the
   tunnel throughput ceiling masks H1's cost. Becomes relevant after
   we fix #1 or #3.

7. **Production SLO** — "production ready" has no target. Need:
   p50 first-load < Ns, p95 inference TTFT < Ms, failure budget of %
   per pod per month. Without this we're measuring against vibes.

---

## 6 · Proposed sequence (for discussion — not decided)

**Quick win phase (1–2 weeks each):**
- #2 persistent cache — ships without design debate, helps every user
- #5 per-hop telemetry — unlocks data-driven decisions for the rest

**Design-required phase (needs architecture sign-off first):**
- #1 data/control plane split — the biggest win, but requires a
  signed-asset model, CDN choice, and cache invalidation design
- #4 regional droplets — needs Scalesys-level changes

**Large, Apple-dependent:**
- #3 Kernel WG on macOS — last, because #1 makes it less urgent

**Ongoing, always:**
- #7 define + monitor an SLO — should start as soon as #5 lands

---

## 7 · Open questions for design sessions

1. **CDN choice for static assets** — Cloudflare R2? S3 + CloudFront?
   Bunny? What signing model keeps assets per-pod-isolated?
2. **Asset versioning** — pin OpenClaw version in Scalesys so the hash
   is stable per pod-generation? Or per-user?
3. **Fallback when CDN is unreachable** — does the tunnel serve as
   backup? Or do we hard-fail and surface an error?
4. **Kernel WG — ship it ourselves or require user install?**
   (WireGuardKit vs. one-time System Extension approval)
5. **Regional allocation** — single user can span regions? Or pinned
   to region-at-first-login?
6. **SLO targets** — what numbers? p50 first-paint < 5 s? < 2 s?
   How does that change pricing tiers?
7. **Does "production ready" mean the same on all platforms?** macOS
   has the boringtun problem; Linux users with kernel WG are already
   fast. Do we separate their product experiences?

These are design questions, not implementation questions. None should
be answered unilaterally by an engineer reaching for their keyboard.

---

## 8 · Context links

- `tunnel/src/wireguard.rs` — boringtun integration
- `cli/src/main.rs:4425–4428` — the "known slow" comment
- `cli/src/main.rs:4944` — the cache prefetcher
- `docs/SPRINT-2026-04-19-negotiated.md` — Phase 4 deferred telemetry
- `docs/SPRINT-2026-04-19.md` — original ship plan for pod UI
- `docs/DESIGN-TYTUS-LOPE-TEAMMATES.md` §14 — implementation reality
  notes from the previous sprint (useful for "why this is the shape it
  is" questions)

---

## 9 · How this audit was produced

Method + tools used, so future audits can reproduce:

- Live measurements with `curl -w` (connect / ttfb / total / speed)
  against the forwarder on `127.0.0.1:18702` and directly to
  `10.42.42.1:18080` over WG
- Cache miss simulated by `rm` of the cached file before `curl`
- Cache inspection: `ls -lSh /tmp/tytus/ui-02-cache/`
- Code investigation via subagent (Explore): read forwarder +
  boringtun + OpenClaw container bootstrap; grep for known-slow
  comments and deferred-telemetry scope
- Cross-referenced findings against `cli/src/main.rs:4425` comment
  block — confirms the 689 KB / 3 KB/s figures are already documented
  in-source as known limits

Cost: ~20 min wall time. Cheap relative to the clarity it buys.

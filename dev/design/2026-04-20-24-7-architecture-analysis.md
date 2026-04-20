# Tytus 24/7 Architecture — Deep-Dive Analysis

**Date:** 2026-04-20
**Status:** Analysis only. No decision made. No implementation.
**Trigger:** User question — "With current architecture can the user run OpenClaw
24/7 and chat/work via browser *and* Telegram, without restart/maintenance?
It should be fast. We have time to think about another architecture."

This is the design counterpart to
[`audits/2026-04-20-performance-audit.md`](../audits/2026-04-20-performance-audit.md).
The audit said *what's slow*. This doc says *can the system even do what we want it to*.

**TL;DR:** No, not as it stands. The current architecture has three
structural properties that are fundamentally incompatible with "always-on
pod accessible from anywhere":

1. **The user's laptop is the tunnel origin.** Close the laptop → pod
   goes dark. There's no way around this without replacing the tunnel
   model.
2. **All pod access goes through a 4 KB/s macOS userspace WireGuard
   tunnel.** Even when the laptop is open, the pipe is too narrow for
   the 689 KB OpenClaw bundle + ongoing LLM traffic.
3. **Telegram doesn't exist as an input path.** We have pod → Harvey
   (one-way notifications). We don't have user → pod except through
   the browser UI that requires the laptop's tunnel.

These aren't bugs. They're the shape of the system. Six failure modes
require manual intervention today; two of them can't be fixed without
redesigning the data-plane. The question is which alternative shape
buys us the most future without blowing up what we already have.

## 0 · Framing corrections (added 2026-04-20 after user feedback)

This doc was drafted twice. Two corrections from the user changed the
shape of the problem.

### 0.1 · "Pod invisible to internet" was wrong framing

An earlier draft framed "pod invisible to the internet" as a constraint.
That was wrong. The real constraint is **"only the owner can reach their
pod"** — which is an **auth** property, not a network-layer property.

With this reframe:

- **Pod having a public endpoint is fine**, as long as the endpoint
  requires owner auth. Hiding the pod behind a tunnel is one way to
  implement owner-only, but it's not the only way — and it's the one
  that's biting us.
- **Option A below is architecturally dead.** It can never deliver
  autonomous agent because the laptop-as-broker model prevents the
  agent from having any voice when the laptop is closed.

### 0.2 · Telegram is one channel, not THE channel

A subsequent draft centred on "pod has a Telegram bot" as the design
anchor. Also wrong. Telegram is just one example. The agent should be
able to talk to the owner via whichever chat app(s) the owner uses —
Telegram, WhatsApp, Signal, iMessage, Slack, Discord, email, whatever.

The right primitive is a **channel-agnostic messaging layer**, not a
specific chat integration. Channels plug in as adapters; users pick
which ones to enable at install time.

### 0.3 · What the owner actually wants (final)

- **Owner-only access** — auth-gated, works from any device (phone,
  browser, another laptop, chat app).
- **Autonomous agent** — runs on the pod 24/7 without the owner's laptop.
  Does work while owner sleeps.
- **Bi-directional communication through the owner's preferred chat
  channel(s)** — agent sends: updates, completions, questions. Agent
  receives: instructions, replies.
- **Not pinned to one chat app** — Telegram, Signal, WhatsApp, email,
  etc. should all be plug-in options.
- **Fast** (seconds not minutes).

### 0.4 · The new architectural primitive: TML

Based on 0.2, the architecture needs a **Tytus Messaging Layer** — a
broker that sits between the owner's chat apps and their pod.

```
                ┌─────────── chat apps (owner's choice) ──────────┐
                │ Telegram / Signal / WhatsApp / Email / Slack /   │
                │ Discord / iMessage / ...                          │
                └─────────────────────┬────────────────────────────┘
                                      │ (per-adapter)
                                      ▼
                   ┌───────────────────────────────────┐
                   │  Tytus Messaging Layer (TML)       │
                   │  - channel adapters (plugins)       │
                   │  - identity linking (owner ↔ chan)  │
                   │  - routing (chan ↔ pod)             │
                   └─────────────────────┬──────────────┘
                                         │ WebSocket (pod-initiated)
                                         ▼
                   ┌───────────────────────────────────┐
                   │  Pod (OpenClaw / Hermes / …)       │
                   │  - TML client SDK                   │
                   │  - agent.notify(msg)                │
                   │  - on_message(owner → agent)         │
                   └───────────────────────────────────┘
```

- Each **channel adapter** handles the specifics of one chat app
  (webhook, bot API, OAuth, etc.)
- **Identity linking** proves "this Telegram account belongs to this
  Traylinx user" so routing is unambiguous
- **Pod side** sees a single clean API: `notify()` / `on_message()`.
  It has no idea whether the owner is on Signal or email.

### 0.5 · What this collapses from §6 below

- **Option A** — dead (can't deliver autonomy).
- **Option B (reverse tunnel)** — becomes the default shape. TML *is*
  the relay. Pod → TML is the single outbound connection.
- **Option C (public pod endpoint)** — still viable, but each pod now
  has to host its own TML. More operational overhead; harder multi-
  channel support.
- **Option D (Telegram-first)** — not a separate option. Becomes "the
  first channel adapter TML ships."
- **Option E** — collapses into "TML with multiple adapters + an
  optional direct browser surface."

### 0.6 · MVP changes

**Before:** build a Telegram bot on the pod.

**Now:** build TML with two starter adapters (email + Telegram — easy
APIs), the channel-adapter plugin contract, and a pod-side SDK. Adding
Signal / WhatsApp / Slack / Discord becomes ~hundreds of lines each,
not architectural work.

The rest of this doc proceeds on this corrected framing. §6 option
descriptions below still have value as history but should be read
through this lens.

---

## 1 · What the user actually wants

Distilling from the ask (with 2026-04-20 reframe):

| Requirement | Implication |
|---|---|
| **OpenClaw runs 24/7** | Pod container uptime ≥ 99% (already ~OK); agent runs autonomously without laptop |
| **Owner-only access** | Auth layer gates all paths to the pod; not a network-hiding property |
| **Reachable from anywhere** | Browser, phone, Telegram, other computers — not just one specific laptop |
| **Agent is autonomous** | Does work while owner sleeps; can initiate outbound (Telegram to owner) |
| **Chat via Telegram — first-class** | Owner can Telegram the agent; agent can Telegram back |
| **No restart/maintenance** | Launch once → forget |
| **Fast** | First-paint: seconds not minutes. Inference TTFT: <2 s |
| **We have time to rethink** | Architectural changes on the table |

The single most important implication: **the owner should not need to
keep their laptop open for the agent to exist or communicate.** This is
the requirement that kills any "laptop-as-broker" architecture.

---

## 2 · Current architecture — operability scorecard

What breaks without user intervention on a real day of use:

| Failure mode | Auto-recovers? | Root cause | Fixable in current shape? |
|---|---|---|---|
| Laptop sleep/wake | ❌ — tunnel dies, manual `tytus connect` | launchd oneshot plist kills tunnel daemon on wake; no supervisor | Partially (plist `AbandonProcessGroup` or supervisor model, deferred in SPRINT-2026-04-19 item 4) |
| Wi-Fi network switch | ❌ — tunnel silently stales | No network-change handler; boringtun session decoheres | Partially (TCP-probe watchdog exists, but Noise-protocol state is opaque) |
| Keychain ACL expiry (post-rebuild) | ❌ — 3s timeout, silent "not logged in" | Unsigned binary invalidates ACL | Yes via code signing + notarization |
| Daemon crash | ✅ launchd `KeepAlive=true` respawns | — | Already fine |
| Tray crash | ❌ no restart plist | — | Yes (add plist) |
| Pod container crash | ✅ Docker `restart_policy=unless-stopped` | — | Already fine |
| Droplet reboot | ❌ user must `tytus disconnect && tytus connect` | Tunnel endpoint IP may change; no auto-reallocation | ❌ architectural — would need pod-side client |
| Tunnel rekey idle bug (~20 min) | Partially (watchdog fires, may fail again) | Boringtun `update_timers()` bug (external crate) | ❌ architectural — not our code |
| Access token expiry | ✅ daemon refreshes every 5 min | — | Already fine |
| Pod-side token expiry (for scheduled tasks) | ❌ pod has static token, no refresh path | Pod can't refresh Sentinel tokens on its own | ❌ architectural — auth model issue |

**Count:** 6 failure modes require user intervention today. 2 of those
can't be fixed incrementally. **The single biggest blocker is sleep/wake**
— every time the user closes the laptop lid, the tunnel dies and doesn't
come back on its own.

---

## 3 · The laptop-as-broker problem

The current architecture makes the user's laptop the **mandatory intermediary**
for every pod interaction:

```
                    ┌──────────────────────────┐
                    │  User's laptop (required) │
                    │  ┌────────────────────┐   │
 Browser ──────────►│  │ tytus ui forwarder │   │
 Telegram ───✗─────►│  └─────────┬──────────┘   │
 Cron agents ──────►│  ┌─────────▼──────────┐   │
                    │  │  boringtun tunnel   │   │
                    │  └─────────┬──────────┘   │
                    └────────────┼──────────────┘
                                 │ WG over UDP
                                 ▼
                         ┌─────────────────┐
                         │  Droplet (Strato)│
                         │  ┌────────────┐  │
                         │  │ WG sidecar │  │
                         │  └──────┬─────┘  │
                         │  ┌──────▼─────┐  │
                         │  │ OpenClaw   │  │
                         │  │ container  │  │
                         │  └────────────┘  │
                         └─────────────────┘
```

Things that require the laptop to be open and connected:

- Any browser access to OpenClaw (browser ↔ localhost forwarder ↔ tunnel ↔ pod)
- Any Python SDK `ask()` call (same path, different caller)
- Lope validator calls (same path, subprocess wrapper)
- Token refresh (daemon runs on laptop)
- Receiving notifications from the pod (bridge_daemon polls pod outbox)

Things the pod can do while the laptop is closed:

- Run the container (Docker restarts it on crash)
- Respond to healthz pings from the sidecar
- ...that's it

**The pod can't:**
- Accept a Telegram message (no webhook handler; no Telegram integration exists)
- Refresh its own auth token (no Sentinel credentials on pod)
- Notify the user (bridge runs on laptop)
- Continue a long-running task that needs the LLM gateway (token stale)

This is the core of the question. **If the user wants "always-on pod",
the current architecture cannot deliver it.**

---

## 4 · The Telegram gap

**Short version: there is no user → pod path via Telegram today.**

What exists:
- `tytus_sdk/bridge_daemon.py` — HTTP listener on laptop's `127.0.0.1:18099`,
  polls pod's `/app/workspace/.harvey-outbox.jsonl` every 10 s, writes to
  Harvey's Brain journal. **Pod → Harvey only. One-way.**
- `harvey_telegram_send` MCP tool — Harvey → Telegram outbound. Unrelated
  to Tytus.

What's missing:
- Telegram bot that accepts inbound webhooks on a public endpoint
- Pod-side inbox poller (reverse of bridge_daemon's outbox poller)
- Session/user mapping (Telegram chat ID → pod session)
- Auth bridge (Telegram user → Traylinx user → pod allocation)
- Message routing (inbound Telegram → which pod, which agent session?)

**Size of the gap:** moderate. ~3 new services. Design questions
unresolved (who hosts the Telegram bot? how does Telegram auth tie to
Traylinx auth? how do we prevent Telegram-driven pod abuse?).

---

## 5 · Three architectural dead-ends

Things that *cannot* be fixed by incremental work on the current shape.
Any of these becoming a requirement forces a design change.

### DE-1: macOS userspace WireGuard ceiling

- ~4 KB/s sustained on an M-series MacBook
- Root cause: synchronous Noise protocol + utun userspace API + boringtun
  single-Tunn-Mutex design
- Kernel WireGuard doesn't exist on macOS without a signed System
  Extension ($99/yr Apple Developer + notarization + user approval dialog)
- **Consequence:** Any architecture where large assets travel through the
  tunnel is capped by this

### DE-2: Laptop-as-broker model

- Tunnel origin is the user's laptop; pod is a passive UDP peer
- Pod cannot initiate outbound connections (no Sentinel creds; by design
  — security isolation)
- Tunnel dies on laptop sleep; there's no alternate path for user →
  pod communication
- **Consequence:** Any requirement for "pod responds to external events
  while laptop is asleep" is impossible without redesigning the
  communication model

### DE-3: External boringtun dead-session bug

- After ~20 min idle + certain network conditions, boringtun's state
  machine desyncs
- Watchdog can detect and force rekey, but success is not guaranteed
- Boringtun is a third-party crate; patching upstream is months of lead
  time
- **Consequence:** Even with a supervisor, long-running sessions over
  flaky networks will occasionally need hard reconnect

---

## 6 · Candidate architectures

Five shapes of "what could Tytus be instead." Each is a full system
description, not a partial fix. I've included a brutal honest tradeoff
section for each — there are no free lunches here.

### Option A: Keep current + layer fixes

**Shape:** Same as today. Add a supervisor to keep the tunnel alive, ship
Telegram as a separate always-on broker (running on the laptop *or* a
dedicated tiny VM), ship code signing to fix keychain ACL, sign DMG.

**What becomes possible:**
- Laptop-closed → pod stops being reachable (no change)
- Laptop-open → tunnel survives sleep/wake/Wi-Fi switch
- Telegram works only when laptop is on

**What doesn't get better:**
- 4 KB/s tunnel ceiling remains. 3-minute cold loads remain.
- Still laptop-dependent; still no real 24/7
- Droplet reboot still requires manual reconnect

**Cost:** Weeks, mostly UX polish (supervisor refactor + code signing +
Telegram bot development). Zero new infra.

**Honest assessment:** This is "paper over the problems." It doesn't
make Tytus production-ready; it just shifts which day the user rage-quits.

---

### Option B: Reverse tunnel — pod initiates outbound

**Shape:** Pod runs a WireGuard client (or a Tailscale node, or an
outbound-only relay agent). Pod connects *to* a Traylinx-operated
rendezvous server. User connects to the rendezvous server (not directly
to the pod). Tunnel origin flips: pod → relay → user, instead of user → pod.

```
 User browser ──┐          ┌──────────┐          ┌─────────┐
 Telegram ──────┼──HTTPS──▶│ Relay    │◀── WG ──│ Pod (on Strato)│
 SDK/Cron ──────┘   (TLS)   │ (edge)   │  tunnel  │  + OpenClaw   │
                             └──────────┘          └──────────────┘
                             public HTTPS                ▲
                              signed per-user           │
                              session token         always-connected
```

**What becomes possible:**
- Laptop-closed → pod still reachable via relay (Telegram, cron, mobile)
- User on phone → pod → chat works
- No laptop required for 24/7 operation
- Kernel WG *on the pod side* (Linux) — pod → relay connection is fast
- Relay can terminate TLS, serve OpenClaw static assets from CDN cache,
  only API traffic traverses the slow leg (if any)

**What we pay:**
- New infra: edge relay (K8s? single-region to start? regional later?)
- New auth model: relay validates user sessions, pod trusts relay
- Users now trust the relay with their traffic (can be mitigated with E2E
  encryption inside the relay tunnel)
- Latency: user → relay → pod is one extra hop (but the relay is on the
  public internet, so probably faster than user → pod through boringtun)

**What doesn't get better:**
- If the relay is down, everyone is down (single point of failure until
  multi-region)
- The macOS boringtun problem goes away *only if we stop using it* —
  which this architecture does, since traffic is TLS to the relay, not WG
  from the laptop

**Honest assessment:** This is the most architecturally correct answer.
It turns Tytus into a proper distributed system with a sensible data
path. It's also the most expensive to build. **6–12 weeks of focused
work to ship a v1 relay.**

---

### Option C: Stable external surface — OpenClaw on the public web

**Shape:** Each pod gets a public subdomain (`userid-02.pod.tytus.io`).
TLS-terminated at Cloudflare / a tiny nginx. Routes to the pod's
OpenClaw port via a lightweight authenticated reverse proxy. Browser
and Telegram both talk to the subdomain over public HTTPS.

```
 Browser ──┐
 Telegram ─┼──HTTPS──▶ edge (CF/Fastly) ──▶ pod's public endpoint ──▶ OpenClaw
 Mobile ───┘                                  (auth'd reverse proxy)
```

**What becomes possible:**
- Browser first-paint = CDN speed (tens of ms, not minutes)
- Access from phone / work laptop / tablet with no tunnel install
- Telegram → pod is just HTTPS with a webhook
- Works when the user's laptop is off
- WireGuard optionally becomes for-admin-only (tytus-cli still uses it
  for SSH / debug); normal chat doesn't need it

**What we pay:**
- Every pod becomes a public attack surface — needs real auth, rate
  limits, anti-abuse
- DNS / TLS / subdomain management per pod — automation required
- Security review: pod-side auth gateway must be bulletproof
  (currently OpenClaw trusts its WG-peer; we'd need to layer per-user
  auth)
- Lose the "invisible to the internet" isolation property (was a selling
  point)

**What doesn't get better:**
- Inference LLM latency (still depends on droplet region)
- Pod-side token refresh (still a separate problem; needed here too for
  Telegram-driven flows while laptop is off)

**Honest assessment:** Fastest perceived performance of any option.
Highest security rewrite. If we can ship the hardened gateway in front of
OpenClaw, this is a huge UX unlock. **4–8 weeks, most of it security
work + DNS/TLS automation.**

---

### Option D: Telegram-first, no browser UI

**Shape:** Stop investing in the OpenClaw web UI delivery. Telegram (or
Slack, or iMessage) becomes the primary chat surface. Pod runs a bot
worker. Browser UI is for admin-only, accepted as "sometimes slow."

```
 User Telegram app ──▶ Telegram BotAPI ──▶ Harvey-Bot (relay) ──▶ pod's bot worker
                                         (polling or webhook)
```

**What becomes possible:**
- UX matches where users already are (phone, desktop Telegram)
- No tunnel needed for 99% of daily interactions
- Pod works 24/7 via Telegram even when laptop is off
- Mobile-first (phone is always on)

**What we pay:**
- Give up the rich browser UI experience (workspace diffing, file tree,
  etc.)
- Telegram has message size + rate limits that constrain what an agent
  can show
- Building the bot's UX surface is non-trivial (inline keyboards, long
  messages, file uploads)

**What doesn't get better:**
- The tunnel still exists for admin + file transfer; still slow for
  those; still laptop-dependent

**Honest assessment:** This is the "accept the architecture for what it's
good at" path. Pod agents are great at chat. The browser UI is the wrong
primary surface. **4–6 weeks to ship a solid Telegram bot; shipping the
bot doesn't require changing anything else.**

---

### Option E: Hybrid — B + D together, deprecate current for user-facing

**Shape:** Reverse tunnel (Option B) for the data plane. Telegram
(Option D) as the primary UX. Browser UI kept for power users but
served through the relay (not directly through the laptop tunnel).
The laptop's tunnel becomes an admin/debug tool, not a user dependency.

**What becomes possible:**
- Everything in B + everything in D
- Users pick their surface (phone chat / full web UI / CLI)
- Pod is always reachable
- Laptop being open is a preference, not a requirement

**What we pay:**
- Everything in B + everything in D. This is the biggest investment.
- Long transition period where multiple UX surfaces must be kept
  consistent
- Hardest to get right on the first try

**Honest assessment:** This is the right endgame, assuming we commit to
rebuilding. **3–6 months**, phased.

---

## 7 · Decision matrix

|  | A: patch | B: reverse tunnel | C: public surface | D: Telegram-first | E: B+D |
|---|---|---|---|---|---|
| Fixes 4 KB/s cold-load | ❌ | ✅ | ✅ | ✅ (side-steps) | ✅ |
| Works when laptop closed | ❌ | ✅ | ✅ | ✅ | ✅ |
| Mobile / phone access | ❌ | ✅ | ✅ | ✅ | ✅ |
| Telegram as chat | 🟡 (only if laptop on) | ✅ | ✅ | ✅ (primary) | ✅ |
| Preserves "pod invisible to internet" | ✅ | ✅ (relay mediates) | ❌ | ✅ (Telegram mediates) | ✅ |
| New attack surface | none | relay | pod public gateway | Telegram bot | relay + bot |
| Time to ship v1 | 2–4 wk | 6–12 wk | 4–8 wk | 4–6 wk | 3–6 mo |
| New infra dependency | none | relay cluster | CDN + DNS automation | Telegram bot host | relay + bot |
| Ongoing ops cost | low | medium | medium | low | medium-high |
| User trust boundary | unchanged | Traylinx relay | Traylinx edge | Telegram Corp | all three |
| Degrades gracefully when component N fails | N/A | user sees "relay down" | user sees "pod offline" | user sees bot offline | multiple fallbacks |

No row is unambiguously best. This is genuinely a strategic choice
about what kind of product Tytus becomes.

---

## 8 · Underlying tensions (the hard stuff)

These are the things you can't paper over. They drive the choice:

### Trust model

The current design leans on "the pod is *your* pod, invisible to the
internet." This is a *security* story ("nobody can DDoS your pod because
they can't reach it") but it's also a *reliability* story ("the pod only
works when you're there to tunnel in"). The two are the same property
viewed from different angles.

Any architecture that makes the pod reachable without the user's laptop
softens that story. It's a feature trade: **privacy vs. always-on**.
We have to pick.

### Who pays for the relay / public surface

Options B, C, E all require Traylinx to operate infrastructure on the
critical path of every user's pod access. This is a real operating cost
(VMs, bandwidth, on-call). Today we have zero critical-path infra
between the user and their pod except the droplet itself. That's about
to change.

If we're moving to a pricing model that can support ongoing per-user
infra ($39/$79/$149), fine. If we're not, keep the laptop-as-broker
model and live with its limits.

### What is OpenClaw, product-wise

Current framing: "OpenClaw web UI + isolated pod." If the web UI is
the selling point, Options B/C are the right answers (make the web UI
fast). If the chat is the selling point, Option D is right (Telegram
where users are). If both equally: Option E.

**We should answer this question before picking an architecture.**

### Single user vs multi-device

Today: one user, one laptop, one pod. The tunnel lives on the laptop.
If the user has a phone and a tablet, neither of them reach the pod.

Future: one user, many devices, one pod. Every device hits the same
pod via a shared surface (relay or public subdomain).

Multi-device support is table stakes for any "production-ready" SaaS.
Options A and the current architecture don't deliver it at all.

---

## 9 · What's not answered here

These are questions that need you, not more investigation:

1. **Is the pod security story a hard floor, or negotiable?** (Drives
   A vs B/C/E.)
2. **Is Telegram a primary UX or a notification sink?** (Drives D vs
   others.)
3. **Can Traylinx operate a critical-path service (the relay) for
   every user?** (Drives whether B/E is even possible.)
4. **Price point for v2** — does the pricing model support per-user
   relay operating cost?
5. **Multi-device support deadline** — if Q3 2026, that eliminates
   Option A. If unspecified, Option A stays viable.
6. **Launch region** — EU/US/APAC. Drives whether we need regional
   droplets + regional relays from day one.

These are decisions for the user, not for a design doc to resolve.

---

## 10 · What we should NOT do right now

- Start implementing any of these. The decision is upstream of code.
- Optimize boringtun further. The 4 KB/s ceiling is real; squeezing 5
  KB/s out of it doesn't change the answer.
- Ship Phase-2 supervisor as if it's the answer. It's Option A.
  It *might* be the right interim step, but it's not the production
  answer.
- Build the Telegram bot in isolation. Whatever bot we build has to
  fit whichever architecture we pick (B vs D vs E imply different bot
  designs).

---

## 11 · Suggested next step (revised 2026-04-20 after both reframes)

With Option A dead and "Telegram" generalised to "any chat channel",
the real path forward has two parallel tracks that can be pursued
independently but land in the same shape: **Tytus Messaging Layer (TML)
+ pod-side client SDK**.

### Phase 0 — specify TML (1-2 weeks, mostly design)

- Define the **channel-adapter plugin contract**:
  `send(owner, message, attachments)` / `receive → event`.
- Define the **identity linking flow** for a first pair of channels
  (recommend email + Telegram — easiest APIs, broadest reach).
- Define the **pod-to-TML protocol** (WebSocket, message shape, auth).
- Define the **multi-channel fan-out model** — if owner has 3 channels
  linked, does `notify()` send to all, primary, or owner-configurable?

### Phase 1 — build TML v1 (3-4 weeks)

- TML core: user registry, channel registry, routing table.
- Adapter: **email** (IMAP/SMTP, ~200 LOC, universal baseline).
- Adapter: **Telegram** (bot API, ~300 LOC, immediate obvious value).
- Pod-side SDK (`tytus_sdk/messaging/`) — Python + Rust, one clean API.
- Identity linking UX — how does a user prove a Telegram account is
  theirs? Magic-link email? Deep-link from web?

### Phase 2 — add adapters as users ask (ongoing)

- **Slack** — bot token + events API (~2 weeks)
- **Discord** — bot + intents (~2 weeks)
- **WhatsApp** — Meta Business API, requires business verification (~4 weeks)
- **Signal** — signald or libsignal integration (~4 weeks, harder)
- **iMessage** — effectively locked to Apple; very hard (probably skip)

### What this UNBLOCKS on day 1

Once TML + email adapter are live (Phase 1a, ~2 weeks), the user's
agent can:

- Send status updates / completions / questions to the owner via email
  while the laptop is off
- Receive replies and route them to the agent session
- Run scheduled tasks autonomously and report results

This alone solves ~60% of the "not production ready" gap. The remaining
40% is the browser UI performance, which is a separate track (Options
B/C for data plane — can happen in parallel with Phase 2).

### The only decision blocking Phase 0

**Does Traylinx commit to operating TML as infrastructure on the
critical path of every pod's messaging?**

If yes — we can write the Phase 0 spec starting today.
If no — we stay laptop-bound forever; no amount of engineering fixes it.

Everything else (which adapters first, where TML is hosted, multi-region,
pricing impact) is downstream of that single commitment.

## 12 · Cost model (added 2026-04-20 after user question)

The "commitment" in §11 sounded vague. Here are concrete numbers so
the tradeoff is obvious.

### 12.1 · TML infrastructure cost

| Component | Cost | Notes |
|---|---|---|
| Small VM (Strato/Hetzner, 2-4 GB RAM) | €5-10/mo | Hosts the broker. Good for hundreds of users. |
| Postgres DB | €0 on same VM, €15/mo managed | User/channel/identity registry. MB-scale data. |
| Subdomain + TLS | ~€10/yr + free | `tml.tytus.traylinx.com` as the public webhook target. Let's Encrypt for TLS. |
| Monitoring (optional) | €0-10/mo | Can start with logs; add Uptime Kuma / Grafana when scaled. |

**Baseline: €5–15/mo. Scales slowly with users.**

### 12.2 · Per-channel costs

Only the channels we actually enable cost anything.

| Channel | Cost | Complexity |
|---|---|---|
| **Telegram** | **€0** (bot API is free) | Easy — MVP candidate |
| **Email** | €10-30/mo (Postmark/SendGrid) + IMAP | Easy — MVP candidate, universal reach |
| Slack | €0 | Medium — OAuth app registration |
| Discord | €0 | Medium — bot + intents |
| Signal | €0 (self-hosted signald) | Hard — needs operating signald |
| WhatsApp Business | ~€0.005-0.08/conversation + Meta verification | Hard — business verification process |
| SMS (via Twilio if ever) | ~€0.007/SMS + €1/mo/number | Easy but metered |

**MVP (email + Telegram): add ~€10-30/mo on top of TML baseline.**

### 12.3 · Scaling cost

| User count | TML infra | Email (if enabled) | Total est. |
|---|---|---|---|
| 10–50 (alpha) | €10/mo | €10/mo | **€20/mo** |
| 100-500 | €15-25/mo | €20/mo | **€35-45/mo** |
| 1,000-5,000 | €50-150/mo | €50/mo | **€100-200/mo** |
| 10,000+ | Needs a different sizing conversation | | |

For comparison: a single pod droplet is €1–20/mo depending on provider.
TML at alpha scale is roughly the cost of **two to four pods**.

### 12.4 · Why the "no central infra" alternative is more expensive

If we refuse TML and put everything pod-side:

- Every pod needs its own public endpoint → DNS automation in Scalesys,
  TLS cert per pod, per-pod reverse proxy. Engineering cost: weeks.
  Ongoing cost: nonzero (cert renewal, DNS records).
- Every pod holds its own bot tokens → N pods × M channels secret
  sprawl. User rotates a pod → loses all chat links. Engineering cost:
  per-channel credential migration on pod rotation.
- No cross-pod routing → if user has two pods, their phone can't send
  one Telegram message and have the right pod answer. Each pod needs
  a different bot. User has to remember which bot is which.
- No central auth for identity linking → each pod reimplements the
  "prove Telegram user = Traylinx user" flow. Inconsistent UX.

Rough comparison for 100 users, 2 channels (Telegram + email):

| Cost | Centralized TML | Decentralized per-pod |
|---|---|---|
| Infra | €35-45/mo | €0 (in pod) |
| Engineering to build | 3-4 weeks once | 3-4 weeks × refactor on each new channel |
| Operational load | Single service, single alerting | 100 pods × N channels to monitor |
| Per-pod TLS/DNS | 0 | Automation burden on Scalesys |
| User experience | 1 Telegram bot, linked once | 1 bot per pod, relink on rotation |

**Centralized is cheaper in every axis except the accounting line**
that says "Traylinx runs a service." The overall product TCO is lower
with TML.

### 12.5 · Pricing-model fit

Given the current pricing tiers (Free/€39/€79/€149, per prior project
memory), TML at €35-45/mo for 100-500 users is ~0.1% of revenue at
free-tier saturation — negligible. It pays for itself on the first
paid conversion.

If the concern is "free tier shouldn't be subsidised by paid tiers,"
TML can be:

- Enabled only for paid tiers (free tier = laptop-only, current arch)
- Or: free tier gets email-only (the cheap channel), paid tiers unlock
  Telegram/Slack/etc.

Both keep the unit economics sane.

---

## Appendix A · File references

Evidence for the claims above:

- `cli/src/main.rs:4425–4428` — "689 KB, 3 KB/s, 4 min" comment
- `tunnel/src/wireguard.rs:275–284` — dead-session watchdog + boringtun bug
- `tunnel/src/monitor.rs` — 36-line TCP-probe health model
- `cli/src/daemon.rs:500–544` — token refresh loop (laptop-side only)
- `tytus_sdk/bridge_daemon.py` — one-way pod→Harvey notifier
- `tytus_sdk/adapters/openclaw.py:66` — adapter requires localhost forwarder
- `agent-manager/app.py:344,385` — pod container lifecycle (auto-restart, static token)
- `docs/SPRINT-2026-04-19-negotiated.md` items 4, 5, 11 — deferred fixes

## Appendix B · Live measurements from 2026-04-20

From the perf audit (same day):
- Bundle size: 689 KB main JS
- Cold load: 171.5 s (4,022 B/s) sustained through tunnel
- Warm cache: 3.4 ms (disk speed)
- WG ping: 24 ms RTT
- First-request-after-idle stall: 3.8 s (rekey)
- Cache at `/tmp/tytus/ui-02-cache/` — 376 KB current, wiped by macOS
  periodic sweep

Full data: [`../audits/2026-04-20-performance-audit.md`](../audits/2026-04-20-performance-audit.md)

# Unblock OpenClaw's Existing Channels

**Date:** 2026-04-20
**Status:** Design — pre-sprint. Replaces the overbuilt "TML messaging broker" direction.
**Relationship to prior docs:**
- Supersedes (for implementation) the TML design in
  [`2026-04-20-24-7-architecture-analysis.md`](2026-04-20-24-7-architecture-analysis.md) §11–12.
- The 24/7 analysis doc remains valid for the *strategic* framing
  (owner-only, autonomous agent, channel-agnostic). This doc is the
  *tactical* answer — we don't need a new service, we need to unblock
  OpenClaw.

---

## 0 · Reset

Earlier drafts of this design proposed a "Tytus Messaging Layer" —
a new Node.js broker service that would speak to chat apps and route
messages between them and the pod. **That was overbuilt.**

OpenClaw already ships with 20+ chat-channel extensions as part of
its extension system:

```
/Users/sebastian/projects/makakoo/agents/sample_apps/openclaw/extensions/
├── telegram/       (grammy-based, needs TELEGRAM_BOT_TOKEN)
├── whatsapp/
├── signal/
├── slack/          (SLACK_BOT_TOKEN + SLACK_APP_TOKEN + SLACK_USER_TOKEN)
├── discord/        (DISCORD_BOT_TOKEN)
├── imessage/
├── line/           (LINE_CHANNEL_ACCESS_TOKEN + LINE_CHANNEL_SECRET)
├── matrix/         (MATRIX_HOMESERVER + MATRIX_USER_ID + ...)
├── msteams/        (MSTEAMS_APP_ID + MSTEAMS_APP_PASSWORD + ...)
├── feishu/ googlechat/ mattermost/ nextcloud-talk/ synology-chat/
├── tlon/ nostr/ bluebubbles/ zalo/ twitch/ qqbot/ irc/
└── (more)
```

Each extension is a self-contained plugin with its own manifest
(`openclaw.plugin.json`). They talk directly to chat APIs. OpenClaw
already is the messaging broker — it just can't currently reach
the chat APIs or receive their webhooks.

**So the real design question isn't "build a broker." It's:**

> What minimum changes to Tytus/Scalesys let OpenClaw's built-in
> channels actually work?

---

## 1 · What works, what doesn't — channel capability matrix

Classified by their connectivity model:

| Channel | Inbound style | Outbound | Needs pod public endpoint? | Works today? |
|---|---|---|---|---|
| **Telegram** | long-poll (default) or webhook | api.telegram.org | No (with long-poll) | ❌ egress blocks api.telegram.org |
| **Discord** | gateway (pod→Discord WS) or webhook | discord.com | No (gateway) | ❌ egress blocks |
| **Signal** | signald / libsignal (pod-local) | signal-server | No | ❌ egress blocks |
| **Email** | IMAP poll | SMTP | No | ❌ egress blocks |
| **Slack** | Events API (**webhook**) or Socket Mode | slack.com | No (Socket Mode) / Yes (Events API) | ❌ egress blocks |
| **WhatsApp** | webhook required | graph.facebook.com | **Yes** | ❌ egress + no public URL |
| **iMessage** | needs a Mac relay (Apple) | Apple services | Varies | ❌ complex — skip MVP |
| **Line / msteams / matrix / …** | varies | various | varies | ❌ egress blocks |

**Two things block every channel:**
1. The pod can't reach any chat API (egress filtering)
2. Channels that *require* inbound webhooks also need a public endpoint to the pod

Most channels have a webhook-less variant (long-poll, Socket Mode,
gateway, IMAP). If we pick those, gap #2 disappears for the first
batch.

---

## 2 · The three gaps

### Gap A — Pod egress policy

`services/wannolot-infrastructure/user-data.strato-eu-001.yml` currently:

```bash
iptables -A FORWARD -s ${POD_SUBNET} -d 212.227.205.146 :18080 -j ACCEPT   # SwitchAILocal only
iptables -A FORWARD -s ${POD_SUBNET} -p udp --dport 53     -j ACCEPT        # DNS
iptables -A FORWARD -s ${POD_SUBNET} -p tcp --dport 53     -j ACCEPT        # DNS
iptables -A FORWARD -j DROP   (implicit default)
```

Pod can reach SwitchAILocal + DNS. Every chat API is blocked.

### Gap B — Webhook inbound path

Pod is behind the WG sidecar with no public IP. Chat APIs that
POST webhooks (Slack Events API, WhatsApp, Meta stuff, any "App
→ your-URL" flow) can't deliver.

### Gap C — Configuration UX

User needs per-extension credentials on the pod:
- Telegram: `TELEGRAM_BOT_TOKEN`
- Slack: 3 tokens
- Discord: 1 token
- Matrix: homeserver + user + password
- etc.

Today this means: tunnel into OpenClaw UI (slow) or `tytus exec` to
write files (painful, error-prone).

---

## 3 · Gap A design options — egress

### A1: Blanket outbound HTTPS (port 443)

```bash
iptables -A FORWARD -s ${POD_SUBNET} -p tcp --dport 443 -j ACCEPT
```

**Pros:**
- One line of config. Shipable today.
- Every channel extension works instantly.
- Matches the implicit trust model: the user owns the pod, the pod
  runs their agent, the agent is allowed to call APIs on their behalf.

**Cons:**
- Pod can also reach any other HTTPS endpoint (supply-chain risk — a
  compromised package on the pod can exfiltrate).
- Not "least-privilege" by textbook.

**Mitigations:**
- Keep cross-pod FORWARD drops (still enforced).
- Keep metadata block (169.254.169.254).
- Rely on container integrity (pod image is known, user-installed code
  lives in `/app/workspace`).
- Add DNS logging at the nginx resolver (optional, later).

### A2: Per-channel IP allowlist

```bash
iptables -A FORWARD -s ${POD_SUBNET} -d api.telegram.org-IPs -j ACCEPT
# ...
```

**Problem:** iptables doesn't resolve hostnames. Telegram's IPs change.
Would need a cron job to re-resolve + re-apply rules. Fragile.

**Verdict:** not worth the ops cost.

### A3: Egress proxy (squid/mitmproxy) with hostname allowlist

- Pod sends all outbound through a proxy on the droplet
- Proxy enforces domain allowlist (api.telegram.org, discord.com, …)
- User enables a channel → Scalesys adds domain to proxy allowlist

**Pros:** real hostname-level enforcement, auditable.

**Cons:** new component, per-channel rules to maintain, proxy is an
extra failure point.

**Verdict:** good v2 if A1's trust model proves too loose. **Not
MVP.**

### Recommendation for Gap A

**A1 (blanket outbound HTTPS)** for MVP. Ship today, revisit with A3
as a hardening step after we have usage data.

---

## 4 · Gap B design options — webhook inbound

### B1: Long-poll / Socket Mode only (zero infra)

- Ship Telegram (long-poll), Discord (gateway), Signal, email (IMAP),
  Slack (Socket Mode)
- These account for a majority of the user base most people would pick
- No pod public endpoint needed at all

**Pros:** nothing new to build. Works with Gap A fix alone.

**Cons:** WhatsApp Business API and some Slack features require
webhook. Deferred until later.

### B2: Per-pod public subdomain via nginx on droplet

- Droplet already runs nginx for SwitchAILocal
- Add server block: `pod02.webhook.${DROPLET_DOMAIN}` → pod's
  port 3000 (OpenClaw default)
- Scalesys registers DNS (Traylinx-owned) when pod is allocated
- TLS via Let's Encrypt DNS-01 (no need to open port 80 on pod)

**Pros:** per-pod isolation; each pod gets its own URL.

**Cons:**
- DNS automation work in Scalesys (~1 week)
- TLS cert lifecycle (Let's Encrypt automation, ~3 days)
- Every webhook from chat apps hits the droplet directly —
  rate-limit/abuse handling lands in nginx

### B3: Shared webhook relay (small Traylinx service)

- Single public endpoint: `webhook.traylinx.com/{user_id}/{channel}`
- Relay looks up user's pod, forwards the webhook
- One DNS record, one TLS cert, centralized auth

**Pros:** zero per-pod infrastructure; any chat API can POST to one URL.

**Cons:**
- New service to build (small — just a router, ~few hundred LOC)
- New single point of failure for all webhook delivery
- Still needs auth mapping (chat app's HMAC → user_id)

### Recommendation for Gap B

**B1 for MVP** — unblocks Telegram + Discord + Signal + email + Slack
(Socket Mode). Covers most use cases users actually care about.

**B2 (per-pod subdomain) later** when users demand WhatsApp Business.
This is the point where Scalesys earns its name: it *allocates* pods
+ DNS entries as a single operation.

**B3 is a trap** — we start building TML-lite again. Don't.

---

## 5 · Gap C design options — config UX

### C1: CLI command `tytus channels add`

```bash
tytus channels add --pod 02 --type telegram --token 1234:ABCD
tytus channels add --pod 02 --type discord --token ...
tytus channels list --pod 02
tytus channels remove --pod 02 --type telegram
```

Implementation:
- Writes into `/app/workspace/.openclaw/config.user.json` via
  `tytus exec` (already existing overlay mechanism — see the
  project CLAUDE.md section on `tytus configure`).
- Restarts the OpenClaw container to pick up new env vars (via
  DAM restart API already implemented).

**Pros:**
- Uses only existing Tytus-CLI primitives (`tytus exec` + DAM restart)
- Works over the slow tunnel (small messages, not bundle-sized)
- Scriptable for users who want to configure programmatically

**Cons:**
- Still text-based; users have to know the token format per channel
- No "did this token work?" feedback beyond "container restarted"

### C2: Tray menu "Install a channel…" wizard

- Tray opens a localhost wizard page (we already have the pattern
  from `tytus-cli/tray/src/web_server.rs`)
- Page shows a list of available channels, deep-linked to each
  chat app's bot-creation flow
- User pastes token → CLI writes to pod → restart

**Pros:** much friendlier UX for non-dev users.

**Cons:** more work than C1, redundant if C1 is already the engine.

**Recommendation:** **ship C1 first**, use it as the engine under C2
later. C1 is ~1 day; C2 is ~1 week.

### C3: OpenClaw web UI itself

OpenClaw *already has* a config UI for extensions (the big bundle we
fight over the slow tunnel). Users could configure it directly there.

**Pros:** no Tytus-side work at all.

**Cons:** dependent on the unfixed tunnel-throughput problem. Until
that's addressed, users suffer a 3-minute cold load to change a
bot token.

**Verdict:** works as a fallback but don't rely on it for MVP.

### Recommendation for Gap C

**C1 as the MVP**. Implement `tytus channels add/list/remove` using
the existing overlay + DAM restart. ~1 day of CLI work.

---

## 6 · MVP sprint scope

Minimum end-to-end: "user installs OpenClaw + configures Telegram +
sends message → OpenClaw responds → user's laptop is closed and it
still works."

### Work items

1. **Gap A1 — open outbound HTTPS on pod**
   - Add `iptables -A FORWARD -s ${POD_SUBNET} -p tcp --dport 443 -j ACCEPT`
     to `user-data.strato-eu-001.yml`
   - Document in `services/wannolot-infrastructure/docs/architecture/`
     that pods now have general-HTTPS outbound
   - Estimated: **1 day** (+ test on pod 02)

2. **Gap C1 — `tytus channels add` CLI**
   - New subcommand in `cli/src/main.rs` (model after `tytus configure`)
   - Implements: `add`, `list`, `remove`, per-pod
   - Writes to `/app/workspace/.openclaw/config.user.json` via `tytus exec`
   - Restarts the OpenClaw container via DAM's `/restart-agent` (existing)
   - Estimated: **1–2 days**

3. **E2E smoke test** (`scripts/e2e-channels.sh`)
   - Install Telegram bot on pod 02 via new CLI
   - Send message to bot from Sebastian's Telegram
   - Confirm OpenClaw receives it + responds
   - Close laptop; send another message; confirm it still works
   - Estimated: **1 day**

### Total MVP

**3–4 days of focused work.** Unlocks:

- Telegram (long-poll)
- Discord (gateway)
- Signal (self-contained via signald inside pod if we bundle it — otherwise later)
- Email (IMAP poll, ping via SMTP — no external service needed)
- Slack (Socket Mode — no webhook needed)

### What we're NOT doing in MVP

- WhatsApp Business API (needs webhook path, Gap B2 later)
- Meta-family channels that require webhook delivery
- iMessage (Apple-locked, separate hard design)
- Pod public subdomain automation (defer until a user needs WhatsApp)
- The TML broker service (confirmed: not building)
- Changes to the laptop tunnel (separate workstream — the slow bundle
  doesn't affect chat flows, so it can be solved later)

---

## 7 · Follow-on sprints (in order)

**Sprint 2 (1 week):** Tray wizard (Gap C2). Non-dev user can configure
channels via menu → localhost page, not just CLI.

**Sprint 3 (1–2 weeks):** Per-pod public subdomain (Gap B2) + TLS auto.
Unlocks WhatsApp Business, Meta webhook channels, any future
webhook-required channel.

**Sprint 4 (1–2 weeks):** Egress hardening (Gap A3 — proxy with
hostname allowlist). Tighten the trust model once we have usage data
for which APIs channels actually hit.

**Sprint 5+ :** User-specific hardening (channel quotas per plan tier,
rate limits, audit log of what extensions hit which APIs).

---

## 8 · What this doesn't solve

Being explicit about what's still broken after this sprint:

- **Browser UI is still slow over the tunnel.** The 689 KB bundle
  still takes ~3 min cold. But users who primarily chat via Telegram
  won't hit this anymore — UI becomes optional.
- **Pod reboot still requires tunnel re-establishment.** Chat
  channels resume fine once the pod container comes back; laptop still
  needs to `tytus connect` if the user ever wants the browser.
- **Pod auth refresh still laptop-dependent.** If the user never opens
  their laptop for a week, the pod-side Sentinel access token
  eventually expires. Not critical for MVP — SwitchAILocal uses
  the stable per-user key which doesn't expire. But future features
  that need Sentinel on the pod will break.

These are known-remaining gaps. Each is a separate workstream.

---

## 9 · Open questions for user

1. **Egress trust model — ship A1 (blanket HTTPS), or insist on A3 (hostname proxy) from day one?** My recommendation: A1 for MVP, A3 later.

2. **Should `tytus channels add` fetch tokens from a keychain, or accept them on the CLI?** Safer: keychain-backed (matches existing refresh-token storage). Quicker: CLI arg + env var.

3. **Do we pre-enable a default set of channels at pod allocation time?** E.g., pod ships with `@openclaw/telegram` already loaded (just waiting for a bot token). Saves the user one step.

4. **When users have multiple pods, should each pod have its own bot or should bots multiplex across pods?** First answer is simpler; second is user-preferred long-term. MVP: per-pod bots, document the limit.

5. **Skip iMessage from the supported-channels list entirely, or put it as "experimental / requires a Mac mini"?** Voting for "not supported."

---

## 10 · Why this shape

The biggest reason this design is smaller than the TML one: **it respects
OpenClaw as the product it already is.** OpenClaw ships channel
integrations that took Nemotron/OpenClaw engineers years to get right.
Our job at the Tytus layer isn't to re-implement them — it's to make
sure the pod network and configuration plumbing let those integrations
do their job.

When in doubt, **ask "what does OpenClaw already do?" before proposing
new infrastructure.** That check would have caught the TML
overbuilding two iterations ago.

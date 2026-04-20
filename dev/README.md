# Tytus — Development Planning

Home for **in-flight** architectural work: audits, design docs, sprint plans
for the next evolution of Tytus. Separate from `docs/` — that folder holds
reference material + security audits + completed sprints.

## Why this folder exists

After the 2026-04-20 production-readiness audit, we identified
architectural limits that require **design-level re-thinking**, not
incremental fixes:

- Userspace WireGuard on macOS tops out at ~4 KB/s — the 689 KB
  OpenClaw bundle takes ~3 min to cold-load through the tunnel.
- All traffic (static assets + API/inference) shares the same tunnel.
- No region awareness; no CDN; no persistent asset cache.
- No per-hop observability below TCP-probe granularity.

"Tytus is absolutely not production ready" — user, 2026-04-20.

This folder is where we plan the fix.

## Layout

```
dev/
├── README.md                 ← this file, the index
├── audits/                   ← raw findings (before we decide what to do)
│   └── 2026-04-20-performance-audit.md
├── design/                   ← architectural design docs (one per question)
└── sprints/                  ← executable sprint plans (one per ship unit)
```

**Workflow:** audit surfaces a problem → design doc proposes a solution →
sprint doc breaks it into ship-able chunks → code. Each stage lives under
its own subdir so nothing gets conflated.

## Active audits

| Date | Audit | Status |
|---|---|---|
| 2026-04-20 | [Performance audit — why OpenClaw UI feels slow](audits/2026-04-20-performance-audit.md) | **Open** — blocking design work |

## Active designs

| Date | Design | Status |
|---|---|---|
| 2026-04-20 | [24/7 architecture analysis — strategic framing (owner-only, autonomous agent, channel-agnostic)](design/2026-04-20-24-7-architecture-analysis.md) | **Strategic context** — informed by the TML rabbit-hole, now mostly useful for framing |
| 2026-04-20 | [Unblock OpenClaw's existing channels (MVP)](design/2026-04-20-unblock-openclaw-channels.md) | **Ready for review** — 3–4 day sprint; replaces the TML-broker direction |

## Active sprints

_None yet._ Once a design is signed off, a sprint doc gets written here.

## Conventions

- **File naming:** `YYYY-MM-DD-short-slug.md` (audits + sprints); design
  docs can use any descriptive name.
- **Frontmatter optional** — only add when a doc is going to be consumed
  by tooling.
- **Commit message prefix:** `dev(plan):` for docs in this folder to keep
  them out of release notes.
- **Move, don't delete:** when work ships, move the sprint doc to
  `docs/SPRINT-YYYY-MM-DD-<slug>.md` so the planning trail stays grep-able.

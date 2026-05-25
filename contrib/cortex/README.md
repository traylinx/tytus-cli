# Local Cortex bundle

This directory holds the `docker-compose.yml` + `.env.example` shipped with
`tytus-cli` to bring up a private Cortex instance on the user's machine.

## Why

Production TytusOS routes every chat through `tytus.traylinx.com` → remote
Cortex on Strato. That adds latency and means memory (STM/LTM) lives on
Traylinx infrastructure. Local Cortex flips that: chat memory lives on the
user's Mac, brain-mode chats hit `127.0.0.1:8098` instead of the public
Provider.

See the full sprint plan at:
`services/tytus-os/development/sprints/2026-05-21-chat-with-pods-local-cortex-parity/`.

## What's bundled

- `docker-compose.yml` — Postgres 16 (pgvector) + Redis 7 + `tytus-cortex` API.
  - Postgres + Redis store STM, LTM (with 1024-dim embeddings), sessions, users.
  - Cortex API binds `127.0.0.1:8098` (loopback only — never on public LAN).
- `.env.example` — required secrets. The CLI auto-fills random values per user.

## How tytus-cli uses this

1. Operator runs `tytus cortex up`.
2. CLI copies `contrib/cortex/docker-compose.yml` → `<state_dir>/cortex/docker-compose.yml`.
3. CLI generates random `POSTGRES_PASSWORD`, `ENCRYPTION_KEY`,
   `INTERNAL_SERVICE_TOKEN` and writes them to `<state_dir>/cortex/.env`.
4. CLI mirrors `INTERNAL_SERVICE_TOKEN` into `state.json` so the tray daemon
   can present it on `/tytus/chat` calls.
5. CLI shells out: `docker compose -f <state_dir>/cortex/docker-compose.yml up -d`.
6. CLI polls `http://127.0.0.1:8098/health/live` until `{postgres: ok, redis: ok}`
   or times out at 90s.
7. CLI mints a per-user `ctx_*` token via `POST /v1/users` and stashes it as
   `state.json::cortex_local_token`. This token is for `/v1/*` user-scoped
   endpoints (memory search, session list) — NOT for `/tytus/chat`.

## Two-token model (the R14 trap)

| Token | Where it lives | Who uses it | What it unlocks |
|---|---|---|---|
| `INTERNAL_SERVICE_TOKEN` | `state.json` (mirror) + Cortex env | Tray daemon → Cortex | `/tytus/chat` (passthrough) |
| `ctx_*` user token | `state.json::cortex_local_token` | TytusOS apps via tray proxy | `/v1/*` user-scoped routes |

Mixing them returns 401. See risk register R14 + sprint 04-API-CONTRACTS.md §6.

## Cross-repo dependency

The bundled compose uses `image: ghcr.io/traylinx/tytus-cortex:<tag>`. The CLI currently pins `ghcr.io/traylinx/tytus-cortex:2026-05-17`.

Launch rule: before telling users that `tytus cortex up` is ready, verify the GHCR package is public from a clean machine:

```bash
docker manifest inspect ghcr.io/traylinx/tytus-cortex:2026-05-17 >/dev/null
```

If that fails with `manifest unknown` or `unauthorized`, the docs must say local Cortex is temporarily unavailable and support must fix GHCR package visibility.

For local dev override:

```bash
cd services/tytus-cortex
docker build -t ghcr.io/traylinx/tytus-cortex:2026-05-17 .
```

## Resource footprint

Approximate steady-state on macOS:

- Postgres: ~512 MB
- Redis: ~128 MB
- Cortex API: ~600 MB
- **Total: ~1.2 GB resident, ~4 GB peak during pulls.**

Disk: Postgres data grows with LTM corpus (1 KB per memory + embedding). A
user with 10k memories uses ~30 MB Postgres + indexes.

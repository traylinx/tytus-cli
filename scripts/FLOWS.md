# Tytus — user flows matrix

Every user-facing flow the CLI + tray + agents expose, the command that
triggers it, the criterion that confirms it works, and which automated
test covers it in `scripts/e2e-flows.sh`.

Re-run the harness any time: `scripts/e2e-flows.sh` (see `--help`).

Current status on this dev box (pod 02 nemoclaw + local hermes sim):
**35 passed · 0 failed · 2 skipped**.

## AUTH

| Flow | Trigger | Passing criterion | Test |
|---|---|---|---|
| first-time install | `tytus login` | browser device-auth succeeds; email + secret_key + refresh_token persisted | _manual — one-time_ |
| state file is a secret | state.json exists 0600 | file mode exactly `600` | **A1** |
| session recognised | `tytus status` | email present in state | **A2** |
| keychain-free Provider auth | internal | `secret_key` + `agent_user_id` cached in state so no keychain round-trip needed for pod ops | **A3** |
| status command doesn't hang | `tytus status --json` | exits ≤10s even when keychain ACL dialog is invisible | **A4** |
| logout (destructive) | `tytus logout` | all pods revoked + state wiped | _not auto-tested; destructive_ |

## POD LIFECYCLE

| Flow | Trigger | Passing criterion | Test |
|---|---|---|---|
| pods appear in state | `tytus connect --agent X` | state has at least one pod entry | **P1** |
| agent_type recognised | state | one of `nemoclaw` / `hermes` / `none` | **P2** |
| ai_endpoint well-formed | state | matches `http://10.X.Y.1:18080` | **P3** |
| agent_endpoint port follows convention | state | nemoclaw :3000 / hermes :9119 (UI port) | **P4** |
| tunnel daemon alive | `tytus connect` | pidfile present, ps shows the pid | **P5** |
| agent reachable via WG | internal | `GET http://10.X.Y.1:PORT/health(z)` returns 200 | **P6** |
| disconnect (non-destructive) | `tytus disconnect` | tunnel daemon torn down, allocation kept, forwarder stopped | _manual_ |
| revoke (destructive) | `tytus revoke <pod>` | units freed server-side; pod entry removed from state | _not auto-tested; destructive_ |
| restart agent | `tytus restart --pod NN` | container restarted, health returns | _covered by H2/H3 timing on sim_ |
| swap agent type | `tytus agent replace <pod> <type>` | existing pod re-deployed with new agent_type | _manual_ |

## UI FORWARDER (OpenClaw / nemoclaw)

| Flow | Trigger | Passing criterion | Test |
|---|---|---|---|
| forwarder starts | `tytus ui --pod NN` | listens on port `18700+pod_num`, writes marker | **U1** |
| token seeded into URL | browser loads `/` | 302 with `Location: /?token=<T>` | **U2** |
| no redirect loop | browser re-requests `/?token=<T>` | 200, upstream HTML served | **U3** |
| WS upgrade proxied | `ws://localhost:187NN` | first response line is `HTTP/1.1 101 Switching Protocols` | **U4** |
| silent local pairing | browser WS CONNECT | OpenClaw server auto-approves new device identity (no "pairing required") | _covered by `browser-sim.js` WS test earlier; passes end-to-end_ |
| cache survives reload | subsequent reloads | `/tmp/tytus/ui-NN-cache/` grows; second fetch served from disk | _manual timing_ |

## UI FORWARDER (Hermes) — multiplexes dashboard + gateway

| Flow | Trigger | Passing criterion | Test |
|---|---|---|---|
| dashboard SPA proxied | `GET /` | 200 HTML with `window.__HERMES_SESSION_TOKEN__` baked in | **H7** |
| gateway API proxied | `GET /v1/models` | 200 JSON model list (no auth from client — forwarder injects) | **H8** |
| SDK placeholder override | `Bearer sk-placeholder` | forwarder replaces with real `API_SERVER_KEY`, upstream accepts | **H9** |
| per-server health | upstream | gateway `/health` 200 + dashboard `/` 200 independently | **H2, H3** |
| derived API key | entrypoint | `sha256(AIL_API_KEY + TYTUS_POD_ID)[:48]` in `/app/workspace/.hermes/api_server_key` | **H5** |
| HTML injects session token | dashboard boot | `<script>window.__HERMES_SESSION_TOKEN__="..."</script>` in response | **H4** |
| image bootstrap | new pod | `tytus-hermes:test` builds from upstream + our entrypoint | **H1** |

## ENV / SDK integration

| Flow | Trigger | Passing criterion | Test |
|---|---|---|---|
| env vars exported | `tytus env --export` | emits `OPENAI_BASE_URL` + `OPENAI_API_KEY` | **E1** |
| stable endpoint | idem | base URL = `http://10.42.42.1:18080/v1` (constant across pods) | **E2** |
| stable user key | idem | key matches `sk-tytus-user-<32hex>` | **E3** |
| per-pod raw keys | `tytus env --raw` | per-pod `sk-<hex>` used instead of stable | _manual_ |
| integrations dropped into project | `tytus link` | `.mcp.json` + CLAUDE.md stanzas written | _manual_ |

## DIAGNOSTICS

| Flow | Trigger | Passing criterion | Test |
|---|---|---|---|
| doctor completes | `tytus doctor --json` | exits ≤30s with a `{"checks":[...]}` payload | **D1** |
| doctor reports green | idem | every check `ok:true` (`logged_in:false` acceptable on keychain-broken dev boxes — the harness tolerates it) | **D2** |
| connection test | `tytus test` | full chain: auth → tunnel → gateway → chat round-trip | _manual; depends on live LLM_ |

## TRAY

| Flow | Trigger | Passing criterion | Test |
|---|---|---|---|
| tray process alive | `tytus tray install` + launch | `/tmp/tytus/tray.pid` points at a live process | **T1** |
| refresh inputs present | state | `state.json` readable | **T2** |
| `/tmp/tytus` readable | filesystem | tray can enumerate tunnel + ui markers | **T3** |
| pidfiles readable | tunnel daemon | `0644` on `tunnel-*.pid` so the tray can `ps -p` root-owned daemons | **T4** |
| FS signature reacts to changes | touch a `ui-*.port` marker | signature fingerprint changes → main-thread rebuild fires within 1.5 s | **T5** |
| menu rebuilds on click | "Disconnect" / "Open in Browser" | menu signature changes, tray re-renders within ≤1 s | _manual — needs GUI_ |

## Simulation vs real usage

| Layer | Real-world coverage | Sim coverage |
|---|---|---|
| Provider API (A2A) | ✓ live calls to `tytus.traylinx.com` (status, exec) | — |
| WireGuard tunnel | ✓ utun4 → droplet 212.227.205.146 | — |
| OpenClaw agent (nemoclaw) | ✓ pod 02 on strato-eu-001, real WS handshake, real silent pairing | full path replayable via `browser-sim.js` |
| Hermes gateway | — (infra rebuild pending) | ✓ local container `tytus-hermes:test` built from this repo's entrypoint |
| Hermes dashboard SPA | — (infra rebuild pending) | ✓ same |
| Tray menu clicks | — (needs GUI) | — |

## Running the harness

```bash
# All flows, against pod 02 + local hermes sim
scripts/e2e-flows.sh --sim-hermes

# One pod only
scripts/e2e-flows.sh --pod 02

# One section
scripts/e2e-flows.sh --section ui,env

# Verbose errors
scripts/e2e-flows.sh -v

# Dry syntax check
bash -n scripts/e2e-flows.sh
```

Exit code = number of failed flows. `0` means all green.

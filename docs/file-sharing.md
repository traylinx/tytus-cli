# File sharing with Tytus pods

Move files and folders between your Mac and any running Tytus pod from
the command line, the tray menu, or any MCP-capable AI CLI — using plain
human language.

> **5-minute promise.** If your pod is connected, the first three
> sections of this doc get you pushing and pulling files. Everything
> after is reference.

## Mental model

Every pod has a writable `/app/workspace/` mount. By convention:

- **`/app/workspace/inbox/`** — you push into here. Agents running on
  the pod pick up work from this directory.
- **`/app/workspace/out/`** — agents write results here. You pull from
  it.

Outside `/app/workspace/` the pod rootfs is read-only, and pushes that
try to land anywhere else are refused before a single byte is sent.

The transport is the existing `tytus exec` pipeline — base64 chunks
over the already-established WireGuard tunnel. No new infrastructure,
no new ports, no new daemons. The same auth you use for `tytus chat`
and `tytus exec` covers file transfer.


## Web/mobile cloud shared folders

Traylinx web can create **cloud shared folders** for users who have not installed local TytusOS yet. This is the production-safe web path:

1. User creates a folder on `https://traylinx.com/admin/tytus`.
2. Browser uploads files into the private cloud folder.
3. Traylinx attaches that folder to one or more Tytus pods.
4. OpenClaw/Hermes read inputs from `/app/workspace/shared/<folder>/inbox/`.
5. Agents write outputs to `/app/workspace/shared/<folder>/out/`.
6. User downloads outputs from Traylinx web/mobile.
7. Later, local TytusOS can bind that same cloud folder to a computer path with garagetytus.

The browser does **not** background-sync arbitrary local folders. Local folder binding is a TytusOS/garagetytus feature, not a web-only feature.

Recommended pod paths for cloud shared folders:

```text
/app/workspace/shared/<folder>/
/app/workspace/shared/<folder>/brief.md
/app/workspace/shared/<folder>/inbox/
/app/workspace/shared/<folder>/out/
```

## TytusOS Files app

The TytusOS Files app is the user-friendly surface on top of these commands. It opens at `~/Tytus` and can browse:

- Tytus Home (`~/Tytus`)
- `~/Tytus/Shared` and any account-scoped shared-folder bindings
- Pod workspaces under `/app/workspace`
- Pod inbox/downloads views

Empty folders should show friendly empty states. Raw command stderr such as `tytus ls: no such path` is diagnostic text, not user copy.

## Quickstart

```bash
# Push a file to the sole connected pod's inbox:
tytus push ~/report.pdf
# → pushed ~/report.pdf → pod-02:/app/workspace/inbox/report.pdf (845320 bytes)

# Push with explicit pod + destination:
tytus push ~/slides.pdf --pod 04 --to /app/workspace/inbox/slides-final.pdf

# Push a folder (tarred + gzipped automatically):
tytus push ~/Projects/acme --pod 02
# → lands at /app/workspace/inbox/acme/…

# Pull a file back:
tytus pull /app/workspace/out/summary.md --pod 02
# → ./summary.md written locally

# See what's on a pod:
tytus ls --pod 02
# mode     size   mtime                 name
# 644    845320   2026-04-24 12:18:02   /app/workspace/inbox/report.pdf

# Delete something:
tytus rm /app/workspace/inbox/report.pdf --pod 02

# See your recent transfer history:
tytus transfers --tail 10
```

If you have exactly one pod connected, `--pod` is optional — the CLI
auto-picks. If you have several, you'll be asked to specify.

## Tray menu (macOS)

Once `Tytus.app` is installed (`tytus tray install` once per machine),
each pod gets a **Files ▸** submenu under **Pods & Agents ▸ pod-NN ▸**:

| Menu entry | What it does |
|---|---|
| Push file… | Opens a macOS file picker; the chosen file streams to `/app/workspace/inbox/` on that pod. Notification when done. |
| Push folder… | Same, but for a directory. Tarred + gzipped transparently. |
| List inbox in Terminal | Opens a Terminal window running `tytus ls --pod NN /app/workspace/inbox/` so you can see what's there. |
| Open local download folder | Opens `~/Downloads/tytus/pod-NN/` in Finder — this is where tray pulls land. |

Every push posts a macOS notification on completion. Pulls additionally
"Reveal in Finder" the received file so you can open it immediately.

> **Drag-and-drop to the menu bar icon** is deferred — the tray-icon
> crate wraps `NSStatusItem` without the `NSDraggingDestination`
> protocol, and subclassing via `objc2` is non-trivial. A follow-up
> sprint can ship real drag-to-icon. Meanwhile the file picker covers
> the same intent in one extra click.

## Skill discovery (any AI CLI)

The portable skill `plugins-core/skill-tytus-files/` registers itself
via `skill_discover` in every MCP-capable CLI (claude-code, gemini-cli,
pi, opencode, codex, qwen, vibe, cursor). That means inside any of
those CLIs the user can type:

> *"Push the meeting notes to pod 2."*

and the agent routes the call to `tytus push ~/meeting-notes.md --pod 02`.
Triggers cover English and Spanish — "manda el PDF al pod 2" resolves
the same way.

To verify:

```bash
skill_discover --match tytus-files
```

Full trigger table + decision tree live in the skill body:
`plugins-core/skill-tytus-files/SKILL.md`.

## When NOT to use `tytus push / pull`

- **You want an LLM to RAG over the file.** Route to `harvey_knowledge_ingest`
  instead. That path embeds into Qdrant and exposes the file content to
  `harvey_superbrain_query`. `tytus push` just drops raw bytes in
  `/app/workspace/inbox/` with no indexing.
- **You need browser/mobile shared folders.** Use Traylinx web cloud shared folders once the account-side registry is enabled. `tytus push/pull` is still the local CLI transport for one-off Mac/Linux ↔ pod transfers.
- **You need a shared filesystem mounted into multiple pods today.** Use the Garage/garagetytus shared-folder path where provisioned. If the account-side shared-folder registry is not enabled yet, `tytus push/pull` remains per-file between your computer and one pod at a time.
- **Your transfer is larger than 100 MB.** The CLI refuses with a
  pointer to the Garage sprint. This is deliberate — docker-exec base64
  streaming is the wrong foundation for GB-scale transfers, and we
  don't want half-hour hangs pretending to be progress. Options: split
  the file, wait for v0.7, or use an out-of-band upload path (e.g.
  pushing directly to S3 from the pod agent).

## Troubleshooting

| Error | Cause | Fix |
|---|---|---|
| `Not logged in. Run: tytus login` | No valid credentials | `tytus login` |
| `No pods. Run: tytus connect` | No pod allocated yet | `tytus connect` or `tytus setup` |
| `multiple pods connected ([02, 04]); specify --pod NN` | Ambiguous target | Add `--pod NN` |
| `path outside /app/workspace/ is not allowed` | Remote path points at RO rootfs | Target `/app/workspace/…` |
| `path contains '..' or escape segments` | Defensive — CLI refuses any `..` | Rewrite the path as absolute under `/app/workspace/` |
| `transfer exceeds 100 MB ceiling (N bytes)` | Intentional cut-off | See "When NOT to use" — point at Garage sprint |
| `remote path does not exist` | Pull target missing | `tytus ls --pod NN` to discover the actual path |
| `refusing to remove directory without --recursive` | Safety net on `rm` | Re-run with `--recursive` if truly intended |
| `chunk write failed` / `remote finalise failed` | Usually a permissions or diskspace issue on the pod | `tytus exec df -h /app/workspace` to check free space |

Every push, pull, rm event — success or failure — appends one line to
the local JSONL audit log. Tail it for post-mortem:

```bash
tytus transfers --tail 20
tytus transfers --json | jq 'select(.ok == false)'
tytus transfers --pod 02 --tail 0   # every pod-02 event ever
```

On macOS the log lives at
`~/Library/Application Support/tytus/transfers.log`. On Linux it
respects `$XDG_DATA_HOME/tytus/transfers.log` (or
`~/.local/share/tytus/transfers.log` as fallback). The log is
append-only and file-locked (`flock`), so concurrent `tytus push`
processes never interleave lines.

## Command reference

```text
tytus push <LOCAL> [--pod NN] [--to /app/workspace/DEST] [--quiet] [--json]
  Push a file or directory. Dirs are tarred+gzipped locally, streamed,
  extracted on the pod. Default --to is /app/workspace/inbox/.

tytus pull <REMOTE> [--pod NN] [--to LOCAL] [--quiet] [--json]
  Pull a file or directory. Default --to is ./ with remote basename
  preserved.

tytus ls [PATH] [--pod NN] [--json]
  List contents of a remote path under /app/workspace/. Default PATH is
  /app/workspace/inbox/. --json emits machine-readable rows.

tytus rm <REMOTE> [--pod NN] [--recursive] [--json]
  Delete a remote path. --recursive is required for directories.
  Refuses any path outside /app/workspace/.

tytus transfers [--tail N] [--pod NN] [--json]
  Show the local JSONL audit log. Default tail 20; 0 for all.
```

## Size ceiling — why exactly 100 MB?

Base64 encoding balloons payloads by ~33%. Sending via `docker exec`
means the encoded payload rides in shell argv, which has ARG_MAX limits
on every OS (~2 MB on Linux, ~256 KB on macOS — not that macOS is the
pod side, but you get the idea). The CLI chunks in 256 KB blocks to
sidestep this, but the round-trip cost is one `exec` call per chunk.
Every call has a ~100 ms baseline over the WireGuard tunnel, so a
100 MB transfer is already ~40 seconds best case — past that, users
hit "did it freeze?" territory faster than they get useful progress.

For GB-scale transfers, use the Garage/garagetytus shared-folder path where provisioned. `tytus push` keeps the 100 MB safety cap because docker-exec base64 streaming is still the wrong transport for large files.

## Progressive sync (pod → Mac, v1)

Shared-folder bindings historically moved with `rclone bisync` — a full-tree
listing on every run, which freezes on large folders over the tunnel.
Progressive sync replaces the mover per binding with an **event consumer**:
pods emit one immutable event per settled upload under the reserved
`_tytus-sync/` namespace, and the tray applies exactly those keys with
bounded StartAfter listings (hundreds of milliseconds instead of minutes).

Properties:

- **Per-binding opt-in.** Bisync stays the default; toggling a binding
  progressive unloads its bisync LaunchAgent first (proven, fail-closed —
  a binding never has two movers, and never zero).
- **Pull-only in v1.** Pod → Mac. Local edits are counted and shown
  honestly (`local_edits_not_synced`) but do NOT sync up until v2.
- **Never overwrites silently.** Content divergence goes through
  deterministic keep-both conflict naming; sha256 is verified end-to-end;
  cursor state is a fsync'd journal.
- **Self-healing.** Every 24 h a report-only reconcile compares the remote
  listing against the local tree (honest per-prefix timeouts — a slow
  prefix is named, never faked). A gap-halted route is only released by an
  adjudicated reconcile proving the files present.

Tray API (localhost, requires `Sec-Fetch-Site: same-origin`):

```text
POST /api/shared-folders/progressive            {"binding": <id|slug|alias>, "enabled": true|false}
POST /api/shared-folders/progressive/sync-now   {"binding": <id|slug|alias>}
GET  /api/shared-folders/progressive/status     per-binding cursors, errors, reconcile summary
```

Kill switch: `TYTUS_PROGRESSIVE_SHARED_SYNC=0` pauses all consumer polls
without touching state. The sharing master pause gates it too.

Note on identities: producers emit under the Provider registry's folder id
(`grant.folder_id`), which is not the local sidecar id — the tray discovers
the remote namespace automatically and caches it in the binding sidecar as
`progressive.remote_binding_id`. Contract details live in the sprint package
(`docs/sprints/tytus-garagetytus-progressive-sync-2026-06-30/` in
ProjectWannolot).

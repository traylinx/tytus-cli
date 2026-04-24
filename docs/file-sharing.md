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
- **You need a shared filesystem that multiple pods mount at once.**
  Planned for v0.7 (Garage-backed S3 shared filesystem, design at
  `development/audits/garage-s3-shared-filesystem-audit.md`). Today's
  `tytus push/pull` is per-file between your Mac and one pod at a time.
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

For GB-scale transfers, the v0.7 Garage sprint adds an S3-backed
shared filesystem mounted into every pod. At that point `tytus push`
changes its transport under the hood (CLI surface stays identical),
and the 100 MB cap goes away. Don't plan around the cap lifting today.

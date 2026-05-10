# Atomek Workbench

Atomek is the TytusOS Control Tower for local files, code, markdown, chat, artifacts, local agents, pods, shared folders, and app skills.

Use it when you want to open a real folder, edit files, ask AI about the active file, preview patches, or launch an installed local agent from the same context.

## Open Atomek

```bash
tytus tray start
tytus open
```

Then open **Atomek** from the TytusOS dock, launcher, or app list.

## What Atomek is for

| Need | Use |
|---|---|
| Open a local folder or file | Explorer -> Open Folder / Open File |
| Edit code or markdown | Monaco editor tabs |
| Ask AI about the current file | Chat with the active-file context chip |
| Generate a file or patch | Chat -> artifact / preview edit |
| Run a local CLI with context | Control Tower -> Open Terminal |
| Ask an installed local agent | Control Tower -> local job |
| Inspect generated results | Outputs |

Atomek is not a second local-agent runtime. It controls existing Tytus and local Control Tower resources through the Tytus host bridge.

## AI model selection

Atomek uses the Tytus host AI bridge and global AIL configuration. It must not hardcode model IDs.

If the app shows an old model, update the global AIL route/model configuration and refresh the app. Do not patch Atomek source with a fixed model name.

## Control Tower

The **Control Tower** panel discovers allowlisted local tools through the tray/host bridge. Typical tools are:

- Tytus Terminal
- pi
- OpenCode
- Codex
- Claude Code
- Gemini
- Qwen
- Kimi
- Aider

Use **Open Terminal** for supervised interactive work. Use local jobs for background agent work that streams output back into Atomek and returns artifacts or patch previews.

## Mission packs and resource graph

Atomek mission packs live under **Tytus Home / Missions**. They are shared working folders for coordinating pods, local agents, app skills, and shared-folder handoffs without giving agents blind write access to the source workspace.

A mission pack contains `MISSION.md`, `MISSION.json`, `RESOURCES.md`, `TASKS.md`, `HANDOFF.md`, `INBOX.md`, `OUTBOX.md`, `AUDIT.jsonl`, and `NEXT.md`. Atomek can list and resume existing packs through the tray `GET /api/missions` bridge, then restore the mission badge, task graph, selected resources, and handoff prompt.

When a local-agent run starts, Atomek binds it to the selected task card, streams output in Control Tower, saves the final transcript under `runs/`, and exposes **Cancel** through the tray job bridge. Patch-shaped output is still converted into an Atomek preview before any write.

The resource graph shows pods, local CLIs, app skills, shared folders, and the active workspace. Use **Use** to attach a resource to the current mission prompt. Use **Setup** when the resource exists conceptually but needs a local install command, app launch, or bridge.

Security rules:

- no arbitrary shell command execution from model text
- no direct browser fetches to pod/model endpoints that bypass the host bridge
- no blind file writes from AI output
- edits must become preview diffs or artifacts first

## Files vs Atomek

Use **Files** for broad browsing across Tytus Home, Shared, Inbox, Outbox, Downloads, and pod workspaces.

Use **Atomek** for editing and agent work against selected files or folders.

## Troubleshooting

| Problem | Fix |
|---|---|
| Old Atomek UI or duplicate Control Tower icons | Hard-refresh TytusOS. Current app should be `tytus-app-atomek@v0.4.17` or newer. |
| Files appear but editor is blank | Reopen the file, hard-refresh, then check the browser console. |
| Folder does not expand/collapse | Hard-refresh. Mission Control and folder fixes ship in Atomek `v0.4.17`. |
| Local tool missing | Install the local CLI/tool, then click **Refresh capabilities**. |
| Remote model/pod fetch gets CORS | Route through the Tytus host proxy. Browser apps should not direct-fetch pod public URLs. |
| Model list shows an old model | Fix global AIL config. Do not hardcode models in Atomek. |

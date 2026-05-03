#!/usr/bin/env bash
set -euo pipefail

# Regenerates os-docs.md by concatenating TytusOS user-manual sources.
# Run from the tytus-cli repo root whenever ../tytus-os/docs/user-manual/
# changes. The result is bundled into the tytus binary via include_str!
# and exposed as `tytus os-docs` and the `tytus_os_docs` MCP tool.
#
# Source: ../tytus-os/docs/user-manual/*.md
# Output: ./os-docs.md

cd "$(dirname "$0")/.."
SRC="../tytus-os/docs/user-manual"
OUT="os-docs.md"

if [ ! -d "$SRC" ]; then
    echo "FAIL: $SRC not found. Are tytus-cli and tytus-os checked out as siblings?"
    exit 1
fi

# Order: getting-started first (sets context), then desktop surfaces,
# then files/clipboard, then settings/apps catalog, then troubleshooting + about.
ORDER=(
    "getting-started.md"
    "windows.md"
    "desktop.md"
    "dock.md"
    "launcher.md"
    "keyboard-shortcuts.md"
    "files.md"
    "settings.md"
    "apps-catalog.md"
    "troubleshooting.md"
    "about.md"
)

{
    echo "# TytusOS — User Manual (LLM-facing reference)"
    echo
    echo "> You are an AI agent (Claude Code, OpenCode, KiloCode, Gemini, Codex,"
    echo "> Cursor, Vibe, etc.) running on a user's machine that has TytusOS"
    echo "> installed. This document is the complete user manual, concatenated"
    echo "> from the source-of-truth markdown in the tytus-os repository. Read"
    echo "> it BEFORE answering any question about how TytusOS looks, behaves,"
    echo "> or what its keyboard shortcuts / settings panels / apps do."
    echo ">"
    echo "> The companion reference for the \`tytus\` CLI itself is \`tytus llm-docs\`."
    echo "> This document covers the desktop OS surface; \`llm-docs\` covers the"
    echo "> command-line tool."
    echo
    echo "---"
    echo

    for f in "${ORDER[@]}"; do
        if [ ! -f "$SRC/$f" ]; then
            echo "FAIL: $SRC/$f missing" >&2
            exit 1
        fi
        echo
        echo "<!-- ==== $f ==== -->"
        echo
        cat "$SRC/$f"
        echo
    done

    echo
    echo "---"
    echo
    echo "<!-- ==== troubleshooting/clipboard.md ==== -->"
    echo
    if [ -f "../tytus-os/docs/troubleshooting/clipboard.md" ]; then
        cat "../tytus-os/docs/troubleshooting/clipboard.md"
        echo
    fi
} > "$OUT"

LINES=$(wc -l < "$OUT" | tr -d ' ')
SIZE=$(wc -c < "$OUT" | tr -d ' ')
echo "Wrote $OUT ($LINES lines, $SIZE bytes)"

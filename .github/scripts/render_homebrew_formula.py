#!/usr/bin/env python3
"""
Render contrib/homebrew/tytus.rb with a specific version and per-target SHAs.

Why a script instead of sed: we need to DROP entire `on_arm`/`on_intel` blocks
when the corresponding binary wasn't built for this release. Sed can't handle
multi-line conditional removal cleanly. The template is authored with every
target included, and this script prunes the branches where no SHA was supplied.
"""
import argparse
import re
import sys


def parse_args():
    p = argparse.ArgumentParser()
    p.add_argument("--template", required=True)
    p.add_argument("--version", required=True)
    p.add_argument("--sha-macos-aarch64", default="")
    p.add_argument("--sha-macos-x86_64", default="")
    p.add_argument("--sha-linux-x86_64", default="")
    p.add_argument("--sha-linux-aarch64", default="")
    return p.parse_args()


def substitute(text: str, args) -> str:
    text = text.replace("{{VERSION}}", args.version)
    text = text.replace("{{SHA_MACOS_AARCH64}}", args.sha_macos_aarch64)
    text = text.replace("{{SHA_MACOS_X86_64}}",  args.sha_macos_x86_64)
    text = text.replace("{{SHA_LINUX_AARCH64}}", args.sha_linux_aarch64)
    text = text.replace("{{SHA_LINUX_X86_64}}",  args.sha_linux_x86_64)
    return text


def drop_empty_on_arm_blocks(text: str) -> str:
    """
    Drop any on_arm/on_intel block whose sha256 line was rendered as empty
    (`sha256 ""`). This happens when a target had no SHA provided (e.g.
    linux-aarch64 wasn't built for this release).

    Operates line-by-line so we don't need a full Ruby parser.
    """
    lines = text.split("\n")
    out = []
    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()

        if stripped in ("on_arm do", "on_intel do"):
            # Collect lines until matching `end`
            block = [line]
            depth = 1
            i += 1
            while i < len(lines) and depth > 0:
                block.append(lines[i])
                s = lines[i].strip()
                if s.endswith(" do") or s == "do":
                    depth += 1
                elif s == "end":
                    depth -= 1
                i += 1
            block_text = "\n".join(block)
            # Drop if the sha256 line is empty (missing build target)
            if 'sha256 ""' in block_text:
                continue
            out.append(block_text)
            continue

        out.append(line)
        i += 1

    return "\n".join(out)


def drop_empty_platform_blocks(text: str) -> str:
    """
    After dropping empty arches, if an `on_macos`/`on_linux` block is now
    empty (contains no `url` / `sha256`), drop the whole block.
    """
    lines = text.split("\n")
    out = []
    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()

        if stripped in ("on_macos do", "on_linux do"):
            block = [line]
            depth = 1
            i += 1
            while i < len(lines) and depth > 0:
                block.append(lines[i])
                s = lines[i].strip()
                if s.endswith(" do") or s == "do":
                    depth += 1
                elif s == "end":
                    depth -= 1
                i += 1
            block_text = "\n".join(block)
            if "url " not in block_text:
                # Empty platform block — drop it
                continue
            out.append(block_text)
            continue

        out.append(line)
        i += 1

    return "\n".join(out)


def main():
    args = parse_args()
    with open(args.template) as f:
        text = f.read()

    text = substitute(text, args)
    text = drop_empty_on_arm_blocks(text)
    text = drop_empty_platform_blocks(text)

    # Sanity: no {{}} placeholders remain
    leftover = re.findall(r"\{\{[A-Z_]+\}\}", text)
    if leftover:
        sys.exit(f"ERROR: unsubstituted placeholders: {leftover}")

    print(text, end="")


if __name__ == "__main__":
    main()

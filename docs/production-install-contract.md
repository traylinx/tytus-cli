# Tytus production install contract

**Status:** Phase 0 contract for the Tytus OS multiplatform production sprint.
**Source sprint:** `/Users/sebastian/MAKAKOO/development/sprints/queued/TYTUS-OS-MULTIPLATFORM/SPRINT.md`
**Date:** 2026-05-06

## Product law

Tytus production install is grandma-secure or it does not ship as production.

> One trusted install action → Tytus opens → guided setup → ready.

One trusted install action means exactly one of these user-facing starts:

- click a signed installer from `https://get.traylinx.com/download`
- copy one command from `https://get.traylinx.com/download`

After that first action, Tytus owns every technical step. The user never has to understand terminals, Rust, PATH, WireGuard, Wintun, drivers, routes, services, keychains, firewalls, logs, ports, or system administration.

## Full-stack install requirement

Every GA consumer installer/package must install and configure the full stack:

- `tytus`
- `tytus-tray`
- `tytus-mcp`
- embedded TytusOS web shell
- private tunnel runtime
- service/autostart hooks
- credential-store integration
- required local firewall/loopback allowances
- update metadata and update checker
- uninstall and repair path

A platform where only the CLI works is not production-ready.

## Private tunnel rule

WireGuard/Wintun is essential infrastructure for full TytusOS pod functionality, but it is an implementation detail. The user must never manually install, configure, debug, or name WireGuard/Wintun.

Allowed user-visible wording:

> Tytus needs permission to create its private encrypted connection to your pod.

Forbidden user-visible wording in the happy path:

- install WireGuard
- install Wintun
- run this admin command
- edit routes
- enable TUN
- open firewall settings
- copy this token into a config file

If private tunnel setup fails, the product shows a repair button and plain message. It does not print a stack trace or command wall.

## First-run flow

The first-run wizard must complete this sequence on every GA platform:

1. Welcome and privacy summary.
2. Login.
3. Credential-store check.
4. Pod allocation or pod selection.
5. Private tunnel setup behind one clear OS permission prompt where required.
6. Health check.
7. Juli3ta smoke generation.
8. Done screen with “Open Tytus OS”.

Each step must have a troubleshoot branch. Repair must be available during setup, not only after setup succeeds.

## Supported release labels

| Label | Meaning |
|---|---|
| GA | Full grandma-secure path passes fresh-VM smoke. |
| Beta | Install path exists but one or more grandma-secure gates are not proven. |
| Developer preview | Expert path only; CLI/source builds acceptable. |

A platform cannot be called GA if it needs manual dependency installation, manual WireGuard/Wintun setup, CLI-only operation, unsigned public artifacts, or terminal repair instructions in the normal path.

## GA gates

A GA platform passes only with evidence:

- signed installer/package downloaded from `https://get.traylinx.com/download`
- install starts from one user action
- Tytus opens automatically
- first-run wizard completes without terminal knowledge
- credentials survive reboot and are stored securely
- private tunnel works after reboot
- `tytus test` gateway phase passes
- Juli3ta generates a 30-second song
- update from release candidate to final works without re-login
- uninstall/repair works
- support bundle redacts secrets

## Platform stance for this sprint

| Platform | Phase 0 stance |
|---|---|
| macOS 13+ Apple Silicon + Intel | GA target. |
| Ubuntu 22.04/24.04 LTS | GA target. |
| Windows 10 22H2 + Windows 11 | GA target only if signed MSI, Wintun/WireGuard service path, SmartScreen behavior, and fresh-VM smoke pass. |
| Fedora 39+ | Beta by default; promote only after `.rpm`, tray, SELinux/AppArmor, tunnel, and update smoke pass without schedule risk. |
| Arch/other Linux | Developer preview. |

## Non-negotiable release rule

Consumer docs and download pages must not advertise source builds or CLI-only installs as the primary path. `cargo install --git` is developer-only.

## Installer script contract

As of 2026-05-07, `install.sh` and `install.ps1` default to the production contract:

- normal one-command installs use checksum-verified release artifacts
- missing platform artifacts fail with a plain production-unavailable message
- installers do not fall back to Rust/cargo source builds for normal users
- source builds require explicit developer opt-in:
  - `TYTUS_INSTALL_MODE=dev-source`
  - `TYTUS_DEV_SOURCE_INSTALL=1`
- public beta/pre-release installs must name the release explicitly with `TYTUS_RELEASE_TAG`, and still verify `SHA256SUMS`

This is a guardrail, not a GA claim. A platform is still GA only after the GA gates above pass on a fresh VM.

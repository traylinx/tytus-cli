# ADR: Tytus tunnel platform strategy

**Status:** Accepted for Phase 0.
**Date:** 2026-05-06.
**Sprint:** TYTUS-OS-MULTIPLATFORM.

## Decision

Tytus keeps a platform-specific tunnel strategy behind one product abstraction: “private encrypted connection to your pod.”

The user-facing contract is identical everywhere. The implementation is allowed to differ per OS when that reduces install risk.

## macOS

macOS GA keeps the current userspace tunnel approach unless Phase 1 evidence proves it cannot satisfy reboot, signing, or reliability gates.

No Apple Network Extension rewrite is planned for this sprint.

Network Extension becomes required only if one of these triggers fires:

- current userspace tunnel cannot survive required lifecycle/service behavior
- signing/hardened runtime blocks current helper model
- route/interface setup cannot be made repairable without NE
- Apple security model rejects the chosen helper path during signed/notarized smoke

If any trigger fires, macOS GA is blocked until Apple Network Extension entitlement approval is submitted and approved. The release label changes to private beta rather than pretending a manual tunnel is production.

## Linux

Ubuntu GA uses userspace tunnel plus tightly scoped privilege helper/polkit path where elevated route/TUN actions are required.

Linux headless/minimal remains developer preview unless a secure non-graphical credential and tunnel UX is proven.

Fedora remains beta by default because SELinux/AppArmor, tray, and package dependency behavior require additional smoke.

## Windows

Windows GA uses the MSI-managed WireGuard/Wintun service path defined in `docs/architecture/windows-tunnel.md`.

Phase 1 must still run a feasibility spike for the current `boringtun`/`tun` dependency on Windows:

- does it compile
- does it require Wintun anyway
- does it support required route/interface behavior
- does it reduce MSI/service complexity

The spike may change implementation details, but it cannot change the product law: no manual WireGuard/Wintun setup.

## Cross-platform abstraction

The code exposes one tunnel abstraction to CLI/tray/TytusOS:

- status
- start
- stop
- repair
- route/interface evidence
- support-bundle evidence
- health check for `tytus test`

Each platform implementation may use a different backend. UI copy remains platform-neutral.

## Redistribution and legal check

Before Windows MSI implementation work starts, Phase 1 must record Wintun/WireGuard redistribution status:

- license obligations
- driver package source
- whether redistribution is allowed inside Tytus MSI
- whether separate driver signing or attestation is required
- behavior when WireGuard for Windows is already installed

If redistribution or driver signing blocks one-action install, Windows becomes private beta until corrected.

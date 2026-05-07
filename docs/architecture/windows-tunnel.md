# ADR: Windows private tunnel path

**Status:** Accepted for Phase 0.
**Date:** 2026-05-06.
**Sprint:** TYTUS-OS-MULTIPLATFORM.

## Decision

Windows GA uses an MSI-managed WireGuard/Wintun service path as the primary private tunnel implementation.

The user never manually installs WireGuard or Wintun. The installer/wizard owns driver/service setup under one clear UAC prompt and uses plain product wording:

> Tytus needs permission to create its private encrypted connection to your pod.

A bundled userspace Wintun path may be spiked as a fallback, but it is not the default GA bet unless it beats the service path in fresh Windows 10 and Windows 11 smoke.

## Why this decision

TytusOS needs a real private pod tunnel. Windows currently has daemon and tunnel stubs, so pretending this is a small parity task would create a fake release.

Wintun is the practical TUN adapter on Windows. The lowest-risk consumer product path is to let the signed MSI install and manage the required driver/service components and expose only Tytus repair/status UI to the user.

## Product constraints

- One signed MSI or one PowerShell wrapper that downloads and verifies the signed MSI.
- One normal UAC prompt in the happy path.
- No manual WireGuard/Wintun installation.
- No manual route commands.
- No firewall-settings instructions in the happy path.
- Tunnel survives reboot.
- Repair is available from Tytus UI.
- If SmartScreen or unsigned-publisher warnings block normal users, Windows is private beta.

## Technical requirements

The Windows tunnel implementation must include:

- signed `tytus.exe`, `tytus-tray.exe`, `tytus-mcp.exe`, helper binaries, DLLs, and installer catalog
- Wintun/WireGuard driver/service install or verified presence
- service registration through MSI custom action or a tightly scoped helper
- route setup through Windows API where feasible, command fallback only behind a narrow wrapper
- Windows Defender Firewall rules for Tytus private tunnel traffic and local loopback control plane
- reboot persistence through Windows Service or Task Scheduler
- uninstall that removes Tytus service/tray binaries and Tytus-owned firewall/service entries
- repair action that reinstalls service, repairs firewall rule, rotates IPC token, and restarts tray
- support bundle with redacted logs, service status, route/interface status, Windows version, installer version, and update channel

## Kill criteria

Windows remains a GA target only if the selected path passes this gate within five focused engineering days after Phase 2 substrate is ready:

1. Fresh Windows 10 22H2 VM installs from signed MSI.
2. Fresh Windows 11 VM installs from signed MSI.
3. One UAC prompt in the happy path.
4. Tunnel interface exists after setup.
5. Routes are correct after setup.
6. Windows Defender Firewall does not block the pod path.
7. Reboot keeps daemon/tray/tunnel repairable.
8. `tytus test` gateway phase passes.
9. Juli3ta generates a 30-second song through TytusOS.
10. Uninstall and repair work.

If this gate fails, Windows moves to private beta. The product law is not weakened.

## Implementation sequence

1. Capture current Windows stub behavior and compile failures in CI.
2. Implement Phase 2 IPC/path/service substrate first.
3. Implement Windows daemon lifecycle.
4. Build MSI skeleton with signed binaries and service registration.
5. Add Wintun/WireGuard service setup.
6. Add route and firewall management.
7. Add tray/wizard repair UI hooks.
8. Run fresh Windows 10/11 smoke.
9. Decide GA or private beta from evidence.

## Rejected options

| Option | Reason rejected for GA |
|---|---|
| Ask user to install WireGuard manually | Violates grandma-secure install contract. |
| CLI-only Windows release | Violates production goal. |
| Public HTTPS edge instead of private tunnel | Bootstrap/recovery only; not full TytusOS pod functionality. |
| Unsigned MSI or unsigned helper | Blocks public production release. |
| Keep Windows as “experimental” while calling release multiplatform GA | Misleads users and support. |

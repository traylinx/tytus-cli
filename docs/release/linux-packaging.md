# Linux packaging and autostart decision

**Status:** Accepted for Phase 0.  
**Date:** 2026-05-06.  
**Sprint:** TYTUS-OS-MULTIPLATFORM.

## Decision

Ubuntu 22.04/24.04 LTS GA ships as a signed `.deb` installed by `install.sh` from `https://get.traylinx.com/download`.

Fedora 39+ ships as a signed `.rpm` beta unless fresh-VM smoke proves tray, tunnel, SELinux/AppArmor, update, and uninstall behavior without schedule risk.

Snap, Flatpak, AppImage, Arch AUR, and generic tarball are not GA paths for this sprint.

## Ubuntu GA install flow

1. User runs one command from the download page.
2. `install.sh` detects distro and architecture.
3. Script downloads package metadata and signed `.deb`.
4. Script verifies SHA256/signature.
5. Script installs required package dependencies.
6. Script installs Tytus package.
7. Package registers systemd user service and desktop autostart entry where appropriate.
8. Tytus starts tray/daemon and opens browser-served TytusOS.

## Autostart model

Ubuntu desktop uses:

- systemd user service for daemon/tray lifecycle where available
- XDG autostart desktop entry as compatibility fallback for graphical login
- repair command to reinstall service/autostart hooks

Headless/minimal Ubuntu is developer preview unless the credential, tunnel, and repair UX can be made grandma-secure without graphical login.

## Tray fallback

Modern GNOME tray/AppIndicator behavior is not assumed safe until Phase 1 inventory proves it.

If tray icon behavior is unreliable on Ubuntu GNOME, GA fallback is:

- browser opens automatically after install
- small status/repair window or background service notification opens when action is required
- CLI-only fallback is not allowed for GA

## Dependencies to inventory in Phase 1

- `libayatana-appindicator` or equivalent tray/status dependency
- GTK dependencies actually used by `tytus-tray`
- Secret Service/libsecret availability
- systemd user service availability
- Polkit/pkexec availability
- TUN device access
- route tooling
- AppArmor behavior on Ubuntu
- SELinux behavior on Fedora beta

## Uninstall scope

Linux uninstall removes:

- Tytus package files
- Tytus systemd user service files
- Tytus XDG autostart files
- Tytus-owned firewall/helper configuration
- Tytus tray/daemon binaries

Linux uninstall preserves user data by default and offers explicit separate removal for:

- local pod metadata
- cached web assets
- logs
- credentials/tokens
- generated user content

Secrets must not remain after explicit data removal.

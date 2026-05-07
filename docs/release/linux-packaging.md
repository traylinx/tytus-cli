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

## Phase 6 dry-run package shape

The release workflow now builds the Linux release artifact on `ubuntu-22.04`,
not `ubuntu-latest`, so the default `x86_64-unknown-linux-gnu` binary keeps the
Ubuntu 22.04 glibc floor. Ubuntu 24.04 can run that artifact; the inverse is not
safe.

`pkg/build-deb.sh` creates an unsigned internal `.deb` dry-run artifact only:

- `/usr/bin/tytus`
- `/usr/bin/tytus-mcp`
- `/usr/bin/tytus-tray`
- `/usr/lib/systemd/user/tytus-daemon.service`
- `/etc/xdg/autostart/tytus-tray.desktop`
- `/usr/share/applications/tytus.desktop`
- `/usr/share/icons/hicolor/512x512/apps/tytus.png`

The package intentionally does not switch `install.sh` to `.deb` yet and does
not publish `.deb` files as GitHub Release assets. Workflow artifacts are named
`*-unsigned-DO-NOT-DISTRIBUTE-deb` with 7-day retention until Linux package or
repository signing lands.

Runtime dependencies declared by the dry-run package:

- `libc6 (>= 2.35)`
- `ca-certificates`
- `libdbus-1-3`
- `libsecret-1-0`
- `libgtk-3-0`
- `libayatana-appindicator3-1`
- `iproute2`
- `xdg-utils`
- recommends `policykit-1 | polkitd`

Open caveats before Linux GA:

- unsigned `.deb` is internal only;
- `install.sh` must verify signed package metadata before using `.deb` by default;
- Ubuntu 22.04 and 24.04 fresh-VM install smoke must prove Secret Service,
  AppIndicator/tray behavior, browser launch, tunnel elevation, repair, update,
  and uninstall;
- if `ubuntu-22.04` Actions runner deprecation starts, move release packaging to
  an Ubuntu 22.04 container or equivalent glibc-pinned build image before GA.

# Phase 1 Linux tray and dependency inventory

**Status:** Phase 1 evidence seed.  
**Date:** 2026-05-06.  
**Scope:** Ubuntu 22.04/24.04 LTS GA target; Fedora beta.

## Current tray stack

`tytus-tray` uses the Rust `tray-icon` crate plus image/tiny_http/reqwest/keyring dependencies. It is not a Tauri/Electron/WebView shell in this sprint. TytusOS is browser-served from the local daemon/tray web server.

## Linux CI dependency set

The new PR/release workflows install:

```bash
sudo apt-get install -y \
  pkg-config \
  libdbus-1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

These are the first Ubuntu truth dependencies for `keyring`, `tray-icon`, GTK/AppIndicator behavior, and icon rendering.

## Known Linux risks

| Risk | Why it matters | Sprint treatment |
|---|---|---|
| GNOME tray/AppIndicator behavior | Ubuntu GNOME may not expose classic tray icons consistently. | Browser auto-open + small status/repair window fallback; no CLI-only GA. |
| Secret Service/libsecret availability | Credential store may be unavailable before graphical login or in minimal/headless sessions. | Ubuntu desktop GA only; headless/minimal is developer preview. |
| systemd user service availability | Needed for daemon/tray lifecycle after login/reboot. | Phase 2 service abstraction + Phase 6 package scripts. |
| Polkit/pkexec | Needed for controlled privileged tunnel/route operations. | Phase 4 helper design. |
| TUN device/route tooling | Required for private tunnel. | Phase 4 smoke gate. |
| AppArmor/SELinux | May block helper/tunnel behavior. | Ubuntu AppArmor inventory in Phase 1/4; Fedora beta until SELinux smoke passes. |

## CI truth rule

Ubuntu CI is expected to compile and test all Rust packages with the above dependency set. If it fails, the exact missing package or compile error becomes Phase 2/4 input.

Fedora is not a GA blocker in Phase 1. Fedora remains beta until fresh-VM smoke proves packaging, tray, SELinux/AppArmor, tunnel, update, and uninstall.

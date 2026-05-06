# ADR: TytusOS web shell and tray container

**Status:** Accepted for Phase 0.  
**Date:** 2026-05-06.  
**Sprint:** TYTUS-OS-MULTIPLATFORM.

## Decision

For this sprint, the TytusOS web shell is the existing browser-served TytusOS/Tower UI embedded into the `tytus-tray` release artifact and served from the local Tytus daemon/tray web server.

Tytus does not introduce a new Tauri, Electron, Wry, WebView2, WKWebView, or WebKitGTK application shell in this sprint.

The consumer experience is still app-like:

- installer starts tray/daemon
- tray/daemon serves local TytusOS
- default browser opens automatically to the local TytusOS URL
- tray remains the lifecycle/status/repair entry point

## Why this decision

A dedicated webview wrapper would add new platform dependencies at the same time this sprint is already solving signing, installer, tunnel, credential, and update parity.

Browser-served TytusOS keeps the platform surface smaller and avoids WebKitGTK/WebView2 packaging complexity for the GA path. Linux tray support still matters, but Linux GA does not depend on embedding a webview runtime.

## Token bootstrap

The local TytusOS page obtains daemon access through same-origin bootstrap from the local daemon/tray web server.

Rules:

- The browser loads TytusOS from `http://127.0.0.1:<port>/`.
- The server sets an HTTP-only, SameSite=Strict local session cookie or equivalent same-origin bootstrap token.
- The long-lived IPC token is never exposed in URL query parameters.
- The token is never embedded in static JS bundles.
- Support bundles and logs redact any bootstrap/session material.
- Cloud-hosted pages are not allowed to call the local daemon in this sprint.

## Tray technology stance

The tray remains Rust-native using the current `tytus-tray` stack, adapted per platform as needed.

Phase 1 must inventory current tray dependencies and prove the intended Linux/Windows compile path. Phase 2 owns platform substrate changes needed to remove Unix-only assumptions.

If Ubuntu GNOME tray behavior makes a tray icon unreliable, the fallback is a small native status/repair window plus browser launch, not CLI-only operation.

## CORS and origin model

The production control plane accepts browser requests only from the local TytusOS origin served by Tytus.

Allowed:

- `http://127.0.0.1:<active-port>`
- test-only origins under explicit test configuration

Rejected:

- cloud TytusOS origins
- arbitrary localhost ports without the active bootstrap session
- requests with missing or invalid bootstrap/session material

## Deferred

- Tauri/Electron/native webview shell.
- Cloud-hosted TytusOS controlling local daemon.
- Custom URL protocol deep-link control plane.
- Linux WebKitGTK dependency as a GA requirement.

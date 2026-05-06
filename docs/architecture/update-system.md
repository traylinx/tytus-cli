# ADR: Tytus update system ownership

**Status:** Accepted for Phase 0.  
**Date:** 2026-05-06.  
**Sprint:** TYTUS-OS-MULTIPLATFORM.

## Decision

The local daemon owns update checking, manifest verification, artifact download orchestration, and update state. Platform-specific installer/package tooling owns privileged apply.

Tray/TytusOS owns user interaction:

- show update available
- show release notes
- request user confirmation
- show OS permission prompt context
- show progress/failure/rollback state

CLI owns expert escape hatches:

- `tytus update check`
- `tytus update apply`
- `tytus update rollback`
- `tytus update status --json`

## Mechanism by OS

| OS | Apply mechanism |
|---|---|
| macOS | Installer-backed signed/notarized `.pkg` or `.dmg` upgrade launched with user consent. |
| Ubuntu | Signed `.deb` applied through package manager with Polkit/native prompt. |
| Fedora beta | Signed `.rpm` applied through package manager with Polkit/native prompt after beta smoke. |
| Windows | Signed MSI major/minor upgrade launched with user consent and one UAC prompt. |

No silent background update ships in this sprint.

## Manifest contract

The daemon fetches channel manifests from:

- `https://get.traylinx.com/updates/stable/manifest.json`
- `https://get.traylinx.com/updates/beta/manifest.json`

Required manifest fields:

- `version`
- `channel`
- `platform`
- `artifact_url`
- `sha256`
- `signature`
- `key_id`
- `min_client_version`
- `min_compat_version`
- `rollout_percentage`
- `revoked_artifacts`
- `published_at`

The daemon rejects unsigned manifests, unknown keys, hash mismatch, revoked artifacts, channel downgrade, incompatible client versions, and artifacts not signed by the platform signing path.

## Key rotation

Clients embed at least two manifest public-key slots:

- active key
- next key

A manifest may announce the next key only when signed by the active key. Clients accept the next key after successful update to a version that embeds it. Key compromise uses `min_client_version` and `revoked_artifacts`; if compromise cannot be safely recovered in-band, public release is paused and installers are reissued.

## IPC surface

The daemon exposes update endpoints through the local IPC contract:

- `GET /api/update/status`
- `POST /api/update/check`
- `POST /api/update/download`
- `POST /api/update/apply`
- `POST /api/update/rollback`

State-changing endpoints require IPC token auth and explicit user gesture. Apply endpoints must record evidence for support bundles.

## State storage

Update state lives in platform app data through the Phase 2 path substrate:

- last checked time
- current channel
- downloaded artifact metadata
- verification transcript summary
- previous version pointer for rollback
- last failure reason

No update private signing material exists on client machines.

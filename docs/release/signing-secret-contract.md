# Release signing secret contract

**Status:** Phase 6 fail-closed release contract.
**Date:** 2026-05-07.
**GitHub Environment:** `release-production` with required reviewers.

## Rule

Production releases fail closed unless platform signing and update signing are
available and verified. Missing signing credentials may still allow internal
dry-runs or private beta artifacts, but they must not be labeled production.

## Workflow inputs

`.github/workflows/release.yml` exposes:

- `release_tier`
  - `internal-dry-run` — default; validates build/artifacts, never public.
  - `private-beta` — may publish a GitHub prerelease if `publish_release=true`.
  - `production` — currently blocked until signed/notarized package publishing
    steps are implemented and all signing secrets are present.
- `publish_release`
  - `false` by default; validates build/artifacts without creating a public
    GitHub Release.
  - `true` creates/updates a GitHub Release only when release-tier policy passes.

## Required secrets

Store these in the protected `release-production` environment. Do not store
private keys in repo-level unprotected secrets.

| Secret | Required for | Format |
|---|---|---|
| `APPLE_DEVELOPER_ID_APPLICATION_CERT_P12` | macOS binary signing | base64-encoded `.p12` |
| `APPLE_DEVELOPER_ID_INSTALLER_CERT_P12` | macOS `.pkg` signing | base64-encoded `.p12` |
| `APPLE_CERT_PASSWORD` | macOS cert import | secret string |
| `APPLE_NOTARY_APPLE_ID` | Apple notarization | Apple ID email |
| `APPLE_NOTARY_TEAM_ID` | Apple notarization | Team ID |
| `APPLE_NOTARY_APP_PASSWORD` | Apple notarization | app-specific password |
| `WINDOWS_CODESIGN_CERT_PFX` | Windows exe/MSI signing | base64-encoded `.pfx` |
| `WINDOWS_CODESIGN_PASSWORD` | Windows cert import | secret string |
| `WINDOWS_CODESIGN_TIMESTAMP_URL` | Windows timestamping | optional; default `http://timestamp.digicert.com` |
| `LINUX_GPG_PRIVATE_KEY` | Debian/RPM/repo signing | armored private key |
| `LINUX_GPG_KEY_ID` | Debian/RPM/repo signing | key id/fingerprint |
| `LINUX_GPG_PASSPHRASE` | Debian/RPM/repo signing | secret string |
| `UPDATE_MANIFEST_ED25519_PRIVATE_KEY` | update manifest signing | base64 or PEM, implementation-specific |
| `UPDATE_MANIFEST_KEY_ID` | update manifest signing | stable key id |

## Detection contract

The workflow deliberately splits non-production and production gates:

- `check-nonproduction-release-policy` never touches signing secrets and does
  not require `release-production` approval.
- `check-production-signing-secrets` runs only when `release_tier=production`,
  declares `environment: release-production`, and is the only current job that
  reads production signing secrets.

The production signing job emits booleans only:

- `macos_signing_available`
- `windows_signing_available`
- `linux_signing_available`
- `update_signing_available`

It never prints secret values. Production release policy consumes these booleans
before any publish step. If the job is skipped for non-production runs, signing
readiness is reported as `not checked for non-production`.

Configure the `release-production` environment with required reviewers and
deployment branch/tag restrictions (`main` plus `v*` release tags). Keep
"prevent self-review" enabled.

## macOS production gate

macOS production publishing requires:

1. Developer ID Application cert imported.
2. Developer ID Installer cert imported.
3. Binaries signed with hardened runtime.
4. `.pkg` signed with Developer ID Installer.
5. `xcrun notarytool submit --wait` accepted.
6. `xcrun stapler staple` completed.
7. `spctl --assess --type install` passes.

Unsigned `.pkg` files may exist only as workflow artifacts named
`*-unsigned-DO-NOT-DISTRIBUTE-pkg`; they must never be public release assets.

## Windows production gate

Windows production publishing requires:

1. `tytus.exe`, `tytus-tray.exe`, `tytus-mcp.exe`, and MSI signed.
2. Timestamping succeeds.
3. `Get-AuthenticodeSignature` reports valid signatures.
4. Fresh-VM install smoke confirms no unsigned-publisher wall.

## Linux production gate

Ubuntu production publishing requires:

1. Signed `.deb` or signed repository metadata.
2. SHA256 present in release/update metadata.
3. `install.sh` verifies hash/signature before installing.
4. Fresh-VM install smoke confirms package, service/tray, tunnel, and repair.

Fedora remains beta until `.rpm`, SELinux/AppArmor, tray, tunnel, and update
smoke pass.

## Rotation and revocation

- Rotate Apple app-specific password immediately if exposed; update
  `APPLE_NOTARY_APP_PASSWORD` and rerun notarization smoke.
- Revoke and replace `.p12`/`.pfx` certs if private keys leak.
- Rotate `UPDATE_MANIFEST_ED25519_PRIVATE_KEY` by shipping a new public key with
  a new `UPDATE_MANIFEST_KEY_ID`, then revoke the old key in manifest metadata.
- Record every rotation in `docs/release/cert-procurement.md` and the sprint
  verdicts folder.

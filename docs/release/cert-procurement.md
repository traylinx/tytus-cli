# Tytus release certificate and signing procurement

**Status:** Phase 0 procurement track.  
**Date:** 2026-05-06.  
**Sprint:** TYTUS-OS-MULTIPLATFORM.

## Rule

Certificate procurement is calendar-critical. It starts before engineering depends on it and blocks public GA if incomplete.

Phase 6 may build unsigned internal artifacts for testing, but public GA requires trusted signed artifacts and update metadata.

## Day-2 deadline

By 2026-05-08, the project needs either confirmed credentials in place or submitted procurement with owner, provider, expected issuance date, and fallback release label. Default owner for account/provider decisions is Sebastian. Default owner for CI wiring and evidence capture is the release engineer executing this sprint.

## Owner matrix

| Workstream | Owner | Day-2 evidence |
|---|---|---|
| Apple Developer account/certs/notary | Sebastian | account/team/cert/notary status recorded. |
| Windows EV code-signing procurement | Sebastian | provider, order/submission status, expected issuance date recorded. |
| CI signing integration | Release engineer | secret names, protected environment, and dry-run plan recorded. |
| Update signing key | Release engineer | public key path, private-key storage decision, and rotation plan recorded. |
| VM smoke access | Release engineer | available macOS/Ubuntu/Windows/Fedora VM paths recorded. |

## Required assets

| Asset | Required for | Required status before public GA |
|---|---|---|
| Apple Developer Program access | macOS signing/notarization | Active team access confirmed. |
| Developer ID Application certificate | macOS binaries | Installed in CI signing path or signing machine. |
| Developer ID Installer certificate | macOS `.pkg` | Installed in CI signing path or signing machine. |
| Notarytool credentials | macOS notarization | Stored in CI secret store with correct Team ID. |
| Hardened runtime entitlement list | macOS runtime | Documented and verified with `codesign --verify --strict`. |
| Windows EV code-signing certificate | Windows MSI/binaries | Issued and usable from CI or signing host. EV is required for Windows GA unless SmartScreen evidence proves another path safe. |
| Windows timestamp authority | Windows signatures | Configured in signing command. |
| Update manifest signing key | Update system | Public key embedded in clients; private key protected. |
| GitHub protected release environment | Release signing | Secrets scoped to tag/release workflow with approval gate. |

## Preferred storage model

| Secret | Storage |
|---|---|
| Apple notary credentials | GitHub Actions protected environment secret or 1Password-backed CI injection. |
| Apple signing cert private key | GitHub Actions protected environment secret, encrypted P12, or dedicated signing machine. |
| Windows signing cert | Hardware token/HSM if provider requires it; otherwise protected CI secret or dedicated signing host. |
| Update signing private key | Protected GitHub Environment secret for automated releases or hardware-backed signer for manual releases. |

Private keys must not be committed to git, stored in support bundles, or placed in developer docs.

## Update signing decision

Use Ed25519 signatures for update manifests unless implementation evidence forces a different primitive.

Manifest security fields:

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

Clients must reject unsigned manifests, unknown keys, hash mismatch, revoked artifacts, and channel downgrade.

## macOS gates

Public macOS GA needs transcripts for:

```bash
codesign --verify --strict --verbose=2 <binary-or-app>
spctl --assess --type install <pkg-or-dmg>
xcrun notarytool submit <artifact> --wait
xcrun stapler validate <artifact>
```

If Network Extension entitlement becomes necessary for tunnel implementation, open Apple entitlement request immediately and mark macOS GA blocked until approved.

## Windows gates

Public Windows GA needs transcripts for:

```powershell
Get-AuthenticodeSignature .\tytus.exe
Get-AuthenticodeSignature .\tytus-tray.exe
Get-AuthenticodeSignature .\tytus-mcp.exe
Get-AuthenticodeSignature .\TytusSetup.msi
```

The MSI must install without unsigned-publisher wall. If SmartScreen blocks normal install despite signatures, Windows remains private beta until certificate/reputation path is corrected.

## Linux gates

Public Ubuntu GA needs:

- signed `.deb` or repository metadata signature
- SHA256 published on download/update path
- `install.sh` verifies signature/hash before install
- package starts user service/tray and launches Tytus

Fedora `.rpm` starts as beta unless smoke proves tray, SELinux/AppArmor, tunnel, update, and uninstall behavior.

## Procurement log template

Record each procurement event in the sprint verdicts folder and release notes:

```text
Date:
Asset:
Owner:
Provider/account:
Status:
Expected issuance date:
Blocking platforms:
Fallback release label if blocked:
Evidence path:
```

## Fallback labels

| Blocked asset | Release label |
|---|---|
| Apple signing/notary unavailable | macOS private beta only. |
| Windows signing unavailable | Windows private beta only. |
| SmartScreen blocks Windows artifact | Windows private beta only. |
| Update signing unavailable | No public production release. |
| Linux package signing unavailable | Linux beta or developer preview only. |

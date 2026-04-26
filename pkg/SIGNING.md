# Signing + notarizing the Tytus.pkg installer

`pkg/build-pkg.sh` produces an **unsigned** `target/Tytus-<version>-unsigned.pkg`.
This document covers the signed-installer path: Apple Developer ID,
notarization, stapling, verification, and upload.

## Prerequisites

- **Xcode Command Line Tools** (`xcode-select --install`). Provides
  `pkgbuild`, `productbuild`, `productsign`, `xcrun notarytool`,
  `xcrun stapler`, and `spctl`.
- **Developer ID Installer** certificate from
  https://developer.apple.com (NOT "Developer ID Application" — that's
  for code signing binaries, not pkg installers). Keep it in your
  login keychain.
- **App-specific password** for your Apple ID, generated at
  https://appleid.apple.com → Sign-In and Security → App-Specific
  Passwords.
- **Team ID** from https://developer.apple.com/account → Membership.

## One-time setup — store notarization credentials

Run once; the `notarytool` keychain profile persists across sessions:

```bash
xcrun notarytool store-credentials "tytus-notary" \
    --apple-id "your-apple-id@example.com" \
    --team-id "TEAMIDXXXX" \
    --password "abcd-efgh-ijkl-mnop"     # the app-specific password
```

After this, all future `notarytool submit` calls reference
`--keychain-profile tytus-notary` instead of passing creds inline.

## Build → sign → notarize → staple

From the repo root:

```bash
# 0. Build binaries (if not already built).
cargo build --release

# 1. Build the unsigned .pkg
./pkg/build-pkg.sh
# → target/Tytus-<version>-unsigned.pkg

# 2. Sign it. Replace <Your Name> + <Team> with your cert's Common Name
#    (find via: security find-identity -v -p basic | grep "Developer ID Installer")
productsign \
    --sign "Developer ID Installer: <Your Name> (<TEAMIDXXXX>)" \
    "target/Tytus-<version>-unsigned.pkg" \
    "target/Tytus-<version>.pkg"

# 3. Notarize. ~5-10 minutes round-trip; --wait blocks until Apple finishes.
xcrun notarytool submit "target/Tytus-<version>.pkg" \
    --keychain-profile "tytus-notary" \
    --wait

# Expected output: "status: Accepted". On Rejected, check the log:
#   xcrun notarytool log <submission-id> --keychain-profile tytus-notary

# 4. Staple — embeds the notarization ticket so installs work offline.
xcrun stapler staple "target/Tytus-<version>.pkg"

# 5. Verify — Gatekeeper will accept this on any clean Mac.
spctl -a -t install -vv "target/Tytus-<version>.pkg"
# Expected output ends with: source=Notarized Developer ID
```

## Distribute

```bash
# Upload to the GitHub release artifacts.
gh release upload "v<version>" "target/Tytus-<version>.pkg"

# Mirror to https://tytus.traylinx.com/Tytus.pkg per your hosting setup.
# Common patterns: rsync to a static-site bucket, S3 sync, or a redirect
# rule from tytus.traylinx.com/Tytus.pkg to the GitHub release artifact.
```

## What the .pkg does on install

1. Drops `tytus`, `tytus-tray`, `tytus-mcp` into `/usr/local/bin/`.
2. Runs `pkg/scripts/postinstall` (as root) which:
   - Detects the GUI-logged-in user via `stat -f "%Su" /dev/console`.
   - Invokes `sudo -u <user> -H tytus tray install` so the `.app`
     bundle + `~/Library/LaunchAgents/com.traylinx.tytus.tray.plist`
     land in the user's home (not `/var/root/`).
   - Drops `/etc/sudoers.d/tytus` with the same tightly-scoped
     `wg-quick` / `route` / `ifconfig` exemptions `install.sh` uses,
     so the user never has to type their password to bring the
     tunnel up.

After install:
- Menu-bar **T** appears (Tytus.app launches automatically).
- First click on T opens Tower with the 4-step welcome wizard
  (Phase G).
- 60 seconds end-to-end from .pkg double-click to first chat.

## Re-running

`./pkg/build-pkg.sh` is idempotent — clears `target/pkg-build/`
before rebuilding. Bump the workspace version in `Cargo.toml`,
`cargo build --release`, then re-run the build → sign → notarize →
staple → upload pipeline.

## Troubleshooting

| Symptom | Fix |
|---|---|
| `productsign` fails: `unable to find signing identity` | Cert not imported. Open the `.cer` file or use Xcode → Settings → Accounts → Manage Certificates. |
| `notarytool` fails: `unauthenticated` | App-specific password rotated or credentials profile stale. Re-run `notarytool store-credentials`. |
| `notarytool` returns `Invalid` | Run `notarytool log <id> --keychain-profile tytus-notary` to see specifics. Most common: a binary inside the payload isn't signed (see "Code-signing the binaries" below). |
| Gatekeeper still blocks the install | Stapling didn't run. `xcrun stapler staple <pkg>` then re-`spctl`. |

## Optional: code-signing the binaries

Apple's notarization checks every Mach-O binary inside the payload.
If you're seeing notarization rejection on `tytus`/`tytus-tray`/
`tytus-mcp`, sign them before building the .pkg:

```bash
# Use "Developer ID Application" cert (different from "Installer").
codesign --force \
    --options runtime \
    --timestamp \
    --sign "Developer ID Application: <Your Name> (<TEAMIDXXXX>)" \
    target/release/tytus \
    target/release/tytus-tray \
    target/release/tytus-mcp

# Then re-run pkg/build-pkg.sh
```

Notarization tolerates unsigned binaries in some categories, but a
fully-signed payload is bulletproof and futureproofs against Apple
tightening the rules.

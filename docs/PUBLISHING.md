# Publishing Tytus CLI

How the one-liner install flow works end-to-end, and what to do for each release.

## The public surface

```
https://get.traylinx.com/                  ← landing page (web/index.html)
https://get.traylinx.com/install.sh        ← macOS + Linux installer
https://get.traylinx.com/install.ps1       ← Windows installer (experimental)
https://github.com/traylinx/tytus-cli        ← source + issues
https://github.com/traylinx/tytus-cli/releases ← prebuilt binaries + SHA256SUMS
https://github.com/traylinx/homebrew-tap     ← brew install traylinx/tap/tytus
```

## One-liners users see

```bash
# macOS / Linux — stable/latest production release
curl -fsSL https://get.traylinx.com/install.sh | bash

# macOS / Linux — explicit public beta/pre-release
curl -fsSL https://get.traylinx.com/install.sh | TYTUS_RELEASE_TAG=v0.6.14-beta.35 sh

# Windows — stable/latest production release
powershell -c "irm https://get.traylinx.com/install.ps1 | iex"

# Windows — explicit public beta/pre-release
$env:TYTUS_RELEASE_TAG="v0.6.14-beta.35"; irm https://get.traylinx.com/install.ps1 | iex

# Homebrew
brew install traylinx/tap/tytus
```

## How it's wired

```
tytus-cli repo
├── install.sh               ← canonical install script
├── install.ps1              ← canonical Windows installer
├── web/
│   ├── index.html           ← landing page
│   ├── install.sh  -> ../install.sh   (git symlink)
│   ├── install.ps1 -> ../install.ps1  (git symlink)
│   ├── _headers             ← Content-Type + security headers
│   └── _redirects           ← CF routing (currently just comments)
├── .github/workflows/
│   ├── release.yml          ← builds binaries + SHA256SUMS on tag push
│   └── homebrew.yml         ← updates homebrew-tap on release publish
└── contrib/homebrew/
    └── tytus.rb             ← formula template with {{VERSION}}, {{SHA_*}}

Cloudflare Pages/Workers Static Assets project
├── Watches: main branch of tytus-cli
├── Build command: (none — static serve)
├── Root directory: web
└── Serves install.sh / install.ps1 / index.html directly from web/

traylinx/homebrew-tap repo (separate)
└── Formula/tytus.rb    ← auto-updated on each release
```

## Cutting a release

```bash
# 1. Bump version in cli/Cargo.toml and mcp/Cargo.toml
vim cli/Cargo.toml mcp/Cargo.toml

# 2. Commit + tag
git add -A && git commit -m "release: v0.3.0"
git tag v0.3.0
git push origin main v0.3.0

# 3. Dry-run release.yml first (no public release)
gh workflow run release.yml \
  -f tag=v0.3.0 \
  -f tytus_os_ref=<immutable-tytus-os-sha-or-tag> \
  -f release_tier=internal-dry-run \
  -f publish_release=false

# 4. For trusted tester/private beta only, publish a prerelease
gh workflow run release.yml \
  -f tag=v0.3.0 \
  -f tytus_os_ref=<immutable-tytus-os-sha-or-tag> \
  -f release_tier=private-beta \
  -f publish_release=true

# 5. Wait for release.yml to finish (~10 min)
#    → builds/tests one canonical TytusOS web dist, uploads app/dist + manifest,
#      and verifies the same bytes before embedding them everywhere
#    → builds macos-{x86_64,aarch64}, linux-x86_64, windows-x86_64
#    → uploads tar/zip assets and SHA256SUMS
#    → for private-beta only, may publish PUBLIC-BETA-UNSIGNED macOS pkg
#      and Linux deb assets; production remains fail-closed until signing lands
#    → publishes GitHub prerelease only if publish_release=true

# 6. homebrew.yml fires automatically on release:published
#    → renders formula with real SHAs
#    → pushes to traylinx/homebrew-tap
#    → brew install traylinx/tap/tytus immediately gets the new version

# 7. Cloudflare Pages rebuilds on main push (step 2)
#    → new install.sh propagates in ~30 seconds
```

Production release tier is intentionally fail-closed until signed/notarized
package publishing is implemented and the protected `release-production`
environment secrets in `docs/release/signing-secret-contract.md` are present.
Internal dry-runs and private beta runs do not request production signing
secrets or environment approval.

## One-time setup required

### 1. Cloudflare Pages project

1. Go to Cloudflare dashboard → Compute (formerly Workers & Pages) → Create → Pages tab → Connect to Git
2. Select repo: `traylinx/tytus-cli`
3. Production branch: `main`
4. Build command: (leave empty)
5. Build output directory: `web`
6. Save
7. Custom domain: add `get.traylinx.com` or similar (Cloudflare auto-creates the CNAME)

### 2. Homebrew tap repo

1. Create empty public repo: `traylinx/homebrew-tap`
2. Create a PAT on an account that has push access to the tap repo
   - Scope: `repo`
   - No expiration (or rotate annually)
3. On `traylinx/tytus-cli`, add repo secret `HOMEBREW_TAP_TOKEN` with the PAT
4. Done — next release auto-publishes the formula

### 3. Apple code signing (DEFERRED — until first paying customers)

- Apple Developer Program membership: $99/yr
- Without it, macOS Gatekeeper shows "unidentified developer"
- Users can still install (Right-click → Open → Allow), but it adds friction
- When we have customers, add a signing step to release.yml using
  `codesign --sign "Developer ID Application: Traylinx" ...`


## Public beta with unsigned installers

Sebastian explicitly allowed unsigned native installers for public beta while Apple/Windows/Linux signing is deferred. This does **not** change the GA bar.

Use `release_tier=private-beta` and `publish_release=true`. The workflow may publish macOS `.pkg` and Linux `.deb` assets only when their filenames contain `PUBLIC-BETA-UNSIGNED`; production still fails closed. Release copy must say public beta / technical preview and warn about Gatekeeper/SmartScreen trust prompts. The public download page may point at that tag with `TYTUS_RELEASE_TAG=<tag>` so installers fetch the pre-release via `releases/tags/<tag>` instead of `/releases/latest`.

Never use this path for `release_tier=production`.

## Security posture before broad promotion

- [x] Checksum verification in install.sh (C1)
- [x] SHA256SUMS emitted by release.yml (C1)
- [x] Sudoers wildcard tightened to `/tmp/tytus/tunnel-*.json` (H3)
- [x] Public beta artifacts explicitly labeled `PUBLIC-BETA-UNSIGNED`
- [x] Public download copy says public beta / technical preview / not production GA
- [ ] Apple Developer ID signing + notarization for GA
- [ ] Windows MSI/signing/driver packaging for GA
- [ ] Linux package or repository signing for GA
- [ ] **E1:** remove `/bin/kill -TERM *` from existing dev sudoers — local fix
- [ ] **H1:** decide if hardcoded API key `2qQaEiyjeqd0F141C6cFeqpJ353Y7USl` is
      a secret or a public client ID — document or rotate
- [ ] **H2:** move `refresh_token` out of state.json into keychain exclusively
- [ ] **H5:** update MCP `tytus_env` to return stable values by default
- [ ] Infrastructure: close SSH on droplet public IP, block `/metrics`

The public beta page may be shared now because the release is explicitly
pre-GA, checksum-verified, and unsigned artifacts are visibly labeled. Do not
call this GA or promote it as production-ready until the signing and fresh-VM
smoke gates pass.

## Soft-launch channels (when ready)

1. Hacker News — "Show HN: Tytus — private AI pod, one terminal away"
2. r/LocalLLaMA — emphasize the OpenAI-compat gateway
3. r/commandline — emphasize the MCP + `tytus link` story
4. Twitter/X — short video of `tytus setup` → `tytus chat`
5. The Claude Code, OpenCode, Cursor Discord channels

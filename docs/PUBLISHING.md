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
# macOS / Linux
curl -fsSL https://get.traylinx.com/install.sh | bash

# Windows
powershell -c "irm https://get.traylinx.com/install.ps1 | iex"

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

# 3. Wait for release.yml to finish (~10 min)
#    → builds macos-{x86_64,aarch64}, linux-{x86_64,aarch64}
#    → generates SHA256SUMS
#    → publishes GitHub release

# 4. homebrew.yml fires automatically on release:published
#    → renders formula with real SHAs
#    → pushes to traylinx/homebrew-tap
#    → brew install traylinx/tap/tytus immediately gets the new version

# 5. Cloudflare Pages rebuilds on main push (step 2)
#    → new install.sh propagates in ~30 seconds
```

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

## Security checks before posting the one-liner publicly

- [x] Checksum verification in install.sh (C1)
- [x] SHA256SUMS emitted by release.yml (C1)
- [x] Sudoers wildcard tightened to `/tmp/tytus/tunnel-*.json` (H3)
- [ ] **E1:** remove `/bin/kill -TERM *` from existing dev sudoers — local fix
- [ ] **H1:** decide if hardcoded API key `2qQaEiyjeqd0F141C6cFeqpJ353Y7USl` is
      a secret or a public client ID — document or rotate
- [ ] **H2:** move `refresh_token` out of state.json into keychain exclusively
- [ ] **H5:** update MCP `tytus_env` to return stable values by default
- [ ] Infrastructure: close SSH on droplet public IP, block `/metrics`

Until the unchecked items are handled, the install one-liner should only be
shared with trusted testers, not posted publicly.

## Soft-launch channels (when ready)

1. Hacker News — "Show HN: Tytus — private AI pod, one terminal away"
2. r/LocalLLaMA — emphasize the OpenAI-compat gateway
3. r/commandline — emphasize the MCP + `tytus link` story
4. Twitter/X — short video of `tytus setup` → `tytus chat`
5. The Claude Code, OpenCode, Cursor Discord channels

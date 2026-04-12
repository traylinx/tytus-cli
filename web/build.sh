#!/bin/sh
# Cloudflare Pages build step.
#
# Cloudflare Pages project settings:
#   Build command:   sh web/build.sh
#   Build output:    web/dist
#   Root directory:  (leave empty)
#
# This copies the install scripts and the static landing page into web/dist
# so they are served directly from Cloudflare's edge at:
#
#   https://tytus.traylinx.com/install.sh
#   https://tytus.traylinx.com/install.ps1
#   https://tytus.traylinx.com/
#
# Serving them directly (rather than 302-redirecting to raw.githubusercontent.com)
# means:
#   - the final URL shown in `curl -v` stays on our domain
#   - we bypass GitHub's anonymous rate limit on raw.githubusercontent.com
#   - users get a consistent edge-cached fetch path
#
# Every push to main rebuilds, so install.sh changes propagate in seconds.

set -eu

cd "$(dirname "$0")/.."   # repo root

mkdir -p web/dist

# Static landing page
cp web/index.html   web/dist/index.html
cp web/_redirects   web/dist/_redirects 2>/dev/null || true

# Install scripts (mastered at repo root)
cp install.sh       web/dist/install.sh
cp install.ps1      web/dist/install.ps1

# Harden content type headers
cat > web/dist/_headers <<'EOF'
/install.sh
  Content-Type: text/x-shellscript; charset=utf-8
  Cache-Control: public, max-age=300
  X-Content-Type-Options: nosniff

/install.ps1
  Content-Type: text/plain; charset=utf-8
  Cache-Control: public, max-age=300
  X-Content-Type-Options: nosniff

/*
  X-Frame-Options: DENY
  Referrer-Policy: no-referrer
  Strict-Transport-Security: max-age=31536000; includeSubDomains
EOF

echo "Build output:"
ls -la web/dist/

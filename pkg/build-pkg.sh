#!/usr/bin/env bash
# ============================================================
# Build an unsigned Tytus.pkg from target/release/ binaries.
# ============================================================
# Output: target/Tytus-<version>-unsigned.pkg
#
# After this script: sign + notarize + staple per pkg/SIGNING.md.
# Builds in ~10 seconds. Re-runnable; idempotent.
#
# Prereqs:
#   cargo build --release   # produces target/release/{tytus,tytus-tray,tytus-mcp}
#   pkgbuild + productbuild (Xcode CLT, included on every Mac)
#
# Usage:
#   ./pkg/build-pkg.sh                    # auto-detect version from Cargo.toml
#   VERSION=0.6.0 ./pkg/build-pkg.sh      # override version
# ============================================================

set -euo pipefail

# Resolve repo root regardless of where the script is called from.
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "$SCRIPT_DIR/.." && pwd)
cd "$REPO_ROOT"

VERSION=${VERSION:-$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)}
PKG_ID="com.traylinx.tytus"
BUILD_DIR="$REPO_ROOT/target/pkg-build"
PAYLOAD="$BUILD_DIR/payload"
SCRIPTS="$BUILD_DIR/scripts"
COMPONENT_PKG="$BUILD_DIR/Tytus-component.pkg"
PRODUCT_PKG="$REPO_ROOT/target/Tytus-${VERSION}-unsigned.pkg"

# ── Sanity ───────────────────────────────────────────────────
for b in tytus tytus-tray tytus-mcp; do
    if [ ! -x "target/release/$b" ]; then
        echo "ERROR: target/release/$b missing or not executable." >&2
        echo "       Run: cargo build --release" >&2
        exit 1
    fi
done

command -v pkgbuild >/dev/null || { echo "ERROR: pkgbuild not found (Xcode CLT required)" >&2; exit 1; }
command -v productbuild >/dev/null || { echo "ERROR: productbuild not found (Xcode CLT required)" >&2; exit 1; }

# ── Layout ───────────────────────────────────────────────────
rm -rf "$BUILD_DIR"
mkdir -p "$PAYLOAD/usr/local/bin" "$SCRIPTS"

for b in tytus tytus-tray tytus-mcp; do
    cp "target/release/$b" "$PAYLOAD/usr/local/bin/$b"
    chmod 0755 "$PAYLOAD/usr/local/bin/$b"
    # Strip extended attributes (Finder tags, com.apple.quarantine, etc.)
    # so the payload doesn't ship `._tytus` AppleDouble resource forks
    # — those bloat the .pkg and confuse some unzip tools.
    xattr -c "$PAYLOAD/usr/local/bin/$b" 2>/dev/null || true
done

cp "$SCRIPT_DIR/scripts/postinstall" "$SCRIPTS/postinstall"
chmod 0755 "$SCRIPTS/postinstall"

# ── Component pkg (the payload) ──────────────────────────────
pkgbuild \
    --root "$PAYLOAD" \
    --identifier "$PKG_ID" \
    --version "$VERSION" \
    --scripts "$SCRIPTS" \
    --install-location "/" \
    "$COMPONENT_PKG"

# ── Distribution xml ─────────────────────────────────────────
cat > "$BUILD_DIR/distribution.xml" <<XML
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="1">
  <title>Tytus</title>
  <organization>com.traylinx</organization>
  <domains enable_localSystem="true"/>
  <options customize="never" require-scripts="false" rootVolumeOnly="false" hostArchitectures="x86_64,arm64"/>
  <welcome file="welcome.html" mime-type="text/html"/>
  <conclusion file="conclusion.html" mime-type="text/html"/>
  <choices-outline>
    <line choice="default">
      <line choice="$PKG_ID"/>
    </line>
  </choices-outline>
  <choice id="default"/>
  <choice id="$PKG_ID" visible="false">
    <pkg-ref id="$PKG_ID"/>
  </choice>
  <pkg-ref id="$PKG_ID" version="$VERSION" onConclusion="none">Tytus-component.pkg</pkg-ref>
</installer-gui-script>
XML

# Welcome + conclusion screens (plain HTML — Installer.app renders them).
cat > "$BUILD_DIR/welcome.html" <<'HTML'
<html><body style="font-family:-apple-system,sans-serif;padding:20px;color:#1a1a1a">
<h2>Welcome to Tytus</h2>
<p>This installer puts the <code>tytus</code> CLI, menu-bar app, and MCP
server on your Mac. About 60 seconds end-to-end.</p>
<p>After install, click the <strong>T</strong> in your menu bar to start chatting
with your private AI.</p>
</body></html>
HTML

cat > "$BUILD_DIR/conclusion.html" <<'HTML'
<html><body style="font-family:-apple-system,sans-serif;padding:20px;color:#1a1a1a">
<h2>Tytus is installed.</h2>
<p>Look for the <strong>T</strong> icon in your menu bar (top-right of the screen).</p>
<p>Click it to sign in and pick your AI assistant — under 60 seconds.</p>
<p>If the icon doesn't appear: open <strong>Tytus</strong> from Applications,
or run <code>tytus tray install</code> in Terminal.</p>
</body></html>
HTML

# ── Distribution pkg (the user-facing artifact) ──────────────
productbuild \
    --distribution "$BUILD_DIR/distribution.xml" \
    --resources "$BUILD_DIR" \
    --package-path "$BUILD_DIR" \
    "$PRODUCT_PKG"

SIZE=$(du -h "$PRODUCT_PKG" | awk '{print $1}')

cat <<DONE

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ✓ Built  $PRODUCT_PKG  ($SIZE, unsigned)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Next steps (see pkg/SIGNING.md for full reference):

  1. SIGN
       productsign --sign "Developer ID Installer: <Your Name> (<Team>)" \\
           "$PRODUCT_PKG" "target/Tytus-${VERSION}.pkg"

  2. NOTARIZE
       xcrun notarytool submit "target/Tytus-${VERSION}.pkg" \\
           --keychain-profile "tytus-notary" --wait

  3. STAPLE
       xcrun stapler staple "target/Tytus-${VERSION}.pkg"

  4. VERIFY
       spctl -a -t install -vv "target/Tytus-${VERSION}.pkg"

  5. UPLOAD
       gh release upload v${VERSION} "target/Tytus-${VERSION}.pkg"

  6. MIRROR
       Copy/host as https://tytus.traylinx.com/Tytus.pkg per your hosting setup.

DONE

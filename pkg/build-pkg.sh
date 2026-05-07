#!/usr/bin/env bash
# ============================================================
# Build an unsigned Tytus.pkg from release binaries.
# ============================================================
# Output:
#   target/Tytus-<version>-unsigned.pkg
#   target/Tytus-<version>-<target-triple>-unsigned.pkg when TARGET_TRIPLE is set
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
#   TARGET_TRIPLE=x86_64-apple-darwin ./pkg/build-pkg.sh
#   BIN_DIR=target/release TARGET_TRIPLE=x86_64-apple-darwin ./pkg/build-pkg.sh
# ============================================================

set -euo pipefail

# Best-effort AppleDouble/resource-fork suppression. Some macOS versions still
# preserve system provenance metadata in pkg payloads; signing/notarization
# gates remain the final authority for public installer artifacts.
export COPYFILE_DISABLE=1

# Resolve repo root regardless of where the script is called from.
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "$SCRIPT_DIR/.." && pwd)
cd "$REPO_ROOT"

VERSION=${VERSION:-$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)}
TARGET_TRIPLE=${TARGET_TRIPLE:-}
if [ -n "$TARGET_TRIPLE" ]; then
    BIN_DIR=${BIN_DIR:-"$REPO_ROOT/target/$TARGET_TRIPLE/release"}
    OUTPUT_SUFFIX="-$TARGET_TRIPLE"
    case "$TARGET_TRIPLE" in
        x86_64-apple-darwin) PKG_HOST_ARCHITECTURES="x86_64" ;;
        aarch64-apple-darwin) PKG_HOST_ARCHITECTURES="arm64" ;;
        *)
            echo "ERROR: unsupported macOS package target: $TARGET_TRIPLE" >&2
            echo "       Expected x86_64-apple-darwin or aarch64-apple-darwin." >&2
            exit 1
            ;;
    esac
else
    BIN_DIR=${BIN_DIR:-"$REPO_ROOT/target/release"}
    OUTPUT_SUFFIX=""
    PKG_HOST_ARCHITECTURES="x86_64,arm64"
fi
PKG_ID="com.traylinx.tytus"
BUILD_DIR="$REPO_ROOT/target/pkg-build"
PAYLOAD="$BUILD_DIR/payload"
SCRIPTS="$BUILD_DIR/scripts"
COMPONENT_PKG="$BUILD_DIR/Tytus-component.pkg"
PRODUCT_PKG="$REPO_ROOT/target/Tytus-${VERSION}${OUTPUT_SUFFIX}-unsigned.pkg"

# ── Sanity ───────────────────────────────────────────────────
for b in tytus tytus-tray tytus-mcp; do
    if [ ! -x "$BIN_DIR/$b" ]; then
        echo "ERROR: $BIN_DIR/$b missing or not executable." >&2
        if [ -n "$TARGET_TRIPLE" ]; then
            echo "       Run: cargo build --release --target $TARGET_TRIPLE" >&2
        else
            echo "       Run: cargo build --release" >&2
        fi
        exit 1
    fi
done

command -v pkgbuild >/dev/null || { echo "ERROR: pkgbuild not found (Xcode CLT required)" >&2; exit 1; }
command -v productbuild >/dev/null || { echo "ERROR: productbuild not found (Xcode CLT required)" >&2; exit 1; }

# ── Layout ───────────────────────────────────────────────────
rm -rf "$BUILD_DIR"
mkdir -p "$PAYLOAD/usr/local/bin" "$SCRIPTS"

for b in tytus tytus-tray tytus-mcp; do
    cp "$BIN_DIR/$b" "$PAYLOAD/usr/local/bin/$b"
    chmod 0755 "$PAYLOAD/usr/local/bin/$b"
    # Strip removable extended attributes (Finder tags, quarantine, etc.).
    # macOS system provenance may be non-removable on some hosts.
    xattr -c "$PAYLOAD/usr/local/bin/$b" 2>/dev/null || true
done
find "$PAYLOAD" -name '._*' -delete

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
  <options customize="never" require-scripts="false" rootVolumeOnly="false" hostArchitectures="$PKG_HOST_ARCHITECTURES"/>
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
           "$PRODUCT_PKG" "target/Tytus-${VERSION}${OUTPUT_SUFFIX}.pkg"

  2. NOTARIZE
       xcrun notarytool submit "target/Tytus-${VERSION}${OUTPUT_SUFFIX}.pkg" \\
           --keychain-profile "tytus-notary" --wait

  3. STAPLE
       xcrun stapler staple "target/Tytus-${VERSION}${OUTPUT_SUFFIX}.pkg"

  4. VERIFY
       spctl -a -t install -vv "target/Tytus-${VERSION}${OUTPUT_SUFFIX}.pkg"

  5. UPLOAD
       gh release upload v${VERSION} "target/Tytus-${VERSION}${OUTPUT_SUFFIX}.pkg"

  6. MIRROR
       Copy/host as https://tytus.traylinx.com/Tytus.pkg per your hosting setup.

DONE

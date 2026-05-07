#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_TRIPLE="${TARGET_TRIPLE:-x86_64-unknown-linux-gnu}"
BIN_DIR="${BIN_DIR:-$ROOT/target/$TARGET_TRIPLE/release}"
OUT_DIR="${OUT_DIR:-$ROOT/target}"
VERSION="${VERSION:-$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)}"
PACKAGE_NAME="tytus"

case "$TARGET_TRIPLE" in
    x86_64-unknown-linux-gnu) DEB_ARCH="amd64" ;;
    aarch64-unknown-linux-gnu) DEB_ARCH="arm64" ;;
    *)
        echo "Unsupported Debian target triple: $TARGET_TRIPLE" >&2
        exit 1
        ;;
esac

for bin in tytus tytus-mcp tytus-tray; do
    if [ ! -x "$BIN_DIR/$bin" ]; then
        echo "Missing executable: $BIN_DIR/$bin" >&2
        echo "Build first: cargo build --release --target $TARGET_TRIPLE -p atomek-cli -p tytus-mcp -p tytus-tray" >&2
        exit 1
    fi
done

if ! command -v dpkg-deb >/dev/null 2>&1; then
    echo "dpkg-deb is required to build Debian packages." >&2
    exit 1
fi

PKGROOT="$OUT_DIR/deb/${PACKAGE_NAME}_${VERSION}_${DEB_ARCH}"
DEB_OUT="$OUT_DIR/Tytus-${VERSION}-${TARGET_TRIPLE}-unsigned.deb"
rm -rf "$PKGROOT"
mkdir -p \
    "$PKGROOT/DEBIAN" \
    "$PKGROOT/usr/bin" \
    "$PKGROOT/usr/lib/systemd/user" \
    "$PKGROOT/etc/xdg/autostart" \
    "$PKGROOT/usr/share/applications" \
    "$PKGROOT/usr/share/icons/hicolor/512x512/apps" \
    "$PKGROOT/usr/share/doc/$PACKAGE_NAME"

install -m 0755 "$BIN_DIR/tytus" "$PKGROOT/usr/bin/tytus"
install -m 0755 "$BIN_DIR/tytus-mcp" "$PKGROOT/usr/bin/tytus-mcp"
install -m 0755 "$BIN_DIR/tytus-tray" "$PKGROOT/usr/bin/tytus-tray"
install -m 0644 "$ROOT/pkg/deb/systemd/tytus-daemon.service" "$PKGROOT/usr/lib/systemd/user/tytus-daemon.service"
install -m 0644 "$ROOT/pkg/deb/applications/tytus.desktop" "$PKGROOT/usr/share/applications/tytus.desktop"
install -m 0644 "$ROOT/pkg/deb/autostart/tytus-tray.desktop" "$PKGROOT/etc/xdg/autostart/tytus-tray.desktop"
install -m 0644 "$ROOT/tray/web/os/brand/tytusos-mark-512.png" "$PKGROOT/usr/share/icons/hicolor/512x512/apps/tytus.png"
install -m 0755 "$ROOT/pkg/deb/scripts/postinst" "$PKGROOT/DEBIAN/postinst"
install -m 0755 "$ROOT/pkg/deb/scripts/postrm" "$PKGROOT/DEBIAN/postrm"

cat > "$PKGROOT/DEBIAN/control" <<CONTROL
Package: $PACKAGE_NAME
Version: $VERSION
Section: net
Priority: optional
Architecture: $DEB_ARCH
Maintainer: Traylinx <hello@traylinx.com>
Homepage: https://traylinx.com
Depends: libc6 (>= 2.35), ca-certificates, libdbus-1-3, libsecret-1-0, libgtk-3-0, libayatana-appindicator3-1, libxdo3, iproute2, xdg-utils
Recommends: policykit-1 | polkitd
Description: Tytus private AI pod desktop and CLI
 Tytus installs the CLI, MCP server, and tray app used to open Tytus OS,
 authenticate, manage a private AI pod, and keep local desktop integration
 available on supported Linux desktops.
CONTROL

cat > "$PKGROOT/usr/share/doc/$PACKAGE_NAME/copyright" <<'COPYRIGHT'
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: tytus-cli
Source: https://github.com/traylinx/tytus-cli

Files: *
Copyright: Traylinx
License: MIT
COPYRIGHT

cat > "$PKGROOT/usr/share/doc/$PACKAGE_NAME/changelog.Debian" <<CHANGELOG
tytus ($VERSION) unstable; urgency=medium

  * Public beta unsigned Linux package preview.
  * Intended for technical-preview testers; not production GA.

 -- Traylinx <hello@traylinx.com>  Thu, 07 May 2026 00:00:00 +0000
CHANGELOG
gzip -n -9 "$PKGROOT/usr/share/doc/$PACKAGE_NAME/changelog.Debian"

# dpkg-deb uses root owner in the archive without requiring sudo.
dpkg-deb --build --root-owner-group "$PKGROOT" "$DEB_OUT"

echo "$DEB_OUT"

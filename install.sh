#!/bin/bash
# Tytus CLI installer
# Usage: curl -fsSL https://tytus.traylinx.com/install.sh | sh
set -e

REPO="traylinx/tytus-cli"
BINARY="tytus"
INSTALL_DIR="/usr/local/bin"

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "${OS}-${ARCH}" in
  darwin-x86_64)  ASSET="tytus-macos-x86_64.tar.gz" ;;
  darwin-arm64)   ASSET="tytus-macos-aarch64.tar.gz" ;;
  linux-x86_64)   ASSET="tytus-linux-x86_64.tar.gz" ;;
  *)
    echo "Unsupported platform: ${OS}-${ARCH}"
    echo "Build from source: cargo build --release -p atomek-cli"
    exit 1
    ;;
esac

# Get latest release URL
echo "Downloading tytus for ${OS}/${ARCH}..."
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep "browser_download_url.*${ASSET}" | cut -d'"' -f4)

if [ -z "$LATEST" ]; then
  echo "Error: Could not find release for ${ASSET}"
  echo "Check https://github.com/${REPO}/releases"
  exit 1
fi

# Download and install
TMP=$(mktemp -d)
curl -fsSL "$LATEST" -o "${TMP}/${ASSET}"
tar xzf "${TMP}/${ASSET}" -C "${TMP}"

if [ -w "$INSTALL_DIR" ]; then
  mv "${TMP}/${BINARY}" "${INSTALL_DIR}/"
else
  echo "Installing to ${INSTALL_DIR} (requires sudo)..."
  sudo mv "${TMP}/${BINARY}" "${INSTALL_DIR}/"
fi

chmod +x "${INSTALL_DIR}/${BINARY}"
rm -rf "$TMP"

echo ""
echo "✓ tytus installed to ${INSTALL_DIR}/${BINARY}"
echo ""
echo "Quick start:"
echo "  tytus login              # Authenticate (one-time)"
echo "  sudo tytus connect       # Connect to your AI pod"
echo "  tytus env --export       # Show connection vars"
echo ""
echo "Docs: https://github.com/${REPO}"

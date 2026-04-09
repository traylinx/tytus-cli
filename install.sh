#!/bin/bash
# Tytus CLI installer — installs both tytus and tytus-mcp (MCP server)
# Usage: curl -fsSL https://tytus.traylinx.com/install.sh | sh
set -e

REPO="traylinx/tytus-cli"
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
    echo "Build from source: cargo build --release -p atomek-cli -p tytus-mcp"
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

# Download and extract
TMP=$(mktemp -d)
curl -fsSL "$LATEST" -o "${TMP}/${ASSET}"
tar xzf "${TMP}/${ASSET}" -C "${TMP}"

# Install both binaries
install_bin() {
  local bin="$1"
  if [ -f "${TMP}/${bin}" ]; then
    if [ -w "$INSTALL_DIR" ]; then
      mv "${TMP}/${bin}" "${INSTALL_DIR}/"
    else
      sudo mv "${TMP}/${bin}" "${INSTALL_DIR}/"
    fi
    chmod +x "${INSTALL_DIR}/${bin}"
    echo "  + ${INSTALL_DIR}/${bin}"
  fi
}

echo "Installing..."
install_bin "tytus"
install_bin "tytus-mcp"
rm -rf "$TMP"

# ── Set up passwordless sudo for tunnel activation ──────────
# tytus connect needs root only for creating the TUN device.
# This sudoers entry allows 'tytus tunnel-up' to run without a password
# so users never have to type sudo themselves.
TYTUS_BIN="${INSTALL_DIR}/tytus"
SUDOERS_FILE="/etc/sudoers.d/tytus"
CURRENT_USER="${SUDO_USER:-$(whoami)}"

setup_sudoers() {
  local entry="${CURRENT_USER} ALL=(root) NOPASSWD: ${TYTUS_BIN} tunnel-up *"
  if [ -f "$SUDOERS_FILE" ] && grep -qF "$entry" "$SUDOERS_FILE" 2>/dev/null; then
    echo "  Passwordless tunnel: already configured"
    return
  fi
  echo "$entry" > "$SUDOERS_FILE" && chmod 440 "$SUDOERS_FILE"
  echo "  Passwordless tunnel: configured for ${CURRENT_USER}"
}

# We're likely running with sudo already (from install_bin), or can elevate
if [ "$(id -u)" = "0" ]; then
  setup_sudoers
elif command -v sudo >/dev/null 2>&1; then
  sudo bash -c "
    echo '${CURRENT_USER} ALL=(root) NOPASSWD: ${TYTUS_BIN} tunnel-up *' > ${SUDOERS_FILE} && chmod 440 ${SUDOERS_FILE}
  " 2>/dev/null && echo "  Passwordless tunnel: configured" || echo "  Note: run with sudo to enable passwordless tunnel activation"
fi

echo ""
echo "Installed:"
echo "  tytus      — CLI for pod management"
echo "  tytus-mcp  — MCP server for AI CLI integration"
echo ""
echo "Quick start:"
echo "  tytus login              # Authenticate (one-time)"
echo "  tytus connect            # Connect to your AI pod"
echo "  tytus env --export       # Show connection vars"
echo ""
echo "Infect any project (adds MCP + context files for all AI CLIs):"
echo "  cd your-project && tytus infect"
echo ""
echo "Docs: https://github.com/${REPO}"

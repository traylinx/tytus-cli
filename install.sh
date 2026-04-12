#!/bin/sh
# ============================================================
# tytus-cli installer — installs both tytus and tytus-mcp
# ============================================================
#
# Usage:
#     curl -fsSL https://get.traylinx.com/install.sh | bash
# (legacy direct-from-github URL also works as a fallback)
#
# What it does:
#   1. Detects your OS + arch
#   2. Downloads a prebuilt release from GitHub + verifies SHA256SUMS
#   3. Falls back to building from source via `cargo install --git`
#      (installs rust via rustup if needed, with consent)
#   4. Sets up a tightly-scoped sudoers entry so `tytus connect` never
#      prompts for a password when opening the WireGuard tunnel
#   5. Prints clear next steps
#
# Env:
#     TYTUS_INSTALL_DIR    Override the install directory (default: /usr/local/bin
#                          for releases, $HOME/.cargo/bin for source builds)
#     TYTUS_SKIP_SUDOERS   Set to "1" to skip sudoers configuration
#     TYTUS_FORCE_SOURCE   Set to "1" to skip the release download and go
#                          straight to cargo install --git
#     TYTUS_SKIP_CHECKSUM  Set to "1" to skip SHA256 verification (NOT RECOMMENDED)
# ============================================================

set -eu

REPO="traylinx/tytus-cli"
REPO_URL="https://github.com/${REPO}"
BRAND="Tytus"
CLI_NAME="tytus"
MCP_NAME="tytus-mcp"

# ── Colors ──────────────────────────────────────────────────
if [ -t 1 ] && command -v tput >/dev/null 2>&1 && [ "$(tput colors 2>/dev/null || echo 0)" -ge 8 ]; then
    BOLD=$(tput bold)
    DIM=$(tput dim)
    RED=$(tput setaf 1)
    GREEN=$(tput setaf 2)
    YELLOW=$(tput setaf 3)
    BLUE=$(tput setaf 4)
    RESET=$(tput sgr0)
else
    BOLD=""; DIM=""; RED=""; GREEN=""; YELLOW=""; BLUE=""; RESET=""
fi

msg()  { printf "%s==>%s %s\n" "$BLUE$BOLD" "$RESET$BOLD" "$1$RESET"; }
ok()   { printf " %s✓%s %s\n" "$GREEN" "$RESET" "$1"; }
warn() { printf " %s!%s %s\n" "$YELLOW" "$RESET" "$1" >&2; }
err()  { printf " %s✗%s %s\n" "$RED" "$RESET" "$1" >&2; }

banner() {
    printf "\n"
    printf "%s┌─────────────────────────────────────────────────┐%s\n" "$BOLD" "$RESET"
    printf "%s│          Installing %sTytus CLI%s                    │%s\n" "$BOLD" "$BLUE" "$RESET$BOLD" "$RESET"
    printf "%s│   %sPrivate AI pods driven from your terminal%s     │%s\n" "$BOLD" "$DIM" "$RESET$BOLD" "$RESET"
    printf "%s└─────────────────────────────────────────────────┘%s\n" "$BOLD" "$RESET"
    printf "\n"
}

# Read from /dev/tty so prompts work when piped from curl
read_reply() {
    _prompt="$1"
    _default="$2"
    printf "%s%s%s " "$YELLOW" "$_prompt" "$RESET"
    if [ -t 0 ]; then
        read -r _reply || _reply="$_default"
    elif [ -e /dev/tty ]; then
        read -r _reply </dev/tty || _reply="$_default"
    else
        _reply="$_default"
    fi
    printf "%s" "$_reply"
}

# ── Detection ───────────────────────────────────────────────

detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case "$OS" in
        darwin) OS_PRETTY="macOS" ;;
        linux) OS_PRETTY="Linux" ;;
        *)
            err "Unsupported OS: $OS. Tytus currently ships for macOS and Linux."
            err "Windows support is planned. Track at ${REPO_URL}/issues."
            exit 1
            ;;
    esac

    # Normalise arch
    case "$ARCH" in
        x86_64|amd64)  ARCH_NORM="x86_64" ;;
        arm64|aarch64) ARCH_NORM="aarch64" ;;
        *)
            warn "Architecture '$ARCH' has no prebuilt binary; will build from source."
            ARCH_NORM="$ARCH"
            ;;
    esac

    ok "Detected: ${OS_PRETTY} ${ARCH_NORM}"
}

# ── Try prebuilt release download ──────────────────────────

try_release_download() {
    [ "${TYTUS_FORCE_SOURCE:-}" = "1" ] && return 1

    RELEASE_ASSET=""
    case "${OS}-${ARCH_NORM}" in
        darwin-x86_64)  RELEASE_ASSET="tytus-macos-x86_64.tar.gz" ;;
        darwin-aarch64) RELEASE_ASSET="tytus-macos-aarch64.tar.gz" ;;
        linux-x86_64)   RELEASE_ASSET="tytus-linux-x86_64.tar.gz" ;;
        linux-aarch64)  RELEASE_ASSET="tytus-linux-aarch64.tar.gz" ;;
        *)              return 1 ;;
    esac

    msg "Looking for prebuilt release (${RELEASE_ASSET})..."
    RELEASES_JSON=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null)
    RELEASE_URL=$(printf "%s" "$RELEASES_JSON" \
        | grep "browser_download_url.*${RELEASE_ASSET}" \
        | cut -d'"' -f4 | head -1)
    SUMS_URL=$(printf "%s" "$RELEASES_JSON" \
        | grep "browser_download_url.*SHA256SUMS" \
        | cut -d'"' -f4 | head -1)

    if [ -z "$RELEASE_URL" ]; then
        warn "No prebuilt binary published yet for ${RELEASE_ASSET}. Falling back to source build."
        return 1
    fi

    ok "Found release: $RELEASE_URL"

    INSTALL_DIR="${TYTUS_INSTALL_DIR:-/usr/local/bin}"
    TMP=$(mktemp -d)
    trap 'rm -rf "$TMP"' EXIT

    msg "Downloading..."
    curl -fsSL "$RELEASE_URL" -o "${TMP}/${RELEASE_ASSET}"

    # ── SHA256 verification ────────────────────────────────
    # Guards against GitHub release tampering, CDN cache poisoning, and MITM.
    # See docs/PENTEST-RESULTS-2026-04-12.md finding C1.
    if [ "${TYTUS_SKIP_CHECKSUM:-}" = "1" ]; then
        warn "TYTUS_SKIP_CHECKSUM=1 — SKIPPING checksum verification. NOT RECOMMENDED."
    elif [ -z "$SUMS_URL" ]; then
        err "No SHA256SUMS found on this release — refusing to install unverified binary."
        err "If you're installing a pre-release and know what you're doing, set TYTUS_SKIP_CHECKSUM=1."
        err "Otherwise, report this at ${REPO_URL}/issues."
        exit 1
    else
        msg "Verifying SHA256..."
        curl -fsSL "$SUMS_URL" -o "${TMP}/SHA256SUMS"
        if command -v sha256sum >/dev/null 2>&1; then
            SHA_TOOL="sha256sum"
        elif command -v shasum >/dev/null 2>&1; then
            SHA_TOOL="shasum -a 256"
        else
            err "Neither sha256sum nor shasum found — cannot verify checksum."
            err "Install coreutils (Linux) or use macOS built-in shasum."
            exit 1
        fi
        EXPECTED=$(grep " ${RELEASE_ASSET}\$" "${TMP}/SHA256SUMS" | awk '{print $1}' | head -1)
        if [ -z "$EXPECTED" ]; then
            err "SHA256SUMS does not contain entry for ${RELEASE_ASSET}."
            exit 1
        fi
        ACTUAL=$(cd "${TMP}" && $SHA_TOOL "${RELEASE_ASSET}" | awk '{print $1}')
        if [ "$EXPECTED" != "$ACTUAL" ]; then
            err "CHECKSUM MISMATCH — refusing to install tampered binary."
            err "  expected: $EXPECTED"
            err "  got:      $ACTUAL"
            err "This is either a GitHub release tampering incident or a bug."
            err "Please report: ${REPO_URL}/issues"
            exit 1
        fi
        ok "Checksum verified"
    fi

    tar xzf "${TMP}/${RELEASE_ASSET}" -C "${TMP}"

    install_one() {
        _bin="$1"
        [ -f "${TMP}/${_bin}" ] || return 0
        if [ -w "$INSTALL_DIR" ]; then
            mv "${TMP}/${_bin}" "${INSTALL_DIR}/"
        else
            sudo mv "${TMP}/${_bin}" "${INSTALL_DIR}/"
        fi
        chmod +x "${INSTALL_DIR}/${_bin}"
        ok "${INSTALL_DIR}/${_bin}"
    }
    msg "Installing to ${INSTALL_DIR}..."
    install_one "${CLI_NAME}"
    install_one "${MCP_NAME}"

    BIN_PATH="${INSTALL_DIR}/${CLI_NAME}"
    return 0
}

# ── Fallback: cargo install --git ──────────────────────────

ensure_cargo() {
    if command -v cargo >/dev/null 2>&1; then
        ok "Rust toolchain: $(cargo --version)"
        return 0
    fi

    warn "Rust (cargo) not found. Tytus is built from source with cargo."
    reply=$(read_reply "Install Rust via rustup now? [y/N]" "n")
    case "$reply" in
        [yY]*)
            msg "Installing Rust via rustup (~2 minutes)..."
            curl --proto '=https' --tlsv1.2 -sSfL https://sh.rustup.rs \
                | sh -s -- -y --default-toolchain stable --profile minimal
            # shellcheck disable=SC1091
            . "$HOME/.cargo/env"
            if command -v cargo >/dev/null 2>&1; then
                ok "Rust installed: $(cargo --version)"
            else
                err "rustup finished but cargo is still not on PATH."
                err "Open a new terminal and re-run this installer."
                exit 1
            fi
            ;;
        *)
            err "Rust is required to install Tytus from source."
            err "Install manually from https://rustup.rs and re-run this script."
            err "Or wait for us to ship prebuilt binaries — coming soon."
            exit 1
            ;;
    esac
}

install_from_source() {
    ensure_cargo
    msg "Building ${CLI_NAME} and ${MCP_NAME} from source via cargo install --git..."
    msg "First build takes 3–5 minutes. Subsequent upgrades take ~30 seconds."

    CARGO_ARGS="--git ${REPO_URL} --branch main --bin ${CLI_NAME} --bin ${MCP_NAME} --force"
    if [ -n "${TYTUS_INSTALL_DIR:-}" ]; then
        msg "Installing to ${TYTUS_INSTALL_DIR}"
        # shellcheck disable=SC2086
        cargo install $CARGO_ARGS --root "${TYTUS_INSTALL_DIR%/bin}"
        BIN_PATH="${TYTUS_INSTALL_DIR}/${CLI_NAME}"
    else
        # shellcheck disable=SC2086
        cargo install $CARGO_ARGS
        BIN_PATH="${HOME}/.cargo/bin/${CLI_NAME}"
    fi
}

# ── Sudoers setup ──────────────────────────────────────────

setup_sudoers() {
    [ "${TYTUS_SKIP_SUDOERS:-}" = "1" ] && { ok "Skipping sudoers setup (TYTUS_SKIP_SUDOERS=1)"; return 0; }

    SUDOERS_FILE="/etc/sudoers.d/tytus"
    CURRENT_USER="${SUDO_USER:-$(whoami)}"
    # Tight sudoers entry: only the tytus binary, only the two subcommands
    # needed for tunnel lifecycle, and tunnel-up is restricted to config files
    # under /tmp/tytus/tunnel-*.json so attackers can't point it at arbitrary
    # files like /etc/shadow. The `tunnel-down` helper internally validates
    # the target PID against /tmp/tytus/tunnel-*.pid so it cannot be used to
    # SIGTERM arbitrary system processes — that mistake from the previous
    # design (`/bin/kill -TERM *`) was a real privilege escalation vector.
    ENTRY="${CURRENT_USER} ALL=(root) NOPASSWD: ${BIN_PATH} tunnel-up /tmp/tytus/tunnel-*.json, ${BIN_PATH} tunnel-down *"

    msg "Configuring passwordless tunnel (optional)..."
    if [ -f "$SUDOERS_FILE" ] && grep -qF "$ENTRY" "$SUDOERS_FILE" 2>/dev/null; then
        ok "Passwordless tunnel already configured"
        return 0
    fi

    write_entry() {
        echo "$ENTRY" > "$SUDOERS_FILE"
        chmod 440 "$SUDOERS_FILE"
    }

    if [ "$(id -u)" = "0" ]; then
        write_entry && ok "Passwordless tunnel configured for ${CURRENT_USER}"
    elif command -v sudo >/dev/null 2>&1; then
        if sudo -n true 2>/dev/null; then
            sudo sh -c "echo '$ENTRY' > '$SUDOERS_FILE' && chmod 440 '$SUDOERS_FILE'" \
                && ok "Passwordless tunnel configured for ${CURRENT_USER}"
        else
            warn "Passwordless tunnel not configured — you'll be prompted for sudo on 'tytus connect'."
            warn "To configure later, run: sudo ${BIN_PATH} install-sudoers (coming soon)"
        fi
    else
        warn "sudo not available; passwordless tunnel not configured."
    fi
}

# ── Verify ─────────────────────────────────────────────────

verify_install() {
    if ! command -v "${CLI_NAME}" >/dev/null 2>&1; then
        err "${CLI_NAME} was installed but isn't on PATH."
        err "Add this to your shell profile and open a new terminal:"
        err "    export PATH=\"\$HOME/.cargo/bin:\$PATH\""
        exit 1
    fi
    ok "$(${CLI_NAME} --version)"
    if command -v "${MCP_NAME}" >/dev/null 2>&1; then
        ok "${MCP_NAME} ready (MCP server for Claude Code / OpenCode)"
    fi
}

# ── Next steps ─────────────────────────────────────────────

print_next_steps() {
    printf "\n"
    printf "%s┌─────────────────────────────────────────────────┐%s\n" "$GREEN$BOLD" "$RESET"
    printf "%s│             %sTytus is ready to use!%s               │%s\n" "$GREEN$BOLD" "$RESET$GREEN$BOLD" "$RESET$GREEN$BOLD" "$RESET"
    printf "%s└─────────────────────────────────────────────────┘%s\n" "$GREEN$BOLD" "$RESET"
    printf "\n"
    printf "${BOLD}Next steps:${RESET}\n"
    printf "\n"
    printf "  ${GREEN}1.${RESET} Interactive first-run wizard (login → plan → pod → tunnel → test):\n"
    printf "       ${BOLD}tytus setup${RESET}\n"
    printf "\n"
    printf "  ${GREEN}2.${RESET} Or drive it manually:\n"
    printf "       ${BOLD}tytus login${RESET}          # browser device-auth\n"
    printf "       ${BOLD}tytus connect${RESET}        # allocate a pod + activate tunnel\n"
    printf "       ${BOLD}tytus env --export${RESET}   # OPENAI_BASE_URL + OPENAI_API_KEY\n"
    printf "       ${BOLD}tytus chat${RESET}           # REPL against your private pod\n"
    printf "\n"
    printf "  ${GREEN}3.${RESET} Make Claude Code / OpenCode / Cursor drive Tytus natively:\n"
    printf "       ${BOLD}tytus bootstrap-prompt${RESET}   # short paste prompt for any AI tool\n"
    printf "       ${BOLD}tytus link .${RESET}              # drop integration files into a project\n"
    printf "\n"
    printf "  ${GREEN}4.${RESET} Full LLM-facing reference (for AI agents):\n"
    printf "       ${BOLD}tytus llm-docs${RESET}\n"
    printf "\n"
    printf "${DIM}Docs: %s${RESET}\n" "${REPO_URL}"
    printf "\n"
}

# ── Main ───────────────────────────────────────────────────

main() {
    banner
    detect_platform

    if try_release_download; then
        :
    else
        install_from_source
    fi

    verify_install
    setup_sudoers
    print_next_steps
}

main "$@"

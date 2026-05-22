#!/bin/sh
# ============================================================
# tytus-cli installer — installs Tytus + shared-folder tools
# ============================================================
#
# Usage:
#     curl -fsSL https://get.traylinx.com/install.sh | bash
# (legacy direct-from-github URL also works as a fallback)
#
# Production install policy:
#   The one-command installer must never make normal users build from source.
#   Default installs use checksum-verified release artifacts only. If no
#   matching artifact exists, the installer fails with a friendly message
#   instead of asking a non-technical user to install Rust, cargo, or admin
#   tooling. Source builds are developer-only and require an explicit opt-in.
#
# What it does:
#   1. Detects your OS + arch
#   2. Downloads a checksum-verified GitHub release artifact
#   3. Installs `tytus`, `tytus-mcp`, `tytus-tray`, and bundled
#      `garagetytus` shared-folder tools when present in the release
#   4. Developer-only: builds from source when explicitly requested
#   5. Sets up a tightly-scoped sudoers entry so `tytus connect` never
#      prompts for a password when opening the WireGuard tunnel
#   6. Prints clear next steps
#
# Env:
#     TYTUS_INSTALL_DIR    Override the install directory
#                          (default: /usr/local/bin for releases,
#                          $HOME/.cargo/bin for dev-source builds)
#     TYTUS_INSTALL_MODE   "production" (default) or "dev-source"
#     TYTUS_DEV_SOURCE_INSTALL
#                          Set to "1" as a short alias for
#                          TYTUS_INSTALL_MODE=dev-source
#     TYTUS_SKIP_SUDOERS   Set to "1" to skip sudoers configuration
#     TYTUS_SKIP_CHECKSUM  Set to "1" to skip SHA256 verification when
#                          using release artifacts (NOT RECOMMENDED)
#     TYTUS_RELEASE_TAG    Install a specific GitHub release tag instead of
#                          the public catalog tag, e.g. v0.7.0
#     TYTUS_CATALOG_URL    Override public catalog URL used to resolve the
#                          default release tag
#     TYTUS_SKIP_GARAGETYTUS
#                          Set to "1" to skip installing bundled shared-folder
#                          tools. Normal users should not set this.
# ============================================================

set -eu

REPO="traylinx/tytus-cli"
REPO_URL="https://github.com/${REPO}"
PUBLIC_CATALOG_URL="${TYTUS_CATALOG_URL:-https://get.traylinx.com/catalog.json}"
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

install_mode() {
    if [ "${TYTUS_DEV_SOURCE_INSTALL:-}" = "1" ]; then
        printf "dev-source"
        return 0
    fi
    printf "%s" "${TYTUS_INSTALL_MODE:-production}"
}

validate_install_mode() {
    case "$1" in
        production|dev-source) return 0 ;;
        *)
            err "Unsupported TYTUS_INSTALL_MODE='$1'. Use 'production' or 'dev-source'."
            exit 1
            ;;
    esac
}

banner() {
    _mode="$1"
    printf "\n"
    printf "%s┌─────────────────────────────────────────────────┐%s\n" "$BOLD" "$RESET"
    printf "%s│          Installing %sTytus CLI%s                    │%s\n" "$BOLD" "$BLUE" "$RESET$BOLD" "$RESET"
    printf "%s│   %sPrivate AI pods driven from your terminal%s     │%s\n" "$BOLD" "$DIM" "$RESET$BOLD" "$RESET"
    printf "%s└─────────────────────────────────────────────────┘%s\n" "$BOLD" "$RESET"
    printf "\n"
    if [ "$_mode" = "dev-source" ]; then
        printf "%sDeveloper mode:%s building from source against main branch.\n" "$YELLOW$BOLD" "$RESET"
        printf "%sRequires:%s Rust toolchain + cargo.\n" "$DIM" "$RESET"
    else
        printf "%sProduction mode:%s release artifact only; no Rust/cargo source builds.\n" "$GREEN$BOLD" "$RESET"
        printf "%sSafety:%s checksum verification is required unless TYTUS_SKIP_CHECKSUM=1.\n" "$DIM" "$RESET"
    fi
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
    RELEASE_ASSET=""
    case "${OS}-${ARCH_NORM}" in
        darwin-x86_64)  RELEASE_ASSET="tytus-macos-x86_64.tar.gz" ;;
        darwin-aarch64) RELEASE_ASSET="tytus-macos-aarch64.tar.gz" ;;
        linux-x86_64)   RELEASE_ASSET="tytus-linux-x86_64.tar.gz" ;;
        linux-aarch64)  RELEASE_ASSET="tytus-linux-aarch64.tar.gz" ;;
        *)              return 1 ;;
    esac

    if [ -n "${TYTUS_RELEASE_TAG:-}" ]; then
        EFFECTIVE_RELEASE_TAG="$TYTUS_RELEASE_TAG"
        RELEASE_API_URL="https://api.github.com/repos/${REPO}/releases/tags/${EFFECTIVE_RELEASE_TAG}"
        msg "Looking for prebuilt release (${RELEASE_ASSET}) on ${EFFECTIVE_RELEASE_TAG}..."
    else
        EFFECTIVE_RELEASE_TAG=$(curl -fsSL "$PUBLIC_CATALOG_URL" 2>/dev/null \
            | sed -n 's/.*"release_tag"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' \
            | head -1)
        if [ -n "$EFFECTIVE_RELEASE_TAG" ]; then
            RELEASE_API_URL="https://api.github.com/repos/${REPO}/releases/tags/${EFFECTIVE_RELEASE_TAG}"
            msg "Looking for prebuilt release (${RELEASE_ASSET}) from public catalog (${EFFECTIVE_RELEASE_TAG})..."
        else
            RELEASE_API_URL="https://api.github.com/repos/${REPO}/releases/latest"
            warn "Could not read public catalog release_tag; falling back to GitHub latest."
            msg "Looking for prebuilt release (${RELEASE_ASSET})..."
        fi
    fi
    RELEASES_JSON=$(curl -fsSL "$RELEASE_API_URL" 2>/dev/null)
    RELEASE_URL=$(printf "%s" "$RELEASES_JSON" \
        | grep "browser_download_url.*${RELEASE_ASSET}" \
        | cut -d'"' -f4 | head -1)
    SUMS_URL=$(printf "%s" "$RELEASES_JSON" \
        | grep "browser_download_url.*SHA256SUMS" \
        | cut -d'"' -f4 | head -1)

    if [ -z "$RELEASE_URL" ]; then
        warn "No production release artifact published yet for ${RELEASE_ASSET}."
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
        EXPECTED=$(grep -E "[[:space:]](\./)?${RELEASE_ASSET}\$" "${TMP}/SHA256SUMS" | awk '{print $1}' | head -1)
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

    if [ ! -d "$INSTALL_DIR" ]; then
        if mkdir -p "$INSTALL_DIR" 2>/dev/null; then
            :
        else
            sudo mkdir -p "$INSTALL_DIR"
        fi
    fi

    install_one() {
        _bin="$1"
        [ -f "${TMP}/${_bin}" ] || return 0
        if [ -w "$INSTALL_DIR" ]; then
            mv "${TMP}/${_bin}" "${INSTALL_DIR}/"
            chmod +x "${INSTALL_DIR}/${_bin}"
        else
            sudo mv "${TMP}/${_bin}" "${INSTALL_DIR}/"
            sudo chmod +x "${INSTALL_DIR}/${_bin}"
        fi
        ok "${INSTALL_DIR}/${_bin}"
    }
    msg "Installing to ${INSTALL_DIR}..."
    install_one "${CLI_NAME}"
    install_one "${MCP_NAME}"
    install_one "tytus-tray"
    if [ "${TYTUS_SKIP_GARAGETYTUS:-}" = "1" ]; then
        ok "Skipping shared-folder tools (TYTUS_SKIP_GARAGETYTUS=1)"
    else
        install_one "garagetytus"
        for _helper_path in "${TMP}"/garagetytus-*; do
            [ -f "$_helper_path" ] || continue
            install_one "$(basename "$_helper_path")"
        done
        if [ -f "${INSTALL_DIR}/garagetytus" ]; then
            ok "Shared-folder tools installed (garagetytus + helpers)"
        else
            warn "This Tytus release does not bundle garagetytus yet — shared-folder sync tools were not installed."
        fi
    fi

    BIN_PATH="${INSTALL_DIR}/${CLI_NAME}"
    return 0
}

production_unavailable() {
    err "Production installer is not available for ${OS_PRETTY} ${ARCH_NORM} yet."
    err "Tytus production installs must use checksum-verified release artifacts."
    err "Normal users should not install Rust, cargo, or build Tytus from source."
    err ""
    err "Developer escape hatch:"
    err "    curl -fsSL https://get.traylinx.com/install.sh | TYTUS_INSTALL_MODE=dev-source sh"
    err "or:"
    err "    curl -fsSL https://get.traylinx.com/install.sh | TYTUS_DEV_SOURCE_INSTALL=1 sh"
    exit 1
}

# ── Fallback: cargo install --git ──────────────────────────

ensure_cargo() {
    if command -v cargo >/dev/null 2>&1; then
        ok "Rust toolchain: $(cargo --version)"
        return 0
    fi

    warn "Rust (cargo) not found. Developer source installs require cargo."
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
            err "Rust is required for developer source installs."
            err "Install manually from https://rustup.rs and re-run this script."
            exit 1
            ;;
    esac
}

install_from_source() {
    ensure_cargo
    msg "Building ${CLI_NAME} and ${MCP_NAME} from source via cargo install --git..."
    msg "First build takes 3–5 minutes. Subsequent upgrades take ~30 seconds."

    # Workspace has three bin-producing packages (atomek-cli, tytus-mcp,
    # tytus-tray). cargo install --git needs explicit crate selection when
    # the target repo has >1 package with binaries, otherwise it errors
    # out with "multiple packages with binaries found". Crate names are
    # passed positionally to `cargo install` (not via -p flags — that's
    # `cargo build` syntax).
    #
    # tytus-tray is included on macOS so `tytus tray install` (run at
    # the end of this script) finds the binary. Locally-compiled
    # binaries don't carry the `com.apple.quarantine` xattr so
    # Gatekeeper doesn't intercept them — this is the curl-pipe path
    # that intentionally sidesteps Apple Developer ID signing for
    # the v0.6.x beta. See `pkg/SIGNING.md` for the eventual signed
    # `.pkg` path (deferred until paid Developer Program enrollment).
    if [ "$(uname -s)" = "Darwin" ]; then
        CARGO_ARGS="--git ${REPO_URL} --branch main atomek-cli tytus-mcp tytus-tray --force"
    else
        CARGO_ARGS="--git ${REPO_URL} --branch main atomek-cli tytus-mcp --force"
    fi
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
    if [ "${TYTUS_SKIP_GARAGETYTUS:-}" != "1" ]; then
        if command -v garagetytus >/dev/null 2>&1; then
            ok "garagetytus ready (shared-folder CLI)"
        elif command -v garagetytus-folder-list >/dev/null 2>&1; then
            ok "garagetytus helpers ready (shared-folder scripts)"
        else
            warn "garagetytus shared-folder tools are not on PATH."
        fi
    fi
}

# ── macOS tray bundle install ──────────────────────────────
#
# On macOS, run `tytus tray install` so the user gets the menubar
# T + Tytus.app + LaunchAgent (auto-start at login) without a
# second manual step. Locally-compiled tray binary doesn't carry
# the `com.apple.quarantine` xattr so Gatekeeper doesn't intercept
# — this is the curl-pipe path that intentionally sidesteps Apple
# Developer ID signing for the v0.6.x beta. See `pkg/SIGNING.md`
# for the eventual signed `.pkg` path.

install_tray_macos() {
    [ "$(uname -s)" = "Darwin" ] || return 0
    [ "${TYTUS_SKIP_TRAY:-}" = "1" ] && { ok "Skipping tray install (TYTUS_SKIP_TRAY=1)"; return 0; }
    if ! command -v tytus-tray >/dev/null 2>&1; then
        warn "tytus-tray binary not on PATH — skipping menubar install."
        warn "Re-run after fixing PATH: tytus tray install"
        return 0
    fi
    msg "Installing menubar app (Tytus.app)..."
    if "${CLI_NAME}" tray install >/dev/null 2>&1; then
        ok "Tytus.app installed in /Applications + auto-start at login"
    else
        warn "Menubar install failed — run manually: tytus tray install"
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
    printf "  ${GREEN}4.${RESET} Open the desktop UI:\n"
    if [ "$(uname -s)" = "Darwin" ]; then
        printf "       ${BOLD}Tytus.app${RESET}                 # menu bar app; opens TytusOS in your browser\n"
        printf "       after ${BOLD}tytus tray install${RESET}: ${BOLD}http://127.0.0.1:\$(cat /tmp/tytus/tray-web.port)/${RESET}\n"
    else
        printf "       TytusOS browser UI is packaged with the tray app today.\n"
        printf "       Linux CLI works now; Linux desktop-tray/browser packaging is tracked separately.\n"
    fi
    printf "\n"
    printf "  ${GREEN}5.${RESET} Shared folders:\n"
    printf "       Open ${BOLD}TytusOS → Settings → Sharing${RESET} or use ${BOLD}garagetytus folder list${RESET}\n"
    printf "       to connect files, photos, briefs, and outputs between your computer and pods.\n"
    printf "\n"
    printf "  ${GREEN}6.${RESET} Full LLM-facing reference (for AI agents):\n"
    printf "       ${BOLD}tytus llm-docs${RESET}\n"
    printf "\n"
    printf "${DIM}Docs: %s${RESET}\n" "${REPO_URL}"
    printf "\n"
}

# ── Main ───────────────────────────────────────────────────

main() {
    MODE=$(install_mode)
    validate_install_mode "$MODE"
    banner "$MODE"
    detect_platform

    case "$MODE" in
        production)
            try_release_download || production_unavailable
            ;;
        dev-source)
            warn "Developer source install requested explicitly; this is not the grandma-safe production path."
            install_from_source
            ;;
    esac

    verify_install
    setup_sudoers
    install_tray_macos
    print_next_steps
}

main "$@"

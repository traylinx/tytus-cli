# ============================================================
# tytus-cli installer for Windows (PowerShell)
# ============================================================
#
# Usage:
#     powershell -c "irm https://get.traylinx.com/install.ps1 | iex"
#
# Early-access policy:
#   Tytus is under active development. The installer builds from source
#   against `main` via `cargo install --git` so every user gets the latest
#   fixes without us cutting a release for every bugfix. Prebuilt binaries
#   will return once the CLI is stable and versioned.
#
# What it does:
#   1. Detects architecture (x86_64 or arm64)
#   2. Ensures a Rust toolchain is present (offers rustup install if not)
#   3. Builds tytus + tytus-mcp from the main branch
#   4. (Opt-in) Uses the last published release if $env:TYTUS_USE_RELEASE=1
#   5. Adds install dir to user PATH
#
# Env vars:
#     $env:TYTUS_INSTALL_DIR    Override install directory
#     $env:TYTUS_USE_RELEASE    Prefer the last published release (may be stale)
#     $env:TYTUS_SKIP_CHECKSUM  Skip SHA256 verification when using
#                                TYTUS_USE_RELEASE (NOT RECOMMENDED)
#
# NOTE: Windows tunnel support is experimental. The `tytus connect` command
# needs wintun.dll to function — we're bundling it in a future release.
# Until then, `tytus` works fine for login, chat, env, MCP, and link
# operations; `tytus connect` will fail with a clear error message.
# ============================================================

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$Repo = 'traylinx/tytus-cli'
$RepoUrl = "https://github.com/$Repo"

function Write-Step($msg)    { Write-Host "==> $msg" -ForegroundColor Blue }
function Write-Ok($msg)      { Write-Host " OK  $msg" -ForegroundColor Green }
function Write-Warn2($msg)   { Write-Host " !   $msg" -ForegroundColor Yellow }
function Write-Err2($msg)    { Write-Host " X   $msg" -ForegroundColor Red }

function Show-Banner {
    Write-Host ""
    Write-Host "┌─────────────────────────────────────────────────┐" -ForegroundColor White
    Write-Host "│          Installing Tytus CLI (Windows)         │" -ForegroundColor White
    Write-Host "│   Private AI pods driven from your terminal     │" -ForegroundColor White
    Write-Host "└─────────────────────────────────────────────────┘" -ForegroundColor White
    Write-Host ""
}

function Get-Arch {
    $a = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
    switch ($a) {
        'X64'   { return 'x86_64' }
        'Arm64' { return 'aarch64' }
        default {
            Write-Err2 "Unsupported architecture: $a"
            exit 1
        }
    }
}

function Get-InstallDir {
    if ($env:TYTUS_INSTALL_DIR) { return $env:TYTUS_INSTALL_DIR }
    return (Join-Path $env:LOCALAPPDATA 'Programs\Tytus')
}

function Add-ToUserPath($dir) {
    $currentPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if ($currentPath -notlike "*$dir*") {
        $newPath = if ($currentPath) { "$currentPath;$dir" } else { $dir }
        [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
        Write-Ok "Added $dir to user PATH (restart shell to pick up)"
    } else {
        Write-Ok "$dir already on PATH"
    }
}

function Install-FromRelease {
    # Early-access policy: releases are opt-in.
    if ($env:TYTUS_USE_RELEASE -ne '1') { return $false }

    $arch = Get-Arch
    $asset = "tytus-windows-$arch.zip"

    Write-Step "Looking for prebuilt release ($asset)..."
    try {
        $release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
    } catch {
        Write-Warn2 "Could not reach GitHub releases API."
        return $false
    }

    $assetUrl = ($release.assets | Where-Object { $_.name -eq $asset } | Select-Object -First 1).browser_download_url
    $sumsUrl  = ($release.assets | Where-Object { $_.name -eq 'SHA256SUMS' } | Select-Object -First 1).browser_download_url

    if (-not $assetUrl) {
        Write-Warn2 "No prebuilt binary published yet for $asset. Falling back to source build."
        return $false
    }

    Write-Ok "Found release: $assetUrl"

    $tmp = New-Item -ItemType Directory -Path (Join-Path $env:TEMP "tytus-install-$(Get-Random)")
    try {
        $zipPath = Join-Path $tmp $asset
        Write-Step "Downloading..."
        Invoke-WebRequest -Uri $assetUrl -OutFile $zipPath -UseBasicParsing

        # ── SHA256 verification ────────────────────────────────
        if ($env:TYTUS_SKIP_CHECKSUM -eq '1') {
            Write-Warn2 "TYTUS_SKIP_CHECKSUM=1 — SKIPPING checksum verification. NOT RECOMMENDED."
        } elseif (-not $sumsUrl) {
            Write-Err2 "No SHA256SUMS found on this release — refusing to install unverified binary."
            Write-Err2 "Report at $RepoUrl/issues"
            exit 1
        } else {
            Write-Step "Verifying SHA256..."
            $sumsPath = Join-Path $tmp 'SHA256SUMS'
            Invoke-WebRequest -Uri $sumsUrl -OutFile $sumsPath -UseBasicParsing
            $expected = (Get-Content $sumsPath | Where-Object { $_ -match "\s$([regex]::Escape($asset))$" } | ForEach-Object { ($_ -split '\s+')[0] } | Select-Object -First 1)
            if (-not $expected) {
                Write-Err2 "SHA256SUMS does not contain entry for $asset"
                exit 1
            }
            $actual = (Get-FileHash $zipPath -Algorithm SHA256).Hash.ToLower()
            if ($expected.ToLower() -ne $actual) {
                Write-Err2 "CHECKSUM MISMATCH — refusing to install tampered binary"
                Write-Err2 "  expected: $expected"
                Write-Err2 "  got:      $actual"
                exit 1
            }
            Write-Ok "Checksum verified"
        }

        $installDir = Get-InstallDir
        New-Item -ItemType Directory -Force -Path $installDir | Out-Null

        Write-Step "Extracting to $installDir..."
        Expand-Archive -Path $zipPath -DestinationPath $installDir -Force

        Write-Ok "$installDir\tytus.exe"
        if (Test-Path (Join-Path $installDir 'tytus-mcp.exe')) {
            Write-Ok "$installDir\tytus-mcp.exe"
        }

        Add-ToUserPath $installDir
        return $true
    } finally {
        Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
    }
}

function Ensure-Cargo {
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        Write-Ok "Rust toolchain: $(cargo --version)"
        return
    }

    Write-Warn2 "Rust (cargo) not found. Tytus needs cargo to build from source."
    $reply = Read-Host "Install Rust via rustup now? [y/N]"
    if ($reply -notmatch '^[yY]') {
        Write-Err2 "Rust is required. Install from https://rustup.rs and re-run this script."
        exit 1
    }

    Write-Step "Installing Rust via rustup (~2 minutes)..."
    $rustupUrl = 'https://win.rustup.rs/x86_64'
    $rustupPath = Join-Path $env:TEMP 'rustup-init.exe'
    Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath -UseBasicParsing
    & $rustupPath -y --default-toolchain stable --profile minimal
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Err2 "rustup finished but cargo is still not on PATH."
        Write-Err2 "Open a new terminal and re-run this installer."
        exit 1
    }
    Write-Ok "Rust installed: $(cargo --version)"
}

function Install-FromSource {
    Ensure-Cargo
    Write-Step "Building tytus and tytus-mcp from source via cargo install --git..."
    Write-Step "First build takes 5-8 minutes. Subsequent upgrades take ~30 seconds."

    # Workspace has three bin-producing packages (atomek-cli, tytus-mcp,
    # tytus-tray). Crate names are passed positionally to cargo install.
    $installRoot = if ($env:TYTUS_INSTALL_DIR) {
        Split-Path $env:TYTUS_INSTALL_DIR -Parent
    } else {
        $null
    }

    if ($installRoot) {
        cargo install --git $RepoUrl --branch main atomek-cli tytus-mcp --force --root $installRoot
        $binDir = Join-Path $installRoot 'bin'
    } else {
        cargo install --git $RepoUrl --branch main atomek-cli tytus-mcp --force
        $binDir = Join-Path $env:USERPROFILE '.cargo\bin'
    }

    Add-ToUserPath $binDir
}

function Verify-Install {
    $tytus = Get-Command tytus -ErrorAction SilentlyContinue
    if (-not $tytus) {
        $cargoBin = Join-Path $env:USERPROFILE '.cargo\bin\tytus.exe'
        if (Test-Path $cargoBin) {
            Write-Warn2 "tytus installed at $cargoBin but not on PATH yet."
            Write-Warn2 "Open a new PowerShell window and try: tytus --version"
            return
        }
        Write-Err2 "tytus was installed but cannot be found on PATH."
        exit 1
    }
    $version = & tytus --version 2>&1
    Write-Ok "$version"
}

function Print-NextSteps {
    Write-Host ""
    Write-Host "┌─────────────────────────────────────────────────┐" -ForegroundColor Green
    Write-Host "│             Tytus is ready to use!              │" -ForegroundColor Green
    Write-Host "└─────────────────────────────────────────────────┘" -ForegroundColor Green
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor White
    Write-Host ""
    Write-Host "  1. Interactive first-run wizard:" -ForegroundColor White
    Write-Host "       tytus setup" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  2. Drive it manually:" -ForegroundColor White
    Write-Host "       tytus login" -ForegroundColor Cyan
    Write-Host "       tytus connect" -ForegroundColor Cyan
    Write-Host "       tytus chat" -ForegroundColor Cyan
    Write-Host ""
    Write-Warn2 "Windows tunnel support is experimental."
    Write-Warn2 "'tytus connect' currently needs wintun.dll — this is being bundled in a future release."
    Write-Warn2 "For now, you can use 'tytus login', 'tytus env', 'tytus chat', 'tytus link', and 'tytus mcp' fully."
    Write-Host ""
    Write-Host "Docs: $RepoUrl" -ForegroundColor Gray
    Write-Host ""
}

# ── Main ────────────────────────────────────────────────────

Show-Banner

$arch = Get-Arch
Write-Ok "Detected: Windows $arch"
Write-Warn2 "Early access — building from main branch source."

# Default path: source build from main. Opt in to stale prebuilt
# release via $env:TYTUS_USE_RELEASE=1.
$ok = $false
if ($env:TYTUS_USE_RELEASE -eq '1') {
    $ok = Install-FromRelease
}
if (-not $ok) {
    Install-FromSource
}

Verify-Install
Print-NextSteps

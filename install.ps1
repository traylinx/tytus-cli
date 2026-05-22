# ============================================================
# tytus-cli installer for Windows (PowerShell)
# ============================================================
#
# Usage:
#     powershell -c "irm https://get.traylinx.com/install.ps1 | iex"
#
# Production install policy:
#   The one-command installer must never make normal users build from source.
#   Default installs use checksum-verified release artifacts only. If no
#   matching artifact exists, the installer fails with a friendly message
#   instead of asking a non-technical user to install Rust, cargo, or admin
#   tooling. Source builds are developer-only and require an explicit opt-in.
#
# What it does:
#   1. Detects architecture (x86_64 or arm64)
#   2. Downloads a checksum-verified GitHub release artifact
#   3. Installs tytus + tytus-mcp
#   4. Developer-only: builds from source when explicitly requested
#   5. Adds install dir to user PATH
#
# Env vars:
#     $env:TYTUS_INSTALL_DIR    Override install directory
#     $env:TYTUS_INSTALL_MODE   "production" (default) or "dev-source"
#     $env:TYTUS_DEV_SOURCE_INSTALL
#                                Set to "1" as a short alias for
#                                TYTUS_INSTALL_MODE=dev-source
#     $env:TYTUS_SKIP_CHECKSUM  Skip SHA256 verification when using
#                                release artifacts (NOT RECOMMENDED)
#     $env:TYTUS_RELEASE_TAG    Install a specific GitHub release tag instead
#                                of the public catalog tag, e.g. v0.7.0
#     $env:TYTUS_CATALOG_URL    Override public catalog URL used to resolve
#                                the default release tag
#
# NOTE: Windows tunnel support is experimental. The `tytus connect` command
# needs wintun.dll to function — we're bundling it in a future release.
# Until then, `tytus` works fine for login, chat, env, MCP, and link
# operations; `tytus connect` will fail with a clear error message.
# TytusOS browser UI is not full-parity on Windows yet; use CLI mode until
# the Windows tray/browser packaging sprint lands.
# ============================================================

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$Repo = 'traylinx/tytus-cli'
$RepoUrl = "https://github.com/$Repo"
$CatalogUrl = if ($env:TYTUS_CATALOG_URL) { $env:TYTUS_CATALOG_URL } else { 'https://get.traylinx.com/catalog.json' }

function Write-Step($msg)    { Write-Host "==> $msg" -ForegroundColor Blue }
function Write-Ok($msg)      { Write-Host " OK  $msg" -ForegroundColor Green }
function Write-Warn2($msg)   { Write-Host " !   $msg" -ForegroundColor Yellow }
function Write-Err2($msg)    { Write-Host " X   $msg" -ForegroundColor Red }

function Show-Banner {
    param([string]$Mode)
    Write-Host ""
    Write-Host "┌─────────────────────────────────────────────────┐" -ForegroundColor White
    Write-Host "│          Installing Tytus CLI (Windows)         │" -ForegroundColor White
    Write-Host "│   Private AI pods driven from your terminal     │" -ForegroundColor White
    Write-Host "└─────────────────────────────────────────────────┘" -ForegroundColor White
    Write-Host ""
    if ($Mode -eq 'dev-source') {
        Write-Warn2 "Developer mode: building from source against main branch."
        Write-Host "Requires: Rust toolchain + cargo." -ForegroundColor Gray
    } else {
        Write-Ok "Production mode: release artifact only; no Rust/cargo source builds."
        Write-Host "Safety: checksum verification is required unless TYTUS_SKIP_CHECKSUM=1." -ForegroundColor Gray
    }
    Write-Host ""
}

function Get-InstallMode {
    if ($env:TYTUS_DEV_SOURCE_INSTALL -eq '1') { return 'dev-source' }
    if ($env:TYTUS_INSTALL_MODE) { return $env:TYTUS_INSTALL_MODE }
    return 'production'
}

function Assert-InstallMode([string]$Mode) {
    if ($Mode -ne 'production' -and $Mode -ne 'dev-source') {
        Write-Err2 "Unsupported TYTUS_INSTALL_MODE='$Mode'. Use 'production' or 'dev-source'."
        exit 1
    }
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
    $parts = @()
    if ($currentPath) {
        $parts = $currentPath -split ';' | Where-Object { $_ -ne '' }
    }
    $alreadyPersisted = $parts | Where-Object { $_.TrimEnd('\') -ieq $dir.TrimEnd('\') } | Select-Object -First 1

    if (-not $alreadyPersisted) {
        $newPath = if ($currentPath) { "$currentPath;$dir" } else { $dir }
        [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
        Write-Ok "Added $dir to user PATH (restart shell to pick up)"
    } else {
        Write-Ok "$dir already on user PATH"
    }

    $processParts = $env:Path -split ';' | Where-Object { $_ -ne '' }
    $alreadyLive = $processParts | Where-Object { $_.TrimEnd('\') -ieq $dir.TrimEnd('\') } | Select-Object -First 1
    if (-not $alreadyLive) {
        $env:Path = "$env:Path;$dir"
        Write-Ok "Added $dir to this PowerShell session PATH"
    }
}

function Install-FromRelease {
    $arch = Get-Arch
    $asset = "tytus-windows-$arch.zip"

    if ($env:TYTUS_RELEASE_TAG) {
        $effectiveReleaseTag = $env:TYTUS_RELEASE_TAG
        $releaseApiUrl = "https://api.github.com/repos/$Repo/releases/tags/$effectiveReleaseTag"
        Write-Step "Looking for prebuilt release ($asset) on $effectiveReleaseTag..."
    } else {
        $effectiveReleaseTag = $null
        try {
            $catalog = Invoke-RestMethod $CatalogUrl
            $effectiveReleaseTag = $catalog.release_tag
        } catch {
            Write-Warn2 "Could not read public catalog release_tag; falling back to GitHub latest."
        }
        if ($effectiveReleaseTag) {
            $releaseApiUrl = "https://api.github.com/repos/$Repo/releases/tags/$effectiveReleaseTag"
            Write-Step "Looking for prebuilt release ($asset) from public catalog ($effectiveReleaseTag)..."
        } else {
            $releaseApiUrl = "https://api.github.com/repos/$Repo/releases/latest"
            Write-Step "Looking for prebuilt release ($asset)..."
        }
    }
    try {
        $release = Invoke-RestMethod $releaseApiUrl
    } catch {
        Write-Warn2 "Could not reach GitHub releases API."
        return $false
    }

    $assetUrl = ($release.assets | Where-Object { $_.name -eq $asset } | Select-Object -First 1).browser_download_url
    $sumsUrl  = ($release.assets | Where-Object { $_.name -eq 'SHA256SUMS' } | Select-Object -First 1).browser_download_url

    if (-not $assetUrl) {
        Write-Warn2 "No production release artifact published yet for $asset."
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
            $assetPattern = "\s(\./)?$([regex]::Escape($asset))$"
            $expected = (Get-Content $sumsPath | Where-Object { $_ -match $assetPattern } | ForEach-Object { ($_ -split '\s+')[0] } | Select-Object -First 1)
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

function Stop-ProductionUnavailable([string]$Arch) {
    Write-Err2 "Production installer is not available for Windows $Arch yet."
    Write-Err2 "Tytus production installs must use checksum-verified release artifacts."
    Write-Err2 "Normal users should not install Rust, cargo, or build Tytus from source."
    Write-Err2 ""
    Write-Err2 "Developer escape hatch:"
    Write-Err2 '    $env:TYTUS_INSTALL_MODE="dev-source"; irm https://get.traylinx.com/install.ps1 | iex'
    Write-Err2 "or:"
    Write-Err2 '    $env:TYTUS_DEV_SOURCE_INSTALL="1"; irm https://get.traylinx.com/install.ps1 | iex'
    exit 1
}

function Ensure-Cargo {
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        Write-Ok "Rust toolchain: $(cargo --version)"
        return
    }

    Write-Warn2 "Rust (cargo) not found. Developer source installs require cargo."
    $reply = Read-Host "Install Rust via rustup now? [y/N]"
    if ($reply -notmatch '^[yY]') {
        Write-Err2 "Rust is required for developer source installs. Install from https://rustup.rs and re-run this script."
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
    Write-Host "  3. TytusOS desktop/browser UI:" -ForegroundColor White
    Write-Warn2 "Windows TytusOS UI packaging is not full-parity yet."
    Write-Warn2 "Use CLI mode today; tray/browser UI support is tracked in the TytusOS cutover sprint."
    Write-Host ""
    Write-Warn2 "Windows tunnel support is experimental."
    Write-Warn2 "'tytus connect' currently needs wintun.dll — this is being bundled in a future release."
    Write-Warn2 "For now, you can use 'tytus login', 'tytus env', 'tytus chat', 'tytus link', and 'tytus mcp' fully."
    Write-Host ""
    Write-Host "Docs: $RepoUrl" -ForegroundColor Gray
    Write-Host ""
}

# ── Main ────────────────────────────────────────────────────

$mode = Get-InstallMode
Assert-InstallMode $mode
Show-Banner -Mode $mode

$arch = Get-Arch
Write-Ok "Detected: Windows $arch"

switch ($mode) {
    'production' {
        if (-not (Install-FromRelease)) {
            Stop-ProductionUnavailable $arch
        }
    }
    'dev-source' {
        Write-Warn2 "Developer source install requested explicitly; this is not the grandma-safe production path."
        Install-FromSource
    }
}

Verify-Install
Print-NextSteps

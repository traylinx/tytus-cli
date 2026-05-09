# Public Beta Install Guide

> For `v0.6.14-beta.13`. This is a technical preview, not production GA.

## Start here

Open:

```text
https://get.traylinx.com/
```

Download one file for your computer, open it, then open **Tytus** and follow the setup wizard. The wizard signs you in, lets you pick OpenClaw/NemoClaw or Hermes, starts the private tunnel, and runs the test.

## macOS

Choose the right pkg:

| Mac | File |
|---|---|
| Apple Silicon / M1/M2/M3/M4 | `Tytus-0.6.14-aarch64-apple-darwin-unsigned-PUBLIC-BETA-UNSIGNED.pkg` |
| Intel | `Tytus-0.6.14-x86_64-apple-darwin-unsigned-PUBLIC-BETA-UNSIGNED.pkg` |

Steps:

1. Download the pkg from `https://get.traylinx.com/`.
2. Open it.
3. If macOS blocks it, Control-click the pkg and choose **Open**.
4. Finish the installer.
5. Open **Tytus** from the menu bar or Applications.
6. Follow the setup wizard.

Expected warning: macOS may say the developer cannot be verified. That is expected for this public beta because Apple signing/notarization is deferred.

## Linux Ubuntu/Debian x86_64

Download:

```text
Tytus-0.6.14-x86_64-unknown-linux-gnu-unsigned-PUBLIC-BETA-UNSIGNED.deb
```

Install:

```bash
cd ~/Downloads
sudo apt install ./Tytus-0.6.14-x86_64-unknown-linux-gnu-unsigned-PUBLIC-BETA-UNSIGNED.deb
```

Then open Tytus from the app launcher or run:

```bash
tytus-tray
```

Expected warning: the package is unsigned. That is expected for this public beta because Linux package/repository signing is deferred.

## Windows x86_64

Download:

```text
tytus-windows-x86_64.zip
```

Unzip it and run from PowerShell. Windows is still CLI/MCP technical preview: MSI, SmartScreen signing, Wintun/driver packaging, and full daemon/tray/tunnel parity are GA gates.

Optional PowerShell installer:

```powershell
$env:TYTUS_RELEASE_TAG="v0.6.14-beta.13"; irm https://get.traylinx.com/install.ps1 | iex
```

## Command-line installers

macOS/Linux:

```bash
curl -fsSL https://get.traylinx.com/install.sh | TYTUS_RELEASE_TAG=v0.6.14-beta.13 sh
```

Windows:

```powershell
$env:TYTUS_RELEASE_TAG="v0.6.14-beta.13"; irm https://get.traylinx.com/install.ps1 | iex
```

## First-run wizard

The setup wizard does this:

1. Sign in through the browser.
2. Check your plan.
3. Pick an assistant: OpenClaw/NemoClaw or Hermes.
4. Start the private encrypted tunnel.
5. Run a health test.

Done means `tytus test` passes and TytusOS can reach your pod.

## If something fails

Run:

```bash
tytus doctor
tytus status
tytus test
```

Report the OS, CPU architecture, installer filename, the command output, and a screenshot of the error.

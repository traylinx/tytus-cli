# Public Beta Install Guide

> For `v0.7.12` (live public beta as of 2026-05-27). This is a technical preview, not production GA.

## Start here

Open:

```text
https://get.traylinx.com/
```

Download one file for your computer, open it, then open **Tytus** and follow the setup wizard. The wizard signs you in, lets you pick OpenClaw or Hermes, starts the private tunnel, and runs the test.

## macOS

Choose the right pkg:

| Mac | File |
|---|---|
| Apple Silicon / M1/M2/M3/M4 | `Tytus-0.7.12-aarch64-apple-darwin-unsigned-PUBLIC-BETA-UNSIGNED.pkg` |
| Intel | `Tytus-0.7.12-x86_64-apple-darwin-unsigned-PUBLIC-BETA-UNSIGNED.pkg` |

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
Tytus-0.7.12-x86_64-unknown-linux-gnu-unsigned-PUBLIC-BETA-UNSIGNED.deb
```

Install:

```bash
cd ~/Downloads
sudo apt install ./Tytus-0.7.12-x86_64-unknown-linux-gnu-unsigned-PUBLIC-BETA-UNSIGNED.deb
```

Then open Tytus from the app launcher or run:

```bash
tytus-tray
```

Expected warning: the package is unsigned. That is expected for this public beta because Linux package/repository signing is deferred.

## Windows x86_64

Primary PowerShell installer:

```powershell
powershell -c "irm https://get.traylinx.com/install.ps1 | iex"
```

Direct zip fallback remains available from the GitHub release if PowerShell execution is locked down by policy. Windows is still CLI/MCP technical preview: MSI, SmartScreen signing, Wintun/driver packaging, and full daemon/tray/tunnel parity are GA gates.

Fallback direct package: `tytus-windows-x86_64.zip`.

## Command-line installers

macOS/Linux:

```bash
curl -fsSL https://get.traylinx.com/install.sh | bash
```

Windows:

```powershell
powershell -c "irm https://get.traylinx.com/install.ps1 | iex"
```

## First-run wizard

The setup wizard does this:

1. Sign in through the browser.
2. Check your plan.
3. Pick an assistant: OpenClaw or Hermes.
4. Start the private encrypted tunnel.
5. Run a health test.

Done means `tytus test` passes and TytusOS can reach your pod.

## Update an existing install

From the tray, choose **Settings -> Check for Updates** or **Update Tytus**. CLI fallback:

```bash
tytus update
tytus --version
tytus doctor
```

If your build is too old to have `tytus update`, reinstall from `https://get.traylinx.com/`, then run:

```bash
tytus login
tytus connect
tytus test
```

## If something fails

Run:

```bash
tytus doctor
tytus status
tytus test
```

Report the OS, CPU architecture, installer filename, the command output, and a screenshot of the error.

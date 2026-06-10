use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
#[cfg(not(target_os = "macos"))]
use std::process::{Command, Stdio};

use super::{open, paths};

/// Open a shell command in a new terminal window without putting the command
/// itself on the terminal process argv. The command is written to a private,
/// nonce-named script under the platform runtime dir, then the terminal is
/// asked to execute that script path.
///
/// This keeps API keys and gateway URLs out of `ps`/`/proc/*/cmdline` on Unix
/// and gives product code one launch surface instead of ad-hoc `open`,
/// `osascript`, `gnome-terminal`, or `cmd.exe` calls.
pub fn open_shell_command(command: &str) -> io::Result<()> {
    let script = write_launch_script(command)?;
    launch_script(&script)
}

fn write_launch_script(command: &str) -> io::Result<PathBuf> {
    let dir = paths::runtime_dir().join("launch");
    paths::ensure_private_dir(&dir)?;
    let nonce = random_nonce_hex()?;

    #[cfg(windows)]
    let path = dir.join(format!("launch-{nonce}.cmd"));
    #[cfg(not(windows))]
    let path = dir.join(format!("launch-{nonce}.command"));

    let body = script_body(command);
    write_private_script(&path, body.as_bytes())?;
    Ok(path)
}

fn random_nonce_hex() -> io::Result<String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|e| io::Error::other(e.to_string()))?;
    Ok(hex::encode(bytes))
}

fn write_private_script(path: &Path, body: &[u8]) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o700);
    }
    let mut file = options.open(path)?;
    file.write_all(body)?;
    file.sync_all()?;
    Ok(())
}

#[cfg(windows)]
fn script_body(command: &str) -> String {
    format!(
        "@echo off\r\n\
         set PATH=%USERPROFILE%\\bin;%LOCALAPPDATA%\\Programs\\Tytus;%PATH%\r\n\
         cd /d %USERPROFILE%\r\n\
         del \"%~f0\" >NUL 2>NUL\r\n\
         {command}\r\n"
    )
}

#[cfg(not(windows))]
fn script_body(command: &str) -> String {
    format!(
        "#!/bin/bash\n\
         export PATH=\"$HOME/bin:$HOME/.local/bin:/usr/local/bin:/opt/homebrew/bin:$PATH\"\n\
         for d in \"$HOME\"/.nvm/versions/node/*/bin; do\n\
           [ -d \"$d\" ] && export PATH=\"$d:$PATH\"\n\
         done\n\
         cd \"$HOME\"\n\
         rm -f \"$0\"\n\
         {command}\n"
    )
}

#[cfg(target_os = "macos")]
fn launch_script(path: &Path) -> io::Result<()> {
    open::open_path(path)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn launch_script(path: &Path) -> io::Result<()> {
    let candidates: &[(&str, &[&str])] = &[
        ("x-terminal-emulator", &["-e", "bash"]),
        ("gnome-terminal", &["--", "bash"]),
        ("konsole", &["-e", "bash"]),
        ("xterm", &["-e", "bash"]),
    ];

    for (program, args) in candidates {
        let mut cmd = Command::new(program);
        cmd.args(*args)
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if cmd.spawn().is_ok() {
            return Ok(());
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "no supported terminal emulator found",
    ))
}

#[cfg(windows)]
fn launch_script(path: &Path) -> io::Result<()> {
    // Prefer Windows Terminal when present; fall back to cmd.exe in a new
    // console. The script path is the only argument; secrets remain inside the
    // private script body.
    if Command::new("wt.exe")
        .args(["new-tab", "cmd", "/K"])
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
    {
        return Ok(());
    }

    use std::os::windows::process::CommandExt;
    const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
    Command::new("cmd.exe")
        .args(["/K"])
        .arg(path)
        .creation_flags(CREATE_NEW_CONSOLE)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
}

#[cfg(not(any(unix, windows)))]
fn launch_script(_path: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "terminal launch unsupported on this platform",
    ))
}

#[cfg(test)]
mod tests {
    use super::{random_nonce_hex, script_body};

    #[test]
    fn unix_script_self_deletes_before_running_command() {
        #[cfg(not(windows))]
        {
            let body = script_body("echo ok");
            assert!(body.contains("rm -f \"$0\""));
            assert!(body.contains("echo ok"));
            assert!(body.contains(".nvm/versions/node"));
            assert!(body.starts_with("#!/bin/bash"));
        }
    }

    #[test]
    fn windows_script_self_deletes_before_running_command() {
        #[cfg(windows)]
        {
            let body = script_body("echo ok");
            assert!(body.contains("del \"%~f0\""));
            assert!(body.contains("echo ok"));
            assert!(body.starts_with("@echo off"));
        }
    }

    #[test]
    fn launch_script_nonce_is_128_bit_hex() {
        let nonce = random_nonce_hex().expect("nonce");
        assert_eq!(nonce.len(), 32);
        assert!(nonce.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

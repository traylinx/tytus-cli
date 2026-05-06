use std::io;
use std::path::{Path, PathBuf};
#[cfg(not(target_os = "macos"))]
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

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
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    #[cfg(windows)]
    let path = dir.join(format!("launch-{nonce}.cmd"));
    #[cfg(not(windows))]
    let path = dir.join(format!("launch-{nonce}.command"));

    let body = script_body(command);
    std::fs::write(&path, body)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(path)
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
         export PATH=\"$HOME/bin:/usr/local/bin:/opt/homebrew/bin:$PATH\"\n\
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
    use super::script_body;

    #[test]
    fn unix_script_self_deletes_before_running_command() {
        #[cfg(not(windows))]
        {
            let body = script_body("echo ok");
            assert!(body.contains("rm -f \"$0\""));
            assert!(body.contains("echo ok"));
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
}

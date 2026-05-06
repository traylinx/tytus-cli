use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn open_url(url: &str) -> io::Result<()> {
    spawn_open(url)
}

pub fn open_path(path: &Path) -> io::Result<()> {
    spawn_open(path.as_os_str())
}

fn spawn_open<S: AsRef<std::ffi::OsStr>>(target: S) -> io::Result<()> {
    let mut cmd = open_command();
    cmd.arg(target)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
}

fn open_command() -> Command {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Command::new("true")
    }
}

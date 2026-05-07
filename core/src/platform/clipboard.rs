use std::io::{self, Write};
use std::process::{Command, Stdio};

pub fn read_text() -> io::Result<String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("pbpaste")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()?;
        if output.status.success() {
            return String::from_utf8(output.stdout)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e));
        }
        return Err(io::Error::other("pbpaste failed"));
    }
    #[cfg(target_os = "linux")]
    {
        for (bin, args) in [
            ("wl-paste", &["--no-newline"][..]),
            ("xclip", &["-selection", "clipboard", "-o"][..]),
            ("xsel", &["--clipboard", "--output"][..]),
        ] {
            if let Ok(output) = Command::new(bin)
                .args(args)
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output()
            {
                if output.status.success() {
                    return String::from_utf8(output.stdout)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e));
                }
            }
        }
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "no clipboard reader found (install wl-clipboard, xclip, or xsel)",
        ));
    }
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", "Get-Clipboard -Raw"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()?;
        if output.status.success() {
            return String::from_utf8(output.stdout)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e));
        }
        return Err(io::Error::other("Get-Clipboard failed"));
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "system clipboard not supported on this platform",
        ))
    }
}

pub fn write_text(text: &str) -> io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        return write_to_command("pbcopy", &[], text);
    }
    #[cfg(target_os = "linux")]
    {
        for (bin, args) in [
            ("wl-copy", &[][..]),
            ("xclip", &["-selection", "clipboard"][..]),
            ("xsel", &["--clipboard", "--input"][..]),
        ] {
            if write_to_command(bin, args, text).is_ok() {
                return Ok(());
            }
        }
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "no clipboard writer found (install wl-clipboard, xclip, or xsel)",
        ));
    }
    #[cfg(target_os = "windows")]
    {
        return write_to_command(
            "powershell",
            &["-NoProfile", "-Command", "Set-Clipboard"],
            text,
        );
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = text;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "system clipboard not supported on this platform",
        ))
    }
}

fn write_to_command(bin: &str, args: &[&str], text: &str) -> io::Result<()> {
    let mut child = Command::new(bin)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    }
    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("{} failed", bin)))
    }
}

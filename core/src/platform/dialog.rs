use std::io;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogAnswer {
    Accepted,
    Cancelled,
    Unsupported,
}

pub fn show_error(title: &str, message: &str) -> io::Result<DialogAnswer> {
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "display dialog {} with title {} buttons {{\"OK\"}} default button \"OK\" with icon caution",
            applescript_quote(message),
            applescript_quote(title)
        );
        let status = Command::new("osascript")
            .args(["-e", &script])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        return Ok(if status.success() {
            DialogAnswer::Accepted
        } else {
            DialogAnswer::Cancelled
        });
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (title, message);
        Ok(DialogAnswer::Unsupported)
    }
}

pub fn ask_permission(title: &str, message: &str) -> io::Result<DialogAnswer> {
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "display dialog {} with title {} buttons {{\"Cancel\", \"OK\"}} default button \"OK\" with icon caution",
            applescript_quote(message),
            applescript_quote(title)
        );
        let status = Command::new("osascript")
            .args(["-e", &script])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        return Ok(if status.success() {
            DialogAnswer::Accepted
        } else {
            DialogAnswer::Cancelled
        });
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (title, message);
        Ok(DialogAnswer::Unsupported)
    }
}

#[cfg(target_os = "macos")]
fn applescript_quote(s: &str) -> String {
    format!("{:?}", s)
}

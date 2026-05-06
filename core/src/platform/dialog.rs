use std::io;
#[cfg(target_os = "macos")]
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogAnswer {
    Accepted,
    Cancelled,
    Unsupported,
}

pub fn show_error(title: &str, message: &str) -> io::Result<DialogAnswer> {
    show_dialog(title, message, "caution")
}

pub fn show_info(title: &str, message: &str) -> io::Result<DialogAnswer> {
    show_dialog(title, message, "note")
}

fn show_dialog(title: &str, message: &str, icon: &str) -> io::Result<DialogAnswer> {
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "display dialog {} with title {} buttons {{\"OK\"}} default button \"OK\" with icon {}",
            applescript_quote(message),
            applescript_quote(title),
            icon,
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
        let _ = (title, message, icon);
        Ok(DialogAnswer::Unsupported)
    }
}

pub fn ask_permission(title: &str, message: &str) -> io::Result<DialogAnswer> {
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "display dialog {} with title {} buttons {{\"Cancel\", \"OK\"}} default button \"Cancel\" cancel button \"Cancel\" with icon caution",
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

pub fn notify(title: &str, message: &str) -> io::Result<DialogAnswer> {
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "display notification {} with title {}",
            applescript_quote(message),
            applescript_quote(title),
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

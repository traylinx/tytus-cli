use std::io;
#[cfg(target_os = "macos")]
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogAnswer {
    Accepted,
    Cancelled,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickKind {
    File,
    Folder,
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

pub fn prompt_text(
    title: &str,
    message: &str,
    default: Option<&str>,
) -> io::Result<Option<String>> {
    #[cfg(target_os = "macos")]
    {
        let default_clause = default
            .map(|d| format!(" default answer {}", applescript_quote(d)))
            .unwrap_or_else(|| " default answer \"\"".to_string());
        let script = format!(
            "set r to display dialog {} with title {}{}
text returned of r",
            applescript_quote(message),
            applescript_quote(title),
            default_clause,
        );
        let output = Command::new("osascript")
            .args(["-e", &script])
            .stdin(Stdio::null())
            .output()?;
        if !output.status.success() {
            return Ok(None);
        }
        let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok(if value.is_empty() { None } else { Some(value) });
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (title, message, default);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "text prompt unsupported on this platform",
        ))
    }
}

pub fn pick_path(kind: PickKind, prompt: &str) -> io::Result<Option<String>> {
    #[cfg(target_os = "macos")]
    {
        let verb = match kind {
            PickKind::File => "choose file",
            PickKind::Folder => "choose folder",
        };
        let script = format!(
            "POSIX path of ({} with prompt {})",
            verb,
            applescript_quote(prompt),
        );
        let output = Command::new("osascript")
            .args(["-e", &script])
            .stdin(Stdio::null())
            .output()?;
        if !output.status.success() {
            return Ok(None);
        }
        let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok(if value.is_empty() { None } else { Some(value) });
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (kind, prompt);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "file picker unsupported on this platform",
        ))
    }
}

#[cfg(target_os = "macos")]
fn applescript_quote(s: &str) -> String {
    format!("{:?}", s)
}

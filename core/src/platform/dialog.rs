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
        Ok(if status.success() {
            DialogAnswer::Accepted
        } else {
            DialogAnswer::Cancelled
        })
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
        Ok(if status.success() {
            DialogAnswer::Accepted
        } else {
            DialogAnswer::Cancelled
        })
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
        Ok(if status.success() {
            DialogAnswer::Accepted
        } else {
            DialogAnswer::Cancelled
        })
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (title, message);
        Ok(DialogAnswer::Unsupported)
    }
}

pub fn choose_button(
    title: &str,
    message: &str,
    buttons: &[&str],
    default_button: &str,
    cancel_button: Option<&str>,
    icon: &str,
) -> io::Result<Option<String>> {
    #[cfg(target_os = "macos")]
    {
        let button_list = buttons
            .iter()
            .map(|button| applescript_quote(button))
            .collect::<Vec<_>>()
            .join(", ");
        let cancel_clause = cancel_button
            .map(|button| format!(" cancel button {}", applescript_quote(button)))
            .unwrap_or_default();
        let script = format!(
            "set r to display dialog {} with title {} buttons {{{}}} default button {}{} with icon {}
button returned of r",
            applescript_quote(message),
            applescript_quote(title),
            button_list,
            applescript_quote(default_button),
            cancel_clause,
            icon,
        );
        let output = Command::new("osascript")
            .args(["-e", &script])
            .stdin(Stdio::null())
            .output()?;
        if !output.status.success() {
            return Ok(None);
        }
        let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(if value.is_empty() { None } else { Some(value) })
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (title, message, buttons, default_button, cancel_button, icon);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "button dialog unsupported on this platform",
        ))
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
        Ok(if value.is_empty() { None } else { Some(value) })
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
        Ok(if value.is_empty() { None } else { Some(value) })
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

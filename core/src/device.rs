use sha2::{Sha256, Digest};

/// Generate a stable device fingerprint.
/// SHA-256 of hostname:username:homedir — same approach as claurst's bridge.
/// Always returns 64 hex chars. Stable across app restarts.
pub fn device_fingerprint() -> String {
    let mut input = String::with_capacity(128);

    // Hostname
    if let Ok(host) = hostname::get() {
        input.push_str(&host.to_string_lossy());
    }
    input.push(':');

    // Username
    if let Ok(user) = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
    {
        input.push_str(&user);
    }
    input.push(':');

    // Home directory
    if let Some(home) = dirs::home_dir() {
        input.push_str(&home.display().to_string());
    }

    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

// hostname crate isn't a dependency — use gethostname from std
mod hostname {
    pub fn get() -> std::io::Result<std::ffi::OsString> {
        #[cfg(unix)]
        {
            use std::ffi::CStr;
            let mut buf = [0u8; 256];
            let ret = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut i8, buf.len()) };
            if ret == 0 {
                let cstr = unsafe { CStr::from_ptr(buf.as_ptr() as *const i8) };
                Ok(std::ffi::OsString::from(cstr.to_string_lossy().into_owned()))
            } else {
                Err(std::io::Error::last_os_error())
            }
        }
        #[cfg(not(unix))]
        {
            // Fallback for non-unix
            Ok(std::ffi::OsString::from("unknown"))
        }
    }
}

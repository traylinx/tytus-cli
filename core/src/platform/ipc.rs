use crate::platform::paths;
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};

pub const CONTROL_SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_BIND_IP: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlFile {
    pub schema_version: u32,
    pub bind: IpAddr,
    pub port: u16,
    pub token_file: PathBuf,
    pub pid: u32,
    pub started_at: DateTime<Utc>,
}

impl ControlFile {
    pub fn new(port: u16, pid: u32) -> Self {
        Self {
            schema_version: CONTROL_SCHEMA_VERSION,
            bind: DEFAULT_BIND_IP,
            port,
            token_file: paths::control_token_file(),
            pid,
            started_at: Utc::now(),
        }
    }

    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.bind, self.port)
    }
}

pub fn write_control_file(path: &Path, control: &ControlFile) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        paths::ensure_private_dir(parent)?;
    }
    let body =
        serde_json::to_vec_pretty(control).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(path, body)
}

pub fn read_control_file(path: &Path) -> io::Result<ControlFile> {
    let body = fs::read(path)?;
    serde_json::from_slice(&body).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

pub fn generate_control_token() -> io::Result<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes))
}

pub fn write_control_token_file(path: &Path, token: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        paths::ensure_private_dir(parent)?;
    }
    fs::write(path, token.as_bytes())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

pub fn read_control_token_file(path: &Path) -> io::Result<String> {
    fs::read_to_string(path).map(|s| s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_file_round_trips() {
        let dir = std::env::temp_dir().join(format!(
            "tytus-control-test-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let path = dir.join("control.json");
        let mut c = ControlFile::new(49152, 1234);
        c.token_file = dir.join("control.token");
        write_control_file(&path, &c).unwrap();
        let read = read_control_file(&path).unwrap();
        assert_eq!(read.schema_version, CONTROL_SCHEMA_VERSION);
        assert_eq!(read.socket_addr(), SocketAddr::new(DEFAULT_BIND_IP, 49152));
        assert_eq!(read.pid, 1234);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn control_token_is_urlsafe_and_round_trips() {
        let token = generate_control_token().unwrap();
        assert_eq!(token.len(), 43);
        assert!(token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));

        let dir = std::env::temp_dir().join(format!(
            "tytus-token-test-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let path = dir.join("control.token");
        write_control_token_file(&path, &token).unwrap();
        assert_eq!(read_control_token_file(&path).unwrap(), token);
        let _ = fs::remove_dir_all(dir);
    }
}

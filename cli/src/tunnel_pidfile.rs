use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TunnelPidfile {
    pub pid: i32,
    pub account: Option<String>,
    pub pod_id: Option<String>,
    pub iface: Option<String>,
    pub started_at: Option<i64>,
    pub legacy: bool,
}

pub fn canonical_email(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}

pub fn account_hash(email: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(canonical_email(email).as_bytes());
    let digest = hasher.finalize();
    let mut out = String::with_capacity(16);
    for b in digest.iter().take(8) {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

pub fn parse(contents: &str) -> Option<TunnelPidfile> {
    let trimmed = contents.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.chars().all(|c| c.is_ascii_digit()) {
        return trimmed.parse::<i32>().ok().map(|pid| TunnelPidfile {
            pid,
            account: None,
            pod_id: None,
            iface: None,
            started_at: None,
            legacy: true,
        });
    }

    let mut pid = None;
    let mut account = None;
    let mut pod_id = None;
    let mut iface = None;
    let mut started_at = None;
    for line in contents.lines() {
        let (k, v) = line.split_once('=')?;
        let key = k.trim();
        let val = v.trim();
        match key {
            "pid" => pid = val.parse::<i32>().ok(),
            "account" => account = Some(val.to_string()),
            "pod_id" => pod_id = Some(val.to_string()),
            "iface" => iface = Some(val.to_string()),
            "started_at" => started_at = val.parse::<i64>().ok(),
            _ => {}
        }
    }
    Some(TunnelPidfile {
        pid: pid?,
        account,
        pod_id,
        iface,
        started_at,
        legacy: false,
    })
}

pub fn read(path: &Path) -> Option<TunnelPidfile> {
    std::fs::read_to_string(path).ok().and_then(|s| parse(&s))
}

pub fn owner_matches(meta: &TunnelPidfile, email: &str) -> bool {
    meta.account.as_deref() == Some(account_hash(email).as_str())
}

pub fn write(
    path: &Path,
    pid: i32,
    email: &str,
    pod_id: &str,
    iface: Option<&str>,
) -> std::io::Result<()> {
    let started_at = chrono::Utc::now().timestamp();
    let body = format!(
        "pid={}\naccount={}\npod_id={}\niface={}\nstarted_at={}\n",
        pid,
        account_hash(email),
        pod_id,
        iface.unwrap_or(""),
        started_at
    );
    std::fs::write(path, body)
}

pub fn write_legacy(path: &Path, pid: i32) -> std::io::Result<()> {
    std::fs::write(path, format!("{}", pid))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_canonicalizes_email() {
        assert_eq!(
            account_hash(" User@Example.COM "),
            account_hash("user@example.com")
        );
        assert_eq!(account_hash("user@example.com").len(), 16);
    }

    #[test]
    fn parses_legacy_pid() {
        let p = parse("12345\n").unwrap();
        assert_eq!(p.pid, 12345);
        assert!(p.legacy);
    }

    #[test]
    fn parses_metadata_pidfile() {
        let p = parse("pid=42\naccount=abcdef\npod_id=02\niface=utun7\nstarted_at=123\n").unwrap();
        assert_eq!(p.pid, 42);
        assert_eq!(p.account.as_deref(), Some("abcdef"));
        assert_eq!(p.pod_id.as_deref(), Some("02"));
        assert_eq!(p.iface.as_deref(), Some("utun7"));
        assert!(!p.legacy);
    }
}

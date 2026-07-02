//! Bucket access for the consumer: a small trait so tests use an in-memory
//! store and a native client can replace the subprocess later, plus the
//! rclone implementation (already-installed binary, existing `[garagetytus]`
//! remote, zero new credentials).
//!
//! Policy (TECH-SPEC §8): every invocation is prefix/key-scoped — a tree-wide
//! flag in this module is a code-review reject. `lsjson` 30 s, `cat` 10 s with
//! the 8 KiB event guard, `copyto` 300 s base. The CONSUMER path is pull-only:
//! this module exposes no write or delete operation at all.

use std::path::Path;
use std::process::Command;

pub const MAX_EVENT_OBJECT_BYTES: usize = 8 * 1024;
pub const LSJSON_TIMEOUT_SECS: u32 = 30;
pub const CAT_TIMEOUT_SECS: u32 = 10;
pub const COPYTO_TIMEOUT_SECS: u32 = 300;

#[derive(Debug, Clone)]
pub struct RemoteObject {
    pub key: String,
    pub size: u64,
}

#[derive(Debug)]
pub enum S3Error {
    /// Transient: retry against the event's backoff budget.
    Transient(String),
    /// The object does not exist (blob GC'd, listing raced a delete).
    NotFound(String),
    /// Permanent for this attempt (bad output shape, oversize guard).
    Permanent(String),
}

impl std::fmt::Display for S3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transient(m) => write!(f, "transient: {m}"),
            Self::NotFound(m) => write!(f, "not found: {m}"),
            Self::Permanent(m) => write!(f, "permanent: {m}"),
        }
    }
}

impl std::error::Error for S3Error {}

pub trait S3Ops {
    /// Bounded listing of `prefix`, keys AFTER `start_after` only (G0
    /// decision: StartAfter is the normative steady-state poll shape).
    fn list_after(&self, prefix: &str, start_after: Option<&str>, max: usize)
        -> Result<Vec<RemoteObject>, S3Error>;
    /// Fetch a small object (event JSON) with the 8 KiB guard.
    fn cat_small(&self, key: &str) -> Result<Vec<u8>, S3Error>;
    /// Download one object to a local temp path (same-filesystem apply).
    fn download(&self, key: &str, target: &Path) -> Result<(), S3Error>;
}

/// rclone subprocess implementation against `remote:bucket`.
pub struct RcloneS3 {
    pub rclone_bin: String,
    pub rclone_conf: Option<String>,
    pub remote: String,
    pub bucket: String,
}

impl RcloneS3 {
    pub fn new(remote: &str, bucket: &str, rclone_conf: Option<String>) -> Self {
        Self {
            rclone_bin: "/usr/local/bin/rclone".into(),
            rclone_conf,
            remote: remote.into(),
            bucket: bucket.into(),
        }
    }

    fn base(&self, timeout_secs: u32) -> Command {
        let mut cmd = Command::new(&self.rclone_bin);
        if let Some(conf) = &self.rclone_conf {
            cmd.arg("--config").arg(conf);
        }
        cmd.arg("--contimeout").arg("10s");
        cmd.arg("--timeout").arg(format!("{timeout_secs}s"));
        cmd.arg("--retries").arg("1");
        cmd.arg("--low-level-retries").arg("2");
        cmd
    }

    fn target(&self, key: &str) -> String {
        format!("{}:{}/{}", self.remote, self.bucket, key)
    }
}

impl S3Ops for RcloneS3 {
    fn list_after(&self, prefix: &str, start_after: Option<&str>, max: usize)
        -> Result<Vec<RemoteObject>, S3Error>
    {
        let output = self
            .base(LSJSON_TIMEOUT_SECS)
            .arg("lsjson")
            .arg("--files-only")
            .arg("--no-modtime")
            .arg("--no-mimetype")
            .arg(self.target(prefix))
            .output()
            .map_err(|e| S3Error::Transient(format!("spawn rclone: {e}")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("directory not found") || stderr.contains("not found") {
                return Ok(Vec::new()); // empty prefix — NOT an error, NOT a gap
            }
            return Err(S3Error::Transient(stderr.chars().take(300).collect()));
        }
        let parsed: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)
            .map_err(|e| S3Error::Permanent(format!("lsjson parse: {e}")))?;
        let mut rows: Vec<RemoteObject> = parsed
            .iter()
            .filter_map(|v| {
                let name = v.get("Path").and_then(|p| p.as_str())?;
                let size = v.get("Size").and_then(|s| s.as_u64()).unwrap_or(0);
                Some(RemoteObject { key: format!("{prefix}{name}"), size })
            })
            .collect();
        rows.sort_by(|a, b| a.key.cmp(&b.key)); // lexicographic == sequence order
        let rows = rows
            .into_iter()
            .filter(|o| start_after.map_or(true, |s| o.key.as_str() > s))
            .take(max)
            .collect();
        Ok(rows)
    }

    fn cat_small(&self, key: &str) -> Result<Vec<u8>, S3Error> {
        let output = self
            .base(CAT_TIMEOUT_SECS)
            .arg("cat")
            .arg("--count")
            .arg((MAX_EVENT_OBJECT_BYTES + 1).to_string())
            .arg(self.target(key))
            .output()
            .map_err(|e| S3Error::Transient(format!("spawn rclone: {e}")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("not found") || stderr.contains("404") {
                return Err(S3Error::NotFound(key.into()));
            }
            return Err(S3Error::Transient(stderr.chars().take(300).collect()));
        }
        if output.stdout.len() > MAX_EVENT_OBJECT_BYTES {
            return Err(S3Error::Permanent(format!("event object over {MAX_EVENT_OBJECT_BYTES} bytes")));
        }
        Ok(output.stdout)
    }

    fn download(&self, key: &str, target: &Path) -> Result<(), S3Error> {
        let output = self
            .base(COPYTO_TIMEOUT_SECS)
            .arg("copyto")
            .arg(self.target(key))
            .arg(target)
            .output()
            .map_err(|e| S3Error::Transient(format!("spawn rclone: {e}")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("not found") || stderr.contains("404") {
                return Err(S3Error::NotFound(key.into()));
            }
            return Err(S3Error::Transient(stderr.chars().take(300).collect()));
        }
        if !target.exists() {
            return Err(S3Error::Transient("copyto reported success but target missing".into()));
        }
        Ok(())
    }
}

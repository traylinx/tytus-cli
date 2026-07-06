//! Bucket access for the consumer: a small trait so tests use an in-memory
//! store and a native client can replace the subprocess later, plus the
//! rclone implementation (already-installed binary, existing `[garagetytus]`
//! remote, zero new credentials).
//!
//! Policy (TECH-SPEC §8): every invocation is prefix/key-scoped — a tree-wide
//! flag in this module is a code-review reject. `lsjson` 30 s, `cat` 10 s with
//! the 8 KiB event guard, `copyto` 300 s base. The CONSUMER path is pull-only:
//! this module exposes no write or delete operation at all.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub const MAX_EVENT_OBJECT_BYTES: usize = 8 * 1024;
pub const LSJSON_TIMEOUT_SECS: u32 = 30;
pub const CAT_TIMEOUT_SECS: u32 = 10;
pub const COPYTO_TIMEOUT_SECS: u32 = 300;
/// Wall-clock cap = 2x the rclone idle timeout + this slack. rclone's
/// --timeout is an IDLE timeout: a transfer that trickles bytes forever
/// never trips it, and `Command::output()` waits forever — one download
/// attempt over a degraded path ran 15-30+ minutes, so the retry budget in
/// apply.rs accrued too slowly to ever demote the event (F-SOAK-4,
/// 2026-07-06). The wall cap turns a hang into a countable failed attempt.
pub const WALL_CAP_SLACK_SECS: u64 = 30;

fn wall_cap(idle_timeout_secs: u32) -> Duration {
    Duration::from_secs(u64::from(idle_timeout_secs) * 2 + WALL_CAP_SLACK_SECS)
}

/// Run a subprocess with a hard wall-clock deadline. Stdout/stderr are
/// drained on threads so a chatty child can never deadlock against a full
/// pipe while we wait. On deadline: kill + reap + Transient error.
fn run_wall_capped(mut cmd: Command, cap: Duration) -> Result<std::process::Output, S3Error> {
    cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd
        .spawn()
        .map_err(|e| S3Error::Transient(format!("spawn rclone: {e}")))?;
    let mut out_pipe = child
        .stdout
        .take()
        .ok_or_else(|| S3Error::Transient("rclone stdout pipe missing".into()))?;
    let mut err_pipe = child
        .stderr
        .take()
        .ok_or_else(|| S3Error::Transient("rclone stderr pipe missing".into()))?;
    let out_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = out_pipe.read_to_end(&mut buf);
        buf
    });
    let err_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = err_pipe.read_to_end(&mut buf);
        buf
    });
    let deadline = Instant::now() + cap;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = out_thread.join();
                    let _ = err_thread.join();
                    return Err(S3Error::Transient(format!(
                        "wall-clock cap {}s exceeded",
                        cap.as_secs()
                    )));
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = out_thread.join();
                let _ = err_thread.join();
                return Err(S3Error::Transient(format!("wait rclone: {e}")));
            }
        }
    };
    let stdout = out_thread.join().unwrap_or_default();
    let stderr = err_thread.join().unwrap_or_default();
    Ok(std::process::Output { status, stdout, stderr })
}

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
        let mut cmd = self.base(LSJSON_TIMEOUT_SECS);
        cmd.arg("lsjson")
            .arg("--files-only")
            .arg("--no-modtime")
            .arg("--no-mimetype")
            .arg(self.target(prefix));
        let output = run_wall_capped(cmd, wall_cap(LSJSON_TIMEOUT_SECS))?;
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
        let mut cmd = self.base(CAT_TIMEOUT_SECS);
        cmd.arg("cat")
            .arg("--count")
            .arg((MAX_EVENT_OBJECT_BYTES + 1).to_string())
            .arg(self.target(key));
        let output = run_wall_capped(cmd, wall_cap(CAT_TIMEOUT_SECS))?;
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
        let mut cmd = self.base(COPYTO_TIMEOUT_SECS);
        cmd.arg("copyto").arg(self.target(key)).arg(target);
        let output = run_wall_capped(cmd, wall_cap(COPYTO_TIMEOUT_SECS))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    fn sh(script: &str) -> Command {
        let mut cmd = Command::new("/bin/sh");
        cmd.arg("-c").arg(script);
        cmd
    }

    #[cfg(unix)]
    #[test]
    fn wall_cap_kills_a_hung_child_and_reports_transient() {
        let started = Instant::now();
        let err = run_wall_capped(sh("sleep 30"), Duration::from_millis(300)).unwrap_err();
        assert!(started.elapsed() < Duration::from_secs(5), "child was not killed promptly");
        match err {
            S3Error::Transient(msg) => assert!(msg.contains("wall-clock cap"), "{msg}"),
            other => panic!("expected Transient, got {other}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn wall_cap_passes_through_a_fast_child_with_output() {
        let out = run_wall_capped(sh("printf hello; printf world >&2; exit 3"),
                                  Duration::from_secs(10)).unwrap();
        assert_eq!(out.stdout, b"hello");
        assert_eq!(out.stderr, b"world");
        assert_eq!(out.status.code(), Some(3));
    }

    #[cfg(unix)]
    #[test]
    fn wall_cap_drains_output_larger_than_a_pipe_buffer() {
        // 256 KiB >> the 64 KiB pipe buffer: without the reader threads the
        // child would block on write and the wait loop would hit the cap.
        let out = run_wall_capped(sh("head -c 262144 /dev/zero"), Duration::from_secs(10)).unwrap();
        assert_eq!(out.stdout.len(), 262_144);
        assert!(out.status.success());
    }

    #[test]
    fn wall_cap_scales_from_idle_timeout() {
        assert_eq!(wall_cap(300).as_secs(), 630);
        assert_eq!(wall_cap(10).as_secs(), 50);
    }
}

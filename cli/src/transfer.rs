// ============================================================
// transfer — shared helpers for `tytus push/pull/ls/rm/transfers`
// ============================================================
// Path validation, size ceiling, smart --pod resolution,
// and the append-only JSONL transfer log (flock-serialised).
// ============================================================

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::state::CliState;

// ── Constants ───────────────────────────────────────────────

/// Only writable path inside a pod. Anything outside this is
/// refused before a single byte hits the wire. See sprint LM#1.
pub const POD_WORKSPACE_ROOT: &str = "/app/workspace/";

/// Default inbox. Created on push if missing.
pub const POD_INBOX: &str = "/app/workspace/inbox/";

/// Default outbox (pulled from).
#[allow(dead_code)]
pub const POD_OUTBOX: &str = "/app/workspace/out/";

/// Hard upper bound on a single push/pull. Above this, point at
/// the v0.7 Garage sprint — docker-exec base64 streaming is the
/// wrong tool for GB-scale. Deliberate feature, not a TODO.
pub const MAX_TRANSFER_BYTES: u64 = 100 * 1024 * 1024;

/// Progress-bar threshold. Below this, transfers are silent.
pub const PROGRESS_THRESHOLD_BYTES: u64 = 1024 * 1024;

/// Shell-argument chunk size for chunked base64 upload. 256 KB
/// payload = ~350 KB base64, comfortably under Linux ARG_MAX
/// (~2 MB) and macOS ARG_MAX (~256 KB for execve, but the pod
/// is Linux). Kept identical for pull so chunk math matches.
pub const CHUNK_PAYLOAD_BYTES: usize = 256 * 1024;

// ── Errors ──────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum TransferError {
    #[error("path outside /app/workspace/ is not allowed: {0}")]
    PathOutsideWorkspace(String),
    #[error("path contains '..' or escape segments: {0}")]
    PathEscape(String),
    #[error("path contains NUL byte")]
    PathNul,
    #[error(
        "transfer exceeds 100 MB ceiling ({0} bytes). For GB-scale transfers, \
         use the Garage-backed shared filesystem (planned for v0.7)."
    )]
    SizeCeiling(u64),
    #[error("no pods connected. Run: tytus connect")]
    NoPods,
    #[error(
        "multiple pods connected ({0:?}); specify --pod NN"
    )]
    AmbiguousPod(Vec<String>),
    #[error("local path does not exist: {0}")]
    LocalMissing(String),
}

// ── Path validation ─────────────────────────────────────────

/// Reject any path the CLI should never ship bytes to/from.
///
/// Rules:
///  - must start with `/app/workspace/`
///  - no NUL bytes
///  - no `..` component (even after `/app/workspace/` prefix;
///    we refuse rather than canonicalise because we can't stat
///    the pod side anyway).
pub fn validate_pod_path(raw: &str) -> Result<String, TransferError> {
    if raw.as_bytes().contains(&0) {
        return Err(TransferError::PathNul);
    }
    if !raw.starts_with(POD_WORKSPACE_ROOT) {
        return Err(TransferError::PathOutsideWorkspace(raw.to_string()));
    }
    for seg in raw.split('/') {
        if seg == ".." {
            return Err(TransferError::PathEscape(raw.to_string()));
        }
    }
    Ok(raw.to_string())
}

/// Normalise a user-supplied `--to` for push. Empty or missing
/// defaults to the inbox. Trailing `/` is preserved so callers
/// know "this is a directory destination".
pub fn resolve_push_destination(
    local: &Path,
    to: Option<&str>,
) -> Result<String, TransferError> {
    let base = to.unwrap_or(POD_INBOX);
    validate_pod_path(base)?;
    // If user gave a bare directory (ends in /) we append the
    // local basename so `push report.pdf` lands at
    // `/app/workspace/inbox/report.pdf`. If they gave a full
    // path (no trailing /), respect it verbatim.
    if base.ends_with('/') {
        let name = local
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| TransferError::LocalMissing(local.display().to_string()))?;
        Ok(format!("{}{}", base, name))
    } else {
        Ok(base.to_string())
    }
}

// ── Size enforcement ────────────────────────────────────────

pub fn enforce_size_ceiling(bytes: u64) -> Result<(), TransferError> {
    if bytes > MAX_TRANSFER_BYTES {
        Err(TransferError::SizeCeiling(bytes))
    } else {
        Ok(())
    }
}

// ── Smart --pod default ─────────────────────────────────────

/// If the caller gave `--pod`, return it verbatim. Otherwise
/// auto-pick when there's exactly one connected pod, else
/// surface the list so the user knows which to pick.
pub fn resolve_pod(explicit: Option<&str>, state: &CliState) -> Result<String, TransferError> {
    if let Some(p) = explicit {
        return Ok(p.to_string());
    }
    match state.pods.len() {
        0 => Err(TransferError::NoPods),
        1 => Ok(state.pods[0].pod_id.clone()),
        _ => Err(TransferError::AmbiguousPod(
            state.pods.iter().map(|p| p.pod_id.clone()).collect(),
        )),
    }
}

// ── Transfer log (JSONL, flock-serialised) ─────────────────

/// Where transfer events land. Honours `XDG_DATA_HOME` on
/// Linux; uses `~/Library/Application Support/tytus/` on macOS.
pub fn transfer_log_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        if let Some(h) = dirs::home_dir() {
            return h.join("Library/Application Support/tytus/transfers.log");
        }
    }
    // Linux / BSD / others.
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("tytus/transfers.log");
        }
    }
    if let Some(h) = dirs::home_dir() {
        return h.join(".local/share/tytus/transfers.log");
    }
    PathBuf::from("/tmp/tytus/transfers.log")
}

/// One row of the append-only JSONL log.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TransferEvent {
    pub ts: String,
    pub verb: String,
    pub pod: String,
    pub remote: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local: Option<String>,
    pub size_bytes: u64,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub err_reason: Option<String>,
}

impl TransferEvent {
    pub fn now(
        verb: &str,
        pod: &str,
        remote: &str,
        local: Option<&str>,
        size_bytes: u64,
        ok: bool,
        err_reason: Option<&str>,
    ) -> Self {
        TransferEvent {
            ts: chrono::Utc::now().to_rfc3339(),
            verb: verb.to_string(),
            pod: pod.to_string(),
            remote: remote.to_string(),
            local: local.map(|s| s.to_string()),
            size_bytes,
            ok,
            err_reason: err_reason.map(|s| s.to_string()),
        }
    }
}

/// Append one event to the transfer log. Holds an advisory
/// exclusive flock across the write so concurrent `tytus push`
/// processes cannot interleave lines.
pub fn append_transfer_log(ev: &TransferEvent) -> std::io::Result<()> {
    let path = transfer_log_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;

    // Serialise to one line first so the flock window is minimal.
    let mut line = serde_json::to_string(ev).unwrap_or_else(|_| "{}".into());
    line.push('\n');

    write_with_flock(&f, line.as_bytes())?;
    Ok(())
}

#[cfg(unix)]
fn write_with_flock(file: &File, buf: &[u8]) -> std::io::Result<()> {
    use std::os::unix::io::AsRawFd;
    let fd = file.as_raw_fd();
    // LOCK_EX blocks until granted. Short critical section —
    // one small write then release.
    let rc = unsafe { libc::flock(fd, libc::LOCK_EX) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let mut f = file;
    let write_res = f.write_all(buf).and_then(|_| f.flush());
    // Always release, even if the write failed.
    unsafe {
        let _ = libc::flock(fd, libc::LOCK_UN);
    }
    write_res
}

#[cfg(not(unix))]
fn write_with_flock(file: &File, buf: &[u8]) -> std::io::Result<()> {
    let mut f = file;
    f.write_all(buf)?;
    f.flush()
}

// ── Shell-escape helper (POSIX sh, dash-safe) ───────────────

/// Single-quote a string for POSIX `sh` — wraps in `'...'` and
/// escapes embedded single quotes via `'\''`. Safe for dash.
pub fn shell_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

// ── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn rejects_path_outside_workspace() {
        assert!(matches!(
            validate_pod_path("/etc/passwd"),
            Err(TransferError::PathOutsideWorkspace(_))
        ));
        assert!(matches!(
            validate_pod_path("/root/evil"),
            Err(TransferError::PathOutsideWorkspace(_))
        ));
    }

    #[test]
    fn rejects_dotdot_segment() {
        assert!(matches!(
            validate_pod_path("/app/workspace/../etc/passwd"),
            Err(TransferError::PathEscape(_))
        ));
    }

    #[test]
    fn rejects_nul_byte() {
        assert!(matches!(
            validate_pod_path("/app/workspace/foo\0bar"),
            Err(TransferError::PathNul)
        ));
    }

    #[test]
    fn accepts_valid_workspace_path() {
        assert_eq!(
            validate_pod_path("/app/workspace/inbox/report.pdf").unwrap(),
            "/app/workspace/inbox/report.pdf"
        );
    }

    #[test]
    fn size_ceiling_rejects_over_limit() {
        assert!(enforce_size_ceiling(MAX_TRANSFER_BYTES + 1).is_err());
        assert!(enforce_size_ceiling(MAX_TRANSFER_BYTES).is_ok());
        assert!(enforce_size_ceiling(0).is_ok());
    }

    #[test]
    fn resolve_push_destination_default_is_inbox() {
        let dest = resolve_push_destination(Path::new("/tmp/report.pdf"), None).unwrap();
        assert_eq!(dest, "/app/workspace/inbox/report.pdf");
    }

    #[test]
    fn resolve_push_destination_trailing_slash_appends_basename() {
        let dest = resolve_push_destination(
            Path::new("/tmp/report.pdf"),
            Some("/app/workspace/sub/"),
        )
        .unwrap();
        assert_eq!(dest, "/app/workspace/sub/report.pdf");
    }

    #[test]
    fn resolve_push_destination_explicit_path_preserved() {
        let dest = resolve_push_destination(
            Path::new("/tmp/report.pdf"),
            Some("/app/workspace/sub/renamed.pdf"),
        )
        .unwrap();
        assert_eq!(dest, "/app/workspace/sub/renamed.pdf");
    }

    #[test]
    fn shell_escape_handles_single_quotes() {
        assert_eq!(shell_escape("foo"), "'foo'");
        assert_eq!(shell_escape("fo'o"), "'fo'\\''o'");
        assert_eq!(shell_escape(""), "''");
    }

    #[test]
    fn transfer_log_path_respects_xdg_data_home() {
        // macOS skips the XDG branch; no meaningful assert there.
        if cfg!(target_os = "linux") {
            std::env::set_var("XDG_DATA_HOME", "/tmp/xdg-test");
            let p = transfer_log_path();
            assert_eq!(p, PathBuf::from("/tmp/xdg-test/tytus/transfers.log"));
            std::env::remove_var("XDG_DATA_HOME");
        }
    }

    #[test]
    fn transfer_log_append_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let log_path = tmp.path().join("transfers.log");
        // We can't easily override transfer_log_path() without
        // env vars, so write directly using the same flock path
        // to exercise the codepath.
        let f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .unwrap();
        let ev = TransferEvent::now("push", "02", "/app/workspace/inbox/x", Some("/tmp/x"), 42, true, None);
        let mut line = serde_json::to_string(&ev).unwrap();
        line.push('\n');
        write_with_flock(&f, line.as_bytes()).unwrap();

        let contents = std::fs::read_to_string(&log_path).unwrap();
        assert!(contents.contains("\"verb\":\"push\""));
        assert!(contents.contains("\"pod\":\"02\""));
        assert!(contents.ends_with('\n'));
    }

    #[test]
    fn transfer_log_concurrent_writers_do_not_interleave() {
        // 8 writer threads × 25 lines each → 200 lines, all
        // well-formed JSON with no torn bytes.
        let tmp = tempfile::tempdir().unwrap();
        let log_path = tmp.path().join("transfers.log");
        let n_threads = 8;
        let lines_per = 25;
        let mut handles = vec![];
        for tid in 0..n_threads {
            let p = log_path.clone();
            handles.push(std::thread::spawn(move || {
                for i in 0..lines_per {
                    let f = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&p)
                        .unwrap();
                    let ev = TransferEvent::now(
                        "push",
                        "02",
                        &format!("/app/workspace/inbox/t{}-{}", tid, i),
                        None,
                        i as u64,
                        true,
                        None,
                    );
                    let mut line = serde_json::to_string(&ev).unwrap();
                    line.push('\n');
                    write_with_flock(&f, line.as_bytes()).unwrap();
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }

        let contents = std::fs::read_to_string(&log_path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), (n_threads * lines_per) as usize);
        for line in &lines {
            // Each line must be parseable JSON — if interleaving
            // happened, parse fails.
            let parsed: serde_json::Value = serde_json::from_str(line)
                .unwrap_or_else(|e| panic!("malformed line {:?}: {}", line, e));
            assert_eq!(parsed["verb"], "push");
        }
    }
}

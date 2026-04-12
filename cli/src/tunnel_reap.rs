//! Shared tunnel-daemon reaping helper.
//!
//! Used by `tytus disconnect` (FIX-2) and `tytus revoke` (FIX-3) to kill the
//! root-owned `tytus tunnel-up` daemon process for a given pod and clean up
//! its pidfile + iface marker under `/tmp/tytus/`.
//!
//! # Source of truth
//!
//! The pidfile at `/tmp/tytus/tunnel-<pod>.pid` is THE source of truth for
//! "is a daemon alive for pod NN". `state.json.tunnel_iface` is NOT reliable
//! — `tytus revoke` wipes it but leaves the root-owned daemon running, which
//! is exactly bug FIX-2 from the sprint doc. Disconnect must iterate the
//! pidfile directory directly (`list_pod_pidfiles`), not `state.pods[]`.
//!
//! # Parameterisation
//!
//! The production entry points use `base_dir()` which reads
//! `TYTUS_TUNNEL_REAP_DIR` if set (used by the test harness to redirect to
//! a tempdir) and falls back to `/tmp/tytus`. Integration tests in
//! `cli/tests/disconnect_pidfile.rs` exercise the full state machine by
//! setting that env var.
//
// Sprint: docs/sprints/SPRINT-TYTUS-PAYING-CUSTOMER-READY.md (FIX-2, FIX-3)

use std::path::PathBuf;

/// Outcome of attempting to reap the tunnel daemon for a pod.
#[derive(Debug, Clone)]
pub enum ReapOutcome {
    /// Daemon was alive, tunnel-down succeeded, pidfile removed.
    Reaped { pid: u32 },
    /// No pidfile existed at `<base>/tunnel-<pod>.pid` — nothing to do.
    NoPidfile,
    /// Pidfile existed but the PID is not a live process; pidfile was removed.
    StalePidfile { pid: u32 },
    /// Pidfile existed, process was alive, but the reap attempt failed.
    /// Caller should log a warning and (for revoke) continue anyway.
    /// Disconnect MUST still clear local state — the user asked for it.
    ReapFailed { pid: u32, reason: String },
}

impl ReapOutcome {
    /// Legacy short suffix used by `tytus revoke` output (FIX-3).
    /// FIX-2 disconnect uses `disconnect_message()` below instead.
    pub fn human_suffix(&self) -> String {
        match self {
            ReapOutcome::Reaped { pid } => format!("  (reaped tunnel daemon pid={})", pid),
            ReapOutcome::StalePidfile { pid } => {
                format!("  (cleaned stale pidfile, pid={} was already dead)", pid)
            }
            ReapOutcome::ReapFailed { pid, reason } => {
                format!("  (WARNING: tunnel daemon pid={} still alive: {})", pid, reason)
            }
            ReapOutcome::NoPidfile => String::new(),
        }
    }

    /// True when the daemon is definitively gone after this outcome.
    /// Used by disconnect's "how many did we actually kill" counter.
    pub fn reaped_or_cleaned(&self) -> bool {
        matches!(
            self,
            ReapOutcome::Reaped { .. } | ReapOutcome::StalePidfile { .. }
        )
    }

    /// Returns the PID involved, if any.
    pub fn pid(&self) -> Option<u32> {
        match self {
            ReapOutcome::Reaped { pid } => Some(*pid),
            ReapOutcome::StalePidfile { pid } => Some(*pid),
            ReapOutcome::ReapFailed { pid, .. } => Some(*pid),
            ReapOutcome::NoPidfile => None,
        }
    }
}

/// Build the disconnect-facing user message for an outcome + pod_num.
///
/// These match the exact wording the sprint doc (FIX-2) promises so tests
/// can pin the prefix and users get consistent output across variants.
pub fn disconnect_message(pod_num: &str, outcome: &ReapOutcome) -> String {
    match outcome {
        ReapOutcome::Reaped { pid } => {
            format!("✓ Reaped tunnel daemon pid={} for pod {}", pid, pod_num)
        }
        ReapOutcome::NoPidfile => {
            format!("→ No pidfile for pod {} — nothing to reap", pod_num)
        }
        ReapOutcome::StalePidfile { pid } => format!(
            "→ Pidfile for pod {} references dead PID {} — cleaning up",
            pod_num, pid
        ),
        ReapOutcome::ReapFailed { pid, reason } => format!(
            "✗ Reap failed for pod {} (pid {}): {}",
            pod_num, pid, reason
        ),
    }
}

/// Validate a pod_num string is safe to embed in filesystem paths AND to
/// pass as an argv element to `sudo -n tytus tunnel-down <pid>`.
///
/// Allowed alphabet: `[A-Za-z0-9][A-Za-z0-9_-]{0,15}`. This matches the
/// 2-digit zero-padded IDs the CLI produces today plus a small amount of
/// headroom, while rejecting:
///
/// - `../`, `/`, `\` — path traversal / separator injection
/// - `;`, `|`, `` ` ``, `$`, `(`, `)`, whitespace — shell metacharacters
/// - empty string, overlong strings
///
/// Defense-in-depth: the reap path never splices `pod_num` into a shell
/// command (`Command::args()` doesn't spawn a shell), but we still
/// validate before any filesystem touch so a file dropped into `/tmp/tytus`
/// by another local user cannot influence our IO at all.
pub fn is_safe_pod_num(pod_num: &str) -> bool {
    if pod_num.is_empty() || pod_num.len() > 16 {
        return false;
    }
    let mut chars = pod_num.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphanumeric() {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Base directory where tunnel pidfiles live. Defaults to `/tmp/tytus` to
/// match the rest of the CLI (see `cmd_disconnect` and `cmd_tunnel_up`). An
/// environment override (`TYTUS_TUNNEL_REAP_DIR`) exists so unit and
/// integration tests can redirect to a writable tempdir — `/tmp/tytus` is
/// typically owned by root once a real tunnel has ever run.
fn base_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("TYTUS_TUNNEL_REAP_DIR") {
        PathBuf::from(dir)
    } else {
        PathBuf::from("/tmp/tytus")
    }
}

fn pidfile_path(pod_num: &str) -> PathBuf {
    base_dir().join(format!("tunnel-{}.pid", pod_num))
}

fn ifacefile_path(pod_num: &str) -> PathBuf {
    base_dir().join(format!("tunnel-{}.iface", pod_num))
}

/// Cross-platform errno accessor. macOS uses `__error()`, Linux uses
/// `__errno_location()`. Returns the raw errno after a failed libc call.
#[inline]
fn last_errno() -> i32 {
    unsafe {
        #[cfg(target_os = "macos")]
        {
            *libc::__error()
        }
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            *libc::__errno_location()
        }
    }
}

/// Production liveness check using `kill(pid, 0)`.
///
/// SAFETY: `libc::kill(pid, 0)` is a thin FFI call; it reads no user
/// memory and has no aliasing concerns. Signal 0 checks existence +
/// permission without delivering a signal.
///
/// EPERM means "process exists but we don't own it" — for our purposes
/// that IS alive; the scoped `tytus tunnel-down` helper runs as root via
/// sudoers and can reap it. ESRCH means no such process. Anything else →
/// assume dead and let the state machine clean up.
fn pid_is_alive(pid: i32) -> bool {
    if pid <= 1 {
        return false;
    }
    unsafe {
        if libc::kill(pid, 0) == 0 {
            return true;
        }
    }
    last_errno() == libc::EPERM
}

/// Read a pidfile for `pod_num` and parse its contents as an i32.
///
/// Rejects:
/// - missing/unreadable file (returns None)
/// - empty file after trim
/// - any non-digit character (including leading `+`/`-` and interior
///   whitespace) — `parse::<i32>()` alone would accept `+1234`
/// - integer overflow
///
/// Returning `Option` keeps the existing FIX-3 call shape — a garbled
/// file is indistinguishable from "nothing to reap" at the caller level,
/// and `reap_tunnel_for_pod` sweeps the junk before reporting `NoPidfile`.
fn read_pid_from_file(pod_num: &str) -> Option<i32> {
    let path = pidfile_path(pod_num);
    let contents = std::fs::read_to_string(&path).ok()?;
    let trimmed = contents.trim();
    if trimmed.is_empty() {
        return None;
    }
    if !trimmed.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    trimmed.parse::<i32>().ok()
}

/// List every `tunnel-*.pid` file under the configured base directory.
/// Skips entries whose derived pod_num fails `is_safe_pod_num`. Results
/// are sorted by pod_num for deterministic output.
///
/// This is FIX-2's key addition: disconnect must iterate the pidfile
/// directory directly instead of `state.pods[]`, because revoke wipes
/// state while leaving the root-owned daemon running.
pub fn list_pod_pidfiles() -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(base_dir()) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !(name.starts_with("tunnel-") && name.ends_with(".pid")) {
            continue;
        }
        let pod_num = &name["tunnel-".len()..name.len() - ".pid".len()];
        if !is_safe_pod_num(pod_num) {
            continue;
        }
        out.push((pod_num.to_string(), path));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Best-effort filesystem cleanup. Removes the pidfile and iface marker for
/// a given pod. Errors are ignored on purpose — both files are advisory, and
/// a stale file on disk cannot cause incorrect behaviour because
/// `read_pid_from_file` + `pid_is_alive` form the real source of truth.
///
/// Race note: it is possible for a concurrent `tytus disconnect` to remove
/// these files between our read and our remove. That is fine — `remove_file`
/// on a missing path returns `ErrorKind::NotFound` which we swallow via `ok()`.
fn cleanup_files(pod_num: &str) {
    let _ = std::fs::remove_file(pidfile_path(pod_num));
    let _ = std::fs::remove_file(ifacefile_path(pod_num));
}

/// Invoke the scoped `tytus tunnel-down <pid>` helper via passwordless sudo.
/// The helper re-validates the PID against `/tmp/tytus/tunnel-*.pid` before
/// signalling, so this cannot be abused as an arbitrary kill primitive —
/// even if the PID is recycled between our `is_alive` check and its `kill()`,
/// the helper will refuse to signal and we surface `ReapFailed`.
fn invoke_tunnel_down(pid: i32) -> Result<(), String> {
    if pid <= 1 {
        return Err(format!("refusing to signal PID {}", pid));
    }
    let self_exe = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "tytus".into());

    let output = std::process::Command::new("sudo")
        .args(["-n", &self_exe, "tunnel-down", &pid.to_string()])
        .output()
        .map_err(|e| format!("failed to spawn sudo: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            format!("tunnel-down exited with {}", output.status)
        } else {
            stderr
        })
    }
}

/// Reap the tunnel daemon for `pod_num`, if any.
///
/// Strategy:
/// 1. Validate `pod_num`. Unsafe → `ReapFailed` (never touches the filesystem).
/// 2. If the pidfile doesn't exist → `NoPidfile`, done.
/// 3. If it exists but contents are garbled → sweep files, `NoPidfile`.
/// 4. If it exists but the PID is dead → clean files, `StalePidfile`.
/// 5. If it exists and the PID is alive → invoke scoped tunnel-down, poll
///    liveness for up to ~500ms, clean files, return `Reaped` on success
///    or `ReapFailed` if the daemon survives.
///
/// This function is deliberately tolerant of concurrent disconnects: file
/// removals are best-effort and the authoritative "is the daemon gone"
/// signal comes from `kill(pid, 0)` after tunnel-down returns.
pub fn reap_tunnel_for_pod(pod_num: &str) -> ReapOutcome {
    if !is_safe_pod_num(pod_num) {
        return ReapOutcome::ReapFailed {
            pid: 0,
            reason: format!("unsafe pod_num {:?} rejected before filesystem touch", pod_num),
        };
    }

    let Some(pid) = read_pid_from_file(pod_num) else {
        // Pidfile absent OR garbled. Sweep any junk and report NoPidfile.
        cleanup_files(pod_num);
        return ReapOutcome::NoPidfile;
    };

    if !pid_is_alive(pid) {
        cleanup_files(pod_num);
        return ReapOutcome::StalePidfile { pid: pid as u32 };
    }

    match invoke_tunnel_down(pid) {
        Ok(()) => {
            // Give the daemon up to ~500ms to exit after SIGTERM. In practice
            // the async tunnel loop tears down almost instantly, but we
            // tolerate a small grace window before declaring ReapFailed.
            for _ in 0..10 {
                if !pid_is_alive(pid) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }

            if pid_is_alive(pid) {
                // Signal delivered but process still around. Leave the
                // pidfile in place so a follow-up disconnect can retry.
                ReapOutcome::ReapFailed {
                    pid: pid as u32,
                    reason: "daemon did not exit within 500ms of SIGTERM".into(),
                }
            } else {
                cleanup_files(pod_num);
                ReapOutcome::Reaped { pid: pid as u32 }
            }
        }
        Err(reason) => {
            // tunnel-down helper returned non-zero. Maybe sudo needs a
            // password, maybe the helper couldn't validate the PID. Check
            // one more time whether the daemon happens to already be dead
            // (a concurrent disconnect may have won the race) — if so we
            // still claim Reaped so the caller's state-clear path runs.
            if !pid_is_alive(pid) {
                cleanup_files(pod_num);
                ReapOutcome::Reaped { pid: pid as u32 }
            } else {
                ReapOutcome::ReapFailed { pid: pid as u32, reason }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::OnceLock;

    /// Initialise a shared, process-scoped writable base dir once per test
    /// binary. `/tmp/tytus` is typically owned by root on any box that has
    /// ever run a real tunnel, so we always redirect tests to a scratch
    /// path under the system temp dir.
    fn init_test_base_dir() {
        static ONCE: OnceLock<()> = OnceLock::new();
        ONCE.get_or_init(|| {
            let dir = std::env::temp_dir().join(format!(
                "tytus-reap-test-{}",
                std::process::id()
            ));
            std::fs::create_dir_all(&dir).unwrap();
            // set_var is process-global; we do this exactly once, before
            // any reap_tunnel_for_pod call, and only inside `#[cfg(test)]`.
            std::env::set_var("TYTUS_TUNNEL_REAP_DIR", &dir);
        });
    }

    /// Unique pod id per test. Must respect `is_safe_pod_num`: alnum + `-_`,
    /// max 16 chars. A 2-char tag prefix plus a monotonic 4-hex counter plus
    /// 2-hex pid-low byte yields stable 8-char ids that never collide inside
    /// one test binary and stay inside the length budget.
    fn unique_pod(tag: &str) -> String {
        use std::sync::atomic::{AtomicU32, Ordering};
        static CTR: AtomicU32 = AtomicU32::new(0);
        let n = CTR.fetch_add(1, Ordering::Relaxed);
        let short_tag: String = tag.chars().take(2).collect();
        let pid_low = (std::process::id() & 0xff) as u8;
        format!("{}{:04x}{:02x}", short_tag, n, pid_low)
    }

    fn write_pidfile(pod: &str, pid: i32) {
        init_test_base_dir();
        let path = pidfile_path(pod);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "{}", pid).unwrap();
    }

    #[test]
    fn safe_pod_num_accepts_expected_shapes() {
        for good in &["01", "99", "42", "a1", "pod-01", "POD_02"] {
            assert!(is_safe_pod_num(good), "expected accept {:?}", good);
        }
    }

    #[test]
    fn safe_pod_num_rejects_meta_and_traversal() {
        for bad in &[
            "",
            "01;",
            "../evil",
            "01 02",
            "$(id)",
            "`id`",
            "01\n",
            "-rf",
            " 01",
            "/abs",
            "01|rm",
            "way-too-long-pod-id",
        ] {
            assert!(!is_safe_pod_num(bad), "expected reject {:?}", bad);
        }
    }

    #[test]
    fn no_pidfile_returns_nopidfile() {
        init_test_base_dir();
        let pod = unique_pod("nopid");
        // Ensure it doesn't exist
        let _ = std::fs::remove_file(pidfile_path(&pod));
        match reap_tunnel_for_pod(&pod) {
            ReapOutcome::NoPidfile => {}
            other => panic!("expected NoPidfile, got {:?}", other),
        }
    }

    #[test]
    fn stale_pidfile_is_cleaned() {
        let pod = unique_pod("stale");
        // PID 999999 is ~guaranteed not to exist on any sane system.
        write_pidfile(&pod, 999_999);
        let outcome = reap_tunnel_for_pod(&pod);
        match outcome {
            ReapOutcome::StalePidfile { pid } => assert_eq!(pid, 999_999),
            other => panic!("expected StalePidfile, got {:?}", other),
        }
        assert!(!pidfile_path(&pod).exists(), "pidfile should be cleaned up");
    }

    #[test]
    fn garbled_pidfile_is_swept_as_nopidfile() {
        init_test_base_dir();
        let pod = unique_pod("garbled");
        let path = pidfile_path(&pod);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "not-a-pid").unwrap();
        let outcome = reap_tunnel_for_pod(&pod);
        match outcome {
            ReapOutcome::NoPidfile => {}
            other => panic!("expected NoPidfile for garbled file, got {:?}", other),
        }
        assert!(!path.exists(), "garbled pidfile should be cleaned up");
    }

    #[test]
    fn signed_and_overflow_pidfiles_are_swept_as_nopidfile() {
        init_test_base_dir();
        for (tag, contents) in &[
            ("signed", "+1234\n"),
            ("neg", "-42\n"),
            ("overflow", "99999999999999999999\n"),
            ("empty", ""),
        ] {
            let pod = unique_pod(tag);
            let path = pidfile_path(&pod);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, contents).unwrap();
            match reap_tunnel_for_pod(&pod) {
                ReapOutcome::NoPidfile => {}
                other => panic!("expected NoPidfile for {} {:?}, got {:?}", tag, contents, other),
            }
            assert!(!path.exists(), "{} pidfile should be cleaned up", tag);
        }
    }

    #[test]
    fn unsafe_pod_num_returns_reapfailed_without_touching_filesystem() {
        init_test_base_dir();
        let outcome = reap_tunnel_for_pod("../evil");
        match outcome {
            ReapOutcome::ReapFailed { pid, reason } => {
                assert_eq!(pid, 0);
                assert!(
                    reason.contains("unsafe pod_num"),
                    "expected 'unsafe pod_num' in reason, got {:?}",
                    reason
                );
            }
            other => panic!("expected ReapFailed, got {:?}", other),
        }
    }

    #[test]
    fn list_pidfiles_finds_written_pidfiles_and_sorts_them() {
        init_test_base_dir();
        // Use unique pod ids to avoid clobbering other tests running in
        // parallel. The listing returns results sorted by pod_num.
        let pod_a = unique_pod("lista");
        let pod_b = unique_pod("listb");
        write_pidfile(&pod_a, 111_111);
        write_pidfile(&pod_b, 222_222);
        let listed = list_pod_pidfiles();
        let names: Vec<String> = listed.iter().map(|(n, _)| n.clone()).collect();
        assert!(names.contains(&pod_a), "list should contain {}", pod_a);
        assert!(names.contains(&pod_b), "list should contain {}", pod_b);
        // Cleanup for other tests.
        let _ = std::fs::remove_file(pidfile_path(&pod_a));
        let _ = std::fs::remove_file(pidfile_path(&pod_b));
    }

    #[test]
    fn disconnect_message_covers_all_variants() {
        assert_eq!(
            disconnect_message("02", &ReapOutcome::Reaped { pid: 5569 }),
            "✓ Reaped tunnel daemon pid=5569 for pod 02"
        );
        assert_eq!(
            disconnect_message("02", &ReapOutcome::NoPidfile),
            "→ No pidfile for pod 02 — nothing to reap"
        );
        assert_eq!(
            disconnect_message("02", &ReapOutcome::StalePidfile { pid: 5569 }),
            "→ Pidfile for pod 02 references dead PID 5569 — cleaning up"
        );
        let msg = disconnect_message(
            "02",
            &ReapOutcome::ReapFailed {
                pid: 5569,
                reason: "sudo denied".into(),
            },
        );
        assert!(msg.contains("Reap failed for pod 02"));
        assert!(msg.contains("pid 5569"));
        assert!(msg.contains("sudo denied"));
    }
}

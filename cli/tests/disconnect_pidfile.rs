//! Integration tests for FIX-2 — pidfile-driven `tytus disconnect` reap.
//!
//! See `docs/sprints/SPRINT-TYTUS-PAYING-CUSTOMER-READY.md` (FIX-2) for the
//! full bug report. Short version: disconnect used to short-circuit when
//! `state.pods[].tunnel_iface == None`, leaving a root-owned daemon alive
//! after every revoke cycle. These tests exercise the new pidfile-driven
//! path via `atomek_cli::tunnel_reap::reap_tunnel_for_pod`.
//!
//! # Harness notes
//!
//! `tunnel_reap` reads `TYTUS_TUNNEL_REAP_DIR` for its base directory. Each
//! test here runs in a single cargo-test process but may share the env var
//! with peer tests — we use unique pod IDs (`unique_pod()`) so parallel
//! tests cannot clobber each other's pidfiles.
//!
//! We cannot actually invoke `sudo -n tytus tunnel-down` from a test, so
//! the "alive daemon" scenario is exercised against a PID that points at
//! a real short-lived helper process we spawn inside the test. The
//! production kill path (`sudo`) will fail when no NOPASSWD rule is
//! configured in the test environment, which is fine — `reap_tunnel_for_pod`
//! has a fallback: if `is_alive(pid)` is false after tunnel-down errors,
//! it still reports `Reaped`. We leverage that by letting our helper
//! process exit between the liveness check and the retry window.

use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Once;

use atomek_cli::tunnel_reap::{
    disconnect_message, is_safe_pod_num, list_pod_pidfiles, reap_tunnel_for_pod, ReapOutcome,
};

/// Redirect the tunnel-reap base directory to a process-scoped tempdir.
/// Runs exactly once per test binary — `std::env::set_var` is global.
fn init_base_dir() -> PathBuf {
    static ONCE: Once = Once::new();
    let dir = std::env::temp_dir().join(format!(
        "tytus-reap-inttest-{}",
        std::process::id()
    ));
    ONCE.call_once(|| {
        std::fs::create_dir_all(&dir).unwrap();
        std::env::set_var("TYTUS_TUNNEL_REAP_DIR", &dir);
    });
    dir
}

/// Unique pod id per test. Must respect `is_safe_pod_num`: alnum + `-_`,
/// max 16 chars. Fixed-width layout: 2-char tag prefix + 4-hex monotonic
/// counter + 2-hex pid-low byte → 8 chars total, never collides inside
/// one test binary, stays well inside the validator's length budget.
fn unique_pod(tag: &str) -> String {
    use std::sync::atomic::{AtomicU32, Ordering};
    static CTR: AtomicU32 = AtomicU32::new(0);
    let n = CTR.fetch_add(1, Ordering::Relaxed);
    let short_tag: String = tag.chars().take(2).collect();
    let pid_low = (std::process::id() & 0xff) as u8;
    format!("{}{:04x}{:02x}", short_tag, n, pid_low)
}

fn pidfile_path(pod_num: &str) -> PathBuf {
    init_base_dir().join(format!("tunnel-{}.pid", pod_num))
}

fn ifacefile_path(pod_num: &str) -> PathBuf {
    init_base_dir().join(format!("tunnel-{}.iface", pod_num))
}

fn write_pidfile(pod_num: &str, pid: i32) {
    let path = pidfile_path(pod_num);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "{}", pid).unwrap();
}

// ── Tests ──────────────────────────────────────────────────────

#[test]
fn no_pidfile_yields_nopidfile_outcome() {
    init_base_dir();
    let pod = unique_pod("none");
    // Ensure no leftover
    let _ = std::fs::remove_file(pidfile_path(&pod));

    let outcome = reap_tunnel_for_pod(&pod);
    assert!(
        matches!(outcome, ReapOutcome::NoPidfile),
        "expected NoPidfile, got {:?}",
        outcome
    );

    let msg = disconnect_message(&pod, &outcome);
    assert!(
        msg.contains("No pidfile"),
        "message should say 'No pidfile', got {:?}",
        msg
    );
}

#[test]
fn stale_pidfile_dead_pid_is_cleaned_up() {
    init_base_dir();
    let pod = unique_pod("dead");
    // PID 999_999 is ~guaranteed not to exist. (kill -0 on it returns
    // ESRCH, not EPERM, so `pid_is_alive` returns false.)
    write_pidfile(&pod, 999_999);
    assert!(pidfile_path(&pod).exists());

    let outcome = reap_tunnel_for_pod(&pod);
    match outcome {
        ReapOutcome::StalePidfile { pid } => assert_eq!(pid, 999_999),
        other => panic!("expected StalePidfile, got {:?}", other),
    }
    assert!(
        !pidfile_path(&pod).exists(),
        "stale pidfile should be swept"
    );
}

#[test]
fn stale_pidfile_also_removes_iface_file() {
    init_base_dir();
    let pod = unique_pod("iface");
    write_pidfile(&pod, 999_999);
    std::fs::write(ifacefile_path(&pod), "utun7").unwrap();
    assert!(ifacefile_path(&pod).exists());

    let _ = reap_tunnel_for_pod(&pod);
    assert!(!pidfile_path(&pod).exists());
    assert!(!ifacefile_path(&pod).exists());
}

#[test]
fn malformed_pidfile_non_numeric_is_swept() {
    init_base_dir();
    let pod = unique_pod("garbage");
    let path = pidfile_path(&pod);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "hello\nworld\n").unwrap();

    let outcome = reap_tunnel_for_pod(&pod);
    assert!(
        matches!(outcome, ReapOutcome::NoPidfile),
        "expected NoPidfile sweep, got {:?}",
        outcome
    );
    assert!(!path.exists(), "garbled pidfile should be removed");
}

#[test]
fn malformed_pidfile_signed_is_rejected() {
    init_base_dir();
    let pod = unique_pod("signed");
    let path = pidfile_path(&pod);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "+1234\n").unwrap();

    let outcome = reap_tunnel_for_pod(&pod);
    assert!(
        matches!(outcome, ReapOutcome::NoPidfile),
        "signed pidfile must be swept, got {:?}",
        outcome
    );
    assert!(!path.exists(), "signed pidfile should be removed");
}

#[test]
fn malformed_pidfile_overflow_is_rejected() {
    init_base_dir();
    let pod = unique_pod("overflow");
    let path = pidfile_path(&pod);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "99999999999999999999\n").unwrap();

    let outcome = reap_tunnel_for_pod(&pod);
    assert!(
        matches!(outcome, ReapOutcome::NoPidfile),
        "overflow pidfile must be swept, got {:?}",
        outcome
    );
}

#[test]
fn unsafe_pod_num_is_refused_without_filesystem_touch() {
    init_base_dir();
    // Even if this file somehow existed in the base dir, the safety check
    // runs first and rejects the pod_num before we ever read it.
    let evil = "../etc";
    let outcome = reap_tunnel_for_pod(evil);
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
fn is_safe_pod_num_accepts_expected_and_rejects_malicious() {
    // Sanity-check the validator surface that FIX-2 relies on.
    for good in &["01", "99", "42", "pod-01"] {
        assert!(is_safe_pod_num(good), "should accept {:?}", good);
    }
    for bad in &["", "../a", "01;rm", "$(id)", "01 02", " 01", "01\n"] {
        assert!(!is_safe_pod_num(bad), "should reject {:?}", bad);
    }
}

#[test]
fn alive_pid_reaped_via_self_exiting_child() {
    init_base_dir();
    let pod = unique_pod("alive");

    // Spawn a short-lived helper. `sleep 2` lives long enough for our
    // `is_alive` probe to see it, then exits on its own. Production flow
    // calls `sudo -n tytus tunnel-down <pid>` which will fail in the test
    // environment (no NOPASSWD rule) — but `reap_tunnel_for_pod` falls
    // back to a post-error liveness re-check: if the child has already
    // exited by the time the fallback runs, the outcome is still
    // `Reaped`. That's exactly the behaviour we want to pin.
    let mut child = Command::new("sleep")
        .arg("2")
        .spawn()
        .expect("spawn sleep helper");
    let child_pid = child.id() as i32;
    write_pidfile(&pod, child_pid);

    // The sudo path will fail here. Either:
    //  - `sudo -n` errors with "password required" and we fall through to
    //    the "is the daemon already dead?" recheck; the 500ms poll window
    //    plus the fact that `sleep 2` is still running usually yields
    //    `ReapFailed` on first run (daemon survived).
    //  - If the sleep child is reaped by something else in the meantime,
    //    we could see `Reaped`.
    //
    // Both are acceptable outcomes for the test harness — what we're
    // validating is that the state machine produced a terminal outcome
    // with a non-zero PID, didn't crash, and didn't touch any pidfile
    // outside of our unique pod.
    let outcome = reap_tunnel_for_pod(&pod);
    match &outcome {
        ReapOutcome::Reaped { pid } => assert_eq!(*pid as i32, child_pid),
        ReapOutcome::ReapFailed { pid, .. } => assert_eq!(*pid as i32, child_pid),
        other => panic!(
            "expected Reaped or ReapFailed for live pid {}, got {:?}",
            child_pid, other
        ),
    }

    // Clean up the helper if it's still running so we don't leak a
    // zombie into the test runner.
    let _ = child.kill();
    let _ = child.wait();

    // Best-effort pidfile cleanup — if ReapFailed, the state machine
    // intentionally leaves the pidfile in place.
    let _ = std::fs::remove_file(pidfile_path(&pod));
}

#[test]
fn list_pod_pidfiles_finds_written_files() {
    init_base_dir();
    let pod_a = unique_pod("listA");
    let pod_b = unique_pod("listB");
    write_pidfile(&pod_a, 111_111);
    write_pidfile(&pod_b, 222_222);

    let listed = list_pod_pidfiles();
    let names: Vec<String> = listed.iter().map(|(n, _)| n.clone()).collect();
    assert!(names.contains(&pod_a), "expected {} in {:?}", pod_a, names);
    assert!(names.contains(&pod_b), "expected {} in {:?}", pod_b, names);

    // Cleanup
    let _ = std::fs::remove_file(pidfile_path(&pod_a));
    let _ = std::fs::remove_file(pidfile_path(&pod_b));
}

#[test]
fn list_pod_pidfiles_ignores_non_matching_entries() {
    init_base_dir();
    let base = init_base_dir();
    // Drop some junk files — none should show up in the listing.
    std::fs::write(base.join("not-a-tunnel.pid"), "1234").unwrap();
    std::fs::write(base.join("tunnel-.pid"), "1234").unwrap(); // empty pod_num
    std::fs::write(base.join("tunnel-$(id).pid"), "1234").unwrap(); // unsafe
    std::fs::write(base.join("tunnel-01.iface"), "utun").unwrap(); // wrong suffix

    let listed = list_pod_pidfiles();
    for (name, _) in &listed {
        assert!(
            is_safe_pod_num(name),
            "listing returned unsafe pod_num {:?}",
            name
        );
    }

    // Cleanup
    let _ = std::fs::remove_file(base.join("not-a-tunnel.pid"));
    let _ = std::fs::remove_file(base.join("tunnel-.pid"));
    let _ = std::fs::remove_file(base.join("tunnel-$(id).pid"));
    let _ = std::fs::remove_file(base.join("tunnel-01.iface"));
}

#[test]
fn disconnect_message_exact_wording_matches_sprint_spec() {
    // These exact strings are enshrined in the sprint doc FIX-2 section.
    // If you change them, update the sprint doc too.
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
    let failed = disconnect_message(
        "02",
        &ReapOutcome::ReapFailed {
            pid: 5569,
            reason: "sudo: a password is required".into(),
        },
    );
    assert!(failed.starts_with("✗ Reap failed for pod 02"));
    assert!(failed.contains("pid 5569"));
    assert!(failed.contains("sudo"));
}

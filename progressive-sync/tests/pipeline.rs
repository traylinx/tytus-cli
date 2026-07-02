//! Apply-pipeline integration matrix (TEST-PLAN consumer state + conflict +
//! failure injection) against an in-memory S3 that records every call — so
//! the tests also prove prefix scoping and StartAfter cursor listing (no
//! full-tree hot path).

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};
use tytus_progressive_sync::apply::{file_sha256, BindingConsumer};
use tytus_progressive_sync::s3::{RemoteObject, S3Error, S3Ops};
use tytus_progressive_sync::state::{BindingIdentity, DeadLetterClass, JournalRecord, StateStore};

const BINDING: &str = "sf_testbinding0001";
const BUCKET: &str = "tytus-progressive-canary";
const ROUTE: &str = "r0consumer";
const ULID: &str = "01J2Z7P2V8QJ0S7KX6YQ8M9H3A";

#[derive(Default)]
struct MockS3 {
    objects: RefCell<BTreeMap<String, Vec<u8>>>,
    calls: RefCell<Vec<String>>,
    fail_downloads: RefCell<u32>,
}

impl MockS3 {
    fn put(&self, key: &str, bytes: &[u8]) {
        self.objects.borrow_mut().insert(key.to_string(), bytes.to_vec());
    }
    fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}

impl S3Ops for MockS3 {
    fn list_after(&self, prefix: &str, start_after: Option<&str>, max: usize)
        -> Result<Vec<RemoteObject>, S3Error>
    {
        self.calls.borrow_mut().push(format!("list {prefix} after={}", start_after.unwrap_or("-")));
        Ok(self
            .objects
            .borrow()
            .iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .filter(|(k, _)| start_after.map_or(true, |s| k.as_str() > s))
            .take(max)
            .map(|(k, v)| RemoteObject { key: k.clone(), size: v.len() as u64 })
            .collect())
    }

    fn cat_small(&self, key: &str) -> Result<Vec<u8>, S3Error> {
        self.calls.borrow_mut().push(format!("cat {key}"));
        self.objects.borrow().get(key).cloned().ok_or_else(|| S3Error::NotFound(key.into()))
    }

    fn download(&self, key: &str, target: &Path) -> Result<(), S3Error> {
        self.calls.borrow_mut().push(format!("get {key}"));
        let mut failures = self.fail_downloads.borrow_mut();
        if *failures > 0 {
            *failures -= 1;
            return Err(S3Error::Transient("injected".into()));
        }
        match self.objects.borrow().get(key) {
            Some(bytes) => {
                fs::write(target, bytes).map_err(|e| S3Error::Transient(e.to_string()))?;
                Ok(())
            }
            None => Err(S3Error::NotFound(key.into())),
        }
    }
}

fn sha_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Stage a producer-shaped event + blob into the mock bucket.
fn stage_event(s3: &MockS3, sequence: u64, key: &str, body: &[u8]) -> String {
    stage_event_ext(s3, sequence, key, body, "put", true)
}

fn stage_event_ext(s3: &MockS3, sequence: u64, key: &str, body: &[u8], op: &str, with_blob: bool) -> String {
    let hex_digest = sha_hex(body);
    let event_id = format!("{sequence:020}-{ULID}");
    let payload = serde_json::json!({
        "schema_version": "sync-event-v1",
        "binding_id": BINDING,
        "bucket": BUCKET,
        "route_id": ROUTE,
        "sequence": sequence,
        "event_id": event_id,
        "op": op,
        "key": key,
        "content_ref": format!("_tytus-sync/blobs/sha256/{hex_digest}"),
        "object": {"size": body.len(), "sha256": format!("sha256:{hex_digest}")},
        "created_at": "2026-07-02T12:00:00Z",
        "producer": {"agent_label": "MockPod", "implementation": "test"},
    });
    if with_blob {
        s3.put(&format!("_tytus-sync/blobs/sha256/{hex_digest}"), body);
    }
    s3.put(
        &format!("_tytus-sync/events/{BINDING}/{ROUTE}/{event_id}.json"),
        serde_json::to_string(&payload).unwrap().as_bytes(),
    );
    event_id
}

struct Harness {
    _tmp: tempfile::TempDir,
    identity: BindingIdentity,
    store: StateStore,
    local_root: std::path::PathBuf,
}

fn harness() -> Harness {
    let tmp = tempfile::tempdir().unwrap();
    let local_root = tmp.path().join("folder");
    let state_root = tmp.path().join("state");
    fs::create_dir_all(&local_root).unwrap();
    let identity = BindingIdentity {
        binding_id: BINDING.into(),
        bucket: BUCKET.into(),
        local_path: local_root.to_string_lossy().into_owned(),
        alias: None,
    };
    let store = StateStore::open(&state_root, &identity).unwrap();
    Harness { _tmp: tmp, identity, store, local_root }
}

fn consumer<'a>(h: &'a Harness, s3: &'a MockS3) -> (BindingConsumer<'a, MockS3>, Vec<JournalRecord>) {
    let (state, pending) = h.store.load(&h.identity, "test-consumer").unwrap();
    let state = Box::leak(Box::new(state));
    (BindingConsumer::new(s3, &h.store, state, &h.identity, vec![ROUTE.into()]), pending)
}

#[test]
fn applies_events_in_order_and_advances_cursor() {
    let h = harness();
    let s3 = MockS3::default();
    stage_event(&s3, 1, "notes/a.md", b"first");
    stage_event(&s3, 2, "notes/b.md", b"second");
    let (mut c, _) = consumer(&h, &s3);
    let outcome = c.poll_once();
    assert_eq!(outcome.applied, 2, "{:?}", outcome.transient_errors);
    assert_eq!(fs::read(h.local_root.join("notes/a.md")).unwrap(), b"first");
    assert_eq!(fs::read(h.local_root.join("notes/b.md")).unwrap(), b"second");
    assert_eq!(c.state.route(ROUTE).last_applied_sequence, 2);
    assert_eq!(c.state.ledger.len(), 2);
}

#[test]
fn duplicate_listing_is_suppressed_and_start_after_used() {
    let h = harness();
    let s3 = MockS3::default();
    stage_event(&s3, 1, "a.md", b"one");
    let (mut c, _) = consumer(&h, &s3);
    assert_eq!(c.poll_once().applied, 1);
    assert_eq!(c.poll_once().applied, 0);
    let calls = s3.calls();
    let second_list = calls.iter().filter(|c| c.starts_with("list")).nth(1).unwrap();
    // steady-state poll lists AFTER the cursor sequence — the G0 decision
    assert!(second_list.contains(&format!("after=_tytus-sync/events/{BINDING}/{ROUTE}/{:020}~", 1)),
            "{second_list}");
    // and every call in the whole run was prefix-scoped to the route
    assert!(calls.iter().filter(|c| c.starts_with("list")).all(|c| c.contains(&format!("events/{BINDING}/{ROUTE}/"))));
}

#[test]
fn sequence_gap_holds_cursor_and_flags_repair_after_grace() {
    let h = harness();
    let s3 = MockS3::default();
    stage_event(&s3, 1, "a.md", b"one");
    stage_event(&s3, 3, "c.md", b"three"); // hole at 2
    let (mut c, _) = consumer(&h, &s3);
    let first = c.poll_once();
    assert_eq!(first.applied, 1);
    assert!(first.repair_required.is_empty(), "grace window first");
    c.poll_once();
    let third = c.poll_once();
    assert_eq!(third.repair_required.get(ROUTE).map(String::as_str), Some("sequence_gap"));
    assert_eq!(c.state.route(ROUTE).last_applied_sequence, 1, "never applied out of order");
    assert!(!h.local_root.join("c.md").exists());
}

#[test]
fn retention_gap_when_min_visible_above_cursor_plus_one() {
    let h = harness();
    let s3 = MockS3::default();
    stage_event(&s3, 5, "late.md", b"late"); // janitor pruned 1..4, fresh consumer
    let (mut c, _) = consumer(&h, &s3);
    for _ in 0..3 {
        c.poll_once();
    }
    assert_eq!(
        c.state.route(ROUTE).repair_required.as_deref(),
        Some("retention_gap")
    );
}

#[test]
fn empty_prefix_with_cursor_is_not_a_gap() {
    let h = harness();
    let s3 = MockS3::default();
    stage_event(&s3, 1, "a.md", b"one");
    let (mut c, _) = consumer(&h, &s3);
    c.poll_once();
    s3.objects.borrow_mut().clear(); // prefix vanishes entirely
    for _ in 0..4 {
        let outcome = c.poll_once();
        assert!(outcome.repair_required.is_empty());
    }
}

#[test]
fn hash_mismatch_dead_letters_after_budget_and_route_continues() {
    let h = harness();
    let s3 = MockS3::default();
    let event_id = stage_event(&s3, 1, "bad.md", b"expected");
    // corrupt the blob AFTER staging so sha in event != blob bytes
    let blob_key = format!("_tytus-sync/blobs/sha256/{}", sha_hex(b"expected"));
    s3.put(&blob_key, b"corrupted");
    stage_event(&s3, 2, "good.md", b"fine");
    let (mut c, _) = consumer(&h, &s3);
    let mut dead = 0;
    for _ in 0..6 {
        dead += c.poll_once().dead_letters;
    }
    assert_eq!(dead, 1);
    assert!(h.store.dir.join("dead-letter").join(format!("{event_id}.json")).exists());
    // poison rule: sequence 2 applied after the dead-letter advance
    assert!(h.local_root.join("good.md").exists());
    assert_eq!(c.state.route(ROUTE).last_applied_sequence, 2);
    assert_eq!(c.state.route(ROUTE).dead_letter_count, 1);
    assert!(!h.local_root.join("bad.md").exists(), "mismatched bytes never land");
}

#[test]
fn missing_blob_dead_letters_as_missing_content_ref() {
    let h = harness();
    let s3 = MockS3::default();
    stage_event_ext(&s3, 1, "gone.md", b"body", "put", false); // no blob
    let (mut c, _) = consumer(&h, &s3);
    for _ in 0..6 {
        c.poll_once();
    }
    let journal = fs::read_to_string(h.store.journal_path()).unwrap();
    assert!(journal.contains("missing_content_ref"), "{journal}");
}

#[test]
fn poison_filename_quarantines_without_blocking() {
    let h = harness();
    let s3 = MockS3::default();
    stage_event(&s3, 1, "reports/AUX.txt", b"reserved-name");
    stage_event(&s3, 2, "ok.md", b"fine");
    let (mut c, _) = consumer(&h, &s3);
    let outcome = c.poll_once();
    assert_eq!(outcome.dead_letters, 1);
    assert_eq!(outcome.applied, 1);
    assert!(h.local_root.join("ok.md").exists());
    assert!(!h.local_root.join("reports").exists());
    let journal = fs::read_to_string(h.store.journal_path()).unwrap();
    assert!(journal.contains("path_invalid"));
}

#[test]
fn reserved_namespace_key_never_applies() {
    let h = harness();
    let s3 = MockS3::default();
    stage_event(&s3, 1, "_tytus-sync/evil.json", b"nope");
    let (mut c, _) = consumer(&h, &s3);
    let outcome = c.poll_once();
    assert_eq!(outcome.dead_letters, 1);
    assert!(!h.local_root.join("_tytus-sync").exists());
}

#[test]
fn delete_event_counted_ignored_cursor_advances() {
    let h = harness();
    let s3 = MockS3::default();
    stage_event_ext(&s3, 1, "gone.md", b"x", "delete", true);
    stage_event(&s3, 2, "kept.md", b"kept");
    let (mut c, _) = consumer(&h, &s3);
    let outcome = c.poll_once();
    assert_eq!(outcome.delete_ignored, 1);
    assert_eq!(outcome.applied, 1);
    assert!(!h.local_root.join("gone.md").exists(), "delete never removes anything in v1");
    assert!(h.local_root.join("kept.md").exists());
    assert_eq!(c.state.delete_events_ignored, 1);
}

#[test]
fn conflict_keep_both_preserves_local_bytes() {
    let h = harness();
    let s3 = MockS3::default();
    // local file exists with content the ledger never applied -> untracked
    fs::write(h.local_root.join("doc.md"), b"my local work").unwrap();
    stage_event(&s3, 1, "doc.md", b"pod version");
    let (mut c, _) = consumer(&h, &s3);
    let outcome = c.poll_once();
    assert_eq!(outcome.conflicts, 1);
    assert_eq!(fs::read(h.local_root.join("doc.md")).unwrap(), b"my local work");
    let conflict_file = fs::read_dir(&h.local_root)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .find(|n| n.contains(".conflict-"))
        .expect("incoming written to conflict name");
    assert!(conflict_file.starts_with("doc.conflict-") && conflict_file.ends_with(".md"));
    assert_eq!(fs::read(h.local_root.join(&conflict_file)).unwrap(), b"pod version");
    let records: Vec<_> = fs::read_dir(h.store.dir.join("conflicts")).unwrap().flatten().collect();
    assert_eq!(records.len(), 1);
    assert_eq!(c.state.route(ROUTE).last_applied_sequence, 1, "conflict advances cursor");
}

#[test]
fn replace_allowed_when_hash_matches_ledger_and_displaced_bytes_quarantined() {
    let h = harness();
    let s3 = MockS3::default();
    stage_event(&s3, 1, "doc.md", b"v1");
    let (mut c, _) = consumer(&h, &s3);
    assert_eq!(c.poll_once().applied, 1);
    stage_event(&s3, 2, "doc.md", b"v2");
    let outcome = c.poll_once();
    assert_eq!(outcome.applied, 1, "{:?}", outcome.transient_errors);
    assert_eq!(fs::read(h.local_root.join("doc.md")).unwrap(), b"v2");
    // displaced v1 bytes recoverable from quarantine
    let mut found = false;
    for entry in walk(&h.store.dir.join("quarantine")) {
        if fs::read(&entry).map(|b| b == b"v1").unwrap_or(false) {
            found = true;
        }
    }
    assert!(found, "displaced bytes quarantined before replace");
}

#[test]
fn locally_modified_since_apply_conflicts_instead_of_overwrite() {
    let h = harness();
    let s3 = MockS3::default();
    stage_event(&s3, 1, "doc.md", b"v1");
    let (mut c, _) = consumer(&h, &s3);
    c.poll_once();
    fs::write(h.local_root.join("doc.md"), b"local edit after apply").unwrap();
    stage_event(&s3, 2, "doc.md", b"v2");
    let outcome = c.poll_once();
    assert_eq!(outcome.conflicts, 1);
    assert_eq!(outcome.applied, 0);
    assert_eq!(fs::read(h.local_root.join("doc.md")).unwrap(), b"local edit after apply");
}

#[test]
fn case_collision_conflicts_never_overwrites_alias() {
    let h = harness();
    let s3 = MockS3::default();
    fs::write(h.local_root.join("Readme.md"), b"cased local").unwrap();
    stage_event(&s3, 1, "readme.md", b"incoming lower");
    let (mut c, _) = consumer(&h, &s3);
    let outcome = c.poll_once();
    assert_eq!(outcome.conflicts, 1);
    assert_eq!(fs::read(h.local_root.join("Readme.md")).unwrap(), b"cased local");
}

#[test]
fn transient_download_failure_retries_within_budget_then_succeeds() {
    let h = harness();
    let s3 = MockS3::default();
    stage_event(&s3, 1, "a.md", b"body");
    *s3.fail_downloads.borrow_mut() = 2;
    let (mut c, _) = consumer(&h, &s3);
    assert_eq!(c.poll_once().applied, 0);
    assert_eq!(c.poll_once().applied, 0);
    assert_eq!(c.poll_once().applied, 1, "third attempt succeeds, no dead-letter");
    assert_eq!(c.state.route(ROUTE).dead_letter_count, 0);
}

#[test]
fn restart_replays_journal_and_survives_snapshot_compaction() {
    let h = harness();
    let s3 = MockS3::default();
    stage_event(&s3, 1, "a.md", b"one");
    stage_event(&s3, 2, "b.md", b"two");
    {
        let (mut c, _) = consumer(&h, &s3);
        assert_eq!(c.poll_once().applied, 2);
        h.store.compact(c.state).unwrap(); // snapshot + truncate journal
    }
    stage_event(&s3, 3, "c.md", b"three");
    {
        let (mut c, pending) = consumer(&h, &s3);
        c.recover(pending);
        assert_eq!(c.state.route(ROUTE).last_applied_sequence, 2, "cursor from snapshot");
        assert_eq!(c.state.ledger.len(), 2, "ledger from snapshot");
        assert_eq!(c.poll_once().applied, 1);
        assert_eq!(c.state.route(ROUTE).last_applied_sequence, 3);
    }
}

#[test]
fn crash_between_rename_and_applied_completes_idempotently() {
    let h = harness();
    let s3 = MockS3::default();
    let body = b"crashed bytes";
    let hex_digest = sha_hex(body);
    // simulate the crash window: final file present, pending_apply journaled,
    // no applied record
    fs::write(h.local_root.join("crash.md"), body).unwrap();
    h.store.append(&JournalRecord::PendingApply {
        route_id: ROUTE.into(),
        sequence: 1,
        event_id: format!("{:020}-{ULID}", 1),
        key: "crash.md".into(),
        sha256: format!("sha256:{hex_digest}"),
        tmp_path: h.local_root.join(".tytus-sync-tmp-x").to_string_lossy().into_owned(),
        at: "2026-07-02T12:00:00Z".into(),
    }).unwrap();
    let (mut c, pending) = consumer(&h, &s3);
    assert_eq!(pending.len(), 1);
    c.recover(pending);
    assert_eq!(c.state.route(ROUTE).last_applied_sequence, 1, "completed idempotently");
    assert_eq!(
        c.state.ledger.get("crash.md").map(|l| l.sha256.clone()),
        Some(format!("sha256:{hex_digest}"))
    );
}

#[test]
fn skip_sequence_record_clears_gap_and_cursor_moves() {
    let h = harness();
    let s3 = MockS3::default();
    stage_event(&s3, 1, "a.md", b"one");
    stage_event(&s3, 3, "c.md", b"three");
    let (mut c, _) = consumer(&h, &s3);
    for _ in 0..3 {
        c.poll_once();
    }
    assert_eq!(c.state.route(ROUTE).repair_required.as_deref(), Some("sequence_gap"));
    // reconcile adjudicates the hole with a durable, evidenced skip
    h.store.append(&JournalRecord::SkipSequence {
        route_id: ROUTE.into(),
        sequence: 2,
        reason: "producer_tombstone".into(),
        evidence: "reconcile-report-test".into(),
        at: "2026-07-02T12:00:00Z".into(),
    }).unwrap();
    drop(c);
    let (mut c, pending) = consumer(&h, &s3);
    c.recover(pending);
    assert_eq!(c.state.route(ROUTE).last_applied_sequence, 2);
    assert!(c.state.route(ROUTE).repair_required.is_none(), "skip clears repair");
    assert_eq!(c.poll_once().applied, 1, "sequence 3 now applies");
}

#[test]
fn cursor_never_beyond_last_durable_record() {
    // fold() only moves cursors on Applied/DeadLetter/Conflict/SkipSequence —
    // prove a pending_apply alone moves nothing.
    let h = harness();
    let s3 = MockS3::default();
    h.store.append(&JournalRecord::PendingApply {
        route_id: ROUTE.into(),
        sequence: 9,
        event_id: format!("{:020}-{ULID}", 9),
        key: "x.md".into(),
        sha256: "sha256:".to_string() + &"0".repeat(64),
        tmp_path: "/nonexistent".into(),
        at: "2026-07-02T12:00:00Z".into(),
    }).unwrap();
    let (c, pending) = consumer(&h, &s3);
    assert_eq!(pending.len(), 1);
    assert_eq!(c.state.routes.get(ROUTE).map(|r| r.last_applied_sequence).unwrap_or(0), 0);
}

fn walk(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(walk(&path));
            } else {
                out.push(path);
            }
        }
    }
    out
}

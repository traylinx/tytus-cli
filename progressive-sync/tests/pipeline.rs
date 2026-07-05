//! Apply-pipeline integration matrix (TEST-PLAN consumer state + conflict +
//! failure injection) against an in-memory S3 that records every call — so
//! the tests also prove prefix scoping and StartAfter cursor listing (no
//! full-tree hot path).

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};
use tytus_progressive_sync::apply::BindingConsumer;
use tytus_progressive_sync::s3::{RemoteObject, S3Error, S3Ops};
use tytus_progressive_sync::state::{utc_now, BindingIdentity, JournalRecord, StateStore};

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
fn retention_gap_flags_immediately_for_fresh_consumer() {
    let h = harness();
    let s3 = MockS3::default();
    stage_event(&s3, 5, "late.md", b"late"); // janitor pruned 1..4, fresh consumer
    let (mut c, _) = consumer(&h, &s3);
    // Contract: min visible > cursor+1 with cursor==0 is retention pruning by
    // definition (the janitor never deletes the high-water event) — no grace.
    let first = c.poll_once();
    assert_eq!(first.repair_required.get(ROUTE).map(String::as_str), Some("retention_gap"));
    assert!(!h.local_root.join("late.md").exists());
}

#[test]
fn snapshot_never_carries_repair_state() {
    let h = harness();
    let s3 = MockS3::default();
    stage_event(&s3, 5, "late.md", b"late");
    let (mut c, _) = consumer(&h, &s3);
    c.poll_once();
    assert!(c.state.route(ROUTE).repair_required.is_some());
    h.store.compact(c.state).unwrap();
    let snapshot = fs::read_to_string(h.store.snapshot_path()).unwrap();
    assert!(!snapshot.contains("repair_required"), "derived state must not be durable: {snapshot}");
    drop(c);
    // after restart the condition re-derives from the very next poll
    let (mut c, pending) = consumer(&h, &s3);
    c.recover(pending);
    assert!(c.state.route(ROUTE).repair_required.is_none(), "not resurrected from snapshot");
    let outcome = c.poll_once();
    assert_eq!(outcome.repair_required.get(ROUTE).map(String::as_str), Some("retention_gap"));
}

#[test]
fn retry_budget_survives_restart() {
    let h = harness();
    let s3 = MockS3::default();
    stage_event_ext(&s3, 1, "gone.md", b"body", "put", false); // blob permanently missing
    // two attempts in one process...
    {
        let (mut c, _) = consumer(&h, &s3);
        c.poll_once();
        c.poll_once();
    }
    // ...then a "restart": budget continues counting, not reset
    let (mut c, pending) = consumer(&h, &s3);
    c.recover(pending);
    let mut dead = 0;
    for _ in 0..3 {
        dead += c.poll_once().dead_letters;
    }
    assert_eq!(dead, 1, "5 attempts total ACROSS restarts dead-letters the poison event");
}

#[test]
fn repair_record_clears_repair_state() {
    let h = harness();
    let s3 = MockS3::default();
    stage_event(&s3, 1, "a.md", b"one");
    stage_event(&s3, 3, "c.md", b"three");
    let (mut c, _) = consumer(&h, &s3);
    for _ in 0..3 {
        c.poll_once();
    }
    assert_eq!(c.state.route(ROUTE).repair_required.as_deref(), Some("sequence_gap"));
    h.store.append(&JournalRecord::Repair {
        source: "reconcile".into(),
        corrections: serde_json::json!([{"route_id": ROUTE, "set_cursor_sequence": 2}]),
        at: "2026-07-02T12:00:00Z".into(),
    }).unwrap();
    drop(c);
    let (mut c, pending) = consumer(&h, &s3);
    c.recover(pending);
    assert!(c.state.route(ROUTE).repair_required.is_none());
    assert_eq!(c.state.route(ROUTE).last_applied_sequence, 2);
    assert_eq!(c.poll_once().applied, 1, "sequence 3 applies after the repair");
}

#[test]
fn payload_event_id_must_match_filename() {
    let h = harness();
    let s3 = MockS3::default();
    // stage a valid event, then overwrite its payload with a DIFFERENT ULID
    let event_id = stage_event(&s3, 1, "a.md", b"one");
    let object_key = format!("_tytus-sync/events/{BINDING}/{ROUTE}/{event_id}.json");
    let bytes = s3.objects.borrow().get(&object_key).cloned().unwrap();
    let mut payload: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    payload["event_id"] = format!("{:020}-{}", 1, "00000000000000000000000000").into();
    s3.put(&object_key, serde_json::to_string(&payload).unwrap().as_bytes());
    let (mut c, _) = consumer(&h, &s3);
    let outcome = c.poll_once();
    assert_eq!(outcome.dead_letters, 1);
    assert!(!h.local_root.join("a.md").exists());
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

#[test]
fn gap_observation_raises_observed_high_water() {
    // The first visible sequence beyond a gap must raise
    // observed_high_water_sequence even though nothing applies — the
    // auto-reconcile Repair correction (tray, PR#30) skips the cursor TO
    // this value once a complete reconcile proves the files present.
    let h = harness();
    let s3 = MockS3::default();
    stage_event(&s3, 5, "notes/late.md", b"beyond gap"); // cursor 0, min visible 5
    let (mut c, _) = consumer(&h, &s3);
    let outcome = c.poll_once();
    assert_eq!(outcome.applied, 0);
    assert_eq!(
        outcome.repair_required.get(ROUTE).map(String::as_str),
        Some("retention_gap")
    );
    assert_eq!(
        c.state.route(ROUTE).observed_high_water_sequence,
        5,
        "gap observation must raise the high water"
    );
}

/// Stage an event under an arbitrary remote binding id (the Provider grant
/// id producers actually use, which differs from the local sidecar id).
fn stage_event_remote(s3: &MockS3, remote_binding: &str, sequence: u64, key: &str, body: &[u8]) -> String {
    let hex_digest = sha_hex(body);
    let event_id = format!("{sequence:020}-{ULID}");
    let payload = serde_json::json!({
        "schema_version": "sync-event-v1",
        "binding_id": remote_binding,
        "bucket": BUCKET,
        "route_id": ROUTE,
        "sequence": sequence,
        "event_id": event_id,
        "op": "put",
        "key": key,
        "content_ref": format!("_tytus-sync/blobs/sha256/{hex_digest}"),
        "object": {"size": body.len(), "sha256": format!("sha256:{hex_digest}")},
        "created_at": "2026-07-02T12:00:00Z",
        "producer": {"agent_label": "MockPod", "implementation": "test"},
    });
    s3.put(&format!("_tytus-sync/blobs/sha256/{hex_digest}"), body);
    s3.put(
        &format!("_tytus-sync/events/{remote_binding}/{ROUTE}/{event_id}.json"),
        serde_json::to_string(&payload).unwrap().as_bytes(),
    );
    event_id
}

#[test]
fn remote_binding_id_overrides_local_namespace() {
    // Producers emit under grant.folder_id (Provider registry) — NOT the
    // local sidecar folder_id. With remote_binding_id set, the consumer must
    // poll the remote namespace and accept payloads carrying the remote id,
    // while all local state stays keyed by the local identity.
    let h = harness();
    let s3 = MockS3::default();
    stage_event_remote(&s3, "sf_provider_registry", 1, "docs/from-pod.md", b"remote namespace");
    // Default (remote == local): the remote namespace is invisible.
    let (mut c, _) = consumer(&h, &s3);
    assert_eq!(c.poll_once().applied, 0, "local-id poll must not see the remote namespace");
    drop(c);
    // With the discovered remote id: the event applies end-to-end.
    let (mut c, _) = consumer(&h, &s3);
    c.remote_binding_id = "sf_provider_registry".into();
    let outcome = c.poll_once();
    assert_eq!(outcome.applied, 1, "{:?}", outcome.transient_errors);
    assert_eq!(outcome.dead_letters, 0);
    assert_eq!(fs::read(h.local_root.join("docs/from-pod.md")).unwrap(), b"remote namespace");
    assert_eq!(c.state.route(ROUTE).last_applied_sequence, 1);
}

#[test]
fn remote_namespace_payload_with_foreign_binding_id_dead_letters() {
    // Trust boundary under a remote namespace: the payload's binding_id must
    // match the namespace it was listed under, not the local id.
    let h = harness();
    let s3 = MockS3::default();
    // payload claims BINDING (the local id) but sits under sf_provider_registry
    let body: &[u8] = b"spoof";
    let hex_digest = sha_hex(body);
    let event_id = format!("{:020}-{ULID}", 1);
    let payload = serde_json::json!({
        "schema_version": "sync-event-v1",
        "binding_id": BINDING,
        "bucket": BUCKET,
        "route_id": ROUTE,
        "sequence": 1,
        "event_id": event_id,
        "op": "put",
        "key": "docs/spoof.md",
        "content_ref": format!("_tytus-sync/blobs/sha256/{hex_digest}"),
        "object": {"size": body.len(), "sha256": format!("sha256:{hex_digest}")},
        "created_at": "2026-07-02T12:00:00Z",
        "producer": {"agent_label": "MockPod", "implementation": "test"},
    });
    s3.put(&format!("_tytus-sync/blobs/sha256/{hex_digest}"), body);
    s3.put(
        &format!("_tytus-sync/events/sf_provider_registry/{ROUTE}/{event_id}.json"),
        serde_json::to_string(&payload).unwrap().as_bytes(),
    );
    let (mut c, _) = consumer(&h, &s3);
    c.remote_binding_id = "sf_provider_registry".into();
    let outcome = c.poll_once();
    assert_eq!(outcome.applied, 0);
    assert_eq!(outcome.dead_letters, 1, "payload/prefix identity mismatch must dead-letter");
    assert!(!h.local_root.join("docs/spoof.md").exists());
}

#[test]
fn sequence_gap_trips_across_consumer_rebuilds() {
    // THE production lifecycle (tray poll_binding): a FRESH StateStore +
    // BindingConsumer per poll. Gap grace must accumulate in the durable
    // side file (gap_polls.json) — an in-memory counter resets on every
    // rebuild and the sequence_gap halt would never fire (sprint phase-3
    // exit condition).
    let h = harness();
    let s3 = MockS3::default();
    stage_event(&s3, 1, "a.md", b"one");
    stage_event(&s3, 3, "c.md", b"three"); // hole at 2
    for poll in 0..3 {
        let (mut c, pending) = consumer(&h, &s3);
        c.recover(pending);
        let outcome = c.poll_once();
        if poll < 2 {
            assert!(outcome.repair_required.is_empty(), "grace window poll {poll}");
        } else {
            assert_eq!(
                outcome.repair_required.get(ROUTE).map(String::as_str),
                Some("sequence_gap"),
                "third poll across rebuilds must trip the halt"
            );
        }
        drop(c); // exactly the tray's per-poll teardown
    }
}

#[test]
fn contiguous_apply_resets_persisted_gap_counter() {
    // Companion to the cross-rebuild trip: once the hole fills and a
    // contiguous apply lands, the persisted counter must restart — a LATER,
    // unrelated gap gets its own full grace window instead of tripping off
    // the stale count.
    let h = harness();
    let s3 = MockS3::default();
    stage_event(&s3, 1, "a.md", b"one");
    stage_event(&s3, 3, "c.md", b"three"); // hole at 2
    // two grace polls on rebuilt consumers → durable counter at 2, not tripped
    for _ in 0..2 {
        let (mut c, pending) = consumer(&h, &s3);
        c.recover(pending);
        assert!(c.poll_once().repair_required.is_empty());
    }
    // the missing event arrives late; a rebuilt consumer applies contiguously
    stage_event(&s3, 2, "b.md", b"two");
    {
        let (mut c, pending) = consumer(&h, &s3);
        c.recover(pending);
        let outcome = c.poll_once();
        assert_eq!(outcome.applied, 2, "{:?}", outcome.transient_errors); // 2 then 3
        assert!(outcome.repair_required.is_empty());
    }
    // re-gap: two polls must NOT trip (fresh grace window, not 2 stale + 2)
    stage_event(&s3, 5, "e.md", b"five"); // hole at 4
    for poll in 0..2 {
        let (mut c, pending) = consumer(&h, &s3);
        c.recover(pending);
        assert!(
            c.poll_once().repair_required.is_empty(),
            "stale counter re-tripped early on poll {poll}"
        );
    }
    // and the third gap poll trips exactly on schedule
    let (mut c, pending) = consumer(&h, &s3);
    c.recover(pending);
    assert_eq!(
        c.poll_once().repair_required.get(ROUTE).map(String::as_str),
        Some("sequence_gap")
    );
}

#[test]
fn repair_cleared_gap_counter_does_not_retrip_next_gap_instantly() {
    // The durable counter sits at >= GAP_GRACE_POLLS after a trip. A Repair
    // record clears the halt and advances the cursor — a NEW gap right after
    // must get its own full grace window (the persisted count is keyed by
    // the cursor it was observed at, so the advance invalidates it).
    let h = harness();
    let s3 = MockS3::default();
    stage_event(&s3, 1, "a.md", b"one");
    stage_event(&s3, 3, "c.md", b"three"); // hole at 2
    for _ in 0..3 {
        let (mut c, pending) = consumer(&h, &s3);
        c.recover(pending);
        c.poll_once();
    }
    // reconcile proved the files present → Repair skips cursor to high water
    h.store
        .append(&JournalRecord::Repair {
            source: "reconcile".into(),
            corrections: serde_json::json!([{"route_id": ROUTE, "set_cursor_sequence": 3}]),
            at: "2026-07-05T12:00:00Z".into(),
        })
        .unwrap();
    // a NEW hole appears immediately (4 missing, 5 visible)
    stage_event(&s3, 5, "e.md", b"five");
    for poll in 0..2 {
        let (mut c, pending) = consumer(&h, &s3);
        c.recover(pending);
        assert!(
            c.poll_once().repair_required.is_empty(),
            "stale counter re-tripped instantly on poll {poll}"
        );
    }
    let (mut c, pending) = consumer(&h, &s3);
    c.recover(pending);
    assert_eq!(
        c.poll_once().repair_required.get(ROUTE).map(String::as_str),
        Some("sequence_gap"),
        "the new gap still trips after its own grace"
    );
}

/// Stage a producer-health-v1 heartbeat for ROUTE under the local binding id.
fn stage_heartbeat(s3: &MockS3, written_at: &str, scan_interval_seconds: u64) {
    let payload = serde_json::json!({
        "schema_version": "producer-health-v1",
        "route_id": ROUTE,
        "binding_id": BINDING,
        "written_at": written_at,
        "scan_interval_seconds": scan_interval_seconds,
    });
    s3.put(
        &format!("_tytus-sync/health/{BINDING}/{ROUTE}.json"),
        payload.to_string().as_bytes(),
    );
}

#[test]
fn producer_heartbeat_fresh_reads_ok() {
    let h = harness();
    let s3 = MockS3::default();
    stage_heartbeat(&s3, &utc_now(), 60);
    let (mut c, _) = consumer(&h, &s3);
    let outcome = c.poll_once();
    assert_eq!(outcome.producer_health.get(ROUTE).copied(), Some("ok"));
}

#[test]
fn producer_heartbeat_old_written_at_reads_stale_without_halting() {
    let h = harness();
    let s3 = MockS3::default();
    stage_heartbeat(&s3, "2020-01-01T00:00:00Z", 60); // years past 3*scan+poll
    stage_event(&s3, 1, "a.md", b"one");
    let (mut c, _) = consumer(&h, &s3);
    let outcome = c.poll_once();
    assert_eq!(outcome.producer_health.get(ROUTE).copied(), Some("stale"));
    // dead-vs-quiet is a STATUS signal: applies proceed, repair untouched
    assert_eq!(outcome.applied, 1, "{:?}", outcome.transient_errors);
    assert!(outcome.repair_required.is_empty());
}

#[test]
fn producer_heartbeat_missing_or_unparsable_reads_unknown() {
    let h = harness();
    let s3 = MockS3::default();
    // no heartbeat object at all: an OLDER producer — must NOT alarm
    let (mut c, _) = consumer(&h, &s3);
    assert_eq!(c.poll_once().producer_health.get(ROUTE).copied(), Some("unknown"));
    // an unparsable body reads unknown too, never stale
    s3.put(&format!("_tytus-sync/health/{BINDING}/{ROUTE}.json"), b"not json");
    assert_eq!(c.poll_once().producer_health.get(ROUTE).copied(), Some("unknown"));
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

#[test]
fn producer_heartbeat_malformed_timestamps_read_unknown() {
    // codex B1/B2: impossible calendar dates and non-digit fractional
    // seconds are malformed input — they must read "unknown", never
    // normalize into an ok/stale verdict.
    for written_at in [
        "2026-02-31T00:00:00Z",     // impossible date (Feb 31)
        "2025-02-29T00:00:00Z",     // non-leap Feb 29
        "2026-01-01T00:00:00.fooZ", // non-digit fractional
        "2026-01-01T00:00:00.Z",    // empty fractional
        "2026-01-01T00:00:00",      // missing Z (non-UTC rejected)
    ] {
        let h = harness();
        let s3 = MockS3::default();
        stage_heartbeat(&s3, written_at, 60);
        let (mut c, _) = consumer(&h, &s3);
        let outcome = c.poll_once();
        assert_eq!(
            outcome.producer_health.get(ROUTE).copied(),
            Some("unknown"),
            "written_at={written_at:?}"
        );
    }
    // control: valid digit fractional stays parseable
    let h = harness();
    let s3 = MockS3::default();
    stage_heartbeat(&s3, &format!("{}", utc_now().replace('Z', ".123Z")), 60);
    let (mut c, _) = consumer(&h, &s3);
    let outcome = c.poll_once();
    assert_eq!(outcome.producer_health.get(ROUTE).copied(), Some("ok"));
}

#[test]
fn gap_counter_for_removed_route_is_pruned_at_build() {
    // codex B3: a route removed from the sidecar must not leave a durable
    // grace counter behind that ambushes the same route on re-add.
    let h = harness();
    let s3 = MockS3::default();
    stage_event(&s3, 5, "later.md", b"x"); // gap: cursor 0 handled separately,
    let (mut c, _) = consumer(&h, &s3);
    let _ = c.poll_once();
    drop(c);
    let gap_file = h.store.dir.join("gap_polls.json");
    // simulate a stale counter for a route that is no longer authorized
    let mut disk: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_slice(&std::fs::read(&gap_file).unwrap_or_else(|_| b"{}".to_vec()))
            .unwrap_or_default();
    disk.insert("r0removed00".into(), serde_json::json!({"cursor": 0, "polls": 9}));
    std::fs::write(&gap_file, serde_json::to_vec(&disk).unwrap()).unwrap();
    let (c2, _) = consumer(&h, &s3); // build prunes unauthorized routes
    drop(c2);
    let disk2: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_slice(&std::fs::read(&gap_file).unwrap()).unwrap();
    assert!(!disk2.contains_key("r0removed00"));
}

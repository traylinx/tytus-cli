//! The consumer apply pipeline (TECH-SPEC §8, contract sync-cursor-v1):
//! bounded StartAfter listing per authorized route, strict contiguous
//! sequence order, hash-verified blob download, ledger-adjudicated keep-both
//! conflicts, dead-letter poison handling, and the normative 6-step
//! durability order. Pull-only: this module never writes to the bucket.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::conflict::{
    bump_conflict_name, case_variant_in_dir, conflict_id, conflict_name, conflict_target,
    utc_basic, ConflictRecordV1, CONFLICT_RECORD_SCHEMA_VERSION,
};
use crate::event::{validate_sync_event, SyncEventV1};
use crate::keysafety::{keys_collide, validate_key};
use crate::s3::{S3Error, S3Ops};
use crate::state::{
    atomic_write, utc_now, BindingIdentity, ConsumerState, DeadLetterClass, JournalRecord,
    StateStore, COMPACT_THRESHOLD,
};

pub const RETRY_BUDGET: u32 = 5;
pub const GAP_GRACE_POLLS: u32 = 3;
pub const DEFAULT_CYCLE_BUDGET: usize = 50;
/// Default consumer poll cadence (seconds) — overridden by the tray, which
/// owns the real scheduler interval; used in the producer-heartbeat
/// staleness rule below.
pub const DEFAULT_POLL_INTERVAL_SECS: u64 = 20;
/// producer-health-v1: the producer is presumed dead once its heartbeat is
/// older than this many scan intervals (plus one consumer poll interval of
/// listing slack).
pub const PRODUCER_DEAD_SCAN_MULTIPLIER: u64 = 3;
pub const PRODUCER_HEALTH_SCHEMA_VERSION: &str = "producer-health-v1";

#[derive(Debug, Default, Clone)]
pub struct PollOutcome {
    pub applied: u64,
    pub conflicts: u64,
    pub dead_letters: u64,
    pub delete_ignored: u64,
    pub transient_errors: Vec<String>,
    /// route_id -> repair_required class currently in effect
    pub repair_required: HashMap<String, String>,
    pub events_seen: u64,
    /// route_id -> "ok" | "stale" | "unknown" from the producer-health-v1
    /// heartbeat (dead-vs-quiet). Truthful-status signal ONLY: a stale
    /// producer never halts the consumer or touches repair_required.
    pub producer_health: HashMap<String, &'static str>,
}

/// Per-route gap-grace progress, DURABLE (gap_polls.json). The tray builds a
/// fresh BindingConsumer for every poll, so an in-memory counter alone can
/// never reach GAP_GRACE_POLLS in production — the halt would simply never
/// fire (sprint phase-3 exit condition). Same side-file discipline as retry
/// attempts (attempts.json, G3 review change 2).
///
/// The count is keyed by the cursor it was observed AT: any durable cursor
/// advance (contiguous apply, dead-letter, SkipSequence, Repair) makes the
/// stored cursor stale, so the next gap observation starts a FRESH grace
/// window instead of re-tripping off a counter left >= grace by an already
/// adjudicated gap. This is the "reset when a Repair/SkipSequence record is
/// folded" requirement expressed without hooking fold() — fold() is pure
/// state with no store access, and Repair records are appended by the tray,
/// outside this consumer.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
struct GapGrace {
    /// last_applied_sequence at the time the gap was observed
    cursor: u64,
    polls: u32,
}

fn attempts_path(store: &StateStore) -> PathBuf {
    store.dir.join("attempts.json")
}

fn load_attempts(store: &StateStore) -> HashMap<String, u32> {
    fs::read(attempts_path(store))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

/// Record one attempt DURABLY and return the new count. Fail closed: if the
/// attempts file cannot be persisted, no retry/dead-letter decision may
/// advance (G3-B review) — the caller surfaces a transient system error and
/// the route halts for the cycle (cursor untouched).
fn record_attempt(store: &StateStore, event_id: &str) -> Result<u32, std::io::Error> {
    let mut attempts = load_attempts(store);
    let count = attempts.get(event_id).copied().unwrap_or(0) + 1;
    if count >= RETRY_BUDGET {
        attempts.remove(event_id);
    } else {
        attempts.insert(event_id.to_string(), count);
    }
    atomic_write(&attempts_path(store), &serde_json::to_vec(&attempts).unwrap_or_default())?;
    Ok(count)
}

fn gap_polls_path(store: &StateStore) -> PathBuf {
    store.dir.join("gap_polls.json")
}

fn load_gap_polls(store: &StateStore) -> HashMap<String, GapGrace> {
    fs::read(gap_polls_path(store))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

/// Persist the gap-grace map. BEST EFFORT, unlike record_attempt's fail
/// closed: a lost gap count merely degrades to the old per-process grace
/// (conservative — the halt fires later, never spuriously), whereas a lost
/// retry attempt would let a poison event dodge its dead-letter budget
/// forever. Writes happen only when a value actually changed — polls are
/// frequent and the steady state is "no gap".
fn save_gap_polls(store: &StateStore, map: &HashMap<String, GapGrace>) {
    let _ = atomic_write(&gap_polls_path(store), &serde_json::to_vec(map).unwrap_or_default());
}

pub struct BindingConsumer<'a, S: S3Ops> {
    pub s3: &'a S,
    pub store: &'a StateStore,
    pub state: &'a mut ConsumerState,
    pub identity: &'a BindingIdentity,
    /// Routes from the LOCAL sidecar only — remote data can never add one.
    pub authorized_routes: Vec<String>,
    pub cycle_budget: usize,
    /// The binding id producers write events under. Producers take it from
    /// the Provider grant (`grant.folder_id`), which is NOT the same registry
    /// as the Mac-local sidecar `folder_id` — the two ids differ on every
    /// binding provisioned through Provider. Local state (journal, conflicts,
    /// dead letters) stays keyed by `identity.binding_id`; only the remote
    /// namespace and the payload trust check use this id.
    pub remote_binding_id: String,
    /// The scheduler's poll cadence, used only in the producer-heartbeat
    /// staleness rule. The tray overwrites this with its real interval.
    pub poll_interval_secs: u64,
    /// Durable gap-grace counters, loaded from gap_polls.json at build so a
    /// per-poll consumer lifecycle still accumulates grace across rebuilds.
    gap_grace: HashMap<String, GapGrace>,
}

impl<'a, S: S3Ops> BindingConsumer<'a, S> {
    pub fn new(
        s3: &'a S,
        store: &'a StateStore,
        state: &'a mut ConsumerState,
        identity: &'a BindingIdentity,
        authorized_routes: Vec<String>,
    ) -> Self {
        Self {
            s3,
            store,
            state,
            identity,
            authorized_routes,
            cycle_budget: DEFAULT_CYCLE_BUDGET,
            remote_binding_id: identity.binding_id.clone(),
            poll_interval_secs: DEFAULT_POLL_INTERVAL_SECS,
            gap_grace: load_gap_polls(store),
        }
    }

    /// Record one gap observation at `cursor`; returns the accumulated count.
    /// A stored count taken at a DIFFERENT cursor is stale by definition (the
    /// cursor only moves via durable records — apply, dead-letter, skip,
    /// repair) and restarts the grace window at 1.
    fn bump_gap_polls(&mut self, route_id: &str, cursor: u64) -> u32 {
        let prev = self.gap_grace.get(route_id).copied().unwrap_or_default();
        let next = GapGrace {
            cursor,
            polls: if prev.cursor == cursor { prev.polls.saturating_add(1) } else { 1 },
        };
        if prev != next {
            self.gap_grace.insert(route_id.to_string(), next);
            save_gap_polls(self.store, &self.gap_grace);
        }
        next.polls
    }

    /// Clear the durable grace counter (empty prefix, contiguous apply).
    /// Only touches the side file when there was a live count to clear.
    fn reset_gap_polls(&mut self, route_id: &str) {
        if self.gap_grace.remove(route_id).is_some() {
            save_gap_polls(self.store, &self.gap_grace);
        }
    }

    fn events_prefix(&self, route_id: &str) -> String {
        format!("_tytus-sync/events/{}/{}/", self.remote_binding_id, route_id)
    }

    fn local_root(&self) -> PathBuf {
        PathBuf::from(&self.identity.local_path)
    }

    /// Recovery: complete or discard pending_apply records replayed from the
    /// journal (call once after load, before the first poll).
    pub fn recover(&mut self, pending: Vec<JournalRecord>) {
        for record in pending {
            if let JournalRecord::PendingApply { route_id, sequence, event_id, key, sha256, tmp_path, .. } = record {
                let finalp = self.local_root().join(&key);
                let done = file_sha256(&finalp).map(|h| h == sha256).unwrap_or(false);
                let _ = fs::remove_file(&tmp_path);
                if done {
                    // pending-apply with matching final hash completes idempotently
                    let _ = self.append_and_fold(&JournalRecord::Applied {
                        route_id, sequence, event_id, key, sha256, at: utc_now(),
                    });
                }
                // otherwise: cursor never advanced, the event re-lists and retries
            }
        }
        // stale temp files from crashed downloads
        if let Ok(entries) = fs::read_dir(self.local_root()) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with(".tytus-sync-tmp-") {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }

    fn append_and_fold(&mut self, record: &JournalRecord) -> Result<(), String> {
        self.store.append(record).map_err(|e| e.to_string())?;
        self.state.fold(record);
        Ok(())
    }

    /// One poll cycle over all authorized routes, sharing the event budget.
    pub fn poll_once(&mut self) -> PollOutcome {
        let mut outcome = PollOutcome::default();
        let mut budget = self.cycle_budget;
        for route_id in self.authorized_routes.clone() {
            if budget == 0 {
                break;
            }
            let used = self.poll_route(&route_id, budget, &mut outcome);
            budget = budget.saturating_sub(used);
        }
        // surface persisted repair states even when nothing was listed
        for (route_id, cursor) in &self.state.routes {
            if let Some(class) = &cursor.repair_required {
                outcome.repair_required.entry(route_id.clone()).or_insert_with(|| class.clone());
            }
        }
        // Producer heartbeat (producer-health-v1, dead-vs-quiet): one small
        // GET per route per poll. Status signal only — never gates applies.
        let now_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        for route_id in self.authorized_routes.clone() {
            let key = format!(
                "_tytus-sync/health/{}/{}.json",
                self.remote_binding_id, route_id
            );
            // Any fetch failure (absent object, transient error) reads as
            // "unknown" — an older producer without heartbeats must NOT alarm.
            let body = self.s3.cat_small(&key).ok();
            let health =
                producer_health_from_heartbeat(body.as_deref(), now_epoch, self.poll_interval_secs);
            outcome.producer_health.insert(route_id, health);
        }
        if self.store.journal_len() > COMPACT_THRESHOLD {
            let _ = self.store.compact(self.state);
        }
        outcome
    }

    fn poll_route(&mut self, route_id: &str, budget: usize, outcome: &mut PollOutcome) -> usize {
        let prefix = self.events_prefix(route_id);
        let cursor_sequence = self.state.route(route_id).last_applied_sequence;
        // StartAfter from the cursor SEQUENCE (not event id): '~' (0x7E) sorts
        // after the '-<ULID>' suffix of any same-sequence key, so this lists
        // strictly-later sequences even when the advance came from a
        // dead-letter or skip_sequence record.
        let start_after = if cursor_sequence > 0 {
            Some(format!("{prefix}{cursor_sequence:020}~"))
        } else {
            None
        };
        let listed = match self.s3.list_after(&prefix, start_after.as_deref(), budget) {
            Ok(rows) => rows,
            Err(e) => {
                outcome.transient_errors.push(format!("{route_id}: list: {e}"));
                return 0;
            }
        };
        if listed.is_empty() {
            // Empty prefix with an existing cursor is NOT a gap (the janitor
            // never deletes the high-water event).
            self.reset_gap_polls(route_id);
            return 0;
        }
        let mut used = 0;
        for object in listed {
            outcome.events_seen += 1;
            used += 1;
            let filename = object.key.rsplit('/').next().unwrap_or("").to_string();
            let event_id = filename.strip_suffix(".json").unwrap_or(&filename).to_string();
            let sequence = match parse_sequence(&event_id) {
                Some(s) => s,
                None => continue, // non-event shapes in the prefix are ignored
            };
            let cursor = self.state.route(route_id).last_applied_sequence;
            if sequence <= cursor {
                continue; // already adjudicated (duplicate listing)
            }
            if self.state.route(route_id).processed_window.iter().any(|e| e == &event_id) {
                continue;
            }
            if sequence != cursor + 1 {
                // "Observed" means observed: the first visible sequence
                // beyond the gap raises the high water even though nothing
                // applies (codex PR#30 round B). Gap adjudication (the
                // auto-reconcile Repair correction) skips the cursor TO this
                // value — the reconcile proved the FILES present, and
                // re-applying a visible event onto an unledgered local file
                // would raise a spurious keep-both conflict.
                {
                    let route = self.state.route(route_id);
                    route.observed_high_water_sequence =
                        sequence.max(route.observed_high_water_sequence);
                }
                if cursor == 0 {
                    // Fresh consumer, non-empty prefix, min visible > 1: the
                    // janitor never deletes the high-water event, so this is
                    // retention pruning by definition — flag IMMEDIATELY, no
                    // grace (G3 review change 3). Grace exists only for a
                    // predecessor that may still be listing.
                    self.state.route(route_id).repair_required = Some("retention_gap".into());
                    outcome.repair_required.insert(route_id.into(), "retention_gap".into());
                    return used;
                }
                // Hole above an existing cursor: bounded grace for eventual
                // listing, then repair-required. The counter is DURABLE
                // (gap_polls.json) — the tray rebuilds this consumer every
                // poll, so grace must accumulate across rebuilds to ever
                // reach the threshold.
                if self.bump_gap_polls(route_id, cursor) >= GAP_GRACE_POLLS {
                    self.state.route(route_id).repair_required = Some("sequence_gap".into());
                    outcome.repair_required.insert(route_id.into(), "sequence_gap".into());
                }
                return used; // never apply out of order
            }
            self.reset_gap_polls(route_id);
            self.state.route(route_id).observed_high_water_sequence =
                sequence.max(self.state.route(route_id).observed_high_water_sequence);
            self.process_event(route_id, sequence, &event_id, &object.key, outcome);
            // Contiguity: if this event did not adjudicate (transient failure,
            // journal trouble), the cursor did not move — the NEXT listed
            // event would look like a hole. Stop the route for this cycle.
            if self.state.route(route_id).last_applied_sequence < sequence
                || outcome.repair_required.contains_key(route_id)
            {
                return used;
            }
        }
        used
    }

    fn process_event(
        &mut self,
        route_id: &str,
        sequence: u64,
        event_id: &str,
        object_key: &str,
        outcome: &mut PollOutcome,
    ) {
        let bytes = match self.s3.cat_small(object_key) {
            Ok(b) => b,
            Err(S3Error::Permanent(detail)) => {
                self.dead_letter(route_id, sequence, event_id, DeadLetterClass::OversizeEvent,
                                 &detail, None, outcome);
                return;
            }
            Err(e) => {
                self.transient(route_id, sequence, event_id, &format!("event fetch: {e}"),
                               DeadLetterClass::IoErrorPermanent, None, outcome);
                return;
            }
        };
        let raw: serde_json::Value = match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(e) => {
                self.dead_letter(route_id, sequence, event_id, DeadLetterClass::SchemaInvalid,
                                 &format!("json: {e}"), None, outcome);
                return;
            }
        };
        let event = match validate_sync_event(&raw) {
            Ok(event) => event,
            Err(e) => {
                self.dead_letter(route_id, sequence, event_id, DeadLetterClass::SchemaInvalid,
                                 &e.to_string(), Some(&raw), outcome);
                return;
            }
        };
        // Trust boundary: the payload's identity must match the prefix and
        // filename it was listed under — including the full event_id (ULID
        // and all, G3 review change 6); the listing is the authority. The
        // payload carries the producer's (remote) binding id, not the local
        // sidecar folder_id.
        if event.route_id != route_id || event.binding_id != self.remote_binding_id
            || event.sequence != sequence || event.event_id != event_id
        {
            self.dead_letter(route_id, sequence, event_id, DeadLetterClass::SchemaInvalid,
                             "payload identity does not match its prefix/filename", Some(&raw), outcome);
            return;
        }
        if event.op == "delete" {
            // v1 policy: parses, counted, ignored — auditable cursor advance.
            // Counter mutates only AFTER the durable record exists (G3
            // review change 7): snapshots never carry non-durable counts.
            if self.append_and_fold(&JournalRecord::SkipSequence {
                route_id: route_id.into(),
                sequence,
                reason: "delete_ignored_v1".into(),
                evidence: event_id.into(),
                at: utc_now(),
            }).is_ok() {
                self.state.delete_events_ignored += 1;
                outcome.delete_ignored += 1;
            }
            return;
        }
        if let Err(e) = validate_key(&event.key) {
            self.dead_letter(route_id, sequence, event_id, DeadLetterClass::PathInvalid,
                             &e.to_string(), Some(&raw), outcome);
            return;
        }
        self.download_verify_apply(route_id, sequence, event_id, &event, &raw, outcome);
    }

    fn download_verify_apply(
        &mut self,
        route_id: &str,
        sequence: u64,
        event_id: &str,
        event: &SyncEventV1,
        raw: &serde_json::Value,
        outcome: &mut PollOutcome,
    ) {
        let root = self.local_root();
        let tmp = root.join(format!(".tytus-sync-tmp-{event_id}"));
        match self.s3.download(&event.content_ref, &tmp) {
            Ok(()) => {}
            Err(S3Error::NotFound(_)) => {
                let _ = fs::remove_file(&tmp);
                self.transient(route_id, sequence, event_id, "blob missing",
                               DeadLetterClass::MissingContentRef, Some(raw), outcome);
                return;
            }
            Err(e) => {
                let _ = fs::remove_file(&tmp);
                self.transient(route_id, sequence, event_id, &format!("blob download: {e}"),
                               DeadLetterClass::IoErrorPermanent, Some(raw), outcome);
                return;
            }
        }
        let got_size = fs::metadata(&tmp).map(|m| m.len()).unwrap_or(0);
        let got_sha = file_sha256(&tmp).unwrap_or_default();
        if got_size != event.object.size || got_sha != event.object.sha256 {
            let _ = fs::remove_file(&tmp);
            self.transient(route_id, sequence, event_id,
                           &format!("hash/size mismatch (got {got_size}B {got_sha})"),
                           DeadLetterClass::HashMismatch, Some(raw), outcome);
            return;
        }

        let finalp = root.join(&event.key);
        let parent = finalp.parent().unwrap_or(&root).to_path_buf();
        let basename = finalp.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();

        // Case/Unicode collision: directory probe + ledger variant check.
        let dir_variant = case_variant_in_dir(&parent, &basename);
        let ledger_variant = self
            .state
            .ledger
            .keys()
            .find(|k| keys_collide(k, &event.key))
            .cloned();
        let collided = dir_variant.is_some() || ledger_variant.is_some();
        if collided && !finalp.exists() {
            let reason = if dir_variant.is_some() { "case_collision" } else { "unicode_collision" };
            self.write_conflict(route_id, sequence, event_id, event, &tmp, reason, outcome);
            return;
        }

        // Ledger adjudication for the exact key.
        if finalp.exists() {
            let current = file_sha256(&finalp).unwrap_or_default();
            let ledger_hash = self.state.ledger.get(&event.key).map(|l| l.sha256.clone());
            let replace_allowed = ledger_hash.as_deref() == Some(current.as_str());
            if current == event.object.sha256 {
                // Bytes already there (e.g. bisync delivered it pre-cutover):
                // record the apply idempotently, no rename needed.
                let _ = fs::remove_file(&tmp);
                if self.append_and_fold(&JournalRecord::Applied {
                    route_id: route_id.into(), sequence, event_id: event_id.into(),
                    key: event.key.clone(), sha256: event.object.sha256.clone(), at: utc_now(),
                }).is_ok() {
                    outcome.applied += 1;
                }
                return;
            }
            if !replace_allowed {
                let reason = if ledger_hash.is_none() { "untracked" } else { "modified_since_last_apply" };
                self.write_conflict(route_id, sequence, event_id, event, &tmp, reason, outcome);
                return;
            }
            // Replace allowed — displaced bytes quarantined FIRST, belt and braces.
            if let Err(e) = self.quarantine(&finalp, &event.key) {
                let _ = fs::remove_file(&tmp);
                self.transient(route_id, sequence, event_id, &format!("quarantine: {e}"),
                               DeadLetterClass::IoErrorPermanent, Some(raw), outcome);
                return;
            }
        }

        // Normative durability order: pending_apply(fsync) -> rename ->
        // fsync dir (best effort) -> applied(fsync).
        if fs::create_dir_all(&parent).is_err() {
            let _ = fs::remove_file(&tmp);
            self.transient(route_id, sequence, event_id, "mkdir failed",
                           DeadLetterClass::IoErrorPermanent, Some(raw), outcome);
            return;
        }
        if self.append_and_fold(&JournalRecord::PendingApply {
            route_id: route_id.into(), sequence, event_id: event_id.into(),
            key: event.key.clone(), sha256: event.object.sha256.clone(),
            tmp_path: tmp.to_string_lossy().into_owned(), at: utc_now(),
        }).is_err() {
            let _ = fs::remove_file(&tmp);
            outcome.transient_errors.push(format!("{route_id}: journal append failed"));
            return;
        }
        if let Err(e) = fs::rename(&tmp, &finalp) {
            let _ = fs::remove_file(&tmp);
            self.transient(route_id, sequence, event_id, &format!("rename: {e}"),
                           DeadLetterClass::IoErrorPermanent, Some(raw), outcome);
            return;
        }
        if let Ok(dir) = fs::File::open(&parent) {
            let _ = dir.sync_all(); // best effort on macOS
        }
        if self.append_and_fold(&JournalRecord::Applied {
            route_id: route_id.into(), sequence, event_id: event_id.into(),
            key: event.key.clone(), sha256: event.object.sha256.clone(), at: utc_now(),
        }).is_ok() {
            outcome.applied += 1;
        }
    }

    fn write_conflict(
        &mut self,
        route_id: &str,
        sequence: u64,
        event_id: &str,
        event: &SyncEventV1,
        tmp: &Path,
        reason_state: &str,
        outcome: &mut PollOutcome,
    ) {
        let root = self.local_root();
        let now = utc_now();
        let basic = utc_basic(&now);
        let sha_hex = event.object.sha256.strip_prefix("sha256:").unwrap_or("").to_string();
        let producer_label = event
            .producer
            .as_ref()
            .and_then(|p| p.agent_label.clone())
            .filter(|l| !l.is_empty())
            .unwrap_or_else(|| route_id.to_string());
        let name = conflict_name(&event.key, &basic, &producer_label, &sha_hex);
        let mut written: Option<String> = None;
        let mut quarantined: Option<String> = None;
        for attempt in 0..10u32 {
            let candidate = if attempt == 0 { name.clone() } else { bump_conflict_name(&name, attempt + 1) };
            match conflict_target(&candidate) {
                None => break, // over-length decorated name -> quarantine below
                Some(valid) => {
                    let target = root.join(valid);
                    if target.exists() {
                        if file_sha256(&target).as_deref() == Some(event.object.sha256.as_str()) {
                            written = Some(valid.to_string()); // idempotent same-sha collision
                            let _ = fs::remove_file(tmp);
                            break;
                        }
                        continue; // different sha -> bump -2, -3, ...
                    }
                    let _ = fs::create_dir_all(target.parent().unwrap_or(&root));
                    if fs::rename(tmp, &target).is_ok() {
                        written = Some(valid.to_string());
                    }
                    break;
                }
            }
        }
        if written.is_none() {
            // decorated name failed policy — incoming copy goes to quarantine
            let qdir = self.store.dir.join("quarantine").join(safe_component(&event.key)).join(&basic);
            let _ = fs::create_dir_all(&qdir);
            let qpath = qdir.join("incoming.bin");
            if fs::rename(tmp, &qpath).is_ok() {
                quarantined = Some(qpath.to_string_lossy().into_owned());
            }
        }
        let local_current = file_sha256(&root.join(&event.key)).unwrap_or_default();
        let id = conflict_id(&basic, &sha_hex);
        let record = ConflictRecordV1 {
            schema_version: CONFLICT_RECORD_SCHEMA_VERSION.into(),
            conflict_id: id.clone(),
            binding_id: self.identity.binding_id.clone(),
            bucket: self.identity.bucket.clone(),
            key: event.key.clone(),
            detected_at: now.clone(),
            policy: "keep_both".into(),
            reason: if reason_state == "case_collision" || reason_state == "unicode_collision" {
                reason_state.into()
            } else {
                "content_conflict".into()
            },
            local_path_kept: event.key.clone(),
            local_sha256: local_current,
            local_ledger_state: reason_state.into(),
            incoming_event_id: event_id.into(),
            incoming_route_id: route_id.into(),
            incoming_sha256: event.object.sha256.clone(),
            incoming_path_written: written,
            quarantine_path: quarantined,
            resolved_at: None,
        };
        let path = self.store.dir.join("conflicts").join(format!("{id}.json"));
        let _ = atomic_write(&path, &serde_json::to_vec_pretty(&record).unwrap_or_default());
        if self.append_and_fold(&JournalRecord::Conflict {
            route_id: route_id.into(), sequence, event_id: event_id.into(),
            key: event.key.clone(), conflict_id: id, at: utc_now(),
        }).is_ok() {
            outcome.conflicts += 1;
        }
    }

    fn quarantine(&self, path: &Path, key: &str) -> Result<(), std::io::Error> {
        let dir = self
            .store
            .dir
            .join("quarantine")
            .join(safe_component(key))
            .join(utc_basic(&utc_now()));
        fs::create_dir_all(&dir)?;
        let target = dir.join(path.file_name().unwrap_or_default());
        fs::copy(path, &target)?;
        // Displaced bytes must be DURABLE before the caller overwrites the
        // original (G3 review change 1): fsync the copy and its directory,
        // else a crash right after the rename keeps the new file and loses
        // the only other copy of the old bytes. The handle needs write
        // access: Windows' FlushFileBuffers rejects read-only handles with
        // ERROR_ACCESS_DENIED (Unix fsync accepts either).
        let file = fs::OpenOptions::new().write(true).open(&target)?;
        file.sync_all()?;
        if let Ok(dirf) = fs::File::open(&dir) {
            let _ = dirf.sync_all(); // best effort on macOS
        }
        Ok(())
    }

    /// Transient failure: retry against the DURABLE budget; past budget ->
    /// dead-letter with the given class (poison rule: later events keep
    /// applying). Attempts persist across restarts (attempts.json) so a
    /// poison event cannot block a route forever by riding restarts.
    fn transient(
        &mut self,
        route_id: &str,
        sequence: u64,
        event_id: &str,
        detail: &str,
        class: DeadLetterClass,
        raw: Option<&serde_json::Value>,
        outcome: &mut PollOutcome,
    ) {
        match record_attempt(self.store, event_id) {
            Err(e) => {
                // Attempt not durable -> no decision advances; the route
                // halts for this cycle and the event retries next poll
                // without consuming budget.
                outcome
                    .transient_errors
                    .push(format!("{route_id}/{event_id}: attempts persist failed: {e}"));
            }
            Ok(count) if count >= RETRY_BUDGET => {
                self.dead_letter(route_id, sequence, event_id, class, detail, raw, outcome);
            }
            Ok(_) => {
                outcome.transient_errors.push(format!("{route_id}/{event_id}: {detail}"));
            }
        }
    }

    fn dead_letter(
        &mut self,
        route_id: &str,
        sequence: u64,
        event_id: &str,
        class: DeadLetterClass,
        detail: &str,
        raw: Option<&serde_json::Value>,
        outcome: &mut PollOutcome,
    ) {
        let payload = serde_json::json!({
            "event_id": event_id,
            "route_id": route_id,
            "sequence": sequence,
            "class": class,
            "detail": detail,
            "payload": raw,
            "at": utc_now(),
        });
        let path = self.store.dir.join("dead-letter").join(format!("{event_id}.json"));
        let _ = atomic_write(&path, &serde_json::to_vec_pretty(&payload).unwrap_or_default());
        if self.append_and_fold(&JournalRecord::DeadLetter {
            route_id: route_id.into(), sequence, event_id: event_id.into(),
            class, detail: detail.chars().take(300).collect(), at: utc_now(),
        }).is_ok() {
            outcome.dead_letters += 1;
        }
    }
}

/// Pure dead-vs-quiet rule over a producer-health-v1 heartbeat body.
///
/// - `None` body, unparsable JSON, wrong/missing schema fields → "unknown"
///   (an older producer never wrote heartbeats — that must NOT alarm).
/// - `now - written_at > 3 * scan_interval + poll_interval` → "stale": the
///   producer missed three consecutive scan cycles plus one poll of listing
///   slack, which distinguishes a DEAD producer from a QUIET one (a quiet
///   producer still rewrites its heartbeat every scan).
/// - otherwise → "ok". A future `written_at` (clock skew) saturates to ok:
///   skew must not fabricate a dead producer.
pub fn producer_health_from_heartbeat(
    body: Option<&[u8]>,
    now_epoch: u64,
    poll_interval_secs: u64,
) -> &'static str {
    let Some(bytes) = body else { return "unknown" };
    let Ok(json) = serde_json::from_slice::<serde_json::Value>(bytes) else { return "unknown" };
    if json.get("schema_version").and_then(|v| v.as_str()) != Some(PRODUCER_HEALTH_SCHEMA_VERSION) {
        return "unknown";
    }
    let Some(written_at) = json
        .get("written_at")
        .and_then(|v| v.as_str())
        .and_then(parse_utc_epoch)
    else {
        return "unknown";
    };
    let Some(scan_interval) = json.get("scan_interval_seconds").and_then(|v| v.as_u64()) else {
        return "unknown";
    };
    let dead_after = PRODUCER_DEAD_SCAN_MULTIPLIER
        .saturating_mul(scan_interval)
        .saturating_add(poll_interval_secs);
    if now_epoch.saturating_sub(written_at) > dead_after {
        "stale"
    } else {
        "ok"
    }
}

/// Parse `YYYY-MM-DDTHH:MM:SS[.fff]Z` (ISO 8601, UTC only) to epoch seconds.
/// Mirror of `state::utc_now()` — days-from-civil (Howard Hinnant), no
/// chrono dependency. Anything non-UTC or malformed is None (→ "unknown").
fn parse_utc_epoch(iso: &str) -> Option<u64> {
    let rest = iso.strip_suffix('Z')?;
    let (date, time) = rest.split_once('T')?;
    let mut parts = date.split('-');
    let y: i64 = parts.next()?.parse().ok()?;
    let mo: i64 = parts.next()?.parse().ok()?;
    let d: i64 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    let time = time.split('.').next()?; // tolerate fractional seconds
    let mut parts = time.split(':');
    let h: u64 = parts.next()?.parse().ok()?;
    let mi: u64 = parts.next()?.parse().ok()?;
    let s: u64 = parts.next()?.parse().ok()?;
    let ranges_ok =
        (1..=12).contains(&mo) && (1..=31).contains(&d) && h <= 23 && mi <= 59 && s <= 60;
    if parts.next().is_some() || !ranges_ok {
        return None;
    }
    // days-from-civil, the exact inverse of utc_now()'s civil-from-days
    let y_adj = if mo <= 2 { y - 1 } else { y };
    let era = y_adj.div_euclid(400);
    let yoe = y_adj - era * 400;
    let doy = (153 * (if mo > 2 { mo - 3 } else { mo + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    if days < 0 {
        return None; // pre-epoch heartbeat can only be garbage
    }
    Some(days as u64 * 86_400 + h * 3600 + mi * 60 + s)
}

pub fn parse_sequence(event_id: &str) -> Option<u64> {
    if event_id.len() < 21 || event_id.as_bytes().get(20) != Some(&b'-') {
        return None;
    }
    event_id[..20].parse().ok()
}

pub fn file_sha256(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Some(format!("sha256:{}", hex::encode(hasher.finalize())))
}

fn safe_component(key: &str) -> String {
    key.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .take(120)
        .collect()
}

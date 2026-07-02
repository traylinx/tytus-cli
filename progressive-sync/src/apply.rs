//! The consumer apply pipeline (TECH-SPEC §8, contract sync-cursor-v1):
//! bounded StartAfter listing per authorized route, strict contiguous
//! sequence order, hash-verified blob download, ledger-adjudicated keep-both
//! conflicts, dead-letter poison handling, and the normative 6-step
//! durability order. Pull-only: this module never writes to the bucket.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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
}

/// Per-route in-memory progress (attempts/backoff/grace are re-derived after
/// a restart; durable facts live in the journal only).
#[derive(Default)]
struct RouteRuntime {
    gap_polls: u32,
    attempts: HashMap<String, u32>, // event_id -> failed attempts
}

pub struct BindingConsumer<'a, S: S3Ops> {
    pub s3: &'a S,
    pub store: &'a StateStore,
    pub state: &'a mut ConsumerState,
    pub identity: &'a BindingIdentity,
    /// Routes from the LOCAL sidecar only — remote data can never add one.
    pub authorized_routes: Vec<String>,
    pub cycle_budget: usize,
    runtime: HashMap<String, RouteRuntime>,
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
            runtime: HashMap::new(),
        }
    }

    fn events_prefix(&self, route_id: &str) -> String {
        format!("_tytus-sync/events/{}/{}/", self.identity.binding_id, route_id)
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
            self.runtime.entry(route_id.into()).or_default().gap_polls = 0;
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
                // Hole: bounded grace for eventual listing, then repair-required.
                let runtime = self.runtime.entry(route_id.into()).or_default();
                runtime.gap_polls += 1;
                let class = if cursor == 0 { "retention_gap" } else { "sequence_gap" };
                if runtime.gap_polls >= GAP_GRACE_POLLS {
                    self.state.route(route_id).repair_required = Some(class.into());
                    outcome.repair_required.insert(route_id.into(), class.into());
                }
                return used; // never apply out of order
            }
            self.runtime.entry(route_id.into()).or_default().gap_polls = 0;
            self.state.route(route_id).observed_high_water_sequence =
                sequence.max(self.state.route(route_id).observed_high_water_sequence);
            self.process_event(route_id, sequence, &event_id, &object.key, outcome);
            if outcome.repair_required.contains_key(route_id) {
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
        // Trust boundary: the payload's route/binding must match the prefix
        // it was listed under; the prefix is the authority.
        if event.route_id != route_id || event.binding_id != self.identity.binding_id
            || event.sequence != sequence
        {
            self.dead_letter(route_id, sequence, event_id, DeadLetterClass::SchemaInvalid,
                             "payload identity does not match its prefix", Some(&raw), outcome);
            return;
        }
        if event.op == "delete" {
            // v1 policy: parses, counted, ignored — auditable cursor advance.
            self.state.delete_events_ignored += 1;
            outcome.delete_ignored += 1;
            let _ = self.append_and_fold(&JournalRecord::SkipSequence {
                route_id: route_id.into(),
                sequence,
                reason: "delete_ignored_v1".into(),
                evidence: event_id.into(),
                at: utc_now(),
            });
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
        let mut name = conflict_name(&event.key, &basic, &producer_label, &sha_hex);
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
        fs::copy(path, target)?;
        Ok(())
    }

    /// Transient failure: retry against the budget; past budget -> dead-letter
    /// with the given class (poison rule: later events keep applying).
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
        let attempts = {
            let runtime = self.runtime.entry(route_id.into()).or_default();
            let count = runtime.attempts.entry(event_id.into()).or_insert(0);
            *count += 1;
            *count
        };
        if attempts >= RETRY_BUDGET {
            self.dead_letter(route_id, sequence, event_id, class, detail, raw, outcome);
            if let Some(runtime) = self.runtime.get_mut(route_id) {
                runtime.attempts.remove(event_id);
            }
        } else {
            outcome.transient_errors.push(format!("{route_id}/{event_id} attempt {attempts}: {detail}"));
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

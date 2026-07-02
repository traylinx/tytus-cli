//! Consumer durable state (contract sync-cursor-v1): fsync'd append-only
//! journal as the source of truth, periodically compacted into a snapshot.
//! No cursor value may exist anywhere beyond the last durable journal record.

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const SNAPSHOT_SCHEMA_VERSION: &str = "sync-consumer-state-v1";
pub const PROCESSED_WINDOW: usize = 200;
/// Compact once the journal tail exceeds this many records.
pub const COMPACT_THRESHOLD: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeadLetterClass {
    SchemaInvalid,
    PathInvalid,
    CaseConflict,
    HashMismatch,
    MissingContentRef,
    UnexpectedRoute,
    OversizeEvent,
    IoErrorPermanent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum JournalRecord {
    PendingApply {
        route_id: String,
        sequence: u64,
        event_id: String,
        key: String,
        sha256: String,
        tmp_path: String,
        at: String,
    },
    Applied {
        route_id: String,
        sequence: u64,
        event_id: String,
        key: String,
        sha256: String,
        at: String,
    },
    DeadLetter {
        route_id: String,
        sequence: u64,
        event_id: String,
        class: DeadLetterClass,
        detail: String,
        at: String,
    },
    Conflict {
        route_id: String,
        sequence: u64,
        event_id: String,
        key: String,
        conflict_id: String,
        at: String,
    },
    QuarantineRoute {
        route_id: String,
        reason: String,
        at: String,
    },
    SkipSequence {
        route_id: String,
        sequence: u64,
        reason: String,
        evidence: String,
        at: String,
    },
    Repair {
        source: String,
        corrections: serde_json::Value,
        at: String,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RouteCursor {
    pub last_applied_sequence: u64,
    #[serde(default)]
    pub last_applied_event_id: Option<String>,
    #[serde(default)]
    pub observed_high_water_sequence: u64,
    #[serde(default)]
    pub processed_window: Vec<String>,
    /// DERIVED state (G3 review): set from live listing observations, cleared
    /// by durable skip_sequence/repair records. Never serialized — a snapshot
    /// must contain nothing that is not journal-derived; after a restart the
    /// condition re-derives from the next polls.
    #[serde(skip)]
    pub repair_required: Option<String>,
    #[serde(default)]
    pub dead_letter_count: u64,
    #[serde(default)]
    pub last_applied_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub route_id: String,
    pub sequence: u64,
    pub sha256: String,
    pub applied_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerState {
    pub schema_version: String,
    pub binding_id: String,
    pub bucket: String,
    #[serde(default)]
    pub consumer_id: String,
    #[serde(default)]
    pub routes: BTreeMap<String, RouteCursor>,
    #[serde(default)]
    pub ledger: BTreeMap<String, LedgerEntry>,
    #[serde(default)]
    pub unexpected_route_count: u64,
    #[serde(default)]
    pub delete_events_ignored: u64,
}

impl ConsumerState {
    pub fn new(binding_id: &str, bucket: &str, consumer_id: &str) -> Self {
        Self {
            schema_version: SNAPSHOT_SCHEMA_VERSION.into(),
            binding_id: binding_id.into(),
            bucket: bucket.into(),
            consumer_id: consumer_id.into(),
            routes: BTreeMap::new(),
            ledger: BTreeMap::new(),
            unexpected_route_count: 0,
            delete_events_ignored: 0,
        }
    }

    pub fn route(&mut self, route_id: &str) -> &mut RouteCursor {
        self.routes.entry(route_id.to_string()).or_default()
    }

    fn push_window(cursor: &mut RouteCursor, event_id: &str) {
        cursor.processed_window.push(event_id.to_string());
        let excess = cursor.processed_window.len().saturating_sub(PROCESSED_WINDOW);
        if excess > 0 {
            cursor.processed_window.drain(0..excess);
        }
    }

    /// Fold one journal record into the in-memory state (used both at replay
    /// and after each live append). Cursor advances are exactly the durable
    /// records that permit them.
    pub fn fold(&mut self, record: &JournalRecord) {
        match record {
            JournalRecord::PendingApply { .. } => {}
            JournalRecord::Applied { route_id, sequence, event_id, key, sha256, at } => {
                let cursor = self.route(route_id);
                cursor.last_applied_sequence = (*sequence).max(cursor.last_applied_sequence);
                cursor.last_applied_event_id = Some(event_id.clone());
                cursor.last_applied_at = Some(at.clone());
                Self::push_window(cursor, event_id);
                self.ledger.insert(key.clone(), LedgerEntry {
                    route_id: route_id.clone(),
                    sequence: *sequence,
                    sha256: sha256.clone(),
                    applied_at: at.clone(),
                });
            }
            JournalRecord::DeadLetter { route_id, sequence, event_id, .. } => {
                let cursor = self.route(route_id);
                cursor.last_applied_sequence = (*sequence).max(cursor.last_applied_sequence);
                cursor.dead_letter_count += 1;
                Self::push_window(cursor, event_id);
            }
            JournalRecord::Conflict { route_id, sequence, event_id, .. } => {
                let cursor = self.route(route_id);
                cursor.last_applied_sequence = (*sequence).max(cursor.last_applied_sequence);
                Self::push_window(cursor, event_id);
            }
            JournalRecord::QuarantineRoute { .. } => {
                self.unexpected_route_count += 1;
            }
            JournalRecord::SkipSequence { route_id, sequence, .. } => {
                let cursor = self.route(route_id);
                cursor.last_applied_sequence = (*sequence).max(cursor.last_applied_sequence);
                // A durable, evidenced skip clears the repair-required state
                // for the gap it adjudicated (contract: auditable-or-not-at-all).
                cursor.repair_required = None;
            }
            JournalRecord::Repair { corrections, .. } => {
                // Reconcile is ground truth: a durable repair record clears
                // the repair-required state for every route it corrected
                // (G3 review change 5). Cursor rewrites ride the corrections.
                if let Some(rows) = corrections.as_array() {
                    for row in rows {
                        let Some(route_id) = row.get("route_id").and_then(|v| v.as_str()) else {
                            continue;
                        };
                        let cursor = self.route(route_id);
                        cursor.repair_required = None;
                        if let Some(seq) = row.get("set_cursor_sequence").and_then(|v| v.as_u64()) {
                            cursor.last_applied_sequence = seq.max(cursor.last_applied_sequence);
                        }
                    }
                }
            }
        }
    }
}

/// The on-disk store for one binding: journal + snapshot + side dirs.
pub struct StateStore {
    pub dir: PathBuf,
}

#[derive(Debug)]
pub enum StateError {
    Io(std::io::Error),
    Corrupt(String),
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "state io: {e}"),
            Self::Corrupt(m) => write!(f, "state corrupt: {m}"),
        }
    }
}

impl std::error::Error for StateError {}

impl From<std::io::Error> for StateError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingIdentity {
    pub binding_id: String,
    pub bucket: String,
    pub local_path: String,
    #[serde(default)]
    pub alias: Option<String>,
}

impl StateStore {
    pub fn open(root: &Path, identity: &BindingIdentity) -> Result<Self, StateError> {
        let dir = root.join(&identity.binding_id);
        fs::create_dir_all(dir.join("dead-letter"))?;
        fs::create_dir_all(dir.join("conflicts"))?;
        fs::create_dir_all(dir.join("quarantine"))?;
        let identity_path = dir.join("binding.json");
        if !identity_path.exists() {
            atomic_write(&identity_path, &serde_json::to_vec_pretty(identity).unwrap())?;
        }
        Ok(Self { dir })
    }

    pub fn journal_path(&self) -> PathBuf {
        self.dir.join("journal.jsonl")
    }

    pub fn snapshot_path(&self) -> PathBuf {
        self.dir.join("snapshot.json")
    }

    /// Append one record with fsync (normative durability step).
    pub fn append(&self, record: &JournalRecord) -> Result<(), StateError> {
        let mut file = OpenOptions::new().create(true).append(true).open(self.journal_path())?;
        let mut line = serde_json::to_vec(record).map_err(|e| StateError::Corrupt(e.to_string()))?;
        line.push(b'\n');
        file.write_all(&line)?;
        file.sync_data()?;
        Ok(())
    }

    /// Load state: snapshot (if any) + full journal replay on top. Returns the
    /// state plus any pending_apply records with no later matching applied —
    /// the recovery set the apply pipeline re-verifies idempotently.
    pub fn load(&self, identity: &BindingIdentity, consumer_id: &str)
        -> Result<(ConsumerState, Vec<JournalRecord>), StateError>
    {
        let mut state = match fs::read(self.snapshot_path()) {
            Ok(bytes) => serde_json::from_slice::<ConsumerState>(&bytes)
                .map_err(|e| StateError::Corrupt(format!("snapshot: {e}")))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                ConsumerState::new(&identity.binding_id, &identity.bucket, consumer_id)
            }
            Err(e) => return Err(e.into()),
        };
        let mut pending: BTreeMap<String, JournalRecord> = BTreeMap::new();
        match File::open(self.journal_path()) {
            Ok(file) => {
                for line in BufReader::new(file).lines() {
                    let line = line?;
                    if line.trim().is_empty() {
                        continue;
                    }
                    // A torn final line (crash mid-append) is tolerated: the
                    // fsync ordering guarantees every COMPLETE line is durable
                    // and the torn one had not yet taken effect.
                    let record: JournalRecord = match serde_json::from_str(&line) {
                        Ok(r) => r,
                        Err(_) => continue,
                    };
                    match &record {
                        JournalRecord::PendingApply { event_id, .. } => {
                            pending.insert(event_id.clone(), record.clone());
                        }
                        JournalRecord::Applied { event_id, .. }
                        | JournalRecord::DeadLetter { event_id, .. }
                        | JournalRecord::Conflict { event_id, .. } => {
                            pending.remove(event_id);
                        }
                        _ => {}
                    }
                    state.fold(&record);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        Ok((state, pending.into_values().collect()))
    }

    /// Compact: write the folded state as the snapshot, then truncate the
    /// journal. Snapshot write is atomic + fsync'd BEFORE truncation, so the
    /// cursor can never regress past what the snapshot folded in.
    pub fn compact(&self, state: &ConsumerState) -> Result<(), StateError> {
        let bytes = serde_json::to_vec_pretty(state).map_err(|e| StateError::Corrupt(e.to_string()))?;
        atomic_write(&self.snapshot_path(), &bytes)?;
        let file = OpenOptions::new().create(true).write(true).truncate(true).open(self.journal_path())?;
        file.sync_data()?;
        Ok(())
    }

    pub fn journal_len(&self) -> usize {
        fs::read_to_string(self.journal_path())
            .map(|s| s.lines().filter(|l| !l.trim().is_empty()).count())
            .unwrap_or(0)
    }
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), std::io::Error> {
    let tmp = path.with_extension("tmp");
    {
        let mut file = File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_data()?;
    }
    fs::rename(&tmp, path)
}

pub fn utc_now() -> String {
    // Seconds precision is plenty for diagnostics; ordering authority is the
    // sequence, never timestamps.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86_400;
    let (h, m, s) = ((secs % 86_400) / 3600, (secs % 3600) / 60, secs % 60);
    // civil-from-days (Howard Hinnant's algorithm), UTC
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

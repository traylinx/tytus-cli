//! Contract `sync-event-v1`: one immutable object per changed key, written by
//! the pod owning the route prefix. Consumers discover changes by bounded
//! listing (`StartAfter=<cursor>`) and download the immutable `content_ref` —
//! never the mutable user key.

use serde::{Deserialize, Serialize};

pub const SYNC_EVENT_SCHEMA_VERSION: &str = "sync-event-v1";

/// Why an event payload failed validation. Mirrors the reason strings of the
/// Python producer gate so cross-repo test fixtures stay comparable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventValidationError {
    NotAnObject,
    SchemaVersion,
    MissingField(&'static str),
    EventIdFormat,
    EventIdSequenceMismatch,
    Op,
    ContentRefFormat,
    ContentRefDigestMismatch,
}

impl std::fmt::Display for EventValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAnObject => write!(f, "not-an-object"),
            Self::SchemaVersion => write!(f, "schema_version"),
            Self::MissingField(name) => write!(f, "missing:{name}"),
            Self::EventIdFormat => write!(f, "event_id:format"),
            Self::EventIdSequenceMismatch => write!(f, "event_id:sequence-mismatch"),
            Self::Op => write!(f, "op"),
            Self::ContentRefFormat => write!(f, "content_ref:format"),
            Self::ContentRefDigestMismatch => write!(f, "content_ref:digest-mismatch"),
        }
    }
}

impl std::error::Error for EventValidationError {}

/// Object metadata carried by an event. `etag` is metadata only — NEVER
/// identity (multipart ETags are not content hashes); `sha256` is the only
/// identity, and it must match the digest embedded in `content_ref`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEventObject {
    pub size: u64,
    pub sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mtime_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
}

/// Producer identity block (diagnostic only; ordering authority is
/// `sequence`, trust authority is the route prefix the event was listed under).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEventProducer {
    #[serde(default)]
    pub agent_label: Option<String>,
    #[serde(default)]
    pub implementation: Option<String>,
}

/// One immutable `sync-event-v1` object. Unknown JSON fields are ignored by
/// design (forward compatibility); serde's default non-strict deserialization
/// gives us that for free.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEventV1 {
    pub schema_version: String,
    pub binding_id: String,
    pub bucket: String,
    pub route_id: String,
    pub sequence: u64,
    pub event_id: String,
    pub op: String,
    pub key: String,
    pub content_ref: String,
    pub object: SyncEventObject,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub producer: Option<SyncEventProducer>,
}

fn is_lower_hex64(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Crockford base32 alphabet as used by ULID (uppercase, no I, L, O, U).
fn is_ulid(s: &str) -> bool {
    s.len() == 26
        && s.bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'A'..=b'H' | b'J' | b'K' | b'M' | b'N' | b'P'..=b'T' | b'V'..=b'Z'))
}

/// Validate a raw JSON value as a `SyncEventV1` payload.
///
/// Takes `serde_json::Value` (not the typed struct) because the consumer must
/// classify malformed payloads as `schema_invalid` dead-letters rather than
/// fail deserialization opaquely — and because "unknown fields are ignored"
/// while "missing required fields reject" both need field-level checks.
pub fn validate_sync_event(doc: &serde_json::Value) -> Result<SyncEventV1, EventValidationError> {
    let map = doc.as_object().ok_or(EventValidationError::NotAnObject)?;
    if map.get("schema_version").and_then(|v| v.as_str()) != Some(SYNC_EVENT_SCHEMA_VERSION) {
        return Err(EventValidationError::SchemaVersion);
    }
    let str_field = |name: &'static str| -> Result<&str, EventValidationError> {
        map.get(name)
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .ok_or(EventValidationError::MissingField(name))
    };
    for field in ["binding_id", "bucket", "route_id", "event_id", "op", "key", "content_ref", "created_at"] {
        str_field(field)?;
    }
    let sequence = map
        .get("sequence")
        .and_then(|v| v.as_u64())
        .filter(|&s| s >= 1)
        .ok_or(EventValidationError::MissingField("sequence"))?;
    let obj = map
        .get("object")
        .and_then(|v| v.as_object())
        .ok_or(EventValidationError::MissingField("object"))?;
    obj.get("size")
        .and_then(|v| v.as_u64())
        .ok_or(EventValidationError::MissingField("object.size"))?;

    let event_id = str_field("event_id")?;
    if event_id.len() != 20 + 1 + 26 || !event_id.is_ascii() {
        return Err(EventValidationError::EventIdFormat);
    }
    let (seq_part, rest) = event_id.split_at(20);
    let ulid_part = rest.strip_prefix('-').ok_or(EventValidationError::EventIdFormat)?;
    if !seq_part.bytes().all(|b| b.is_ascii_digit()) || !is_ulid(ulid_part) {
        return Err(EventValidationError::EventIdFormat);
    }
    let id_seq: u64 = seq_part.parse().map_err(|_| EventValidationError::EventIdFormat)?;
    if id_seq != sequence {
        return Err(EventValidationError::EventIdSequenceMismatch);
    }

    let op = str_field("op")?;
    if op != "put" && op != "delete" {
        return Err(EventValidationError::Op);
    }

    let sha = obj
        .get("sha256")
        .and_then(|v| v.as_str())
        .and_then(|s| s.strip_prefix("sha256:"))
        .filter(|hex| is_lower_hex64(hex))
        .ok_or(EventValidationError::MissingField("object.sha256"))?;

    let content_ref = str_field("content_ref")?;
    let ref_hex = content_ref
        .strip_prefix("_tytus-sync/blobs/sha256/")
        .filter(|hex| is_lower_hex64(hex))
        .ok_or(EventValidationError::ContentRefFormat)?;
    if ref_hex != sha {
        return Err(EventValidationError::ContentRefDigestMismatch);
    }

    serde_json::from_value(doc.clone()).map_err(|_| EventValidationError::NotAnObject)
}

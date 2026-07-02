//! Keep-both conflict handling (contract conflict-record-v1): ledger-based
//! detection, deterministic conflict naming, durable records. Never
//! last-writer-wins; never deletes either version.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::keysafety::{validate_key, MAX_SEGMENT_BYTES};

pub const CONFLICT_RECORD_SCHEMA_VERSION: &str = "conflict-record-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRecordV1 {
    pub schema_version: String,
    pub conflict_id: String,
    pub binding_id: String,
    pub bucket: String,
    pub key: String,
    pub detected_at: String,
    pub policy: String,
    pub reason: String,
    pub local_path_kept: String,
    pub local_sha256: String,
    pub local_ledger_state: String,
    pub incoming_event_id: String,
    pub incoming_route_id: String,
    pub incoming_sha256: String,
    pub incoming_path_written: Option<String>,
    pub quarantine_path: Option<String>,
    pub resolved_at: Option<String>,
}

fn sanitize_producer(label: &str) -> String {
    let cleaned: String = label
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' })
        .collect();
    let trimmed = cleaned.trim_matches('-');
    if trimmed.is_empty() { "unknown".into() } else { trimmed.chars().take(24).collect() }
}

/// Split a key's basename per the normative rule: ext = substring from the
/// LAST dot (`a.tar.gz` -> stem `a.tar`, ext `.gz`; no dot -> empty ext).
fn split_stem_ext(basename: &str) -> (&str, &str) {
    match basename.rfind('.') {
        Some(pos) if pos > 0 => (&basename[..pos], &basename[pos..]),
        _ => (basename, ""),
    }
}

/// The deterministic conflict name for an incoming event, WITHOUT the `-N`
/// bump (the caller bumps on different-sha collisions).
///
/// `<stem>.conflict-<UTCbasic>-<producer>-<8hex><ext>`
pub fn conflict_name(key: &str, detected_at_basic: &str, producer_label: &str, incoming_sha_hex: &str) -> String {
    let (dir, basename) = match key.rfind('/') {
        Some(pos) => (&key[..=pos], &key[pos + 1..]),
        None => ("", key),
    };
    let (stem, ext) = split_stem_ext(basename);
    let producer = sanitize_producer(producer_label);
    let short = &incoming_sha_hex[..8.min(incoming_sha_hex.len())];
    format!("{dir}{stem}.conflict-{detected_at_basic}-{producer}-{short}{ext}")
}

pub fn bump_conflict_name(name: &str, attempt: u32) -> String {
    let (dir, basename) = match name.rfind('/') {
        Some(pos) => (&name[..=pos], &name[pos + 1..]),
        None => ("", name),
    };
    let (stem, ext) = split_stem_ext(basename);
    format!("{dir}{stem}-{attempt}{ext}")
}

/// Where the incoming bytes land: the conflict name when it passes key
/// safety + length policy, else None (caller quarantines instead).
pub fn conflict_target(name: &str) -> Option<&str> {
    let basename = name.rsplit('/').next().unwrap_or(name);
    if basename.len() > MAX_SEGMENT_BYTES {
        return None;
    }
    validate_key(name).ok()
}

/// "20260702T121530Z" from "2026-07-02T12:15:30Z".
pub fn utc_basic(iso: &str) -> String {
    iso.chars().filter(|c| c.is_ascii_alphanumeric()).collect()
}

pub fn conflict_id(detected_at_basic: &str, sha_hex: &str) -> String {
    format!("cf_{detected_at_basic}_{}", &sha_hex[..8.min(sha_hex.len())])
}

/// Probe a directory for an entry that collides with `basename` by case or
/// Unicode normalization (macOS-default-volume hazard). Directory-listing
/// compare, not stat — stat would follow the alias and lie.
pub fn case_variant_in_dir(dir: &Path, basename: &str) -> Option<String> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if crate::keysafety::keys_collide(&name, basename) {
            return Some(name);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn naming_matches_contract_example() {
        assert_eq!(
            conflict_name("proposal.md", "20260702T121530Z", "Hermie", "a1b2c3d4ffff"),
            "proposal.conflict-20260702T121530Z-hermie-a1b2c3d4.md"
        );
    }

    #[test]
    fn tar_gz_splits_on_last_dot() {
        let name = conflict_name("a.tar.gz", "20260702T121530Z", "lisa", "deadbeef00");
        assert_eq!(name, "a.tar.conflict-20260702T121530Z-lisa-deadbeef.gz");
    }

    #[test]
    fn no_extension_appends_suffix() {
        let name = conflict_name("Makefile", "20260702T121530Z", "r0", "cafebabe11");
        assert_eq!(name, "Makefile.conflict-20260702T121530Z-r0-cafebabe");
    }

    #[test]
    fn nested_key_keeps_directory() {
        let name = conflict_name("docs/spec/plan.md", "20260702T121530Z", "Claus Pod", "0011223344");
        assert_eq!(name, "docs/spec/plan.conflict-20260702T121530Z-claus-pod-00112233.md");
    }

    #[test]
    fn producer_falls_back_and_sanitizes() {
        assert_eq!(sanitize_producer(""), "unknown");
        // ä, /, … and the space each fold to '-'; edges trimmed
        assert_eq!(sanitize_producer("Läsa/… pod"), "l-sa---pod");
        assert_eq!(sanitize_producer("---"), "unknown");
    }

    #[test]
    fn bump_inserts_before_ext() {
        assert_eq!(bump_conflict_name("a.conflict-x-y-z.md", 2), "a.conflict-x-y-z-2.md");
        assert_eq!(bump_conflict_name("noext", 3), "noext-3");
    }

    #[test]
    fn overlength_decorated_name_rejected() {
        let long = format!("{}.md", "x".repeat(250));
        let name = conflict_name(&long, "20260702T121530Z", "hermie", "a1b2c3d4");
        assert!(conflict_target(&name).is_none(), "must route to quarantine");
    }

    #[test]
    fn utc_basic_strips_punctuation() {
        assert_eq!(utc_basic("2026-07-02T12:15:30Z"), "20260702T121530Z");
    }
}

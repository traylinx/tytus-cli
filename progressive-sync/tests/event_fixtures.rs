//! Contract sync-event-v1 — fixture-driven validator tests (Phase 1 freeze).
//!
//! The fixtures are the canonical cross-repo executable spec, byte-identical
//! with wannolot-infrastructure/tests/fixtures/sync-event-v1/. The manifest
//! carries the expected verdict per fixture; key SAFETY verdicts (`key_safe`)
//! are exercised by the Phase-3 apply pipeline, not here.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use tytus_progressive_sync::{validate_sync_event, EventValidationError};

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sync-event-v1")
}

fn load(name: &str) -> serde_json::Value {
    let raw = fs::read_to_string(fixture_dir().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn manifest() -> serde_json::Value {
    load("manifest.json")
}

#[test]
fn manifest_covers_every_fixture() {
    let on_disk: BTreeSet<String> = fs::read_dir(fixture_dir())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".json") && n != "manifest.json")
        .collect();
    let in_manifest: BTreeSet<String> = manifest()["verdicts"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    assert_eq!(on_disk, in_manifest, "fixture set drifted from manifest");
}

#[test]
fn fixture_verdicts() {
    let man = manifest();
    for (name, verdict) in man["verdicts"].as_object().unwrap() {
        let expected = verdict["schema_valid"].as_bool().unwrap();
        let got = validate_sync_event(&load(name));
        assert_eq!(
            got.is_ok(),
            expected,
            "{name}: got {:?}, expected schema_valid={expected}",
            got.as_ref().err()
        );
    }
}

#[test]
fn rejection_reasons_are_specific() {
    assert_eq!(
        validate_sync_event(&load("missing-content-ref.json")).unwrap_err(),
        EventValidationError::MissingField("content_ref")
    );
    assert_eq!(
        validate_sync_event(&load("seq-eventid-mismatch.json")).unwrap_err(),
        EventValidationError::EventIdSequenceMismatch
    );
}

#[test]
fn digest_consistency_is_enforced() {
    let mut doc = load("valid-pod-put.json");
    doc["content_ref"] = format!("_tytus-sync/blobs/sha256/{}", "0".repeat(64)).into();
    assert_eq!(
        validate_sync_event(&doc).unwrap_err(),
        EventValidationError::ContentRefDigestMismatch
    );
}

#[test]
fn etag_is_optional_metadata() {
    let mut doc = load("valid-pod-put.json");
    doc["object"]["etag"] = "whatever-not-identity".into();
    assert!(validate_sync_event(&doc).is_ok());
}

#[test]
fn sequence_must_be_positive_int() {
    for bad in [
        serde_json::json!(0),
        serde_json::json!(-1),
        serde_json::json!("42"),
        serde_json::json!(4.2),
        serde_json::json!(true),
        serde_json::Value::Null,
    ] {
        let mut doc = load("valid-pod-put.json");
        doc["sequence"] = bad.clone();
        assert!(validate_sync_event(&doc).is_err(), "sequence={bad}");
    }
}

#[test]
fn op_delete_parses_but_unknown_ops_reject() {
    let event = validate_sync_event(&load("delete-reserved.json")).unwrap();
    assert_eq!(event.op, "delete");
    let mut doc = load("valid-pod-put.json");
    doc["op"] = "rename".into();
    assert_eq!(validate_sync_event(&doc).unwrap_err(), EventValidationError::Op);
}

#[test]
fn typed_struct_roundtrips_valid_fixture() {
    let event = validate_sync_event(&load("valid-pod-put.json")).unwrap();
    assert_eq!(event.sequence, 42);
    assert!(event.event_id.starts_with("00000000000000000042-"));
    assert_eq!(event.object.sha256.strip_prefix("sha256:").unwrap().len(), 64);
    // reserialize -> revalidate: the typed struct never emits an invalid event
    let round = serde_json::to_value(&event).unwrap();
    assert!(validate_sync_event(&round).is_ok());
}

#[test]
fn unicode_key_survives_typed_roundtrip_nfc() {
    let event = validate_sync_event(&load("nfc-key.json")).unwrap();
    assert_eq!(event.key, "docs/r\u{00e9}sum\u{00e9}.md"); // é as single NFC codepoints
}

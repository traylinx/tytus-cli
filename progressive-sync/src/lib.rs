//! Progressive shared-folder sync — contract types and validation.
//!
//! Phase 1 (contract freeze): `SyncEventV1` and its validator, the
//! mirror-image of `validate_sync_event()` in the pod-side producer
//! (`wannolot-infrastructure/{nemoclaw,hermes}/tytus-shared-watch`). Both are
//! pinned to the shared fixture set under `tests/fixtures/sync-event-v1/`;
//! keep the fixture sets byte-identical across repos.
//!
//! The consumer treats every event as UNTRUSTED input: schema validation here,
//! key SAFETY (Unicode/Windows/traversal rules, TECH-SPEC §9) as a separate
//! layer that lands with the apply pipeline (Phase 3).

pub mod event;

pub use event::{validate_sync_event, EventValidationError, SyncEventV1, SYNC_EVENT_SCHEMA_VERSION};

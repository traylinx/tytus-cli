//! Library surface of `atomek-cli` — exists purely so integration tests
//! (and, down the line, FIX-2's disconnect test harness) can import helpers
//! that would otherwise be locked inside the binary crate.
//!
//! The binary itself (`src/main.rs`) does NOT depend on this lib target; it
//! declares the same modules via `mod` directives so the binary stays
//! self-contained and this lib can be extended without dragging the whole
//! CLI surface into integration tests.
//
// TODO: FIX-2 will likely want to reexport a disconnect-reap helper here too;
// keep this file narrow so conflicts stay minimal.

pub mod tunnel_reap;

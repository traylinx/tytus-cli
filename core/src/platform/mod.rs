//! Cross-platform runtime substrate for Tytus.
//!
//! Phase 2 rule: product code calls these modules instead of embedding
//! `/tmp/tytus`, Unix sockets, `osascript`, `kill(2)`, launchd/systemd, or
//! Windows-specific process/service behavior directly.

pub mod dialog;
pub mod firewall_policy;
pub mod ipc;
pub mod logging;
pub mod open;
pub mod paths;
pub mod process;
pub mod service;

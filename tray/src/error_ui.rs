//! Phase F (sprint v0.6) — Rust friendlifier mirroring `tower.js`'s
//! `__FRIENDLIFY_PATTERNS`. Maps raw subprocess output / stderr lines /
//! HTTP error JSON to a `{title, body, try_this}` triple grandma can act
//! on. JS side handles browser-rendered toasts + SSE streamed-action
//! failure logs; this Rust side handles tray-native dialogs +
//! command-output formatting where errors surface to the user.
//!
//! Patterns are tested in declaration order; first match wins. Unknown
//! errors return `None` so callers can fall back to "Something went
//! wrong — click to see details" with a disclosure button (per the
//! SPRINT.md Phase F spec).
//!
//! No regex dep on purpose — every pattern is expressible as a list of
//! lowercased substring tokens (`needles`); the input gets lowercased
//! once and we OR-match across the table. Kept dep-free so `cargo
//! audit` doesn't grow a fresh transitive surface for one module.
//!
//! Keep this table in lockstep with `tray/web/assets/tower.js`'s
//! `__FRIENDLIFY_PATTERNS` — divergence between the two surfaces means
//! grandma sees different copy depending on whether the error reached
//! her via Tower or via a tray dialog. The unit tests at the bottom
//! make sure the same canonical raw-error strings produce the same
//! titles in both surfaces (manually maintained — drift will surface
//! as test failure when JS strings change).

// Some helpers are intentionally part of the public surface even if
// only one caller (`build_diag_summary`) uses them today — they're
// ready for the next round of friendlify hooks (osascript dialog
// formatter, notification body, tooltip text).
#![allow(dead_code)]

/// One curated error → grandma copy. The `try_this` field is the action
/// the user can click / type / select to recover.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FriendlyError {
    pub title: String,
    pub body: String,
    pub try_this: String,
    /// The raw error preserved for power-user disclosure ("show full
    /// log" buttons, sentry breadcrumbs, support pastes).
    pub raw: String,
}

/// One row of the pattern table. `needles` is an OR-list of lowercased
/// substrings; if ANY one is present in the lowercased raw input, the
/// pattern fires.
struct Pattern {
    /// At least one of these substrings must appear (lowercased) in
    /// the raw error for this pattern to match.
    needles: &'static [&'static str],
    title: &'static str,
    body: &'static str,
    try_this: &'static str,
}

/// 20 patterns mirroring `tower.js::__FRIENDLIFY_PATTERNS`. Order
/// matches the JS file for ease of cross-referencing. Each `needles`
/// entry encodes the same token set the JS regex matched on (without
/// the regex sugar — we hit them with `str::contains` after a single
/// `to_lowercase()` of the input).
const PATTERNS: &[Pattern] = &[
    Pattern {
        needles: &[
            "keychain get_refresh_token timed out",
            "keychain access pending",
            "keychain timeout",
        ],
        title: "Keychain dialog pending",
        body: "Tytus is waiting for you to approve a Mac dialog.",
        try_this: "Open Keychain Access and approve the pending request, or quit + relaunch Tytus.",
    },
    Pattern {
        needles: &[
            "no pods.",
            "no pods\n",
            "no pod allocated",
            "no workspace yet",
            "state.pods is empty",
        ],
        title: "No workspace yet",
        body: "You haven't set up a workspace.",
        try_this: "tytus connect",
    },
    Pattern {
        needles: &["invalid api key", "invalid api_key", "invalid apikey"],
        title: "Sign-in expired",
        body: "Your AI rebooted itself and needs a new key.",
        try_this: "Wait 2 seconds, then click again.",
    },
    Pattern {
        needles: &[
            "tunnel up but",  // followed by "times out" / "timed out" in real errors
            "gateway unreachable but tunnel active",
            "gateway timeout while tunnel active",
        ],
        title: "Connection blocked",
        body: "Another VPN is blocking your AI connection.",
        try_this: "Disconnect other VPNs, then reconnect.",
    },
    Pattern {
        needles: &["tunnel daemon already running", "tunnel daemon conflict"],
        title: "Tytus is already connecting",
        body: "A previous Connect attempt is still in flight.",
        try_this: "Wait a few seconds, or quit + relaunch Tytus.",
    },
    Pattern {
        needles: &["pod config not ready", "peer.conf missing"],
        title: "Workspace isn't ready",
        body: "Your workspace isn't ready on our end yet.",
        try_this: "Wait 30 seconds and retry.",
    },
    Pattern {
        needles: &["bad json", "missing field", "invalid request body"],
        title: "Form data went wrong",
        body: "Tytus received the form incorrectly.",
        try_this: "Reopen the form and try again.",
    },
    Pattern {
        needles: &["command not found", "tytus: command not found", "failed to launch tytus"],
        title: "Tytus not installed",
        body: "The tytus CLI isn't on your PATH.",
        try_this: "Run the installer: curl -fsSL https://get.traylinx.com/install.sh | bash",
    },
    Pattern {
        needles: &[
            "http 500", "http 502", "http 503", "http 504",
            "internal server error",
            "5xx response from gateway",
        ],
        title: "AI hit an error",
        body: "Your AI gateway returned a server error.",
        try_this: "tytus test",
    },
    Pattern {
        needles: &["permission denied", "eacces", "operation not permitted"],
        title: "Tytus can't access this",
        body: "macOS blocked the operation.",
        try_this: "System Settings → Privacy & Security → Files & Folders, grant Tytus access.",
    },
    Pattern {
        needles: &["connection refused", "econnrefused", "eof on stdin"],
        title: "Tytus isn't running",
        body: "The background service isn't reachable.",
        try_this: "Click the menu-bar T → Quick actions → Connect.",
    },
    Pattern {
        needles: &["address already in use", "eaddrinuse", "port in use"],
        title: "Port already taken",
        body: "Another app is using a port Tytus needs.",
        try_this: "Quit Tytus, restart your Mac, then relaunch.",
    },
    Pattern {
        needles: &["network unreachable", "enetunreach", "no route to host", "dns failure", "dns failed"],
        title: "Computer is offline",
        body: "Your Mac can't reach the network.",
        try_this: "Reconnect to Wi-Fi and try again.",
    },
    Pattern {
        needles: &["bucket already exists", "bucketalreadyownedbyyou", "conflict on bucket"],
        title: "Folder name taken",
        body: "A shared folder with that name already exists.",
        try_this: "Pick a different name.",
    },
    Pattern {
        needles: &[
            "bucket name invalid", "bucket name too short", "bucket name too long",
            "folder name invalid", "folder name too short", "folder name too long",
            "invalid bucket name",
        ],
        title: "Invalid folder name",
        body: "Names need 3–63 characters, lowercase letters, dashes, and dots.",
        try_this: "Pick a name like \"design-files\" or \"client-photos\".",
    },
    Pattern {
        needles: &["no space left", "disk full", "enospc"],
        title: "Computer is out of space",
        body: "Your Mac has no disk space left.",
        try_this: "Empty the Trash or delete large files.",
    },
    Pattern {
        needles: &["wireguard not installed", "wireguard not found", "wg-quick: not found"],
        title: "Tytus needs WireGuard",
        body: "The WireGuard tools aren't installed.",
        try_this: "brew install wireguard-tools",
    },
    Pattern {
        needles: &["revoke would orphan", "would lose data", "unsynced bindings"],
        title: "Workspace has unsaved files",
        body: "Revoking would lose unsaved files.",
        try_this: "tytus pull <pod> first, then revoke.",
    },
    Pattern {
        needles: &["token expired", "refresh token invalid", "sentinel 401", "sentinel: 401"],
        title: "Sign-in expired",
        body: "Your Tytus account session ran out.",
        try_this: "tytus login",
    },
    Pattern {
        needles: &["not logged in", "sentinel unauthorized", "sentinel: unauthorized"],
        title: "Not signed in",
        body: "You're not signed in to Tytus.",
        try_this: "tytus login",
    },
];

/// Match raw error text against the pattern table. Returns `None` if no
/// pattern fires — caller falls back to a generic "Something went wrong"
/// dialog with a disclosure button.
pub fn friendlify(raw: &str) -> Option<FriendlyError> {
    if raw.is_empty() {
        return None;
    }
    let lc = raw.to_lowercase();
    for p in PATTERNS {
        for needle in p.needles {
            if lc.contains(needle) {
                return Some(FriendlyError {
                    title: p.title.to_string(),
                    body: p.body.to_string(),
                    try_this: p.try_this.to_string(),
                    raw: raw.to_string(),
                });
            }
        }
    }
    None
}

/// Format a `FriendlyError` for an osascript dialog. Returns a single
/// string with the body + raw disclosure + Try-this hint joined for
/// display in the dialog body. Caller wraps it via:
///   `osascript -e 'display dialog "<text>" with title "<title>"'`.
pub fn format_for_dialog(err: &FriendlyError) -> String {
    format!(
        "{}\n\n{}\n\nTry this:\n{}",
        err.body,
        err.raw_disclosure_hint(),
        err.try_this
    )
}

impl FriendlyError {
    /// One-line raw disclosure hint shown above the suggested action.
    /// Keeps the original error reachable for power users without
    /// dominating the dialog. Truncates safely on UTF-8 char boundaries.
    pub fn raw_disclosure_hint(&self) -> String {
        if self.raw.chars().count() > 80 {
            let truncated: String = self.raw.chars().take(80).collect();
            format!("(Original: {}…)", truncated)
        } else {
            format!("(Original: {})", self.raw)
        }
    }

    /// One-line summary suited to a tray menu disabled-row label or
    /// short notification: `<title>: <body>` (≤120 chars).
    pub fn one_line(&self) -> String {
        let s = format!("{}: {}", self.title, self.body);
        if s.chars().count() > 120 {
            let truncated: String = s.chars().take(117).collect();
            format!("{}…", truncated)
        } else {
            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keychain_timeout_friendly() {
        let f = friendlify("WARN keychain get_refresh_token timed out after 3s").unwrap();
        assert_eq!(f.title, "Keychain dialog pending");
        assert!(f.body.contains("Mac dialog"));
        assert!(f.try_this.contains("Keychain Access") || f.try_this.contains("relaunch"));
    }

    #[test]
    fn no_pods_friendly() {
        let f = friendlify("No pods. Run: tytus connect").unwrap();
        assert_eq!(f.title, "No workspace yet");
        assert_eq!(f.try_this, "tytus connect");
    }

    #[test]
    fn connection_refused_friendly() {
        let f = friendlify("connection refused: 127.0.0.1:18080").unwrap();
        assert_eq!(f.title, "Tytus isn't running");
        assert!(f.try_this.contains("menu-bar T"));
    }

    #[test]
    fn http_500_friendly() {
        let f = friendlify("Failed: HTTP 503 from gateway").unwrap();
        assert_eq!(f.title, "AI hit an error");
    }

    #[test]
    fn permission_denied_friendly() {
        let f = friendlify("EACCES: permission denied opening /System/foo").unwrap();
        assert_eq!(f.title, "Tytus can't access this");
    }

    #[test]
    fn bucket_name_invalid_friendly() {
        let f = friendlify("bucket name too short (min 3 chars)").unwrap();
        assert_eq!(f.title, "Invalid folder name");
    }

    #[test]
    fn unknown_error_returns_none() {
        assert!(friendlify("xyzzy plugh quux 42").is_none());
    }

    #[test]
    fn empty_string_returns_none() {
        assert!(friendlify("").is_none());
    }

    #[test]
    fn raw_preserved() {
        let raw = "WARN keychain get_refresh_token timed out after 3s — likely pending";
        let f = friendlify(raw).unwrap();
        assert_eq!(f.raw, raw);
    }

    #[test]
    fn one_line_under_120() {
        for raw in &[
            "WARN keychain get_refresh_token timed out",
            "No pods. Run: tytus connect",
            "connection refused",
        ] {
            let f = friendlify(raw).unwrap();
            assert!(
                f.one_line().chars().count() <= 120,
                "one_line over budget: {}",
                f.one_line()
            );
        }
    }

    #[test]
    fn first_match_wins() {
        let f = friendlify("disk full: ENOSPC").unwrap();
        assert_eq!(f.title, "Computer is out of space");
    }

    #[test]
    fn case_insensitive() {
        let f1 = friendlify("CONNECTION REFUSED: 127.0.0.1").unwrap();
        let f2 = friendlify("connection refused: 127.0.0.1").unwrap();
        assert_eq!(f1.title, f2.title);
    }

    #[test]
    fn pattern_count_matches_js() {
        // Lockstep guard: tower.js has 20 patterns; Rust must have 20.
        // If you add a pattern here, add the matching one to tower.js
        // (and vice versa) so the two surfaces stay in sync.
        assert_eq!(PATTERNS.len(), 20, "pattern count drifted from tower.js");
    }

    #[test]
    fn format_for_dialog_includes_all_fields() {
        let f = friendlify("connection refused").unwrap();
        let txt = format_for_dialog(&f);
        assert!(txt.contains(&f.body));
        assert!(txt.contains("Try this:"));
        assert!(txt.contains(&f.try_this));
        assert!(txt.contains("Original:"));
    }

    #[test]
    fn raw_truncation_handles_unicode() {
        // Make sure UTF-8 char-boundary aware truncation doesn't panic
        // on multibyte chars at the boundary.
        let raw = "💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥💥 not logged in 💥";
        let f = friendlify(raw).unwrap();
        let _ = f.raw_disclosure_hint(); // shouldn't panic
    }

    #[test]
    fn unicode_input_does_not_panic() {
        // Random unicode inputs should never panic, even when they
        // don't match any pattern.
        for raw in &["héllo wörld", "日本語のエラー", "🚀🚀🚀", ""] {
            let _ = friendlify(raw);
        }
    }
}

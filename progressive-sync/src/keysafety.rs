//! Key/filename safety (TECH-SPEC §9) — the consumer-side authoritative gate.
//!
//! Events are untrusted input: a malicious or buggy producer must not be able
//! to write outside the binding directory, smuggle a reserved-namespace
//! target, wedge a future Windows consumer, or overwrite through a Unicode or
//! case alias. Violations classify per-event (`path_invalid`) and never block
//! later valid events.

use unicode_normalization::{is_nfc, UnicodeNormalization};

/// Maximum bytes per path segment (macOS/most filesystems: 255) and for the
/// whole relative key. Decorated conflict names re-check against these.
pub const MAX_SEGMENT_BYTES: usize = 255;
pub const MAX_KEY_BYTES: usize = 1024;

pub const RESERVED_NAMESPACE: &str = "_tytus-sync";

const TEMP_SUFFIXES: [&str; 5] = [".part", ".tmp", ".partial", ".swp", "~"];
const WINDOWS_ILLEGAL: [char; 7] = ['<', '>', ':', '"', '|', '?', '*'];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeySafetyError {
    AbsolutePath,
    ParentTraversal,
    EmptySegment,
    DrivePrefix,
    ControlChar,
    Backslash,
    ReservedNamespace,
    TempSuffix,
    NotNfc,
    WindowsReservedName(String),
    WindowsTrailingDotOrSpace,
    WindowsIllegalChar(char),
    TooLong,
}

impl std::fmt::Display for KeySafetyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AbsolutePath => write!(f, "absolute path"),
            Self::ParentTraversal => write!(f, "parent traversal segment"),
            Self::EmptySegment => write!(f, "empty segment"),
            Self::DrivePrefix => write!(f, "windows drive prefix"),
            Self::ControlChar => write!(f, "NUL/control character"),
            Self::Backslash => write!(f, "backslash in key"),
            Self::ReservedNamespace => write!(f, "reserved _tytus-sync namespace"),
            Self::TempSuffix => write!(f, "transient temp suffix"),
            Self::NotNfc => write!(f, "key not NFC-normalized"),
            Self::WindowsReservedName(n) => write!(f, "windows reserved name {n}"),
            Self::WindowsTrailingDotOrSpace => write!(f, "trailing dot/space segment"),
            Self::WindowsIllegalChar(c) => write!(f, "illegal character {c:?}"),
            Self::TooLong => write!(f, "key or segment over length policy"),
        }
    }
}

impl std::error::Error for KeySafetyError {}

fn windows_reserved(segment: &str) -> Option<String> {
    // Reserved DEVICE names, extension-insensitive: `CON`, `con.txt`, `LPT3.log`.
    let base = segment.split('.').next().unwrap_or("");
    let upper = base.to_ascii_uppercase();
    let reserved = matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ((upper.starts_with("COM") || upper.starts_with("LPT"))
            && upper.len() == 4
            && upper.as_bytes()[3].is_ascii_digit()
            && upper.as_bytes()[3] != b'0');
    reserved.then_some(upper)
}

fn temp_suffixed(name: &str) -> bool {
    TEMP_SUFFIXES.iter().any(|s| name.ends_with(s))
        || name.starts_with("~$")
        || name == "4913"
        || name.starts_with(".goutputstream-")
        || name.contains("___jb_tmp___")
}

/// Validate an event `key` as a safe, relative, NFC path. Returns the key
/// unchanged on success — the consumer never rewrites keys (a producer that
/// mints non-NFC keys is out of contract; rewriting would desynchronize the
/// ledger from the bucket).
pub fn validate_key(key: &str) -> Result<&str, KeySafetyError> {
    if key.is_empty() || key.starts_with('/') {
        return Err(KeySafetyError::AbsolutePath);
    }
    if key.contains('\\') {
        return Err(KeySafetyError::Backslash);
    }
    if key.chars().any(|c| c.is_control()) {
        return Err(KeySafetyError::ControlChar);
    }
    if !is_nfc(key) {
        return Err(KeySafetyError::NotNfc);
    }
    if key.len() > MAX_KEY_BYTES {
        return Err(KeySafetyError::TooLong);
    }
    let bytes = key.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return Err(KeySafetyError::DrivePrefix);
    }
    let segments: Vec<&str> = key.split('/').collect();
    for (index, segment) in segments.iter().enumerate() {
        if segment.is_empty() || *segment == "." {
            return Err(KeySafetyError::EmptySegment);
        }
        if *segment == ".." {
            return Err(KeySafetyError::ParentTraversal);
        }
        if *segment == RESERVED_NAMESPACE {
            return Err(KeySafetyError::ReservedNamespace);
        }
        if segment.len() > MAX_SEGMENT_BYTES {
            return Err(KeySafetyError::TooLong);
        }
        if let Some(name) = windows_reserved(segment) {
            return Err(KeySafetyError::WindowsReservedName(name));
        }
        if segment.ends_with('.') || segment.ends_with(' ') {
            return Err(KeySafetyError::WindowsTrailingDotOrSpace);
        }
        if let Some(c) = segment.chars().find(|c| WINDOWS_ILLEGAL.contains(c)) {
            return Err(KeySafetyError::WindowsIllegalChar(c));
        }
        let is_last = index == segments.len() - 1;
        if is_last && temp_suffixed(segment) {
            return Err(KeySafetyError::TempSuffix);
        }
    }
    Ok(key)
}

/// NFC form of a string, for collision comparisons (never for rewriting keys).
pub fn nfc(s: &str) -> String {
    s.nfc().collect()
}

/// True when two keys collide once case and Unicode-normalization differences
/// are folded away — the macOS-default-volume overwrite hazard.
pub fn keys_collide(a: &str, b: &str) -> bool {
    a != b && nfc(a).to_lowercase() == nfc(b).to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ordinary_relative_keys() {
        for key in ["notes/report.md", "a/b/c.txt", "docs/r\u{00e9}sum\u{00e9}.md", "no-ext"] {
            assert!(validate_key(key).is_ok(), "{key}");
        }
    }

    #[test]
    fn rejects_traversal_and_absolute() {
        assert_eq!(validate_key("/etc/passwd"), Err(KeySafetyError::AbsolutePath));
        assert_eq!(validate_key("a/../b"), Err(KeySafetyError::ParentTraversal));
        assert_eq!(validate_key("a//b"), Err(KeySafetyError::EmptySegment));
        assert_eq!(validate_key("a/./b"), Err(KeySafetyError::EmptySegment));
        assert_eq!(validate_key("C:\\x"), Err(KeySafetyError::Backslash));
        assert_eq!(validate_key("C:x/y"), Err(KeySafetyError::DrivePrefix));
        assert_eq!(validate_key("a\u{0000}b"), Err(KeySafetyError::ControlChar));
    }

    #[test]
    fn rejects_reserved_namespace_at_any_depth() {
        assert_eq!(validate_key("_tytus-sync/events/x"), Err(KeySafetyError::ReservedNamespace));
        assert_eq!(validate_key("sub/_tytus-sync/blobs/y"), Err(KeySafetyError::ReservedNamespace));
        assert!(validate_key("_tytus-sync.txt").is_ok(), "lookalike file stays valid");
    }

    #[test]
    fn rejects_windows_hazards_at_key_level() {
        assert!(matches!(validate_key("reports/AUX.txt"), Err(KeySafetyError::WindowsReservedName(_))));
        assert!(matches!(validate_key("com1.log"), Err(KeySafetyError::WindowsReservedName(_))));
        assert!(validate_key("com0.log").is_ok());
        assert!(validate_key("common.log").is_ok());
        assert_eq!(validate_key("bad./x"), Err(KeySafetyError::WindowsTrailingDotOrSpace));
        assert_eq!(validate_key("bad "), Err(KeySafetyError::WindowsTrailingDotOrSpace));
        assert_eq!(validate_key("a<b.txt"), Err(KeySafetyError::WindowsIllegalChar('<')));
    }

    #[test]
    fn rejects_temp_suffixes_and_nfd() {
        assert_eq!(validate_key("doc.md.part"), Err(KeySafetyError::TempSuffix));
        assert_eq!(validate_key("x/~$doc.docx"), Err(KeySafetyError::TempSuffix));
        assert_eq!(validate_key("x/4913"), Err(KeySafetyError::TempSuffix));
        // NFD é (e + combining acute) must be rejected, NFC é accepted.
        assert_eq!(validate_key("docs/re\u{0301}sume.md"), Err(KeySafetyError::NotNfc));
    }

    #[test]
    fn collision_folding() {
        assert!(keys_collide("Readme.md", "readme.md"));
        assert!(keys_collide("caf\u{00e9}.md", "cafe\u{0301}.md"));
        assert!(!keys_collide("a.md", "b.md"));
        assert!(!keys_collide("same.md", "same.md"));
    }

    #[test]
    fn length_policy() {
        let long_segment = "x".repeat(256);
        assert_eq!(validate_key(&long_segment), Err(KeySafetyError::TooLong));
        let long_key = format!("{}/{}", "d".repeat(200), "x".repeat(200)).repeat(4);
        assert_eq!(validate_key(&long_key), Err(KeySafetyError::TooLong));
    }
}

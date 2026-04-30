use atomek_cli::state::{CliState, PodEntry};
use std::sync::{Mutex, OnceLock};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn with_state_path<T>(f: impl FnOnce(&std::path::Path) -> T) -> T {
    let _guard = env_lock().lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    std::env::set_var("TYTUS_STATE_PATH", &path);
    let out = f(&path);
    std::env::remove_var("TYTUS_STATE_PATH");
    out
}

#[test]
fn v1_file_loads_as_v2_shape() {
    let raw = include_str!("fixtures/state_v1.json");
    let state = CliState::from_legacy_v1(raw);
    assert_eq!(state.schema_version, 2);
    assert_eq!(state.active_email.as_deref(), Some("sebastian@example.com"));
    assert_eq!(state.accounts.len(), 1);
    assert_eq!(
        state.active_account().unwrap().email,
        "sebastian@example.com"
    );
    assert_eq!(
        state.active_account().unwrap().pods[0]
            .stable_user_key
            .as_deref(),
        Some("sk-tytus-user-test")
    );
}

#[test]
fn save_writes_v2_and_legacy_snapshot_without_refresh_token() {
    with_state_path(|path| {
        let mut state = CliState::from_legacy_v1(include_str!("fixtures/state_v1.json"));
        state.refresh_token = Some("must-not-hit-disk".into());
        state.save_critical().unwrap();
        let saved = std::fs::read_to_string(path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&saved).unwrap();
        assert_eq!(json["schema_version"], 2);
        assert_eq!(json["active_email"], "sebastian@example.com");
        assert!(json["accounts"].is_array());
        assert_eq!(json["email"], json["accounts"][0]["email"]);
        assert_eq!(json["pods"], json["accounts"][0]["pods"]);
        assert!(json.get("refresh_token").is_none());
    });
}

#[test]
fn v2_roundtrip_is_stable() {
    with_state_path(|path| {
        let state = CliState::from_legacy_v1(include_str!("fixtures/state_v1.json"));
        state.save_critical().unwrap();
        let first = std::fs::read_to_string(path).unwrap();
        let loaded = CliState::load_file_only();
        loaded.save_critical().unwrap();
        let second = std::fs::read_to_string(path).unwrap();
        assert_eq!(first, second);
    });
}

#[test]
fn mode_0600_preserved_on_overwrite() {
    with_state_path(|path| {
        let mut state = CliState::from_legacy_v1(include_str!("fixtures/state_v1.json"));
        state.save_critical().unwrap();
        state.tier = Some("creator".into());
        state.save_critical().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
    });
}

#[test]
fn account_helpers_find_active_and_named_accounts() {
    let state = CliState::from_legacy_v1(include_str!("fixtures/state_v1.json"));
    assert!(state.active_account().is_some());
    assert!(state.find_account("sebastian@example.com").is_some());
    assert!(state.find_account("missing@example.com").is_none());
}

#[test]
fn active_account_defaults_sane_on_fresh_file() {
    let state = CliState::default();
    assert!(state.active_account().is_none());
    assert!(state.active_email.is_none());
}

#[test]
fn sync_active_snapshot_keeps_top_level_legacy_fields_in_sync() {
    let mut state = CliState::from_legacy_v1(include_str!("fixtures/state_v1.json"));
    state.accounts[0].pods = vec![PodEntry {
        pod_id: "09".into(),
        droplet_id: "drop-9".into(),
        ..PodEntry::default()
    }];
    state.sync_active_snapshot();
    assert_eq!(
        state.email.as_deref(),
        Some(state.active_account().unwrap().email.as_str())
    );
    assert_eq!(state.pods, state.active_account().unwrap().pods);
}

#[test]
fn state_lock_lives_next_to_state_json() {
    with_state_path(|path| {
        assert_eq!(
            CliState::state_lock_path(),
            path.parent().unwrap().join("state.lock")
        );
    });
}

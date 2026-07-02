//! G3 live canary — the REAL rclone subprocess against the synthetic scratch
//! bucket over the tunnel (sprint gate; also reused by the P6 ladder).
//!
//! Ignored by default: requires the tunnel up, the `[garagetytus]` rclone
//! remote, and staged events. Run explicitly:
//!
//! ```bash
//! TYTUS_PROGRESSIVE_LIVE_CANARY=1 cargo test -p tytus-progressive-sync \
//!     --test live_canary -- --ignored --nocapture
//! ```

use std::fs;

use tytus_progressive_sync::apply::BindingConsumer;
use tytus_progressive_sync::s3::RcloneS3;
use tytus_progressive_sync::state::{BindingIdentity, StateStore};

const BUCKET: &str = "tytus-progressive-canary";
const BINDING: &str = "sf_g3canary";
const ROUTE: &str = "r0g3";

#[test]
#[ignore = "live: needs tunnel + staged events in tytus-progressive-canary"]
fn consumes_staged_events_end_to_end() {
    if std::env::var("TYTUS_PROGRESSIVE_LIVE_CANARY").as_deref() != Ok("1") {
        eprintln!("skipped: set TYTUS_PROGRESSIVE_LIVE_CANARY=1");
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let local_root = tmp.path().join("folder");
    fs::create_dir_all(&local_root).unwrap();
    let identity = BindingIdentity {
        binding_id: BINDING.into(),
        bucket: BUCKET.into(),
        local_path: local_root.to_string_lossy().into_owned(),
        alias: Some("g3-canary".into()),
    };
    let store = StateStore::open(&tmp.path().join("state"), &identity).unwrap();
    let (mut state, pending) = store.load(&identity, "g3").unwrap();
    let conf = std::env::var("HOME").ok().map(|h| format!("{h}/.config/rclone/rclone.conf"));
    let s3 = RcloneS3::new("garagetytus", BUCKET, conf);
    let mut consumer = BindingConsumer::new(&s3, &store, &mut state, &identity, vec![ROUTE.into()]);
    consumer.recover(pending);

    let started = std::time::Instant::now();
    let mut applied_total = 0;
    let mut conflicts_total = 0;
    while started.elapsed().as_secs() < 120 {
        let outcome = consumer.poll_once();
        applied_total += outcome.applied;
        conflicts_total += outcome.conflicts;
        if applied_total >= 2 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
    eprintln!("applied={applied_total} conflicts={conflicts_total} in {:?}", started.elapsed());
    assert!(applied_total >= 2, "expected the two staged synthetic events to apply");
    let hello = local_root.join("g3/hello.md");
    assert!(hello.exists(), "g3/hello.md must land");
    let body = fs::read_to_string(&hello).unwrap();
    assert!(body.contains("g3 canary"), "{body}");
    // cursor durability across a fresh load
    drop(consumer);
    let (state2, _) = store.load(&identity, "g3").unwrap();
    assert!(state2.routes.get(ROUTE).map(|r| r.last_applied_sequence >= 2).unwrap_or(false));
}

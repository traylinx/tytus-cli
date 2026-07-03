//! Progressive shared-folder sync — tray-side consumer orchestration
//! (sprint tytus-garagetytus-progressive-sync-2026-06-30, Phase 3).
//!
//! Runs inside the resident tray daemon: a background thread polls each
//! enabled binding's authorized route prefixes with jitter/backoff and drives
//! the `tytus-progressive-sync` apply pipeline. Enable/disable implements the
//! NORMATIVE order (unload the binding's bisync LaunchAgent → drain its
//! rclone lock → set sidecar flags → polls start): consumer applies must
//! never race a live bisync run. v1 is pull-only.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tytus_progressive_sync::apply::{file_sha256, BindingConsumer, DEFAULT_CYCLE_BUDGET};
use tytus_progressive_sync::s3::RcloneS3;
use tytus_progressive_sync::state::{BindingIdentity, JournalRecord, StateStore};

pub const POLL_INTERVAL_SECS: u64 = 20;
pub const BACKOFF_MAX_SECS: u64 = 300;
pub const LOCK_DRAIN_TIMEOUT_SECS: u64 = 120;
pub const LOCAL_EDIT_SCAN_INTERVAL_SECS: u64 = 3600;
/// Report-only auto-reconcile cadence (sprint P6 deferred item): every 24 h
/// per enabled binding, plus a 1 h retry while any route is gap-halted
/// (`repair_required`) — the adjudicated report is what durably clears it.
pub const RECONCILE_INTERVAL_SECS: u64 = 86_400;
pub const RECONCILE_REPAIR_RETRY_SECS: u64 = 3_600;
/// First run after tray start is delayed so polls settle first.
pub const RECONCILE_STARTUP_DELAY_SECS: u64 = 600;
const RCLONE_REMOTE: &str = "garagetytus";
/// Absolute path — the tray runs under launchd (no /usr/local/bin in PATH).
const GARAGETYTUS_BIN: &str = "/usr/local/bin/garagetytus";

/// Global kill switch (TECH-SPEC §12): `TYTUS_PROGRESSIVE_SHARED_SYNC=0`
/// pauses every consumer poll without touching any state.
pub fn kill_switch_engaged() -> bool {
    std::env::var("TYTUS_PROGRESSIVE_SHARED_SYNC").map(|v| v == "0").unwrap_or(false)
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn bindings_cache_dir() -> Option<PathBuf> {
    home().map(|h| h.join(".cache").join("garagetytus").join("bisync"))
}

pub fn state_root() -> Option<PathBuf> {
    home().map(|h| h.join(".local").join("state").join("garagetytus").join("progressive-sync"))
}

fn rclone_conf_path() -> Option<String> {
    home().map(|h| h.join(".config").join("rclone").join("rclone.conf").to_string_lossy().into_owned())
}

/// One enabled binding, resolved from its sidecar. Route authority is the
/// LOCAL sidecar only (targets[].route_id / routes_provisioned).
#[derive(Debug, Clone)]
pub struct EnabledBinding {
    pub sidecar_path: PathBuf,
    pub binding_id: String,
    pub bucket: String,
    pub local_path: String,
    pub alias: Option<String>,
    pub routes: Vec<String>,
    pub plist_label: Option<String>,
    pub workdir: Option<PathBuf>,
    /// The binding id producers emit under (`grant.folder_id` from the
    /// Provider registry). It differs from the local sidecar `folder_id` on
    /// every Provider-provisioned binding. None = not discovered yet; the
    /// scheduler re-discovers lazily and persists the answer to the sidecar.
    pub remote_binding_id: Option<String>,
}

fn parse_binding(path: &Path, json: &serde_json::Value) -> Option<EnabledBinding> {
    let binding_id = json.get("folder_id")?.as_str()?.to_string();
    let bucket = json.get("bucket")?.as_str()?.to_string();
    let local_path = json.get("local_path")?.as_str()?.trim_end_matches('/').to_string();
    let mut routes: Vec<String> = json
        .get("targets")
        .and_then(|t| t.as_array())
        .map(|targets| {
            targets
                .iter()
                .filter(|t| t.get("enabled").and_then(|e| e.as_bool()).unwrap_or(true))
                .filter_map(|t| t.get("route_id").and_then(|r| r.as_str()))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    if routes.is_empty() {
        routes = json
            .get("routes_provisioned")
            .and_then(|r| r.as_array())
            .map(|rows| rows.iter().filter_map(|v| v.as_str()).map(str::to_string).collect())
            .unwrap_or_default();
    }
    routes.sort();
    routes.dedup();
    Some(EnabledBinding {
        sidecar_path: path.to_path_buf(),
        binding_id,
        bucket,
        local_path,
        alias: json.get("alias").and_then(|a| a.as_str()).map(str::to_string),
        routes,
        plist_label: json.get("plist_label").and_then(|p| p.as_str()).map(str::to_string),
        workdir: json.get("workdir").and_then(|w| w.as_str()).map(PathBuf::from),
        remote_binding_id: json
            .get("progressive")
            .and_then(|p| p.get("remote_binding_id"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
    })
}

// ── remote binding-id discovery ─────────────────────────────────────────────
//
// Producers emit under `_tytus-sync/events/<grant.folder_id>/<route_id>/`.
// `grant.folder_id` comes from the Provider registry and does NOT match the
// Mac-local sidecar `folder_id`, so the consumer must discover the remote
// namespace instead of assuming its own id. Route ids are globally unique
// (Scalesys invariant), which makes the match unambiguous unless the folder
// was re-registered; ambiguity fails closed.

/// List immediate subdirectory names under `bucket/prefix` (no trailing `/`).
fn list_remote_dirs(bucket: &str, prefix: &str) -> Result<Vec<String>, String> {
    // Absolute path: the tray runs under launchd, whose PATH does not
    // include /usr/local/bin (same convention as RcloneS3::new).
    let mut cmd = Command::new("/usr/local/bin/rclone");
    if let Some(conf) = rclone_conf_path() {
        cmd.arg("--config").arg(conf);
    }
    cmd.arg("--contimeout").arg("10s");
    cmd.arg("--timeout").arg("30s");
    cmd.arg("--retries").arg("1");
    cmd.arg("--low-level-retries").arg("2");
    cmd.arg("lsf").arg("--dirs-only").arg("--max-depth").arg("1");
    cmd.arg(format!("{RCLONE_REMOTE}:{bucket}/{prefix}"));
    let out = cmd.output().map_err(|e| format!("rclone spawn: {e}"))?;
    if !out.status.success() {
        // rclone exits non-zero on a missing directory too; treat a clean
        // "directory not found" the same as empty (producer never emitted).
        let stderr = String::from_utf8_lossy(&out.stderr);
        if stderr.contains("directory not found") {
            return Ok(Vec::new());
        }
        return Err(format!("rclone lsf failed: {}", stderr.trim()));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.trim().trim_end_matches('/').to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

/// Pure matcher: candidates are `(binding_dir, route_dirs)`; a candidate
/// matches when it contains at least one of the binding's authorized routes.
fn match_remote_candidates(
    candidates: &[(String, Vec<String>)],
    routes: &[String],
) -> Result<Option<String>, String> {
    let matches: Vec<&String> = candidates
        .iter()
        .filter(|(_, route_dirs)| route_dirs.iter().any(|r| routes.iter().any(|x| x == r)))
        .map(|(id, _)| id)
        .collect();
    match matches.len() {
        0 => Ok(None),
        1 => Ok(Some(matches[0].clone())),
        _ => Err(format!(
            "remote binding id is ambiguous: {} event namespaces claim this binding's routes",
            matches.len()
        )),
    }
}

/// Resolve the remote binding id from the events namespace. `Ok(None)` means
/// no producer has emitted yet (harmless — retry on later scheduler passes).
fn discover_remote_binding_id(bucket: &str, routes: &[String]) -> Result<Option<String>, String> {
    let mut candidates = Vec::new();
    for dir in list_remote_dirs(bucket, "_tytus-sync/events/")? {
        let route_dirs = list_remote_dirs(bucket, &format!("_tytus-sync/events/{dir}/"))?;
        candidates.push((dir, route_dirs));
    }
    match_remote_candidates(&candidates, routes)
}

/// Persist a discovered remote binding id into the sidecar's progressive
/// block so later passes (and other readers) skip discovery.
fn persist_remote_binding_id(sidecar_path: &Path, remote_id: &str) -> Result<(), String> {
    let mut json = read_sidecar(sidecar_path).ok_or("sidecar unreadable")?;
    let Some(progressive) = json.get_mut("progressive").and_then(|p| p.as_object_mut()) else {
        return Err("sidecar has no progressive block".into());
    };
    progressive.insert("remote_binding_id".into(), serde_json::json!(remote_id));
    write_sidecar_atomic(sidecar_path, &json).map_err(|e| format!("sidecar write: {e}"))
}

fn read_sidecar(path: &Path) -> Option<serde_json::Value> {
    serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

fn consumer_enabled(json: &serde_json::Value) -> bool {
    json.get("progressive")
        .and_then(|p| p.get("consumer_enabled"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// All sidecars, with their parsed binding and enablement.
pub fn discover() -> Vec<(EnabledBinding, bool)> {
    let Some(dir) = bindings_cache_dir() else { return Vec::new() };
    let Ok(entries) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.to_string_lossy().ends_with(".bindings.json") {
            continue;
        }
        let Some(json) = read_sidecar(&path) else { continue };
        if let Some(binding) = parse_binding(&path, &json) {
            let enabled = consumer_enabled(&json);
            out.push((binding, enabled));
        }
    }
    out
}

// ── status surface (read by /api and the P4 status enrichment) ─────────────

#[derive(Default)]
pub struct BindingStatus {
    pub last_poll_at: Option<u64>,
    pub last_outcome: Option<serde_json::Value>,
    pub consecutive_errors: u32,
    pub unpushed_local_edits: Option<u64>,
    pub last_local_scan_at: Option<u64>,
    /// 24 h report-only auto-reconcile: when it last ran and a counts-only
    /// summary (never key paths — bindings can hold client data).
    pub last_reconcile_at: Option<u64>,
    pub last_reconcile: Option<serde_json::Value>,
}

static STATUS: OnceLock<Mutex<HashMap<String, BindingStatus>>> = OnceLock::new();
static NUDGE: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
/// Bindings with a reconcile subprocess in flight (one at a time each).
static RECONCILING: OnceLock<Mutex<std::collections::HashSet<String>>> = OnceLock::new();
/// Adjudicated corrections handed back to the scheduler thread, which is the
/// only journal writer in this process (append ordering stays serialized).
#[allow(clippy::type_complexity)]
static RECONCILE_RESULTS: OnceLock<Mutex<Vec<(String, ReconcileOutcome)>>> = OnceLock::new();
static STARTED: AtomicBool = AtomicBool::new(false);

fn status_map() -> &'static Mutex<HashMap<String, BindingStatus>> {
    STATUS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn nudges() -> &'static Mutex<Vec<String>> {
    NUDGE.get_or_init(|| Mutex::new(Vec::new()))
}

fn reconciling() -> &'static Mutex<std::collections::HashSet<String>> {
    RECONCILING.get_or_init(|| Mutex::new(std::collections::HashSet::new()))
}

fn reconcile_results() -> &'static Mutex<Vec<(String, ReconcileOutcome)>> {
    RECONCILE_RESULTS.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn sync_now(binding_id: &str) {
    nudges().lock().unwrap().push(binding_id.to_string());
}

fn epoch_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

pub fn status_snapshot() -> serde_json::Value {
    let map = status_map().lock().unwrap();
    let mut out = serde_json::Map::new();
    for (binding_id, status) in map.iter() {
        out.insert(binding_id.clone(), serde_json::json!({
            "last_poll_at": status.last_poll_at,
            "outcome": status.last_outcome,
            "consecutive_errors": status.consecutive_errors,
            "local_edits": {"unpushed_count": status.unpushed_local_edits,
                             "scanned_at": status.last_local_scan_at},
            "reconcile": {"last_run_at": status.last_reconcile_at,
                           "summary": status.last_reconcile},
        }));
    }
    serde_json::Value::Object(out)
}

// ── enable / disable (the normative order) ──────────────────────────────────

#[derive(Debug)]
pub struct ToggleReport {
    pub binding_id: String,
    pub enabled: bool,
    pub bisync_agent_unloaded: bool,
    pub bisync_agent_reloaded: bool,
    pub lock_drained: bool,
}

fn uid_string() -> String {
    Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "501".to_string())
}

fn plist_path(label: &str) -> Option<PathBuf> {
    if label.contains('/') || label.contains("..") {
        return None;
    }
    home().map(|h| h.join("Library").join("LaunchAgents").join(format!("{label}.plist")))
}

fn unload_bisync_agent(label: &str) -> bool {
    let Some(plist) = plist_path(label) else { return false };
    if !plist.exists() {
        return false;
    }
    // bootout stops the schedule; the plist FILE stays for rollback reload.
    Command::new("launchctl")
        .arg("bootout")
        .arg(format!("gui/{}", uid_string()))
        .arg(&plist)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Positive proof the agent is NOT loaded (`launchctl print` on the service
/// target fails when it does not exist). The toggle fails closed on ambiguity:
/// a binding must never end up with BOTH movers active (G3-B review).
fn bisync_agent_loaded(label: &str) -> bool {
    Command::new("launchctl")
        .arg("print")
        .arg(format!("gui/{}/{}", uid_string(), label))
        .output()
        .map(|out| out.status.success())
        .unwrap_or(true) // cannot prove it's gone -> treat as loaded
}

fn reload_bisync_agent(label: &str) -> bool {
    let Some(plist) = plist_path(label) else { return false };
    if !plist.exists() {
        return false;
    }
    Command::new("launchctl")
        .arg("bootstrap")
        .arg(format!("gui/{}", uid_string()))
        .arg(&plist)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn drain_lock(workdir: Option<&Path>) -> bool {
    let Some(workdir) = workdir else { return true };
    let deadline = Instant::now() + Duration::from_secs(LOCK_DRAIN_TIMEOUT_SECS);
    loop {
        let locked = std::fs::read_dir(workdir)
            .map(|entries| {
                entries.flatten().any(|e| e.file_name().to_string_lossy().ends_with(".lck"))
            })
            .unwrap_or(false);
        if !locked {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

fn write_sidecar_atomic(path: &Path, json: &serde_json::Value) -> std::io::Result<()> {
    let tmp = path.with_extension("bindings.json.tmp");
    let bytes = serde_json::to_vec_pretty(json).unwrap_or_else(|_| b"{}".to_vec());
    match std::fs::write(&tmp, bytes).and_then(|_| std::fs::rename(&tmp, path)) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            Err(e)
        }
    }
}

/// Find a binding sidecar by folder_id, slug, or alias.
pub fn find_binding(reference: &str) -> Option<(EnabledBinding, serde_json::Value)> {
    for (binding, _) in discover() {
        if binding.binding_id == reference
            || binding.alias.as_deref() == Some(reference)
            || read_sidecar(&binding.sidecar_path)
                .and_then(|j| j.get("slug").and_then(|s| s.as_str()).map(|s| s == reference))
                .unwrap_or(false)
        {
            let json = read_sidecar(&binding.sidecar_path)?;
            return Some((binding, json));
        }
    }
    None
}

pub fn set_enabled(reference: &str, enable: bool) -> Result<ToggleReport, String> {
    let (binding, mut json) = find_binding(reference).ok_or_else(|| format!("no binding matches {reference:?}"))?;
    if enable && binding.routes.is_empty() {
        return Err("binding has no provisioned routes — nothing to consume".into());
    }
    let mut unloaded = false;
    let mut reloaded = false;
    let mut drained = true;
    let mut remote_binding_id = binding.remote_binding_id.clone();
    if enable {
        // Resolve the remote events namespace BEFORE touching the movers —
        // discovery is read-only, and an ambiguous namespace must abort the
        // toggle while bisync is still the active mover. `None` (producer has
        // not emitted yet) is fine: the scheduler re-discovers lazily.
        if remote_binding_id.is_none() {
            remote_binding_id = discover_remote_binding_id(&binding.bucket, &binding.routes)
                .map_err(|e| format!("remote binding-id discovery: {e}"))?;
        }
        // Normative order: (1) unload bisync agent, (2) drain the lock,
        // (3) set flags — polls start on the next scheduler tick.
        if let Some(label) = binding.plist_label.as_deref() {
            unloaded = unload_bisync_agent(label);
            // Fail closed (G3-B review): flags are set only with positive
            // proof the agent is NOT loaded — both movers must never run.
            if bisync_agent_loaded(label) {
                return Err(format!(
                    "bisync agent {label} is still loaded after bootout; toggle aborted (fail closed)"
                ));
            }
        }
        drained = drain_lock(binding.workdir.as_deref());
        if !drained {
            // Roll back step 1: never leave a binding with neither mover.
            let rollback_ok = binding
                .plist_label
                .as_deref()
                .map(reload_bisync_agent)
                .unwrap_or(true);
            return Err(if rollback_ok {
                "bisync lock did not drain within timeout; binding left on bisync".into()
            } else {
                "bisync lock did not drain AND the agent reload failed — binding has \
                 NO active mover; reload the LaunchAgent manually"
                    .into()
            });
        }
    }
    let schema_version = json.get("schema_version").and_then(|v| v.as_u64()).unwrap_or(2);
    if let Some(obj) = json.as_object_mut() {
        obj.insert("schema_version".into(), serde_json::json!(schema_version.max(3)));
        let state_dir = state_root()
            .map(|r| r.join(&binding.binding_id).to_string_lossy().into_owned());
        obj.insert("progressive".into(), serde_json::json!({
            "consumer_enabled": enable,
            "post_delta": enable,
            "remote_binding_id": remote_binding_id,
            "state_dir": state_dir,
            "toggled_at": tytus_progressive_sync::state::utc_now(),
        }));
    }
    if let Err(e) = write_sidecar_atomic(&binding.sidecar_path, &json) {
        if enable {
            // Bisync is already unloaded; a failed flag write must not leave
            // the binding with NO mover (codex PR#28 finding 2) — mirror the
            // drain-failure rollback.
            let rollback_ok = binding
                .plist_label
                .as_deref()
                .map(reload_bisync_agent)
                .unwrap_or(true);
            return Err(if rollback_ok {
                format!("sidecar write failed ({e}); binding left on bisync")
            } else {
                format!(
                    "sidecar write failed ({e}) AND the bisync agent reload failed — binding \
                     has NO active mover; reload the LaunchAgent manually"
                )
            });
        }
        // Disable path: sidecar still says consumer_enabled=true, so the
        // consumer remains the active mover — consistent, just not disabled.
        return Err(format!("sidecar write: {e}"));
    }
    if !enable {
        // Pre-delta rollback: resume bisync for this binding.
        if let Some(label) = binding.plist_label.as_deref() {
            reloaded = reload_bisync_agent(label);
        }
    }
    Ok(ToggleReport {
        binding_id: binding.binding_id,
        enabled: enable,
        bisync_agent_unloaded: unloaded,
        bisync_agent_reloaded: reloaded,
        lock_drained: drained,
    })
}

// ── 24 h report-only auto-reconcile (sprint P6, deferred from P5) ───────────
//
// Cadence: every RECONCILE_INTERVAL_SECS per enabled binding, escalated to
// RECONCILE_REPAIR_RETRY_SECS while any route is gap-halted. The subprocess
// (`garagetytus sync reconcile --binding <id> --json`, report-only — never
// `--repair-keys`) runs on its own thread; the ADJUDICATION comes back to the
// scheduler thread, the process's only journal writer. A durable Repair
// record is appended ONLY when the report proves the local tree is whole:
// complete=true and zero missing-local keys. Size mismatches never block
// (divergence is the keep-both conflict flow's job, not the gap's).

/// Counts-only reconcile outcome (no key paths — client data stays local).
#[derive(Debug, Clone)]
pub struct ReconcileOutcome {
    pub summary: serde_json::Value,
    pub corrections: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// When is a reconcile due? Pure so the cadence is unit-testable.
pub fn reconcile_due(last_run_at: Option<u64>, started_at: u64, now: u64, any_repair_required: bool) -> bool {
    match last_run_at {
        None => now.saturating_sub(started_at) >= RECONCILE_STARTUP_DELAY_SECS,
        Some(last) => {
            let interval = if any_repair_required {
                RECONCILE_REPAIR_RETRY_SECS
            } else {
                RECONCILE_INTERVAL_SECS
            };
            now.saturating_sub(last) >= interval
        }
    }
}

/// Adjudicate a sync-reconcile-report-v1 against the routes currently halted
/// on `repair_required`. Returns Repair corrections ONLY for a complete
/// report with zero missing-local keys; each halted route's cursor may then
/// skip to its observed high water (the gap's events are pruned — the FILES
/// are proven present, which is what the halt protects).
pub fn adjudicate_report(
    report: &serde_json::Value,
    halted_routes: &[(String, u64)],
) -> Option<serde_json::Value> {
    let complete = report.get("complete").and_then(|v| v.as_bool()).unwrap_or(false);
    let missing_local = report
        .get("missing_local")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(usize::MAX);
    if !complete || missing_local != 0 || halted_routes.is_empty() {
        return None;
    }
    let rows: Vec<serde_json::Value> = halted_routes
        .iter()
        .map(|(route_id, high_water)| {
            serde_json::json!({
                "route_id": route_id,
                "set_cursor_sequence": high_water,
            })
        })
        .collect();
    Some(serde_json::Value::Array(rows))
}

/// Counts-only summary of a report (never echoes keys/paths).
fn summarize_report(report: &serde_json::Value) -> serde_json::Value {
    let count = |k: &str| report.get(k).and_then(|v| v.as_array()).map(|a| a.len());
    serde_json::json!({
        "complete": report.get("complete"),
        "remote_keys": report.get("remote_keys"),
        "local_files": report.get("local_files"),
        "missing_local": count("missing_local"),
        "missing_remote": report.get("missing_remote_count"),
        "size_mismatch": count("size_mismatch"),
        "timed_out_prefixes": count("timed_out_prefixes"),
        "failed_prefixes": count("failed_prefixes"),
    })
}

/// Subprocess leg, run OFF the scheduler thread. Report-only by design.
fn run_reconcile_report(binding_id: &str) -> Result<serde_json::Value, String> {
    let out = Command::new(GARAGETYTUS_BIN)
        .arg("sync")
        .arg("reconcile")
        .arg("--binding")
        .arg(binding_id)
        .arg("--prefix-timeout-seconds")
        .arg("600")
        .arg("--json")
        .output()
        .map_err(|e| format!("spawn {GARAGETYTUS_BIN}: {e}"))?;
    // exit 2 = honest-incomplete (named timed-out prefixes) — still a report.
    let parsed: serde_json::Value = serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("reconcile output parse: {e} (exit {:?})", out.status.code()))?;
    Ok(parsed)
}

/// Spawn the reconcile thread for one binding (idempotent per binding).
fn start_reconcile(binding: &EnabledBinding, halted_routes: Vec<(String, u64)>) {
    {
        let mut inflight = reconciling().lock().unwrap();
        if !inflight.insert(binding.binding_id.clone()) {
            return;
        }
    }
    let binding_id = binding.binding_id.clone();
    let spawn_id = binding_id.clone();
    let spawned = std::thread::Builder::new()
        .name(format!("reconcile-{spawn_id}"))
        .spawn(move || {
            let outcome = match run_reconcile_report(&spawn_id) {
                Ok(report) => ReconcileOutcome {
                    summary: summarize_report(&report),
                    corrections: adjudicate_report(&report, &halted_routes),
                    error: None,
                },
                Err(e) => ReconcileOutcome {
                    summary: serde_json::Value::Null,
                    corrections: None,
                    error: Some(e),
                },
            };
            reconcile_results().lock().unwrap().push((spawn_id.clone(), outcome));
            reconciling().lock().unwrap().remove(&spawn_id);
        });
    if let Err(e) = spawned {
        // A failed spawn must not wedge the in-flight set (codex PR#30
        // finding 1) — queue an error outcome so cadence/status behave
        // exactly like any other reconcile failure.
        reconcile_results().lock().unwrap().push((
            binding_id.clone(),
            ReconcileOutcome {
                summary: serde_json::Value::Null,
                corrections: None,
                error: Some(format!("reconcile thread spawn: {e}")),
            },
        ));
        reconciling().lock().unwrap().remove(&binding_id);
    }
}

/// Routes currently gap-halted, extracted from the last poll outcome:
/// `(route_id, observed_high_water_sequence)` for every route whose
/// `repair_required` is set. Pure so it is unit-testable.
pub fn halted_routes_from_outcome(outcome: Option<&serde_json::Value>) -> Vec<(String, u64)> {
    let Some(routes) = outcome.and_then(|o| o.get("routes")).and_then(|r| r.as_object()) else {
        return Vec::new();
    };
    routes
        .iter()
        .filter(|(_, v)| v.get("repair_required").map(|r| !r.is_null()).unwrap_or(false))
        .map(|(route_id, v)| {
            (
                route_id.clone(),
                v.get("observed_high_water_sequence").and_then(|s| s.as_u64()).unwrap_or(0),
            )
        })
        .collect()
}

/// Scheduler-thread half: drain finished reconciles, record status, and
/// append the durable Repair record (single journal writer stays single).
fn drain_reconcile_results() {
    let finished: Vec<(String, ReconcileOutcome)> =
        std::mem::take(&mut *reconcile_results().lock().unwrap());
    for (binding_id, outcome) in finished {
        // Truthful append accounting (codex PR#30 finding 2): the status may
        // claim a repair correction ONLY after the Repair record is durably
        // appended. On failure the halt stays, the 1 h retry cadence re-runs
        // the reconcile, and the status says exactly what happened.
        let mut appended = false;
        let mut append_error: Option<String> = None;
        if let Some(corrections) = outcome.corrections.as_ref() {
            let result: Result<(), String> = (|| {
                let (binding, _) =
                    find_binding(&binding_id).ok_or("binding vanished before append")?;
                let root = state_root().ok_or("no HOME")?;
                let identity = BindingIdentity {
                    binding_id: binding.binding_id.clone(),
                    bucket: binding.bucket.clone(),
                    local_path: binding.local_path.clone(),
                    alias: binding.alias.clone(),
                };
                let store = StateStore::open(&root, &identity).map_err(|e| e.to_string())?;
                store
                    .append(&JournalRecord::Repair {
                        source: "tray-auto-reconcile-v1".into(),
                        corrections: corrections.clone(),
                        at: tytus_progressive_sync::state::utc_now(),
                    })
                    .map_err(|e| e.to_string())
            })();
            match result {
                Ok(()) => appended = true,
                Err(e) => append_error = Some(e),
            }
        }
        let mut map = status_map().lock().unwrap();
        let status = map.entry(binding_id).or_default();
        status.last_reconcile_at = Some(epoch_now());
        status.last_reconcile = Some(match (&outcome.error, &append_error) {
            (Some(e), _) => serde_json::json!({"error": e}),
            (None, Some(e)) => serde_json::json!({
                "report": outcome.summary,
                "repair_corrections": false,
                "repair_append_error": e,
            }),
            (None, None) => serde_json::json!({
                "report": outcome.summary,
                "repair_corrections": appended,
            }),
        });
    }
}

// ── the poll scheduler ──────────────────────────────────────────────────────

fn jittered_interval(binding_id: &str, tick: u64) -> Duration {
    // Deterministic ±25% jitter from a cheap hash — no rand dependency.
    let mut acc: u64 = tick.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    for b in binding_id.bytes() {
        acc = acc.rotate_left(7) ^ u64::from(b);
    }
    let base = POLL_INTERVAL_SECS * 1000;
    let spread = base / 2; // ±25%
    let offset = acc % spread;
    Duration::from_millis(base - spread / 2 + offset)
}

fn poll_binding(binding: &EnabledBinding) -> Result<serde_json::Value, String> {
    let root = state_root().ok_or("no HOME")?;
    let identity = BindingIdentity {
        binding_id: binding.binding_id.clone(),
        bucket: binding.bucket.clone(),
        local_path: binding.local_path.clone(),
        alias: binding.alias.clone(),
    };
    let store = StateStore::open(&root, &identity).map_err(|e| e.to_string())?;
    let (mut state, pending) = store.load(&identity, "tray").map_err(|e| e.to_string())?;
    let s3 = RcloneS3::new(RCLONE_REMOTE, &binding.bucket, rclone_conf_path());
    let mut consumer = BindingConsumer::new(&s3, &store, &mut state, &identity, binding.routes.clone());
    consumer.cycle_budget = DEFAULT_CYCLE_BUDGET;
    if let Some(remote_id) = binding.remote_binding_id.as_ref() {
        consumer.remote_binding_id = remote_id.clone();
    }
    consumer.recover(pending);
    let outcome = consumer.poll_once();
    let value = serde_json::json!({
        "remote_binding_id": binding.remote_binding_id,
        "applied": outcome.applied,
        "conflicts": outcome.conflicts,
        "dead_letters": outcome.dead_letters,
        "delete_ignored": outcome.delete_ignored,
        "events_seen": outcome.events_seen,
        "repair_required": outcome.repair_required,
        "transient_errors": outcome.transient_errors,
        "routes": state.routes.iter().map(|(route, cursor)| {
            (route.clone(), serde_json::json!({
                "last_applied_sequence": cursor.last_applied_sequence,
                "observed_high_water_sequence": cursor.observed_high_water_sequence,
                "repair_required": cursor.repair_required,
                "dead_letter_count": cursor.dead_letter_count,
            }))
        }).collect::<serde_json::Map<_, _>>(),
    });
    let had_errors = !outcome.transient_errors.is_empty();
    if had_errors {
        Err(value.to_string())
    } else {
        Ok(value)
    }
}

/// Hourly LOCAL-only scan (no remote listing): count files that differ from
/// the ledger — pull-only honesty (`local_edits_not_synced`).
fn scan_local_edits(binding: &EnabledBinding) -> Option<u64> {
    let root = state_root()?;
    let identity = BindingIdentity {
        binding_id: binding.binding_id.clone(),
        bucket: binding.bucket.clone(),
        local_path: binding.local_path.clone(),
        alias: binding.alias.clone(),
    };
    let store = StateStore::open(&root, &identity).ok()?;
    let (state, _) = store.load(&identity, "tray").ok()?;
    let base = PathBuf::from(&binding.local_path);
    let mut unpushed = 0u64;
    let mut stack = vec![base.clone()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || name == "_tytus-sync" {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let Ok(rel) = path.strip_prefix(&base) else { continue };
            let key = rel.to_string_lossy().replace('\\', "/");
            match state.ledger.get(&key) {
                None => unpushed += 1,
                Some(entry_record) => {
                    // size check first; hash only when it can differ
                    let changed = file_sha256(&path)
                        .map(|h| h != entry_record.sha256)
                        .unwrap_or(true);
                    if changed {
                        unpushed += 1;
                    }
                }
            }
        }
    }
    Some(unpushed)
}

/// Start the resident scheduler thread. Idempotent.
pub fn start_background() {
    if STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::Builder::new()
        .name("progressive-sync".into())
        .spawn(|| {
            let mut next_poll: HashMap<String, Instant> = HashMap::new();
            let mut backoff: HashMap<String, u64> = HashMap::new();
            let mut tick: u64 = 0;
            let started_at = epoch_now();
            loop {
                std::thread::sleep(Duration::from_secs(2));
                tick += 1;
                if kill_switch_engaged() || !crate::web_server::sharing_enabled_now() {
                    continue;
                }
                drain_reconcile_results();
                let nudged: Vec<String> = std::mem::take(&mut *nudges().lock().unwrap());
                for (mut binding, enabled) in discover() {
                    if !enabled || binding.routes.is_empty() {
                        continue;
                    }
                    let id = binding.binding_id.clone();
                    let due = next_poll.get(&id).map_or(true, |t| Instant::now() >= *t)
                        || nudged.iter().any(|n| n == &id);
                    if !due {
                        continue;
                    }
                    // Lazy remote-namespace discovery: bindings toggled before
                    // the producer's first emit have no remote_binding_id yet.
                    // Ambiguity AND persist failure fail closed (skip the poll,
                    // surface the error) — events may only be applied under an
                    // id that is durably bound to the sidecar. "Not found yet"
                    // polls the local id, which is harmless.
                    if binding.remote_binding_id.is_none() {
                        let resolved = discover_remote_binding_id(&binding.bucket, &binding.routes)
                            .map_err(|e| format!("remote binding-id discovery: {e}"))
                            .and_then(|found| match found {
                                Some(remote_id) => {
                                    persist_remote_binding_id(&binding.sidecar_path, &remote_id)
                                        .map_err(|e| format!("remote binding-id persist: {e}"))
                                        .map(|()| Some(remote_id))
                                }
                                None => Ok(None),
                            });
                        match resolved {
                            Ok(Some(remote_id)) => {
                                binding.remote_binding_id = Some(remote_id);
                            }
                            Ok(None) => {}
                            Err(e) => {
                                let mut map = status_map().lock().unwrap();
                                let status = map.entry(id.clone()).or_default();
                                status.last_poll_at = Some(epoch_now());
                                status.last_outcome = Some(serde_json::json!({
                                    "transient_errors": [e],
                                }));
                                status.consecutive_errors = status.consecutive_errors.saturating_add(1);
                                drop(map);
                                let delay = backoff
                                    .entry(id.clone())
                                    .and_modify(|d| *d = (*d * 2).min(BACKOFF_MAX_SECS))
                                    .or_insert(POLL_INTERVAL_SECS * 2);
                                next_poll.insert(id.clone(), Instant::now() + Duration::from_secs(*delay));
                                continue;
                            }
                        }
                    }
                    let result = poll_binding(&binding);
                    let mut map = status_map().lock().unwrap();
                    let status = map.entry(id.clone()).or_default();
                    status.last_poll_at = Some(epoch_now());
                    match result {
                        Ok(outcome) => {
                            status.last_outcome = Some(outcome);
                            status.consecutive_errors = 0;
                            backoff.remove(&id);
                            next_poll.insert(id.clone(), Instant::now() + jittered_interval(&id, tick));
                        }
                        Err(outcome) => {
                            status.last_outcome = serde_json::from_str(&outcome).ok();
                            status.consecutive_errors = status.consecutive_errors.saturating_add(1);
                            let delay = backoff
                                .entry(id.clone())
                                .and_modify(|d| *d = (*d * 2).min(BACKOFF_MAX_SECS))
                                .or_insert(POLL_INTERVAL_SECS * 2);
                            next_poll.insert(id.clone(), Instant::now() + Duration::from_secs(*delay));
                        }
                    }
                    let scan_due = status
                        .last_local_scan_at
                        .map_or(true, |at| epoch_now().saturating_sub(at) >= LOCAL_EDIT_SCAN_INTERVAL_SECS);
                    let halted = halted_routes_from_outcome(status.last_outcome.as_ref());
                    let recon_due = reconcile_due(
                        status.last_reconcile_at,
                        started_at,
                        epoch_now(),
                        !halted.is_empty(),
                    );
                    drop(map);
                    if scan_due {
                        let count = scan_local_edits(&binding);
                        let mut map = status_map().lock().unwrap();
                        let status = map.entry(id.clone()).or_default();
                        status.unpushed_local_edits = count;
                        status.last_local_scan_at = Some(epoch_now());
                    }
                    if recon_due {
                        start_reconcile(&binding, halted);
                    }
                }
            }
        })
        .ok();
}

// ── status v2 (contract shared-binding-sync-status-v2, Phase 4) ────────────

/// Count unresolved conflict records for a binding.
fn unresolved_conflicts(binding_id: &str) -> u64 {
    let Some(dir) = state_root().map(|r| r.join(binding_id).join("conflicts")) else { return 0 };
    let Ok(entries) = std::fs::read_dir(&dir) else { return 0 };
    entries
        .flatten()
        .filter(|e| {
            std::fs::read_to_string(e.path())
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .map(|record| record.get("resolved_at").map_or(true, |v| v.is_null()))
                .unwrap_or(false)
        })
        .count() as u64
}

/// Pure state derivation — the STRICT `synced` definition (contract):
/// synced ⇔ fresh poll ∧ no repair_required ∧ zero unresolved conflicts ∧
/// zero dead-letters ∧ every authorized route cursor == observed high water.
/// Everything else is `syncing` or `attention` with explicit reasons.
/// Recorded v1 deviation: the 15-min PRODUCER echo cap is deferred to the
/// reconcile report (P5) — surfaced as `producer_echo: "unverified"`.
pub fn derive_progressive_state(
    outcome: Option<&serde_json::Value>,
    last_poll_at: Option<u64>,
    consecutive_errors: u32,
    conflicts_unresolved: u64,
    unpushed_local_edits: Option<u64>,
    now_epoch: u64,
) -> (String, Vec<String>) {
    let mut reasons = Vec::new();
    let mut attention = false;
    let mut syncing = false;
    match last_poll_at {
        None => {
            syncing = true;
            reasons.push("no_poll_yet".into());
        }
        Some(at) => {
            let age = now_epoch.saturating_sub(at);
            if age > 600 {
                attention = true;
                reasons.push("polls_stalled".into());
            } else if age > POLL_INTERVAL_SECS * 4 {
                syncing = true;
                reasons.push("poll_stale".into());
            }
        }
    }
    if consecutive_errors >= 3 {
        attention = true;
        reasons.push("repeated_poll_errors".into());
    }
    if conflicts_unresolved > 0 {
        attention = true;
        reasons.push("unresolved_conflicts".into());
    }
    if let Some(outcome) = outcome {
        if outcome
            .get("repair_required")
            .and_then(|r| r.as_object())
            .map(|m| !m.is_empty())
            .unwrap_or(false)
        {
            attention = true;
            reasons.push("repair_required".into());
        }
        if let Some(routes) = outcome.get("routes").and_then(|r| r.as_object()) {
            let mut behind = false;
            let mut dead = 0u64;
            for cursor in routes.values() {
                let applied = cursor.get("last_applied_sequence").and_then(|v| v.as_u64()).unwrap_or(0);
                let high = cursor.get("observed_high_water_sequence").and_then(|v| v.as_u64()).unwrap_or(0);
                if high > applied {
                    behind = true;
                }
                dead += cursor.get("dead_letter_count").and_then(|v| v.as_u64()).unwrap_or(0);
                if cursor.get("repair_required").map_or(false, |v| !v.is_null()) {
                    attention = true;
                    if !reasons.iter().any(|r| r == "repair_required") {
                        reasons.push("repair_required".into());
                    }
                }
            }
            if dead > 0 {
                attention = true;
                reasons.push("dead_letters".into());
            }
            if behind {
                syncing = true;
                reasons.push("events_pending".into());
            }
        }
    }
    if unpushed_local_edits.unwrap_or(0) > 0 {
        // Pull-only honesty: local edits are NOT syncing on a v1 binding.
        // Informational reason + UI badge; does not fake `attention`.
        reasons.push("local_edits_not_synced".into());
    }
    let state = if attention {
        "attention"
    } else if syncing {
        "syncing"
    } else {
        "synced"
    };
    (state.into(), reasons)
}

/// Full progressive `sync_status` block for one enabled binding, or None
/// when the binding has no consumer state yet.
pub fn binding_status_v2(binding_id: &str, checked_at: u64) -> Option<serde_json::Value> {
    let map = status_map().lock().unwrap();
    let status = map.get(binding_id);
    let conflicts = unresolved_conflicts(binding_id);
    let (outcome, last_poll_at, consecutive_errors, local_edits, scanned_at) = match status {
        Some(s) => (
            s.last_outcome.clone(),
            s.last_poll_at,
            s.consecutive_errors,
            s.unpushed_local_edits,
            s.last_local_scan_at,
        ),
        None => (None, None, 0, None, None),
    };
    let (state, reasons) = derive_progressive_state(
        outcome.as_ref(),
        last_poll_at,
        consecutive_errors,
        conflicts,
        local_edits,
        checked_at,
    );
    Some(serde_json::json!({
        "schema_version": "shared-binding-sync-status-v2",
        "mode": "progressive",
        "pull_only": true,
        "state": state,
        "reasons": reasons,
        "checked_at": checked_at,
        "last_poll_at": last_poll_at,
        "consecutive_errors": consecutive_errors,
        "conflicts_unresolved": conflicts,
        "local_edits": {"unpushed_count": local_edits, "scanned_at": scanned_at},
        "producer_echo": "unverified",
        "outcome": outcome,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outcome(routes: serde_json::Value, repair: serde_json::Value) -> serde_json::Value {
        serde_json::json!({"routes": routes, "repair_required": repair})
    }

    #[test]
    fn strict_synced_requires_full_conjunction() {
        let ok = outcome(
            serde_json::json!({"r1": {"last_applied_sequence": 5, "observed_high_water_sequence": 5, "dead_letter_count": 0}}),
            serde_json::json!({}),
        );
        let (state, reasons) = derive_progressive_state(Some(&ok), Some(1000), 0, 0, Some(0), 1010);
        assert_eq!(state, "synced");
        assert!(reasons.is_empty(), "{reasons:?}");
    }

    #[test]
    fn backlog_is_syncing_never_synced() {
        let behind = outcome(
            serde_json::json!({"r1": {"last_applied_sequence": 3, "observed_high_water_sequence": 5, "dead_letter_count": 0}}),
            serde_json::json!({}),
        );
        let (state, reasons) = derive_progressive_state(Some(&behind), Some(1000), 0, 0, None, 1010);
        assert_eq!(state, "syncing");
        assert!(reasons.contains(&"events_pending".to_string()));
    }

    #[test]
    fn repair_conflicts_dead_letters_stalls_are_attention() {
        let repair = outcome(serde_json::json!({}), serde_json::json!({"r1": "sequence_gap"}));
        assert_eq!(derive_progressive_state(Some(&repair), Some(1000), 0, 0, None, 1010).0, "attention");
        let clean = outcome(serde_json::json!({}), serde_json::json!({}));
        assert_eq!(derive_progressive_state(Some(&clean), Some(1000), 0, 2, None, 1010).0, "attention");
        let dead = outcome(
            serde_json::json!({"r1": {"last_applied_sequence": 5, "observed_high_water_sequence": 5, "dead_letter_count": 1}}),
            serde_json::json!({}),
        );
        assert_eq!(derive_progressive_state(Some(&dead), Some(1000), 0, 0, None, 1010).0, "attention");
        // polls stalled >10min — the incident-replay shape (frozen movement,
        // reachable endpoint) can NEVER read synced
        assert_eq!(derive_progressive_state(Some(&clean), Some(1000), 0, 0, None, 1000 + 601).0, "attention");
        assert_eq!(derive_progressive_state(Some(&clean), None, 0, 0, None, 1010).0, "syncing");
    }

    #[test]
    fn local_edits_reason_is_informational() {
        let clean = outcome(
            serde_json::json!({"r1": {"last_applied_sequence": 5, "observed_high_water_sequence": 5, "dead_letter_count": 0}}),
            serde_json::json!({}),
        );
        let (state, reasons) = derive_progressive_state(Some(&clean), Some(1000), 0, 0, Some(7), 1010);
        assert_eq!(state, "synced");
        assert!(reasons.contains(&"local_edits_not_synced".to_string()));
    }

    #[test]
    fn jitter_stays_within_25_percent() {
        for tick in 0..50 {
            let d = jittered_interval("sf_test", tick).as_millis() as u64;
            assert!((15_000..=25_000).contains(&d), "{d}");
        }
    }

    #[test]
    fn parse_binding_extracts_routes_from_targets() {
        let json = serde_json::json!({
            "folder_id": "sf_x", "bucket": "b", "local_path": "/tmp/x/",
            "plist_label": "com.traylinx.garagetytus.bisync.x",
            "targets": [
                {"route_id": "r1", "enabled": true},
                {"route_id": "r2", "enabled": false},
                {"route_id": "r1", "enabled": true},
            ],
            "routes_provisioned": ["r9"],
        });
        let binding = parse_binding(Path::new("/tmp/x.bindings.json"), &json).unwrap();
        assert_eq!(binding.routes, vec!["r1"]);
        assert_eq!(binding.local_path, "/tmp/x");
    }

    #[test]
    fn parse_binding_falls_back_to_routes_provisioned() {
        let json = serde_json::json!({
            "folder_id": "sf_x", "bucket": "b", "local_path": "/tmp/x",
            "routes_provisioned": ["r2", "r1", "r2"],
        });
        let binding = parse_binding(Path::new("/tmp/x.bindings.json"), &json).unwrap();
        assert_eq!(binding.routes, vec!["r1", "r2"]);
    }

    #[test]
    fn consumer_enabled_reads_progressive_block() {
        let on = serde_json::json!({"progressive": {"consumer_enabled": true}});
        let off = serde_json::json!({"progressive": {"consumer_enabled": false}});
        let absent = serde_json::json!({});
        assert!(consumer_enabled(&on));
        assert!(!consumer_enabled(&off));
        assert!(!consumer_enabled(&absent));
    }

    #[test]
    fn parse_binding_reads_remote_binding_id() {
        let with = serde_json::json!({
            "folder_id": "sf_local", "bucket": "b", "local_path": "/tmp/x",
            "routes_provisioned": ["r1"],
            "progressive": {"consumer_enabled": true, "remote_binding_id": "sf_remote"},
        });
        let without = serde_json::json!({
            "folder_id": "sf_local", "bucket": "b", "local_path": "/tmp/x",
            "routes_provisioned": ["r1"],
            "progressive": {"consumer_enabled": true, "remote_binding_id": null},
        });
        assert_eq!(
            parse_binding(Path::new("/tmp/x.bindings.json"), &with).unwrap().remote_binding_id,
            Some("sf_remote".to_string())
        );
        assert_eq!(
            parse_binding(Path::new("/tmp/x.bindings.json"), &without).unwrap().remote_binding_id,
            None
        );
    }

    #[test]
    fn match_remote_candidates_route_scoped() {
        let routes = vec!["r-lisa".to_string(), "r-claus".to_string()];
        // one namespace claims our routes → unambiguous
        let one = vec![
            ("sf_provider".to_string(), vec!["r-lisa".to_string()]),
            ("sf_other".to_string(), vec!["r-foreign".to_string()]),
        ];
        assert_eq!(match_remote_candidates(&one, &routes).unwrap(), Some("sf_provider".into()));
        // none claim them → not discovered yet
        let none = vec![("sf_other".to_string(), vec!["r-foreign".to_string()])];
        assert_eq!(match_remote_candidates(&none, &routes).unwrap(), None);
        assert_eq!(match_remote_candidates(&[], &routes).unwrap(), None);
        // two claim them → fail closed
        let two = vec![
            ("sf_a".to_string(), vec!["r-lisa".to_string()]),
            ("sf_b".to_string(), vec!["r-claus".to_string()]),
        ];
        assert!(match_remote_candidates(&two, &routes).is_err());
    }

    #[test]
    fn reconcile_due_cadence() {
        let start = 1_000_000;
        // never ran: waits out the startup delay
        assert!(!reconcile_due(None, start, start + RECONCILE_STARTUP_DELAY_SECS - 1, false));
        assert!(reconcile_due(None, start, start + RECONCILE_STARTUP_DELAY_SECS, false));
        // ran: 24 h cadence normally
        let last = start + 10_000;
        assert!(!reconcile_due(Some(last), start, last + RECONCILE_INTERVAL_SECS - 1, false));
        assert!(reconcile_due(Some(last), start, last + RECONCILE_INTERVAL_SECS, false));
        // gap-halted route escalates to the 1 h retry
        assert!(!reconcile_due(Some(last), start, last + RECONCILE_REPAIR_RETRY_SECS - 1, true));
        assert!(reconcile_due(Some(last), start, last + RECONCILE_REPAIR_RETRY_SECS, true));
    }

    #[test]
    fn adjudicate_report_requires_complete_and_whole() {
        let halted = vec![("r1".to_string(), 42u64)];
        let whole = serde_json::json!({"complete": true, "missing_local": []});
        let rows = adjudicate_report(&whole, &halted).expect("whole report adjudicates");
        assert_eq!(rows[0]["route_id"], "r1");
        assert_eq!(rows[0]["set_cursor_sequence"], 42);
        // incomplete report never clears a halt
        let partial = serde_json::json!({"complete": false, "missing_local": []});
        assert!(adjudicate_report(&partial, &halted).is_none());
        // missing-local keys never clear a halt
        let holes = serde_json::json!({"complete": true, "missing_local": ["k"]});
        assert!(adjudicate_report(&holes, &halted).is_none());
        // absent field fails closed
        let bare = serde_json::json!({"complete": true});
        assert!(adjudicate_report(&bare, &halted).is_none());
        // nothing halted -> nothing to correct
        assert!(adjudicate_report(&whole, &[]).is_none());
    }

    #[test]
    fn halted_routes_extracted_from_outcome() {
        let outcome = serde_json::json!({
            "routes": {
                "r-ok": {"repair_required": null, "observed_high_water_sequence": 7},
                "r-halted": {"repair_required": "retention_gap", "observed_high_water_sequence": 9},
            }
        });
        let halted = halted_routes_from_outcome(Some(&outcome));
        assert_eq!(halted, vec![("r-halted".to_string(), 9)]);
        assert!(halted_routes_from_outcome(None).is_empty());
    }
}

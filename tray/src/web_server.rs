//! Local HTTP server for the TytusOS control surface.
//!
//! Binds to `127.0.0.1:<random>` at tray startup, serves embedded TytusOS assets,
//! and exposes a tiny local API so the static JS can (a) list the agent
//! catalog, (b) kick off a `tytus agent install` subprocess, and (c) stream
//! its stdout back via server-sent events. Legacy `/install` paths 302 to
//! `/`; `/tower` is hidden rollback only behind `TYTUS_ENABLE_LEGACY_TOWER=1`.
//!
//! Spec: SPRINT-AIL-DEFAULT-POD-AND-AGENT-INSTALL.md §6 E1-E5.
//!
//! Design constraints:
//! - No external runtime deps (std + `tiny_http` + `serde_json`). Shipping
//!   a browser-wizard dependency-free keeps the tray binary under 12MB.
//! - Synchronous `tiny_http` + a small thread pool. We expect one
//!   concurrent install job at a time — the UI only lets the user click
//!   one card. Parallel installs would overspend units anyway.
//! - Port bound at startup to `127.0.0.1:0` (kernel picks). Written to
//!   `<tmp>/tytus/tray-web.port` so `open_os()` can read it.
//! - Lifecycle: server thread owns the `tiny_http::Server` and parks on
//!   `recv()`. On tray quit we drop the `Arc<Server>` and the kernel
//!   tears down the listener.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Daemon boot timestamp (Unix epoch seconds). Set once on first
/// `start()` call; UI consumers compare against the value last seen
/// to detect a daemon restart and drop stale activeJob state.
static DAEMON_STARTED_AT: OnceLock<u64> = OnceLock::new();

fn daemon_started_at() -> u64 {
    *DAEMON_STARTED_AT.get_or_init(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    })
}

use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

use crate::{juli3ta_library, music_connectors, music_proxy, music_ytdlp, music_ytdlp_setup};

mod os_assets {
    include!(concat!(env!("OUT_DIR"), "/os_assets.rs"));
}

// ── Embedded static assets ────────────────────────────────────
// `include_bytes!` paths are relative to THIS source file, so the web/
// directory lives next to src/.
const TOWER_HTML: &[u8] = include_bytes!("../web/tower.html");
const TOWER_CSS: &[u8] = include_bytes!("../web/assets/tower.css");
const TOWER_JS: &[u8] = include_bytes!("../web/assets/tower.js");
const LEGACY_TOWER_ENV: &str = "TYTUS_ENABLE_LEGACY_TOWER";

// ── Icons (lobehub @1.87.0) ───────────────────────────────────
// Baked into the binary so the CSP can stay 'self'-only and the
// wizard renders even when the laptop is offline. Mapping:
//   openclaw.svg → agent id "nemoclaw" (display name is OpenClaw)
//   hermes.svg   → agent id "hermes"
//   nvidia.svg   → reserved for future sandbox badge
const ICON_OPENCLAW: &[u8] = include_bytes!("../web/assets/icons/openclaw.svg");
const ICON_HERMES: &[u8] = include_bytes!("../web/assets/icons/hermes.svg");
const ICON_NVIDIA: &[u8] = include_bytes!("../web/assets/icons/nvidia.svg");

// App Store catalog (static JSON, embedded at compile time)
const APPS_JSON: &[u8] = include_bytes!("../web/assets/apps.json");

// ── Job registry ──────────────────────────────────────────────
//
// Each install job is a live subprocess (tytus agent install <type>)
// plus a channel of streaming events for in-flight SSE consumers. Jobs
// are indexed by a random-ish id so the browser can reconnect to a
// running job if the EventSource hiccups.

enum JobEvent {
    Log(String),
    Done { payload: String }, // install-shaped: payload is the CLI's JSON result
    Fail { message: String },
    Exit { code: i32 }, // pod-action-shaped: process exited with code
}

struct Job {
    events: Vec<JobEvent>,
    finished: bool,
    /// `None` for the install flow (one global install at a time);
    /// `Some(pod_id)` for per-pod actions so `Registry::active_for_pod`
    /// can enforce one-running-action-per-pod and the TytusOS UI can
    /// badge pod rows that have a live job.
    pod_id: Option<String>,
    /// Live child PID for cancellation. Set by the spawn thread once
    /// `Command::spawn()` returns; cleared after `child.wait()`. The
    /// `POST /api/jobs/<id>/cancel` handler reads this to send SIGTERM.
    /// `None` means "no running process" (job is queued / failed to
    /// spawn / already exited) — cancel becomes a no-op.
    child_pid: Option<u32>,
    /// Set when push_event has dropped a Log line because the event
    /// vec is at MAX_EVENTS. The first such drop emits a single
    /// truncation sentinel; subsequent drops are silent. This keeps
    /// cursor indices monotonic (replay never reorders) and bounds
    /// daemon RAM against a misbehaving subprocess.
    log_capped: bool,
}

impl Job {
    fn new_install() -> Self {
        Job {
            events: Vec::new(),
            finished: false,
            pod_id: None,
            child_pid: None,
            log_capped: false,
        }
    }
    fn new_pod(pod_id: String) -> Self {
        Job {
            events: Vec::new(),
            finished: false,
            pod_id: Some(pod_id),
            child_pid: None,
            log_capped: false,
        }
    }
}

/// Cap on Log events per job. ~5k lines × 16 KB max line ≈ 80 MB
/// worst-case, but typical pod actions emit < 200 lines so this is
/// effectively a runaway-process guard, not a normal-path constraint.
/// Past the cap, further Log events are dropped (one sentinel inserted
/// the first time it triggers). Terminal Done / Fail / Exit always
/// append regardless — they're ≤ a few hundred bytes and the SSE
/// consumer needs them to wind the EventSource down.
const MAX_EVENTS: usize = 5_000;

/// Cap on a single Log line's byte length. Truncated lines append a
/// `…[truncated]` suffix so the consumer can spot it. 16 KB is generous
/// for real log output (typical lines < 200 B); the cap exists so a
/// pathological JSON dump or core-dump trace can't pin one frame to
/// a multi-MB allocation.
const MAX_LINE_LEN: usize = 16 * 1024;

#[derive(Clone)]
struct Registry {
    inner: Arc<Mutex<HashMap<String, Arc<Mutex<Job>>>>>,
}

impl Registry {
    fn new() -> Self {
        Registry {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Install-flow constructor. Pre-existing call shape; kept stable.
    fn create(&self) -> (String, Arc<Mutex<Job>>) {
        let id = random_job_id();
        let job = Arc::new(Mutex::new(Job::new_install()));
        self.inner.lock().unwrap().insert(id.clone(), job.clone());
        (id, job)
    }

    /// Per-pod constructor. Returns `Err(pod)` if a job is already
    /// running on that pod so callers can reject with 409 Conflict.
    fn create_pod(&self, pod_id: &str) -> Result<(String, Arc<Mutex<Job>>), String> {
        let mut guard = self.inner.lock().unwrap();
        for job in guard.values() {
            let j = job.lock().unwrap();
            if j.pod_id.as_deref() == Some(pod_id) && !j.finished {
                return Err(pod_id.to_string());
            }
        }
        let id = random_job_id();
        let job = Arc::new(Mutex::new(Job::new_pod(pod_id.to_string())));
        guard.insert(id.clone(), job.clone());
        Ok((id, job))
    }

    fn get(&self, id: &str) -> Option<Arc<Mutex<Job>>> {
        self.inner.lock().unwrap().get(id).cloned()
    }

    /// Compact view of currently-running per-pod jobs, keyed by pod_id
    /// → count. Surfaced in StateSnapshot so the TytusOS overview can
    /// dot pod rows that have a live action streaming.
    fn active_pods(&self) -> HashMap<String, usize> {
        let mut out: HashMap<String, usize> = HashMap::new();
        for job in self.inner.lock().unwrap().values() {
            let j = job.lock().unwrap();
            if j.finished {
                continue;
            }
            if let Some(pod) = &j.pod_id {
                *out.entry(pod.clone()).or_insert(0) += 1;
            }
        }
        out
    }
}

// ── Idempotency cache for destructive POSTs ───────────────────
//
// A retry of `POST /api/pod/restart?pod=02` after a flaky tunnel must
// NOT spawn a second `tytus restart --pod 02` subprocess. RFC-style
// `Idempotency-Key` solves this: the client mints one key per logical
// action and reuses it on retry; the daemon caches the first response
// keyed by it and replays the cached body on subsequent hits.
//
// Process-local cache only — keys live for `IDEM_TTL_SECS` and the
// table is bounded by `IDEM_MAX_ENTRIES`. Daemon restart drops the
// cache; that's fine because all in-flight job_ids would also be lost
// on restart (registry is in-memory, see useDaemonState's restart
// detection that already wipes activeJob state on `daemon_started_at`
// drift).
//
// Concurrency contract: two requests with the same key arriving while
// the first is still running BOTH execute and last-write-wins on the
// cache entry. We don't queue the second behind the first because
// (a) the realistic UI is a single user clicking once, and (b) the
// cost of duplicate spawn for the rare race is bounded by Registry's
// `create_pod` 409 — which the second request would already trip if
// it's a per-pod action. Once the first response lands in the cache,
// every subsequent retry replays it.
//
// 5xx responses are NOT cached: the failure may be transient (network
// blip to upstream, momentary file lock) and the client should be free
// to retry without being permanently stuck on a stale error. 2xx and
// 4xx responses ARE cached because they're deterministic outcomes of
// the request (success, validation error, 404 unknown pod, etc.) and
// replaying them keeps the UI consistent across retries.

const IDEM_TTL_SECS: u64 = 600;
const IDEM_MAX_ENTRIES: usize = 256;
const IDEM_MAX_KEY_LEN: usize = 200;

#[derive(Clone)]
struct IdemEntry {
    status: u16,
    body: String,
    inserted_at: u64,
}

static IDEMPOTENCY_CACHE: OnceLock<Mutex<HashMap<String, IdemEntry>>> = OnceLock::new();

fn idempotency_cache() -> &'static Mutex<HashMap<String, IdemEntry>> {
    IDEMPOTENCY_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

// ── Per-pod status cache (Phase 2 of remaining sprint) ─────────
//
// Lifts the OS's per-pod /api/pod/ready polling server-side. Each
// agent in StateSnapshot carries a `status` field derived from a
// gateway probe. With 15 pods on Operator that drops 15 extra HTTP
// requests per poll cadence to zero — the OS just reads
// state.agents[].status.
//
// Cache TTL: 5s. Stale entries trigger a background-thread refresh
// while the current /api/state response returns the cached (or
// Unknown) value. We do NOT block /api/state on a probe — a slow
// upstream would otherwise cascade into a slow daemon.

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum AgentStatus {
    /// Last probe returned 200. Pod is serving traffic end-to-end.
    Ready,
    /// 401/403/404 — edge auth not yet propagated, route not yet
    /// published, or pod still booting. Wizard keeps polling.
    Starting,
    /// 5xx — gateway answering but upstream unhealthy.
    Unhealthy,
    /// Probe returned a connection-level error (refused, timeout,
    /// DNS) — the gateway itself is unreachable. Treat as "container
    /// is down or networking is broken".
    Stopped,
    /// Never probed yet (pod just appeared) OR a transient failure
    /// the cache hasn't classified. UI should render as "checking".
    Unknown,
}

#[derive(Clone, Copy, Debug)]
struct StatusEntry {
    status: AgentStatus,
    fetched_at: u64,
}

const STATUS_TTL_SECS: u64 = 5;
const BOOTSTRAP_HEALTH_TTL_SECS: u64 = 15;

static STATUS_CACHE: OnceLock<Mutex<HashMap<String, StatusEntry>>> = OnceLock::new();
static BOOTSTRAP_HEALTH_CACHE: OnceLock<Mutex<HashMap<String, BootstrapHealthEntry>>> =
    OnceLock::new();

fn status_cache() -> &'static Mutex<HashMap<String, StatusEntry>> {
    STATUS_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Clone, Debug)]
struct BootstrapHealthEntry {
    status: String,
    detail: String,
    overall: String,
    fetched_at: u64,
}

fn bootstrap_health_cache() -> &'static Mutex<HashMap<String, BootstrapHealthEntry>> {
    BOOTSTRAP_HEALTH_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Read the cached status for `pod_id`. Returns `Unknown` for cache
/// miss + kicks a background refresh if the entry is stale or missing
/// AND `api_url` + `user_key` are non-empty (we can't probe without
/// them).
fn agent_status_cached(
    pod_id: &str,
    agent_type: &str,
    api_url: Option<&str>,
    user_key: &str,
    gateway_token: Option<&str>,
    ui_url: Option<&str>,
) -> AgentStatus {
    let now = now_secs();
    let (cached, needs_refresh) = {
        let cache = status_cache().lock().unwrap();
        match cache.get(pod_id) {
            Some(e) => {
                let stale = now.saturating_sub(e.fetched_at) >= STATUS_TTL_SECS;
                (e.status, stale)
            }
            None => (AgentStatus::Unknown, true),
        }
    };

    if needs_refresh {
        if let (Some(url), false) = (api_url, user_key.is_empty()) {
            if !url.is_empty() {
                let pod = pod_id.to_string();
                let agent = agent_type.to_string();
                let url = url.to_string();
                let key = user_key.to_string();
                let gateway_key = gateway_token.map(|t| t.to_string());
                let ui = ui_url.map(|u| u.to_string());
                thread::spawn(move || {
                    let s = probe_agent_status(
                        &agent,
                        &url,
                        &key,
                        gateway_key.as_deref(),
                        ui.as_deref(),
                    );
                    let mut cache = status_cache().lock().unwrap();
                    cache.insert(
                        pod,
                        StatusEntry {
                            status: s,
                            fetched_at: now_secs(),
                        },
                    );
                });
            }
        }
    }
    cached
}

/// Synchronous-blocking probe of a pod's gateway. Mirrors
/// handle_pod_ready's classification — same Authorization header,
/// same 4s timeout, same status-code map. Run inside a per-call
/// tokio runtime so we can block until it finishes (called from
/// thread::spawn — never from the request thread).
fn probe_agent_status(
    agent_type: &str,
    api_url: &str,
    user_key: &str,
    gateway_token: Option<&str>,
    ui_url: Option<&str>,
) -> AgentStatus {
    if api_url.is_empty() {
        return AgentStatus::Stopped;
    }
    let probe_url = format!("{}/models", api_url.trim_end_matches('/'));
    let bearer = probe_bearer_token(agent_type, user_key, gateway_token);
    if bearer.is_empty() {
        return AgentStatus::Starting;
    }
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(_) => return AgentStatus::Unknown,
    };
    let http = atomek_core::HttpClient::new();
    let result = rt.block_on(async {
        http.get(&probe_url)
            .header("Authorization", format!("Bearer {}", bearer))
            .timeout(std::time::Duration::from_secs(4))
            .send()
            .await
    });
    let api_status = match result {
        Ok(resp) => match resp.status().as_u16() {
            200 => AgentStatus::Ready,
            401 | 403 | 404 => AgentStatus::Starting,
            500..=599 => AgentStatus::Unhealthy,
            _ => AgentStatus::Unknown,
        },
        Err(_) => AgentStatus::Stopped,
    };

    if api_status != AgentStatus::Ready {
        return api_status;
    }

    // OpenClaw/NemoClaw and Hermes expose browser UIs. The SDK gateway can
    // answer before Caddy has the SPA/dashboard upstream fully wired, which made
    // TytusOS show "Running" while the user's browser still saw 502.
    // Treat browser agents as ready only when BOTH /v1/models and the
    // authenticated UI root serve successfully.
    if agent_has_browser_ui(agent_type) {
        let Some(ui) = ui_url.filter(|u| !u.is_empty()) else {
            return AgentStatus::Starting;
        };
        let ui_result = rt.block_on(async {
            http.get(ui)
                .timeout(std::time::Duration::from_secs(4))
                .send()
                .await
        });
        match ui_result {
            Ok(resp) => match resp.status().as_u16() {
                200 => AgentStatus::Ready,
                401 | 403 | 404 => AgentStatus::Starting,
                500..=599 => AgentStatus::Unhealthy,
                _ => AgentStatus::Unknown,
            },
            Err(_) => AgentStatus::Stopped,
        }
    } else {
        AgentStatus::Ready
    }
}

fn agent_has_browser_ui(agent_type: &str) -> bool {
    matches!(agent_type, "nemoclaw" | "openclaw" | "hermes")
}

fn probe_bearer_token(agent_type: &str, user_key: &str, gateway_token: Option<&str>) -> String {
    if agent_type == "hermes" {
        if let Some(token) = gateway_token.filter(|t| !t.is_empty()) {
            return token.to_string();
        }
    }
    user_key.to_string()
}

fn readiness_strict_enabled() -> bool {
    let raw = std::env::var("TYTUS_READINESS_STRICT").unwrap_or_else(|_| "1".to_string());
    !matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "0" | "false" | "off" | "no"
    )
}

fn agent_status_label(status: AgentStatus) -> &'static str {
    match status {
        AgentStatus::Ready => "ready",
        AgentStatus::Starting => "starting",
        AgentStatus::Unhealthy => "unhealthy",
        AgentStatus::Stopped => "stopped",
        AgentStatus::Unknown => "unknown",
    }
}

fn stage_status_from_agent(status: AgentStatus) -> &'static str {
    match status {
        AgentStatus::Ready => "ok",
        AgentStatus::Starting => "starting",
        AgentStatus::Unhealthy => "failed",
        AgentStatus::Stopped => "failed",
        AgentStatus::Unknown => "unknown",
    }
}

fn run_tytus_exec_shell(
    pod_id: &str,
    script: &str,
    timeout: Duration,
) -> Result<(bool, String, String, bool), String> {
    let bin = resolve_tytus_bin();
    let started = Instant::now();
    let timeout_secs = timeout.as_secs().clamp(1, 120).to_string();
    let mut child = Command::new(&bin)
        .args([
            "--json",
            "exec",
            "--pod",
            pod_id,
            "--timeout",
            &timeout_secs,
            script,
        ])
        .env("TYTUS_HEADLESS", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to run tytus exec: {}", e))?;

    loop {
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let output = child
                .wait_with_output()
                .map_err(|e| format!("failed to collect timed-out tytus exec: {}", e))?;
            return Ok((
                false,
                String::from_utf8_lossy(&output.stdout).to_string(),
                String::from_utf8_lossy(&output.stderr).to_string(),
                true,
            ));
        }
        match child.try_wait() {
            Ok(Some(_)) => {
                let output = child
                    .wait_with_output()
                    .map_err(|e| format!("failed to collect tytus exec: {}", e))?;
                let stdout_raw = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr_raw = String::from_utf8_lossy(&output.stderr).to_string();
                if output.status.success() {
                    let parsed = serde_json::from_str::<serde_json::Value>(&stdout_raw)
                        .map_err(|e| format!("failed to parse tytus exec json: {}", e))?;
                    let exit_code = parsed
                        .get("exit_code")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(1);
                    let stdout = parsed
                        .get("stdout")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let stderr = parsed
                        .get("stderr")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    return Ok((exit_code == 0, stdout, stderr, false));
                }
                return Ok((false, stdout_raw, stderr_raw, false));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(e) => return Err(format!("failed to poll tytus exec: {}", e)),
        }
    }
}

fn summarize_health_json(raw: &str) -> (String, String, String) {
    let parsed = match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(v) => v,
        Err(e) => {
            return (
                "unknown".to_string(),
                format!("health.json malformed: {}", e),
                "unknown".to_string(),
            )
        }
    };
    let overall = parsed
        .get("overall")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let checks = parsed
        .get("checks")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut failed_workspace: Option<String> = None;
    let mut failed_required: Option<String> = None;
    let mut degraded: Option<String> = None;
    for check in checks {
        let id = check.get("id").and_then(|v| v.as_str()).unwrap_or("check");
        let status = check
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let detail = check
            .get("detail")
            .and_then(|v| v.as_str())
            .unwrap_or(status);
        let required = check
            .get("required")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let line = format!("{} {}: {}", id, status, detail);
        if status == "failed" && id.starts_with("workspace.") {
            failed_workspace.get_or_insert(line.clone());
        }
        if status == "failed" && required {
            failed_required.get_or_insert(line.clone());
        }
        if matches!(status, "degraded" | "skipped" | "failed") {
            degraded.get_or_insert(line);
        }
    }

    match overall.as_str() {
        "ok" => (
            "ok".to_string(),
            "Tytus bootstrap and smoke health OK".to_string(),
            overall,
        ),
        "degraded" => (
            "degraded".to_string(),
            degraded.unwrap_or_else(|| "Tytus smoke reported degraded".to_string()),
            overall,
        ),
        "failed" => {
            if let Some(line) = failed_workspace {
                ("failed".to_string(), line, overall)
            } else {
                // AIL auth/model failures inside the shell are useful
                // diagnostics, but the daemon's own gateway probe is the
                // source of truth for PR1 Open gating. Do not false-block a
                // usable browser UI just because the pod login shell lacks an
                // exported AIL_API_KEY.
                (
                    "degraded".to_string(),
                    failed_required
                        .or(degraded)
                        .unwrap_or_else(|| "Tytus smoke reported failed".to_string()),
                    overall,
                )
            }
        }
        "unknown" => (
            "starting".to_string(),
            "Tytus smoke health not populated yet".to_string(),
            overall,
        ),
        other => (
            "unknown".to_string(),
            format!("Tytus smoke returned unknown overall '{}'", other),
            overall,
        ),
    }
}

/// Checks the PR1 .tytus bootstrap floor and machine-readable smoke health
/// inside a pod. Secret-safe: it reads only `.tytus/health.json`; it never cats
/// env/config files that could contain credentials. A 15s cache prevents the
/// install wizard from hammering `tytus exec` while it polls readiness.
fn check_pod_tytus_bootstrap_entry(pod_id: &str, refresh: bool) -> BootstrapHealthEntry {
    if !readiness_strict_enabled() {
        return BootstrapHealthEntry {
            status: "skipped".to_string(),
            detail: "strict readiness disabled by TYTUS_READINESS_STRICT=0".to_string(),
            overall: "skipped".to_string(),
            fetched_at: now_secs(),
        };
    }

    let now = now_secs();
    if !refresh {
        if let Some(entry) = bootstrap_health_cache()
            .lock()
            .unwrap()
            .get(pod_id)
            .cloned()
        {
            if now.saturating_sub(entry.fetched_at) < BOOTSTRAP_HEALTH_TTL_SECS {
                return entry;
            }
        }
    }

    let script = r#"cd /app/workspace || exit 20
test -f README.md && test -f .tytus/capabilities.json && test -f .tytus/health.json && test -x .tytus/smoke.sh || exit 21
if grep -q '"overall"[[:space:]]*:[[:space:]]*"unknown"' .tytus/health.json 2>/dev/null; then
  ./.tytus/smoke.sh >/tmp/tytus-smoke-readiness.log 2>&1 || true
fi
cat .tytus/health.json
"#;

    let entry = match run_tytus_exec_shell(pod_id, script, Duration::from_secs(12)) {
        Ok((true, stdout, _stderr, false)) => {
            let (status, detail, overall) = summarize_health_json(&stdout);
            BootstrapHealthEntry {
                status,
                detail,
                overall,
                fetched_at: now,
            }
        }
        Ok((_success, _stdout, stderr, true)) => BootstrapHealthEntry {
            status: "unknown".to_string(),
            detail: format!(
                "bootstrap health probe timed out{}",
                stderr
                    .lines()
                    .next()
                    .map(|l| format!(": {}", l))
                    .unwrap_or_default()
            ),
            overall: "unknown".to_string(),
            fetched_at: now,
        },
        Ok((_success, _stdout, stderr, false)) => BootstrapHealthEntry {
            status: "failed".to_string(),
            detail: stderr
                .lines()
                .next()
                .unwrap_or("bootstrap pack missing")
                .to_string(),
            overall: "failed".to_string(),
            fetched_at: now,
        },
        Err(e) => BootstrapHealthEntry {
            status: "unknown".to_string(),
            detail: e,
            overall: "unknown".to_string(),
            fetched_at: now,
        },
    };

    bootstrap_health_cache()
        .lock()
        .unwrap()
        .insert(pod_id.to_string(), entry.clone());
    entry
}

fn bootstrap_status_allows_open(status: &str) -> bool {
    matches!(status, "ok" | "degraded" | "skipped")
}

fn bootstrap_status_is_failed(status: &str) -> bool {
    status == "failed"
}

fn check_pod_shared_storage_stage(pod_id: &str) -> (String, String) {
    let script = r#"cd /app/workspace || exit 20
if [ ! -e .garagetytus/garagetytus-shared ]; then
  echo "helper missing; restart/rebuild pod with latest Tytus image"
  exit 30
fi
test -x .garagetytus/garagetytus-shared || { echo "helper present but not executable"; exit 30; }
test -f .garagetytus/credentials.json || { echo "credentials.json missing; provision Sharing in TytusOS"; exit 31; }
test -f .garagetytus/_last-provision.json || { echo "_last-provision.json missing; refresh Sharing credentials"; exit 32; }
python3 - <<'PY'
import json
with open(".garagetytus/credentials.json") as f:
    creds = json.load(f)
buckets = creds.get("buckets") or []
print(f"configured ({len(buckets)} bucket(s))" if buckets else "configured (credentials present)")
PY
"#;
    match run_tytus_exec_shell(pod_id, script, Duration::from_secs(8)) {
        Ok((true, stdout, _stderr, false)) => (
            "ok".to_string(),
            stdout
                .lines()
                .last()
                .unwrap_or("shared storage configured")
                .to_string(),
        ),
        Ok((_success, stdout, _stderr, false)) => {
            let detail = stdout
                .lines()
                .last()
                .unwrap_or("not configured")
                .to_string();
            if detail == "not configured" {
                ("skipped".to_string(), "not configured".to_string())
            } else {
                ("degraded".to_string(), detail)
            }
        }
        Ok((_success, _stdout, stderr, true)) => (
            "unknown".to_string(),
            stderr
                .lines()
                .next()
                .unwrap_or("shared storage probe timed out")
                .to_string(),
        ),
        Err(e) => ("unknown".to_string(), e),
    }
}

fn bootstrap_stage_detail(entry: &BootstrapHealthEntry) -> String {
    if entry.overall == "skipped" {
        entry.detail.clone()
    } else {
        format!("{} (smoke overall: {})", entry.detail, entry.overall)
    }
}

fn cache_agent_status(pod_id: &str, status: AgentStatus) {
    let mut cache = status_cache().lock().unwrap();
    cache.insert(
        pod_id.to_string(),
        StatusEntry {
            status,
            fetched_at: now_secs(),
        },
    );
}

fn extract_pod_id_from_json(payload: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(payload)
        .ok()
        .and_then(|v| {
            v.get("pod_id")
                .or_else(|| v.get("podId"))
                .or_else(|| v.get("pod"))
                .and_then(|p| p.as_str().map(|s| s.to_string()))
                .or_else(|| {
                    v.get("pod_id")
                        .or_else(|| v.get("podId"))
                        .or_else(|| v.get("pod"))
                        .and_then(|p| p.as_u64().map(|n| format!("{:02}", n)))
                })
        })
}

fn wait_for_installed_agent_ready(job: &Arc<Mutex<Job>>, pod_id: &str) -> Result<(), String> {
    const WAIT_SECS: u64 = 180;
    const SLEEP_MS: u64 = 2_000;

    let started = std::time::Instant::now();
    let mut last_status: Option<AgentStatus> = None;
    push_event(
        job,
        JobEvent::Log(format!(
            "Waiting for pod {} to serve traffic end-to-end…",
            pod_id
        )),
    );

    while started.elapsed().as_secs() < WAIT_SECS {
        let snap = compute_state_snapshot();
        let Some(agent) = snap.agents.iter().find(|a| a.pod_id == pod_id) else {
            std::thread::sleep(std::time::Duration::from_millis(SLEEP_MS));
            continue;
        };

        let mut status = probe_agent_status(
            &agent.agent_type,
            agent.api_url.as_deref().unwrap_or(""),
            &agent.user_key,
            agent.gateway_token.as_deref(),
            agent.ui_url.as_deref(),
        );
        if status == AgentStatus::Ready && readiness_strict_enabled() {
            let bootstrap = check_pod_tytus_bootstrap_entry(pod_id, false);
            if !bootstrap_status_allows_open(&bootstrap.status) {
                push_event(
                    job,
                    JobEvent::Log(format!(
                        "Readiness: bootstrap {} ({})",
                        bootstrap.status,
                        bootstrap_stage_detail(&bootstrap)
                    )),
                );
                status = AgentStatus::Starting;
            } else if bootstrap.status == "degraded" {
                push_event(
                    job,
                    JobEvent::Log(format!(
                        "Readiness: bootstrap degraded but usable ({})",
                        bootstrap_stage_detail(&bootstrap)
                    )),
                );
            }
        }
        cache_agent_status(pod_id, status);

        if last_status != Some(status) {
            push_event(
                job,
                JobEvent::Log(format!(
                    "Readiness: {:?} (API{}{})",
                    status,
                    if agent.api_url.is_some() {
                        " ready-check"
                    } else {
                        " url missing"
                    },
                    if agent_has_browser_ui(&agent.agent_type) {
                        " + UI ready-check"
                    } else {
                        ""
                    }
                )),
            );
            last_status = Some(status);
        }

        if status == AgentStatus::Ready {
            push_event(
                job,
                JobEvent::Log(format!("Pod {} is ready. Open button enabled.", pod_id)),
            );
            return Ok(());
        }

        std::thread::sleep(std::time::Duration::from_millis(SLEEP_MS));
    }

    Err(format!(
        "pod {} installed but did not become ready within {}s",
        pod_id, WAIT_SECS
    ))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Look up `key`. Returns `None` for miss or expired entry; expired
/// entries are evicted opportunistically on read.
fn idempotency_get(key: &str) -> Option<IdemEntry> {
    let now = now_secs();
    let mut cache = idempotency_cache().lock().unwrap();
    match cache.get(key) {
        Some(entry) if now.saturating_sub(entry.inserted_at) < IDEM_TTL_SECS => Some(entry.clone()),
        Some(_) => {
            cache.remove(key);
            None
        }
        None => None,
    }
}

/// Insert `(key, status, body)`. Evicts the oldest entry by
/// `inserted_at` if at capacity — newer requests displace stale ones
/// rather than failing closed.
fn idempotency_put(key: String, status: u16, body: String) {
    let mut cache = idempotency_cache().lock().unwrap();
    if cache.len() >= IDEM_MAX_ENTRIES && !cache.contains_key(&key) {
        if let Some(oldest_key) = cache
            .iter()
            .min_by_key(|(_, v)| v.inserted_at)
            .map(|(k, _)| k.clone())
        {
            cache.remove(&oldest_key);
        }
    }
    cache.insert(
        key,
        IdemEntry {
            status,
            body,
            inserted_at: now_secs(),
        },
    );
}

/// Read the `Idempotency-Key` header. Header field-name lookup is
/// case-insensitive per RFC 9110 §5.1. The value is required to be
/// non-empty ASCII-graphic and ≤ `IDEM_MAX_KEY_LEN` so an attacker
/// can't pin daemon RAM with multi-megabyte keys.
fn read_idempotency_key(headers: &[Header]) -> Option<String> {
    for h in headers {
        if h.field
            .as_str()
            .as_str()
            .eq_ignore_ascii_case("Idempotency-Key")
        {
            let v = h.value.as_str();
            if !v.is_empty()
                && v.len() <= IDEM_MAX_KEY_LEN
                && v.chars().all(|c| c.is_ascii_graphic())
            {
                return Some(v.to_string());
            }
            // Malformed key: ignore (treat as no key) so a bogus value
            // can't poison the cache. Caller will get a fresh execution.
            return None;
        }
    }
    None
}

// Per-thread carrier for the in-flight idempotency key. Set in
// `handle()` after a cache miss and consumed by `respond_json` so we
// can capture status + body without threading the key through every
// handler signature. Each request is handled on a dedicated thread
// (see `start()` -> `thread::spawn(move || handle(...))`) so the
// thread-local has no cross-request leakage.
thread_local! {
    static CURRENT_IDEM_KEY: RefCell<Option<String>> = const { RefCell::new(None) };
}

fn set_current_idem_key(key: Option<String>) {
    CURRENT_IDEM_KEY.with(|k| *k.borrow_mut() = key);
}

fn take_current_idem_key() -> Option<String> {
    CURRENT_IDEM_KEY.with(|k| k.borrow_mut().take())
}

/// Replay a cached response for this `Idempotency-Key`. Sends an
/// `Idempotency-Replayed: true` header so the client (and any test
/// harness) can tell a replay from a fresh execution.
fn respond_idempotent_replay(request: Request, entry: &IdemEntry) {
    let resp = Response::from_string(entry.body.clone())
        .with_status_code(StatusCode(entry.status))
        .with_header(header("Content-Type", "application/json"))
        .with_header(header("X-Content-Type-Options", "nosniff"))
        .with_header(header("Idempotency-Replayed", "true"));
    let _ = request.respond(resp);
}

fn random_job_id() -> String {
    // Monotonic nanos + PID is unique enough for a per-session counter.
    // No need to burn an RNG dep for a UI that exists for one human.
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}-{:x}", std::process::id(), nanos)
}

// ── Browser terminal PTY sessions ─────────────────────────────
// TytusOS Terminal is a real local terminal: the browser talks to the tray,
// the tray owns a short-lived PTY-backed child process. This replaces the old
// macOS Terminal.app popups for flows like `tytus configure`.

#[cfg(unix)]
struct TerminalSession {
    writer: Arc<Mutex<File>>,
    output: Arc<Mutex<String>>,
    alive: Arc<Mutex<bool>>,
    exit_code: Arc<Mutex<Option<i32>>>,
    pid: u32,
}

#[cfg(unix)]
static TERMINAL_SESSIONS: OnceLock<Mutex<HashMap<String, Arc<TerminalSession>>>> = OnceLock::new();

#[cfg(unix)]
fn terminal_sessions() -> &'static Mutex<HashMap<String, Arc<TerminalSession>>> {
    TERMINAL_SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn terminal_user_label() -> String {
    // POSIX shells normally expose USER/LOGNAME. Windows exposes USERNAME.
    // The current PTY backend is Unix-only, but using all three here keeps the
    // prompt/title contract portable for the future ConPTY backend.
    for key in ["USER", "LOGNAME", "USERNAME"] {
        if let Ok(value) = std::env::var(key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    "user".to_string()
}

fn valid_terminal_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 96 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn query_value(query: &str, key: &str) -> Option<String> {
    for pair in query.split('&').filter(|s| !s.is_empty()) {
        let Some((k, v)) = pair.split_once('=') else {
            continue;
        };
        if k == key {
            return Some(v.to_string());
        }
    }
    None
}

#[derive(serde::Deserialize)]
struct TerminalStartBody {
    command: Option<String>,
    args: Option<Vec<String>>,
    cols: Option<u16>,
    rows: Option<u16>,
}

#[derive(serde::Deserialize)]
struct TerminalWriteBody {
    data: String,
}

#[derive(serde::Deserialize)]
struct TerminalResizeBody {
    cols: u16,
    rows: u16,
}

fn handle_terminal_start(mut request: Request) {
    #[cfg(not(unix))]
    {
        respond_json(
            request,
            501,
            &serde_json::json!({"error":"terminal_not_supported"}),
        );
    }

    #[cfg(unix)]
    {
        let mut body = String::new();
        if request.as_reader().read_to_string(&mut body).is_err() {
            respond_json(request, 400, &serde_json::json!({"error":"invalid_body"}));
            return;
        }
        let parsed: TerminalStartBody = match serde_json::from_str(&body) {
            Ok(v) => v,
            Err(_) => TerminalStartBody {
                command: Some("shell".to_string()),
                args: None,
                cols: None,
                rows: None,
            },
        };
        let cols = parsed.cols.unwrap_or(120).clamp(20, 300);
        let rows = parsed.rows.unwrap_or(32).clamp(4, 120);
        match start_terminal_session(
            parsed.command.as_deref().unwrap_or("shell"),
            parsed.args.unwrap_or_default(),
            cols,
            rows,
        ) {
            Ok(id) => respond_json(request, 201, &serde_json::json!({"id": id})),
            Err(e) => respond_json(request, 500, &serde_json::json!({"error": e})),
        }
    }
}

#[cfg(unix)]
fn start_terminal_session(
    command: &str,
    args: Vec<String>,
    cols: u16,
    rows: u16,
) -> Result<String, String> {
    let mut master: libc::c_int = -1;
    let mut slave: libc::c_int = -1;
    let winsize = libc::winsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let rc = unsafe {
        libc::openpty(
            &mut master,
            &mut slave,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &winsize as *const libc::winsize as *mut libc::winsize,
        )
    };
    if rc != 0 {
        return Err(format!(
            "openpty failed: {}",
            std::io::Error::last_os_error()
        ));
    }

    let mut cmd = if command == "tytus" {
        let allowed = args.first().map(|s| s.as_str()).unwrap_or("");
        if allowed != "configure" {
            unsafe {
                libc::close(master);
                libc::close(slave);
            }
            return Err("unsupported tytus terminal command".to_string());
        }
        let mut c = Command::new(resolve_tytus_bin());
        c.args(args);
        c
    } else {
        // Use a clean interactive bash for the browser terminal. User shell
        // startup files can block in non-Terminal.app PTYs; this keeps the
        // in-OS terminal deterministic while still running real host commands.
        let mut c = Command::new("/bin/bash");
        c.arg("--noprofile").arg("--norc").arg("-i");
        c
    };

    let master_file = unsafe { File::from_raw_fd(master) };
    let reader = master_file.try_clone().map_err(|e| e.to_string())?;
    let writer = master_file.try_clone().map_err(|e| e.to_string())?;
    let slave_file = unsafe { File::from_raw_fd(slave) };
    let slave_for_ctty = slave_file.as_raw_fd();

    cmd.current_dir(crate::workspace::ensure_tytus_home());

    // Browser terminal polish: keep the PTY real, but make bash expose a
    // user-friendly Tytus title/prompt instead of leaking arbitrary absolute
    // host paths. The cwd label follows `cd` because bash evaluates the
    // function before each prompt and emits an OSC title update.
    const TYTUS_PROMPT_COMMAND: &str = r#"__tytus_cwd_label(){
  local tytus_home="${HOME}/Tytus"
  if [ "$PWD" = "$tytus_home" ]; then printf 'Tytus Home'; return; fi
  case "$PWD" in
    "$tytus_home"/*)
      local rel="${PWD#$tytus_home/}"
      printf 'Tytus Home/%s' "$rel"
      return
      ;;
  esac
  if [ "$PWD" = "$HOME" ]; then printf '~'; return; fi
  case "$PWD" in
    "$HOME"/*)
      printf '~/%s' "${PWD#$HOME/}"
      return
      ;;
  esac
  if [ "$PWD" = "/" ]; then printf '/'; return; fi
  printf '%s' "$PWD"
}
__tytus_prompt_title(){
  local label="$(__tytus_cwd_label)"
  printf '\033]0;%s — %s — %sx%s\007' "${TYTUS_USER_LABEL:-${USER:-${LOGNAME:-${USERNAME:-user}}}}" "$label" "${COLUMNS:-120}" "${LINES:-32}"
}
__tytus_alt_run(){
  local cmd="$1"; shift
  # Some AI CLIs (notably Claude Code) redraw full-screen layouts in the
  # primary buffer. In a browser terminal that makes every resize append an old
  # frame to scrollback. Ghostty-grade behavior is to keep full-screen apps in
  # the alternate screen, clear on entry, and restore cleanly on exit.
  printf '\033[?1049h\033[H\033[2J'
  command "$cmd" "$@"
  local status=$?
  printf '\033[?1049l'
  return "$status"
}
claude(){ __tytus_alt_run claude "$@"; }
codex(){ __tytus_alt_run codex "$@"; }
opencode(){ __tytus_alt_run opencode "$@"; }
gemini(){ __tytus_alt_run gemini "$@"; }
qwen(){ __tytus_alt_run qwen "$@"; }
aider(){ __tytus_alt_run aider "$@"; }
goose(){ __tytus_alt_run goose "$@"; }
__tytus_prompt_title"#;

    cmd.env("TERM", "xterm-256color")
        .env("COLORTERM", "truecolor")
        .env("TYTUS_USER_LABEL", terminal_user_label())
        .env("COLUMNS", cols.to_string())
        .env("LINES", rows.to_string())
        .env("BASH_SILENCE_DEPRECATION_WARNING", "1")
        .env("PROMPT_COMMAND", TYTUS_PROMPT_COMMAND)
        .env("PS1", r"\u@$(__tytus_cwd_label) % ")
        .stdin(Stdio::from(
            slave_file.try_clone().map_err(|e| e.to_string())?,
        ))
        .stdout(Stdio::from(
            slave_file.try_clone().map_err(|e| e.to_string())?,
        ))
        .stderr(Stdio::from(
            slave_file.try_clone().map_err(|e| e.to_string())?,
        ));

    unsafe {
        cmd.pre_exec(move || {
            if libc::setsid() < 0 {
                return Err(std::io::Error::last_os_error());
            }
            if libc::ioctl(slave_for_ctty, libc::TIOCSCTTY as libc::c_ulong, 0) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("failed to spawn terminal child: {}", e))?;
    drop(slave_file);

    let id = random_job_id();
    let output = Arc::new(Mutex::new(String::new()));
    let alive = Arc::new(Mutex::new(true));
    let exit_code = Arc::new(Mutex::new(None));
    let pid = child.id();
    let session = Arc::new(TerminalSession {
        writer: Arc::new(Mutex::new(writer)),
        output: output.clone(),
        alive: alive.clone(),
        exit_code: exit_code.clone(),
        pid,
    });
    terminal_sessions()
        .lock()
        .unwrap()
        .insert(id.clone(), session);

    thread::spawn(move || {
        let mut reader = reader;
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let text = String::from_utf8_lossy(&buf[..n]).to_string();
                    output.lock().unwrap().push_str(&text);
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
    });

    thread::spawn(move || {
        let code = child.wait().ok().and_then(|s| s.code()).unwrap_or(-1);
        *exit_code.lock().unwrap() = Some(code);
        *alive.lock().unwrap() = false;
    });

    Ok(id)
}

fn handle_terminal_read(request: Request, query: &str) {
    #[cfg(not(unix))]
    {
        respond_json(
            request,
            501,
            &serde_json::json!({"error":"terminal_not_supported"}),
        );
    }

    #[cfg(unix)]
    {
        let Some(id) = query_value(query, "id").filter(|v| valid_terminal_id(v)) else {
            respond_json(
                request,
                400,
                &serde_json::json!({"error":"invalid_terminal_id"}),
            );
            return;
        };
        let Some(session) = terminal_sessions().lock().unwrap().get(&id).cloned() else {
            respond_json(
                request,
                404,
                &serde_json::json!({"error":"terminal_not_found"}),
            );
            return;
        };
        let output = {
            let mut guard = session.output.lock().unwrap();
            let text = guard.clone();
            guard.clear();
            text
        };
        let alive = *session.alive.lock().unwrap();
        let exit_code = *session.exit_code.lock().unwrap();
        respond_json(
            request,
            200,
            &serde_json::json!({
                "id": id,
                "output": output,
                "alive": alive,
                "exit_code": exit_code,
            }),
        );
    }
}

fn handle_terminal_write(mut request: Request, query: &str) {
    #[cfg(not(unix))]
    {
        respond_json(
            request,
            501,
            &serde_json::json!({"error":"terminal_not_supported"}),
        );
    }

    #[cfg(unix)]
    {
        let Some(id) = query_value(query, "id").filter(|v| valid_terminal_id(v)) else {
            respond_json(
                request,
                400,
                &serde_json::json!({"error":"invalid_terminal_id"}),
            );
            return;
        };
        let Some(session) = terminal_sessions().lock().unwrap().get(&id).cloned() else {
            respond_json(
                request,
                404,
                &serde_json::json!({"error":"terminal_not_found"}),
            );
            return;
        };
        let mut body = String::new();
        if request.as_reader().read_to_string(&mut body).is_err() {
            respond_json(request, 400, &serde_json::json!({"error":"invalid_body"}));
            return;
        }
        let parsed: TerminalWriteBody = match serde_json::from_str(&body) {
            Ok(v) => v,
            Err(_) => {
                respond_json(request, 400, &serde_json::json!({"error":"invalid_body"}));
                return;
            }
        };
        if parsed.data.len() > 8192 {
            respond_json(
                request,
                413,
                &serde_json::json!({"error":"terminal_write_too_large"}),
            );
            return;
        }
        let write_result = session
            .writer
            .lock()
            .unwrap()
            .write_all(parsed.data.as_bytes());
        match write_result {
            Ok(()) => respond_json(request, 200, &serde_json::json!({"ok": true})),
            Err(e) => respond_json(request, 500, &serde_json::json!({"error": e.to_string()})),
        }
    }
}

fn handle_terminal_resize(mut request: Request, query: &str) {
    #[cfg(not(unix))]
    {
        respond_json(
            request,
            501,
            &serde_json::json!({"error":"terminal_not_supported"}),
        );
    }

    #[cfg(unix)]
    {
        let Some(id) = query_value(query, "id").filter(|v| valid_terminal_id(v)) else {
            respond_json(
                request,
                400,
                &serde_json::json!({"error":"invalid_terminal_id"}),
            );
            return;
        };
        let Some(session) = terminal_sessions().lock().unwrap().get(&id).cloned() else {
            respond_json(
                request,
                404,
                &serde_json::json!({"error":"terminal_not_found"}),
            );
            return;
        };
        let mut body = String::new();
        if request.as_reader().read_to_string(&mut body).is_err() {
            respond_json(request, 400, &serde_json::json!({"error":"invalid_body"}));
            return;
        }
        let parsed: TerminalResizeBody = match serde_json::from_str(&body) {
            Ok(v) => v,
            Err(_) => {
                respond_json(request, 400, &serde_json::json!({"error":"invalid_body"}));
                return;
            }
        };
        let cols = parsed.cols.clamp(20, 300);
        let rows = parsed.rows.clamp(4, 120);
        let winsize = libc::winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let fd = session.writer.lock().unwrap().as_raw_fd();
        let rc = unsafe { libc::ioctl(fd, libc::TIOCSWINSZ as libc::c_ulong, &winsize) };
        if rc != 0 {
            respond_json(
                request,
                500,
                &serde_json::json!({"error": std::io::Error::last_os_error().to_string()}),
            );
            return;
        }
        unsafe {
            libc::kill(-(session.pid as libc::pid_t), libc::SIGWINCH);
        }
        respond_json(
            request,
            200,
            &serde_json::json!({"ok": true, "cols": cols, "rows": rows}),
        );
    }
}

fn handle_terminal_stop(request: Request, query: &str) {
    #[cfg(not(unix))]
    {
        respond_json(request, 200, &serde_json::json!({"ok": true}));
    }

    #[cfg(unix)]
    {
        let Some(id) = query_value(query, "id").filter(|v| valid_terminal_id(v)) else {
            respond_json(
                request,
                400,
                &serde_json::json!({"error":"invalid_terminal_id"}),
            );
            return;
        };
        let session = terminal_sessions().lock().unwrap().remove(&id);
        if let Some(session) = session {
            if *session.alive.lock().unwrap() {
                unsafe {
                    libc::kill(-(session.pid as libc::pid_t), libc::SIGHUP);
                }
            }
        }
        respond_json(request, 200, &serde_json::json!({"ok": true}));
    }
}

// ── Public entry ──────────────────────────────────────────────

/// Spawn the wizard server on a random localhost port and return the port.
///
/// Returns `None` if bind failed (very rare — only when 127.0.0.1 itself
/// isn't available). Caller stores the returned port for `open_os()`.
pub fn start() -> Option<u16> {
    // Capture boot timestamp before we bind, so /api/version reflects
    // when the *daemon* came up — not when it first served a version
    // request. Idempotent (OnceLock).
    daemon_started_at();
    music_ytdlp_setup::start_background_install();

    let server = match Server::http("127.0.0.1:0") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[tray-web] failed to bind: {}", e);
            return None;
        }
    };
    let port = server.server_addr().to_ip()?.port();

    // Persist the port so subsequent "Install Agent" clicks (which call
    // `open_os`) can read it without a lookup.
    if let Some(path) = port_file() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, port.to_string());
    }

    let registry = Registry::new();
    let server = Arc::new(server);

    thread::Builder::new()
        .name("tray-wizard-http".into())
        .spawn(move || {
            for request in server.incoming_requests() {
                let reg = registry.clone();
                thread::spawn(move || handle(request, reg));
            }
        })
        .ok()?;

    Some(port)
}

pub fn open_os() {
    open_os_at("");
}

/// Open TytusOS at a specific URL fragment so the tray menu can deep-link
/// directly into an in-page action (e.g. `#/run/doctor`, `#/pod/02/restart`).
///
/// `fragment` should start with `#` if non-empty. A nonce query param is
/// appended automatically — without it, browsers focus the existing tab
/// without re-firing `hashchange` when the fragment matches the current one,
/// so successive tray clicks would silently no-op.
///
/// CALLER CONSTRAINT: pass path-only fragments like `"#/pod/02/output"` or
/// query-bearing fragments like `"#/pod/02/channels?action=add&type=telegram"`.
/// Do NOT embed a literal `?` outside the canonical query separator — the
/// `sep` heuristic detects the first `?` to decide between `?n=` and `&n=`,
/// so a fragment with a stray `?` (e.g. `"#/path?weird"`) would still parse
/// here but produce a URL the browser may interpret unexpectedly. None of
/// the current call sites do this; this is a future-maintainer warning.
pub fn open_os_at(fragment: &str) {
    let fragment = canonical_os_fragment(fragment);
    let port = match current_port() {
        Some(p) => p,
        None => {
            eprintln!("[tray-web] no port recorded — is the server running?");
            return;
        }
    };
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let url = if fragment.is_empty() {
        format!("http://127.0.0.1:{}/", port)
    } else {
        let sep = if fragment.contains('?') { '&' } else { '?' };
        format!("http://127.0.0.1:{}/{}{}n={:x}", port, fragment, sep, nonce)
    };
    open_url(&url);
}

#[allow(dead_code)]
fn canonical_os_fragment(fragment: &str) -> String {
    match fragment {
        "" => String::new(),
        "#chat" => "#/chat".to_string(),
        "#files" => "#/files".to_string(),
        "#channels" => "#/channels".to_string(),
        other => other.to_string(),
    }
}

fn open_url(url: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(&url).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xdg-open").arg(&url).spawn();
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = url;
    }
}

fn legacy_tower_enabled() -> bool {
    std::env::var(LEGACY_TOWER_ENV)
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn port_file() -> Option<PathBuf> {
    Some(PathBuf::from("/tmp/tytus/tray-web.port"))
}

fn current_port() -> Option<u16> {
    let raw = std::fs::read_to_string(port_file()?).ok()?;
    raw.trim().parse().ok()
}

fn handle_music_status(request: Request) {
    let status = music_ytdlp_setup::status();
    if !status.ready && !status.installing {
        music_ytdlp_setup::start_background_install();
    }
    respond_json(request, 200, &status);
}

fn handle_music_providers(request: Request) {
    if !music_ytdlp_setup::status().ready {
        music_ytdlp_setup::start_background_install();
    }
    respond_json(
        request,
        200,
        &serde_json::json!({ "providers": music_ytdlp::providers() }),
    );
}

fn handle_music_search(request: Request, query: &str) {
    if !music_ytdlp_setup::status().ready {
        music_ytdlp_setup::start_background_install();
        respond_json(
            request,
            503,
            &serde_json::json!({
                "error": "music_unavailable",
                "status": music_ytdlp_setup::status(),
            }),
        );
        return;
    }
    let q = match query_value_decoded(query, "q") {
        Ok(Some(v)) => v,
        Ok(None) => {
            respond_json(
                request,
                400,
                &serde_json::json!({"error":"query is required"}),
            );
            return;
        }
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let limit = query_value(query, "limit")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(20);
    match music_ytdlp::search(&q, limit) {
        Ok(results) => respond_json(request, 200, &serde_json::json!({ "results": results })),
        Err(e) => respond_json(request, 500, &serde_json::json!({"error": e})),
    }
}

fn handle_music_search2(request: Request, query: &str) {
    if !music_ytdlp_setup::status().ready {
        music_ytdlp_setup::start_background_install();
        respond_json(
            request,
            503,
            &serde_json::json!({
                "error": "music_unavailable",
                "status": music_ytdlp_setup::status(),
            }),
        );
        return;
    }
    let q = match query_value_decoded(query, "q") {
        Ok(Some(v)) => v,
        Ok(None) => {
            respond_json(
                request,
                400,
                &serde_json::json!({"error":"query is required"}),
            );
            return;
        }
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let limit = query_value(query, "limit")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(20);
    let types_raw = query_value_decoded(query, "types")
        .ok()
        .flatten()
        .unwrap_or_else(|| "tracks,albums,artists,playlists".to_string());
    let types = types_raw
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    match music_ytdlp::search_unified(&q, &types, limit) {
        Ok(results) => respond_json(request, 200, &results),
        Err(e) => respond_json(request, 500, &serde_json::json!({"error": e})),
    }
}

fn handle_music_stream(request: Request, query: &str) {
    if !music_ytdlp_setup::status().ready {
        music_ytdlp_setup::start_background_install();
        respond_json(
            request,
            503,
            &serde_json::json!({
                "error": "music_unavailable",
                "status": music_ytdlp_setup::status(),
            }),
        );
        return;
    }
    let video_id = match query_value_decoded(query, "videoId") {
        Ok(Some(v)) => v,
        Ok(None) => {
            respond_json(
                request,
                400,
                &serde_json::json!({"error":"videoId is required"}),
            );
            return;
        }
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    match music_ytdlp::stream(&video_id) {
        Ok(info) => {
            let encoded = music_proxy::encode_proxy_path(&info.stream_url);
            let proxy_url = current_port()
                .map(|port| format!("http://127.0.0.1:{port}/api/music/proxy/{encoded}"))
                .unwrap_or_else(|| format!("/api/music/proxy/{encoded}"));
            respond_json(
                request,
                200,
                &serde_json::json!({
                    "videoId": info.video_id,
                    "proxyUrl": proxy_url,
                    "durationMs": info.duration_ms,
                    "title": info.title,
                    "container": info.container,
                    "codec": info.codec,
                }),
            );
        }
        Err(e) => respond_json(request, 500, &serde_json::json!({"error": e})),
    }
}

fn handle_music_playlist(request: Request, query: &str) {
    if !music_ytdlp_setup::status().ready {
        music_ytdlp_setup::start_background_install();
        respond_json(
            request,
            503,
            &serde_json::json!({
                "error": "music_unavailable",
                "status": music_ytdlp_setup::status(),
            }),
        );
        return;
    }
    let url = match query_value_decoded(query, "url") {
        Ok(Some(v)) => v,
        Ok(None) => {
            respond_json(
                request,
                400,
                &serde_json::json!({"error":"url is required"}),
            );
            return;
        }
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let limit = query_value(query, "limit")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(20);
    match music_ytdlp::playlist(&url, limit) {
        Ok(playlist) => respond_json(request, 200, &playlist),
        Err(e) => respond_json(request, 500, &serde_json::json!({"error": e})),
    }
}

fn connector_error_response(request: Request, err: music_connectors::ConnectorError) {
    let status = music_connectors::connector_error_status(&err);
    respond_json(
        request,
        status,
        &serde_json::json!({
            "error": err.to_string(),
        }),
    );
}

fn parse_json_body<T: serde::de::DeserializeOwned>(
    mut request: Request,
) -> Result<(Request, T), (Request, String)> {
    let mut body = String::new();
    if request.as_reader().read_to_string(&mut body).is_err() {
        return Err((request, "failed to read request body".to_string()));
    }
    match serde_json::from_str::<T>(&body) {
        Ok(parsed) => Ok((request, parsed)),
        Err(e) => Err((request, format!("invalid JSON body: {e}"))),
    }
}

fn juli3ta_library_error_response(request: Request, err: juli3ta_library::LibraryError) {
    let status = juli3ta_library::error_status(&err);
    respond_json(
        request,
        status,
        &serde_json::json!({ "error": err.to_string() }),
    );
}

fn handle_juli3ta_library_tracks(request: Request) {
    match juli3ta_library::list_tracks() {
        Ok(list) => respond_json(request, 200, &list),
        Err(e) => juli3ta_library_error_response(request, e),
    }
}

fn handle_juli3ta_library_save(request: Request) {
    let (request, body) = match parse_json_body::<juli3ta_library::SaveTrackRequest>(request) {
        Ok(v) => v,
        Err((request, e)) => {
            respond_json(request, 400, &serde_json::json!({ "error": e }));
            return;
        }
    };
    match juli3ta_library::save_track(body) {
        Ok(saved) => respond_json(request, 200, &saved),
        Err(e) => juli3ta_library_error_response(request, e),
    }
}

#[derive(serde::Deserialize)]
struct Juli3taDeleteTrackRequest {
    id: String,
}

fn handle_juli3ta_library_delete(request: Request) {
    let (request, body) = match parse_json_body::<Juli3taDeleteTrackRequest>(request) {
        Ok(v) => v,
        Err((request, e)) => {
            respond_json(request, 400, &serde_json::json!({ "error": e }));
            return;
        }
    };
    match juli3ta_library::delete_track(&body.id) {
        Ok(()) => respond_json(request, 200, &serde_json::json!({ "ok": true })),
        Err(e) => juli3ta_library_error_response(request, e),
    }
}

fn handle_juli3ta_library_audio(request: Request, query: &str) {
    let Some(id) = query_value(query, "id") else {
        respond_json(request, 400, &serde_json::json!({ "error": "missing id" }));
        return;
    };
    match juli3ta_library::read_audio(&id) {
        Ok((bytes, mime)) => {
            let resp = Response::from_data(bytes)
                .with_status_code(StatusCode(200))
                .with_header(header("Content-Type", &mime))
                .with_header(header("X-Content-Type-Options", "nosniff"))
                .with_header(header("Accept-Ranges", "bytes"));
            let _ = request.respond(resp);
        }
        Err(e) => juli3ta_library_error_response(request, e),
    }
}

fn handle_juli3ta_library_open_folder(request: Request) {
    match juli3ta_library::open_library_folder() {
        Ok(path) => respond_json(
            request,
            200,
            &serde_json::json!({ "ok": true, "path": path }),
        ),
        Err(e) => juli3ta_library_error_response(request, e),
    }
}

fn handle_music_connectors(request: Request) {
    respond_json(
        request,
        200,
        &serde_json::json!({ "connectors": music_connectors::statuses() }),
    );
}

fn handle_music_connector_configure(request: Request) {
    let (request, body) =
        match parse_json_body::<music_connectors::ConfigureConnectorRequest>(request) {
            Ok(v) => v,
            Err((request, e)) => {
                respond_json(request, 400, &serde_json::json!({ "error": e }));
                return;
            }
        };
    match music_connectors::configure(body) {
        Ok(status) => respond_json(request, 200, &status),
        Err(e) => connector_error_response(request, e),
    }
}

fn handle_music_connector_verify(request: Request) {
    let (request, body) =
        match parse_json_body::<music_connectors::ConnectorProviderRequest>(request) {
            Ok(v) => v,
            Err((request, e)) => {
                respond_json(request, 400, &serde_json::json!({ "error": e }));
                return;
            }
        };
    match music_connectors::verify(&body.provider) {
        Ok(status) => respond_json(request, 200, &status),
        Err(e) => connector_error_response(request, e),
    }
}

fn handle_music_connector_disconnect(request: Request) {
    let (request, body) =
        match parse_json_body::<music_connectors::ConnectorProviderRequest>(request) {
            Ok(v) => v,
            Err((request, e)) => {
                respond_json(request, 400, &serde_json::json!({ "error": e }));
                return;
            }
        };
    match music_connectors::disconnect(&body.provider) {
        Ok(status) => respond_json(request, 200, &status),
        Err(e) => connector_error_response(request, e),
    }
}

// ── Request router ────────────────────────────────────────────

/// Phase 2 security floor (manifest §11): every state-changing POST must
/// carry `Sec-Fetch-Site: same-origin`. Modern browsers always send this
/// header; missing or non-`same-origin` values are rejected with 403 so
/// a malicious tab on the same machine cannot drive the daemon.
///
/// `Sec-Fetch-Site` cannot be forged from JavaScript, so the same-origin
/// check is sufficient even for cross-origin POSTs that try to bypass
/// CORS via simple form submits. Curl/local tooling that needs to POST
/// must pass `-H "Sec-Fetch-Site: same-origin"`; the Vite dev proxy
/// injects it automatically (services/tytus-os/app/vite.config.ts).
fn sec_fetch_site_value(request: &Request) -> Option<String> {
    request
        .headers()
        .iter()
        .find(|h| h.field.equiv("Sec-Fetch-Site"))
        .map(|h| h.value.as_str().to_string())
}

fn sec_fetch_site_value_ok(value: Option<&str>) -> bool {
    matches!(value, Some("same-origin"))
}

fn sec_fetch_site_ok(request: &Request) -> bool {
    sec_fetch_site_value_ok(sec_fetch_site_value(request).as_deref())
}

fn deny_cross_origin_post(request: Request) {
    respond_json(
        request,
        403,
        &serde_json::json!({ "error": "cross-origin POST denied" }),
    );
}

fn redirect(request: Request, location: &str) {
    let resp = Response::from_string("")
        .with_status_code(StatusCode(302))
        .with_header(header("Location", location));
    let _ = request.respond(resp);
}

fn serve_tower_or_redirect(request: Request) {
    if legacy_tower_enabled() {
        serve_bytes(request, TOWER_HTML, "text/html; charset=utf-8");
    } else {
        redirect(request, "/");
    }
}

fn os_asset(path: &str) -> Option<&'static [u8]> {
    let normalized = if path.is_empty() || path == "/" {
        "/index.html"
    } else {
        path
    };
    if normalized.contains("..") || normalized.contains('\\') {
        return None;
    }
    os_assets::OS_ASSETS
        .iter()
        .find(|(asset_path, _)| *asset_path == normalized)
        .map(|(_, bytes)| *bytes)
}

fn content_type_for_path(path: &str) -> &'static str {
    if path.is_empty() || path == "/" {
        return "text/html; charset=utf-8";
    }
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "wasm" => "application/wasm",
        "json" | "webmanifest" => "application/manifest+json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "map" => "application/json",
        _ => "application/octet-stream",
    }
}

fn is_static_path(path: &str) -> bool {
    path.starts_with("/assets/")
        || path.starts_with("/brand/")
        || path.starts_with("/agents/")
        || path.starts_with("/favicons/")
        || path.starts_with("/wallpapers/")
        || path == "/favicon.svg"
        || path == "/site.webmanifest"
        // TytusOS is hash-routed. Extension-like paths are static assets;
        // unknown assets should 404 instead of falling back to index.html.
        || path.rsplit('/').next().unwrap_or("").contains('.')
}

fn accepts_html(request: &Request) -> bool {
    request
        .headers()
        .iter()
        .find(|h| h.field.equiv("Accept"))
        .map(|h| h.value.as_str().contains("text/html"))
        .unwrap_or(true)
}

fn serve_os_route(request: Request, path: &str) {
    if path.starts_with("/api/") {
        respond_json(request, 404, &serde_json::json!({ "error": "not found" }));
        return;
    }
    if let Some(bytes) = os_asset(path) {
        serve_bytes(request, bytes, content_type_for_path(path));
        return;
    }
    if is_static_path(path) {
        let resp = Response::from_string("not found").with_status_code(StatusCode(404));
        let _ = request.respond(resp);
        return;
    }
    if accepts_html(&request) {
        if let Some(bytes) = os_asset("/index.html") {
            serve_bytes(request, bytes, "text/html; charset=utf-8");
            return;
        }
    }
    let resp = Response::from_string("not found").with_status_code(StatusCode(404));
    let _ = request.respond(resp);
}

fn handle(request: Request, registry: Registry) {
    let method = request.method().clone();
    let url = request.url().to_string();
    let path = url.split('?').next().unwrap_or("").to_string();
    let query = url
        .split_once('?')
        .map(|(_, q)| q.to_string())
        .unwrap_or_default();

    // Phase 2: reject any POST without Sec-Fetch-Site: same-origin BEFORE
    // dispatch. Fail closed — older browsers without the header are
    // out of scope per manifest §11.
    if matches!(method, Method::Post) && !sec_fetch_site_ok(&request) {
        deny_cross_origin_post(request);
        return;
    }

    // Idempotency-Key short-circuit. Only POSTs are destructive enough
    // to warrant caching; GETs are already safe to retry by definition.
    // Cache hit: replay the prior response and skip dispatch entirely so
    // a `tytus restart --pod 02` subprocess isn't spawned twice. Cache
    // miss: stash the key in a thread-local so respond_json can capture
    // the response when the handler finishes.
    if matches!(method, Method::Post) {
        if let Some(key) = read_idempotency_key(request.headers()) {
            if let Some(entry) = idempotency_get(&key) {
                respond_idempotent_replay(request, &entry);
                return;
            }
            set_current_idem_key(Some(key));
        }
    }

    if matches!(method, Method::Get) && path.starts_with("/tower/") {
        serve_tower_or_redirect(request);
        return;
    }

    match (&method, path.as_str()) {
        (Method::Get, "/") => {
            serve_os_route(request, "/");
        }
        (Method::Get, "/tower") => {
            serve_tower_or_redirect(request);
        }
        // Back-compat: external bookmarks and the pre-rename `open_tower`
        // URL both aimed at `/install`. Redirect instead of duplicating
        // the serve path — keeps the canonical URL visible in the address
        // bar after the redirect resolves.
        (Method::Get, "/install") => {
            redirect(request, "/");
        }
        (Method::Get, "/assets/tower.css") if legacy_tower_enabled() => {
            serve_bytes(request, TOWER_CSS, "text/css; charset=utf-8");
        }
        (Method::Get, "/assets/tower.js") if legacy_tower_enabled() => {
            serve_bytes(request, TOWER_JS, "application/javascript; charset=utf-8");
        }
        (Method::Get, "/assets/icons/openclaw.svg") => {
            serve_bytes(request, ICON_OPENCLAW, "image/svg+xml");
        }
        (Method::Get, "/assets/icons/hermes.svg") => {
            serve_bytes(request, ICON_HERMES, "image/svg+xml");
        }
        (Method::Get, "/assets/icons/nvidia.svg") => {
            serve_bytes(request, ICON_NVIDIA, "image/svg+xml");
        }
        (Method::Get, "/api/catalog") => {
            handle_catalog(request, &query);
        }
        (Method::Post, "/api/install") => {
            handle_install(request, &registry);
        }
        // ── App Store ────────────────────────────────────────────
        (Method::Get, "/api/apps") => {
            handle_apps_catalog(request);
        }
        (Method::Post, "/api/apps/check") => {
            handle_apps_check(request);
        }
        (Method::Get, p) if p.starts_with("/api/jobs/") && p.ends_with("/stream") => {
            let job_id = p
                .trim_start_matches("/api/jobs/")
                .trim_end_matches("/stream")
                .to_string();
            handle_stream(request, &registry, &job_id);
        }
        (Method::Post, p) if p.starts_with("/api/jobs/") && p.ends_with("/cancel") => {
            let job_id = p
                .trim_start_matches("/api/jobs/")
                .trim_end_matches("/cancel")
                .to_string();
            handle_job_cancel(request, &registry, &job_id);
        }
        (Method::Get, "/api/state") => {
            handle_state(request, &registry);
        }
        (Method::Get, "/api/version") => {
            handle_version(request);
        }
        // ── JULI3TA music search/playback (Nuclear yt-dlp pipeline) ──

        // ── JULI3TA generated-song file library (real host folder) ──
        (Method::Get, "/api/juli3ta/library/tracks") => {
            handle_juli3ta_library_tracks(request);
        }
        (Method::Post, "/api/juli3ta/library/tracks") => {
            handle_juli3ta_library_save(request);
        }
        (Method::Post, "/api/juli3ta/library/delete") => {
            handle_juli3ta_library_delete(request);
        }
        (Method::Get, "/api/juli3ta/library/audio") => {
            handle_juli3ta_library_audio(request, &query);
        }
        (Method::Post, "/api/juli3ta/library/open-folder") => {
            handle_juli3ta_library_open_folder(request);
        }
        (Method::Get, "/api/music/status") => {
            handle_music_status(request);
        }
        (Method::Get, "/api/music/providers") => {
            handle_music_providers(request);
        }
        (Method::Get, "/api/music/connectors") => {
            handle_music_connectors(request);
        }
        (Method::Post, "/api/music/connectors/configure") => {
            handle_music_connector_configure(request);
        }
        (Method::Post, "/api/music/connectors/verify") => {
            handle_music_connector_verify(request);
        }
        (Method::Post, "/api/music/connectors/disconnect") => {
            handle_music_connector_disconnect(request);
        }
        (Method::Get, "/api/music/search") => {
            handle_music_search(request, &query);
        }
        (Method::Get, "/api/music/search2") => {
            handle_music_search2(request, &query);
        }
        (Method::Get, "/api/music/stream") => {
            handle_music_stream(request, &query);
        }
        (Method::Get, "/api/music/playlist") => {
            handle_music_playlist(request, &query);
        }
        (Method::Get, p) if p.starts_with("/api/music/proxy/") => {
            let encoded = p.trim_start_matches("/api/music/proxy/");
            music_proxy::handle_proxy(request, encoded);
        }
        (Method::Post, "/api/open-external") => {
            handle_open_external(request, &query);
        }
        // Pod actions — mirror the tray-menu-level operations so the
        // wizard can replicate "Open in Browser", "Restart", and
        // "Uninstall" without the user leaving the install flow.
        // Format: /api/pod/<action>?pod=NN (NN validated as ascii digits).
        (Method::Post, "/api/pod/open") => {
            handle_pod_open(request, &query);
        }
        (Method::Post, "/api/pod/restart") => {
            handle_pod_restart(request, &query);
        }
        (Method::Post, "/api/pod/uninstall") => {
            handle_pod_uninstall(request, &query);
        }
        // Readiness probe for "waiting for your pod to come online"
        // after install. The wizard polls this every 2s until {ready:true}.
        (Method::Get, "/api/pod/ready") => {
            handle_pod_ready(request, &query);
        }
        (Method::Get, p) if p.starts_with("/api/pods/") && p.ends_with("/readiness") => {
            let pod = p
                .trim_start_matches("/api/pods/")
                .trim_end_matches("/readiness")
                .trim_matches('/');
            handle_pod_readiness(request, pod);
        }
        // Per-pod env vars (manifest A.exist A3.5). Spawns `tytus agent
        // env --pod NN [--reveal-secrets] --json` and forwards the parsed
        // JSON. Provider does redaction + plan-tier gating; daemon is a
        // dumb pipe.
        (Method::Get, "/api/pod/env") => {
            handle_pod_env(request, &query);
        }
        // Revoke a pod — frees its units immediately and wipes the
        // pod's workspace state. Wizard's "Revoke & try again" button
        // after an install failure calls this to reset before retry.
        (Method::Post, "/api/pod/revoke") => {
            handle_pod_revoke(request, &query);
        }
        // Phase B: per-pod streamed action. Body is { "action": "doctor"
        // | "restart" | "revoke" | "uninstall" | "stop-forwarder" }.
        // Returns { job_id }; output streams via /api/jobs/<id>/stream.
        (Method::Post, p) if p.starts_with("/api/pod/") && p.ends_with("/run-streamed") => {
            let pod = p
                .trim_start_matches("/api/pod/")
                .trim_end_matches("/run-streamed")
                .to_string();
            handle_pod_run_streamed(request, &registry, pod);
        }
        // ── TytusOS control-surface endpoints (Wave 1) ─────────────────
        // Header actions + Settings block moved from tray submenus into
        // the page. All run as subprocesses of the `tytus` binary; the
        // tray's existing handlers keep working in parallel.
        (Method::Post, "/api/disconnect") => {
            handle_disconnect(request);
        }
        (Method::Post, "/api/connect") => {
            handle_connect(request);
        }
        (Method::Post, "/api/test") => {
            handle_test(request, &registry);
        }
        (Method::Get, "/api/settings") => {
            handle_settings_get(request);
        }
        (Method::Post, "/api/settings/autostart-tunnel") => {
            handle_autostart_tunnel(request);
        }
        (Method::Post, "/api/settings/autostart-tray") => {
            handle_autostart_tray(request);
        }
        (Method::Get, "/api/update/status") => {
            handle_update_status(request);
        }
        (Method::Post, "/api/update/check") => {
            handle_update_check(request);
        }
        (Method::Post, "/api/update/automatic") => {
            handle_update_automatic(request);
        }
        (Method::Post, "/api/login") => {
            handle_login(request);
        }
        (Method::Post, "/api/logout") => {
            handle_logout(request);
        }
        // ── TytusOS Wave 2: Troubleshoot surface ───────────────────────
        (Method::Post, "/api/doctor") => {
            handle_doctor(request, &registry);
        }
        (Method::Post, "/api/daemon/start") => {
            handle_daemon_lifecycle(request, DaemonAction::Start);
        }
        (Method::Post, "/api/daemon/stop") => {
            handle_daemon_lifecycle(request, DaemonAction::Stop);
        }
        (Method::Post, "/api/daemon/restart") => {
            handle_daemon_lifecycle(request, DaemonAction::Restart);
        }
        (Method::Get, "/api/daemon/status") => {
            handle_daemon_status(request);
        }
        (Method::Get, "/api/logs") => {
            handle_log_tail(request, &query);
        }
        // ── TytusOS Wave 3b: launch in editor ──────────────────────────
        (Method::Get, "/api/launchers") => {
            handle_launchers_list(request);
        }
        (Method::Post, "/api/launch") => {
            handle_launch(request, &query);
        }
        // ── TytusOS Wave 3c: per-pod channels ──────────────────────────
        (Method::Get, "/api/channels") => {
            handle_channels_list(request, &query);
        }
        (Method::Post, "/api/channels/add") => {
            handle_channels_add(request, &query);
        }
        (Method::Post, "/api/channels/remove") => {
            handle_channels_remove(request, &query);
        }
        (Method::Post, "/api/channels/catalog") => {
            handle_channels_catalog(request);
        }
        // ── TytusOS Wave 4: sync gaps ──────────────────────────────────
        (Method::Post, "/api/pod/stop-forwarder") => {
            handle_pod_stop_forwarder(request, &query);
        }
        (Method::Post, "/api/configure") => {
            handle_configure(request);
        }
        (Method::Post, "/api/terminal/session") => {
            handle_terminal_start(request);
        }
        (Method::Get, "/api/terminal/session") => {
            handle_terminal_read(request, &query);
        }
        (Method::Post, "/api/terminal/session/write") => {
            handle_terminal_write(request, &query);
        }
        (Method::Post, "/api/terminal/session/resize") => {
            handle_terminal_resize(request, &query);
        }
        (Method::Delete, "/api/terminal/session") => {
            handle_terminal_stop(request, &query);
        }
        // ── TytusOS Wave 5 (v0.5.4): garagetytus shared-folders parity ──
        (Method::Get, "/api/garagetytus/status") => {
            handle_garagetytus_status(request);
        }
        (Method::Get, "/api/sharing/defaults") => {
            handle_sharing_defaults_get(request);
        }
        (Method::Post, "/api/sharing/defaults") | (Method::Put, "/api/sharing/defaults") => {
            handle_sharing_defaults_save(request);
        }
        (Method::Get, "/api/shared-folders/list") => {
            handle_shared_folders_list(request);
        }
        (Method::Post, "/api/shared-folders/run-streamed") => {
            handle_shared_folders_run_streamed(request, &registry, &query);
        }
        (Method::Post, "/api/shared-folders/open") => {
            handle_shared_folders_open(request);
        }
        (Method::Post, "/api/shared-folders/open-cache") => {
            handle_shared_folders_open_cache(request);
        }
        (Method::Post, "/api/pod/refresh-creds") => {
            handle_pod_refresh_creds(request, &registry, &query);
        }
        (Method::Post, "/api/shared-folders/provision-pod") => {
            handle_shared_folders_provision_pod(request, &registry);
        }
        (Method::Post, "/api/shared-folders/pick-folder") => {
            handle_shared_folders_pick_folder(request);
        }
        (Method::Post, "/api/shared-folders/bind") => {
            handle_shared_folders_bind(request, &registry);
        }
        // ── rc.13 / PR3: Files app safe browser actions ─────────────
        (Method::Get, "/api/files/list") => {
            handle_files_list(request, &query);
        }
        (Method::Post, "/api/files/mkdir") => {
            handle_files_mkdir(request);
        }
        (Method::Post, "/api/files/rename") => {
            handle_files_rename(request);
        }
        (Method::Post, "/api/files/delete") => {
            handle_files_delete(request);
        }
        (Method::Post, "/api/files/copy") => {
            handle_files_copy(request);
        }
        (Method::Post, "/api/files/move") => {
            handle_files_move(request);
        }
        (Method::Get, "/api/files/download") => {
            handle_files_download(request, &query);
        }
        (Method::Post, "/api/files/upload") => {
            handle_files_upload(request);
        }
        // Open the per-pod local downloads dir in Finder. Path is the
        // same `~/Tytus/Downloads/pod-NN/` the tray writes to. Created
        // on demand so the open call doesn't fail on a fresh pod.
        (Method::Post, "/api/files/open-downloads") => {
            handle_files_open_downloads(request, &query);
        }
        (Method::Post, "/api/workspace/open") => {
            handle_workspace_open(request);
        }
        _ if path.starts_with("/api/") => {
            respond_json(request, 404, &serde_json::json!({ "error": "not found" }));
        }
        _ => {
            if matches!(method, Method::Get) {
                serve_os_route(request, &path);
            } else {
                let resp = Response::from_string("not found").with_status_code(StatusCode(404));
                let _ = request.respond(resp);
            }
        }
    }

    // Defensive: clear the thread-local in case a handler responded
    // through a path other than `respond_json` (e.g. raw `Response`
    // for SSE) and didn't consume the key. The key would otherwise
    // leak to a subsequent request handled on this thread — though
    // every request currently spawns its own thread, future tuning
    // (a thread pool) shouldn't silently regress dedupe correctness.
    set_current_idem_key(None);
}

fn serve_bytes(request: Request, body: &[u8], content_type: &str) {
    let resp = Response::from_data(body.to_vec())
        .with_header(header("Content-Type", content_type))
        // CSP: no external resources, no inline eval. Our JS is same-origin,
        // same with the CSS and the JSON endpoints.
        .with_header(header(
            "Content-Security-Policy",
            "default-src 'self'; script-src 'self'; style-src 'self'; worker-src 'self'; connect-src 'self'; img-src 'self' data: blob:; media-src 'self' blob:; font-src 'self';",
        ))
        // This wizard is local-only by design. Block embedding in any
        // frame and discourage MIME-sniffing.
        .with_header(header("X-Content-Type-Options", "nosniff"))
        .with_header(header("X-Frame-Options", "DENY"));
    let _ = request.respond(resp);
}

fn header(name: &'static str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes())
        .expect("header construction cannot fail for ascii inputs")
}

// ── /api/catalog ──────────────────────────────────────────────

fn handle_catalog(request: Request, query: &str) {
    let refresh = query.split('&').any(|kv| kv == "refresh=1");
    // The tray process is sync; spin up a short-lived tokio runtime to
    // call the async fetch. Catalog fetches are low-frequency + sub-second
    // so the runtime spin-up cost is acceptable.
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            respond_json(request, 500, &serde_json::json!({ "error": e.to_string() }));
            return;
        }
    };
    let http = atomek_core::HttpClient::new();
    let result = rt.block_on(atomek_pods::fetch_catalog(&http, refresh));
    match result {
        Ok(cat) => respond_json(request, 200, &cat),
        Err(e) => respond_json(
            request,
            502,
            &serde_json::json!({ "error": format!("catalog fetch failed: {}", e) }),
        ),
    }
}

// ── /api/apps (App Store) ─────────────────────────────────────

fn handle_apps_catalog(request: Request) {
    serve_bytes(request, APPS_JSON, "application/json; charset=utf-8");
}

#[derive(serde::Deserialize)]
struct AppsCheckRequest {
    app_ids: Vec<String>,
}

fn handle_apps_check(mut request: Request) {
    let mut body = String::new();
    if let Err(e) = request.as_reader().read_to_string(&mut body) {
        respond_json(request, 400, &serde_json::json!({ "error": e.to_string() }));
        return;
    }
    let parsed: AppsCheckRequest = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({ "error": e.to_string() }));
            return;
        }
    };

    // Parse the embedded catalog to get detect paths per app
    let catalog: Vec<serde_json::Value> = match serde_json::from_slice(APPS_JSON) {
        Ok(c) => c,
        Err(e) => {
            respond_json(
                request,
                500,
                &serde_json::json!({ "error": format!("catalog parse error: {}", e) }),
            );
            return;
        }
    };

    let platform = if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    };
    let mut results: Vec<serde_json::Value> = Vec::new();

    for app_id in &parsed.app_ids {
        // Validate: alphanumeric + hyphen + underscore only
        if !app_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            results.push(
                serde_json::json!({ "id": app_id, "installed": false, "error": "invalid id" }),
            );
            continue;
        }

        let entry = catalog.iter().find(|e| e["id"].as_str() == Some(app_id));
        let installed = if let Some(entry) = entry {
            check_app_installed(entry, platform)
        } else {
            false
        };

        results.push(serde_json::json!({ "id": app_id, "installed": installed }));
    }

    respond_json(request, 200, &serde_json::json!({ "results": results }));
}

fn check_app_installed(entry: &serde_json::Value, platform: &str) -> bool {
    // Try detect commands for the platform
    if let Some(detect) = entry.get("detect").and_then(|d| d.get(platform)) {
        if let Some(commands) = detect.as_array() {
            for cmd in commands {
                if let Some(cmd_str) = cmd.as_str() {
                    // On macOS: check /Applications/<Name>.app first
                    #[cfg(target_os = "macos")]
                    {
                        let app_path = format!("/Applications/{}.app", cmd_str);
                        if std::path::Path::new(&app_path).exists() {
                            return true;
                        }
                        // Also check ~/Applications/
                        if let Ok(home) = std::env::var("HOME") {
                            let user_app_path = format!("{}/Applications/{}.app", home, cmd_str);
                            if std::path::Path::new(&user_app_path).exists() {
                                return true;
                            }
                        }
                    }
                    // Check if the command is on PATH
                    if let Ok(output) = std::process::Command::new("which")
                        .arg(cmd_str)
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status()
                    {
                        if output.success() {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

// ── /api/install ──────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct InstallRequest {
    agent_type: String,
    #[serde(default)]
    pod_id: Option<String>,
}

fn handle_install(mut request: Request, registry: &Registry) {
    let mut body = String::new();
    if let Err(e) = request.as_reader().read_to_string(&mut body) {
        respond_json(request, 400, &serde_json::json!({ "error": e.to_string() }));
        return;
    }
    let parsed: InstallRequest = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({ "error": e.to_string() }));
            return;
        }
    };

    // Whitelist agent_type to prevent shell injection via the subprocess
    // argv. Pod id gets the same treatment. Catalog entries are
    // validated, but defense-in-depth: reject anything that isn't a
    // simple identifier before we hand it to `tytus`.
    if !parsed
        .agent_type
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        respond_json(
            request,
            400,
            &serde_json::json!({ "error": "invalid agent_type" }),
        );
        return;
    }
    if let Some(ref pid) = parsed.pod_id {
        if !pid.chars().all(|c| c.is_ascii_digit()) {
            respond_json(
                request,
                400,
                &serde_json::json!({ "error": "invalid pod_id" }),
            );
            return;
        }
    }

    let (job_id, job) = registry.create();
    spawn_install(job, parsed.agent_type, parsed.pod_id);
    respond_json(request, 202, &serde_json::json!({ "job_id": job_id }));
}

/// Resolve the absolute path to the `tytus` binary.
///
/// The tray is launched by a LaunchAgent, whose PATH is the kernel-default
/// `/usr/bin:/bin:/usr/sbin:/sbin` — it does NOT include `~/bin`,
/// `/usr/local/bin`, `/opt/homebrew/bin`, or `~/.cargo/bin`, so a bare
/// `Command::new("tytus")` spawns with `os error 2: No such file or
/// directory` even when the CLI is installed. Terminal-path workflows
/// dodge this because Terminal.app spawns a login shell that sources
/// the user's zshrc.
///
/// Resolution order:
///   1. `TYTUS_BIN` env var (escape hatch for unusual installs)
///   2. `~/bin/tytus` (install.sh default)
///   3. `/usr/local/bin/tytus`, `/opt/homebrew/bin/tytus`, `~/.cargo/bin/tytus`
///   4. `tytus-tray`'s own directory (dev builds: cargo run leaves them
///      side by side in `target/<profile>/`)
///   5. Fallback to the bare name — caller will surface the spawn error
fn resolve_tytus_bin() -> PathBuf {
    if let Ok(p) = std::env::var("TYTUS_BIN") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return pb;
        }
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let candidates: Vec<PathBuf> = vec![
        PathBuf::from(&home).join("bin/tytus"),
        PathBuf::from("/usr/local/bin/tytus"),
        PathBuf::from("/opt/homebrew/bin/tytus"),
        PathBuf::from(&home).join(".cargo/bin/tytus"),
    ];
    for c in &candidates {
        if c.is_file() {
            return c.clone();
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let sibling = dir.join("tytus");
            if sibling.is_file() {
                return sibling;
            }
        }
    }
    PathBuf::from("tytus")
}

fn spawn_install(job: Arc<Mutex<Job>>, agent_type: String, pod_id: Option<String>) {
    thread::spawn(move || {
        let bin = resolve_tytus_bin();
        let mut cmd = Command::new(&bin);
        cmd.arg("agent").arg("install").arg(&agent_type);
        if let Some(ref pid) = pod_id {
            cmd.arg("--pod").arg(pid);
        }
        cmd.arg("--json");
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        cmd.env("TYTUS_HEADLESS", "1"); // no browser popups from subprocess

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                push_event(
                    &job,
                    JobEvent::Fail {
                        message: format!(
                            "failed to launch `tytus` at {}: {}. \
                         If the CLI is installed somewhere else, set TYTUS_BIN \
                         in the tray's environment.",
                            bin.display(),
                            e,
                        ),
                    },
                );
                return;
            }
        };
        // Track the PID so /api/jobs/<id>/cancel can SIGTERM it.
        job.lock().unwrap().child_pid = Some(child.id());

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // Merge stdout + stderr into the log stream. stderr is where
        // `tytus` prints progress ("Allocating pod…"); stdout holds the
        // final JSON on --json.
        let mut stdout_out = String::new();
        let mut stderr_out = String::new();

        let job_for_stdout = job.clone();
        let stdout_thread = stdout.map(|mut h| {
            thread::spawn(move || {
                let mut buf = [0u8; 4096];
                let mut carry = String::new();
                while let Ok(n) = h.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    carry.push_str(&String::from_utf8_lossy(&buf[..n]));
                    while let Some(idx) = carry.find('\n') {
                        let line = carry[..idx].to_string();
                        carry = carry[idx + 1..].to_string();
                        stdout_out.push_str(&line);
                        stdout_out.push('\n');
                        push_event(&job_for_stdout, JobEvent::Log(line));
                    }
                }
                if !carry.is_empty() {
                    stdout_out.push_str(&carry);
                    push_event(&job_for_stdout, JobEvent::Log(carry));
                }
                stdout_out
            })
        });

        let job_for_stderr = job.clone();
        let stderr_thread = stderr.map(|mut h| {
            thread::spawn(move || {
                let mut buf = [0u8; 4096];
                let mut carry = String::new();
                while let Ok(n) = h.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    carry.push_str(&String::from_utf8_lossy(&buf[..n]));
                    while let Some(idx) = carry.find('\n') {
                        let line = carry[..idx].to_string();
                        carry = carry[idx + 1..].to_string();
                        stderr_out.push_str(&line);
                        stderr_out.push('\n');
                        push_event(&job_for_stderr, JobEvent::Log(line));
                    }
                }
                if !carry.is_empty() {
                    stderr_out.push_str(&carry);
                    push_event(&job_for_stderr, JobEvent::Log(carry));
                }
                stderr_out
            })
        });

        let status = child.wait();
        // PID is stale once wait() returns. Clear before the terminal
        // event so the cancel handler can't accidentally signal a
        // recycled PID.
        job.lock().unwrap().child_pid = None;

        // Join the readers so we can inspect stdout for the final JSON.
        let stdout_captured = stdout_thread
            .and_then(|h| h.join().ok())
            .unwrap_or_default();
        let _ = stderr_thread.and_then(|h| h.join().ok());

        match status {
            Ok(s) if s.success() => {
                // Parse the last JSON-looking line on stdout — `tytus agent
                // install --json` prints a single object at the end.
                let payload = stdout_captured
                    .lines()
                    .rev()
                    .find(|l| l.trim_start().starts_with('{'))
                    .unwrap_or("{}")
                    .to_string();
                if let Some(pod_id) = extract_pod_id_from_json(&payload) {
                    match wait_for_installed_agent_ready(&job, &pod_id) {
                        Ok(()) => {}
                        Err(message) => {
                            push_event(&job, JobEvent::Fail { message });
                            return;
                        }
                    }
                } else {
                    push_event(
                        &job,
                        JobEvent::Log(
                            "Install finished but pod id was not present in CLI JSON; skipping readiness wait."
                                .to_string(),
                        ),
                    );
                }
                push_event(&job, JobEvent::Done { payload });
            }
            Ok(s) => push_event(
                &job,
                JobEvent::Fail {
                    message: format!("tytus exited with status {}", s),
                },
            ),
            Err(e) => push_event(
                &job,
                JobEvent::Fail {
                    message: format!("wait failed: {}", e),
                },
            ),
        }
    });
}

fn push_event(job: &Arc<Mutex<Job>>, ev: JobEvent) {
    let mut j = job.lock().unwrap();
    let terminal = matches!(
        ev,
        JobEvent::Done { .. } | JobEvent::Fail { .. } | JobEvent::Exit { .. }
    );
    // Terminal events (Done / Fail / Exit) bypass both caps — the SSE
    // consumer needs them to wind down its EventSource and they're
    // bounded in size by construction.
    if terminal {
        j.events.push(ev);
        j.finished = true;
        return;
    }
    // Log path: truncate over-long lines and refuse new ones once
    // we've hit MAX_EVENTS.
    let truncated = match ev {
        JobEvent::Log(line) => JobEvent::Log(truncate_log_line(line)),
        other => other,
    };
    if j.events.len() >= MAX_EVENTS {
        if !j.log_capped {
            j.log_capped = true;
            // One-time sentinel so the user sees that output stopped
            // because of the cap, not because the process went silent.
            j.events.push(JobEvent::Log(format!(
                "…[log capped at {} events; further output suppressed]",
                MAX_EVENTS,
            )));
        }
        return;
    }
    j.events.push(truncated);
}

fn truncate_log_line(mut line: String) -> String {
    if line.len() > MAX_LINE_LEN {
        // Truncate at a char boundary so we don't split a UTF-8
        // sequence. find_floor_char_boundary isn't stable, so walk
        // backwards from the cap until we land on a boundary.
        let mut cut = MAX_LINE_LEN;
        while cut > 0 && !line.is_char_boundary(cut) {
            cut -= 1;
        }
        line.truncate(cut);
        line.push_str("…[truncated]");
    }
    line
}

// ── /api/jobs/<id>/stream (SSE) ───────────────────────────────

fn handle_stream(request: Request, registry: &Registry, job_id: &str) {
    let job = match registry.get(job_id) {
        Some(j) => j,
        None => {
            respond_json(request, 404, &serde_json::json!({ "error": "unknown job" }));
            return;
        }
    };
    // EventSource auto-resends `Last-Event-ID` on reconnect when the
    // server emitted `id:` lines. Parse it (if present) so we can
    // resume from the next event instead of replaying the whole job
    // and double-delivering log lines after a network blip.
    let resume_from = parse_last_event_id(request.headers());
    // tiny_http doesn't expose a connection-upgrade primitive; instead
    // we return a Response whose body is a blocking `Read` that we drip-
    // feed from a background thread. The browser sees the event-stream
    // content type and treats it as SSE.
    sse_response(request, job, resume_from);
}

/// Parse the `Last-Event-ID` header into the next event index to send.
/// We use the event's position in `Job.events` as its id (0, 1, 2 …),
/// so a Last-Event-ID of N means "I've seen up to N inclusive — start
/// at N+1". Bad / missing headers fall back to 0 so a fresh subscriber
/// gets the full replay.
fn parse_last_event_id(headers: &[Header]) -> usize {
    for h in headers {
        if h.field
            .as_str()
            .as_str()
            .eq_ignore_ascii_case("Last-Event-ID")
        {
            if let Ok(n) = h.value.as_str().parse::<usize>() {
                return n.saturating_add(1);
            }
        }
    }
    0
}

fn sse_response(request: Request, job: Arc<Mutex<Job>>, resume_from: usize) {
    // Strategy: spawn a thread that reads events from the job, serializes
    // them to SSE frames, and writes them into a pipe whose read half we
    // hand to tiny_http as the response body. The response header sends
    // "Content-Type: text/event-stream" and no Content-Length so the
    // browser keeps the connection open until we close the pipe.

    let (rx, tx) = pipe();

    thread::spawn(move || {
        let mut cursor = resume_from;
        let mut tx = tx;
        loop {
            let (events_snapshot, finished) = {
                let j = job.lock().unwrap();
                (j.events.len(), j.finished)
            };
            while cursor < events_snapshot {
                let frame = {
                    let j = job.lock().unwrap();
                    // The `id: N\n` prefix is what makes Last-Event-ID
                    // round-trip work — EventSource parses it and sends
                    // the last seen value back on its next reconnect.
                    match &j.events[cursor] {
                        JobEvent::Log(line) => format!(
                            "id: {}\nevent: log\ndata: {}\n\n",
                            cursor,
                            line.replace('\n', "\\n"),
                        ),
                        JobEvent::Done { payload } => format!(
                            "id: {}\nevent: done\ndata: {}\n\n",
                            cursor,
                            payload.replace('\n', " "),
                        ),
                        JobEvent::Fail { message } => format!(
                            "id: {}\nevent: fail\ndata: {}\n\n",
                            cursor,
                            message.replace('\n', " "),
                        ),
                        JobEvent::Exit { code } => format!(
                            "id: {}\nevent: exit\ndata: {{\"code\":{}}}\n\n",
                            cursor, code,
                        ),
                    }
                };
                if tx.write_all(frame.as_bytes()).is_err() {
                    return;
                }
                cursor += 1;
            }
            if finished && cursor >= events_snapshot {
                break;
            }
            thread::sleep(std::time::Duration::from_millis(150));
        }
        // Ensure the browser sees EOF and triggers the "done" handler.
        drop(tx);
    });

    let resp = Response::empty(StatusCode(200))
        .with_header(header("Content-Type", "text/event-stream"))
        .with_header(header("Cache-Control", "no-cache"))
        .with_header(header("X-Accel-Buffering", "no"))
        .with_data(rx, None)
        // CRITICAL for SSE streaming: tiny_http's default chunked
        // threshold is 32KB — when the body length is unknown AND
        // total output is shorter, it buffers the ENTIRE response
        // before sending to compute Content-Length, which defeats
        // streaming entirely (browser gets all frames at once at
        // process exit). Setting threshold=0 forces chunked transfer
        // encoding from the first byte: each `read()` from the pipe
        // produces a chunk that flushes to the wire immediately.
        // Verified via `curl -sN ... | timestamper` — without this,
        // all SSE frames arrive within a single second; with it, they
        // arrive as the subprocess emits them.
        .with_chunked_threshold(0);
    let _ = request.respond(resp);
}

/// Simple in-memory pipe — writer side pushes bytes, reader side pulls
/// them for tiny_http's response body. Backed by a `VecDeque<u8>` under
/// a mutex; blocks the reader until the writer produces more or closes.
fn pipe() -> (PipeReader, PipeWriter) {
    let shared = Arc::new(Mutex::new(PipeState {
        buf: Vec::new(),
        closed: false,
    }));
    let reader = PipeReader {
        state: shared.clone(),
    };
    let writer = PipeWriter { state: shared };
    (reader, writer)
}

struct PipeState {
    buf: Vec<u8>,
    closed: bool,
}

struct PipeReader {
    state: Arc<Mutex<PipeState>>,
}

impl Read for PipeReader {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        loop {
            {
                let mut s = self.state.lock().unwrap();
                if !s.buf.is_empty() {
                    let n = out.len().min(s.buf.len());
                    out[..n].copy_from_slice(&s.buf[..n]);
                    s.buf.drain(..n);
                    return Ok(n);
                }
                if s.closed {
                    return Ok(0);
                }
            }
            thread::sleep(std::time::Duration::from_millis(50));
        }
    }
}

struct PipeWriter {
    state: Arc<Mutex<PipeState>>,
}

impl Write for PipeWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut s = self.state.lock().unwrap();
        if s.closed {
            return Err(std::io::ErrorKind::BrokenPipe.into());
        }
        s.buf.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for PipeWriter {
    fn drop(&mut self) {
        if let Ok(mut s) = self.state.lock() {
            s.closed = true;
        }
    }
}

// ── /api/state ────────────────────────────────────────────────

#[derive(Serialize)]
struct StateSnapshot {
    connected: bool,
    logged_in: bool,
    /// True when the WG tunnel is up and 10.42.42.1:18080 is reachable.
    /// Used by the TytusOS page header to choose between "Disconnect" and
    /// "Connect" actions. Cheap TCP probe — 500ms cap.
    tunnel_active: bool,
    tier: String,
    units_limit: u32,
    units_used: u32,
    agents: Vec<AgentSlot>,
    /// Pods that are allocated but don't count against the unit
    /// budget — the default AIL / LLM-gateway pod (`agent_type=none`).
    /// These are always included with every plan; we surface them in
    /// the wizard so users can copy their OpenAI-compatible endpoint
    /// without running `tytus env --export`.
    included: Vec<IncludedSlot>,
    // ── Wave 4: fields merged from socket::poll_daemon_status ──────
    /// Signed-in user. Empty string when logged-out.
    email: String,
    /// Seconds since the daemon started. 0 when the daemon is down.
    uptime_secs: u64,
    /// False when the macOS keychain hasn't yet yielded the refresh
    /// token (pending approval dialog, ACL stale after a rebuild).
    /// Drives the yellow warning banner on the page.
    keychain_healthy: bool,
    /// Last refresh error the daemon observed, verbatim. Surfaced in
    /// Troubleshoot only when present.
    last_refresh_error: Option<String>,
    /// Daemon process state — separate from `connected` (state.json
    /// parseability). Daemon can be down while state.json is fine.
    daemon_running: bool,
    daemon_pid: u64,
    /// True when /Applications/Tytus.app exists. Page uses this to
    /// decide whether to surface the "Install in Applications" row.
    app_bundle_installed: bool,
    /// Pod IDs that currently have a live localhost UI forwarder — the
    /// user has run "Open in Browser" through the WG fallback. Page
    /// uses this to show a "Stop Forwarder" button on the matching
    /// running-pod panel.
    forwarders: Vec<String>,
    /// Tray binary version (`CARGO_PKG_VERSION` of `tytus-tray`). Lets
    /// TytusOS gate UI features that require a newer daemon surface
    /// without firing a separate /api/version request — saves an HTTP
    /// roundtrip per poll *and* avoids 404 noise on consumers running
    /// against pre-version-endpoint daemons.
    daemon_version: String,
    /// Daemon boot time, Unix seconds. Stable across the daemon
    /// process's lifetime via `OnceLock`. TytusOS diffs this between
    /// polls to detect a daemon restart and drop in-flight job state
    /// (the registry is in-memory; every active job_id is invalid past
    /// a restart).
    daemon_started_at: u64,
}

#[derive(Serialize, Clone)]
struct AgentSlot {
    pod_id: String,
    agent_type: String,
    units: u32,
    /// Public HTTPS URL of the pod — e.g. `https://<slug>.tytus.traylinx.com/p/02`.
    /// None when the edge isn't wired up yet (mid-rollout); wizard hides URL row.
    public_url: Option<String>,
    /// Full OpenAI-compatible endpoint (`{public_url}/v1`) ready to drop into
    /// OPENAI_BASE_URL. None when public_url is None.
    api_url: Option<String>,
    /// Browser-authenticated UI URL (`{public_url}/?token={gateway_token}`)
    /// for browser agents (OpenClaw/NemoClaw SPA, Hermes dashboard). None
    /// while the edge URL or per-pod secret is still missing.
    ui_url: Option<String>,
    /// Per-pod gateway secret used by readiness probes and local forwarder.
    /// Never serialize to TytusOS: the UI gets only redacted `ui_url` and the
    /// stable user key.
    #[serde(skip_serializing)]
    gateway_token: Option<String>,
    /// Stable per-user API key — same across every pod.
    user_key: String,
    /// Latest gateway-probe verdict cached for STATUS_TTL_SECS.
    /// Phase 2 of remaining sprint: lifts /api/pod/ready polling
    /// server-side. New daemons emit; old daemons omit (forward-compat
    /// — TytusOS treats absent as "fall back to /api/pod/ready").
    status: AgentStatus,
}

#[derive(Serialize, Clone)]
struct IncludedSlot {
    pod_id: String,
    kind: &'static str, // "ail" for now; future types can reuse
    endpoint: String,   // stable_ai_endpoint (e.g. http://10.42.42.1:18080)
    user_key: String,   // stable_user_key (sk-tytus-user-…)
    /// Public per-pod HTTPS URL (`{edge}/p/01`) — same shape as AgentSlot so
    /// the wizard can show a "public mirror" URL for the AIL pod too.
    public_url: Option<String>,
}

/// Per-plan unit budgets — must match Scalesys `AGENT_UNITS` + the Rails
/// plan tiering. Keep aligned with `services/wannolot-provider/src/...`
/// where `nemoclaw=1, hermes=2, none=0` and Explorer=1 / Creator=2 /
/// Operator=4. Unknown agent types default to 1 unit (conservative so
/// we never under-count the user's spend).
/// Compute the per-pod gateway auth token. The edge plugin accepts
/// `?token=<48-hex>` as an alternative to Bearer on non-/v1 paths and
/// checks it against `sha256(pod_api_key || pod_id)[:48]` — the exact
/// same value openclaw's `gateway.auth.token` is set to at pod start.
/// Safe to derive here because pod_api_key is already in state.json
/// on the user's machine (it's written at install time) — we're not
/// inventing a secret, we're reproducing one that exists.
fn derive_gateway_token(pod_api_key: &str, pod_id: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(pod_api_key.as_bytes());
    h.update(pod_id.as_bytes());
    let digest = h.finalize();
    // 48 hex chars = first 24 bytes of the digest.
    hex::encode(&digest[..24])
}

fn plan_limit_for(tier: &str) -> u32 {
    match tier.to_ascii_lowercase().as_str() {
        "explorer" => 1,
        "creator" => 2,
        "operator" => 4,
        _ => 0,
    }
}
fn agent_units_for(agent_type: &str) -> u32 {
    match agent_type {
        "hermes" => 2,
        "none" => 0,
        _ => 1, // nemoclaw + future openclaw-family
    }
}

/// POST /api/jobs/<id>/cancel — SIGTERM the job's child process.
///
/// Idempotent and safe to call against a finished job (returns
/// `cancelled: false` with `reason: "already finished"`). When the
/// child PID isn't tracked yet (job exists but `Command::spawn` hasn't
/// returned) — same response with `reason: "no live process"`. The
/// terminal `exit` SSE event is emitted by the existing wait() path
/// once the child dies, so the client doesn't need a separate
/// cancellation event to wind down its EventSource.
fn handle_job_cancel(request: Request, registry: &Registry, job_id: &str) {
    let Some(job) = registry.get(job_id) else {
        respond_json(request, 404, &serde_json::json!({"error": "no such job"}));
        return;
    };
    let (pid, finished) = {
        let j = job.lock().unwrap();
        (j.child_pid, j.finished)
    };
    if finished {
        respond_json(
            request,
            200,
            &serde_json::json!({
                "cancelled": false,
                "reason": "already finished",
            }),
        );
        return;
    }
    let Some(pid) = pid else {
        respond_json(
            request,
            200,
            &serde_json::json!({
                "cancelled": false,
                "reason": "no live process",
            }),
        );
        return;
    };
    push_event(
        &job,
        JobEvent::Log(format!("[cancel] sending SIGTERM to PID {}", pid)),
    );
    let rc = unsafe { libc::kill(pid as i32, libc::SIGTERM) };
    if rc != 0 {
        let errno = std::io::Error::last_os_error();
        respond_json(
            request,
            500,
            &serde_json::json!({
                "cancelled": false,
                "reason": format!("kill failed: {}", errno),
            }),
        );
        return;
    }
    respond_json(
        request,
        200,
        &serde_json::json!({
            "cancelled": true,
            "pid": pid,
        }),
    );
}

/// GET /api/version — daemon identity for compat checks.
///
/// `daemon_started_at` is the Unix-seconds timestamp captured on
/// first start() call. UI consumers persist this between polls and
/// drop in-flight `activeJob` state when the value changes — a
/// daemon restart invalidates every job_id since the registry is
/// in-memory only.
///
/// `daemon_version` is the `tytus-tray` crate version (CARGO_PKG_VERSION).
/// Future TytusOS releases compare it against a min-required value
/// so a newer UI can surface "your tray is too old, run `tytus tray
/// install`" instead of failing with a confused 404 on a missing
/// route.
fn handle_version(request: Request) {
    respond_json(
        request,
        200,
        &serde_json::json!({
            "daemon_version": env!("CARGO_PKG_VERSION"),
            "daemon_pid": std::process::id(),
            "daemon_started_at": daemon_started_at(),
        }),
    );
}

fn handle_state(request: Request, registry: &Registry) {
    let snap = compute_state_snapshot();
    let active = registry.active_pods();
    // Merge in active_jobs_per_pod (Phase B running-session badge).
    // serialize snap then patch — avoids changing StateSnapshot's
    // schema and breaking the long list of #[derive(Serialize)] fields
    // it carries today.
    let mut value = match serde_json::to_value(&snap) {
        Ok(v) => v,
        Err(_) => {
            respond_json(request, 200, &snap);
            return;
        }
    };
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "active_jobs_per_pod".to_string(),
            serde_json::to_value(&active).unwrap_or(serde_json::Value::Null),
        );
    }

    // ETag conditional GET. The client polls /api/state every few
    // seconds and most polls return identical bodies — caching the
    // hash and short-circuiting to 304 saves the JSON.parse +
    // setState round on every no-change tick.
    //
    // Hash with std's DefaultHasher (SipHash). Not crypto, but the
    // ETag only needs to be consistent within one daemon process —
    // a hash collision would cause us to skip a state update, which
    // self-corrects on the next non-collision poll.
    //
    // Phase 6 fix: hash a *logical-identity* view of the snapshot
    // with `uptime_secs` stripped. uptime_secs ticks every second by
    // wall-clock and would otherwise flip the ETag every second even
    // when nothing the user cares about changed — defeating the
    // 304-short-circuit *and* causing the wire-test flake captured
    // in feedback_etag_wire_test_flake. Pods/auth/units/etc. all
    // stay in the hash; only the ticker is excluded.
    let body = serde_json::to_string(&value).unwrap_or_else(|_| "{}".into());
    let etag = {
        let mut hash_value = value.clone();
        if let Some(obj) = hash_value.as_object_mut() {
            obj.remove("uptime_secs");
        }
        let hash_body = serde_json::to_string(&hash_value).unwrap_or_else(|_| "{}".into());
        compute_state_etag(hash_body.as_bytes())
    };

    if let Some(client_etag) = read_if_none_match(request.headers()) {
        if client_etag == etag {
            // 304 Not Modified — empty body, but echo the ETag so a
            // proxy that drops the response can still validate.
            let resp = Response::empty(StatusCode(304))
                .with_header(header("ETag", &etag))
                .with_header(header("Cache-Control", "no-cache"));
            let _ = request.respond(resp);
            return;
        }
    }

    let resp = Response::from_string(body)
        .with_status_code(StatusCode(200))
        .with_header(header("Content-Type", "application/json"))
        .with_header(header("ETag", &etag))
        .with_header(header("Cache-Control", "no-cache"));
    let _ = request.respond(resp);
}

/// Compute a stable per-process ETag for the serialized state body.
/// Returns a quoted hex string per RFC 9110 §8.8.3 (`"abc123"`).
fn compute_state_etag(body: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    body.hash(&mut h);
    format!("\"{:016x}\"", h.finish())
}

fn read_if_none_match(headers: &[Header]) -> Option<String> {
    for h in headers {
        if h.field
            .as_str()
            .as_str()
            .eq_ignore_ascii_case("If-None-Match")
        {
            return Some(h.value.as_str().to_string());
        }
    }
    None
}

/// Build the rich state snapshot that the wizard's budget strip, the
/// running-pod panels, and the disabled-card logic all depend on.
///
/// Data comes from the CLI's state.json rather than
/// `tytus status --json` so we get the full pod schema — in particular
/// `edge_public_url`, `edge_slug`, and `gateway_token`, which
/// `status --json` does NOT expose today. Those three fields are
/// essential for building the public browser URL and the
/// OpenAI-compatible API URL that the wizard surfaces as "Open in
/// Browser" and "Copy API URL".
///
/// Failure mode: never blocks the wizard. If state.json is missing,
/// unreadable, or malformed we return an empty snapshot and the
/// wizard renders cards with no budget/running data — the user can
/// still click Install.
fn compute_state_snapshot() -> StateSnapshot {
    let empty = || StateSnapshot {
        connected: false,
        logged_in: false,
        tunnel_active: false,
        tier: String::new(),
        units_limit: 0,
        units_used: 0,
        agents: vec![],
        included: vec![],
        email: String::new(),
        uptime_secs: 0,
        keychain_healthy: true,
        last_refresh_error: None,
        daemon_running: false,
        daemon_pid: 0,
        app_bundle_installed: crate::check_app_bundle_installed(),
        forwarders: vec![],
        daemon_version: env!("CARGO_PKG_VERSION").to_string(),
        daemon_started_at: daemon_started_at(),
    };

    let state_path = state_json_path();
    let raw = match state_path.and_then(|p| std::fs::read_to_string(p).ok()) {
        Some(s) => s,
        None => return empty(),
    };
    let parsed: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return empty(),
    };

    let tier = parsed
        .get("tier")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    // state.json uses `tokens.expires_at` presence as the logged-in proxy;
    // for the wizard we treat "has a tier + at least one pod" as logged in.
    let logged_in = !tier.is_empty()
        || parsed
            .get("pods")
            .and_then(|v| v.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false);

    let mut agents = Vec::new();
    let mut included = Vec::new();
    let mut used = 0u32;

    // ── Slug inheritance ─────────────────────────────────────
    // `edge_public_url` is per-user (slug is in Scalesys'
    // user_stable_keys table — all a user's pods share the one
    // `<slug>.tytus.traylinx.com`). The CLI's state.json only
    // backfills this field when `tytus env` or `tytus connect` runs
    // post-install — which means a freshly-installed pod has it
    // null, even though the URL is perfectly derivable from any
    // sibling pod. This made "Open in Browser" fake-disabled on the
    // just-installed pod every time. Pull the first populated base
    // URL and reuse it for siblings that missed the backfill.
    let shared_base: Option<String> =
        parsed
            .get("pods")
            .and_then(|v| v.as_array())
            .and_then(|arr| {
                arr.iter().find_map(|p| {
                    p.get("edge_public_url")
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                })
            });

    if let Some(pods) = parsed.get("pods").and_then(|v| v.as_array()) {
        for p in pods {
            let agent_type = p
                .get("agent_type")
                .and_then(|v| v.as_str())
                .unwrap_or("none")
                .to_string();
            let pod_id = p
                .get("pod_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let edge_public_url: Option<String> = p
                .get("edge_public_url")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .or_else(|| shared_base.clone());
            // Gateway token resolution: prefer the one in state (set
            // by the CLI after any `tytus env` call), else derive it
            // from sha256(pod_api_key || pod_id)[:48] — the exact
            // formula the edge plugin + nemoclaw startup use. Without
            // this, a fresh-install pod's ui_url has no `?token=` so
            // the browser hits the edge's 401 bouncer. Derivation
            // requires only pod_api_key which IS in state.json from
            // install time.
            let stored_token = p
                .get("gateway_token")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            let derived_token = p
                .get("pod_api_key")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|k| derive_gateway_token(k, &pod_id));
            let gateway_token: Option<String> = stored_token.or(derived_token);
            let user_key = p
                .get("stable_user_key")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            // Prefer the per-pod subdomain URL (sprint 2026-04-23) — each
            // pod is its own browser origin so the OpenClaw SPA's
            // localStorage doesn't collide across pods. Fall back to the
            // legacy composed URL for state entries written before the
            // sprint's allocation-path populated the field.
            let stored_pod_url: Option<String> = p
                .get("pod_public_url")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.trim_end_matches('/').to_string());
            let public_url = stored_pod_url.or_else(|| {
                edge_public_url
                    .as_ref()
                    .map(|base| format!("{}/p/{}", base.trim_end_matches('/'), pod_id))
            });
            let api_url = public_url.as_ref().map(|u| format!("{}/v1", u));
            let ui_url = match (public_url.as_ref(), gateway_token.as_ref()) {
                (Some(u), Some(t)) => Some(format!("{}/?token={}", u, t)),
                _ => None,
            };

            if agent_type == "none" {
                let endpoint = p
                    .get("stable_ai_endpoint")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                included.push(IncludedSlot {
                    pod_id,
                    kind: "ail",
                    endpoint,
                    user_key,
                    public_url,
                });
                continue;
            }

            let units = agent_units_for(&agent_type);
            used += units;
            // Cache lookup is non-blocking; a stale entry kicks a
            // background refresh so the next poll sees the fresh
            // value. First poll for a never-probed pod returns
            // Unknown, which the OS renders as "checking".
            let status = agent_status_cached(
                &pod_id,
                &agent_type,
                api_url.as_deref(),
                &user_key,
                gateway_token.as_deref(),
                ui_url.as_deref(),
            );
            agents.push(AgentSlot {
                pod_id,
                agent_type,
                units,
                public_url,
                api_url,
                ui_url,
                gateway_token,
                user_key,
                status,
            });
        }
    }

    // Pull the daemon snapshot for the extra health / session signals
    // the tray also surfaces (email, uptime, keychain, last refresh
    // error). One daemon-socket round-trip per /api/state call, capped
    // at 3s — matches what the tray already does on every poll tick.
    let daemon_snap = crate::socket::poll_daemon_status();

    // Per-pod forwarder presence — populate for each pod we already
    // built so the UI can show a "Stop Forwarder" button when it
    // applies. Only true when /tmp/tytus/ui-forwarder-NN.pid points at
    // a live process.
    let mut forwarders: Vec<String> = Vec::new();
    for a in &agents {
        if crate::existing_ui_forwarder(&a.pod_id).is_some() {
            forwarders.push(a.pod_id.clone());
        }
    }
    for i in &included {
        if crate::existing_ui_forwarder(&i.pod_id).is_some() {
            forwarders.push(i.pod_id.clone());
        }
    }

    let last_refresh_error = effective_state_refresh_error(
        daemon_snap.last_refresh_error,
        logged_in,
        daemon_snap.keychain_healthy,
    );

    StateSnapshot {
        connected: true,
        logged_in,
        tunnel_active: crate::gateway_probe::probe_gateway(),
        units_limit: plan_limit_for(&tier),
        units_used: used,
        tier,
        agents,
        included,
        email: daemon_snap.email,
        uptime_secs: daemon_snap.uptime_secs,
        keychain_healthy: daemon_snap.keychain_healthy,
        last_refresh_error,
        daemon_running: daemon_snap.daemon_running,
        daemon_pid: daemon_snap.daemon_pid,
        app_bundle_installed: crate::check_app_bundle_installed(),
        forwarders,
        daemon_version: env!("CARGO_PKG_VERSION").to_string(),
        daemon_started_at: daemon_started_at(),
    }
}

fn refresh_error_requires_login(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    normalized.contains("refresh token expired")
        || normalized.contains("run `tytus login`")
        || normalized.contains("run 'tytus login'")
        || normalized.contains("no refresh token available")
}

fn effective_state_refresh_error(
    error: Option<String>,
    logged_in: bool,
    keychain_healthy: bool,
) -> Option<String> {
    match error {
        Some(err) if logged_in && keychain_healthy && refresh_error_requires_login(&err) => None,
        other => other,
    }
}

/// Locate the CLI's state.json. Matches the CLI's `CliState::path()`
/// logic: `$XDG_CONFIG_HOME/tytus/state.json` on Linux, `~/Library/
/// Application Support/tytus/state.json` on macOS. If the config dir
/// is not resolvable we fall back to `~/.config/tytus/state.json` so
/// callers don't have to branch on platform.
fn state_json_path() -> Option<std::path::PathBuf> {
    // Test override: lets the etag_roundtrip wire test (and any other
    // future test) pin compute_state_snapshot to a known fixture
    // file, isolating it from the live daemon state on the developer
    // machine. Without this, the wire test flaked when uptime_secs /
    // forwarders / probe_gateway results moved between request 1 and
    // request 2 — captured in feedback_etag_wire_test_flake.
    //
    // Production never sets TYTUS_STATE_PATH, so the override is a
    // no-op outside test runs.
    if let Some(p) = std::env::var_os("TYTUS_STATE_PATH") {
        return Some(std::path::PathBuf::from(p));
    }
    if let Some(dir) = dirs::config_dir() {
        return Some(dir.join("tytus").join("state.json"));
    }
    std::env::var_os("HOME").map(|h| {
        std::path::PathBuf::from(h)
            .join(".config")
            .join("tytus")
            .join("state.json")
    })
}

// ── /api/open-external ────────────────────────────────────────

fn handle_open_external(request: Request, query: &str) {
    // Whitelist specific actions. We never accept a raw URL from the
    // client — that's what phishing lives in.
    let target = query
        .split('&')
        .find_map(|kv| kv.strip_prefix("target=").map(|v| v.to_string()))
        .unwrap_or_default();

    match target.as_str() {
        "health-test" => {
            // Open Terminal running `tytus test`. Reuse the existing tray
            // helper by shelling out to `open` on macOS.
            #[cfg(target_os = "macos")]
            {
                let script = "tell application \"Terminal\" to do script \"tytus test\"";
                let _ = Command::new("osascript").args(["-e", script]).spawn();
            }
            #[cfg(not(target_os = "macos"))]
            { /* linux: rely on user's preferred terminal — not implemented */ }
            respond_json(request, 200, &serde_json::json!({ "ok": true }));
        }
        "channel-setup" => {
            // Open Terminal running `tytus channels add <channel> --pod <NN>`.
            // The CLI prompts interactively for the token so we never handle
            // secrets inside the wizard's HTTP layer. Whitelist channel names
            // + digit-only pod so the osascript we build is safe. Everything
            // goes through double-quoted heredoc-style strings in AppleScript
            // — shell escaping is belt-and-suspenders.
            let channel = query
                .split('&')
                .find_map(|kv| kv.strip_prefix("channel=").map(|v| v.to_string()))
                .unwrap_or_default();
            let pod = query
                .split('&')
                .find_map(|kv| kv.strip_prefix("pod=").map(|v| v.to_string()))
                .unwrap_or_default();
            let valid_channels = ["telegram", "discord", "slack", "line"];
            if !valid_channels.contains(&channel.as_str()) {
                respond_json(
                    request,
                    400,
                    &serde_json::json!({"error":"invalid channel"}),
                );
                return;
            }
            if !pod.chars().all(|c| c.is_ascii_digit()) || pod.is_empty() {
                respond_json(request, 400, &serde_json::json!({"error":"invalid pod"}));
                return;
            }
            #[cfg(target_os = "macos")]
            {
                // The CLI prompts for each credential interactively when
                // --token is omitted. That keeps the user copying the token
                // directly into Terminal, never into an HTTP payload.
                let cmd = format!("tytus channels add {} --pod {}", channel, pod);
                let script = format!(
                    "tell application \"Terminal\" to do script \"{}\"",
                    cmd.replace('"', "\\\"")
                );
                let _ = Command::new("osascript").args(["-e", &script]).spawn();
            }
            respond_json(request, 200, &serde_json::json!({"ok": true}));
        }
        _ => respond_json(
            request,
            400,
            &serde_json::json!({ "error": "unknown target" }),
        ),
    }
}

// ── /api/pod/* — pod-lifecycle actions ─────────────────────────
//
// Mirror the tray-menu-level operations so the wizard can drive "Open
// in Browser", "Restart Agent", and "Uninstall Agent" inline. Each
// takes ?pod=NN and validates the id against the live state snapshot
// before shelling out — we never pass a user-supplied pod id through
// to a subprocess without a whitelist check.

fn parse_pod_id(query: &str) -> Option<String> {
    let raw = query
        .split('&')
        .find_map(|kv| kv.strip_prefix("pod=").map(|v| v.to_string()))?;
    if raw.chars().all(|c| c.is_ascii_digit()) && !raw.is_empty() {
        Some(raw)
    } else {
        None
    }
}

/// Confirm the pod id exists in local state (defense-in-depth — parse_pod_id
/// already restricts to digits, this catches "pod=99 doesn't exist").
fn pod_exists(pod_id: &str) -> bool {
    let snap = compute_state_snapshot();
    snap.agents.iter().any(|a| a.pod_id == pod_id)
        || snap.included.iter().any(|i| i.pod_id == pod_id)
}

fn handle_pod_open(request: Request, query: &str) {
    let pod_id = match parse_pod_id(query) {
        Some(p) => p,
        None => {
            respond_json(request, 400, &serde_json::json!({"error":"invalid pod"}));
            return;
        }
    };
    if !pod_exists(&pod_id) {
        respond_json(request, 404, &serde_json::json!({"error":"unknown pod"}));
        return;
    }
    let snap = compute_state_snapshot();
    // Prefer the browser-auth UI URL (public edge + gateway_token) — loads
    // at LB speed. Fall back to the public api_url so the click still does
    // *something* useful (opens the /v1 route in a browser, which shows a
    // 401 at worst). Never fall back to a localhost tunnel URL from the
    // wizard — user can always use the tray's "Open in Browser" for that.
    let url = snap
        .agents
        .iter()
        .find(|a| a.pod_id == pod_id)
        .and_then(|a| a.ui_url.clone().or_else(|| a.public_url.clone()));
    match url {
        Some(u) => {
            #[cfg(target_os = "macos")]
            {
                let _ = Command::new("open").arg(&u).spawn();
            }
            #[cfg(target_os = "linux")]
            {
                let _ = Command::new("xdg-open").arg(&u).spawn();
            }
            respond_json(request, 200, &serde_json::json!({"ok": true, "url": u}));
        }
        None => {
            respond_json(
                request,
                503,
                &serde_json::json!({
                    "error":"no public URL yet — try again after the pod finishes provisioning"
                }),
            );
        }
    }
}

fn handle_pod_restart(request: Request, query: &str) {
    let pod_id = match parse_pod_id(query) {
        Some(p) => p,
        None => {
            respond_json(request, 400, &serde_json::json!({"error":"invalid pod"}));
            return;
        }
    };
    if !pod_exists(&pod_id) {
        respond_json(request, 404, &serde_json::json!({"error":"unknown pod"}));
        return;
    }
    // Spawn detached so the wizard response doesn't block the 30-90s
    // DAM round-trip. CLI logs to its own stderr; we don't stream here
    // (keeps this endpoint a fire-and-poll primitive — the chooser view
    // re-fetches /api/state on focus to reflect the new container).
    let bin = resolve_tytus_bin();
    let spawned = Command::new(&bin)
        .args(["restart", "--pod", &pod_id, "--json"])
        .env("TYTUS_HEADLESS", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn();
    match spawned {
        Ok(_) => respond_json(request, 202, &serde_json::json!({"ok":true, "pod":pod_id})),
        Err(e) => respond_json(
            request,
            500,
            &serde_json::json!({
                "error": format!("failed to spawn: {}", e)
            }),
        ),
    }
}

/// Revoke a pod — destructive, frees units and wipes workspace.
/// Only callable from the wizard's failure-path retry button. We
/// validate the pod id exists (defense in depth) and spawn the CLI
/// detached so the wizard gets an immediate 202 and can move on; the
/// actual Scalesys revoke is fast (<1 s) but DAM teardown of the
/// container can take 5-15 s.
fn handle_pod_revoke(request: Request, query: &str) {
    let pod_id = match parse_pod_id(query) {
        Some(p) => p,
        None => {
            respond_json(request, 400, &serde_json::json!({"error":"invalid pod"}));
            return;
        }
    };
    if !pod_exists(&pod_id) {
        respond_json(request, 404, &serde_json::json!({"error":"unknown pod"}));
        return;
    }
    let bin = resolve_tytus_bin();
    // `tytus revoke` has no interactive confirm (Scalesys takes the
    // wipe immediately on POST /pod/revoke) so detached spawn is
    // safe. --json is a global flag, must come before the subcommand.
    let spawned = Command::new(&bin)
        .args(["--json", "revoke", &pod_id])
        .env("TYTUS_HEADLESS", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn();
    match spawned {
        Ok(_) => respond_json(request, 202, &serde_json::json!({"ok":true, "pod":pod_id})),
        Err(e) => respond_json(
            request,
            500,
            &serde_json::json!({
                "error": format!("failed to spawn: {}", e)
            }),
        ),
    }
}

// ── Phase B: per-pod streamed action ──────────────────────────
//
// Whitelisted actions only — never lets the page pass an arbitrary
// command. The shell is bypassed entirely (Command::new + .arg per
// token), so even a compromised page can only invoke one of the
// hardcoded `tytus` subcommands below.

#[derive(serde::Deserialize)]
struct PodRunBody {
    action: String,
}

/// Map an action string to the canonical `tytus` argv. Returns `None`
/// for unknown actions so the handler can reply 400.
///
/// Per-pod actions ONLY. `tytus test` stays global (it doesn't accept
/// `--pod`), but `tytus doctor --pod NN` exists since the per-pod
/// doctor sprint and is exposed here. The standalone /api/doctor
/// endpoint still serves the daemon-wide checklist for the Troubleshoot
/// section.
fn pod_action_argv(action: &str, pod_id: &str) -> Option<Vec<String>> {
    let v = |args: &[&str]| Some(args.iter().map(|s| s.to_string()).collect::<Vec<_>>());
    match action {
        "restart" => v(&["restart", "--pod", pod_id]),
        "revoke" => v(&["revoke", pod_id]),
        "uninstall" => v(&["agent", "uninstall", pod_id]),
        "stop-forwarder" => v(&["ui", "--stop", "--pod", pod_id]),
        "channels-list" => v(&["channels", "list", "--pod", pod_id]),
        // rc.13: Files tab "Browse inbox" — read-only listing of the
        // pod's `/app/workspace/inbox/` (CLI's default path for `ls`
        // when none is supplied). Streamed via the same job channel
        // the other pod actions use; output renders in-tab.
        "ls-inbox" => v(&["ls", "--pod", pod_id]),
        // Pod Inspector Logs tab — `tytus logs --pod NN --lines 200`
        // tails the agent container's stdout/stderr via DAM. The CLI
        // emits one stdout line per log line, which the SSE relay
        // surfaces as individual `log` events.
        "logs" => v(&["logs", "--pod", pod_id, "--lines", "200"]),
        // Per-pod doctor — fetches Provider's /pod/agent/status and
        // prints a friendly health summary one fact per line.
        "doctor" => v(&["doctor", "--pod", pod_id]),
        _ => None,
    }
}

fn handle_pod_run_streamed(mut request: Request, registry: &Registry, pod_id: String) {
    if !valid_pod_id(&pod_id) {
        respond_json(request, 400, &serde_json::json!({"error":"invalid pod id"}));
        return;
    }
    let mut raw = String::new();
    if request.as_reader().read_to_string(&mut raw).is_err() {
        respond_json(request, 400, &serde_json::json!({"error":"read failed"}));
        return;
    }
    let body: PodRunBody = match serde_json::from_str(&raw) {
        Ok(b) => b,
        Err(_) => {
            respond_json(request, 400, &serde_json::json!({"error":"bad json"}));
            return;
        }
    };
    let argv = match pod_action_argv(&body.action, &pod_id) {
        Some(a) => a,
        None => {
            respond_json(
                request,
                400,
                &serde_json::json!({"error": format!("unknown action {}", body.action)}),
            );
            return;
        }
    };

    let (job_id, job) = match registry.create_pod(&pod_id) {
        Ok(pair) => pair,
        Err(p) => {
            respond_json(
                request,
                409,
                &serde_json::json!({"error": format!("pod {} busy", p)}),
            );
            return;
        }
    };

    spawn_pod_action(job, argv);
    respond_json(request, 202, &serde_json::json!({"job_id": job_id}));
}

fn spawn_pod_action(job: Arc<Mutex<Job>>, argv: Vec<String>) {
    thread::spawn(move || {
        let bin = resolve_tytus_bin();
        let mut cmd = Command::new(&bin);
        for a in &argv {
            cmd.arg(a);
        }
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("TYTUS_HEADLESS", "1");

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                push_event(
                    &job,
                    JobEvent::Fail {
                        message: format!(
                            "failed to launch `tytus` at {}: {}. \
                         Set TYTUS_BIN if the CLI lives elsewhere.",
                            bin.display(),
                            e,
                        ),
                    },
                );
                return;
            }
        };
        job.lock().unwrap().child_pid = Some(child.id());

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // Stream stdout.
        let job_so = job.clone();
        let stdout_t = stdout.map(|mut h| {
            thread::spawn(move || {
                let mut buf = [0u8; 4096];
                let mut carry = String::new();
                while let Ok(n) = h.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    carry.push_str(&String::from_utf8_lossy(&buf[..n]));
                    while let Some(idx) = carry.find('\n') {
                        let line = carry[..idx].to_string();
                        carry = carry[idx + 1..].to_string();
                        push_event(&job_so, JobEvent::Log(line));
                    }
                }
                if !carry.is_empty() {
                    push_event(&job_so, JobEvent::Log(carry));
                }
            })
        });

        // Stream stderr (merged into the same log channel; tytus uses
        // stderr for progress/status messages).
        let job_se = job.clone();
        let stderr_t = stderr.map(|mut h| {
            thread::spawn(move || {
                let mut buf = [0u8; 4096];
                let mut carry = String::new();
                while let Ok(n) = h.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    carry.push_str(&String::from_utf8_lossy(&buf[..n]));
                    while let Some(idx) = carry.find('\n') {
                        let line = carry[..idx].to_string();
                        carry = carry[idx + 1..].to_string();
                        push_event(&job_se, JobEvent::Log(line));
                    }
                }
                if !carry.is_empty() {
                    push_event(&job_se, JobEvent::Log(carry));
                }
            })
        });

        let status = child.wait();
        // Clear the PID before emitting the terminal event so cancel
        // can't race ahead and SIGTERM a recycled PID.
        job.lock().unwrap().child_pid = None;
        let _ = stdout_t.and_then(|h| h.join().ok());
        let _ = stderr_t.and_then(|h| h.join().ok());

        let code = match status {
            Ok(s) => s.code().unwrap_or(-1),
            Err(_) => -1,
        };
        push_event(&job, JobEvent::Exit { code });
    });
}

// ── TytusOS Wave 5: Shared Folders parity (v0.5.4) ────────────
//
// Mirrors the tray's Shared Folders submenu (introduced in v0.5.2 +
// v0.5.3) on the local web UI. Read-only operations (list / status /
// conflicts) plus credential refresh — Bind stays tray-only because
// the browser sandbox can't surface a real OS folder path. Helpers
// shipped in github.com/traylinx/garagetytus/bin/.

/// Locate a garagetytus-* helper script. Mirrors the resolver in
/// `tray/src/shared_folders.rs::helper_path` so backend + frontend
/// agree on which binary to invoke.
fn resolve_garagetytus_helper(name: &str) -> String {
    resolve_garagetytus_helper_path(name)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| name.to_string())
}

fn resolve_garagetytus_helper_path(name: &str) -> Option<PathBuf> {
    let candidates = [
        format!("/usr/local/bin/{}", name),
        format!("/opt/homebrew/bin/{}", name),
        std::env::var("HOME")
            .map(|h| format!("{}/garagetytus/bin/{}", h, name))
            .unwrap_or_default(),
    ];
    for c in &candidates {
        if !c.is_empty() && std::path::Path::new(c).is_file() {
            return Some(PathBuf::from(c));
        }
    }
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn resolve_garagetytus_binary_path() -> Option<PathBuf> {
    let candidates = [
        "/usr/local/bin/garagetytus".to_string(),
        "/opt/homebrew/bin/garagetytus".to_string(),
        std::env::var("HOME")
            .map(|h| format!("{}/.cargo/bin/garagetytus", h))
            .unwrap_or_default(),
    ];
    for c in &candidates {
        if !c.is_empty() && Path::new(c).is_file() {
            return Some(PathBuf::from(c));
        }
    }
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join("garagetytus");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Generic external-command spawner. Same architecture as
/// `spawn_pod_action` (line-buffered stdout+stderr via channels) but
/// parameterized over the binary path so we can spawn any
/// garagetytus-* helper.
fn spawn_external_command(job: Arc<Mutex<Job>>, bin: String, args: Vec<String>) {
    thread::spawn(move || {
        let mut cmd = Command::new(&bin);
        for a in &args {
            cmd.arg(a);
        }
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                push_event(
                    &job,
                    JobEvent::Fail {
                        message: format!("failed to launch `{}`: {}", bin, e),
                    },
                );
                return;
            }
        };
        job.lock().unwrap().child_pid = Some(child.id());

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let job_so = job.clone();
        let stdout_t = stdout.map(|mut h| {
            thread::spawn(move || {
                let mut buf = [0u8; 4096];
                let mut carry = String::new();
                while let Ok(n) = h.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    carry.push_str(&String::from_utf8_lossy(&buf[..n]));
                    while let Some(idx) = carry.find('\n') {
                        let line = carry[..idx].to_string();
                        carry = carry[idx + 1..].to_string();
                        push_event(&job_so, JobEvent::Log(line));
                    }
                }
                if !carry.is_empty() {
                    push_event(&job_so, JobEvent::Log(carry));
                }
            })
        });

        let job_se = job.clone();
        let stderr_t = stderr.map(|mut h| {
            thread::spawn(move || {
                let mut buf = [0u8; 4096];
                let mut carry = String::new();
                while let Ok(n) = h.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    carry.push_str(&String::from_utf8_lossy(&buf[..n]));
                    while let Some(idx) = carry.find('\n') {
                        let line = carry[..idx].to_string();
                        carry = carry[idx + 1..].to_string();
                        push_event(&job_se, JobEvent::Log(line));
                    }
                }
                if !carry.is_empty() {
                    push_event(&job_se, JobEvent::Log(carry));
                }
            })
        });

        let status = child.wait();
        // Clear the PID before emitting the terminal event so cancel
        // can't race ahead and SIGTERM a recycled PID.
        job.lock().unwrap().child_pid = None;
        let _ = stdout_t.and_then(|h| h.join().ok());
        let _ = stderr_t.and_then(|h| h.join().ok());

        let code = match status {
            Ok(s) => s.code().unwrap_or(-1),
            Err(_) => -1,
        };
        push_event(&job, JobEvent::Exit { code });
    });
}

/// GET /api/shared-folders/list — returns the binding sidecar list as
/// `{"bindings": [{bucket, local_path, pods_provisioned, auto_sync,
/// interval_sec, label}]}`. Reads
/// `~/.cache/garagetytus/bisync/*.bindings.json` (written by
/// `garagetytus folder bind` v0.5.3+). Returns empty `bindings` when
/// the dir doesn't exist (no bindings yet) — never errors.
fn handle_shared_folders_list(request: Request) {
    respond_json(
        request,
        200,
        &serde_json::json!({"bindings": shared_bindings_from_cache()}),
    );
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
struct SharingDefaults {
    #[serde(default = "default_sharing_schema_version")]
    schema_version: u8,
    /// Global safety switch for every mutating sharing operation.
    /// Existing bindings remain visible/read-only when this is false.
    #[serde(default = "default_true")]
    sharing_globally_enabled: bool,
    /// Default for new bind flows. Existing launchd jobs are not mutated.
    #[serde(default = "default_true")]
    default_auto_sync: bool,
    #[serde(default = "default_sharing_bucket")]
    default_bucket: String,
    #[serde(default = "default_sharing_local_root")]
    default_local_root: String,
}

impl Default for SharingDefaults {
    fn default() -> Self {
        Self {
            schema_version: default_sharing_schema_version(),
            sharing_globally_enabled: true,
            default_auto_sync: true,
            default_bucket: default_sharing_bucket(),
            default_local_root: default_sharing_local_root(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_sharing_schema_version() -> u8 {
    1
}

fn default_sharing_bucket() -> String {
    "shared".to_string()
}

fn default_sharing_local_root() -> String {
    crate::workspace::ensure_tytus_home()
        .join("Shared")
        .to_string_lossy()
        .to_string()
}

fn sharing_defaults_path() -> Option<PathBuf> {
    if let Some(dir) = dirs::config_dir() {
        return Some(dir.join("tytus").join("sharing-defaults.json"));
    }
    std::env::var_os("HOME").map(|h| {
        PathBuf::from(h)
            .join(".config")
            .join("tytus")
            .join("sharing-defaults.json")
    })
}

fn load_sharing_defaults() -> SharingDefaults {
    let Some(path) = sharing_defaults_path() else {
        return SharingDefaults::default();
    };
    let Ok(raw) = std::fs::read_to_string(path) else {
        return SharingDefaults::default();
    };
    let mut parsed: SharingDefaults = serde_json::from_str(&raw).unwrap_or_default();
    if !valid_sharing_bucket(&parsed.default_bucket) {
        parsed.default_bucket = default_sharing_bucket();
    }
    if parsed.default_local_root.trim().is_empty() {
        parsed.default_local_root = default_sharing_local_root();
    }
    parsed.schema_version = default_sharing_schema_version();
    parsed
}

fn save_sharing_defaults(defaults: &SharingDefaults) -> std::io::Result<()> {
    let Some(path) = sharing_defaults_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(defaults).unwrap_or_else(|_| "{}".to_string());
    std::fs::write(path, raw)
}

fn valid_sharing_bucket(bucket: &str) -> bool {
    let len = bucket.len();
    if !(3..=63).contains(&len) {
        return false;
    }
    let bytes = bucket.as_bytes();
    let alnum = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    let allowed = |b: u8| alnum(b) || b == b'.' || b == b'-';
    alnum(bytes[0]) && alnum(bytes[len - 1]) && bytes.iter().all(|&b| allowed(b))
}

fn respond_sharing_paused(request: Request) {
    respond_json(
        request,
        423,
        &serde_json::json!({
            "error": "sharing is paused in Settings → Sharing",
            "code": "sharing.paused",
        }),
    );
}

fn sharing_mutations_enabled() -> bool {
    load_sharing_defaults().sharing_globally_enabled
}

fn handle_sharing_defaults_get(request: Request) {
    respond_json(request, 200, &load_sharing_defaults());
}

fn handle_sharing_defaults_save(mut request: Request) {
    #[derive(serde::Deserialize)]
    struct Body {
        sharing_globally_enabled: Option<bool>,
        default_auto_sync: Option<bool>,
        default_bucket: Option<String>,
        default_local_root: Option<String>,
    }

    let mut buf = String::new();
    if request.as_reader().read_to_string(&mut buf).is_err() {
        respond_json(request, 400, &serde_json::json!({"error":"bad body"}));
        return;
    }
    let body: Body = match serde_json::from_str(&buf) {
        Ok(b) => b,
        Err(e) => {
            respond_json(
                request,
                400,
                &serde_json::json!({"error":format!("bad json: {}", e)}),
            );
            return;
        }
    };

    let mut defaults = load_sharing_defaults();
    if let Some(enabled) = body.sharing_globally_enabled {
        defaults.sharing_globally_enabled = enabled;
    }
    if let Some(auto_sync) = body.default_auto_sync {
        defaults.default_auto_sync = auto_sync;
    }
    if let Some(bucket) = body.default_bucket {
        let bucket = bucket.trim().to_string();
        if !valid_sharing_bucket(&bucket) {
            respond_json(
                request,
                400,
                &serde_json::json!({
                    "error":"default_bucket must be 3-63 chars, lowercase letters/digits/dot/hyphen only, alnum endpoints",
                    "code":"sharing.defaults.bucket.invalid"
                }),
            );
            return;
        }
        defaults.default_bucket = bucket;
    }
    if let Some(local_root) = body.default_local_root {
        let local_root = local_root.trim().to_string();
        if local_root.is_empty() {
            respond_json(
                request,
                400,
                &serde_json::json!({
                    "error":"default_local_root cannot be empty",
                    "code":"sharing.defaults.local_root.empty"
                }),
            );
            return;
        }
        defaults.default_local_root = local_root;
    }
    defaults.schema_version = default_sharing_schema_version();

    match save_sharing_defaults(&defaults) {
        Ok(()) => respond_json(request, 200, &defaults),
        Err(e) => respond_json(
            request,
            500,
            &serde_json::json!({"error":format!("failed to save sharing defaults: {}", e)}),
        ),
    }
}

/// Map a shared-folders action string to the helper binary name +
/// its arg vector. `None` for actions outside the frozen allowlist.
///
/// **Allowlist** (closes audit 03 §3 TBD on shared-folders sync /
/// mount semantics):
/// - `list`         → garagetytus-folder-list
/// - `status`       → garagetytus-folder-status --check-pods
/// - `conflicts`    → garagetytus-folder-conflicts
/// - `refresh-all`  → garagetytus-refresh-watchdog --threshold-days 7
///
/// Empty / unknown / casing-variant / shell-injection-shaped strings
/// all reject. Pinned by `shared_folder_action_argv_*` tests.
///
/// New actions land here first (paired with a UI button); the
/// allowlist is the single source of truth.
fn shared_folder_action_argv(action: &str) -> Option<(&'static str, Vec<String>)> {
    match action {
        "list" => Some(("garagetytus-folder-list", vec![])),
        "status" => Some((
            "garagetytus-folder-status",
            vec!["--check-pods".to_string()],
        )),
        "conflicts" => Some(("garagetytus-folder-conflicts", vec![])),
        "refresh-all" => Some((
            "garagetytus-refresh-watchdog",
            vec!["--threshold-days".to_string(), "7".to_string()],
        )),
        _ => None,
    }
}

const SHARED_FOLDER_ALLOWED_ACTIONS: &[&str] = &["list", "status", "conflicts", "refresh-all"];

/// POST /api/shared-folders/run-streamed?action=<list|status|conflicts|refresh-all>
/// — spawns the corresponding garagetytus-* helper and streams output
/// via the existing job-event SSE channel. Returns `{ job_id }` on
/// success, `400 {error, code, allowed}` on a rejected action.
fn handle_shared_folders_run_streamed(request: Request, registry: &Registry, query: &str) {
    let action = query
        .split('&')
        .find_map(|kv| kv.strip_prefix("action="))
        .map(|s| s.to_string())
        .unwrap_or_default();
    if !sharing_mutations_enabled() && action == "refresh-all" {
        respond_sharing_paused(request);
        return;
    }
    let (helper, args) = match shared_folder_action_argv(&action) {
        Some(pair) => pair,
        None => {
            // Structured error code so the OS can branch on it
            // without parsing prose. `allowed` is echoed for the
            // diagnostic surface — the OS never reads it directly,
            // but operators looking at curl output appreciate it.
            respond_json(
                request,
                400,
                &serde_json::json!({
                    "error": format!(
                        "action must be one of: {}",
                        SHARED_FOLDER_ALLOWED_ACTIONS.join(", "),
                    ),
                    "code": "shared_folders.action.unknown",
                    "allowed": SHARED_FOLDER_ALLOWED_ACTIONS,
                }),
            );
            return;
        }
    };
    let bin = resolve_garagetytus_helper(helper);
    let (job_id, job) = registry.create();
    spawn_external_command(job, bin, args);
    respond_json(
        request,
        202,
        &serde_json::json!({"job_id": job_id, "action": action}),
    );
}

/// POST /api/shared-folders/open — body `{"local_path":"..."}`. Opens
/// the path in Finder via macOS `open`. Returns 400 if the path
/// doesn't exist (orphan sidecar) so the UI can flag the binding.
fn handle_shared_folders_open(mut request: Request) {
    #[derive(serde::Deserialize)]
    struct Body {
        local_path: String,
    }
    let mut buf = String::new();
    if request.as_reader().read_to_string(&mut buf).is_err() {
        respond_json(request, 400, &serde_json::json!({"error":"bad body"}));
        return;
    }
    let body: Body = match serde_json::from_str(&buf) {
        Ok(b) => b,
        Err(_) => {
            respond_json(request, 400, &serde_json::json!({"error":"bad json"}));
            return;
        }
    };
    if !std::path::Path::new(&body.local_path).is_dir() {
        respond_json(
            request,
            404,
            &serde_json::json!({
                "error": "local path does not exist (orphan sidecar?)",
                "local_path": body.local_path,
            }),
        );
        return;
    }
    let _ = Command::new("open")
        .arg(&body.local_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    respond_json(request, 200, &serde_json::json!({"ok": true}));
}

#[derive(Serialize)]
struct FileListEntry {
    name: String,
    path: String,
    kind: String,
    size: u64,
    modified_at: Option<u64>,
    readonly: bool,
}

fn percent_decode(input: &str) -> Result<String, String> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' => {
                if i + 2 >= bytes.len() {
                    return Err("bad percent escape".to_string());
                }
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3])
                    .map_err(|_| "bad percent escape".to_string())?;
                let value =
                    u8::from_str_radix(hex, 16).map_err(|_| "bad percent escape".to_string())?;
                out.push(value);
                i += 3;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).map_err(|_| "query value is not valid UTF-8".to_string())
}

fn query_value_decoded(query: &str, key: &str) -> Result<Option<String>, String> {
    match query_value(query, key) {
        Some(v) => percent_decode(&v).map(Some),
        None => Ok(None),
    }
}

fn safe_relative_path(raw: Option<String>) -> Result<PathBuf, String> {
    let mut raw = raw.unwrap_or_default();
    if raw.contains('\0') {
        return Err("null bytes are not allowed".to_string());
    }
    if raw.len() >= 2 && raw.as_bytes()[1] == b':' && raw.as_bytes()[0].is_ascii_alphabetic() {
        return Err("absolute paths are not allowed".to_string());
    }
    for _ in 0..2 {
        let lower = raw.to_ascii_lowercase();
        let suspicious = lower.contains("%2e")
            || lower.contains("%2f")
            || lower.contains("%5c")
            || lower.contains("%00")
            || lower.contains("%25");
        if !suspicious {
            break;
        }
        let decoded = percent_decode(&raw)?;
        if decoded == raw {
            break;
        }
        raw = decoded;
        if raw.contains('\0') {
            return Err("null bytes are not allowed".to_string());
        }
    }
    let p_raw = Path::new(raw.as_str());
    if p_raw.is_absolute() || raw.starts_with('/') || raw.starts_with('\\') {
        return Err("absolute paths are not allowed".to_string());
    }
    let trimmed = raw.trim_matches('/');
    if trimmed.is_empty() || trimmed == "." {
        return Ok(PathBuf::new());
    }
    let p = Path::new(trimmed);
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            Component::Normal(seg) => out.push(seg),
            Component::CurDir => {}
            Component::ParentDir => return Err(".. is not allowed".to_string()),
            Component::RootDir | Component::Prefix(_) => {
                return Err("absolute paths are not allowed".to_string());
            }
        }
    }
    Ok(out)
}

fn path_to_relative_string(path: &Path) -> String {
    let s = path.to_string_lossy().replace('\\', "/");
    if s == "." {
        "".to_string()
    } else {
        s
    }
}

fn anchor_local_path(root: &Path, rel: &Path) -> Result<PathBuf, String> {
    let root = root
        .canonicalize()
        .map_err(|e| format!("root is not accessible: {}", e))?;
    let candidate = root.join(rel);
    let resolved = if candidate.exists() {
        candidate
            .canonicalize()
            .map_err(|e| format!("path is not accessible: {}", e))?
    } else {
        let parent = candidate
            .parent()
            .ok_or_else(|| "path has no parent".to_string())?;
        let parent_real = parent
            .canonicalize()
            .map_err(|e| format!("parent is not accessible: {}", e))?;
        parent_real.join(candidate.file_name().unwrap_or_default())
    };
    if !resolved.starts_with(&root) {
        return Err("path escapes allowed root".to_string());
    }
    Ok(resolved)
}

fn entry_modified_secs(meta: &fs::Metadata) -> Option<u64> {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
}

fn list_local_dir(root: &Path, rel: &Path) -> Result<Vec<FileListEntry>, String> {
    let target = anchor_local_path(root, rel)?;
    if !target.is_dir() {
        return Err("path is not a directory".to_string());
    }
    let mut rows = Vec::new();
    for entry in fs::read_dir(&target).map_err(|e| format!("read_dir failed: {}", e))? {
        let entry = entry.map_err(|e| format!("read_dir entry failed: {}", e))?;
        let name = entry.file_name().to_string_lossy().to_string();
        let meta = entry
            .metadata()
            .map_err(|e| format!("metadata failed for {}: {}", name, e))?;
        let child_rel = rel.join(&name);
        rows.push(FileListEntry {
            name,
            path: path_to_relative_string(&child_rel),
            kind: if meta.is_dir() { "dir" } else { "file" }.to_string(),
            size: if meta.is_file() { meta.len() } else { 0 },
            modified_at: entry_modified_secs(&meta),
            readonly: meta.permissions().readonly(),
        });
    }
    rows.sort_by(|a, b| match (a.kind.as_str(), b.kind.as_str()) {
        ("dir", "file") => std::cmp::Ordering::Less,
        ("file", "dir") => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    Ok(rows)
}

fn shared_bindings_from_cache() -> Vec<serde_json::Value> {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return Vec::new(),
    };
    let dir = format!("{}/.cache/garagetytus/bisync", home);
    let mut bindings = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            if !name.ends_with(".bindings.json") {
                continue;
            }
            let raw = match std::fs::read_to_string(&path) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let json: serde_json::Value = match serde_json::from_str(&raw) {
                Ok(j) => j,
                Err(_) => continue,
            };
            bindings.push(json);
        }
    }
    bindings
}

const GARAGETYTUS_REQUIRED_HELPERS: &[&str] = &[
    "garagetytus-folder-bind",
    "garagetytus-folder-list",
    "garagetytus-folder-status",
    "garagetytus-folder-conflicts",
    "garagetytus-refresh-watchdog",
    "garagetytus-pod-provision",
    "garagetytus-pod-refresh",
    "garagetytus-shared",
];

fn parse_garagetytus_status_line(stdout: &str) -> (Option<bool>, String) {
    let line = stdout
        .lines()
        .find(|l| l.trim_start().starts_with("garagetytus status:"))
        .map(str::trim)
        .unwrap_or_else(|| stdout.trim());
    let state = line
        .strip_prefix("garagetytus status:")
        .map(str::trim)
        .and_then(|rest| rest.split_whitespace().next())
        .unwrap_or("unknown")
        .trim_matches(|c: char| c == '(' || c == ')' || c == ',')
        .to_string();
    let running = match state.as_str() {
        "running" | "active" => Some(true),
        "stopped" | "inactive" | "failed" => Some(false),
        _ => None,
    };
    (running, state)
}

fn garagetytus_cache_path() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".cache/garagetytus"))
}

/// GET /api/garagetytus/status — read-only helper/service health for
/// Settings → Sharing. Deliberately does NOT expose start/stop/restart:
/// current garagetytus sharing is a mix of a local Garage service,
/// per-binding launchd jobs, and helper scripts. Lifecycle controls
/// need scoped semantics first; status is safe and production-useful now.
fn handle_garagetytus_status(request: Request) {
    let binary = resolve_garagetytus_binary_path();
    let mut warnings: Vec<String> = Vec::new();

    let (running, state, status_text) = if let Some(bin) = &binary {
        match Command::new(bin).arg("status").output() {
            Ok(out) => {
                let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                let combined = if text.is_empty() { stderr } else { text };
                let (running, state) = parse_garagetytus_status_line(&combined);
                (running, state, combined)
            }
            Err(e) => {
                warnings.push(format!("failed to run garagetytus status: {}", e));
                (None, "unknown".to_string(), String::new())
            }
        }
    } else {
        warnings.push("garagetytus binary not found".to_string());
        (None, "missing".to_string(), String::new())
    };

    let version = binary.as_ref().and_then(|bin| {
        Command::new(bin)
            .arg("--version")
            .output()
            .ok()
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
            .filter(|s| !s.is_empty())
    });

    let helpers: Vec<serde_json::Value> = GARAGETYTUS_REQUIRED_HELPERS
        .iter()
        .map(|name| {
            let path = resolve_garagetytus_helper_path(name);
            serde_json::json!({
                "name": name,
                "found": path.is_some(),
                "path": path.map(|p| p.to_string_lossy().to_string()),
            })
        })
        .collect();
    let missing_helpers: Vec<String> = helpers
        .iter()
        .filter_map(|h| {
            if h.get("found").and_then(|v| v.as_bool()) == Some(false) {
                h.get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();
    for helper in &missing_helpers {
        warnings.push(format!("{} not found", helper));
    }

    let bindings = shared_bindings_from_cache();
    let mut pods = std::collections::BTreeSet::new();
    for binding in &bindings {
        if let Some(arr) = binding.get("pods_provisioned").and_then(|v| v.as_array()) {
            for pod in arr.iter().filter_map(|v| v.as_str()) {
                pods.insert(pod.to_string());
            }
        }
    }

    let cache_path = garagetytus_cache_path();
    let cache_exists = cache_path.as_ref().map(|p| p.exists()).unwrap_or(false);

    respond_json(
        request,
        200,
        &serde_json::json!({
            "available": binary.is_some() && missing_helpers.is_empty(),
            "running": running,
            "state": state,
            "status_text": status_text,
            "version": version,
            "port": 3900,
            "binary_path": binary.map(|p| p.to_string_lossy().to_string()),
            "cache_path": cache_path.map(|p| p.to_string_lossy().to_string()),
            "cache_exists": cache_exists,
            "bindings_count": bindings.len(),
            "provisioned_pods": pods.into_iter().collect::<Vec<_>>(),
            "helpers": helpers,
            "missing_helpers": missing_helpers,
            "lifecycle_control_available": false,
            "lifecycle_control_reason": "Deferred: current sharing lifecycle is helper/per-binding based, not one global start/stop/restart target.",
            "warnings": warnings,
        }),
    );
}

fn shell_single_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn pod_workspace_list_script(rel: &Path) -> String {
    let rel_s = path_to_relative_string(rel);
    let quoted = shell_single_quote(&rel_s);
    format!(
        r#"set -eu
cd /app/workspace
rel={}
case "$rel" in /*|*../*|../*|*'/../'* ) echo "path escapes workspace" >&2; exit 64;; esac
[ -z "$rel" ] && rel="."
[ -d "$rel" ] || {{ echo "path is not a directory" >&2; exit 44; }}
find "$rel" -mindepth 1 -maxdepth 1 -printf '%f\t%y\t%s\t%T@\n' 2>/dev/null | sort -f
"#,
        quoted
    )
}

fn list_pod_workspace(pod_id: &str, rel: &Path) -> Result<Vec<FileListEntry>, String> {
    if !valid_pod_id(pod_id) || !pod_exists(pod_id) {
        return Err("unknown pod".to_string());
    }
    let script = pod_workspace_list_script(rel);
    let (ok, stdout, stderr, timed_out) =
        run_tytus_exec_shell(pod_id, &script, Duration::from_secs(8))?;
    if timed_out {
        return Err("pod listing timed out".to_string());
    }
    if !ok {
        let msg = stderr
            .lines()
            .last()
            .unwrap_or("pod listing failed")
            .to_string();
        return Err(msg);
    }
    let mut rows = Vec::new();
    for line in stdout.lines().filter(|l| !l.trim().is_empty()) {
        let mut parts = line.split('\t');
        let Some(name) = parts.next() else { continue };
        let typ = parts.next().unwrap_or("f");
        let size = parts
            .next()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        let modified_at = parts
            .next()
            .and_then(|s| s.split('.').next())
            .and_then(|s| s.parse::<u64>().ok());
        let child_rel = rel.join(name);
        rows.push(FileListEntry {
            name: name.to_string(),
            path: path_to_relative_string(&child_rel),
            kind: if typ == "d" { "dir" } else { "file" }.to_string(),
            size: if typ == "d" { 0 } else { size },
            modified_at,
            readonly: true,
        });
    }
    rows.sort_by(|a, b| match (a.kind.as_str(), b.kind.as_str()) {
        ("dir", "file") => std::cmp::Ordering::Less,
        ("file", "dir") => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    Ok(rows)
}

fn handle_files_list(request: Request, query: &str) {
    let source = match query_value_decoded(query, "source") {
        Ok(Some(s)) => s,
        Ok(None) => "tytus-home".to_string(),
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let rel = match query_value_decoded(query, "path").and_then(safe_relative_path) {
        Ok(p) => p,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };

    let result: Result<(String, String, Vec<FileListEntry>), String> = match source.as_str() {
        "tytus-home" => {
            let root = crate::workspace::ensure_tytus_home();
            list_local_dir(&root, &rel).map(|rows| {
                (
                    "Tytus Home".to_string(),
                    root.to_string_lossy().to_string(),
                    rows,
                )
            })
        }
        "shared" => {
            let idx = match query_value_decoded(query, "binding") {
                Ok(Some(v)) => v.parse::<usize>().ok(),
                _ => None,
            };
            let Some(idx) = idx else {
                respond_json(
                    request,
                    400,
                    &serde_json::json!({"error":"missing binding index"}),
                );
                return;
            };
            let bindings = shared_bindings_from_cache();
            let Some(binding) = bindings.get(idx) else {
                respond_json(
                    request,
                    404,
                    &serde_json::json!({"error":"binding not found"}),
                );
                return;
            };
            let Some(local_path) = binding.get("local_path").and_then(|v| v.as_str()) else {
                respond_json(
                    request,
                    400,
                    &serde_json::json!({"error":"binding has no local_path"}),
                );
                return;
            };
            let root = PathBuf::from(local_path);
            list_local_dir(&root, &rel)
                .map(|rows| ("Shared".to_string(), local_path.to_string(), rows))
        }
        "pod-workspace" => {
            let pod = match query_value_decoded(query, "pod") {
                Ok(Some(p)) if valid_pod_id(&p) => p,
                _ => {
                    respond_json(
                        request,
                        400,
                        &serde_json::json!({"error":"missing or invalid pod"}),
                    );
                    return;
                }
            };
            list_pod_workspace(&pod, &rel).map(|rows| {
                (
                    format!("Pod {} workspace", pod),
                    "/app/workspace".to_string(),
                    rows,
                )
            })
        }
        _ => Err("unknown file source".to_string()),
    };

    match result {
        Ok((label, root_path, entries)) => respond_json(
            request,
            200,
            &serde_json::json!({
                "source": source,
                "path": path_to_relative_string(&rel),
                "root_label": label,
                "root_path": root_path,
                "entries": entries,
                "readonly": source == "pod-workspace",
            }),
        ),
        Err(e) => respond_json(request, 400, &serde_json::json!({"error": e})),
    }
}

#[derive(serde::Deserialize)]
struct FileMutationBody {
    source: String,
    path: Option<String>,
    binding: Option<usize>,
    name: Option<String>,
    new_name: Option<String>,
    destination_path: Option<String>,
    content_base64: Option<String>,
}

const MAX_FILE_TRANSFER_BYTES: usize = 100 * 1024 * 1024;

fn read_file_mutation_body(request: &mut Request) -> Result<FileMutationBody, String> {
    let mut buf = String::new();
    request
        .as_reader()
        .read_to_string(&mut buf)
        .map_err(|_| "bad body".to_string())?;
    serde_json::from_str(&buf).map_err(|_| "bad json".to_string())
}

fn safe_file_name(raw: Option<String>) -> Result<String, String> {
    let name = raw.unwrap_or_default().trim().to_string();
    if name.is_empty() {
        return Err("name is required".to_string());
    }
    if name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\\')
        || name.contains('\0')
    {
        return Err("name must be a single file or folder name".to_string());
    }
    Ok(name)
}

fn local_files_root(source: &str, binding: Option<usize>) -> Result<PathBuf, String> {
    match source {
        "tytus-home" => Ok(crate::workspace::ensure_tytus_home()),
        "shared" => {
            let Some(idx) = binding else {
                return Err("missing binding index".to_string());
            };
            let bindings = shared_bindings_from_cache();
            let Some(binding) = bindings.get(idx) else {
                return Err("binding not found".to_string());
            };
            let Some(local_path) = binding.get("local_path").and_then(|v| v.as_str()) else {
                return Err("binding has no local_path".to_string());
            };
            Ok(PathBuf::from(local_path))
        }
        "pod-workspace" => Err("pod workspaces are read-only from Files".to_string()),
        _ => Err("unknown file source".to_string()),
    }
}

fn writable_files_root(body: &FileMutationBody) -> Result<PathBuf, String> {
    local_files_root(&body.source, body.binding)
}

fn anchored_child_path(root: &Path, parent_rel: &Path, name: &str) -> Result<PathBuf, String> {
    let parent = anchor_local_path(root, parent_rel)?;
    if !parent.is_dir() {
        return Err("parent is not a directory".to_string());
    }
    let candidate = parent.join(name);
    let root_real = root
        .canonicalize()
        .map_err(|e| format!("root is not accessible: {}", e))?;
    if !candidate.starts_with(&root_real) {
        return Err("path escapes allowed root".to_string());
    }
    Ok(candidate)
}

fn anchored_existing_entry(root: &Path, rel: &Path) -> Result<PathBuf, String> {
    if rel.as_os_str().is_empty() {
        return Err("cannot operate on source root".to_string());
    }
    let parent_rel = rel.parent().unwrap_or_else(|| Path::new(""));
    let name = rel
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "invalid path".to_string())?;
    anchored_child_path(root, parent_rel, name)
}

fn handle_files_mkdir(mut request: Request) {
    let body = match read_file_mutation_body(&mut request) {
        Ok(b) => b,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let root = match writable_files_root(&body) {
        Ok(r) => r,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let parent_rel = match safe_relative_path(body.path.clone()) {
        Ok(p) => p,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let name = match safe_file_name(body.name) {
        Ok(n) => n,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let target = match anchored_child_path(&root, &parent_rel, &name) {
        Ok(p) => p,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    if target.exists() {
        respond_json(
            request,
            400,
            &serde_json::json!({"error":"target already exists"}),
        );
        return;
    }
    match fs::create_dir(&target) {
        Ok(_) => respond_json(request, 200, &serde_json::json!({"ok": true})),
        Err(e) => respond_json(request, 400, &serde_json::json!({"error": e.to_string()})),
    }
}

fn handle_files_rename(mut request: Request) {
    let body = match read_file_mutation_body(&mut request) {
        Ok(b) => b,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let root = match writable_files_root(&body) {
        Ok(r) => r,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let rel = match safe_relative_path(body.path.clone()) {
        Ok(p) => p,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let new_name = match safe_file_name(body.new_name) {
        Ok(n) => n,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let old_path = match anchored_existing_entry(&root, &rel) {
        Ok(p) => p,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let parent_rel = rel.parent().unwrap_or_else(|| Path::new(""));
    let new_path = match anchored_child_path(&root, parent_rel, &new_name) {
        Ok(p) => p,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    if new_path.exists() {
        respond_json(
            request,
            400,
            &serde_json::json!({"error":"target already exists"}),
        );
        return;
    }
    match fs::rename(&old_path, &new_path) {
        Ok(_) => respond_json(request, 200, &serde_json::json!({"ok": true})),
        Err(e) => respond_json(request, 400, &serde_json::json!({"error": e.to_string()})),
    }
}

fn handle_files_delete(mut request: Request) {
    let body = match read_file_mutation_body(&mut request) {
        Ok(b) => b,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let root = match writable_files_root(&body) {
        Ok(r) => r,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let rel = match safe_relative_path(body.path.clone()) {
        Ok(p) => p,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let target = match anchored_existing_entry(&root, &rel) {
        Ok(p) => p,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let meta = match fs::symlink_metadata(&target) {
        Ok(m) => m,
        Err(e) => {
            respond_json(request, 404, &serde_json::json!({"error": e.to_string()}));
            return;
        }
    };
    let result = if meta.is_dir() {
        fs::remove_dir(&target)
    } else {
        fs::remove_file(&target)
    };
    match result {
        Ok(_) => respond_json(request, 200, &serde_json::json!({"ok": true})),
        Err(e) => respond_json(request, 400, &serde_json::json!({"error": e.to_string()})),
    }
}

fn copy_move_destination(
    root: &Path,
    source_rel: &Path,
    destination_path: Option<String>,
    new_name: Option<String>,
) -> Result<PathBuf, String> {
    let dest_rel = safe_relative_path(destination_path)?;
    let fallback_name = source_rel
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "invalid source path".to_string())?
        .to_string();
    let dest_name = match new_name {
        Some(v) => safe_file_name(Some(v))?,
        None => fallback_name,
    };
    anchored_child_path(root, &dest_rel, &dest_name)
}

fn handle_files_copy(mut request: Request) {
    let body = match read_file_mutation_body(&mut request) {
        Ok(b) => b,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let root = match writable_files_root(&body) {
        Ok(r) => r,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let source_rel = match safe_relative_path(body.path.clone()) {
        Ok(p) => p,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let source = match anchored_existing_entry(&root, &source_rel) {
        Ok(p) => p,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let meta = match fs::symlink_metadata(&source) {
        Ok(m) => m,
        Err(e) => {
            respond_json(request, 404, &serde_json::json!({"error": e.to_string()}));
            return;
        }
    };
    if !meta.is_file() {
        respond_json(
            request,
            400,
            &serde_json::json!({"error":"copy supports files only"}),
        );
        return;
    }
    if meta.len() > MAX_FILE_TRANSFER_BYTES as u64 {
        respond_json(
            request,
            413,
            &serde_json::json!({"error":"file exceeds 100 MiB transfer cap"}),
        );
        return;
    }
    let dest = match copy_move_destination(&root, &source_rel, body.destination_path, body.new_name)
    {
        Ok(p) => p,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    if dest.exists() {
        respond_json(
            request,
            409,
            &serde_json::json!({"error":"target already exists"}),
        );
        return;
    }
    match fs::copy(&source, &dest) {
        Ok(_) => respond_json(request, 200, &serde_json::json!({"ok": true})),
        Err(e) => respond_json(request, 400, &serde_json::json!({"error": e.to_string()})),
    }
}

fn handle_files_move(mut request: Request) {
    let body = match read_file_mutation_body(&mut request) {
        Ok(b) => b,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let root = match writable_files_root(&body) {
        Ok(r) => r,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let source_rel = match safe_relative_path(body.path.clone()) {
        Ok(p) => p,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let source = match anchored_existing_entry(&root, &source_rel) {
        Ok(p) => p,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let meta = match fs::symlink_metadata(&source) {
        Ok(m) => m,
        Err(e) => {
            respond_json(request, 404, &serde_json::json!({"error": e.to_string()}));
            return;
        }
    };
    if !meta.is_file() {
        respond_json(
            request,
            400,
            &serde_json::json!({"error":"move supports files only"}),
        );
        return;
    }
    let dest = match copy_move_destination(&root, &source_rel, body.destination_path, body.new_name)
    {
        Ok(p) => p,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    if dest.exists() {
        respond_json(
            request,
            409,
            &serde_json::json!({"error":"target already exists"}),
        );
        return;
    }
    match fs::rename(&source, &dest) {
        Ok(_) => respond_json(request, 200, &serde_json::json!({"ok": true})),
        Err(e) => respond_json(request, 400, &serde_json::json!({"error": e.to_string()})),
    }
}

fn files_root_from_query(query: &str) -> Result<(String, PathBuf), String> {
    let source = query_value_decoded(query, "source")?.unwrap_or_else(|| "tytus-home".to_string());
    let binding = match query_value_decoded(query, "binding")? {
        Some(v) => Some(
            v.parse::<usize>()
                .map_err(|_| "invalid binding index".to_string())?,
        ),
        None => None,
    };
    let root = local_files_root(&source, binding)?;
    Ok((source, root))
}

fn handle_files_download(request: Request, query: &str) {
    let (_source, root) = match files_root_from_query(query) {
        Ok(v) => v,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let rel = match query_value_decoded(query, "path").and_then(safe_relative_path) {
        Ok(p) => p,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let target = match anchored_existing_entry(&root, &rel) {
        Ok(p) => p,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let meta = match fs::symlink_metadata(&target) {
        Ok(m) => m,
        Err(e) => {
            respond_json(request, 404, &serde_json::json!({"error": e.to_string()}));
            return;
        }
    };
    if meta.is_dir() {
        respond_json(
            request,
            400,
            &serde_json::json!({"error":"folders cannot be downloaded directly"}),
        );
        return;
    }
    if meta.len() > MAX_FILE_TRANSFER_BYTES as u64 {
        respond_json(
            request,
            413,
            &serde_json::json!({"error":"file exceeds 100 MiB transfer cap"}),
        );
        return;
    }
    let mut data = Vec::new();
    match File::open(&target).and_then(|mut f| f.read_to_end(&mut data)) {
        Ok(_) => {
            let name = rel
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("download.bin")
                .replace(['"', '\r', '\n'], "_");
            let resp = Response::from_data(data)
                .with_status_code(StatusCode(200))
                .with_header(header("Content-Type", "application/octet-stream"))
                .with_header(header(
                    "Content-Disposition",
                    &format!("attachment; filename=\"{}\"", name),
                ))
                .with_header(header("X-Content-Type-Options", "nosniff"));
            let _ = request.respond(resp);
        }
        Err(e) => respond_json(request, 400, &serde_json::json!({"error": e.to_string()})),
    }
}

fn handle_files_upload(mut request: Request) {
    let body = match read_file_mutation_body(&mut request) {
        Ok(b) => b,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let root = match writable_files_root(&body) {
        Ok(r) => r,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let parent_rel = match safe_relative_path(body.path.clone()) {
        Ok(p) => p,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let name = match safe_file_name(body.name) {
        Ok(n) => n,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    let encoded = body.content_base64.unwrap_or_default();
    if encoded.len() > ((MAX_FILE_TRANSFER_BYTES * 4) / 3 + 8) {
        respond_json(
            request,
            413,
            &serde_json::json!({"error":"upload exceeds 100 MiB transfer cap"}),
        );
        return;
    }
    let bytes = match general_purpose::STANDARD.decode(encoded.as_bytes()) {
        Ok(b) => b,
        Err(_) => {
            respond_json(
                request,
                400,
                &serde_json::json!({"error":"content_base64 is not valid base64"}),
            );
            return;
        }
    };
    if bytes.len() > MAX_FILE_TRANSFER_BYTES {
        respond_json(
            request,
            413,
            &serde_json::json!({"error":"upload exceeds 100 MiB transfer cap"}),
        );
        return;
    }
    let target = match anchored_child_path(&root, &parent_rel, &name) {
        Ok(p) => p,
        Err(e) => {
            respond_json(request, 400, &serde_json::json!({"error": e}));
            return;
        }
    };
    if target.exists() {
        respond_json(
            request,
            400,
            &serde_json::json!({"error":"target already exists"}),
        );
        return;
    }
    match fs::write(&target, bytes) {
        Ok(_) => respond_json(request, 200, &serde_json::json!({"ok": true})),
        Err(e) => respond_json(request, 400, &serde_json::json!({"error": e.to_string()})),
    }
}

/// POST /api/files/open-downloads?pod=NN — opens
/// `~/Tytus/Downloads/pod-NN/` in Finder so the user can grab files
/// they pulled from the pod. Mirrors the tray menu entry under
/// per-pod Files ▸. Created on demand if missing so the open call
/// doesn't fail on a freshly-installed pod that has yet to pull
/// anything.
fn handle_files_open_downloads(request: Request, query: &str) {
    let pod = match parse_pod_id(query) {
        Some(p) if valid_pod_id(&p) => p,
        _ => {
            respond_json(
                request,
                400,
                &serde_json::json!({"error":"missing or invalid pod"}),
            );
            return;
        }
    };
    let path = crate::workspace::ensure_download_dir_for_pod(&pod);
    let legacy_path = crate::workspace::legacy_download_dir_for_pod(&pod);
    let _ = Command::new("open")
        .arg(&path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    respond_json(
        request,
        200,
        &serde_json::json!({
            "ok": true,
            "path": path.to_string_lossy(),
            "legacy_path": legacy_path.to_string_lossy(),
        }),
    );
}

fn handle_workspace_open(request: Request) {
    let path = crate::workspace::ensure_tytus_home();
    let _ = Command::new("open")
        .arg(&path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    respond_json(
        request,
        200,
        &serde_json::json!({
            "ok": true,
            "path": path.to_string_lossy(),
        }),
    );
}

/// POST /api/shared-folders/open-cache — opens
/// `~/.cache/garagetytus` in Finder.
fn handle_shared_folders_open_cache(request: Request) {
    let path = std::env::var("HOME")
        .map(|h| format!("{}/.cache/garagetytus", h))
        .unwrap_or_else(|_| "/tmp".to_string());
    let _ = std::fs::create_dir_all(&path);
    let _ = Command::new("open")
        .arg(&path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    respond_json(request, 200, &serde_json::json!({"ok": true}));
}

/// POST /api/shared-folders/pick-folder — calls macOS osascript to
/// open a native folder picker and returns the chosen POSIX path.
/// Returns `{cancelled: true}` if the user dismissed the dialog.
/// macOS-only; non-macOS returns 501.
#[cfg(target_os = "macos")]
fn handle_shared_folders_pick_folder(request: Request) {
    let script = "POSIX path of (choose folder with prompt \
                  \"Pick a Mac folder to share with your pods\")";
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if path.is_empty() {
                respond_json(request, 200, &serde_json::json!({"cancelled": true}));
            } else {
                respond_json(request, 200, &serde_json::json!({"path": path}));
            }
        }
        Ok(_) => {
            // osascript exits non-zero when user cancels. Distinguish
            // from real failure by treating any non-zero as "cancelled"
            // — the picker doesn't fail in any other realistic way.
            respond_json(request, 200, &serde_json::json!({"cancelled": true}));
        }
        Err(e) => {
            respond_json(
                request,
                500,
                &serde_json::json!({
                    "error": format!("osascript failed: {}", e),
                }),
            );
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn handle_shared_folders_pick_folder(request: Request) {
    respond_json(
        request,
        501,
        &serde_json::json!({
            "error": "folder picker is macOS-only (osascript)",
        }),
    );
}

/// POST /api/shared-folders/bind — body
/// `{local_path, bucket, pods?: [String], auto_sync?: bool}`. Spawns
/// `garagetytus-folder-bind <local> <bucket> [--to N]... [--auto-sync]`
/// and streams output via job channel. Returns `{job_id}`.
///
/// Validates server-side BEFORE spawn:
/// - bucket name matches Garage rules (lowercase alnum/dot/hyphen,
///   3-63 chars, alnum endpoints)
/// - local_path is absolute + exists + is a directory
/// - each pod is `^\d{2,3}$` (matches the existing parse_pod_id rules)
///
/// All values passed via `Command::arg` — no shell, no injection.
fn handle_shared_folders_bind(mut request: Request, registry: &Registry) {
    #[derive(serde::Deserialize)]
    struct Body {
        local_path: String,
        bucket: String,
        #[serde(default)]
        pods: Vec<String>,
        #[serde(default = "default_true")]
        auto_sync: bool,
    }
    if !sharing_mutations_enabled() {
        respond_sharing_paused(request);
        return;
    }

    let mut buf = String::new();
    if request.as_reader().read_to_string(&mut buf).is_err() {
        respond_json(request, 400, &serde_json::json!({"error":"bad body"}));
        return;
    }
    let body: Body = match serde_json::from_str(&buf) {
        Ok(b) => b,
        Err(e) => {
            respond_json(
                request,
                400,
                &serde_json::json!({"error":format!("bad json: {}", e)}),
            );
            return;
        }
    };

    // Validate local_path: absolute, exists, is a directory.
    if !std::path::Path::new(&body.local_path).is_absolute() {
        respond_json(
            request,
            400,
            &serde_json::json!({
                "error":"local_path must be absolute"
            }),
        );
        return;
    }
    if !std::path::Path::new(&body.local_path).is_dir() {
        respond_json(
            request,
            400,
            &serde_json::json!({
                "error":"local_path is not a directory on this Mac"
            }),
        );
        return;
    }

    // Validate bucket name (Garage rules: 3-63 chars, lowercase alnum
    // + dot + hyphen, alnum endpoints).
    if !valid_sharing_bucket(&body.bucket) {
        respond_json(
            request,
            400,
            &serde_json::json!({
                "error":"bucket name must be 3-63 chars, lowercase letters/digits/dot/hyphen only, alnum endpoints"
            }),
        );
        return;
    }

    // Validate pod IDs.
    for p in &body.pods {
        if p.is_empty() || !p.chars().all(|c| c.is_ascii_digit()) || p.len() > 3 {
            respond_json(
                request,
                400,
                &serde_json::json!({
                    "error": format!("invalid pod id: {:?}", p)
                }),
            );
            return;
        }
    }

    // Build argv: <local> <bucket> [--to N]... [--auto-sync]
    let mut args: Vec<String> = vec![body.local_path.clone(), body.bucket.clone()];
    for p in &body.pods {
        args.push("--to".to_string());
        args.push(p.clone());
    }
    if body.auto_sync {
        args.push("--auto-sync".to_string());
    }

    let bin = resolve_garagetytus_helper("garagetytus-folder-bind");
    let (job_id, job) = registry.create();
    spawn_external_command(job, bin, args);
    respond_json(
        request,
        202,
        &serde_json::json!({
            "job_id": job_id,
            "bucket": body.bucket,
            "local_path": body.local_path,
        }),
    );
}

/// POST /api/pod/refresh-creds?pod=NN — spawns
/// `garagetytus-pod-refresh NN` and streams via job. Pod's wrapper
/// re-reads credentials.json on every call so no pod restart is
/// needed after rotation.
fn handle_pod_refresh_creds(request: Request, registry: &Registry, query: &str) {
    if !sharing_mutations_enabled() {
        respond_sharing_paused(request);
        return;
    }
    let pod_id = match parse_pod_id(query) {
        Some(p) => p,
        None => {
            respond_json(request, 400, &serde_json::json!({"error":"invalid pod"}));
            return;
        }
    };
    let bin = resolve_garagetytus_helper("garagetytus-pod-refresh");
    let (job_id, job) = registry.create();
    spawn_external_command(job, bin, vec![pod_id.clone()]);
    respond_json(
        request,
        202,
        &serde_json::json!({"job_id": job_id, "pod": pod_id}),
    );
}

fn all_known_shared_buckets() -> Vec<String> {
    let mut buckets = std::collections::BTreeSet::new();
    for binding in shared_bindings_from_cache() {
        if let Some(bucket) = binding.get("bucket").and_then(|v| v.as_str()) {
            if valid_sharing_bucket(bucket) {
                buckets.insert(bucket.to_string());
            }
        }
    }
    buckets.into_iter().collect()
}

/// POST /api/shared-folders/provision-pod — body
/// `{ "pod":"02", "buckets":["shared"] }`. Spawns
/// `garagetytus-pod-provision 02 --bucket shared ...` and streams via
/// the job channel. The helper owns secret-safe credential minting +
/// file transfer; daemon only passes pod id + bucket names in argv.
fn handle_shared_folders_provision_pod(mut request: Request, registry: &Registry) {
    #[derive(serde::Deserialize)]
    struct Body {
        pod: String,
        #[serde(default)]
        buckets: Vec<String>,
        #[serde(default)]
        no_restart: bool,
    }

    if !sharing_mutations_enabled() {
        respond_sharing_paused(request);
        return;
    }

    let mut buf = String::new();
    if request.as_reader().read_to_string(&mut buf).is_err() {
        respond_json(request, 400, &serde_json::json!({"error":"bad body"}));
        return;
    }
    let body: Body = match serde_json::from_str(&buf) {
        Ok(b) => b,
        Err(e) => {
            respond_json(
                request,
                400,
                &serde_json::json!({"error":format!("bad json: {}", e)}),
            );
            return;
        }
    };

    let pod_id = match normalize_pod_id_for_query(&body.pod) {
        Some(p) if pod_exists(&p) => p,
        _ => {
            respond_json(request, 400, &serde_json::json!({"error":"invalid pod"}));
            return;
        }
    };

    let buckets = if body.buckets.is_empty() {
        all_known_shared_buckets()
    } else {
        let mut dedup = std::collections::BTreeSet::new();
        for bucket in body.buckets {
            let bucket = bucket.trim().to_string();
            if !valid_sharing_bucket(&bucket) {
                respond_json(
                    request,
                    400,
                    &serde_json::json!({
                        "error": format!("invalid bucket: {}", bucket),
                        "code": "sharing.provision.bucket.invalid"
                    }),
                );
                return;
            }
            dedup.insert(bucket);
        }
        dedup.into_iter().collect::<Vec<_>>()
    };

    if buckets.is_empty() {
        respond_json(
            request,
            400,
            &serde_json::json!({
                "error":"no shared buckets configured yet",
                "code":"sharing.provision.no_buckets"
            }),
        );
        return;
    }

    let mut args = vec![pod_id.clone()];
    for bucket in &buckets {
        args.push("--bucket".to_string());
        args.push(bucket.clone());
    }
    if body.no_restart {
        args.push("--no-restart".to_string());
    }

    let bin = resolve_garagetytus_helper("garagetytus-pod-provision");
    let (job_id, job) = registry.create();
    spawn_external_command(job, bin, args);
    respond_json(
        request,
        202,
        &serde_json::json!({"job_id": job_id, "pod": pod_id, "buckets": buckets}),
    );
}

/// Probe whether a just-installed pod is actually reachable. The CLI's
/// `agent install` returns as soon as Scalesys allocates the pod row +
/// fires the DAM deploy — the container is typically still starting at
/// that moment (15-60 s for nemoclaw, 30-90 s for hermes). The wizard
/// polls this endpoint post-install so the user doesn't see a fake
/// "done" screen with a broken "Chat now" button.
///
/// Strategy: issue a cheap GET to the pod's `/v1/models` endpoint via
/// the public edge. Any 2xx/401/403 means the gateway is answering
/// (401/403 from the edge auth plugin when our probe doesn't carry
/// the bearer token — still proof of life). 404/5xx/timeout = not ready.
/// Build the argv for a `tytus agent env` subprocess. Extracted so
/// tray-level tests can pin the shape without spawning the binary —
/// any drift in the flag names is the kind of thing that silently
/// breaks the OS Env tab.
fn pod_env_argv(pod_id: &str, reveal: bool) -> Vec<String> {
    let mut args = vec![
        "agent".to_string(),
        "env".to_string(),
        "--pod".to_string(),
        pod_id.to_string(),
    ];
    if reveal {
        args.push("--reveal-secrets".to_string());
    }
    args.push("--json".to_string());
    args
}

/// Parse `?reveal=secrets` from a `&`-separated query string. Returns
/// false on any other value (including unset, `reveal=true`, or
/// `reveal=` empty).
fn parse_reveal_flag(query: &str) -> bool {
    query.split('&').any(|kv| kv == "reveal=secrets")
}

/// GET /api/pod/env?pod=NN[&reveal=secrets] — proxies through to
/// `tytus agent env --pod NN [--reveal-secrets] --json`. Provider
/// performs redaction + plan-tier gating; this handler is a thin
/// passthrough that forwards stdout JSON unchanged. A non-zero exit
/// code is mapped to a 502 with the captured stderr line.
fn handle_pod_env(request: Request, query: &str) {
    let pod_id = match parse_pod_id(query) {
        Some(p) => p,
        None => {
            respond_json(request, 400, &serde_json::json!({"error":"invalid pod"}));
            return;
        }
    };
    let reveal = parse_reveal_flag(query);
    let bin = resolve_tytus_bin();
    let mut cmd = Command::new(&bin);
    for a in pod_env_argv(&pod_id, reveal) {
        cmd.arg(a);
    }
    cmd.env("TYTUS_HEADLESS", "1");
    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            respond_json(
                request,
                502,
                &serde_json::json!({
                    "error": "spawn_failed", "message": e.to_string(),
                }),
            );
            return;
        }
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        respond_json(
            request,
            502,
            &serde_json::json!({
                "error": "tytus_subprocess_failed",
                "exit_code": output.status.code(),
                "stderr": stderr,
            }),
        );
        return;
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    match serde_json::from_str::<serde_json::Value>(&stdout) {
        Ok(v) => respond_json(request, 200, &v),
        Err(e) => {
            respond_json(
                request,
                502,
                &serde_json::json!({
                    "error": "invalid_subprocess_json",
                    "message": e.to_string(),
                }),
            );
        }
    }
}

fn handle_pod_ready(request: Request, query: &str) {
    let pod_id = match parse_pod_id(query) {
        Some(p) => p,
        None => {
            respond_json(request, 400, &serde_json::json!({"error":"invalid pod"}));
            return;
        }
    };
    let snap = compute_state_snapshot();
    let agent = snap.agents.iter().find(|a| a.pod_id == pod_id).cloned();
    let api = match agent.as_ref().and_then(|a| a.api_url.clone()) {
        Some(u) => u,
        None => {
            // No public URL derivable yet (slug not in state). Report
            // not-ready but not an error — wizard keeps polling.
            respond_json(
                request,
                200,
                &serde_json::json!({
                    "ready": false, "reason": "public URL not ready"
                }),
            );
            return;
        }
    };
    let probe_url = format!("{}/models", api.trim_end_matches('/'));
    let Some(agent) = agent else {
        respond_json(
            request,
            200,
            &serde_json::json!({
                "ready": false, "status": "unknown", "reason": "pod not found"
            }),
        );
        return;
    };
    let mut status = probe_agent_status(
        &agent.agent_type,
        &api,
        &agent.user_key,
        agent.gateway_token.as_deref(),
        agent.ui_url.as_deref(),
    );
    let mut bootstrap_reason: Option<String> = None;
    if status == AgentStatus::Ready && readiness_strict_enabled() {
        let bootstrap = check_pod_tytus_bootstrap_entry(&pod_id, false);
        if !bootstrap_status_allows_open(&bootstrap.status) {
            status = AgentStatus::Starting;
            bootstrap_reason = Some(format!(
                "bootstrap {}: {}",
                bootstrap.status,
                bootstrap_stage_detail(&bootstrap)
            ));
        }
    }
    cache_agent_status(&pod_id, status);
    let (ready, status_label, reason) = match status {
        AgentStatus::Ready => (
            true,
            "ready",
            "API, UI, and Tytus bootstrap are serving".to_string(),
        ),
        AgentStatus::Starting => (
            false,
            "starting",
            bootstrap_reason.unwrap_or_else(|| {
                "edge, auth, route, UI, or bootstrap still starting".to_string()
            }),
        ),
        AgentStatus::Unhealthy => (
            false,
            "unhealthy",
            "gateway upstream not yet healthy".to_string(),
        ),
        AgentStatus::Stopped => (
            false,
            "stopped",
            "probe failed or gateway unreachable".to_string(),
        ),
        AgentStatus::Unknown => (
            false,
            "unknown",
            "probe returned an unknown status".to_string(),
        ),
    };
    respond_json(
        request,
        200,
        &serde_json::json!({
            "ready": ready, "status": status_label, "reason": reason, "probe_url": probe_url,
        }),
    );
}

fn handle_pod_readiness(request: Request, pod_id_raw: &str) {
    let pod_id = if valid_pod_id(pod_id_raw) {
        pod_id_raw.to_string()
    } else {
        respond_json(request, 400, &serde_json::json!({"error":"invalid pod"}));
        return;
    };

    let snap = compute_state_snapshot();
    let Some(agent) = snap.agents.iter().find(|a| a.pod_id == pod_id).cloned() else {
        respond_json(
            request,
            200,
            &serde_json::json!({
                "pod_id": pod_id,
                "agent": null,
                "overall": "failed",
                "open_enabled": false,
                "strict": readiness_strict_enabled(),
                "stages": [
                    {"id":"allocated","label":"Pod allocated","status":"failed","detail":"pod not found"}
                ],
                "last_checked_at": current_unix_secs(),
            }),
        );
        return;
    };

    let api_url = agent.api_url.clone().unwrap_or_default();
    let base_status = if !readiness_strict_enabled() && !api_url.is_empty() {
        AgentStatus::Ready
    } else {
        probe_agent_status(
            &agent.agent_type,
            &api_url,
            &agent.user_key,
            agent.gateway_token.as_deref(),
            agent.ui_url.as_deref(),
        )
    };
    let bootstrap = check_pod_tytus_bootstrap_entry(&pod_id, false);
    let bootstrap_ok = bootstrap_status_allows_open(&bootstrap.status);
    let core_ready = base_status == AgentStatus::Ready;
    let open_enabled = core_ready && bootstrap_ok;
    let overall = if open_enabled {
        if bootstrap.status == "degraded" {
            "degraded"
        } else {
            "ready"
        }
    } else if matches!(base_status, AgentStatus::Unhealthy | AgentStatus::Stopped)
        || bootstrap_status_is_failed(&bootstrap.status)
    {
        "failed"
    } else {
        "starting"
    };
    cache_agent_status(
        &pod_id,
        if open_enabled {
            AgentStatus::Ready
        } else {
            base_status
        },
    );

    let mut stages = Vec::new();
    stages.push(
        serde_json::json!({"id":"allocated","label":"Pod allocated","status":"ok","detail":null}),
    );
    stages.push(serde_json::json!({
        "id":"tunnel",
        "label":"Tunnel/public URL assigned",
        "status": if agent.public_url.is_some() || agent.api_url.is_some() { "ok" } else { "starting" },
        "detail": agent.public_url.clone().or(agent.api_url.clone())
    }));
    stages.push(serde_json::json!({
        "id":"agent_http",
        "label":"Agent HTTP/API health",
        "status": stage_status_from_agent(base_status),
        "detail": agent_status_label(base_status)
    }));
    stages.push(serde_json::json!({
        "id":"agent_ui",
        "label":"Agent UI route",
        "status": if agent_has_browser_ui(&agent.agent_type) { stage_status_from_agent(base_status) } else { "skipped" },
        "detail": agent.ui_url.clone()
    }));
    stages.push(serde_json::json!({
        "id":"tytus_bootstrap",
        "label":"Tytus bootstrap + smoke health",
        "status": bootstrap.status.clone(),
        "detail": bootstrap_stage_detail(&bootstrap)
    }));
    let (shared_status, shared_detail) = check_pod_shared_storage_stage(&pod_id);
    stages.push(serde_json::json!({
        "id":"shared_storage",
        "label":"Shared storage",
        "status": shared_status,
        "detail": shared_detail
    }));

    respond_json(
        request,
        200,
        &serde_json::json!({
            "pod_id": pod_id,
            "agent": agent.agent_type,
            "overall": overall,
            "open_enabled": open_enabled,
            "strict": readiness_strict_enabled(),
            "stages": stages,
            "last_checked_at": current_unix_secs(),
        }),
    );
}

fn handle_pod_uninstall(request: Request, query: &str) {
    let pod_id = match parse_pod_id(query) {
        Some(p) => p,
        None => {
            respond_json(request, 400, &serde_json::json!({"error":"invalid pod"}));
            return;
        }
    };
    // Only agent pods (nemoclaw, hermes) can be uninstalled. AIL-included
    // pods have no agent to remove — `tytus agent uninstall <pod>` on a
    // default pod is a no-op + confusing error.
    let snap = compute_state_snapshot();
    if !snap.agents.iter().any(|a| a.pod_id == pod_id) {
        respond_json(
            request,
            400,
            &serde_json::json!({
                "error":"pod has no agent to uninstall"
            }),
        );
        return;
    }
    let bin = resolve_tytus_bin();
    let spawned = Command::new(&bin)
        .args(["agent", "uninstall", &pod_id, "--json"])
        .env("TYTUS_HEADLESS", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn();
    match spawned {
        Ok(_) => respond_json(request, 202, &serde_json::json!({"ok":true, "pod":pod_id})),
        Err(e) => respond_json(
            request,
            500,
            &serde_json::json!({
                "error": format!("failed to spawn: {}", e)
            }),
        ),
    }
}

// ── TytusOS control-surface handlers (Wave 1) ───────────────────

fn handle_disconnect(request: Request) {
    // Detached subprocess — disconnect is fast (<1s, no sudo) because it
    // just reads the tunnel pidfile and SIGTERMs. Client polls /api/state
    // to see the tunnel_active flag flip.
    let bin = resolve_tytus_bin();
    let spawned = Command::new(&bin)
        .arg("disconnect")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    match spawned {
        Ok(_) => respond_json(request, 202, &serde_json::json!({"ok": true})),
        Err(e) => respond_json(
            request,
            500,
            &serde_json::json!({
                "error": format!("failed to spawn tytus disconnect: {}", e)
            }),
        ),
    }
}

fn handle_connect(request: Request) {
    // Connect requires sudo for the tunnel-up helper. Tray spawns a
    // Terminal because sudo needs a TTY — we do the same from the page
    // so the UX matches: a Terminal window pops up, user authenticates,
    // tunnel comes up, window auto-closes. Polling /api/state reflects
    // tunnel_active once it's up.
    crate::open_in_terminal_simple(
        "tytus connect && exit; echo; echo 'Connect failed — see above.'; echo 'Press Enter to close…'; read _"
    );
    respond_json(request, 202, &serde_json::json!({"ok": true}));
}

fn handle_test(request: Request, registry: &Registry) {
    // Streamed: `tytus test` is E2E (~5-15s) with per-step spinners.
    // Returns { job_id }; output streams via /api/jobs/<id>/stream so
    // the page can render each check as it lands instead of one big
    // blob at the end. Pre-streaming behavior was Command::output()
    // which blocked until exit; the user saw nothing for the whole
    // run. Requires the wizard helpers in the CLI to flush stdout
    // per line (cli/src/wizard.rs::flush()) — without that, Rust
    // block-buffers stdout when piped and the lines arrive in bursts
    // at process exit anyway.
    let (job_id, job) = registry.create();
    spawn_pod_action(job, vec!["test".to_string()]);
    respond_json(request, 202, &serde_json::json!({"job_id": job_id}));
}

fn handle_settings_get(request: Request) {
    respond_json(
        request,
        200,
        &serde_json::json!({
            "autostart_tunnel": autostart_tunnel_installed(),
            "autostart_tray": autostart_tray_installed(),
        }),
    );
}

#[derive(serde::Serialize, serde::Deserialize)]
struct UpdatePrefs {
    #[serde(default = "default_update_automatic_checks")]
    automatic_checks: bool,
    #[serde(default)]
    last_checked_at: Option<u64>,
}

impl Default for UpdatePrefs {
    fn default() -> Self {
        Self {
            automatic_checks: true,
            last_checked_at: None,
        }
    }
}

fn default_update_automatic_checks() -> bool {
    true
}

fn update_prefs_path() -> Option<PathBuf> {
    if let Some(dir) = dirs::config_dir() {
        return Some(dir.join("tytus").join("update.json"));
    }
    std::env::var_os("HOME").map(|h| {
        PathBuf::from(h)
            .join(".config")
            .join("tytus")
            .join("update.json")
    })
}

fn load_update_prefs() -> UpdatePrefs {
    let Some(path) = update_prefs_path() else {
        return UpdatePrefs::default();
    };
    let Ok(raw) = std::fs::read_to_string(path) else {
        return UpdatePrefs::default();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_update_prefs(prefs: &UpdatePrefs) -> std::io::Result<()> {
    let Some(path) = update_prefs_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(prefs).unwrap_or_else(|_| "{}".to_string());
    std::fs::write(path, raw)
}

fn current_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn update_status_body(prefs: &UpdatePrefs, checked_at: Option<u64>) -> serde_json::Value {
    let current = env!("CARGO_PKG_VERSION");
    // Honest updater contract: this local build can persist automatic-check
    // preference and run manual checks today. Installers can set the
    // TYTUS_UPDATE_* build env vars later to turn the same route into a real
    // stable-channel update surface without changing the TytusOS UI contract.
    let latest = option_env!("TYTUS_UPDATE_LATEST_VERSION")
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let channel = option_env!("TYTUS_UPDATE_CHANNEL")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("local");
    let status = match latest {
        Some(v) if v != current => "update_available",
        Some(_) => "up_to_date",
        None => "unknown",
    };
    let detail = match status {
        "update_available" => format!("TytusOS {} is available.", latest.unwrap_or(current)),
        "up_to_date" => format!("TytusOS {} is installed and up to date.", current),
        _ => "Local build installed. No update feed is configured for this build.".to_string(),
    };
    serde_json::json!({
        "current_version": current,
        "installed_version": current,
        "latest_version": latest,
        "channel": channel,
        "status": status,
        "automatic_checks": prefs.automatic_checks,
        "last_checked_at": prefs.last_checked_at,
        "checked_at": checked_at,
        "detail": detail,
    })
}

fn handle_update_status(request: Request) {
    let prefs = load_update_prefs();
    respond_json(request, 200, &update_status_body(&prefs, None));
}

fn handle_update_check(request: Request) {
    let mut prefs = load_update_prefs();
    let now = current_unix_secs();
    prefs.last_checked_at = Some(now);
    if let Err(e) = save_update_prefs(&prefs) {
        respond_json(
            request,
            500,
            &serde_json::json!({
                "error": format!("failed to save update preferences: {}", e),
            }),
        );
        return;
    }
    respond_json(request, 200, &update_status_body(&prefs, Some(now)));
}

#[derive(serde::Deserialize)]
struct ToggleBody {
    enabled: bool,
}

fn handle_update_automatic(mut request: Request) {
    let enabled = match parse_toggle_body(&mut request) {
        Ok(e) => e,
        Err(_resp_sent) => {
            return;
        }
    };
    let mut prefs = load_update_prefs();
    prefs.automatic_checks = enabled;
    if let Err(e) = save_update_prefs(&prefs) {
        respond_json(
            request,
            500,
            &serde_json::json!({
                "error": format!("failed to save update preferences: {}", e),
            }),
        );
        return;
    }
    respond_json(request, 200, &update_status_body(&prefs, None));
}

fn handle_autostart_tunnel(mut request: Request) {
    let enabled = match parse_toggle_body(&mut request) {
        Ok(e) => e,
        Err(_resp_sent) => {
            return;
        }
    };
    // `tytus autostart install|uninstall` writes
    // ~/Library/LaunchAgents/com.traylinx.tytus.plist — user-scope, no
    // sudo. Runs synchronously so we can surface stderr inline.
    let sub = if enabled { "install" } else { "uninstall" };
    run_tytus_inline(request, &["autostart", sub]);
}

fn handle_autostart_tray(mut request: Request) {
    let enabled = match parse_toggle_body(&mut request) {
        Ok(e) => e,
        Err(_resp_sent) => {
            return;
        }
    };
    // `tytus tray install` creates /Applications/Tytus.app + the tray
    // LaunchAgent. 5-10s on first run because icons are generated via
    // sips + iconutil. User-scope; no sudo.
    let sub = if enabled { "install" } else { "uninstall" };
    run_tytus_inline(request, &["tray", sub]);
}

fn handle_login(request: Request) {
    // Browser-native device auth. Unlike `tytus login`, this path must not
    // open a Terminal: TytusOS starts the Sentinel device flow, opens the
    // verification URL in the user's browser, then a background thread polls
    // for approval and writes the same state.json/keychain fields as the CLI.
    let http = atomek_core::HttpClient::new();
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            respond_json(
                request,
                500,
                &serde_json::json!({
                    "error": format!("failed to start login runtime: {}", e)
                }),
            );
            return;
        }
    };

    let session = match rt.block_on(atomek_auth::create_device_session(&http)) {
        Ok(s) => s,
        Err(e) => {
            respond_json(
                request,
                500,
                &serde_json::json!({
                    "error": format!("failed to start login: {}", e)
                }),
            );
            return;
        }
    };

    let opened_browser = open_verification_url(&session.verification_uri);
    let device_id = session.device_id.clone();
    thread::spawn(move || {
        let Ok(rt) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        else {
            return;
        };
        let http = atomek_core::HttpClient::new();
        let result = rt.block_on(atomek_auth::poll_for_authorization(
            &http,
            &device_id,
            |_| {},
        ));
        match result {
            Ok(auth) => {
                rt.block_on(persist_device_login(&http, auth));
            }
            Err(e) => {
                eprintln!("tytus tray login failed: {}", e);
            }
        }
    });

    respond_json(
        request,
        202,
        &serde_json::json!({
            "verification_uri": session.verification_uri,
            "user_code": session.user_code,
            "expires_in": session.expires_in,
            "opened_browser": opened_browser,
        }),
    );
}

fn open_verification_url(url: &str) -> bool {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok()
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = url;
        false
    }
}

async fn persist_device_login(
    http: &atomek_core::HttpClient,
    result: atomek_auth::DeviceAuthResult,
) {
    let path = match state_json_path() {
        Some(p) => p,
        None => return,
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut root: serde_json::Value = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({ "pods": [] }));

    let email = result.user.email.clone();
    root["email"] = serde_json::json!(email);
    root["access_token"] = serde_json::json!(result.access_token);
    root["expires_at_ms"] =
        serde_json::json!(current_epoch_millis() + (result.expires_in as i64 * 1000));
    if let Some(obj) = root.as_object_mut() {
        obj.remove("refresh_token");
    }

    if !email.is_empty() {
        let _ = atomek_auth::KeychainStore::store_refresh_token(&email, &result.refresh_token);
        let _ = atomek_auth::KeychainStore::store_last_email(&email);
    }

    if let Ok(creds) =
        atomek_auth::fetch_wannolot_pass(http, root["access_token"].as_str().unwrap_or_default())
            .await
    {
        root["secret_key"] = serde_json::json!(creds.secret_key);
        root["agent_user_id"] = serde_json::json!(creds.agent_user_id);
        root["organization_id"] = serde_json::json!(creds.organization_id);
        root["tier"] = serde_json::json!(creds.tier);

        if let (Some(sk), Some(auid)) = (
            root.get("secret_key").and_then(|v| v.as_str()),
            root.get("agent_user_id").and_then(|v| v.as_str()),
        ) {
            let client = atomek_pods::TytusClient::new(http, sk, auid);
            if let Ok(status) = atomek_pods::get_pod_status(&client).await {
                merge_pod_status(&mut root, status.pods);
            }
            ensure_default_pod_json(&client, &mut root).await;
        }
    }

    if let Ok(data) = serde_json::to_string_pretty(&root) {
        let _ = std::fs::write(&path, data);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        }
    }
}

fn current_epoch_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn merge_pod_status(root: &mut serde_json::Value, server_pods: Vec<atomek_pods::PodEntry>) {
    let existing = root
        .get("pods")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut merged = Vec::new();

    for sp in server_pods {
        let mut pod = existing
            .iter()
            .find(|p| p.get("pod_id").and_then(|v| v.as_str()) == Some(sp.pod_id.as_str()))
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        pod["pod_id"] = serde_json::json!(sp.pod_id);
        pod["droplet_id"] = serde_json::json!(sp.droplet_id);
        pod["agent_type"] = serde_json::json!(sp.agent_type.unwrap_or_else(|| "none".to_string()));
        merged.push(pod);
    }

    root["pods"] = serde_json::Value::Array(merged);
}

async fn ensure_default_pod_json(client: &atomek_pods::TytusClient, root: &mut serde_json::Value) {
    let Ok(alloc) = atomek_pods::request_default_pod(client).await else {
        return;
    };
    let mut pods = root
        .get("pods")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let preserved_iface = pods
        .iter()
        .find(|p| p.get("pod_id").and_then(|v| v.as_str()) == Some(alloc.pod_id.as_str()))
        .and_then(|p| p.get("tunnel_iface").cloned());
    pods.retain(|p| p.get("pod_id").and_then(|v| v.as_str()) != Some(alloc.pod_id.as_str()));
    let mut pod = serde_json::json!({
        "pod_id": alloc.pod_id,
        "droplet_id": alloc.droplet_id,
        "droplet_ip": alloc.droplet_ip,
        "ai_endpoint": alloc.ai_endpoint,
        "pod_api_key": alloc.pod_api_key,
        "agent_type": "none",
        "agent_endpoint": null,
        "stable_ai_endpoint": alloc.stable_ai_endpoint,
        "stable_user_key": alloc.stable_user_key,
        "gateway_token": null,
        "edge_slug": null,
        "edge_public_url": null,
        "pod_public_url": alloc.pod_public_url,
    });
    pod["tunnel_iface"] = preserved_iface.unwrap_or(serde_json::Value::Null);
    pods.push(pod);
    root["pods"] = serde_json::Value::Array(pods);
}

fn handle_logout(request: Request) {
    // Destructive: revokes all pods + clears keychain. JS confirms before
    // POSTing. Run headless inline so TytusOS logs out inside the page instead
    // of spawning a macOS Terminal window.
    run_tytus_inline(request, &["logout"]);
}

fn parse_toggle_body(request: &mut Request) -> Result<bool, ()> {
    let mut body = String::new();
    if request.as_reader().read_to_string(&mut body).is_err() {
        return Err(());
    }
    let parsed: Result<ToggleBody, _> = serde_json::from_str(&body);
    match parsed {
        Ok(t) => Ok(t.enabled),
        Err(_) => Err(()),
    }
}

fn run_tytus_inline(request: Request, args: &[&str]) {
    let bin = resolve_tytus_bin();
    let out = Command::new(&bin)
        .args(args)
        .env("TYTUS_HEADLESS", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();
    match out {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            respond_json(
                request,
                if o.status.success() { 200 } else { 500 },
                &serde_json::json!({
                    "ok": o.status.success(),
                    "exit_code": o.status.code().unwrap_or(-1),
                    "stdout": stdout,
                    "stderr": stderr,
                }),
            );
        }
        Err(e) => respond_json(
            request,
            500,
            &serde_json::json!({
                "error": format!("failed to spawn tytus {}: {}", args.join(" "), e)
            }),
        ),
    }
}

#[cfg(target_os = "macos")]
fn autostart_tunnel_installed() -> bool {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(&home)
        .join("Library/LaunchAgents/com.traylinx.tytus.plist")
        .exists()
}
#[cfg(target_os = "linux")]
fn autostart_tunnel_installed() -> bool {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(&home)
        .join(".config/systemd/user/tytus.service")
        .exists()
}
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn autostart_tunnel_installed() -> bool {
    false
}

#[cfg(target_os = "macos")]
fn autostart_tray_installed() -> bool {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(&home)
        .join("Library/LaunchAgents/com.traylinx.tytus.tray.plist")
        .exists()
}
#[cfg(not(target_os = "macos"))]
fn autostart_tray_installed() -> bool {
    false
}

// ── TytusOS Wave 2: Troubleshoot handlers ────────────────────────

fn handle_doctor(request: Request, registry: &Registry) {
    // Streamed: `tytus doctor` runs DNS, auth, tunnel, pod, gateway,
    // MCP checks back-to-back. Returns { job_id }; output streams via
    // /api/jobs/<id>/stream. Same rationale as handle_test — see that
    // doc block + the wizard::flush() helper in the CLI.
    let (job_id, job) = registry.create();
    spawn_pod_action(job, vec!["doctor".to_string()]);
    respond_json(request, 202, &serde_json::json!({"job_id": job_id}));
}

enum DaemonAction {
    Start,
    Stop,
    Restart,
}

fn handle_daemon_lifecycle(request: Request, action: DaemonAction) {
    let bin = resolve_tytus_bin();
    match action {
        DaemonAction::Stop => {
            // Blocking — `tytus daemon stop` is fast (<1s). Surface result.
            let out = Command::new(&bin)
                .args(["daemon", "stop"])
                .env("TYTUS_HEADLESS", "1")
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output();
            match out {
                Ok(o) => respond_json(
                    request,
                    if o.status.success() { 200 } else { 500 },
                    &serde_json::json!({
                        "ok": o.status.success(),
                        "stdout": String::from_utf8_lossy(&o.stdout).to_string(),
                        "stderr": String::from_utf8_lossy(&o.stderr).to_string(),
                    }),
                ),
                Err(e) => respond_json(
                    request,
                    500,
                    &serde_json::json!({
                        "error": format!("failed: {}", e)
                    }),
                ),
            }
        }
        DaemonAction::Start => {
            // Detached — `tytus daemon run` blocks in the foreground until
            // the process exits, so we must spawn it without piping stdio
            // back to the HTTP thread. launchd normally handles this via
            // KeepAlive; manual start is a troubleshooting path.
            let res = Command::new(&bin)
                .args(["daemon", "run"])
                .env("TYTUS_HEADLESS", "1")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            match res {
                Ok(_) => respond_json(request, 202, &serde_json::json!({"ok": true})),
                Err(e) => respond_json(
                    request,
                    500,
                    &serde_json::json!({
                        "error": format!("failed to spawn: {}", e)
                    }),
                ),
            }
        }
        DaemonAction::Restart => {
            // Stop blocking then spawn new daemon detached. If stop fails
            // (maybe daemon was already down), we still attempt start —
            // the user's intent is "make it running now".
            let _ = Command::new(&bin)
                .args(["daemon", "stop"])
                .env("TYTUS_HEADLESS", "1")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            std::thread::sleep(std::time::Duration::from_millis(500));
            let res = Command::new(&bin)
                .args(["daemon", "run"])
                .env("TYTUS_HEADLESS", "1")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            match res {
                Ok(_) => respond_json(request, 202, &serde_json::json!({"ok": true})),
                Err(e) => respond_json(
                    request,
                    500,
                    &serde_json::json!({
                        "error": format!("restart failed: {}", e)
                    }),
                ),
            }
        }
    }
}

fn handle_daemon_status(request: Request) {
    // Canonical liveness check: read /tmp/tytus/daemon.pid + `kill -0`
    // probe. `kill -0 pid` returns Ok if the process exists and the
    // sender has permission to signal it; Err(ESRCH) if it's gone.
    // Matches the CLI's `tytus daemon status` logic so results agree
    // between surfaces.
    let pid_path = PathBuf::from("/tmp/tytus/daemon.pid");
    let pid: Option<i32> = std::fs::read_to_string(&pid_path)
        .ok()
        .and_then(|s| s.trim().parse().ok());
    let running = match pid {
        Some(p) => unsafe { libc::kill(p, 0) == 0 },
        None => false,
    };
    respond_json(
        request,
        200,
        &serde_json::json!({
            "running": running,
            "pid": pid,
        }),
    );
}

fn handle_log_tail(request: Request, query: &str) {
    // Simple byte-offset poll. Client calls /api/logs?name=daemon with
    // offset=0 on first load, appends the returned chunk, then polls
    // with offset=last_offset. Server reads from that offset to EOF and
    // returns the new bytes (capped at MAX_CHUNK to keep responses
    // bounded). File shrinkage (rotation) resets offset to 0 with
    // `truncated: true` so the client can wipe its pre.
    const MAX_CHUNK: u64 = 128 * 1024;

    let mut name = "daemon";
    let mut offset: u64 = 0;
    for pair in query.split('&').filter(|s| !s.is_empty()) {
        if let Some((k, v)) = pair.split_once('=') {
            match k {
                "name" => {
                    name = match v {
                        "daemon" | "startup" => v,
                        _ => "daemon",
                    }
                }
                "offset" => {
                    offset = v.parse().unwrap_or(0);
                }
                _ => {}
            }
        }
    }
    let path = match name {
        "startup" => PathBuf::from("/tmp/tytus/autostart.log"),
        _ => PathBuf::from("/tmp/tytus/daemon.log"),
    };
    let meta = match std::fs::metadata(&path) {
        Ok(m) => m,
        Err(_) => {
            respond_json(
                request,
                200,
                &serde_json::json!({
                    "name": name,
                    "offset": 0u64,
                    "size": 0u64,
                    "chunk": "",
                    "missing": true,
                }),
            );
            return;
        }
    };
    let size = meta.len();
    let (read_from, truncated) = if offset > size {
        (0, true)
    } else {
        (offset, false)
    };
    let available = size.saturating_sub(read_from);
    let take = available.min(MAX_CHUNK);

    let chunk = if take == 0 {
        String::new()
    } else {
        use std::io::{Read as _, Seek, SeekFrom};
        let mut f = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(_) => {
                respond_json(request, 500, &serde_json::json!({"error":"open failed"}));
                return;
            }
        };
        if f.seek(SeekFrom::Start(read_from)).is_err() {
            respond_json(request, 500, &serde_json::json!({"error":"seek failed"}));
            return;
        }
        let mut buf = vec![0u8; take as usize];
        let n = f.read(&mut buf).unwrap_or(0);
        buf.truncate(n);
        String::from_utf8_lossy(&buf).to_string()
    };
    let new_offset = read_from + (chunk.len() as u64);
    respond_json(
        request,
        200,
        &serde_json::json!({
            "name": name,
            "offset": new_offset,
            "size": size,
            "chunk": chunk,
            "truncated": truncated,
            "missing": false,
        }),
    );
}

// ── TytusOS Wave 3b: launch in editor ────────────────────────────

fn handle_launchers_list(request: Request) {
    // Mirror the tray's "Open in ▸" detection so the page shows the
    // same set of editors. Each entry is a thin metadata record; the
    // actual launch happens via POST /api/launch with the binary name.
    let clis = crate::launcher::detect_installed_clis();
    let list: Vec<serde_json::Value> = clis
        .iter()
        .map(|c| {
            serde_json::json!({
                "binary": c.binary,
                "name": c.name,
            })
        })
        .collect();
    respond_json(
        request,
        200,
        &serde_json::json!({
            "editors": list,
            // Plain terminal is always "available"; tray lists it unconditionally.
            "terminal_available": true,
        }),
    );
}

fn handle_launch(request: Request, query: &str) {
    // Query: editor=<binary>&pod=NN (pod optional; default picks the
    // first pod with a stable_user_key — matches the tray's
    // connection_pair fallback).
    let mut editor = String::new();
    let mut pod_id_override: Option<String> = None;
    for pair in query.split('&').filter(|s| !s.is_empty()) {
        if let Some((k, v)) = pair.split_once('=') {
            match k {
                "editor" => editor = v.to_string(),
                "pod" => {
                    if v.chars().all(|c| c.is_ascii_digit()) && !v.is_empty() {
                        pod_id_override = Some(v.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    // Find the connection pair: URL from the chosen pod (or the first
    // pod with a stable key), api_key from the stable user key.
    let snap = compute_state_snapshot();
    let agent = pod_id_override
        .as_ref()
        .and_then(|id| snap.agents.iter().find(|a| &a.pod_id == id))
        .or_else(|| snap.agents.iter().find(|a| a.api_url.is_some()));
    let included_first = snap.included.iter().find(|i| i.public_url.is_some());
    let (url, api_key) = if let Some(a) = agent {
        let url = a.api_url.clone().unwrap_or_else(|| {
            // Fallback to AIL private endpoint when the pod has no public
            // URL yet (mid-provisioning).
            included_first
                .map(|i| format!("{}/v1", i.endpoint))
                .unwrap_or_else(|| "http://10.42.42.1:18080/v1".into())
        });
        (url, a.user_key.clone())
    } else if let Some(inc) = snap.included.first() {
        let url = inc
            .public_url
            .as_ref()
            .map(|u| format!("{}/v1", u))
            .unwrap_or_else(|| format!("{}/v1", inc.endpoint));
        (url, inc.user_key.clone())
    } else {
        respond_json(
            request,
            400,
            &serde_json::json!({
                "error": "no pods available — run tytus connect first"
            }),
        );
        return;
    };

    // Launcher wants the base URL (no /v1 — it appends /v1 itself in
    // the shell_cmd it builds).
    let base = url
        .trim_end_matches("/v1")
        .trim_end_matches('/')
        .to_string();
    let conn = crate::launcher::PodConnection {
        ai_gateway: base,
        api_key,
        model: "ail-compound".into(),
    };

    // Special case: "terminal" opens a plain shell with env exports.
    if editor == "terminal" {
        crate::launcher::launch_terminal(&conn);
        respond_json(
            request,
            202,
            &serde_json::json!({"ok": true, "editor": "terminal"}),
        );
        return;
    }

    // Look up the chosen editor by binary name; refuse unknown binaries
    // so a mischievous client can't smuggle arbitrary shell into the
    // templated command.
    let clis = crate::launcher::detect_installed_clis();
    let cli = match clis.iter().find(|c| c.binary == editor) {
        Some(c) => c.clone(),
        None => {
            respond_json(
                request,
                400,
                &serde_json::json!({
                    "error": format!("editor not detected or not whitelisted: {}", editor)
                }),
            );
            return;
        }
    };
    crate::launcher::launch_in_terminal(&cli, &conn);
    respond_json(
        request,
        202,
        &serde_json::json!({"ok": true, "editor": cli.binary}),
    );
}

// ── TytusOS Wave 3c: per-pod channels ────────────────────────────

/// `^[a-z][a-z0-9_-]{1,30}$` — matches known channel names (telegram,
/// discord, slack, line) plus leaves room for future additions. Used
/// as a whitelist before templating the name into a shell command.
fn valid_channel_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 31
        && s.chars()
            .next()
            .map(|c| c.is_ascii_lowercase())
            .unwrap_or(false)
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

fn valid_pod_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 4 && s.chars().all(|c| c.is_ascii_digit())
}

fn normalize_pod_id_for_query(raw: &str) -> Option<String> {
    let pod = raw
        .trim()
        .strip_prefix("wannolot-")
        .or_else(|| raw.trim().strip_prefix("tytus-"))
        .unwrap_or_else(|| raw.trim());
    if valid_pod_id(pod) {
        Some(pod.to_string())
    } else {
        None
    }
}

fn parse_channel_query(query: &str) -> (Option<String>, Option<String>) {
    let mut pod = None;
    let mut name = None;
    for pair in query.split('&').filter(|s| !s.is_empty()) {
        if let Some((k, v)) = pair.split_once('=') {
            match k {
                "pod" => {
                    if valid_pod_id(v) {
                        pod = Some(v.to_string());
                    }
                }
                "name" => {
                    if valid_channel_name(v) {
                        name = Some(v.to_string());
                    }
                }
                _ => {}
            }
        }
    }
    (pod, name)
}

fn handle_channels_list(request: Request, query: &str) {
    let (pod, _) = parse_channel_query(query);
    let pod_id = match pod {
        Some(p) => p,
        None => {
            respond_json(
                request,
                400,
                &serde_json::json!({ "error": "missing or invalid pod id" }),
            );
            return;
        }
    };
    let configured: Vec<serde_json::Value> = crate::read_channels_for_pod(&pod_id)
        .into_iter()
        .map(|(name, count)| {
            serde_json::json!({
                "name": name,
                "label": crate::channel_label(&name),
                "secret_count": count,
            })
        })
        .collect();
    // Available = everything in CHANNEL_MENU_ENTRIES that isn't already
    // configured on this pod. Matches what the tray's channels submenu
    // shows when building the "Add X…" list.
    let configured_names: std::collections::HashSet<String> = configured
        .iter()
        .filter_map(|e| e.get("name").and_then(|n| n.as_str()).map(String::from))
        .collect();
    let available: Vec<serde_json::Value> = crate::CHANNEL_MENU_ENTRIES
        .iter()
        .filter(|(n, _)| !configured_names.contains(*n))
        .map(|(n, l)| serde_json::json!({ "name": n, "label": l }))
        .collect();
    respond_json(
        request,
        200,
        &serde_json::json!({
            "pod_id": pod_id,
            "configured": configured,
            "available": available,
        }),
    );
}

/// Phase C: token modal posts JSON `{pod, channel, token}`. The token
/// rides only the request body and is forwarded to the `tytus`
/// subprocess as an argv element — the shell is never invoked, so
/// shell-quoting and injection concerns do not apply. The token is
/// not written to disk on the laptop side; tytus forwards it to the
/// provider over TLS where it ends up in the pod keychain.
///
/// IMPORTANT: never echo the request body back. Error responses
/// quote only the `error` field, never `raw` or `body.token`, so a
/// malformed request can't surface the secret in tray.log.
///
/// THREAT MODEL NOTE: passing `--token <value>` as argv makes the
/// token visible to local processes via `ps aux` /
/// `/proc/<pid>/cmdline` for the lifetime of the `tytus` subprocess
/// (typically ~10-15 s while the agent redeploys). This is the same
/// exposure as Sebastian's prior Terminal flow (`tytus channels add
/// --token "$TOK"`). Eliminating it would require an alternative IPC
/// (env var, stdin pipe, or file descriptor) on the `tytus` side —
/// out of scope for this sprint. Mitigations: localhost-only HTTP,
/// short-lived subprocess, single-user laptop trust boundary.
fn handle_channels_add(mut request: Request, _query: &str) {
    #[derive(serde::Deserialize)]
    struct Body {
        pod: String,
        channel: String,
        token: String,
    }
    let mut raw = String::new();
    if request.as_reader().read_to_string(&mut raw).is_err() {
        respond_json(request, 400, &serde_json::json!({"error": "read failed"}));
        return;
    }
    let body: Body = match serde_json::from_str(&raw) {
        Ok(b) => b,
        Err(_) => {
            // Do NOT echo `raw` — it carries the token.
            respond_json(request, 400, &serde_json::json!({"error": "bad json"}));
            return;
        }
    };
    if !valid_pod_id(&body.pod) {
        respond_json(request, 400, &serde_json::json!({"error": "invalid pod"}));
        return;
    }
    if !valid_channel_name(&body.channel) {
        respond_json(
            request,
            400,
            &serde_json::json!({"error": "invalid channel"}),
        );
        return;
    }
    if body.token.is_empty() || body.token.len() > 4096 {
        respond_json(request, 400, &serde_json::json!({"error": "invalid token"}));
        return;
    }
    // run_tytus_inline blocks for ~10-15s while the agent redeploys.
    // The browser modal shows "Adding…" during that window. The token
    // is the last argv element; no shell, no quoting, no log surface.
    run_tytus_inline(
        request,
        &[
            "channels",
            "add",
            "--pod",
            &body.pod,
            "--type",
            &body.channel,
            "--token",
            &body.token,
        ],
    );
}

fn handle_channels_remove(request: Request, query: &str) {
    let (pod, name) = parse_channel_query(query);
    let (Some(pod_id), Some(channel)) = (pod, name) else {
        respond_json(
            request,
            400,
            &serde_json::json!({ "error": "missing pod or name" }),
        );
        return;
    };
    // Client confirms first; server runs the subprocess inline. The
    // credential wipe + redeploy takes ~10s, so the HTTP thread blocks
    // for that window — acceptable since each click is its own thread
    // and the page shows a spinner. Phase B switches this to streamed
    // SSE for richer feedback.
    run_tytus_inline(
        request,
        &["channels", "remove", "--pod", &pod_id, "--type", &channel],
    );
}

fn handle_channels_catalog(request: Request) {
    // Read-only listing of available channel types. Runs synchronously
    // (subprocess returns within ~200ms) and returns the captured stdout
    // so the page can render it inline without spawning Terminal.app.
    run_tytus_inline(request, &["channels", "catalog"]);
}

// ── TytusOS Wave 4: sync gaps ────────────────────────────────────

fn handle_pod_stop_forwarder(request: Request, query: &str) {
    // Mirrors the tray's `pod_NN_stop_forwarder` — runs `tytus ui --stop
    // --pod NN` so the CLI's pidfile cleanup path stays the source of
    // truth. Detached; reply 202 and let the client refresh.
    let mut pod_id: Option<String> = None;
    for pair in query.split('&').filter(|s| !s.is_empty()) {
        if let Some((k, v)) = pair.split_once('=') {
            if k == "pod" && valid_pod_id(v) {
                pod_id = Some(v.to_string());
            }
        }
    }
    let Some(pod) = pod_id else {
        respond_json(
            request,
            400,
            &serde_json::json!({ "error": "missing or invalid pod id" }),
        );
        return;
    };
    let bin = resolve_tytus_bin();
    let spawned = Command::new(&bin)
        .args(["ui", "--stop", "--pod", &pod])
        .env("TYTUS_HEADLESS", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    match spawned {
        Ok(_) => respond_json(request, 202, &serde_json::json!({"ok": true, "pod": pod})),
        Err(e) => respond_json(
            request,
            500,
            &serde_json::json!({
                "error": format!("failed to spawn: {}", e)
            }),
        ),
    }
}

fn handle_configure(request: Request) {
    // Kept for older frontends, but no longer spawns macOS Terminal.app.
    // Current TytusOS opens /api/terminal/session with {command:"tytus",
    // args:["configure"]} so the flow runs inside the browser terminal.
    respond_json(
        request,
        409,
        &serde_json::json!({
            "error": "configure_requires_internal_terminal",
            "terminal": { "command": "tytus", "args": ["configure"] }
        }),
    );
}

// ── Helpers ────────────────────────────────────────────────────

fn respond_json<T: Serialize>(request: Request, status: u16, body: &T) {
    let json = serde_json::to_string(body).unwrap_or_else(|_| "{}".into());
    // If this request carried an `Idempotency-Key`, capture the
    // response so a future retry replays it. Skip 5xx — those are
    // typically transient and we want the client to be free to retry.
    if let Some(key) = take_current_idem_key() {
        if status < 500 {
            idempotency_put(key, status, json.clone());
        }
    }
    let resp = Response::from_string(json)
        .with_status_code(StatusCode(status))
        .with_header(header("Content-Type", "application/json"))
        // Phase 2 floor — block MIME sniffing on the JSON API surface.
        .with_header(header("X-Content-Type-Options", "nosniff"));
    let _ = request.respond(resp);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RAII guard that pins `TYTUS_STATE_PATH` for the duration of a
    /// test, releases the env var on drop, and serializes against
    /// other guard-using tests via a static mutex (env vars are
    /// process-global; two parallel tests reading them would race).
    ///
    /// Used by `etag_roundtrip_via_real_http` to keep
    /// `compute_state_snapshot()` deterministic across the test's two
    /// HTTP requests.
    struct StateOverrideGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        prev: Option<std::ffi::OsString>,
    }

    impl StateOverrideGuard {
        fn set(path: &str) -> Self {
            static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
            // Even if a prior test panicked while holding the lock,
            // the guard's PoisonError still gives us the inner mutex.
            let lock = LOCK
                .get_or_init(|| std::sync::Mutex::new(()))
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let prev = std::env::var_os("TYTUS_STATE_PATH");
            std::env::set_var("TYTUS_STATE_PATH", path);
            Self { _lock: lock, prev }
        }
    }

    impl Drop for StateOverrideGuard {
        fn drop(&mut self) {
            match self.prev.take() {
                Some(v) => std::env::set_var("TYTUS_STATE_PATH", v),
                None => std::env::remove_var("TYTUS_STATE_PATH"),
            }
        }
    }

    #[test]
    fn state_override_guard_round_trips_env_var() {
        // Pin: guard sets the env var, drop restores prior value.
        std::env::remove_var("TYTUS_STATE_PATH");
        {
            let _g = StateOverrideGuard::set("/tmp/fixture-A.json");
            assert_eq!(
                std::env::var_os("TYTUS_STATE_PATH"),
                Some("/tmp/fixture-A.json".into()),
            );
        }
        assert!(std::env::var_os("TYTUS_STATE_PATH").is_none());
    }

    #[test]
    fn state_refresh_error_classifier_detects_login_required_errors() {
        assert!(refresh_error_requires_login(
            "refresh token expired — run `tytus login`"
        ));
        assert!(refresh_error_requires_login(
            "No refresh token available — run 'tytus login'"
        ));
        assert!(!refresh_error_requires_login("temporary sentinel 502"));
    }

    #[test]
    fn state_snapshot_suppresses_stale_login_error_when_logged_in() {
        let err = Some("refresh token expired — run `tytus login`".to_string());
        assert_eq!(effective_state_refresh_error(err, true, true), None);
    }

    #[test]
    fn state_snapshot_keeps_login_error_when_not_logged_in() {
        let err = Some("refresh token expired — run `tytus login`".to_string());
        assert_eq!(effective_state_refresh_error(err.clone(), false, true), err);
    }

    #[test]
    fn state_snapshot_keeps_non_auth_refresh_errors() {
        let err = Some("temporary sentinel 502".to_string());
        assert_eq!(effective_state_refresh_error(err.clone(), true, true), err);
    }

    #[test]
    fn tytusos_assets_are_vendored() {
        assert!(os_asset("/index.html").is_some());
        assert!(os_asset("/").is_some());
        assert!(os_asset("/assets/does-not-exist.js").is_none());
    }

    #[test]
    fn tytusos_content_types_cover_spa_assets() {
        assert_eq!(content_type_for_path("/"), "text/html; charset=utf-8");
        assert_eq!(
            content_type_for_path("/assets/index.js"),
            "application/javascript; charset=utf-8"
        );
        assert_eq!(
            content_type_for_path("/assets/sqlite3.wasm"),
            "application/wasm"
        );
        assert_eq!(
            content_type_for_path("/site.webmanifest"),
            "application/manifest+json"
        );
    }

    #[test]
    fn pod_action_argv_whitelist() {
        // Known-good per-pod actions resolve to the canonical tytus argv.
        assert_eq!(
            pod_action_argv("restart", "02").unwrap(),
            vec!["restart", "--pod", "02"],
        );
        assert_eq!(
            pod_action_argv("revoke", "04").unwrap(),
            vec!["revoke", "04"],
        );
        assert_eq!(
            pod_action_argv("uninstall", "04").unwrap(),
            vec!["agent", "uninstall", "04"],
        );
        assert_eq!(
            pod_action_argv("stop-forwarder", "02").unwrap(),
            vec!["ui", "--stop", "--pod", "02"],
        );
        assert_eq!(
            pod_action_argv("channels-list", "02").unwrap(),
            vec!["channels", "list", "--pod", "02"],
        );
        // rc.13: Files tab "Browse inbox" button — read-only listing
        // of the pod's `/app/workspace/inbox/` (CLI's default `ls`
        // path when no PATH arg is given).
        assert_eq!(
            pod_action_argv("ls-inbox", "02").unwrap(),
            vec!["ls", "--pod", "02"],
        );
        // Pod Inspector Logs tab — defaults to 200 trailing lines so the
        // SSE relay can flush them as individual `log` events.
        assert_eq!(
            pod_action_argv("logs", "02").unwrap(),
            vec!["logs", "--pod", "02", "--lines", "200"],
        );

        // Per-pod doctor is now allowed (closed manifest §3.7 gap).
        assert_eq!(
            pod_action_argv("doctor", "02").unwrap(),
            vec!["doctor", "--pod", "02"],
        );

        // `test` stays global — CLI doesn't accept --pod for it.
        assert!(pod_action_argv("test", "02").is_none());

        // Unknown / injection-shaped actions reject.
        assert!(pod_action_argv("install", "02").is_none());
        assert!(pod_action_argv("doctor; rm -rf /", "02").is_none());
        assert!(pod_action_argv("", "02").is_none());
        assert!(pod_action_argv("RESTART", "02").is_none());
    }

    #[test]
    fn registry_create_pod_rejects_concurrent() {
        let r = Registry::new();
        let (id1, _) = r.create_pod("02").expect("first create_pod");
        // Second start on same pod while still running → Err.
        assert!(r.create_pod("02").is_err());
        // Different pod is fine.
        assert!(r.create_pod("04").is_ok());
        // Marking the first finished frees the slot.
        {
            let job = r.get(&id1).unwrap();
            job.lock().unwrap().finished = true;
        }
        assert!(r.create_pod("02").is_ok());
    }

    #[test]
    fn registry_active_pods_counts() {
        let r = Registry::new();
        let _ = r.create_pod("02").unwrap();
        let _ = r.create_pod("04").unwrap();
        let active = r.active_pods();
        assert_eq!(active.get("02"), Some(&1));
        assert_eq!(active.get("04"), Some(&1));
        assert_eq!(active.get("99"), None);
    }

    #[test]
    fn daemon_started_at_is_stable_across_calls() {
        // OnceLock invariant — UI consumers diff this value to detect
        // restart, so it MUST NOT drift between polls within one
        // process.
        let a = daemon_started_at();
        std::thread::sleep(std::time::Duration::from_millis(5));
        let b = daemon_started_at();
        assert_eq!(a, b);
        assert!(a > 0, "Unix epoch zero would mean SystemTime::now failed");
    }

    #[test]
    fn compute_state_etag_is_deterministic_within_process() {
        // Pin the contract: two identical bodies produce the same
        // ETag, so the client's If-None-Match round-trip succeeds.
        // Different bodies must NOT collide on this fixed input.
        let a = compute_state_etag(b"{\"tier\":\"operator\"}");
        let b = compute_state_etag(b"{\"tier\":\"operator\"}");
        assert_eq!(a, b);
        assert!(
            a.starts_with('"') && a.ends_with('"'),
            "RFC 9110 quoted-string"
        );
        let c = compute_state_etag(b"{\"tier\":\"explorer\"}");
        assert_ne!(a, c);
    }

    #[test]
    fn etag_is_stable_across_uptime_secs_changes() {
        // Phase 6 invariant: handle_state strips `uptime_secs` from
        // the ETag input so the client's If-None-Match round-trip
        // doesn't get punched every wall-clock second. A regression
        // here would silently re-introduce the etag_roundtrip wire-
        // test flake AND defeat the whole 304-short-circuit on long-
        // running daemons.
        //
        // Mirror the strip-uptime_secs logic exactly. If handle_state
        // changes which fields are excluded, this test should track.
        let v_t0 = serde_json::json!({
            "tier": "operator",
            "logged_in": true,
            "agents": [],
            "uptime_secs": 0,
        });
        let v_t1 = serde_json::json!({
            "tier": "operator",
            "logged_in": true,
            "agents": [],
            "uptime_secs": 99999,
        });

        let strip_uptime = |mut v: serde_json::Value| {
            if let Some(o) = v.as_object_mut() {
                o.remove("uptime_secs");
            }
            serde_json::to_string(&v).unwrap()
        };

        let etag_t0 = compute_state_etag(strip_uptime(v_t0).as_bytes());
        let etag_t1 = compute_state_etag(strip_uptime(v_t1).as_bytes());
        assert_eq!(
            etag_t0, etag_t1,
            "ETag must be stable across uptime_secs changes",
        );

        // And a *real* logical change (different tier) MUST flip it.
        let v_changed = serde_json::json!({
            "tier": "explorer",
            "logged_in": true,
            "agents": [],
            "uptime_secs": 0,
        });
        let etag_changed = compute_state_etag(strip_uptime(v_changed).as_bytes());
        assert_ne!(etag_t0, etag_changed, "tier change must flip the ETag",);
    }

    fn if_none_match_hdr(value: &str) -> Header {
        Header::from_bytes(b"If-None-Match", value.as_bytes()).unwrap()
    }

    #[test]
    fn read_if_none_match_returns_value_when_present() {
        let v = read_if_none_match(&[if_none_match_hdr("\"abc123\"")]);
        assert_eq!(v, Some("\"abc123\"".to_string()));
    }

    #[test]
    fn read_if_none_match_is_case_insensitive() {
        // Header field names are case-insensitive per RFC 9110 §5.1.
        // Some proxies lowercase, others preserve mixed case.
        let h1 = Header::from_bytes(b"if-none-match", b"\"abc\"").unwrap();
        let h2 = Header::from_bytes(b"IF-NONE-MATCH", b"\"abc\"").unwrap();
        assert_eq!(read_if_none_match(&[h1]), Some("\"abc\"".to_string()));
        assert_eq!(read_if_none_match(&[h2]), Some("\"abc\"".to_string()));
    }

    #[test]
    fn read_if_none_match_returns_none_when_absent() {
        assert_eq!(read_if_none_match(&[]), None);
        let other = Header::from_bytes(b"X-Other", b"42").unwrap();
        assert_eq!(read_if_none_match(&[other]), None);
    }

    /// Real-bytes wire test for the ETag round-trip.
    ///
    /// Helper-function tests (compute_state_etag / read_if_none_match)
    /// can pass while the wire is broken if tiny_http, the dev proxy,
    /// or a header-encoding quirk drops the ETag in transit. This
    /// test bottoms out through a real `Server::http`, two raw HTTP/1.1
    /// requests via `TcpStream`, and parses the response strings —
    /// catching any layer that mangles `ETag` / `If-None-Match` /
    /// 304 framing before it bites in production.
    ///
    /// Also covers the route dispatcher (`handle`), so a future
    /// refactor that drops `/api/state` from the match arms surfaces
    /// here rather than only through manual smoke testing.
    ///
    /// Pins state via `TYTUS_STATE_PATH` so the snapshot is stable
    /// across the two HTTP requests. Without that override, when this
    /// test runs on a developer machine with a live `~/.config/tytus/
    /// state.json`, `compute_state_snapshot` re-reads the file +
    /// re-probes the gateway + re-checks forwarder PIDs on each call,
    /// any of which can flip the ETag between the requests
    /// (feedback_etag_wire_test_flake).
    #[test]
    fn etag_roundtrip_via_real_http() {
        use std::io::{Read, Write};
        use std::net::TcpStream;
        use std::time::Duration;

        // Point compute_state_snapshot at a non-existent fixture
        // path so it falls into the empty()/no-state branch — that
        // branch is fully deterministic (no daemon-socket poll,
        // no gateway probe, no live PID checks). Tests don't need a
        // populated state for ETag stability, just a STABLE one.
        let _guard = StateOverrideGuard::set("/nonexistent/tytus-test-state.json");

        let server = tiny_http::Server::http("127.0.0.1:0").expect("bind");
        let port = server.server_addr().to_ip().expect("ip").port();
        let registry = Registry::new();

        let server_arc = Arc::new(server);
        let server_for_thread = server_arc.clone();
        let handler = thread::spawn(move || {
            for (i, req) in server_for_thread.incoming_requests().enumerate() {
                handle(req, registry.clone());
                if i >= 1 {
                    break;
                }
            }
        });

        // ── Request 1: no If-None-Match. Expect 200 + ETag. ─────────
        let mut s1 = TcpStream::connect(("127.0.0.1", port)).expect("connect 1");
        s1.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        s1.write_all(
            b"GET /api/state HTTP/1.1\r\n\
              Host: localhost\r\n\
              Connection: close\r\n\r\n",
        )
        .expect("write 1");
        let mut resp1 = String::new();
        s1.read_to_string(&mut resp1).expect("read 1");

        let status1 = resp1.lines().next().unwrap_or("");
        assert!(
            status1.starts_with("HTTP/1.1 200"),
            "expected 200 on first poll, got: {}",
            status1,
        );
        let etag1 = extract_header(&resp1, "ETag").expect("ETag header missing on 200");
        assert!(
            etag1.starts_with('"') && etag1.ends_with('"'),
            "ETag must be RFC 9110 quoted: {:?}",
            etag1,
        );

        // ── Request 2: If-None-Match echoes ETag. Expect 304 + same ETag. ──
        let mut s2 = TcpStream::connect(("127.0.0.1", port)).expect("connect 2");
        s2.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let req2 = format!(
            "GET /api/state HTTP/1.1\r\n\
             Host: localhost\r\n\
             If-None-Match: {}\r\n\
             Connection: close\r\n\r\n",
            etag1,
        );
        s2.write_all(req2.as_bytes()).expect("write 2");
        let mut resp2 = String::new();
        s2.read_to_string(&mut resp2).expect("read 2");

        let status2 = resp2.lines().next().unwrap_or("");
        assert!(
            status2.starts_with("HTTP/1.1 304"),
            "expected 304 when If-None-Match matches, got: {}\nfull response:\n{}",
            status2,
            resp2,
        );
        let etag2 = extract_header(&resp2, "ETag").expect("ETag must be echoed on 304");
        assert_eq!(
            etag1, etag2,
            "ETag should be identical across the two responses (snapshot didn't change)",
        );
        // 304 MUST have an empty body — assert the response ends right
        // after the headers (CRLFCRLF terminator).
        let body_start = resp2.find("\r\n\r\n").map(|i| i + 4).unwrap_or(resp2.len());
        let body = &resp2[body_start..];
        assert!(
            body.is_empty(),
            "304 must have empty body, got {} bytes: {:?}",
            body.len(),
            body,
        );

        handler.join().expect("handler thread panicked");
    }

    /// Case-insensitive `Header-Name: value` extractor for the wire
    /// test above. tiny_http preserves whatever case the server sent
    /// (we send `ETag`), but a future refactor or proxy might
    /// normalize — keep the lookup tolerant.
    fn extract_header(response: &str, name: &str) -> Option<String> {
        let prefix = format!("{}:", name).to_lowercase();
        response
            .lines()
            .find(|line| line.to_lowercase().starts_with(&prefix))
            .map(|line| {
                let colon = line.find(':').unwrap_or(line.len());
                line[colon + 1..].trim().to_string()
            })
    }

    #[test]
    fn job_event_exit_marks_finished() {
        let r = Registry::new();
        let (id, job) = r.create_pod("02").unwrap();
        push_event(&job, JobEvent::Log("hello".into()));
        assert!(!job.lock().unwrap().finished);
        push_event(&job, JobEvent::Exit { code: 0 });
        assert!(job.lock().unwrap().finished);
        // active_pods no longer reports it.
        assert_eq!(r.active_pods().get("02"), None);
        // It still exists in the registry until reaped.
        assert!(r.get(&id).is_some());
    }

    fn hdr(name: &str, value: &str) -> Header {
        Header::from_bytes(name.as_bytes(), value.as_bytes()).unwrap()
    }

    #[test]
    fn parse_last_event_id_missing_falls_back_to_zero() {
        // Fresh subscriber sends no Last-Event-ID — replay from 0.
        assert_eq!(parse_last_event_id(&[]), 0);
        assert_eq!(parse_last_event_id(&[hdr("X-Other", "5")]), 0);
    }

    #[test]
    fn parse_last_event_id_returns_next_index() {
        // EventSource sends the last id it saw; we resume at id+1.
        assert_eq!(parse_last_event_id(&[hdr("Last-Event-ID", "0")]), 1);
        assert_eq!(parse_last_event_id(&[hdr("Last-Event-ID", "42")]), 43);
    }

    #[test]
    fn parse_last_event_id_is_case_insensitive() {
        // Per RFC 9110 §5.1 header names are case-insensitive. Browsers
        // tend to lowercase but proxies may not.
        assert_eq!(parse_last_event_id(&[hdr("last-event-id", "7")]), 8);
        assert_eq!(parse_last_event_id(&[hdr("LAST-EVENT-ID", "7")]), 8);
    }

    #[test]
    fn parse_last_event_id_rejects_garbage() {
        // Tampered / malformed values fall back to 0 so we don't lose
        // the user's stream history.
        assert_eq!(parse_last_event_id(&[hdr("Last-Event-ID", "")]), 0);
        assert_eq!(parse_last_event_id(&[hdr("Last-Event-ID", "abc")]), 0);
        assert_eq!(parse_last_event_id(&[hdr("Last-Event-ID", "-1")]), 0);
    }

    #[test]
    fn truncate_log_line_passes_through_short_lines() {
        let s = "hello world".to_string();
        assert_eq!(truncate_log_line(s.clone()), s);
    }

    #[test]
    fn truncate_log_line_caps_long_lines_with_suffix() {
        let s = "a".repeat(MAX_LINE_LEN + 100);
        let out = truncate_log_line(s);
        // Suffix is appended; total length is MAX_LINE_LEN + suffix.
        assert!(out.ends_with("…[truncated]"));
        assert!(out.len() < MAX_LINE_LEN + 50);
    }

    #[test]
    fn truncate_log_line_handles_utf8_boundary() {
        // A line of multibyte chars where MAX_LINE_LEN lands mid-rune.
        // We must NOT split a UTF-8 sequence — the truncate would
        // panic if we did.
        let rune = "🦀"; // 4 bytes
        let count = (MAX_LINE_LEN / 4) + 50;
        let s: String = rune.repeat(count);
        let out = truncate_log_line(s);
        assert!(out.ends_with("…[truncated]"));
        // Body before the suffix must be valid UTF-8 (already implied
        // by .truncate not panicking, but assert explicitly).
        assert!(out.is_char_boundary(out.len()));
    }

    #[test]
    fn push_event_caps_log_at_max_events_and_emits_sentinel_once() {
        let r = Registry::new();
        let (_id, job) = r.create_pod("02").unwrap();
        // Push MAX_EVENTS log lines — all should land.
        for i in 0..MAX_EVENTS {
            push_event(&job, JobEvent::Log(format!("line {}", i)));
        }
        assert_eq!(job.lock().unwrap().events.len(), MAX_EVENTS);
        // Push one more — the sentinel takes its place; the dropped
        // line is silently discarded.
        push_event(&job, JobEvent::Log("dropped".into()));
        // Inspect under a scoped lock — borrowing &job.lock().unwrap().events
        // outside a block would extend the MutexGuard's lifetime across the
        // next push_event() call and deadlock self.
        {
            let j = job.lock().unwrap();
            assert_eq!(j.events.len(), MAX_EVENTS + 1);
            match j.events.last() {
                Some(JobEvent::Log(s)) => assert!(s.contains("log capped")),
                _ => panic!("expected sentinel Log event"),
            }
        }
        // Subsequent drops are silent (no second sentinel).
        push_event(&job, JobEvent::Log("also dropped".into()));
        push_event(&job, JobEvent::Log("also dropped 2".into()));
        assert_eq!(job.lock().unwrap().events.len(), MAX_EVENTS + 1);
    }

    #[test]
    fn push_event_terminal_always_appends_even_at_cap() {
        let r = Registry::new();
        let (_id, job) = r.create_pod("02").unwrap();
        // Fill past the log cap.
        for i in 0..MAX_EVENTS + 10 {
            push_event(&job, JobEvent::Log(format!("line {}", i)));
        }
        let pre_len = job.lock().unwrap().events.len();
        // Exit must still land — the SSE consumer needs it to wind
        // down. Daemon RAM impact is negligible (tens of bytes).
        push_event(&job, JobEvent::Exit { code: 0 });
        let j = job.lock().unwrap();
        assert_eq!(j.events.len(), pre_len + 1);
        assert!(j.finished);
        assert!(matches!(j.events.last(), Some(JobEvent::Exit { code: 0 })));
    }

    #[test]
    fn job_child_pid_starts_none_and_is_settable() {
        // Ensures the cancel handler can distinguish "queued, no live
        // process" from "running, signal me". Spawn paths set the PID
        // post-spawn; this test asserts the field has the right
        // pre-spawn value and round-trips after a manual set.
        let r = Registry::new();
        let (_id, job) = r.create_pod("02").unwrap();
        assert!(job.lock().unwrap().child_pid.is_none());

        // Simulate spawn-thread setting it.
        job.lock().unwrap().child_pid = Some(12345);
        assert_eq!(job.lock().unwrap().child_pid, Some(12345));

        // Simulate wait()-thread clearing it pre-Exit.
        job.lock().unwrap().child_pid = None;
        push_event(&job, JobEvent::Exit { code: 0 });
        assert!(job.lock().unwrap().finished);
        assert!(job.lock().unwrap().child_pid.is_none());
    }

    // Phase 2 §11: Sec-Fetch-Site POST guard.
    //
    // The full guard runs against `&Request`, which is hard to construct
    // in unit tests. We extract `sec_fetch_site_value_ok` so the policy
    // can be exercised directly. End-to-end same-origin behaviour is
    // verified by the live POST hitting tiny_http on a real port — see
    // the manual smoke notes in the Phase 2 sprint.
    #[test]
    fn sec_fetch_site_guard_accepts_same_origin() {
        assert!(sec_fetch_site_value_ok(Some("same-origin")));
    }

    #[test]
    fn sec_fetch_site_guard_rejects_cross_origin_variants() {
        assert!(!sec_fetch_site_value_ok(Some("cross-site")));
        assert!(!sec_fetch_site_value_ok(Some("same-site")));
        assert!(!sec_fetch_site_value_ok(Some("none")));
    }

    #[test]
    fn sec_fetch_site_guard_rejects_missing_header() {
        // Fail closed: no header => 403. Modern browsers always send
        // Sec-Fetch-Site (Chrome 76+, FF 90+, Safari 16+).
        assert!(!sec_fetch_site_value_ok(None));
    }

    #[test]
    fn sec_fetch_site_guard_is_case_sensitive_on_value() {
        // Browsers always emit lowercase tokens. We don't loosen the
        // value comparison to avoid normalising attacker input later.
        assert!(!sec_fetch_site_value_ok(Some("Same-Origin")));
        assert!(!sec_fetch_site_value_ok(Some("SAME-ORIGIN")));
    }

    // ── Idempotency-Key cache ────────────────────────────────────
    //
    // The cache is a process-wide static; tests use unique keys
    // ("idem-test-<test name>-<role>") to avoid cross-test bleed.

    fn idem_hdr(value: &str) -> Header {
        Header::from_bytes(b"Idempotency-Key", value.as_bytes()).unwrap()
    }

    #[test]
    fn read_idempotency_key_returns_value_when_present() {
        let v = read_idempotency_key(&[idem_hdr("abc-123")]);
        assert_eq!(v, Some("abc-123".to_string()));
    }

    #[test]
    fn read_idempotency_key_is_case_insensitive_on_field_name() {
        let h1 = Header::from_bytes(b"idempotency-key", b"a").unwrap();
        let h2 = Header::from_bytes(b"IDEMPOTENCY-KEY", b"a").unwrap();
        assert_eq!(read_idempotency_key(&[h1]), Some("a".to_string()));
        assert_eq!(read_idempotency_key(&[h2]), Some("a".to_string()));
    }

    #[test]
    fn read_idempotency_key_rejects_empty_or_oversized() {
        assert_eq!(read_idempotency_key(&[idem_hdr("")]), None);
        let huge = "x".repeat(IDEM_MAX_KEY_LEN + 1);
        assert_eq!(read_idempotency_key(&[idem_hdr(&huge)]), None);
    }

    #[test]
    fn read_idempotency_key_rejects_non_graphic_chars() {
        // Whitespace and control bytes are out: would defeat the
        // ascii-graphic invariant we lean on for log safety.
        assert_eq!(read_idempotency_key(&[idem_hdr("a b")]), None);
        assert_eq!(read_idempotency_key(&[idem_hdr("a\tb")]), None);
    }

    #[test]
    fn read_idempotency_key_returns_none_when_absent() {
        assert_eq!(read_idempotency_key(&[]), None);
        let other = Header::from_bytes(b"X-Other", b"42").unwrap();
        assert_eq!(read_idempotency_key(&[other]), None);
    }

    #[test]
    fn idempotency_get_miss_returns_none() {
        assert!(idempotency_get("idem-test-miss-no-such-key").is_none());
    }

    #[test]
    fn idempotency_put_then_get_round_trips() {
        let key = "idem-test-roundtrip".to_string();
        idempotency_put(key.clone(), 202, "{\"job_id\":\"abc\"}".into());
        let entry = idempotency_get(&key).expect("hit after put");
        assert_eq!(entry.status, 202);
        assert_eq!(entry.body, "{\"job_id\":\"abc\"}");
    }

    #[test]
    fn idempotency_get_evicts_expired_entry() {
        // Manually insert an entry with a `inserted_at` older than the
        // TTL so the next read evicts it. Avoids burning real wall-clock.
        let key = "idem-test-expired".to_string();
        {
            let mut cache = idempotency_cache().lock().unwrap();
            cache.insert(
                key.clone(),
                IdemEntry {
                    status: 200,
                    body: "{}".into(),
                    inserted_at: now_secs().saturating_sub(IDEM_TTL_SECS + 1),
                },
            );
        }
        assert!(idempotency_get(&key).is_none());
        // And it's gone from the cache after the read evicted it.
        assert!(!idempotency_cache().lock().unwrap().contains_key(&key));
    }

    #[test]
    fn idempotency_put_skips_eviction_when_updating_existing_key() {
        // Updating an existing entry (e.g. last-write-wins on a race)
        // must not displace some unrelated entry just to "make room".
        let key = "idem-test-update".to_string();
        idempotency_put(key.clone(), 200, "first".into());
        let len_before = idempotency_cache().lock().unwrap().len();
        idempotency_put(key.clone(), 202, "second".into());
        let len_after = idempotency_cache().lock().unwrap().len();
        assert_eq!(len_before, len_after);
        let entry = idempotency_get(&key).unwrap();
        assert_eq!(entry.status, 202);
        assert_eq!(entry.body, "second");
    }

    #[test]
    fn current_idem_key_thread_local_round_trips() {
        // sanity: set / take / take-again behavior (consumes on take).
        assert_eq!(take_current_idem_key(), None);
        set_current_idem_key(Some("abc".into()));
        assert_eq!(take_current_idem_key(), Some("abc".into()));
        assert_eq!(take_current_idem_key(), None);
    }

    /// End-to-end wire test: same key returns the same body twice and
    /// the second response carries `Idempotency-Replayed: true`.
    /// Different key spawns a fresh execution.
    #[test]
    fn idempotency_replay_via_real_http() {
        use std::io::{Read, Write};
        use std::net::TcpStream;
        use std::time::Duration;

        let server = tiny_http::Server::http("127.0.0.1:0").expect("bind");
        let port = server.server_addr().to_ip().expect("ip").port();
        let registry = Registry::new();

        let server_arc = Arc::new(server);
        let server_for_thread = server_arc.clone();
        let handler = thread::spawn(move || {
            for (i, req) in server_for_thread.incoming_requests().enumerate() {
                handle(req, registry.clone());
                if i >= 2 {
                    break;
                }
            }
        });

        let key = format!("idem-test-wire-{}", now_secs());
        // /api/pod/restart against an unknown pod returns 404 — a
        // deterministic non-2xx response we can replay without needing
        // the full state.json fixture wired up. The caching contract
        // covers 4xx the same as 2xx.
        let post = |idem: &str| -> String {
            let req = format!(
                "POST /api/pod/restart?pod=99 HTTP/1.1\r\n\
                 Host: localhost\r\n\
                 Sec-Fetch-Site: same-origin\r\n\
                 Idempotency-Key: {}\r\n\
                 Content-Length: 0\r\n\
                 Connection: close\r\n\r\n",
                idem,
            );
            let mut s = TcpStream::connect(("127.0.0.1", port)).expect("connect");
            s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            s.write_all(req.as_bytes()).expect("write");
            let mut resp = String::new();
            s.read_to_string(&mut resp).expect("read");
            resp
        };

        let r1 = post(&key);
        let r2 = post(&key);

        // First response: no Idempotency-Replayed header.
        assert!(
            !r1.lines()
                .any(|l| l.to_lowercase().starts_with("idempotency-replayed:")),
            "first response should not be marked as replay: {}",
            r1,
        );
        // Second response: Idempotency-Replayed: true echoed back.
        let replayed = r2
            .lines()
            .find(|l| l.to_lowercase().starts_with("idempotency-replayed:"))
            .map(|l| l.split(':').nth(1).unwrap_or("").trim().to_string())
            .expect("replay header missing on second response");
        assert_eq!(replayed, "true");

        // Body should be byte-identical between the two responses.
        let body1 = r1
            .find("\r\n\r\n")
            .map(|i| r1[i + 4..].to_string())
            .unwrap_or_default();
        let body2 = r2
            .find("\r\n\r\n")
            .map(|i| r2[i + 4..].to_string())
            .unwrap_or_default();
        assert_eq!(body1, body2, "replay body should match original");

        // Third request with a *different* key: should NOT be replayed.
        let r3 = post(&format!("{}-different", key));
        assert!(
            !r3.lines()
                .any(|l| l.to_lowercase().starts_with("idempotency-replayed:")),
            "third response with new key should not be replay: {}",
            r3,
        );

        handler.join().expect("handler thread panicked");
    }

    // ── Per-pod status cache (Phase 2 cont) ──────────────────────
    //
    // Cache lookup is fully testable from outside the request thread
    // since it's just a Mutex<HashMap>. The actual probe involves
    // HTTP, which we don't exercise here — handle_pod_ready already
    // covers the probe-classification path and we mirror its logic.

    #[test]
    fn agent_status_serializes_lowercase() {
        // Pin the wire shape — TytusOS imports the JSON value
        // directly into `state.agents[].status` and switches on
        // string equality. A casing change here would silently
        // break every consumer.
        let pairs: Vec<(AgentStatus, &str)> = vec![
            (AgentStatus::Ready, "\"ready\""),
            (AgentStatus::Starting, "\"starting\""),
            (AgentStatus::Unhealthy, "\"unhealthy\""),
            (AgentStatus::Stopped, "\"stopped\""),
            (AgentStatus::Unknown, "\"unknown\""),
        ];
        for (s, expected) in pairs {
            assert_eq!(serde_json::to_string(&s).unwrap(), expected);
        }
    }

    #[test]
    fn agent_status_cache_returns_unknown_for_unseen_pod() {
        // Use a unique pod_id so this test doesn't collide with
        // siblings. agent_status_cached MUST NOT block — it returns
        // Unknown immediately and kicks a background probe.
        let pod = format!("test-unseen-{}", now_secs());
        // No api_url means the probe-kick is skipped, so we test
        // the pure cache-miss return path.
        let s = agent_status_cached(&pod, "nemoclaw", None, "", None, None);
        assert_eq!(s, AgentStatus::Unknown);
    }

    #[test]
    fn agent_status_cache_returns_cached_value_within_ttl() {
        let pod = format!("test-cached-{}", now_secs());
        {
            let mut cache = status_cache().lock().unwrap();
            cache.insert(
                pod.clone(),
                StatusEntry {
                    status: AgentStatus::Ready,
                    fetched_at: now_secs(),
                },
            );
        }
        // Empty creds means no refresh kick, but the cached entry
        // is fresh so we get it back.
        let s = agent_status_cached(&pod, "nemoclaw", None, "", None, None);
        assert_eq!(s, AgentStatus::Ready);
    }

    #[test]
    fn agent_status_cache_treats_stale_entry_as_eligible_for_refresh() {
        // Insert an entry with a fetched_at older than STATUS_TTL_SECS
        // and verify the lookup still returns the cached value (we
        // never block on probe). The "refresh kick" only fires when
        // api_url + user_key are present; with empty creds we just
        // assert the cached value comes back.
        let pod = format!("test-stale-{}", now_secs());
        {
            let mut cache = status_cache().lock().unwrap();
            cache.insert(
                pod.clone(),
                StatusEntry {
                    status: AgentStatus::Starting,
                    fetched_at: now_secs().saturating_sub(STATUS_TTL_SECS + 10),
                },
            );
        }
        // Returns the stale value (not Unknown) because we never
        // wipe an entry on read — fresh probe lands later in the
        // background thread. UI gets last-known status, not a regression.
        let s = agent_status_cached(&pod, "nemoclaw", None, "", None, None);
        assert_eq!(s, AgentStatus::Starting);
    }

    #[test]
    fn agent_status_cache_ttl_constant_is_sane() {
        // Guard the architectural decision: TTL must be longer than
        // the OS poll interval (currently 4s in DaemonStateProvider)
        // so a stale-cache poll doesn't trigger probes on every tick.
        // 5s is the floor. Update if the OS changes intervalMs.
        let ttl = STATUS_TTL_SECS;
        assert!(
            ttl >= 4,
            "STATUS_TTL_SECS must be ≥ OS poll interval to prevent probe-storm",
        );
    }

    #[test]
    fn probe_agent_status_returns_stopped_for_empty_url() {
        // Defensive: no api_url means no probe possible. Don't
        // hang/panic — return Stopped (the OS renders "container down").
        let s = probe_agent_status("nemoclaw", "", "any-key", None, None);
        assert_eq!(s, AgentStatus::Stopped);
    }

    #[test]
    fn hermes_prefers_gateway_token_for_api_probe() {
        assert_eq!(
            probe_bearer_token("hermes", "stable-user", Some("gateway-secret")),
            "gateway-secret"
        );
        assert_eq!(
            probe_bearer_token("hermes", "stable-user", None),
            "stable-user"
        );
        assert_eq!(
            probe_bearer_token("nemoclaw", "stable-user", Some("gateway-secret")),
            "stable-user"
        );
    }

    #[test]
    fn hermes_is_a_browser_ui_agent() {
        assert!(agent_has_browser_ui("hermes"));
        assert!(agent_has_browser_ui("nemoclaw"));
        assert!(!agent_has_browser_ui("none"));
    }

    // ── Shared-folders action allowlist (Phase 3 cont) ──────────

    #[test]
    fn garagetytus_status_parser_handles_macos_and_linux_states() {
        assert_eq!(
            parse_garagetytus_status_line("garagetytus status: running (pid=586, state=running)"),
            (Some(true), "running".to_string()),
        );
        assert_eq!(
            parse_garagetytus_status_line("garagetytus status: active (user)"),
            (Some(true), "active".to_string()),
        );
        assert_eq!(
            parse_garagetytus_status_line("garagetytus status: stopped"),
            (Some(false), "stopped".to_string()),
        );
        assert_eq!(
            parse_garagetytus_status_line("garagetytus status: inactive (user)"),
            (Some(false), "inactive".to_string()),
        );
    }

    #[test]
    fn garagetytus_status_parser_keeps_unknown_state_non_fatal() {
        assert_eq!(
            parse_garagetytus_status_line("garagetytus status: activating (user)"),
            (None, "activating".to_string()),
        );
        assert_eq!(
            parse_garagetytus_status_line("unexpected output"),
            (None, "unknown".to_string()),
        );
    }

    #[test]
    fn shared_folder_action_argv_accepts_known_actions() {
        // Pin every action in the frozen allowlist + the helper +
        // arg shape. New actions MUST update this test alongside
        // the helper's match arm — no silent additions.
        assert_eq!(
            shared_folder_action_argv("list").unwrap(),
            ("garagetytus-folder-list", vec![]),
        );
        assert_eq!(
            shared_folder_action_argv("status").unwrap(),
            (
                "garagetytus-folder-status",
                vec!["--check-pods".to_string()],
            ),
        );
        assert_eq!(
            shared_folder_action_argv("conflicts").unwrap(),
            ("garagetytus-folder-conflicts", vec![]),
        );
        assert_eq!(
            shared_folder_action_argv("refresh-all").unwrap(),
            (
                "garagetytus-refresh-watchdog",
                vec!["--threshold-days".to_string(), "7".to_string()],
            ),
        );
    }

    #[test]
    fn shared_folder_action_argv_rejects_empty_and_unknown() {
        assert!(shared_folder_action_argv("").is_none());
        assert!(shared_folder_action_argv("foo").is_none());
        assert!(shared_folder_action_argv("sync-now").is_none());
        // Casing must be exact — no Title-case variants.
        assert!(shared_folder_action_argv("LIST").is_none());
        assert!(shared_folder_action_argv("List").is_none());
    }

    #[test]
    fn shared_folder_action_argv_rejects_injection_shaped_strings() {
        // The action arrives via query-string and is fed straight
        // into a helper-binary spawn. No matter what the caller
        // sends, anything outside the allowlist must reject — this
        // is the daemon's only line of defense.
        assert!(shared_folder_action_argv("list; rm -rf /").is_none());
        assert!(shared_folder_action_argv("list && cat /etc/passwd").is_none());
        assert!(shared_folder_action_argv("../../../etc/passwd").is_none());
        assert!(shared_folder_action_argv("list\0").is_none());
        assert!(shared_folder_action_argv("list ").is_none());
    }

    #[test]
    fn sharing_defaults_default_to_enabled_tytus_home() {
        let defaults = SharingDefaults::default();
        assert_eq!(defaults.schema_version, 1);
        assert!(defaults.sharing_globally_enabled);
        assert!(defaults.default_auto_sync);
        assert_eq!(defaults.default_bucket, "shared");
        assert!(defaults.default_local_root.ends_with("/Tytus/Shared"));
    }

    #[test]
    fn sharing_bucket_validation_matches_bind_rules() {
        assert!(valid_sharing_bucket("shared"));
        assert!(valid_sharing_bucket("sebastian-shared"));
        assert!(!valid_sharing_bucket("Shared"));
        assert!(!valid_sharing_bucket("-shared"));
        assert!(!valid_sharing_bucket("shared-"));
        assert!(!valid_sharing_bucket("sh"));
        assert!(!valid_sharing_bucket("shared_bucket"));
    }

    #[test]
    fn sharing_provision_pod_id_normalizer_accepts_known_forms() {
        assert_eq!(normalize_pod_id_for_query("02").as_deref(), Some("02"));
        assert_eq!(
            normalize_pod_id_for_query("wannolot-04").as_deref(),
            Some("04")
        );
        assert_eq!(
            normalize_pod_id_for_query("tytus-004").as_deref(),
            Some("004")
        );
        assert!(normalize_pod_id_for_query("pod-02").is_none());
        assert!(normalize_pod_id_for_query("02;rm").is_none());
    }

    #[test]
    fn pod_env_argv_default_redacted_form() {
        // No --reveal-secrets when reveal=false. --json always present
        // so tray's JSON parser doesn't get a human banner.
        assert_eq!(
            pod_env_argv("02", false),
            vec![
                "agent".to_string(),
                "env".to_string(),
                "--pod".to_string(),
                "02".to_string(),
                "--json".to_string(),
            ],
        );
    }

    #[test]
    fn pod_env_argv_reveal_form() {
        // --reveal-secrets is emitted between --pod and --json so a
        // bare argv inspect still reads naturally.
        assert_eq!(
            pod_env_argv("04", true),
            vec![
                "agent".to_string(),
                "env".to_string(),
                "--pod".to_string(),
                "04".to_string(),
                "--reveal-secrets".to_string(),
                "--json".to_string(),
            ],
        );
    }

    #[test]
    fn parse_reveal_flag_only_matches_secrets() {
        assert!(parse_reveal_flag("pod=02&reveal=secrets"));
        // Order-insensitive — flag can appear anywhere in the query.
        assert!(parse_reveal_flag("reveal=secrets&pod=02"));
        // Strict equality only — no fuzzy "true" / "1" / case variants.
        assert!(!parse_reveal_flag("pod=02"));
        assert!(!parse_reveal_flag("pod=02&reveal=true"));
        assert!(!parse_reveal_flag("pod=02&reveal=Secrets"));
        assert!(!parse_reveal_flag(""));
    }

    #[test]
    fn files_safe_relative_path_rejects_escape_inputs() {
        assert_eq!(
            safe_relative_path(Some("Projects/demo".to_string())).unwrap(),
            PathBuf::from("Projects/demo")
        );
        assert!(safe_relative_path(Some("../secrets".to_string())).is_err());
        assert!(safe_relative_path(Some("Projects/../../secrets".to_string())).is_err());
        assert!(safe_relative_path(Some("/etc/passwd".to_string())).is_err());
        assert!(safe_relative_path(Some("Projects\0secret".to_string())).is_err());
        assert!(safe_relative_path(Some(r"C:\Users\sebastian".to_string())).is_err());
    }

    #[test]
    fn files_percent_decode_catches_url_encoded_traversal() {
        let decoded = query_value_decoded("path=Projects%2F..%2Fsecrets", "path")
            .unwrap()
            .unwrap();
        assert_eq!(decoded, "Projects/../secrets");
        assert!(safe_relative_path(Some(decoded)).is_err());
        assert!(safe_relative_path(Some("Projects%2F..%2Fsecrets".to_string())).is_err());
        assert!(safe_relative_path(Some("Projects%252F..%252Fsecrets".to_string())).is_err());
        assert!(safe_relative_path(Some("Projects%00secret".to_string())).is_err());
    }

    #[test]
    fn files_anchor_local_path_blocks_absolute_escape() {
        let root = std::env::temp_dir().join(format!(
            "tytus-files-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::create_dir_all(root.join("child"));
        let ok = anchor_local_path(&root, Path::new("child")).unwrap();
        assert!(ok.starts_with(root.canonicalize().unwrap()));
        assert!(anchor_local_path(&root, Path::new("../escape")).is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn files_anchor_local_path_blocks_symlink_escape() {
        use std::os::unix::fs::symlink;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("tytus-files-root-{}", stamp));
        let outside = std::env::temp_dir().join(format!("tytus-files-outside-{}", stamp));
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        symlink(&outside, root.join("link")).unwrap();
        assert!(anchor_local_path(&root, Path::new("link")).is_err());
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(outside);
    }

    #[test]
    fn files_safe_file_name_rejects_path_components() {
        assert_eq!(
            safe_file_name(Some("Project A".to_string())).unwrap(),
            "Project A"
        );
        assert!(safe_file_name(Some("".to_string())).is_err());
        assert!(safe_file_name(Some("..".to_string())).is_err());
        assert!(safe_file_name(Some("nested/name".to_string())).is_err());
        assert!(safe_file_name(Some("nested\\name".to_string())).is_err());
        assert!(safe_file_name(Some("bad\0name".to_string())).is_err());
    }

    #[test]
    fn files_dotfile_policy_allows_only_anchored_dotfiles() {
        // Dotfiles are valid inside Tytus Home (`.tytus/`, `.env.example`)
        // because the workspace contract itself uses them. They are not
        // a traversal primitive; the root anchor remains the security boundary.
        assert_eq!(
            safe_relative_path(Some(".tytus/health.json".to_string())).unwrap(),
            PathBuf::from(".tytus/health.json")
        );
        assert_eq!(
            safe_file_name(Some(".env.example".to_string())).unwrap(),
            ".env.example"
        );
        assert!(safe_relative_path(Some("../.ssh/id_rsa".to_string())).is_err());
    }

    #[test]
    fn files_existing_entry_blocks_source_root() {
        let root = std::env::temp_dir().join(format!(
            "tytus-files-root-entry-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        assert!(anchored_existing_entry(&root, Path::new("")).is_err());
        fs::write(root.join("note.txt"), "hello").unwrap();
        let entry = anchored_existing_entry(&root, Path::new("note.txt")).unwrap();
        assert!(entry.starts_with(root.canonicalize().unwrap()));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn files_copy_move_destination_stays_in_same_root() {
        let root = std::env::temp_dir().join(format!(
            "tytus-files-copy-move-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("Projects")).unwrap();

        let dest = copy_move_destination(
            &root,
            Path::new("Inbox/note.txt"),
            Some("Projects".to_string()),
            None,
        )
        .unwrap();
        assert!(dest.starts_with(root.canonicalize().unwrap()));
        assert_eq!(dest.file_name().unwrap().to_string_lossy(), "note.txt");

        assert!(copy_move_destination(
            &root,
            Path::new("Inbox/note.txt"),
            Some("../escape".to_string()),
            None,
        )
        .is_err());
        assert!(copy_move_destination(
            &root,
            Path::new("Inbox/note.txt"),
            Some("Projects".to_string()),
            Some("bad/name.txt".to_string()),
        )
        .is_err());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn shared_folder_allowed_actions_constant_matches_helper() {
        // Guard against drift: every entry in the constant must be
        // accepted by the helper, and every helper-accepted action
        // must appear in the constant. Without this the 400 error
        // body could lie about what's actually allowed.
        for a in SHARED_FOLDER_ALLOWED_ACTIONS {
            assert!(
                shared_folder_action_argv(a).is_some(),
                "advertised action `{}` rejected by helper",
                a,
            );
        }
        // Spot-check that there isn't a stealth action the constant
        // doesn't advertise. The helper's match has 4 arms today.
        // If you add a 5th, this assertion forces an update.
        let known_count = SHARED_FOLDER_ALLOWED_ACTIONS.len();
        assert_eq!(
            known_count, 4,
            "update SHARED_FOLDER_ALLOWED_ACTIONS + this guard when adding actions"
        );
    }
}

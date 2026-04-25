//! Local HTTP server for the Tytus Tower control page.
//!
//! Binds to `127.0.0.1:<random>` at tray startup, serves embedded HTML/CSS/JS,
//! and exposes a tiny local API so the static JS can (a) list the agent
//! catalog, (b) kick off a `tytus agent install` subprocess, and (c) stream
//! its stdout back via server-sent events. Legacy `/install` paths 302 to
//! `/tower` for anyone with a bookmark or old external link.
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
//!   `<tmp>/tytus/tray-web.port` so `open_tower()` can read it.
//! - Lifecycle: server thread owns the `tiny_http::Server` and parks on
//!   `recv()`. On tray quit we drop the `Arc<Server>` and the kernel
//!   tears down the listener.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

// ── Embedded static assets ────────────────────────────────────
// `include_bytes!` paths are relative to THIS source file, so the web/
// directory lives next to src/.
const TOWER_HTML: &[u8] = include_bytes!("../web/tower.html");
const TOWER_CSS: &[u8] = include_bytes!("../web/assets/tower.css");
const TOWER_JS: &[u8] = include_bytes!("../web/assets/tower.js");

// ── Icons (lobehub @1.87.0) ───────────────────────────────────
// Baked into the binary so the CSP can stay 'self'-only and the
// wizard renders even when the laptop is offline. Mapping:
//   openclaw.svg → agent id "nemoclaw" (display name is OpenClaw)
//   hermes.svg   → agent id "hermes"
//   nvidia.svg   → reserved for future sandbox badge
const ICON_OPENCLAW: &[u8] = include_bytes!("../web/assets/icons/openclaw.svg");
const ICON_HERMES: &[u8] = include_bytes!("../web/assets/icons/hermes.svg");
const ICON_NVIDIA: &[u8] = include_bytes!("../web/assets/icons/nvidia.svg");

// ── Job registry ──────────────────────────────────────────────
//
// Each install job is a live subprocess (tytus agent install <type>)
// plus a channel of streaming events for in-flight SSE consumers. Jobs
// are indexed by a random-ish id so the browser can reconnect to a
// running job if the EventSource hiccups.

enum JobEvent {
    Log(String),
    Done { payload: String },     // install-shaped: payload is the CLI's JSON result
    Fail { message: String },
    Exit { code: i32 },           // pod-action-shaped: process exited with code
}

struct Job {
    events: Vec<JobEvent>,
    finished: bool,
    /// `None` for the install flow (one global install at a time);
    /// `Some(pod_id)` for per-pod actions so `Registry::active_for_pod`
    /// can enforce one-running-action-per-pod and the Tower UI can
    /// badge pod rows that have a live job.
    pod_id: Option<String>,
}

impl Job {
    fn new_install() -> Self {
        Job { events: Vec::new(), finished: false, pod_id: None }
    }
    fn new_pod(pod_id: String) -> Self {
        Job { events: Vec::new(), finished: false, pod_id: Some(pod_id) }
    }
}

#[derive(Clone)]
struct Registry {
    inner: Arc<Mutex<HashMap<String, Arc<Mutex<Job>>>>>,
}

impl Registry {
    fn new() -> Self {
        Registry { inner: Arc::new(Mutex::new(HashMap::new())) }
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
    /// → count. Surfaced in StateSnapshot so the Tower overview can
    /// dot pod rows that have a live action streaming.
    fn active_pods(&self) -> HashMap<String, usize> {
        let mut out: HashMap<String, usize> = HashMap::new();
        for job in self.inner.lock().unwrap().values() {
            let j = job.lock().unwrap();
            if j.finished { continue; }
            if let Some(pod) = &j.pod_id {
                *out.entry(pod.clone()).or_insert(0) += 1;
            }
        }
        out
    }
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

// ── Public entry ──────────────────────────────────────────────

/// Spawn the wizard server on a random localhost port and return the port.
///
/// Returns `None` if bind failed (very rare — only when 127.0.0.1 itself
/// isn't available). Caller stores the returned port for `open_tower()`.
pub fn start() -> Option<u16> {
    let server = match Server::http("127.0.0.1:0") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[tray-web] failed to bind: {}", e);
            return None;
        }
    };
    let port = server.server_addr().to_ip()?.port();

    // Persist the port so subsequent "Install Agent" clicks (which call
    // `open_tower`) can read it without a lookup.
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

pub fn open_tower() {
    open_tower_at("");
}

/// Open Tower at a specific URL fragment so the tray menu can deep-link
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
pub fn open_tower_at(fragment: &str) {
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
        format!("http://127.0.0.1:{}/tower", port)
    } else {
        let sep = if fragment.contains('?') { '&' } else { '?' };
        format!("http://127.0.0.1:{}/tower{}{}n={:x}", port, fragment, sep, nonce)
    };
    #[cfg(target_os = "macos")]
    { let _ = Command::new("open").arg(&url).spawn(); }
    #[cfg(target_os = "linux")]
    { let _ = Command::new("xdg-open").arg(&url).spawn(); }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    { let _ = url; }
}

fn port_file() -> Option<PathBuf> {
    Some(PathBuf::from("/tmp/tytus/tray-web.port"))
}

fn current_port() -> Option<u16> {
    let raw = std::fs::read_to_string(port_file()?).ok()?;
    raw.trim().parse().ok()
}

// ── Request router ────────────────────────────────────────────

fn handle(request: Request, registry: Registry) {
    let method = request.method().clone();
    let url = request.url().to_string();
    let path = url.split('?').next().unwrap_or("").to_string();
    let query = url.split_once('?').map(|(_, q)| q.to_string()).unwrap_or_default();

    match (&method, path.as_str()) {
        (Method::Get, "/tower") | (Method::Get, "/") => {
            serve_bytes(request, TOWER_HTML, "text/html; charset=utf-8");
        }
        // Back-compat: external bookmarks and the pre-rename `open_tower`
        // URL both aimed at `/install`. Redirect instead of duplicating
        // the serve path — keeps the canonical URL visible in the address
        // bar after the redirect resolves.
        (Method::Get, "/install") => {
            let resp = Response::from_string("")
                .with_status_code(StatusCode(302))
                .with_header(header("Location", "/tower"));
            let _ = request.respond(resp);
        }
        (Method::Get, "/assets/tower.css") => {
            serve_bytes(request, TOWER_CSS, "text/css; charset=utf-8");
        }
        (Method::Get, "/assets/tower.js") => {
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
        (Method::Get, p) if p.starts_with("/api/jobs/") && p.ends_with("/stream") => {
            let job_id = p
                .trim_start_matches("/api/jobs/")
                .trim_end_matches("/stream")
                .to_string();
            handle_stream(request, &registry, &job_id);
        }
        (Method::Get, "/api/state") => {
            handle_state(request, &registry);
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
        // Revoke a pod — frees its units immediately and wipes the
        // pod's workspace state. Wizard's "Revoke & try again" button
        // after an install failure calls this to reset before retry.
        (Method::Post, "/api/pod/revoke") => {
            handle_pod_revoke(request, &query);
        }
        // Phase B: per-pod streamed action. Body is { "action": "doctor"
        // | "restart" | "revoke" | "uninstall" | "stop-forwarder" }.
        // Returns { job_id }; output streams via /api/jobs/<id>/stream.
        (Method::Post, p) if p.starts_with("/api/pod/")
                          && p.ends_with("/run-streamed") => {
            let pod = p
                .trim_start_matches("/api/pod/")
                .trim_end_matches("/run-streamed")
                .to_string();
            handle_pod_run_streamed(request, &registry, pod);
        }
        // ── Tower control-surface endpoints (Wave 1) ─────────────────
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
        (Method::Post, "/api/logout") => {
            handle_logout(request);
        }
        // ── Tower Wave 2: Troubleshoot surface ───────────────────────
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
        // ── Tower Wave 3b: launch in editor ──────────────────────────
        (Method::Get, "/api/launchers") => {
            handle_launchers_list(request);
        }
        (Method::Post, "/api/launch") => {
            handle_launch(request, &query);
        }
        // ── Tower Wave 3c: per-pod channels ──────────────────────────
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
        // ── Tower Wave 4: sync gaps ──────────────────────────────────
        (Method::Post, "/api/pod/stop-forwarder") => {
            handle_pod_stop_forwarder(request, &query);
        }
        (Method::Post, "/api/configure") => {
            handle_configure(request);
        }
        _ => {
            let resp = Response::from_string("not found")
                .with_status_code(StatusCode(404));
            let _ = request.respond(resp);
        }
    }
}

fn serve_bytes(request: Request, body: &[u8], content_type: &str) {
    let resp = Response::from_data(body.to_vec())
        .with_header(header("Content-Type", content_type))
        // CSP: no external resources, no inline eval. Our JS is same-origin,
        // same with the CSS and the JSON endpoints.
        .with_header(header(
            "Content-Security-Policy",
            "default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self';",
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
    if !parsed.agent_type.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        respond_json(
            request,
            400,
            &serde_json::json!({ "error": "invalid agent_type" }),
        );
        return;
    }
    if let Some(ref pid) = parsed.pod_id {
        if !pid.chars().all(|c| c.is_ascii_digit()) {
            respond_json(request, 400, &serde_json::json!({ "error": "invalid pod_id" }));
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
        if pb.is_file() { return pb; }
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let candidates: Vec<PathBuf> = vec![
        PathBuf::from(&home).join("bin/tytus"),
        PathBuf::from("/usr/local/bin/tytus"),
        PathBuf::from("/opt/homebrew/bin/tytus"),
        PathBuf::from(&home).join(".cargo/bin/tytus"),
    ];
    for c in &candidates {
        if c.is_file() { return c.clone(); }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let sibling = dir.join("tytus");
            if sibling.is_file() { return sibling; }
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
                push_event(&job, JobEvent::Fail {
                    message: format!(
                        "failed to launch `tytus` at {}: {}. \
                         If the CLI is installed somewhere else, set TYTUS_BIN \
                         in the tray's environment.",
                        bin.display(), e,
                    ),
                });
                return;
            }
        };

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
                    if n == 0 { break; }
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
                    if n == 0 { break; }
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

        // Join the readers so we can inspect stdout for the final JSON.
        let stdout_captured = stdout_thread.and_then(|h| h.join().ok()).unwrap_or_default();
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
                push_event(&job, JobEvent::Done { payload });
            }
            Ok(s) => push_event(&job, JobEvent::Fail {
                message: format!("tytus exited with status {}", s),
            }),
            Err(e) => push_event(&job, JobEvent::Fail {
                message: format!("wait failed: {}", e),
            }),
        }
    });
}

fn push_event(job: &Arc<Mutex<Job>>, ev: JobEvent) {
    let mut j = job.lock().unwrap();
    let terminal = matches!(
        ev,
        JobEvent::Done { .. } | JobEvent::Fail { .. } | JobEvent::Exit { .. }
    );
    j.events.push(ev);
    if terminal { j.finished = true; }
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
    // tiny_http doesn't expose a connection-upgrade primitive; instead
    // we return a Response whose body is a blocking `Read` that we drip-
    // feed from a background thread. The browser sees the event-stream
    // content type and treats it as SSE.
    sse_response(request, job);
}

fn sse_response(request: Request, job: Arc<Mutex<Job>>) {
    // Strategy: spawn a thread that reads events from the job, serializes
    // them to SSE frames, and writes them into a pipe whose read half we
    // hand to tiny_http as the response body. The response header sends
    // "Content-Type: text/event-stream" and no Content-Length so the
    // browser keeps the connection open until we close the pipe.

    let (rx, tx) = pipe();

    thread::spawn(move || {
        let mut cursor = 0usize;
        let mut tx = tx;
        loop {
            let (events_snapshot, finished) = {
                let j = job.lock().unwrap();
                (j.events.len(), j.finished)
            };
            while cursor < events_snapshot {
                let frame = {
                    let j = job.lock().unwrap();
                    match &j.events[cursor] {
                        JobEvent::Log(line) => format!(
                            "event: log\ndata: {}\n\n",
                            line.replace('\n', "\\n"),
                        ),
                        JobEvent::Done { payload } => format!(
                            "event: done\ndata: {}\n\n",
                            payload.replace('\n', " "),
                        ),
                        JobEvent::Fail { message } => format!(
                            "event: fail\ndata: {}\n\n",
                            message.replace('\n', " "),
                        ),
                        JobEvent::Exit { code } => format!(
                            "event: exit\ndata: {{\"code\":{}}}\n\n",
                            code,
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
        .with_data(rx, None);
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
    let reader = PipeReader { state: shared.clone() };
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
    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
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
    /// Used by the Tower page header to choose between "Disconnect" and
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
    /// for the OpenClaw web UI. None for agents with no browser UI (Hermes
    /// dashboard has its own flow) or when tokens are missing.
    ui_url: Option<String>,
    /// Stable per-user API key — same across every pod.
    user_key: String,
}

#[derive(Serialize, Clone)]
struct IncludedSlot {
    pod_id: String,
    kind: &'static str,        // "ail" for now; future types can reuse
    endpoint: String,          // stable_ai_endpoint (e.g. http://10.42.42.1:18080)
    user_key: String,          // stable_user_key (sk-tytus-user-…)
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

fn handle_state(request: Request, registry: &Registry) {
    let snap = compute_state_snapshot();
    let active = registry.active_pods();
    // Merge in active_jobs_per_pod (Phase B running-session badge).
    // serialize snap then patch — avoids changing StateSnapshot's
    // schema and breaking the long list of #[derive(Serialize)] fields
    // it carries today.
    let mut value = match serde_json::to_value(&snap) {
        Ok(v) => v,
        Err(_) => { respond_json(request, 200, &snap); return; }
    };
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "active_jobs_per_pod".to_string(),
            serde_json::to_value(&active).unwrap_or(serde_json::Value::Null),
        );
    }
    respond_json(request, 200, &value);
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
        connected: false, logged_in: false,
        tunnel_active: false,
        tier: String::new(),
        units_limit: 0, units_used: 0,
        agents: vec![], included: vec![],
        email: String::new(),
        uptime_secs: 0,
        keychain_healthy: true,
        last_refresh_error: None,
        daemon_running: false,
        daemon_pid: 0,
        app_bundle_installed: crate::check_app_bundle_installed(),
        forwarders: vec![],
    };

    let state_path = state_json_path();
    let raw = match state_path.and_then(|p| std::fs::read_to_string(p).ok()) {
        Some(s) => s,
        None => return empty(),
    };
    let parsed: serde_json::Value =
        match serde_json::from_str(&raw) { Ok(v) => v, Err(_) => return empty() };

    let tier = parsed.get("tier")
        .and_then(|v| v.as_str()).unwrap_or("").to_string();
    // state.json uses `tokens.expires_at` presence as the logged-in proxy;
    // for the wizard we treat "has a tier + at least one pod" as logged in.
    let logged_in = !tier.is_empty()
        || parsed.get("pods").and_then(|v| v.as_array())
            .map(|a| !a.is_empty()).unwrap_or(false);

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
    let shared_base: Option<String> = parsed
        .get("pods").and_then(|v| v.as_array())
        .and_then(|arr| arr.iter().find_map(|p| {
            p.get("edge_public_url")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        }));

    if let Some(pods) = parsed.get("pods").and_then(|v| v.as_array()) {
        for p in pods {
            let agent_type = p.get("agent_type")
                .and_then(|v| v.as_str()).unwrap_or("none").to_string();
            let pod_id = p.get("pod_id")
                .and_then(|v| v.as_str()).unwrap_or("").to_string();
            let edge_public_url: Option<String> = p.get("edge_public_url")
                .and_then(|v| v.as_str()).filter(|s| !s.is_empty())
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
            let stored_token = p.get("gateway_token")
                .and_then(|v| v.as_str()).filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            let derived_token = p.get("pod_api_key")
                .and_then(|v| v.as_str()).filter(|s| !s.is_empty())
                .map(|k| derive_gateway_token(k, &pod_id));
            let gateway_token: Option<String> = stored_token.or(derived_token);
            let user_key = p.get("stable_user_key")
                .and_then(|v| v.as_str()).unwrap_or("").to_string();
            // Prefer the per-pod subdomain URL (sprint 2026-04-23) — each
            // pod is its own browser origin so the OpenClaw SPA's
            // localStorage doesn't collide across pods. Fall back to the
            // legacy composed URL for state entries written before the
            // sprint's allocation-path populated the field.
            let stored_pod_url: Option<String> = p.get("pod_public_url")
                .and_then(|v| v.as_str()).filter(|s| !s.is_empty())
                .map(|s| s.trim_end_matches('/').to_string());
            let public_url = stored_pod_url.or_else(|| {
                edge_public_url.as_ref().map(|base| {
                    format!("{}/p/{}", base.trim_end_matches('/'), pod_id)
                })
            });
            let api_url = public_url.as_ref().map(|u| format!("{}/v1", u));
            let ui_url = match (public_url.as_ref(), gateway_token.as_ref()) {
                (Some(u), Some(t)) => Some(format!("{}/?token={}", u, t)),
                _ => None,
            };

            if agent_type == "none" {
                let endpoint = p.get("stable_ai_endpoint")
                    .and_then(|v| v.as_str()).unwrap_or("").to_string();
                included.push(IncludedSlot {
                    pod_id, kind: "ail",
                    endpoint, user_key,
                    public_url,
                });
                continue;
            }

            let units = agent_units_for(&agent_type);
            used += units;
            agents.push(AgentSlot {
                pod_id, agent_type, units,
                public_url, api_url, ui_url, user_key,
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
        last_refresh_error: daemon_snap.last_refresh_error,
        daemon_running: daemon_snap.daemon_running,
        daemon_pid: daemon_snap.daemon_pid,
        app_bundle_installed: crate::check_app_bundle_installed(),
        forwarders,
    }
}

/// Locate the CLI's state.json. Matches the CLI's `CliState::path()`
/// logic: `$XDG_CONFIG_HOME/tytus/state.json` on Linux, `~/Library/
/// Application Support/tytus/state.json` on macOS. If the config dir
/// is not resolvable we fall back to `~/.config/tytus/state.json` so
/// callers don't have to branch on platform.
fn state_json_path() -> Option<std::path::PathBuf> {
    if let Some(dir) = dirs::config_dir() {
        return Some(dir.join("tytus").join("state.json"));
    }
    std::env::var_os("HOME").map(|h| {
        std::path::PathBuf::from(h).join(".config").join("tytus").join("state.json")
    })
}

// ── /api/open-external ────────────────────────────────────────

fn handle_open_external(request: Request, query: &str) {
    // Whitelist specific actions. We never accept a raw URL from the
    // client — that's what phishing lives in.
    let target = query.split('&').find_map(|kv| {
        kv.strip_prefix("target=").map(|v| v.to_string())
    }).unwrap_or_default();

    match target.as_str() {
        "health-test" => {
            // Open Terminal running `tytus test`. Reuse the existing tray
            // helper by shelling out to `open` on macOS.
            #[cfg(target_os = "macos")]
            {
                let script =
                    "tell application \"Terminal\" to do script \"tytus test\"";
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
            let channel = query.split('&').find_map(|kv| {
                kv.strip_prefix("channel=").map(|v| v.to_string())
            }).unwrap_or_default();
            let pod = query.split('&').find_map(|kv| {
                kv.strip_prefix("pod=").map(|v| v.to_string())
            }).unwrap_or_default();
            let valid_channels = ["telegram", "discord", "slack", "line"];
            if !valid_channels.contains(&channel.as_str()) {
                respond_json(request, 400, &serde_json::json!({"error":"invalid channel"}));
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
        _ => respond_json(request, 400, &serde_json::json!({ "error": "unknown target" })),
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
    let raw = query.split('&').find_map(|kv| {
        kv.strip_prefix("pod=").map(|v| v.to_string())
    })?;
    if raw.chars().all(|c| c.is_ascii_digit()) && !raw.is_empty() {
        Some(raw)
    } else { None }
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
        None => { respond_json(request, 400, &serde_json::json!({"error":"invalid pod"})); return; }
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
    let url = snap.agents.iter().find(|a| a.pod_id == pod_id)
        .and_then(|a| a.ui_url.clone().or_else(|| a.public_url.clone()));
    match url {
        Some(u) => {
            #[cfg(target_os = "macos")]
            { let _ = Command::new("open").arg(&u).spawn(); }
            #[cfg(target_os = "linux")]
            { let _ = Command::new("xdg-open").arg(&u).spawn(); }
            respond_json(request, 200, &serde_json::json!({"ok": true, "url": u}));
        }
        None => {
            respond_json(request, 503, &serde_json::json!({
                "error":"no public URL yet — try again after the pod finishes provisioning"
            }));
        }
    }
}

fn handle_pod_restart(request: Request, query: &str) {
    let pod_id = match parse_pod_id(query) {
        Some(p) => p,
        None => { respond_json(request, 400, &serde_json::json!({"error":"invalid pod"})); return; }
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
        .stdout(Stdio::null()).stderr(Stdio::null()).stdin(Stdio::null())
        .spawn();
    match spawned {
        Ok(_) => respond_json(request, 202, &serde_json::json!({"ok":true, "pod":pod_id})),
        Err(e) => respond_json(request, 500, &serde_json::json!({
            "error": format!("failed to spawn: {}", e)
        })),
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
        None => { respond_json(request, 400, &serde_json::json!({"error":"invalid pod"})); return; }
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
        .stdout(Stdio::null()).stderr(Stdio::null()).stdin(Stdio::null())
        .spawn();
    match spawned {
        Ok(_) => respond_json(request, 202, &serde_json::json!({"ok":true, "pod":pod_id})),
        Err(e) => respond_json(request, 500, &serde_json::json!({
            "error": format!("failed to spawn: {}", e)
        })),
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
/// Per-pod actions ONLY. Global commands (`tytus doctor`, `tytus test`)
/// are intentionally absent — they aren't pod-scoped (CLI doesn't accept
/// `--pod` for them), so exposing them under `/api/pod/<NN>/run-streamed`
/// would be misleading. They're available via dedicated endpoints
/// (`POST /api/doctor`, `POST /api/test`) and surfaced on the Tower
/// header / Troubleshoot section, not on per-pod subpages.
fn pod_action_argv(action: &str, pod_id: &str) -> Option<Vec<String>> {
    let v = |args: &[&str]| Some(args.iter().map(|s| s.to_string()).collect::<Vec<_>>());
    match action {
        "restart"         => v(&["restart", "--pod", pod_id]),
        "revoke"          => v(&["revoke", pod_id]),
        "uninstall"       => v(&["agent", "uninstall", pod_id]),
        "stop-forwarder"  => v(&["ui", "--stop", "--pod", pod_id]),
        "channels-list"   => v(&["channels", "list", "--pod", pod_id]),
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
                request, 400,
                &serde_json::json!({"error": format!("unknown action {}", body.action)}),
            );
            return;
        }
    };

    let (job_id, job) = match registry.create_pod(&pod_id) {
        Ok(pair) => pair,
        Err(p) => {
            respond_json(
                request, 409,
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
        for a in &argv { cmd.arg(a); }
        cmd.stdin(Stdio::null())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped())
           .env("TYTUS_HEADLESS", "1");

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                push_event(&job, JobEvent::Fail {
                    message: format!(
                        "failed to launch `tytus` at {}: {}. \
                         Set TYTUS_BIN if the CLI lives elsewhere.",
                        bin.display(), e,
                    ),
                });
                return;
            }
        };

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // Stream stdout.
        let job_so = job.clone();
        let stdout_t = stdout.map(|mut h| thread::spawn(move || {
            let mut buf = [0u8; 4096];
            let mut carry = String::new();
            while let Ok(n) = h.read(&mut buf) {
                if n == 0 { break; }
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
        }));

        // Stream stderr (merged into the same log channel; tytus uses
        // stderr for progress/status messages).
        let job_se = job.clone();
        let stderr_t = stderr.map(|mut h| thread::spawn(move || {
            let mut buf = [0u8; 4096];
            let mut carry = String::new();
            while let Ok(n) = h.read(&mut buf) {
                if n == 0 { break; }
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
        }));

        let status = child.wait();
        let _ = stdout_t.and_then(|h| h.join().ok());
        let _ = stderr_t.and_then(|h| h.join().ok());

        let code = match status {
            Ok(s) => s.code().unwrap_or(-1),
            Err(_) => -1,
        };
        push_event(&job, JobEvent::Exit { code });
    });
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
fn handle_pod_ready(request: Request, query: &str) {
    let pod_id = match parse_pod_id(query) {
        Some(p) => p,
        None => { respond_json(request, 400, &serde_json::json!({"error":"invalid pod"})); return; }
    };
    let snap = compute_state_snapshot();
    let agent = snap.agents.iter().find(|a| a.pod_id == pod_id).cloned();
    let api = match agent.as_ref().and_then(|a| a.api_url.clone()) {
        Some(u) => u,
        None => {
            // No public URL derivable yet (slug not in state). Report
            // not-ready but not an error — wizard keeps polling.
            respond_json(request, 200, &serde_json::json!({
                "ready": false, "reason": "public URL not ready"
            }));
            return;
        }
    };
    let probe_url = format!("{}/models", api.trim_end_matches('/'));
    let user_key = agent.as_ref().map(|a| a.user_key.clone()).unwrap_or_default();
    // Probe WITH the stable user key as Bearer — otherwise the edge
    // plugin 401s our unauthenticated probe and we can't distinguish
    // "edge up, pod starting" from "edge up, pod ready". Using the
    // key we'd actually hand the user means a 200 proves the ENTIRE
    // path (edge → user-key map → pod gateway) is live.
    let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
        Ok(r) => r,
        Err(e) => {
            respond_json(request, 500, &serde_json::json!({"error": e.to_string()}));
            return;
        }
    };
    let http = atomek_core::HttpClient::new();
    let result = rt.block_on(async {
        http.get(&probe_url)
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", user_key))
            .timeout(std::time::Duration::from_secs(4))
            .send().await
    });
    let (ready, status, reason) = match result {
        Ok(resp) => {
            let s = resp.status().as_u16();
            // 200 is the only real "ready". 401/403 now mean the edge
            // is rejecting even our authenticated probe — either the
            // user-key map hasn't propagated, or the pod is still
            // starting. 404 = edge route missing. 502/503 = upstream
            // unhealthy. Any non-200 keeps the wizard waiting.
            let ok = s == 200;
            let r = match s {
                200 => "gateway answering with 200".into(),
                401 | 403 => "edge auth not yet propagated".into(),
                404 => "edge route not yet published".into(),
                502 | 503 | 504 => "gateway upstream not yet healthy".into(),
                other => format!("http {}", other),
            };
            (ok, s, r)
        }
        Err(e) => (false, 0u16, format!("probe error: {}", e)),
    };
    respond_json(request, 200, &serde_json::json!({
        "ready": ready, "status": status, "reason": reason, "probe_url": probe_url,
    }));
}

fn handle_pod_uninstall(request: Request, query: &str) {
    let pod_id = match parse_pod_id(query) {
        Some(p) => p,
        None => { respond_json(request, 400, &serde_json::json!({"error":"invalid pod"})); return; }
    };
    // Only agent pods (nemoclaw, hermes) can be uninstalled. AIL-included
    // pods have no agent to remove — `tytus agent uninstall <pod>` on a
    // default pod is a no-op + confusing error.
    let snap = compute_state_snapshot();
    if !snap.agents.iter().any(|a| a.pod_id == pod_id) {
        respond_json(request, 400, &serde_json::json!({
            "error":"pod has no agent to uninstall"
        }));
        return;
    }
    let bin = resolve_tytus_bin();
    let spawned = Command::new(&bin)
        .args(["agent", "uninstall", &pod_id, "--json"])
        .env("TYTUS_HEADLESS", "1")
        .stdout(Stdio::null()).stderr(Stdio::null()).stdin(Stdio::null())
        .spawn();
    match spawned {
        Ok(_) => respond_json(request, 202, &serde_json::json!({"ok":true, "pod":pod_id})),
        Err(e) => respond_json(request, 500, &serde_json::json!({
            "error": format!("failed to spawn: {}", e)
        })),
    }
}

// ── Tower control-surface handlers (Wave 1) ───────────────────

fn handle_disconnect(request: Request) {
    // Detached subprocess — disconnect is fast (<1s, no sudo) because it
    // just reads the tunnel pidfile and SIGTERMs. Client polls /api/state
    // to see the tunnel_active flag flip.
    let bin = resolve_tytus_bin();
    let spawned = Command::new(&bin)
        .arg("disconnect")
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
        .spawn();
    match spawned {
        Ok(_) => respond_json(request, 202, &serde_json::json!({"ok": true})),
        Err(e) => respond_json(request, 500, &serde_json::json!({
            "error": format!("failed to spawn tytus disconnect: {}", e)
        })),
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
    respond_json(request, 200, &serde_json::json!({
        "autostart_tunnel": autostart_tunnel_installed(),
        "autostart_tray": autostart_tray_installed(),
    }));
}

#[derive(serde::Deserialize)]
struct ToggleBody { enabled: bool }

fn handle_autostart_tunnel(mut request: Request) {
    let enabled = match parse_toggle_body(&mut request) {
        Ok(e) => e,
        Err(resp_sent) => { let _ = resp_sent; return; }
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
        Err(resp_sent) => { let _ = resp_sent; return; }
    };
    // `tytus tray install` creates /Applications/Tytus.app + the tray
    // LaunchAgent. 5-10s on first run because icons are generated via
    // sips + iconutil. User-scope; no sudo.
    let sub = if enabled { "install" } else { "uninstall" };
    run_tytus_inline(request, &["tray", sub]);
}

fn handle_logout(request: Request) {
    // Destructive: revokes all pods + clears keychain. JS confirms
    // before POSTing. Spawned in a Terminal because Sentinel logout
    // prints user-facing output and because logout-through-CLI is the
    // canonical path. Returns 202 immediately.
    crate::open_in_terminal_simple(
        "tytus logout; echo; echo 'Press Enter to close…'; read _"
    );
    respond_json(request, 202, &serde_json::json!({"ok": true}));
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
            respond_json(request, if o.status.success() { 200 } else { 500 }, &serde_json::json!({
                "ok": o.status.success(),
                "exit_code": o.status.code().unwrap_or(-1),
                "stdout": stdout,
                "stderr": stderr,
            }));
        }
        Err(e) => respond_json(request, 500, &serde_json::json!({
            "error": format!("failed to spawn tytus {}: {}", args.join(" "), e)
        })),
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
fn autostart_tunnel_installed() -> bool { false }

#[cfg(target_os = "macos")]
fn autostart_tray_installed() -> bool {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(&home)
        .join("Library/LaunchAgents/com.traylinx.tytus.tray.plist")
        .exists()
}
#[cfg(not(target_os = "macos"))]
fn autostart_tray_installed() -> bool { false }

// ── Tower Wave 2: Troubleshoot handlers ────────────────────────

fn handle_doctor(request: Request, registry: &Registry) {
    // Streamed: `tytus doctor` runs DNS, auth, tunnel, pod, gateway,
    // MCP checks back-to-back. Returns { job_id }; output streams via
    // /api/jobs/<id>/stream. Same rationale as handle_test — see that
    // doc block + the wizard::flush() helper in the CLI.
    let (job_id, job) = registry.create();
    spawn_pod_action(job, vec!["doctor".to_string()]);
    respond_json(request, 202, &serde_json::json!({"job_id": job_id}));
}

enum DaemonAction { Start, Stop, Restart }

fn handle_daemon_lifecycle(request: Request, action: DaemonAction) {
    let bin = resolve_tytus_bin();
    match action {
        DaemonAction::Stop => {
            // Blocking — `tytus daemon stop` is fast (<1s). Surface result.
            let out = Command::new(&bin).args(["daemon", "stop"])
                .env("TYTUS_HEADLESS", "1")
                .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
                .output();
            match out {
                Ok(o) => respond_json(request, if o.status.success() { 200 } else { 500 }, &serde_json::json!({
                    "ok": o.status.success(),
                    "stdout": String::from_utf8_lossy(&o.stdout).to_string(),
                    "stderr": String::from_utf8_lossy(&o.stderr).to_string(),
                })),
                Err(e) => respond_json(request, 500, &serde_json::json!({
                    "error": format!("failed: {}", e)
                })),
            }
        }
        DaemonAction::Start => {
            // Detached — `tytus daemon run` blocks in the foreground until
            // the process exits, so we must spawn it without piping stdio
            // back to the HTTP thread. launchd normally handles this via
            // KeepAlive; manual start is a troubleshooting path.
            let res = Command::new(&bin).args(["daemon", "run"])
                .env("TYTUS_HEADLESS", "1")
                .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
                .spawn();
            match res {
                Ok(_) => respond_json(request, 202, &serde_json::json!({"ok": true})),
                Err(e) => respond_json(request, 500, &serde_json::json!({
                    "error": format!("failed to spawn: {}", e)
                })),
            }
        }
        DaemonAction::Restart => {
            // Stop blocking then spawn new daemon detached. If stop fails
            // (maybe daemon was already down), we still attempt start —
            // the user's intent is "make it running now".
            let _ = Command::new(&bin).args(["daemon", "stop"])
                .env("TYTUS_HEADLESS", "1")
                .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
                .status();
            std::thread::sleep(std::time::Duration::from_millis(500));
            let res = Command::new(&bin).args(["daemon", "run"])
                .env("TYTUS_HEADLESS", "1")
                .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
                .spawn();
            match res {
                Ok(_) => respond_json(request, 202, &serde_json::json!({"ok": true})),
                Err(e) => respond_json(request, 500, &serde_json::json!({
                    "error": format!("restart failed: {}", e)
                })),
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
    respond_json(request, 200, &serde_json::json!({
        "running": running,
        "pid": pid,
    }));
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
                "name" => name = match v { "daemon" | "startup" => v, _ => "daemon" },
                "offset" => { offset = v.parse().unwrap_or(0); }
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
            respond_json(request, 200, &serde_json::json!({
                "name": name,
                "offset": 0u64,
                "size": 0u64,
                "chunk": "",
                "missing": true,
            }));
            return;
        }
    };
    let size = meta.len();
    let (read_from, truncated) = if offset > size { (0, true) } else { (offset, false) };
    let available = size.saturating_sub(read_from);
    let take = available.min(MAX_CHUNK);

    let chunk = if take == 0 {
        String::new()
    } else {
        use std::io::{Read as _, Seek, SeekFrom};
        let mut f = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(_) => { respond_json(request, 500, &serde_json::json!({"error":"open failed"})); return; }
        };
        if f.seek(SeekFrom::Start(read_from)).is_err() {
            respond_json(request, 500, &serde_json::json!({"error":"seek failed"})); return;
        }
        let mut buf = vec![0u8; take as usize];
        let n = f.read(&mut buf).unwrap_or(0);
        buf.truncate(n);
        String::from_utf8_lossy(&buf).to_string()
    };
    let new_offset = read_from + (chunk.as_bytes().len() as u64);
    respond_json(request, 200, &serde_json::json!({
        "name": name,
        "offset": new_offset,
        "size": size,
        "chunk": chunk,
        "truncated": truncated,
        "missing": false,
    }));
}

// ── Tower Wave 3b: launch in editor ────────────────────────────

fn handle_launchers_list(request: Request) {
    // Mirror the tray's "Open in ▸" detection so the page shows the
    // same set of editors. Each entry is a thin metadata record; the
    // actual launch happens via POST /api/launch with the binary name.
    let clis = crate::launcher::detect_installed_clis();
    let list: Vec<serde_json::Value> = clis.iter().map(|c| {
        serde_json::json!({
            "binary": c.binary,
            "name": c.name,
        })
    }).collect();
    respond_json(request, 200, &serde_json::json!({
        "editors": list,
        // Plain terminal is always "available"; tray lists it unconditionally.
        "terminal_available": true,
    }));
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
    let agent = pod_id_override.as_ref()
        .and_then(|id| snap.agents.iter().find(|a| &a.pod_id == id))
        .or_else(|| snap.agents.iter().find(|a| a.api_url.is_some()));
    let included_first = snap.included.iter().find(|i| i.public_url.is_some());
    let (url, api_key) = if let Some(a) = agent {
        let url = a.api_url.clone().unwrap_or_else(|| {
            // Fallback to AIL private endpoint when the pod has no public
            // URL yet (mid-provisioning).
            included_first.map(|i| format!("{}/v1", i.endpoint))
                .unwrap_or_else(|| "http://10.42.42.1:18080/v1".into())
        });
        (url, a.user_key.clone())
    } else if let Some(inc) = snap.included.first() {
        let url = inc.public_url.as_ref()
            .map(|u| format!("{}/v1", u))
            .unwrap_or_else(|| format!("{}/v1", inc.endpoint));
        (url, inc.user_key.clone())
    } else {
        respond_json(request, 400, &serde_json::json!({
            "error": "no pods available — run tytus connect first"
        }));
        return;
    };

    // Launcher wants the base URL (no /v1 — it appends /v1 itself in
    // the shell_cmd it builds).
    let base = url.trim_end_matches("/v1").trim_end_matches('/').to_string();
    let conn = crate::launcher::PodConnection {
        ai_gateway: base,
        api_key,
        model: "ail-compound".into(),
    };

    // Special case: "terminal" opens a plain shell with env exports.
    if editor == "terminal" {
        crate::launcher::launch_terminal(&conn);
        respond_json(request, 202, &serde_json::json!({"ok": true, "editor": "terminal"}));
        return;
    }

    // Look up the chosen editor by binary name; refuse unknown binaries
    // so a mischievous client can't smuggle arbitrary shell into the
    // templated command.
    let clis = crate::launcher::detect_installed_clis();
    let cli = match clis.iter().find(|c| c.binary == editor) {
        Some(c) => c.clone(),
        None => {
            respond_json(request, 400, &serde_json::json!({
                "error": format!("editor not detected or not whitelisted: {}", editor)
            }));
            return;
        }
    };
    crate::launcher::launch_in_terminal(&cli, &conn);
    respond_json(request, 202, &serde_json::json!({"ok": true, "editor": cli.binary}));
}

// ── Tower Wave 3c: per-pod channels ────────────────────────────

/// `^[a-z][a-z0-9_-]{1,30}$` — matches known channel names (telegram,
/// discord, slack, line) plus leaves room for future additions. Used
/// as a whitelist before templating the name into a shell command.
fn valid_channel_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 31
        && s.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false)
        && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

fn valid_pod_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 4 && s.chars().all(|c| c.is_ascii_digit())
}

fn parse_channel_query(query: &str) -> (Option<String>, Option<String>) {
    let mut pod = None;
    let mut name = None;
    for pair in query.split('&').filter(|s| !s.is_empty()) {
        if let Some((k, v)) = pair.split_once('=') {
            match k {
                "pod" => { if valid_pod_id(v) { pod = Some(v.to_string()); } }
                "name" => { if valid_channel_name(v) { name = Some(v.to_string()); } }
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
            respond_json(request, 400, &serde_json::json!({ "error": "missing or invalid pod id" }));
            return;
        }
    };
    let configured: Vec<serde_json::Value> = crate::read_channels_for_pod(&pod_id)
        .into_iter()
        .map(|(name, count)| serde_json::json!({
            "name": name,
            "label": crate::channel_label(&name),
            "secret_count": count,
        }))
        .collect();
    // Available = everything in CHANNEL_MENU_ENTRIES that isn't already
    // configured on this pod. Matches what the tray's channels submenu
    // shows when building the "Add X…" list.
    let configured_names: std::collections::HashSet<String> = configured.iter()
        .filter_map(|e| e.get("name").and_then(|n| n.as_str()).map(String::from))
        .collect();
    let available: Vec<serde_json::Value> = crate::CHANNEL_MENU_ENTRIES.iter()
        .filter(|(n, _)| !configured_names.contains(*n))
        .map(|(n, l)| serde_json::json!({ "name": n, "label": l }))
        .collect();
    respond_json(request, 200, &serde_json::json!({
        "pod_id": pod_id,
        "configured": configured,
        "available": available,
    }));
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
    struct Body { pod: String, channel: String, token: String }
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
        respond_json(request, 400, &serde_json::json!({"error": "invalid channel"}));
        return;
    }
    if body.token.is_empty() || body.token.len() > 4096 {
        respond_json(request, 400, &serde_json::json!({"error": "invalid token"}));
        return;
    }
    // run_tytus_inline blocks for ~10-15s while the agent redeploys.
    // The browser modal shows "Adding…" during that window. The token
    // is the last argv element; no shell, no quoting, no log surface.
    run_tytus_inline(request, &[
        "channels", "add",
        "--pod", &body.pod,
        "--type", &body.channel,
        "--token", &body.token,
    ]);
}

fn handle_channels_remove(request: Request, query: &str) {
    let (pod, name) = parse_channel_query(query);
    let (Some(pod_id), Some(channel)) = (pod, name) else {
        respond_json(request, 400, &serde_json::json!({ "error": "missing pod or name" }));
        return;
    };
    // Client confirms first; server runs the subprocess inline. The
    // credential wipe + redeploy takes ~10s, so the HTTP thread blocks
    // for that window — acceptable since each click is its own thread
    // and the page shows a spinner. Phase B switches this to streamed
    // SSE for richer feedback.
    run_tytus_inline(request, &[
        "channels", "remove",
        "--pod", &pod_id,
        "--type", &channel,
    ]);
}

fn handle_channels_catalog(request: Request) {
    // Read-only listing of available channel types. Runs synchronously
    // (subprocess returns within ~200ms) and returns the captured stdout
    // so the page can render it inline without spawning Terminal.app.
    run_tytus_inline(request, &["channels", "catalog"]);
}

// ── Tower Wave 4: sync gaps ────────────────────────────────────

fn handle_pod_stop_forwarder(request: Request, query: &str) {
    // Mirrors the tray's `pod_NN_stop_forwarder` — runs `tytus ui --stop
    // --pod NN` so the CLI's pidfile cleanup path stays the source of
    // truth. Detached; reply 202 and let the client refresh.
    let mut pod_id: Option<String> = None;
    for pair in query.split('&').filter(|s| !s.is_empty()) {
        if let Some((k, v)) = pair.split_once('=') {
            if k == "pod" && valid_pod_id(v) { pod_id = Some(v.to_string()); }
        }
    }
    let Some(pod) = pod_id else {
        respond_json(request, 400, &serde_json::json!({ "error": "missing or invalid pod id" }));
        return;
    };
    let bin = resolve_tytus_bin();
    let spawned = Command::new(&bin)
        .args(["ui", "--stop", "--pod", &pod])
        .env("TYTUS_HEADLESS", "1")
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
        .spawn();
    match spawned {
        Ok(_) => respond_json(request, 202, &serde_json::json!({"ok": true, "pod": pod})),
        Err(e) => respond_json(request, 500, &serde_json::json!({
            "error": format!("failed to spawn: {}", e)
        })),
    }
}

fn handle_configure(request: Request) {
    // `tytus configure` is an interactive overlay editor — needs a TTY
    // for the multi-step prompt flow. Spawn a Terminal, matching the
    // tray's Settings ▸ Configure Agent… item.
    crate::open_in_terminal_simple(
        "tytus configure; echo; echo 'Press Enter to close…'; read _"
    );
    respond_json(request, 202, &serde_json::json!({ "ok": true }));
}

// ── Helpers ────────────────────────────────────────────────────

fn respond_json<T: Serialize>(request: Request, status: u16, body: &T) {
    let json = serde_json::to_string(body).unwrap_or_else(|_| "{}".into());
    let resp = Response::from_string(json)
        .with_status_code(StatusCode(status))
        .with_header(header("Content-Type", "application/json"));
    let _ = request.respond(resp);
}

#[cfg(test)]
mod tests {
    use super::*;

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

        // Global commands are intentionally not pod-scoped — they
        // belong on /api/doctor and /api/test, not here.
        assert!(pod_action_argv("doctor", "02").is_none());
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
}

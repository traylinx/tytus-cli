//! Local HTTP server for the tray's agent-install browser wizard.
//!
//! Binds to `127.0.0.1:<random>` at tray startup, serves embedded HTML/CSS/JS,
//! and exposes a tiny local API so the static JS can (a) list the agent
//! catalog, (b) kick off a `tytus agent install` subprocess, and (c) stream
//! its stdout back via server-sent events.
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
//!   `<tmp>/tytus/tray-web.port` so `open_wizard()` can read it.
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
const INSTALL_HTML: &[u8] = include_bytes!("../web/install.html");
const INSTALL_CSS: &[u8] = include_bytes!("../web/assets/install.css");
const INSTALL_JS: &[u8] = include_bytes!("../web/assets/install.js");

// ── Job registry ──────────────────────────────────────────────
//
// Each install job is a live subprocess (tytus agent install <type>)
// plus a channel of streaming events for in-flight SSE consumers. Jobs
// are indexed by a random-ish id so the browser can reconnect to a
// running job if the EventSource hiccups.

enum JobEvent {
    Log(String),
    Done { payload: String },
    Fail { message: String },
}

struct Job {
    events: Vec<JobEvent>,
    finished: bool,
}

impl Job {
    fn new() -> Self {
        Job { events: Vec::new(), finished: false }
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

    fn create(&self) -> (String, Arc<Mutex<Job>>) {
        let id = random_job_id();
        let job = Arc::new(Mutex::new(Job::new()));
        self.inner.lock().unwrap().insert(id.clone(), job.clone());
        (id, job)
    }

    fn get(&self, id: &str) -> Option<Arc<Mutex<Job>>> {
        self.inner.lock().unwrap().get(id).cloned()
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
/// isn't available). Caller stores the returned port for `open_wizard()`.
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
    // `open_wizard`) can read it without a lookup.
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

pub fn open_wizard() {
    let port = match current_port() {
        Some(p) => p,
        None => {
            eprintln!("[tray-web] no port recorded — is the server running?");
            return;
        }
    };
    let url = format!("http://127.0.0.1:{}/install", port);
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
        (Method::Get, "/install") | (Method::Get, "/") => {
            serve_bytes(request, INSTALL_HTML, "text/html; charset=utf-8");
        }
        (Method::Get, "/assets/install.css") => {
            serve_bytes(request, INSTALL_CSS, "text/css; charset=utf-8");
        }
        (Method::Get, "/assets/install.js") => {
            serve_bytes(request, INSTALL_JS, "application/javascript; charset=utf-8");
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
            handle_state(request);
        }
        (Method::Post, "/api/open-external") => {
            handle_open_external(request, &query);
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

fn spawn_install(job: Arc<Mutex<Job>>, agent_type: String, pod_id: Option<String>) {
    thread::spawn(move || {
        let mut cmd = Command::new("tytus");
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
                push_event(&job, JobEvent::Fail { message: format!("spawn failed: {}", e) });
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
    let terminal = matches!(ev, JobEvent::Done { .. } | JobEvent::Fail { .. });
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
}

fn handle_state(request: Request) {
    // For v1 we return a minimal shape the JS uses to decide UI micro-bits.
    // Richer data (live pods) is available via `tytus status --json` if
    // we need it later.
    let snap = StateSnapshot { connected: true };
    respond_json(request, 200, &snap);
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
        _ => respond_json(request, 400, &serde_json::json!({ "error": "unknown target" })),
    }
}

// ── Helpers ────────────────────────────────────────────────────

fn respond_json<T: Serialize>(request: Request, status: u16, body: &T) {
    let json = serde_json::to_string(body).unwrap_or_else(|_| "{}".into());
    let resp = Response::from_string(json)
        .with_status_code(StatusCode(status))
        .with_header(header("Content-Type", "application/json"));
    let _ = request.respond(resp);
}

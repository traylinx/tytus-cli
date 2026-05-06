//! Windows daemon substrate.
//!
//! The rich Unix-socket command server is not migrated yet, but Windows is no
//! longer a compile-only stub: `daemon run` writes the platform control file,
//! starts the localhost HTTP control plane, and reports minimal daemon status.

use atomek_core::platform::{ipc, paths, process};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tiny_http::{Header, Method, Response as TinyResponse, Server, StatusCode};

#[derive(Debug, Deserialize)]
struct Request {
    pub cmd: String,
    #[serde(default)]
    pub args: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub code: Option<String>,
}

pub fn socket_path() -> PathBuf {
    paths::runtime_dir().join("daemon.sock")
}

pub fn pid_path() -> PathBuf {
    paths::daemon_pid_file()
}

pub fn pid_file_pid() -> Option<i32> {
    std::fs::read_to_string(pid_path())
        .ok()?
        .trim()
        .parse()
        .ok()
}

pub fn process_alive(pid: i32) -> bool {
    pid > 1 && process::process_exists(pid as u32)
}

pub fn pid_is_tytus_daemon(_pid: i32) -> bool {
    false
}

pub async fn is_daemon_running() -> bool {
    if let Some(resp) = send_http_command(
        "status",
        serde_json::Value::Null,
        std::time::Duration::from_secs(1),
    )
    .await
    {
        return resp.status == "ok";
    }
    pid_file_pid().is_some_and(process_alive)
}

pub async fn run_daemon() {
    let runtime = paths::runtime_dir();
    if let Err(e) = paths::ensure_private_dir(&runtime) {
        eprintln!("tytus: failed to create runtime dir: {}", e);
        return;
    }

    let server = match Server::http("127.0.0.1:0") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tytus: failed to start HTTP control plane: {}", e);
            return;
        }
    };
    let port = match server.server_addr().to_ip() {
        Some(addr) => addr.port(),
        None => {
            eprintln!("tytus: HTTP control plane did not bind an IP socket");
            return;
        }
    };
    let token = match ipc::generate_control_token() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("tytus: failed to generate control token: {}", e);
            return;
        }
    };
    let pid = std::process::id();
    let pid_file = pid_path();
    let _ = std::fs::write(&pid_file, pid.to_string());
    let control = ipc::ControlFile::new(port, pid);
    if let Err(e) = ipc::write_control_token_file(&control.token_file, &token) {
        eprintln!("tytus: failed to write control token: {}", e);
        return;
    }
    if let Err(e) = ipc::write_control_file(&paths::control_file(), &control) {
        eprintln!("tytus: failed to write control file: {}", e);
        return;
    }

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_thread = shutdown.clone();
    std::thread::spawn(move || {
        for request in server.incoming_requests() {
            handle_request(request, &token, &shutdown_thread);
            if shutdown_thread.load(Ordering::SeqCst) {
                break;
            }
        }
    });

    eprintln!(
        "tytus-daemon Windows HTTP substrate running (pid {}, control {}, port {})",
        pid,
        paths::control_file().display(),
        port
    );
    while !shutdown.load(Ordering::SeqCst) {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    let _ = std::fs::remove_file(pid_path());
    let _ = std::fs::remove_file(paths::control_file());
    let _ = std::fs::remove_file(paths::control_token_file());
}

fn handle_request(mut request: tiny_http::Request, token: &str, shutdown: &AtomicBool) {
    if !request
        .remote_addr()
        .map(|addr| match addr.ip() {
            IpAddr::V4(ip) => ip.is_loopback(),
            IpAddr::V6(ip) => ip.is_loopback(),
        })
        .unwrap_or(false)
    {
        let _ = request.respond(json_response(
            StatusCode(403),
            serde_json::json!({"status":"error","code":"FORBIDDEN","error":"loopback only"}),
        ));
        return;
    }
    if !authorized(&request, token) {
        let _ = request.respond(json_response(
            StatusCode(401),
            serde_json::json!({"status":"error","code":"UNAUTHORIZED","error":"missing or invalid token"}),
        ));
        return;
    }

    let method = request.method().clone();
    let url = request
        .url()
        .split('?')
        .next()
        .unwrap_or(request.url())
        .to_string();
    let resp = match (method, url.as_str()) {
        (Method::Get, "/v1/health") => status_response(),
        (Method::Get, "/v1/status") => status_response(),
        (Method::Post, "/v1/command") => {
            let mut body = String::new();
            if let Err(e) = request.as_reader().read_to_string(&mut body) {
                error_response("READ_ERROR", format!("failed to read body: {}", e))
            } else {
                match serde_json::from_str::<Request>(&body) {
                    Ok(req) if req.cmd == "status" => {
                        let _ = req.args;
                        status_response()
                    }
                    Ok(req) if req.cmd == "shutdown" => {
                        let _ = req.args;
                        shutdown.store(true, Ordering::SeqCst);
                        Response {
                            status: "ok".into(),
                            data: Some(serde_json::json!({"shutting_down": true})),
                            error: None,
                            code: None,
                        }
                    }
                    Ok(req) => error_response(
                        "WINDOWS_DAEMON_COMMAND_PENDING",
                        format!(
                            "Windows daemon command '{}' awaits full Phase 2 RPC migration",
                            req.cmd
                        ),
                    ),
                    Err(e) => error_response("PARSE_ERROR", format!("Invalid JSON: {}", e)),
                }
            }
        }
        _ => error_response("NOT_FOUND", "not found"),
    };
    let status = if resp.status == "ok" {
        StatusCode(200)
    } else {
        StatusCode(400)
    };
    let _ = request.respond(json_response(status, serde_json::to_value(resp).unwrap()));
}

fn status_response() -> Response {
    let pid = std::process::id();
    Response {
        status: "ok".into(),
        data: Some(serde_json::json!({
            "daemon": {
                "pid": pid,
                "status": "running",
                "uptime_secs": 0,
                "last_refresh_secs_ago": null,
                "last_refresh_error": null,
                "keychain_healthy": true,
                "stuck_for_secs": null
            },
            "auth": {
                "logged_in": false,
                "token_valid": false,
                "email": null,
                "tier": null,
                "expires_at_ms": null
            },
            "pods": [],
            "platform": "windows"
        })),
        error: None,
        code: None,
    }
}

fn error_response(code: &str, message: impl Into<String>) -> Response {
    Response {
        status: "error".into(),
        data: None,
        error: Some(message.into()),
        code: Some(code.into()),
    }
}

fn authorized(request: &tiny_http::Request, token: &str) -> bool {
    let expected = format!("Bearer {}", token);
    request
        .headers()
        .iter()
        .any(|h| h.field.equiv("Authorization") && h.value.as_str().trim() == expected)
}

fn json_response(
    status: StatusCode,
    body: serde_json::Value,
) -> TinyResponse<std::io::Cursor<Vec<u8>>> {
    let mut resp = TinyResponse::from_string(body.to_string()).with_status_code(status);
    if let Ok(header) = Header::from_bytes("Content-Type", "application/json") {
        resp.add_header(header);
    }
    resp
}

pub async fn send_command(cmd: &str, args: serde_json::Value) -> Option<Response> {
    send_command_timeout(cmd, args, std::time::Duration::from_secs(2)).await
}

pub async fn send_command_timeout(
    cmd: &str,
    args: serde_json::Value,
    timeout: std::time::Duration,
) -> Option<Response> {
    if let Some(resp) = send_http_command(cmd, args, timeout).await {
        return Some(resp);
    }
    match cmd {
        "status" => {
            let control = ipc::read_control_file(&paths::control_file()).ok();
            let pid = control.as_ref().map(|c| c.pid as i32).or_else(pid_file_pid);
            let running = pid.is_some_and(process_alive);
            Some(Response {
                status: "ok".into(),
                data: Some(serde_json::json!({
                    "daemon": {
                        "pid": pid.unwrap_or_default(),
                        "status": if running { "running" } else { "stopped" },
                        "uptime_secs": 0,
                        "last_refresh_secs_ago": null,
                        "last_refresh_error": null,
                        "keychain_healthy": true,
                        "stuck_for_secs": null
                    },
                    "auth": {
                        "logged_in": false,
                        "token_valid": false,
                        "email": null,
                        "tier": null,
                        "expires_at_ms": null
                    },
                    "pods": [],
                    "platform": "windows",
                    "ipc": "control_file_minimal",
                    "control_file": paths::control_file(),
                })),
                error: None,
                code: None,
            })
        }
        _ => Some(error_response(
            "WINDOWS_DAEMON_HTTP_IPC_PENDING",
            format!(
                "Windows daemon command '{}' awaits Phase 2 HTTP IPC migration",
                cmd
            ),
        )),
    }
}

async fn send_http_command(
    cmd: &str,
    args: serde_json::Value,
    timeout: std::time::Duration,
) -> Option<Response> {
    let control = ipc::read_control_file(&paths::control_file()).ok()?;
    if control.port == 0 || !control.bind.is_loopback() {
        return None;
    }
    let token = ipc::read_control_token_file(&control.token_file).ok()?;
    let url = format!("http://{}:{}/v1/command", control.bind, control.port);
    let client = reqwest::Client::builder().timeout(timeout).build().ok()?;
    let resp = client
        .post(url)
        .bearer_auth(token)
        .json(&serde_json::json!({"cmd": cmd, "args": args}))
        .send()
        .await
        .ok()?;
    resp.json::<Response>().await.ok()
}

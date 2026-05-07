//! Localhost HTTP control plane for the Tytus daemon.
//!
//! Migration contract: keep the Unix socket alive, but also expose the same
//! command protocol over `127.0.0.1:<dynamic>` with a bearer token discovered
//! through `control.json` + `control.token`.

use crate::daemon::{dispatch_command, DaemonCtx, Request, Response as DaemonResponse};
use serde_json::json;
use std::net::IpAddr;
use std::sync::Arc;
use tiny_http::{Header, Method, Response as TinyResponse, Server, StatusCode};
use tokio::runtime::Handle;
use tokio::sync::watch;

pub(crate) struct HttpControl {
    pub(crate) port: u16,
    pub(crate) token: String,
    #[allow(dead_code)]
    thread: std::thread::JoinHandle<()>,
}

pub(crate) fn spawn_control_plane(
    ctx: Arc<DaemonCtx>,
    shutdown_tx: watch::Sender<bool>,
) -> Result<HttpControl, Box<dyn std::error::Error + Send + Sync>> {
    let server = Server::http("127.0.0.1:0")?;
    let port = server
        .server_addr()
        .to_ip()
        .ok_or("HTTP control plane did not bind an IP socket")?
        .port();
    let token = atomek_core::platform::ipc::generate_control_token()?;
    let thread_token = token.clone();
    let handle = Handle::current();
    let thread = std::thread::Builder::new()
        .name("tytus-daemon-http-control".into())
        .spawn(move || {
            for request in server.incoming_requests() {
                handle_request(
                    request,
                    &thread_token,
                    &handle,
                    ctx.clone(),
                    shutdown_tx.clone(),
                );
            }
        })?;
    Ok(HttpControl {
        port,
        token,
        thread,
    })
}

fn handle_request(
    mut request: tiny_http::Request,
    token: &str,
    handle: &Handle,
    ctx: Arc<DaemonCtx>,
    shutdown_tx: watch::Sender<bool>,
) {
    if !is_loopback_request(&request) {
        let _ = request.respond(json_response(
            StatusCode(403),
            json!({"status":"error","code":"FORBIDDEN","error":"loopback only"}),
        ));
        return;
    }

    if !authorized(&request, token) {
        let _ = request.respond(json_response(
            StatusCode(401),
            json!({"status":"error","code":"UNAUTHORIZED","error":"missing or invalid token"}),
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
    let response = match (method, url.as_str()) {
        (Method::Get, "/v1/health") => DaemonResponse {
            status: "ok".into(),
            data: Some(json!({
                "ok": true,
                "daemon": {"pid": std::process::id()},
                "control": "http"
            })),
            error: None,
            code: None,
        },
        (Method::Get, "/v1/status") => handle.block_on(dispatch_command(
            &Request {
                cmd: "status".into(),
                args: serde_json::Value::Null,
            },
            &ctx,
            &shutdown_tx,
        )),
        (Method::Post, "/v1/command") => {
            let mut body = String::new();
            if let Err(e) = request.as_reader().read_to_string(&mut body) {
                DaemonResponse {
                    status: "error".into(),
                    data: None,
                    error: Some(format!("failed to read body: {}", e)),
                    code: Some("READ_ERROR".into()),
                }
            } else {
                match serde_json::from_str::<Request>(&body) {
                    Ok(req) => handle.block_on(dispatch_command(&req, &ctx, &shutdown_tx)),
                    Err(e) => DaemonResponse {
                        status: "error".into(),
                        data: None,
                        error: Some(format!("Invalid JSON: {}", e)),
                        code: Some("PARSE_ERROR".into()),
                    },
                }
            }
        }
        _ => DaemonResponse {
            status: "error".into(),
            data: None,
            error: Some("not found".into()),
            code: Some("NOT_FOUND".into()),
        },
    };

    let status = if response.status == "ok" {
        StatusCode(200)
    } else {
        StatusCode(400)
    };
    let _ = request.respond(json_response(
        status,
        serde_json::to_value(response).unwrap_or_else(
            |e| json!({"status":"error","code":"SERIALIZE_ERROR","error":e.to_string()}),
        ),
    ));
}

fn is_loopback_request(request: &tiny_http::Request) -> bool {
    request
        .remote_addr()
        .map(|addr| match addr.ip() {
            IpAddr::V4(ip) => ip.is_loopback(),
            IpAddr::V6(ip) => ip.is_loopback(),
        })
        .unwrap_or(false)
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

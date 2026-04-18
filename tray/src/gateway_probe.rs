//! Lightweight data-plane health probe.
//!
//! The tray's primary health signal is "can I reach my pod?" — not
//! "is the daemon happy?". The daemon only refreshes auth tokens; the
//! WireGuard tunnel and the pod gateway live completely independent of it.
//! A daemon-centric health model was showing 🔴 for users whose pods were
//! perfectly reachable via `curl`.
//!
//! This module does a minimal HTTP/1.0 GET against the stable dual-bound
//! endpoint `http://10.42.42.1:18080/v1/models` with a 2s hard timeout.
//! We don't authenticate — a 401 response from the gateway still proves
//! the tunnel is routing packets end-to-end. The check returns:
//!
//!   * Ok  — we received any `HTTP/…` response (tunnel + gateway alive)
//!   * Err — TCP connect failed or no HTTP response within 2s
//!
//! No third-party HTTP client is pulled in — the request/response shape
//! is trivial and `std::net` handles timeouts natively. The cost of the
//! probe is a few bytes over a loopback-like WireGuard hop; typical round
//! trip is <100ms.
//!
//! Gemini review (2026-04-18) flagged two risks that informed this design:
//!   * **Interface stubs**: raw TCP connect can succeed on a dead WG peer.
//!     → Mitigated by requiring an HTTP/* response prefix.
//!   * **Latency spikes**: 1s timeout flickers on mobile. → Bumped to 2s.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// Stable dual-bound endpoint. Every droplet exposes this.
const PROBE_HOST: &str = "10.42.42.1";
const PROBE_PORT: u16 = 18080;
const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// Returns true iff an HTTP/* response came back within 2s.
pub fn probe_gateway() -> bool {
    let addr = format!("{}:{}", PROBE_HOST, PROBE_PORT);
    let addr = match addr.parse::<std::net::SocketAddr>() {
        Ok(a) => a,
        Err(_) => return false,
    };
    let mut stream = match TcpStream::connect_timeout(&addr, PROBE_TIMEOUT) {
        Ok(s) => s,
        Err(_) => return false,
    };
    if stream.set_read_timeout(Some(PROBE_TIMEOUT)).is_err() { return false; }
    if stream.set_write_timeout(Some(PROBE_TIMEOUT)).is_err() { return false; }

    // Minimal request — we don't care about the status code, only that the
    // peer speaks HTTP at all. `Connection: close` so the server hangs up
    // after one response, keeping the read loop bounded.
    let req = format!(
        "GET /v1/models HTTP/1.0\r\nHost: {}:{}\r\nConnection: close\r\n\r\n",
        PROBE_HOST, PROBE_PORT,
    );
    if stream.write_all(req.as_bytes()).is_err() { return false; }

    // Read just enough to validate the status line. 16 bytes is plenty
    // for "HTTP/1.1 200 " and similar.
    let mut buf = [0u8; 16];
    let n = match stream.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return false,
    };
    n >= 5 && buf[..5] == *b"HTTP/"
}

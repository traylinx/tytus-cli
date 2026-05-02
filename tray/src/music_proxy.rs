//! Safe streaming proxy for expiring YouTube CDN URLs returned by yt-dlp.

use std::net::IpAddr;
use std::time::Duration;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use tiny_http::{Header, Request, Response, StatusCode};

pub fn encode_proxy_path(cdn_url: &str) -> String {
    URL_SAFE_NO_PAD.encode(cdn_url.as_bytes())
}

pub fn decode_proxy_url(encoded: &str) -> Result<String, String> {
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded.as_bytes())
        .map_err(|_| "bad proxy URL encoding".to_string())?;
    String::from_utf8(bytes).map_err(|_| "proxy URL is not UTF-8".to_string())
}

pub fn validate_proxy_target(raw: &str) -> Result<reqwest::Url, String> {
    let url = reqwest::Url::parse(raw).map_err(|_| "invalid proxy URL".to_string())?;
    if url.scheme() != "https" {
        return Err("proxy URL must be https".to_string());
    }
    let host = url.host_str().ok_or_else(|| "proxy URL missing host".to_string())?.to_ascii_lowercase();
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_private_or_local_ip(ip) {
            return Err("proxy URL IP target rejected".to_string());
        }
        return Err("proxy URL IP literals are rejected".to_string());
    }
    if host == "localhost" || host.ends_with(".localhost") {
        return Err("proxy URL localhost target rejected".to_string());
    }
    if !(host == "googlevideo.com" || host.ends_with(".googlevideo.com")) {
        return Err("proxy URL host is not allowed".to_string());
    }
    Ok(url)
}

fn is_private_or_local_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_private() || v4.is_loopback() || v4.is_link_local() || v4.is_unspecified(),
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified() || v6.is_unique_local(),
    }
}

pub fn handle_proxy(request: Request, encoded: &str) {
    let raw = match decode_proxy_url(encoded) {
        Ok(v) => v,
        Err(e) => return respond_text(request, 400, &e),
    };
    let url = match validate_proxy_target(&raw) {
        Ok(v) => v,
        Err(e) => return respond_text(request, 403, &e),
    };
    let range = request
        .headers()
        .iter()
        .find(|h| h.field.equiv("Range"))
        .map(|h| h.value.as_str().to_string());

    let client = match reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(300))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36")
        .build()
    {
        Ok(c) => c,
        Err(e) => return respond_text(request, 500, &format!("proxy client failed: {e}")),
    };
    let mut upstream_req = client.get(url);
    if let Some(range) = range {
        upstream_req = upstream_req.header(reqwest::header::RANGE, range);
    }
    let upstream = match upstream_req.send() {
        Ok(r) => r,
        Err(e) => return respond_text(request, 502, &format!("upstream stream failed: {e}")),
    };
    let status = upstream.status().as_u16();
    if !(upstream.status().is_success() || status == 206) {
        return respond_text(request, 502, &format!("upstream returned HTTP {status}"));
    }

    let len = upstream.content_length().and_then(|n| usize::try_from(n).ok());
    let headers = response_headers(upstream.headers());
    let resp = Response::new(StatusCode(status), headers, upstream, len, None);
    let _ = request.respond(resp);
}

fn response_headers(upstream: &reqwest::header::HeaderMap) -> Vec<Header> {
    let mut headers = vec![
        header("Access-Control-Allow-Origin", "*"),
        header("Access-Control-Allow-Methods", "GET, OPTIONS"),
        header("Access-Control-Allow-Headers", "Range"),
        header("Cache-Control", "no-store"),
        header("X-Content-Type-Options", "nosniff"),
    ];
    for name in [
        reqwest::header::CONTENT_TYPE,
        reqwest::header::CONTENT_LENGTH,
        reqwest::header::CONTENT_RANGE,
        reqwest::header::ACCEPT_RANGES,
    ] {
        if let Some(value) = upstream.get(&name) {
            if let Ok(v) = value.to_str() {
                headers.push(header(name.as_str(), v));
            }
        }
    }
    if !headers.iter().any(|h| h.field.equiv("Content-Type")) {
        headers.push(header("Content-Type", "audio/mp4"));
    }
    headers
}

fn respond_text(request: Request, status: u16, text: &str) {
    let resp = Response::from_string(text.to_string())
        .with_status_code(StatusCode(status))
        .with_header(header("Content-Type", "text/plain; charset=utf-8"))
        .with_header(header("Access-Control-Allow-Origin", "*"))
        .with_header(header("X-Content-Type-Options", "nosniff"));
    let _ = request.respond(resp);
}

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("valid header")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_url_roundtrips() {
        let raw = "https://rr4---sn-test.googlevideo.com/videoplayback?id=1";
        let encoded = encode_proxy_path(raw);
        assert_eq!(decode_proxy_url(&encoded).unwrap(), raw);
    }

    #[test]
    fn validates_googlevideo_only() {
        assert!(validate_proxy_target("https://rr4---sn-test.googlevideo.com/videoplayback").is_ok());
        assert!(validate_proxy_target("http://rr4---sn-test.googlevideo.com/videoplayback").is_err());
        assert!(validate_proxy_target("https://example.com/videoplayback").is_err());
        assert!(validate_proxy_target("https://127.0.0.1/videoplayback").is_err());
    }
}

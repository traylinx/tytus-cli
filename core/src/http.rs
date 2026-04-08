use crate::error::AtomekError;
use std::time::Duration;
use tracing::warn;

/// Shared HTTP client with retry logic.
/// Reuses connection pool across all requests (like claurst's AnthropicClient).
#[derive(Clone)]
pub struct HttpClient {
    inner: reqwest::Client,
    max_retries: u32,
    initial_retry_delay: Duration,
    max_retry_delay: Duration,
}

impl HttpClient {
    pub fn new() -> Self {
        let inner = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(5)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            inner,
            max_retries: 3,
            initial_retry_delay: Duration::from_secs(1),
            max_retry_delay: Duration::from_secs(30),
        }
    }

    pub fn get(&self, url: &str) -> reqwest::RequestBuilder {
        self.inner.get(url)
    }

    pub fn post(&self, url: &str) -> reqwest::RequestBuilder {
        self.inner.post(url)
    }

    /// Send a request with automatic retry on retryable errors.
    /// Respects Retry-After header (RFC 7231).
    pub async fn send_with_retry(
        &self,
        build_request: impl Fn() -> reqwest::RequestBuilder,
    ) -> std::result::Result<reqwest::Response, AtomekError> {
        let mut attempts = 0u32;
        let mut delay = self.initial_retry_delay;

        loop {
            attempts += 1;

            let resp = build_request()
                .send()
                .await
                .map_err(|e| AtomekError::Network(e.to_string()))?;

            let status = resp.status().as_u16();

            if resp.status().is_success() {
                return Ok(resp);
            }

            // Check if retryable
            let is_retryable = matches!(status, 429 | 502 | 503 | 529);

            if is_retryable && attempts <= self.max_retries {
                // Respect Retry-After header
                let retry_after = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .map(Duration::from_secs);

                let wait = retry_after.unwrap_or(delay);

                warn!(
                    status,
                    attempt = attempts,
                    wait_secs = wait.as_secs(),
                    "Retryable error, backing off"
                );

                tokio::time::sleep(wait).await;
                delay = (delay * 2).min(self.max_retry_delay);
                continue;
            }

            // Non-retryable or exhausted retries
            return Err(parse_http_error(status, resp).await);
        }
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse an HTTP error response into a semantic AtomekError.
async fn parse_http_error(status: u16, resp: reqwest::Response) -> AtomekError {
    let body = resp.text().await.unwrap_or_default();

    // Try to parse as JSON with "error" field
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
        let message = json["error"].as_str()
            .or_else(|| json["message"].as_str())
            .unwrap_or(&body)
            .to_string();

        match status {
            401 => return AtomekError::AuthExpired,
            403 => {
                // Check for plan_limit_reached
                if let Some(error_key) = json["error"].as_str() {
                    if error_key == "plan_limit_reached" {
                        return AtomekError::PodLimitReached {
                            limit: json["limit"].as_u64().unwrap_or(0) as u32,
                            current: json["current"].as_u64().unwrap_or(0) as u32,
                        };
                    }
                    if error_key == "no_plan" {
                        return AtomekError::NoSubscription;
                    }
                }
                return AtomekError::ApiStatus { status, message };
            }
            404 => {
                if let Some(error_key) = json["error"].as_str() {
                    if error_key == "not_found" || error_key == "no_pod_allocated" {
                        return AtomekError::NoPod;
                    }
                    if error_key == "config_not_found" {
                        return AtomekError::ConfigNotReady;
                    }
                }
                return AtomekError::NoPod;
            }
            429 => return AtomekError::RateLimited,
            503 => {
                let retry_after = json["retry_after"].as_u64().unwrap_or(300);
                return AtomekError::NoCapacity { retry_after };
            }
            _ => return AtomekError::ApiStatus { status, message },
        }
    }

    AtomekError::ApiStatus {
        status,
        message: body,
    }
}

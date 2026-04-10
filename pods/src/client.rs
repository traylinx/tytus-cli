use atomek_core::HttpClient;

const TYTUS_PROVIDER_URL: &str = "https://tytus.traylinx.com";

/// HTTP client for tytus.traylinx.com Provider API.
/// Uses shared HttpClient for connection pooling and retry logic.
/// All requests use Sentinel Pass A2A authentication headers.
pub struct TytusClient {
    http: HttpClient,
    base_url: String,
    secret_token: String,
    agent_user_id: String,
}

impl TytusClient {
    pub fn new(http: &HttpClient, secret_token: &str, agent_user_id: &str) -> Self {
        Self {
            http: http.clone(),
            base_url: TYTUS_PROVIDER_URL.to_string(),
            secret_token: secret_token.to_string(),
            agent_user_id: agent_user_id.to_string(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.http.get(&format!("{}{}", self.base_url, path))
            .header("X-Agent-Secret-Token", &self.secret_token)
            .header("X-Agent-User-Id", &self.agent_user_id)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
    }

    #[allow(dead_code)]
    pub(crate) fn post(&self, path: &str) -> reqwest::RequestBuilder {
        self.http.post(&format!("{}{}", self.base_url, path))
            .header("X-Agent-Secret-Token", &self.secret_token)
            .header("X-Agent-User-Id", &self.agent_user_id)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
    }

    /// Send a GET with retry logic from the shared HttpClient.
    pub(crate) async fn get_with_retry(&self, path: &str) -> atomek_core::Result<reqwest::Response> {
        let url = format!("{}{}", self.base_url, path);
        let st = self.secret_token.clone();
        let auid = self.agent_user_id.clone();

        self.http.send_with_retry(|| {
            self.http.get(&url)
                .header("X-Agent-Secret-Token", &st)
                .header("X-Agent-User-Id", &auid)
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
        }).await
    }

    /// Send a POST with retry logic from the shared HttpClient.
    #[allow(dead_code)] // kept symmetric with get_with_retry; no current call site
    pub(crate) async fn post_with_retry(&self, path: &str) -> atomek_core::Result<reqwest::Response> {
        let url = format!("{}{}", self.base_url, path);
        let st = self.secret_token.clone();
        let auid = self.agent_user_id.clone();

        self.http.send_with_retry(|| {
            self.http.post(&url)
                .header("X-Agent-Secret-Token", &st)
                .header("X-Agent-User-Id", &auid)
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
        }).await
    }
}

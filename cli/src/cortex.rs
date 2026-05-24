//! Local Cortex management.
//!
//! Sprint: `services/tytus-os/development/sprints/2026-05-21-chat-with-pods-local-cortex-parity/`
//!
//! Wraps a bundled `docker-compose.yml` that brings up Postgres + Redis +
//! tytus-cortex on `127.0.0.1:8098`. Exposes `tytus cortex {up,down,status,
//! test,reset,token,logs,upgrade,version}`. The tray daemon's
//! `/api/cortex/*` endpoints are the runtime surface; this CLI is the
//! install + recovery surface.
//!
//! Token model (R14 from the sprint risk register):
//!   - `INTERNAL_SERVICE_TOKEN` is the service-to-service shared secret
//!     baked into `docker-compose.yml`. The tray daemon uses it for
//!     `/tytus/chat` calls. Never user-visible.
//!   - The per-user `ctx_*` token (minted via `POST /v1/users` and stored
//!     in `state.json::cortex_local_token`) is for `/v1/*` user-scoped
//!     endpoints — memory search, session list, profile.
//!
//! Mixing them is the predicted 401 trap.

use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::state::{cortex_profile_is_local, CliState};

/// Compose template baked into the release binary. Lives at
/// `services/tytus-cli/contrib/cortex/docker-compose.yml` in the source
/// tree and is dropped to `<state_dir>/cortex/docker-compose.yml` on first
/// `tytus cortex up` so we never read from a path that depends on where
/// the user installed the binary.
const COMPOSE_TEMPLATE: &str = include_str!("../../contrib/cortex/docker-compose.yml");

/// Default port the local Cortex container binds to.
pub const DEFAULT_PORT: u16 = 8098;

/// How long to wait for `/health/live` to return ok after `compose up`.
pub const HEALTH_WAIT_SECS: u64 = 90;

/// Sentinel image tag bundled with this Tytus CLI release. Bumped per
/// `tytus cortex upgrade` cycle or per sprint release.
pub const CORTEX_PINNED_TAG: &str = "2026-05-17";

#[derive(Debug, Clone, Subcommand)]
pub enum CortexAction {
    /// Bring up local Cortex (Postgres + Redis + Cortex API). Idempotent.
    Up {
        /// Override host port (default 8098).
        #[arg(long)]
        port: Option<u16>,
        /// Pin to a specific Cortex image tag (default: bundled).
        #[arg(long)]
        pin: Option<String>,
    },
    /// Stop containers. With `--purge`, also remove data volumes.
    Down {
        /// Remove Postgres/Redis volumes (DESTRUCTIVE — wipes memory).
        #[arg(long)]
        purge: bool,
    },
    /// Show profile + health + version. Reads through the tray daemon.
    Status {
        /// Emit JSON instead of human text.
        #[arg(long)]
        json: bool,
    },
    /// Send a probe message and report round-trip latency.
    Test {
        /// Message to send (default: "Reply with PONG and nothing else.").
        message: Option<String>,
    },
    /// Stop + remove volumes + clear state.json Cortex fields.
    Reset {
        /// Skip the confirmation prompt.
        #[arg(long)]
        yes: bool,
    },
    /// Manage the per-user `ctx_*` token (NOT the internal service token).
    Token {
        #[command(subcommand)]
        action: CortexTokenAction,
    },
    /// Tail Cortex logs.
    Logs {
        /// Trailing lines to print before tailing.
        #[arg(long, default_value = "200")]
        tail: usize,
        /// Keep tailing.
        #[arg(long)]
        follow: bool,
    },
    /// Run pending Alembic migrations against the local volume.
    Upgrade,
    /// Show pinned vs latest known tag.
    Version,
}

#[derive(Debug, Clone, Subcommand)]
pub enum CortexTokenAction {
    /// Rotate the local `ctx_*` token (ends in-flight chats).
    Rotate,
    /// Show metadata (prefix + age). Never prints the token body.
    Show,
}

/// Snapshot from `GET /api/cortex/status` on the tray daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CortexStatus {
    pub profile: String,
    pub local_running: bool,
    pub local_port: Option<u16>,
    pub local_version: Option<String>,
    pub local_token_present: bool,
    pub local_health: Option<CortexHealth>,
    pub local_uptime_seconds: Option<u64>,
    pub local_storage_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CortexHealth {
    pub postgres: String,
    pub redis: String,
    pub llm_config: String,
}

/// Entry point used by `main.rs::Commands::Cortex { action }`.
pub async fn cmd_cortex(action: CortexAction, json: bool) -> Result<(), String> {
    match action {
        CortexAction::Up { port, pin } => cmd_up(port, pin, json).await,
        CortexAction::Down { purge } => cmd_down(purge, json).await,
        CortexAction::Status { json: json_flag } => cmd_status(json || json_flag).await,
        CortexAction::Test { message } => cmd_test(message, json).await,
        CortexAction::Reset { yes } => cmd_reset(yes, json).await,
        CortexAction::Token { action } => cmd_token(action, json).await,
        CortexAction::Logs { tail, follow } => cmd_logs(tail, follow).await,
        CortexAction::Upgrade => cmd_upgrade(json).await,
        CortexAction::Version => cmd_version(json).await,
    }
}

// --- M1.3: status ---------------------------------------------------------

/// Composite report shown by `tytus cortex status [--json]`.
///
/// Sources, in order:
///   - state.json — profile, port, token presence, pinned version, started_at
///   - `docker ps` — container running status
///   - `GET http://127.0.0.1:<port>/health/live` — Postgres + Redis + LLM
///     reachability from inside Cortex
///
/// Each probe is independently fault-tolerant: a missing Docker daemon
/// reports `docker_status: "unavailable"` rather than failing the whole
/// command, so the operator gets actionable detail in one go.
#[derive(Debug, Clone, Serialize)]
pub struct CortexStatusReport {
    pub profile: String,
    pub local_token_present: bool,
    pub local_user_id_present: bool,
    pub internal_service_token_present: bool,
    pub local_port: u16,
    pub local_version_pinned: Option<String>,
    pub local_started_at: Option<String>,
    pub docker_status: DockerStatus,
    pub api_health: Option<CortexHealth>,
    pub api_reachable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub enum DockerStatus {
    /// Docker is reachable and containers are running.
    Running { containers: Vec<String> },
    /// Docker reachable but Cortex containers are not running.
    Stopped,
    /// `docker` CLI not on PATH or daemon not reachable.
    Unavailable { reason: String },
}

async fn cmd_status(json: bool) -> Result<(), String> {
    let state = CliState::load_file_only();

    let port = state.cortex_local_port.unwrap_or(DEFAULT_PORT);
    let profile = state
        .cortex_profile
        .clone()
        .unwrap_or_else(|| "cloud".to_string());

    let docker_status = probe_docker().await;
    let (api_reachable, api_health) = probe_health(port).await;

    let report = CortexStatusReport {
        profile,
        local_token_present: state.cortex_local_token.is_some(),
        local_user_id_present: state.cortex_local_user_id.is_some(),
        internal_service_token_present: state.cortex_internal_service_token.is_some(),
        local_port: port,
        local_version_pinned: state.cortex_local_version_pinned.clone(),
        local_started_at: state.cortex_local_started_at.clone(),
        docker_status,
        api_health,
        api_reachable,
    };

    if json {
        let pretty = serde_json::to_string_pretty(&report)
            .map_err(|e| format!("serialize status report: {e}"))?;
        println!("{pretty}");
    } else {
        print_status_text(&report, cortex_profile_is_local(&state.cortex_profile));
    }
    Ok(())
}

fn print_status_text(report: &CortexStatusReport, is_local: bool) {
    println!("Cortex profile:           {}", report.profile);
    if !is_local {
        println!("Local stack is NOT active — chat goes to cloud Cortex via the Provider.");
        println!("Run `tytus cortex up` to install and switch.");
        return;
    }

    println!("Local port:               {}", report.local_port);
    println!(
        "Pinned image tag:         {}",
        report.local_version_pinned.as_deref().unwrap_or("(unset)")
    );
    println!(
        "Started at:               {}",
        report.local_started_at.as_deref().unwrap_or("(never)")
    );
    println!(
        "ctx_* token present:      {}",
        if report.local_token_present {
            "yes"
        } else {
            "NO"
        }
    );
    println!(
        "Internal service token:   {}",
        if report.internal_service_token_present {
            "yes"
        } else {
            "NO"
        }
    );
    match &report.docker_status {
        DockerStatus::Running { containers } => {
            println!(
                "Docker:                   running ({} container(s))",
                containers.len()
            );
            for c in containers {
                println!("  - {c}");
            }
        }
        DockerStatus::Stopped => {
            println!("Docker:                   reachable, but Cortex containers are stopped");
        }
        DockerStatus::Unavailable { reason } => {
            println!("Docker:                   UNAVAILABLE — {reason}");
        }
    }
    if report.api_reachable {
        println!("API (/health/live):       reachable");
        if let Some(h) = &report.api_health {
            println!("  postgres:   {}", h.postgres);
            println!("  redis:      {}", h.redis);
            println!("  llm_config: {}", h.llm_config);
        }
    } else {
        println!(
            "API (/health/live):       unreachable on 127.0.0.1:{}",
            report.local_port
        );
    }
}

async fn probe_docker() -> DockerStatus {
    // Cheap reachability probe — `docker ps -q --filter name=tytus-cortex`.
    // If `docker` is missing or the daemon is down, the process exits non-zero
    // within ~50ms (no network) and we surface that to the operator.
    let output = match tokio::process::Command::new("docker")
        .arg("ps")
        .arg("-q")
        .arg("--filter")
        .arg("name=tytus-cortex")
        .output()
        .await
    {
        Ok(o) => o,
        Err(e) => {
            return DockerStatus::Unavailable {
                reason: format!("docker CLI not found or not executable: {e}"),
            };
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return DockerStatus::Unavailable {
            reason: if stderr.is_empty() {
                format!("docker ps exited with status {}", output.status)
            } else {
                stderr
            },
        };
    }

    let ids: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .collect();

    if ids.is_empty() {
        return DockerStatus::Stopped;
    }

    // Resolve container names for readability.
    let mut containers = Vec::with_capacity(ids.len());
    for id in &ids {
        let name_out = tokio::process::Command::new("docker")
            .arg("inspect")
            .arg("-f")
            .arg("{{.Name}}")
            .arg(id)
            .output()
            .await;
        let name = name_out
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(
                        String::from_utf8_lossy(&o.stdout)
                            .trim()
                            .trim_start_matches('/')
                            .to_string(),
                    )
                } else {
                    None
                }
            })
            .unwrap_or_else(|| id.clone());
        containers.push(name);
    }
    DockerStatus::Running { containers }
}

async fn probe_health(port: u16) -> (bool, Option<CortexHealth>) {
    let url = format!("http://127.0.0.1:{port}/health/live");
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
    {
        Ok(c) => c,
        Err(_) => return (false, None),
    };

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            // Cortex's /health/live returns the HealthLiveResponse model.
            // We only care about three fields; tolerate extras.
            let body: serde_json::Value = match resp.json().await {
                Ok(v) => v,
                Err(_) => return (true, None),
            };
            let extract = |k: &str| {
                body.get(k)
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string()
            };
            (
                true,
                Some(CortexHealth {
                    postgres: extract("postgres"),
                    redis: extract("redis"),
                    llm_config: extract("llm_config"),
                }),
            )
        }
        _ => (false, None),
    }
}

// --- M1.4: up / down ------------------------------------------------------

async fn cmd_up(port: Option<u16>, pin: Option<String>, json: bool) -> Result<(), String> {
    // 1. Docker reachable?
    ensure_docker_reachable().await?;

    // 2. Resolve install paths.
    let bundle_dir = bundle_dir()?;
    std::fs::create_dir_all(&bundle_dir)
        .map_err(|e| format!("create {}: {e}", bundle_dir.display()))?;
    let compose_path = bundle_dir.join("docker-compose.yml");
    let env_path = bundle_dir.join(".env");

    // 3. Materialize the compose file (idempotent — overwrite each run so
    //    upgrades pick up template changes).
    std::fs::write(&compose_path, COMPOSE_TEMPLATE)
        .map_err(|e| format!("write {}: {e}", compose_path.display()))?;

    // 4. Read existing state.json so we preserve / mint secrets correctly.
    let mut state = CliState::load_file_only();

    // 5. Ensure all three secrets exist; mint random ones if missing.
    if state.cortex_internal_service_token.is_none() {
        state.cortex_internal_service_token = Some(random_hex(32)?);
    }
    let postgres_password = match read_env_var(&env_path, "POSTGRES_PASSWORD")? {
        Some(v) if !v.is_empty() => v,
        _ => random_hex(24)?,
    };
    let encryption_key = match read_env_var(&env_path, "ENCRYPTION_KEY")? {
        Some(v) if !v.is_empty() => v,
        _ => random_base64_urlsafe(32)?, // Fernet keys are 32-byte URL-safe b64
    };
    if state.cortex_local_user_id.is_none() {
        state.cortex_local_user_id = Some(format!("tytus-local-{}", random_hex(6)?));
    }
    let resolved_port = port.or(state.cortex_local_port).unwrap_or(DEFAULT_PORT);
    state.cortex_local_port = Some(resolved_port);
    let resolved_tag = pin
        .or_else(|| state.cortex_local_version_pinned.clone())
        .unwrap_or_else(|| CORTEX_PINNED_TAG.to_string());
    state.cortex_local_version_pinned = Some(resolved_tag.clone());

    // 6. Discover SwitchAILocal API key.
    //
    // Cortex needs SWITCH_AI_API_KEY in its env to talk to the local
    // SwitchAILocal gateway (the `test://local` DAM escape hatch reads
    // `settings.switch_ai_api_key` directly when chat_target=brain).
    // Priority: existing .env value > AIL_API_KEY env var > empty (with
    // a warning so the user knows chat will 503 until they fix it).
    let switch_ai_api_key = match read_env_var(&env_path, "SWITCH_AI_API_KEY")? {
        Some(v) if !v.is_empty() => v,
        _ => std::env::var("AIL_API_KEY").unwrap_or_default(),
    };
    if switch_ai_api_key.is_empty() {
        eprintln!(
            "warning: SWITCH_AI_API_KEY is empty. Local Cortex will start but chat\n\
                  will fail with `runtime_key_missing` until you set AIL_API_KEY in\n\
                  your shell or edit ~/Library/Application Support/tytus/cortex/.env."
        );
    }

    // 7. Write .env (mode 0600).
    let env_body = format!(
        "POSTGRES_PASSWORD={postgres_password}\n\
         ENCRYPTION_KEY={encryption_key}\n\
         INTERNAL_SERVICE_TOKEN={internal}\n\
         CORTEX_IMAGE_TAG={tag}\n\
         CORTEX_HOST_PORT={port}\n\
         SWITCH_AI_API_KEY={ail_key}\n",
        internal = state.cortex_internal_service_token.as_deref().unwrap_or(""),
        tag = resolved_tag,
        port = resolved_port,
        ail_key = switch_ai_api_key,
    );
    std::fs::write(&env_path, env_body)
        .map_err(|e| format!("write {}: {e}", env_path.display()))?;
    secure_perms(&env_path)?;

    // 8. Mark profile=local + persist BEFORE we start (so a crash mid-up
    //    leaves us in a recoverable state).
    state.cortex_profile = Some("local".to_string());
    state
        .save_critical()
        .map_err(|e| format!("persist state.json: {e}"))?;

    // 9. `docker compose up -d` — let stdout/stderr stream to the user.
    eprintln!("→ docker compose up -d (this can take ~30–90s on first run while images pull)");
    let status = run_compose(&compose_path, &env_path, &["up", "-d"]).await?;
    if !status.success() {
        return Err(format!(
            "docker compose up exited with status {status}. Check `tytus cortex logs` for detail."
        ));
    }

    // 9. Poll /health/live.
    eprintln!("→ waiting for Cortex /health/live on 127.0.0.1:{resolved_port}");
    let healthy = wait_for_health(resolved_port, HEALTH_WAIT_SECS).await;
    if !healthy {
        return Err(format!(
            "Cortex did not become healthy within {HEALTH_WAIT_SECS}s. \
             Run `tytus cortex logs` to investigate."
        ));
    }

    // 10. Per-user ctx_* token mint is deliberately omitted.
    //
    // Dogfood F2 (2026-05-22): when Cortex runs in TYTUS_CORTEX_MODE=true,
    // `POST /v1/users` is disabled by design (returns 404 "User
    // self-registration is disabled in Tytus Cortex mode"). The local
    // stack authenticates via INTERNAL_SERVICE_TOKEN exclusively.
    //
    // Memory-search via `host.ai.cortexSearch` is documented as deferred
    // in DOGFOOD-FINDINGS.md until either (a) Cortex exposes a write
    // endpoint or (b) we change the auth model for /v1/* in Tytus mode.

    state.cortex_local_started_at = Some(chrono::Utc::now().to_rfc3339());
    state
        .save_critical()
        .map_err(|e| format!("persist state.json: {e}"))?;

    if json {
        let payload = serde_json::json!({
            "ok": true,
            "profile": "local",
            "port": resolved_port,
            "pinned_tag": resolved_tag,
            "started_at": state.cortex_local_started_at,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload)
                .map_err(|e| format!("serialize up payload: {e}"))?
        );
    } else {
        println!("✓ Local Cortex is up on http://127.0.0.1:{resolved_port}");
        println!("  Profile flipped to: local");
        println!("  Pinned image tag:   {resolved_tag}");
        println!();
        println!("  Run `tytus cortex test` to send a probe message.");
    }
    Ok(())
}

async fn cmd_down(purge: bool, json: bool) -> Result<(), String> {
    let bundle_dir = bundle_dir()?;
    let compose_path = bundle_dir.join("docker-compose.yml");
    let env_path = bundle_dir.join(".env");
    if !compose_path.exists() {
        return Err(format!(
            "no Cortex install found at {} — nothing to bring down.",
            compose_path.display()
        ));
    }
    let mut sub: Vec<&str> = vec!["down"];
    if purge {
        sub.push("-v");
    }
    let status = run_compose(&compose_path, &env_path, &sub).await?;
    if !status.success() {
        return Err(format!("docker compose down exited with status {status}"));
    }

    let mut state = CliState::load_file_only();
    state.cortex_profile = Some("cloud".to_string());
    state.cortex_local_started_at = None;
    if purge {
        // Wipe everything Cortex-related — caller asked for it.
        state.cortex_local_token = None;
        state.cortex_local_user_id = None;
        state.cortex_internal_service_token = None;
    }
    state
        .save_critical()
        .map_err(|e| format!("persist state.json: {e}"))?;

    if json {
        let payload = serde_json::json!({"ok": true, "purged": purge});
        println!(
            "{}",
            serde_json::to_string_pretty(&payload)
                .map_err(|e| format!("serialize down payload: {e}"))?
        );
    } else {
        if purge {
            println!("✓ Local Cortex stopped and volumes removed. All local memory is gone.");
        } else {
            println!("✓ Local Cortex stopped. Data volumes preserved.");
        }
        println!("  Profile flipped to: cloud");
    }
    Ok(())
}

// --- M1.5: test / token / reset / logs / upgrade -------------------------

async fn cmd_test(message: Option<String>, json: bool) -> Result<(), String> {
    let state = CliState::load_file_only();
    if !cortex_profile_is_local(&state.cortex_profile) {
        return Err(
            "profile is not 'local' — run `tytus cortex up` first, or talk to cloud Cortex \
             via the regular chat path."
                .into(),
        );
    }
    let port = state.cortex_local_port.unwrap_or(DEFAULT_PORT);
    // Dogfood F2 (2026-05-22): /v1/users is disabled in Tytus Cortex mode, so
    // no ctx_* token is mintable. Cortex IS reachable via the service token on
    // /internal/tytus/chat — same path the tray daemon uses. Reuse it here so
    // `tytus cortex test` actually works.
    let service_token = state
        .cortex_internal_service_token
        .as_deref()
        .ok_or_else(|| {
            "no INTERNAL_SERVICE_TOKEN in state.json. Run `tytus cortex up` to mint \
         the local stack secrets."
                .to_string()
        })?;
    let user_id = state
        .cortex_local_user_id
        .clone()
        .unwrap_or_else(|| "tytus-local".to_string());

    let probe = message.unwrap_or_else(|| "Reply with PONG and nothing else.".to_string());
    let body = serde_json::json!({
        "client_id": user_id,
        "pod_id": "99",
        "route_id": "tytuslocal-probe",
        "agent_type": "nemoclaw",
        "session_id": null,
        "message": probe,
        "stream": false,
        "model_preference": "fast",
        "chat_target": "brain",
        "app_id": "tytus-cli-probe",
        "agent_mode": "brain",
    });

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| format!("build http client: {e}"))?;

    let start = std::time::Instant::now();
    let resp = client
        .post(format!("http://127.0.0.1:{port}/internal/tytus/chat"))
        .bearer_auth(service_token)
        .header("X-Tytus-Cortex-Token", service_token)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("POST /internal/tytus/chat: {e}"))?;
    let status = resp.status();
    let payload: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("decode /internal/tytus/chat response: {e}"))?;
    let elapsed_ms = start.elapsed().as_millis();

    if !status.is_success() {
        return Err(format!(
            "Cortex returned HTTP {status}: {}",
            serde_json::to_string(&payload).unwrap_or_default()
        ));
    }

    let content = payload
        .get("content")
        .and_then(|v| v.as_str())
        .or_else(|| payload.get("message").and_then(|v| v.as_str()))
        .unwrap_or("(no content field in response)");

    if json {
        let report = serde_json::json!({
            "ok": true,
            "round_trip_ms": elapsed_ms,
            "response": content,
            "raw": payload,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|e| format!("serialize test payload: {e}"))?
        );
    } else {
        println!("Round-trip: {elapsed_ms}ms");
        println!("Response:   {content}");
    }
    Ok(())
}

async fn cmd_reset(yes: bool, json: bool) -> Result<(), String> {
    if !yes {
        eprintln!(
            "This will stop Cortex, remove its Docker volumes, and clear all local \
             session/memory data. Re-run with `--yes` to confirm."
        );
        return Err("aborted: missing --yes confirmation".into());
    }
    // Re-use down with purge, then clear remaining state.
    cmd_down(true, json).await?;
    let mut state = CliState::load_file_only();
    state.cortex_local_version_pinned = None;
    state.cortex_local_port = None;
    state
        .save_critical()
        .map_err(|e| format!("persist state.json: {e}"))?;
    if !json {
        println!("✓ Local Cortex reset to factory. Run `tytus cortex up` to reinstall.");
    }
    Ok(())
}

async fn cmd_token(action: CortexTokenAction, json: bool) -> Result<(), String> {
    // Dogfood F2: ctx_* token rotation depends on POST /v1/users, which is
    // disabled in Tytus Cortex mode. The internal service token is the
    // only credential the local stack uses; it's minted on `cortex up` and
    // not user-rotatable in v1.
    match action {
        CortexTokenAction::Rotate => Err(
            "ctx_* token rotation is unavailable while Cortex runs in Tytus mode \
             (POST /v1/users is disabled). The local stack authenticates via \
             INTERNAL_SERVICE_TOKEN, which is minted by `tytus cortex up`. \
             To regenerate, run `tytus cortex reset --yes` then `tytus cortex up`."
                .into(),
        ),
        CortexTokenAction::Show => {
            let state = CliState::load_file_only();
            let internal_present = state.cortex_internal_service_token.is_some();
            let internal_prefix: Option<String> = state
                .cortex_internal_service_token
                .as_ref()
                .map(|t| t.chars().take(6).collect());
            if json {
                let p = serde_json::json!({
                    "ctx_token_present": false,
                    "ctx_token_reason": "disabled in Tytus Cortex mode",
                    "internal_service_token_present": internal_present,
                    "internal_service_token_prefix": internal_prefix,
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&p)
                        .map_err(|e| format!("serialize show payload: {e}"))?
                );
            } else {
                println!("ctx_* token:                disabled (Tytus Cortex mode)");
                if internal_present {
                    println!(
                        "INTERNAL_SERVICE_TOKEN:     present, prefix {}…",
                        internal_prefix.unwrap_or_default()
                    );
                } else {
                    println!("INTERNAL_SERVICE_TOKEN:     ABSENT — run `tytus cortex up` to mint.");
                }
            }
            Ok(())
        }
    }
}

async fn cmd_logs(tail: usize, follow: bool) -> Result<(), String> {
    let bundle_dir = bundle_dir()?;
    let compose_path = bundle_dir.join("docker-compose.yml");
    let env_path = bundle_dir.join(".env");
    if !compose_path.exists() {
        return Err(format!(
            "no Cortex install at {} — nothing to tail.",
            compose_path.display()
        ));
    }
    let tail_str = tail.to_string();
    let mut sub: Vec<&str> = vec!["logs", "--tail", &tail_str];
    if follow {
        sub.push("-f");
    }
    let status = run_compose(&compose_path, &env_path, &sub).await?;
    if !status.success() {
        return Err(format!("docker compose logs exited with status {status}"));
    }
    Ok(())
}

async fn cmd_upgrade(json: bool) -> Result<(), String> {
    let bundle_dir = bundle_dir()?;
    let compose_path = bundle_dir.join("docker-compose.yml");
    let env_path = bundle_dir.join(".env");
    if !compose_path.exists() {
        return Err("no Cortex install found — run `tytus cortex up` first.".into());
    }
    // Re-materialize the compose template + Alembic migrations.
    std::fs::write(&compose_path, COMPOSE_TEMPLATE)
        .map_err(|e| format!("refresh compose template: {e}"))?;

    let status = run_compose(&compose_path, &env_path, &["pull"]).await?;
    if !status.success() {
        return Err(format!("docker compose pull exited with status {status}"));
    }
    let status = run_compose(&compose_path, &env_path, &["up", "-d"]).await?;
    if !status.success() {
        return Err(format!("docker compose up exited with status {status}"));
    }
    let status = run_compose(
        &compose_path,
        &env_path,
        &["exec", "-T", "cortex", "alembic", "upgrade", "head"],
    )
    .await?;
    if !status.success() {
        return Err(format!(
            "alembic upgrade exited with status {status}. Check `tytus cortex logs`."
        ));
    }
    if json {
        println!("{}", serde_json::json!({"ok": true}));
    } else {
        println!("✓ Cortex image pulled and migrations applied.");
    }
    Ok(())
}

async fn cmd_version(json: bool) -> Result<(), String> {
    let pinned = CORTEX_PINNED_TAG;
    if json {
        let payload = serde_json::json!({
            "pinned_tag": pinned,
            "default_port": DEFAULT_PORT,
            "health_wait_secs": HEALTH_WAIT_SECS,
        });
        let pretty = serde_json::to_string_pretty(&payload)
            .map_err(|e| format!("serialize version payload: {e}"))?;
        println!("{pretty}");
    } else {
        println!("Pinned Cortex image tag: {pinned}");
        println!("Default port:            {DEFAULT_PORT}");
        println!("Health wait (seconds):   {HEALTH_WAIT_SECS}");
    }
    Ok(())
}

// --- helpers shared across milestones -------------------------------------

/// `<state_dir>/cortex/` — bundle dir for the per-user Cortex install.
fn bundle_dir() -> Result<PathBuf, String> {
    Ok(state_dir()?.join("cortex"))
}

fn state_dir() -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or_else(|| "home dir not found".to_string())?;
        Ok(home
            .join("Library")
            .join("Application Support")
            .join("tytus"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let cfg = dirs::config_dir().ok_or_else(|| "config dir not found".to_string())?;
        Ok(cfg.join("tytus"))
    }
}

async fn ensure_docker_reachable() -> Result<(), String> {
    let output = tokio::process::Command::new("docker")
        .arg("version")
        .arg("--format")
        .arg("{{.Server.Version}}")
        .output()
        .await
        .map_err(|e| {
            format!(
                "Docker CLI not found on PATH. Install Docker Desktop from \
                 https://www.docker.com/products/docker-desktop and retry. (underlying: {e})"
            )
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!(
            "Docker daemon not reachable (`docker version` failed). Start Docker Desktop \
             and retry. Details: {stderr}"
        ));
    }
    Ok(())
}

async fn wait_for_health(port: u16, max_secs: u64) -> bool {
    let url = format!("http://127.0.0.1:{port}/health/live");
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    let deadline = std::time::Instant::now() + Duration::from_secs(max_secs);
    while std::time::Instant::now() < deadline {
        if let Ok(resp) = client.get(&url).send().await {
            if resp.status().is_success() {
                // Cortex returns 200 once postgres + redis + llm_config are all OK.
                return true;
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    false
}

fn random_hex(n_bytes: usize) -> Result<String, String> {
    let mut buf = vec![0u8; n_bytes];
    getrandom::fill(&mut buf).map_err(|e| format!("getrandom: {e}"))?;
    Ok(hex::encode(buf))
}

fn random_base64_urlsafe(n_bytes: usize) -> Result<String, String> {
    use base64::Engine;
    let mut buf = vec![0u8; n_bytes];
    getrandom::fill(&mut buf).map_err(|e| format!("getrandom: {e}"))?;
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf))
}

/// Spawn `docker compose -f COMPOSE --env-file ENV <sub...>` with the
/// vars from `ENV` injected into the subprocess environment.
///
/// Compose v2's `--env-file` flag only affects the *container runtime*
/// — it does NOT drive YAML interpolation (`${POSTGRES_PASSWORD}` etc).
/// Interpolation reads from the shell env or a `.env` file in the
/// compose-file's directory + CWD. Since we can't rely on either of
/// those for tytus's bundled compose path, we parse the .env ourselves
/// and pass each pair via `.env()`. Belt-and-suspenders: keep
/// `--env-file` so the container side stays explicit.
///
/// Caught during dogfood 2026-05-22 — see DOGFOOD-FINDINGS.md F1.
async fn run_compose(
    compose_path: &Path,
    env_path: &Path,
    subcommand: &[&str],
) -> Result<std::process::ExitStatus, String> {
    let env_pairs = if env_path.exists() {
        load_env_file(env_path)?
    } else {
        Vec::new()
    };
    let mut cmd = tokio::process::Command::new("docker");
    cmd.arg("compose").arg("-f").arg(compose_path);
    if env_path.exists() {
        cmd.arg("--env-file").arg(env_path);
    }
    for sub in subcommand {
        cmd.arg(sub);
    }
    for (k, v) in &env_pairs {
        cmd.env(k, v);
    }
    cmd.status()
        .await
        .map_err(|e| format!("spawn docker compose: {e}"))
}

/// Parse a dotenv-style file into KEY=VALUE pairs. Skips blanks and comments.
/// Used to inject vars into the docker subprocess environment for Compose
/// YAML interpolation (see `cmd_up` for context — Compose v2 `--env-file`
/// does NOT cover interpolation).
fn load_env_file(path: &Path) -> Result<Vec<(String, String)>, String> {
    let contents =
        std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let mut pairs = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            pairs.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    Ok(pairs)
}

/// Read a single `KEY=VALUE` from a dotenv-style file. Returns Ok(None) if
/// the file doesn't exist (first install).
fn read_env_var(path: &Path, key: &str) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let contents =
        std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            if k.trim() == key {
                return Ok(Some(v.trim().to_string()));
            }
        }
    }
    Ok(None)
}

#[cfg(unix)]
fn secure_perms(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let perms = std::fs::Permissions::from_mode(0o600);
    std::fs::set_permissions(path, perms).map_err(|e| format!("chmod 0600 {}: {e}", path.display()))
}

#[cfg(not(unix))]
fn secure_perms(_path: &Path) -> Result<(), String> {
    // Windows: rely on the user profile being ACL-restricted by default.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_tag_is_iso_date_like() {
        // Sanity: pinned tag follows a stable shape. Bump deliberately, not by typo.
        assert!(CORTEX_PINNED_TAG.len() >= 8);
        assert!(CORTEX_PINNED_TAG
            .chars()
            .all(|c| c.is_ascii_digit() || c == '-'));
    }

    #[test]
    fn default_port_matches_docker_compose() {
        // Mirror of port published in services/tytus-cortex/docker-compose.tytus.yml.
        assert_eq!(DEFAULT_PORT, 8098);
    }

    #[tokio::test]
    async fn version_command_runs() {
        // Smoke: the only fully-implemented command should not panic.
        let result = cmd_version(true).await;
        assert!(result.is_ok());
    }
}

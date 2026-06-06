//! Desktop LLM provider configuration helpers for the App Store.
//!
//! Narrow by design: app ids come from the embedded catalog, adapters own
//! fixed config paths, and request bodies never choose filesystem locations.

use serde::Serialize;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const TYTUS_PROVIDER_ID: &str = "tytus-ail";
pub const TYTUS_MODEL_ID: &str = "ail-compound";
const PI_SWITCHAI_PACKAGE: &str = "npm:@traylinx/pi-switchai-provider";

#[derive(Clone, Debug)]
pub struct TytusAilProvider {
    /// OpenAI-compatible base URL including `/v1`.
    pub base_url: String,
    /// Stable per-user key. Never serialize this to frontend status.
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Serialize)]
pub struct LlmConfigStatus {
    pub app_id: String,
    pub supported: bool,
    pub configured: bool,
    pub provider: String,
    pub model: String,
    pub base_url: Option<String>,
    pub key_hint: Option<String>,
    pub restart_required: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct LlmConfigResult {
    pub ok: bool,
    pub app_id: String,
    pub configured: bool,
    pub provider: String,
    pub model: String,
    pub backup_path: Option<String>,
    pub restart_required: bool,
    pub message: String,
}

pub fn adapter_supported(adapter: &str) -> bool {
    matches!(adapter, "opencode" | "pi")
}

pub fn status(
    app_id: &str,
    adapter: &str,
    provider: &TytusAilProvider,
) -> Result<LlmConfigStatus, String> {
    match adapter {
        "opencode" => opencode_status(app_id, provider),
        "pi" => pi_status(app_id, provider),
        other => Ok(unsupported_status(app_id, other, provider)),
    }
}

pub fn configure(
    app_id: &str,
    adapter: &str,
    provider: &TytusAilProvider,
) -> Result<LlmConfigResult, String> {
    match adapter {
        "opencode" => configure_opencode(app_id, provider),
        "pi" => configure_pi(app_id, provider),
        other => Err(format!(
            "No safe Tytus AIL adapter is available for {other} yet."
        )),
    }
}

fn unsupported_status(app_id: &str, adapter: &str, provider: &TytusAilProvider) -> LlmConfigStatus {
    LlmConfigStatus {
        app_id: app_id.to_string(),
        supported: false,
        configured: false,
        provider: TYTUS_PROVIDER_ID.to_string(),
        model: provider.model.clone(),
        base_url: None,
        key_hint: None,
        restart_required: true,
        message: format!("No safe Tytus AIL adapter is available for {adapter} yet."),
    }
}

fn key_hint(key: &str) -> String {
    let tail_len = key.chars().count().min(8);
    let tail: String = key
        .chars()
        .rev()
        .take(tail_len)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("…{tail}")
}

fn home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "home directory not found".to_string())
}

fn epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn ensure_parent(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create config directory failed: {e}"))?;
    }
    Ok(())
}

fn backup_existing(path: &Path, stem: &str) -> Result<Option<PathBuf>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let backup = path.with_extension(format!("{stem}.tytus-backup-{}.json", epoch_secs()));
    fs::copy(path, &backup).map_err(|e| format!("backup failed: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&backup, fs::Permissions::from_mode(0o600));
    }
    Ok(Some(backup))
}

fn write_atomic(path: &Path, content: &str) -> Result<(), String> {
    ensure_parent(path)?;
    let tmp = path.with_extension(format!("tmp-{}", epoch_secs()));
    fs::write(&tmp, content).map_err(|e| format!("write temp config failed: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600));
    }
    fs::rename(&tmp, path).map_err(|e| format!("replace config failed: {e}"))?;
    Ok(())
}

fn read_json_object(path: &Path) -> Result<Value, String> {
    if !path.exists() {
        return Ok(json!({}));
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("read config failed: {e}"))?;
    let parsed: Value =
        serde_json::from_str(&raw).map_err(|e| format!("config is not valid JSON: {e}"))?;
    if !parsed.is_object() {
        return Err("config root must be a JSON object".to_string());
    }
    Ok(parsed)
}

fn object_mut<'a>(root: &'a mut Value, key: &str) -> &'a mut Map<String, Value> {
    if !root.get(key).map(|v| v.is_object()).unwrap_or(false) {
        root[key] = json!({});
    }
    root.get_mut(key).unwrap().as_object_mut().unwrap()
}

fn opencode_config_path() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".config/opencode/opencode.json"))
}

fn opencode_provider_value(provider: &TytusAilProvider) -> Value {
    json!({
        "name": "Tytus AIL",
        "npm": "@ai-sdk/openai-compatible",
        "options": { "baseURL": provider.base_url, "apiKey": provider.api_key },
        "models": {
            provider.model.clone(): {
                "name": "Tytus AIL Compound",
                "attachment": true,
                "reasoning": false,
                "tool_call": true
            }
        }
    })
}

fn opencode_status(app_id: &str, provider: &TytusAilProvider) -> Result<LlmConfigStatus, String> {
    let config = read_json_object(&opencode_config_path()?)?;
    let provider_node = config
        .get("provider")
        .and_then(|p| p.get(TYTUS_PROVIDER_ID));
    let base_ok = provider_node
        .and_then(|p| p.get("options"))
        .and_then(|o| o.get("baseURL"))
        .and_then(|v| v.as_str())
        == Some(provider.base_url.as_str());
    let key_ok = provider_node
        .and_then(|p| p.get("options"))
        .and_then(|o| o.get("apiKey"))
        .and_then(|v| v.as_str())
        == Some(provider.api_key.as_str());
    let desired_model = format!("{TYTUS_PROVIDER_ID}/{}", provider.model);
    let model_ok = config.get("model").and_then(|m| m.as_str()) == Some(desired_model.as_str());
    let configured = base_ok && key_ok && model_ok;
    Ok(LlmConfigStatus {
        app_id: app_id.to_string(),
        supported: true,
        configured,
        provider: TYTUS_PROVIDER_ID.to_string(),
        model: provider.model.clone(),
        base_url: provider_node
            .and_then(|p| p.get("options"))
            .and_then(|o| o.get("baseURL"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
        key_hint: if key_ok {
            Some(key_hint(&provider.api_key))
        } else {
            None
        },
        restart_required: true,
        message: if configured {
            "Tytus AIL is configured as the default OpenCode model."
        } else {
            "OpenCode can be configured with Tytus AIL."
        }
        .to_string(),
    })
}

fn configure_opencode(
    app_id: &str,
    provider: &TytusAilProvider,
) -> Result<LlmConfigResult, String> {
    let path = opencode_config_path()?;
    let mut config = read_json_object(&path)?;
    let backup = backup_existing(&path, "opencode")?;
    if config.get("$schema").is_none() {
        config["$schema"] = json!("https://opencode.ai/config.json");
    }
    object_mut(&mut config, "provider").insert(
        TYTUS_PROVIDER_ID.to_string(),
        opencode_provider_value(provider),
    );
    config["model"] = json!(format!("{TYTUS_PROVIDER_ID}/{}", provider.model));
    let rendered =
        serde_json::to_string_pretty(&config).map_err(|e| format!("render config failed: {e}"))?;
    write_atomic(&path, &(rendered + "\n"))?;
    Ok(LlmConfigResult {
        ok: true,
        app_id: app_id.to_string(),
        configured: true,
        provider: TYTUS_PROVIDER_ID.to_string(),
        model: provider.model.clone(),
        backup_path: backup.map(|p| p.display().to_string()),
        restart_required: true,
        message: "Tytus AIL is now the default provider for OpenCode.".to_string(),
    })
}

fn pi_settings_path() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".pi/agent/settings.json"))
}

fn pi_extension_path() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".pi/agent/extensions/tytus-ail/index.ts"))
}

fn pi_extension(provider: &TytusAilProvider) -> String {
    format!(
        r#"// Auto-generated by Tytus. Safe to delete; App Store can recreate it.
import type {{ ExtensionAPI }} from "@mariozechner/pi-coding-agent";

export default function (pi: ExtensionAPI) {{
  pi.registerProvider("tytus-ail", {{
    baseUrl: "{base_url}",
    apiKey: "{api_key}",
    models: [
      {{
        id: "{model}",
        name: "Tytus AIL Compound",
        input: ["text", "image"],
        output: ["text"],
        reasoning: false,
        cache: false,
        cost: {{ input: 0, output: 0, cacheRead: 0, cacheWrite: 0 }},
        contextWindow: 204800,
        maxTokens: 65536
      }}
    ]
  }});
}}
"#,
        base_url = provider.base_url,
        api_key = provider.api_key,
        model = provider.model,
    )
}

fn pi_status(app_id: &str, provider: &TytusAilProvider) -> Result<LlmConfigStatus, String> {
    let settings = read_json_object(&pi_settings_path()?).unwrap_or_else(|_| json!({}));
    let ext = pi_extension_path()?;
    let ext_ok = ext.exists()
        && fs::read_to_string(&ext)
            .map(|s| s.contains(&provider.base_url) && s.contains("registerProvider(\"tytus-ail\""))
            .unwrap_or(false);
    let settings_ok = settings.get("defaultProvider").and_then(|x| x.as_str())
        == Some(TYTUS_PROVIDER_ID)
        && settings.get("defaultModel").and_then(|x| x.as_str()) == Some(provider.model.as_str());
    Ok(LlmConfigStatus {
        app_id: app_id.to_string(),
        supported: true,
        configured: ext_ok && settings_ok,
        provider: TYTUS_PROVIDER_ID.to_string(),
        model: provider.model.clone(),
        base_url: if ext_ok {
            Some(provider.base_url.clone())
        } else {
            None
        },
        key_hint: if ext_ok {
            Some(key_hint(&provider.api_key))
        } else {
            None
        },
        restart_required: true,
        message: if ext_ok && settings_ok {
            "Tytus AIL is configured as the default Pi provider."
        } else {
            "Pi can be configured with a Tytus AIL extension."
        }
        .to_string(),
    })
}

fn configure_pi(app_id: &str, provider: &TytusAilProvider) -> Result<LlmConfigResult, String> {
    let settings_path = pi_settings_path()?;
    let ext_path = pi_extension_path()?;
    let mut settings = read_json_object(&settings_path)?;
    let settings_backup = backup_existing(&settings_path, "settings")?;
    let ext_backup = backup_existing(&ext_path, "tytus-ail")?;

    let packages = settings.get_mut("packages");
    match packages.and_then(|v| v.as_array_mut()) {
        Some(arr) => {
            if !arr.iter().any(|v| v.as_str() == Some(PI_SWITCHAI_PACKAGE)) {
                arr.push(json!(PI_SWITCHAI_PACKAGE));
            }
        }
        None => settings["packages"] = json!([PI_SWITCHAI_PACKAGE]),
    }
    settings["defaultProvider"] = json!(TYTUS_PROVIDER_ID);
    settings["defaultModel"] = json!(provider.model);
    let rendered = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("render Pi settings failed: {e}"))?;
    write_atomic(&ext_path, &pi_extension(provider))?;
    write_atomic(&settings_path, &(rendered + "\n"))?;

    Ok(LlmConfigResult {
        ok: true,
        app_id: app_id.to_string(),
        configured: true,
        provider: TYTUS_PROVIDER_ID.to_string(),
        model: provider.model.clone(),
        backup_path: settings_backup
            .or(ext_backup)
            .map(|p| p.display().to_string()),
        restart_required: true,
        message: "Tytus AIL is now the default provider for Pi.".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_support_is_allowlisted() {
        assert!(adapter_supported("opencode"));
        assert!(adapter_supported("pi"));
        assert!(!adapter_supported("../opencode"));
        assert!(!adapter_supported("unknown"));
    }

    #[test]
    fn key_hint_never_returns_full_key() {
        assert_eq!(key_hint("sk-tytus-user-abcdef1234567890"), "…34567890");
    }

    #[test]
    fn opencode_provider_has_openai_compatible_shape() {
        let provider = TytusAilProvider {
            base_url: "http://10.42.42.1:18080/v1".into(),
            api_key: "sk-test".into(),
            model: TYTUS_MODEL_ID.into(),
        };
        let v = opencode_provider_value(&provider);
        assert_eq!(v["npm"], "@ai-sdk/openai-compatible");
        assert_eq!(v["options"]["baseURL"], "http://10.42.42.1:18080/v1");
        assert!(v["models"][TYTUS_MODEL_ID].is_object());
    }

    #[test]
    fn pi_extension_registers_tytus_provider() {
        let provider = TytusAilProvider {
            base_url: "http://10.42.42.1:18080/v1".into(),
            api_key: "sk-test".into(),
            model: TYTUS_MODEL_ID.into(),
        };
        let ext = pi_extension(&provider);
        assert!(ext.contains("@mariozechner/pi-coding-agent"));
        assert!(ext.contains("registerProvider(\"tytus-ail\""));
        assert!(ext.contains("ail-compound"));
    }
}

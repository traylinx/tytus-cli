//! Desktop LLM provider configuration helpers for the App Store.
//!
//! Narrow by design: app ids come from the embedded catalog, adapters own
//! fixed config paths, and request bodies never choose filesystem locations.

use serde::Serialize;
use serde_json::{json, Map, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
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
    matches!(adapter, "opencode" | "pi" | "odysseus")
}

pub fn status(
    app_id: &str,
    adapter: &str,
    provider: &TytusAilProvider,
) -> Result<LlmConfigStatus, String> {
    match adapter {
        "opencode" => opencode_status(app_id, provider),
        "pi" => pi_status(app_id, provider),
        "odysseus" => odysseus_status(app_id, provider),
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
        "odysseus" => configure_odysseus(app_id, provider),
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

fn odysseus_candidate_roots() -> Result<Vec<PathBuf>, String> {
    let home = home_dir()?;
    let mut roots = Vec::new();
    #[cfg(target_os = "macos")]
    {
        for app in [
            PathBuf::from("/Applications/Odysseus.app/Contents/MacOS/Odysseus"),
            home.join("Applications/Odysseus.app/Contents/MacOS/Odysseus"),
        ] {
            if let Ok(raw) = fs::read_to_string(&app) {
                if let Some(line) = raw.lines().find(|l| l.starts_with("INSTALL_DIR=")) {
                    let value = line
                        .trim_start_matches("INSTALL_DIR=")
                        .trim()
                        .trim_matches('"')
                        .to_string();
                    if !value.is_empty() {
                        roots.push(PathBuf::from(value));
                    }
                }
            }
        }
    }
    roots.push(home.join("Tytus/ExternalApps/odysseus"));
    roots.push(PathBuf::from(
        "/Users/sebastian/projects/makakoo/agents/sample_apps/odysseus",
    ));
    let mut deduped = Vec::new();
    for root in roots {
        if !deduped.contains(&root) {
            deduped.push(root);
        }
    }
    let roots = deduped;
    Ok(roots)
}

fn odysseus_root() -> Result<Option<PathBuf>, String> {
    Ok(odysseus_candidate_roots()?
        .into_iter()
        .find(|p| p.join("app.py").exists() && p.join("data").exists()))
}

fn run_python_json(
    script: &str,
    root: &Path,
    provider: &TytusAilProvider,
) -> Result<Value, String> {
    let mut child = Command::new("python3")
        .arg("-c")
        .arg(script)
        .env("ODYSSEUS_ROOT", root)
        .env("TYTUS_BASE_URL", &provider.base_url)
        .env("TYTUS_API_KEY", &provider.api_key)
        .env("TYTUS_MODEL", &provider.model)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("python3 is required to configure Odysseus: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(b"")
            .map_err(|e| format!("failed to close Python stdin: {e}"))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|e| format!("Odysseus configuration helper failed: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "Odysseus configuration helper failed".to_string()
        } else {
            stderr
        });
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Odysseus helper returned invalid JSON: {e}"))
}

const ODYSSEUS_STATUS_SCRIPT: &str = r#"
import json, os, sqlite3
root=os.environ["ODYSSEUS_ROOT"]
db_path=os.path.join(root,"data","app.db")
settings_path=os.path.join(root,"data","settings.json")
base=os.environ["TYTUS_BASE_URL"]
key=os.environ["TYTUS_API_KEY"]
model=os.environ["TYTUS_MODEL"]
out={"db_exists":os.path.exists(db_path),"configured":False,"base_url":None,"key_ok":False,"endpoint_id":None}
settings={}
if os.path.exists(settings_path):
    try:
        settings=json.load(open(settings_path))
    except Exception:
        settings={}
if os.path.exists(db_path):
    con=sqlite3.connect(db_path)
    con.row_factory=sqlite3.Row
    try:
        rows=con.execute("select id,name,base_url,api_key from model_endpoints").fetchall()
        match=None
        for row in rows:
            if row["id"]=="tytusail" or row["name"]=="Tytus AIL" or row["base_url"]==base:
                match=row
                if row["base_url"]==base:
                    break
        if match:
            out["endpoint_id"]=match["id"]
            out["base_url"]=match["base_url"]
            stored_key=match["api_key"] or ""
            # Odysseus encrypts plaintext provider keys on startup with an
            # enc: prefix. Treat encrypted Tytus rows as configured; the app
            # owns decryption and will use the stored secret.
            out["key_ok"]=(stored_key==key or stored_key.startswith("enc:"))
            out["configured"]=(match["base_url"]==base and out["key_ok"] and settings.get("default_endpoint_id")==match["id"] and settings.get("default_model")==model)
    except sqlite3.Error as e:
        out["error"]=str(e)
    finally:
        con.close()
print(json.dumps(out))
"#;

const ODYSSEUS_CONFIGURE_SCRIPT: &str = r#"
import json, os, shutil, sqlite3, time
from datetime import datetime
root=os.environ["ODYSSEUS_ROOT"]
db_path=os.path.join(root,"data","app.db")
settings_path=os.path.join(root,"data","settings.json")
base=os.environ["TYTUS_BASE_URL"]
key=os.environ["TYTUS_API_KEY"]
model=os.environ["TYTUS_MODEL"]
if not os.path.exists(db_path):
    raise SystemExit("Odysseus data/app.db does not exist. Install or launch Odysseus once, then configure Tytus AIL.")
backup=None
if os.path.exists(settings_path):
    backup=settings_path+f".tytus-backup-{int(time.time())}.json"
    shutil.copy2(settings_path, backup)
con=sqlite3.connect(db_path)
con.row_factory=sqlite3.Row
try:
    cols={r[1] for r in con.execute("PRAGMA table_info(model_endpoints)").fetchall()}
    if not cols:
        raise SystemExit("Odysseus model_endpoints table does not exist. Run Odysseus setup first.")
    now=datetime.utcnow().strftime("%Y-%m-%d %H:%M:%S.%f")
    cached=json.dumps(["ail-compound","ail-fast"])
    pinned=json.dumps(["ail-compound","ail-fast"])
    hidden=json.dumps(["ail-transcribe","ail-speech","ail-image","ail-music-cover","ail-music"])
    values={
      "id":"tytusail","name":"Tytus AIL","base_url":base,"api_key":key,"is_enabled":1,
      "hidden_models":hidden,"cached_models":cached,"pinned_models":pinned,
      "model_type":"llm","endpoint_kind":"proxy","model_refresh_mode":"manual",
      "model_refresh_interval":None,"model_refresh_timeout":15,"supports_tools":1,
      "owner":None,"created_at":now,"updated_at":now,
    }
    row=con.execute("select id from model_endpoints where id=? or name=? or base_url=? order by case when id=? then 0 else 1 end limit 1", ("tytusail","Tytus AIL",base,"tytusail")).fetchone()
    if row:
        ep_id=row["id"]
        updates=[k for k in values if k in cols and k not in ("id","created_at")]
        con.execute("update model_endpoints set "+", ".join(f"{k}=?" for k in updates)+" where id=?", [values[k] for k in updates]+[ep_id])
    else:
        insert=[k for k in values if k in cols]
        ep_id="tytusail"
        con.execute("insert into model_endpoints ("+", ".join(insert)+") values ("+", ".join("?" for _ in insert)+")", [values[k] for k in insert])
    con.commit()
finally:
    con.close()
settings={}
if os.path.exists(settings_path):
    try:
        settings=json.load(open(settings_path))
    except Exception:
        settings={}
settings.update({
  "default_endpoint_id": ep_id,
  "default_model": model,
  "utility_endpoint_id": ep_id,
  "utility_model": "ail-fast",
  "research_endpoint_id": ep_id,
  "research_model": model,
  "task_endpoint_id": ep_id,
  "task_model": model,
  "default_model_fallbacks": [{"endpoint_id": ep_id, "model": "ail-fast"}],
  "utility_model_fallbacks": [{"endpoint_id": ep_id, "model": model}],
})
os.makedirs(os.path.dirname(settings_path), exist_ok=True)
tmp=settings_path+".tmp"
with open(tmp,"w") as f:
    json.dump(settings,f,indent=2)
    f.write("\n")
os.replace(tmp, settings_path)
print(json.dumps({"configured":True,"endpoint_id":ep_id,"backup_path":backup}))
"#;

fn odysseus_status(app_id: &str, provider: &TytusAilProvider) -> Result<LlmConfigStatus, String> {
    let Some(root) = odysseus_root()? else {
        return Ok(LlmConfigStatus {
            app_id: app_id.to_string(),
            supported: true,
            configured: false,
            provider: TYTUS_PROVIDER_ID.to_string(),
            model: provider.model.clone(),
            base_url: None,
            key_hint: None,
            restart_required: true,
            message: "Install Odysseus first, then Tytus can add the Tytus AIL provider."
                .to_string(),
        });
    };
    let status = run_python_json(ODYSSEUS_STATUS_SCRIPT, &root, provider)?;
    let configured = status
        .get("configured")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let key_ok = status
        .get("key_ok")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    Ok(LlmConfigStatus {
        app_id: app_id.to_string(),
        supported: true,
        configured,
        provider: TYTUS_PROVIDER_ID.to_string(),
        model: provider.model.clone(),
        base_url: status
            .get("base_url")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        key_hint: if key_ok {
            Some(key_hint(&provider.api_key))
        } else {
            None
        },
        restart_required: true,
        message: if configured {
            "Tytus AIL is configured as the default Odysseus provider."
        } else if !status
            .get("db_exists")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            "Odysseus is installed, but data/app.db is missing. Launch Odysseus once, then configure Tytus AIL."
        } else {
            "Odysseus can be configured with Tytus AIL."
        }
        .to_string(),
    })
}

fn configure_odysseus(
    app_id: &str,
    provider: &TytusAilProvider,
) -> Result<LlmConfigResult, String> {
    let root = odysseus_root()?.ok_or_else(|| {
        "Install Odysseus first, then Tytus can add the Tytus AIL provider.".to_string()
    })?;
    let result = run_python_json(ODYSSEUS_CONFIGURE_SCRIPT, &root, provider)?;
    Ok(LlmConfigResult {
        ok: true,
        app_id: app_id.to_string(),
        configured: result
            .get("configured")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        provider: TYTUS_PROVIDER_ID.to_string(),
        model: provider.model.clone(),
        backup_path: result
            .get("backup_path")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        restart_required: true,
        message: "Tytus AIL is now the default provider for Odysseus. Restart Odysseus if it is already open.".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_support_is_allowlisted() {
        assert!(adapter_supported("opencode"));
        assert!(adapter_supported("pi"));
        assert!(adapter_supported("odysseus"));
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

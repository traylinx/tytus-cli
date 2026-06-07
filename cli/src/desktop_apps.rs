//! `tytus desktop` — launch native desktop apps from the App Store
//! "Desktop" catalog (the same `apps.json` the tray daemon serves at
//! `/api/apps`). This is distinct from `tytus app`, which manages Tytus OS
//! *web* apps (the Featured catalog / manifest URLs).
//!
//! The catalog is embedded at compile time so the command works without a
//! running daemon. Launch dispatch mirrors the daemon's `/api/apps/open`:
//!   - `kind:"app"`      → launch the GUI app by name (macOS `open -a`,
//!                          Linux/Windows spawn the binary).
//!   - `kind:"terminal"` → open a new terminal running the CLI.
//! Only embedded catalog data reaches the launcher — never arbitrary input.

/// Single source of truth: the same file the tray daemon embeds + serves.
const APPS_JSON: &str = include_str!("../../tray/web/assets/apps.json");

fn platform() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    }
}

fn parse_catalog() -> Vec<serde_json::Value> {
    serde_json::from_str(APPS_JSON).unwrap_or_else(|e| {
        eprintln!("tytus desktop: embedded app catalog is corrupt: {e}");
        std::process::exit(1);
    })
}

/// Detection mirrors the daemon's `check_app_installed`: on macOS look for the
/// app bundle in /Applications and ~/Applications, then fall back to a PATH
/// probe; on Linux probe PATH only.
fn is_installed(entry: &serde_json::Value, platform: &str) -> bool {
    let Some(detect) = entry
        .get("detect")
        .and_then(|d| d.get(platform))
        .and_then(|c| c.as_array())
    else {
        return false;
    };
    for cmd in detect {
        let Some(cmd_str) = cmd.as_str() else {
            continue;
        };
        #[cfg(target_os = "macos")]
        {
            if std::path::Path::new(&format!("/Applications/{cmd_str}.app")).exists() {
                return true;
            }
            if let Ok(home) = std::env::var("HOME") {
                if std::path::Path::new(&format!("{home}/Applications/{cmd_str}.app")).exists() {
                    return true;
                }
            }
        }
        if command_exists_for_desktop_app(cmd_str) {
            return true;
        }
    }
    false
}

fn command_exists_for_desktop_app(cmd: &str) -> bool {
    if cmd.trim().is_empty() || cmd.contains('/') {
        return false;
    }
    let mut dirs: Vec<std::path::PathBuf> = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default();

    if let Ok(home) = std::env::var("HOME") {
        let home = std::path::PathBuf::from(home);
        dirs.push(home.join("bin"));
        dirs.push(home.join(".local/bin"));
        let nvm_versions = home.join(".nvm/versions/node");
        if let Ok(entries) = std::fs::read_dir(nvm_versions) {
            for entry in entries.flatten() {
                dirs.push(entry.path().join("bin"));
            }
        }
    }
    dirs.push(std::path::PathBuf::from("/usr/local/bin"));
    dirs.push(std::path::PathBuf::from("/opt/homebrew/bin"));

    dirs.into_iter().any(|dir| {
        dir.join(cmd).is_file() || {
            #[cfg(target_os = "windows")]
            {
                dir.join(format!("{cmd}.exe")).is_file()
                    || dir.join(format!("{cmd}.cmd")).is_file()
                    || dir.join(format!("{cmd}.bat")).is_file()
            }
            #[cfg(not(target_os = "windows"))]
            {
                false
            }
        }
    })
}

fn resolve_launch_spec<'a>(
    entry: &'a serde_json::Value,
    platform: &str,
) -> Result<(&'a str, &'a str), String> {
    let spec = entry
        .get("launch")
        .and_then(|l| l.get(platform))
        .ok_or_else(|| format!("no launch spec for platform {platform}"))?;
    let kind = spec.get("kind").and_then(|k| k.as_str()).unwrap_or("");
    let target = spec.get("target").and_then(|t| t.as_str()).unwrap_or("");
    if target.is_empty() {
        return Err("empty launch target".to_string());
    }
    if kind != "app" && kind != "terminal" {
        return Err(format!("unknown launch kind: {kind}"));
    }
    Ok((kind, target))
}

fn launch(entry: &serde_json::Value, platform: &str) -> Result<(), String> {
    let (kind, target) = resolve_launch_spec(entry, platform)?;
    match kind {
        "app" => {
            atomek_core::platform::open::open_app(target).map_err(|e| format!("launch failed: {e}"))
        }
        "terminal" => atomek_core::platform::terminal::open_shell_command(target)
            .map_err(|e| format!("terminal launch failed: {e}")),
        _ => unreachable!("kind validated by resolve_launch_spec"),
    }
}

pub fn cmd_desktop(action: crate::DesktopAction, json: bool) {
    let catalog = parse_catalog();
    let platform = platform();

    match action {
        crate::DesktopAction::List => {
            if json {
                let apps: Vec<serde_json::Value> = catalog
                    .iter()
                    .map(|e| {
                        serde_json::json!({
                            "id": e["id"].as_str().unwrap_or(""),
                            "name": e["name"].as_str().unwrap_or(""),
                            "category": e["category"].as_str().unwrap_or(""),
                            "installed": is_installed(e, platform),
                        })
                    })
                    .collect();
                println!("{}", serde_json::json!({ "apps": apps }));
                return;
            }
            println!("Desktop apps ({platform}):");
            for e in &catalog {
                let id = e["id"].as_str().unwrap_or("");
                let name = e["name"].as_str().unwrap_or(id);
                let mark = if is_installed(e, platform) {
                    "✓"
                } else {
                    "·"
                };
                println!("  {mark} {id:<12} {name}");
            }
            println!("\nOpen one:  tytus desktop open <id>");
            println!("Open all:  tytus desktop open --all");
        }

        crate::DesktopAction::Open { id, all } => {
            if all {
                let mut opened: Vec<String> = Vec::new();
                let mut skipped: Vec<(String, String)> = Vec::new();
                for e in &catalog {
                    let app_id = e["id"].as_str().unwrap_or("").to_string();
                    if app_id.is_empty() {
                        continue;
                    }
                    if !is_installed(e, platform) {
                        skipped.push((app_id, "not installed".to_string()));
                        continue;
                    }
                    match launch(e, platform) {
                        Ok(()) => opened.push(app_id),
                        Err(reason) => skipped.push((app_id, reason)),
                    }
                }
                if json {
                    let skipped_json: Vec<serde_json::Value> = skipped
                        .iter()
                        .map(|(id, reason)| serde_json::json!({ "id": id, "reason": reason }))
                        .collect();
                    println!(
                        "{}",
                        serde_json::json!({ "ok": true, "opened": opened, "skipped": skipped_json })
                    );
                } else {
                    if opened.is_empty() {
                        println!("No installed desktop apps to open.");
                    } else {
                        println!("Opened: {}", opened.join(", "));
                    }
                    for (id, reason) in &skipped {
                        println!("  skipped {id}: {reason}");
                    }
                }
                return;
            }

            let Some(id) = id else {
                eprintln!("tytus desktop open: specify an app id (e.g. `discord`) or --all");
                std::process::exit(2);
            };
            let Some(entry) = catalog
                .iter()
                .find(|e| e["id"].as_str() == Some(id.as_str()))
            else {
                eprintln!("tytus desktop open: unknown app `{id}` (try `tytus desktop list`)");
                std::process::exit(1);
            };
            if !is_installed(entry, platform) {
                eprintln!("tytus desktop open: `{id}` is not installed");
                std::process::exit(1);
            }
            match launch(entry, platform) {
                Ok(()) => {
                    if json {
                        println!("{}", serde_json::json!({ "ok": true, "id": id }));
                    } else {
                        println!("Opened {id}.");
                    }
                }
                Err(reason) => {
                    eprintln!("tytus desktop open: {reason}");
                    std::process::exit(1);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_catalog_parses_and_every_entry_is_launchable() {
        let catalog = parse_catalog();
        assert!(!catalog.is_empty());
        for e in &catalog {
            let id = e["id"].as_str().unwrap_or("<no-id>");
            let platforms = e["platforms"].as_array().expect("platforms must be array");
            for platform in platforms.iter().filter_map(|p| p.as_str()) {
                assert!(
                    resolve_launch_spec(e, platform).is_ok(),
                    "{id}: launch.{platform} must resolve"
                );
            }
        }
    }

    #[test]
    fn resolve_launch_spec_rejects_bad_specs() {
        let bad = serde_json::json!({ "launch": { "macos": { "kind": "x", "target": "y" } } });
        assert!(resolve_launch_spec(&bad, "macos").is_err());
        let empty = serde_json::json!({ "launch": { "macos": { "kind": "app", "target": "" } } });
        assert!(resolve_launch_spec(&empty, "macos").is_err());
    }

    #[test]
    fn command_lookup_rejects_paths_and_empty_values() {
        assert!(!command_exists_for_desktop_app(""));
        assert!(!command_exists_for_desktop_app("../pi"));
        assert!(!command_exists_for_desktop_app("/usr/bin/open"));
    }
}

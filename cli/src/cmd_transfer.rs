// ============================================================
// cmd_transfer — push/pull/ls/rm/transfers command implementations
// ============================================================
// Chunked base64 over existing `tytus exec` pipeline; no new
// infrastructure. Sub-MB transfers stay silent; > 1 MB shows a
// progress bar on stderr. Every invocation appends one row to
// the transfer log (append_transfer_log).
// ============================================================

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use base64::Engine as _;

use crate::state::CliState;
use crate::transfer::{
    append_transfer_log, enforce_size_ceiling, resolve_pod, resolve_push_destination,
    shell_escape, validate_pod_path, TransferError, TransferEvent, CHUNK_PAYLOAD_BYTES,
    POD_INBOX, PROGRESS_THRESHOLD_BYTES,
};

// ── Shared auth bootstrap (matches cmd_exec pattern) ────────

async fn bootstrap_client(
    http: &atomek_core::HttpClient,
) -> Option<(CliState, atomek_pods::TytusClient)> {
    let mut state = CliState::load();
    if !state.is_logged_in() {
        eprintln!("Not logged in. Run: tytus login");
        return None;
    }
    if let Err(e) = crate::ensure_token(&mut state, http).await {
        eprintln!("Token refresh failed: {}. Run: tytus login", e);
        return None;
    }
    let (sk, auid) = crate::get_credentials(&mut state, http).await;
    Some((state, atomek_pods::TytusClient::new(http, &sk, &auid)))
}

// ── Remote exec wrapper ─────────────────────────────────────

async fn pod_exec(
    client: &atomek_pods::TytusClient,
    pod_id: &str,
    command: &str,
    timeout: u32,
) -> Result<atomek_pods::ExecResult, String> {
    atomek_pods::exec_in_agent(client, pod_id, command, timeout)
        .await
        .map_err(|e| e.to_string())
}

// ── Progress reporter (stderr, respects --quiet) ────────────

fn make_progress(total: u64, quiet: bool) -> Option<indicatif::ProgressBar> {
    if quiet || total < PROGRESS_THRESHOLD_BYTES {
        return None;
    }
    let pb = indicatif::ProgressBar::new(total);
    pb.set_style(
        indicatif::ProgressStyle::with_template(
            "{bytes}/{total_bytes} {bar:30} {bytes_per_sec} ETA {eta}",
        )
        .unwrap()
        .progress_chars("=> "),
    );
    pb.set_draw_target(indicatif::ProgressDrawTarget::stderr());
    Some(pb)
}

// ── PUSH ────────────────────────────────────────────────────

/// `tytus push <LOCAL> [--pod NN] [--to /app/workspace/PATH]`.
///
/// - Files: single-blob base64 upload, chunked if larger than
///   CHUNK_PAYLOAD_BYTES.
/// - Directories: locally packed into a tarball first
///   (tar+gzip via the host `tar` binary), then uploaded as a
///   file blob and extracted on the pod side.
pub async fn cmd_push(
    http: &atomek_core::HttpClient,
    local: String,
    pod: Option<String>,
    to: Option<String>,
    quiet: bool,
    json: bool,
) {
    let local_path = PathBuf::from(&local);
    if !local_path.exists() {
        log_and_exit("push", "?", to.as_deref().unwrap_or(POD_INBOX), Some(&local),
            0, false, &format!("local path does not exist: {}", local), json);
    }

    let Some((state, client)) = bootstrap_client(http).await else {
        std::process::exit(1);
    };
    let pod_id = match resolve_pod(pod.as_deref(), &state) {
        Ok(p) => p,
        Err(e) => log_and_exit("push", "?", to.as_deref().unwrap_or(POD_INBOX), Some(&local),
            0, false, &e.to_string(), json),
    };

    // If directory, pack to a local tarball and treat it as file
    // transfer with a tar extract on the remote finaliser.
    let is_dir = local_path.is_dir();
    let (payload_path, payload_cleanup, remote_dest): (PathBuf, bool, String) = if is_dir {
        match pack_dir_to_tarball(&local_path) {
            Ok(p) => {
                let dest_root = match resolve_dir_push_destination(&local_path, to.as_deref()) {
                    Ok(d) => d,
                    Err(e) => log_and_exit("push", &pod_id, to.as_deref().unwrap_or(POD_INBOX),
                        Some(&local), 0, false, &e.to_string(), json),
                };
                (p, true, dest_root)
            }
            Err(e) => log_and_exit("push", &pod_id, to.as_deref().unwrap_or(POD_INBOX),
                Some(&local), 0, false, &format!("tar pack failed: {}", e), json),
        }
    } else {
        let dest = match resolve_push_destination(&local_path, to.as_deref()) {
            Ok(d) => d,
            Err(e) => log_and_exit("push", &pod_id, to.as_deref().unwrap_or(POD_INBOX),
                Some(&local), 0, false, &e.to_string(), json),
        };
        (local_path.clone(), false, dest)
    };

    let size = match std::fs::metadata(&payload_path).map(|m| m.len()) {
        Ok(s) => s,
        Err(e) => log_and_exit("push", &pod_id, &remote_dest, Some(&local), 0, false,
            &format!("stat failed: {}", e), json),
    };
    if let Err(e) = enforce_size_ceiling(size) {
        if payload_cleanup {
            let _ = std::fs::remove_file(&payload_path);
        }
        log_and_exit("push", &pod_id, &remote_dest, Some(&local), size, false,
            &e.to_string(), json);
    }

    // Perform the chunked upload.
    let nonce = random_nonce();
    let remote_tmp = format!("/app/workspace/.tytus-push-{}.b64", nonce);
    let pb = make_progress(size, quiet);

    let upload_result = upload_chunked(&client, &pod_id, &payload_path, &remote_tmp, pb.as_ref()).await;
    if let Some(ref p) = pb { p.finish_and_clear(); }

    if payload_cleanup {
        let _ = std::fs::remove_file(&payload_path);
    }

    if let Err(e) = upload_result {
        let _ = pod_exec(&client, &pod_id, &format!("rm -f {}", shell_escape(&remote_tmp)), 30).await;
        log_and_exit("push", &pod_id, &remote_dest, Some(&local), size, false, &e, json);
    }

    // Finalise: decode + (if dir) untar.
    let finalise_cmd = if is_dir {
        format!(
            "mkdir -p {dest} && base64 -d < {tmp} | tar xzf - -C {dest} && rm -f {tmp}",
            dest = shell_escape(&remote_dest),
            tmp = shell_escape(&remote_tmp),
        )
    } else {
        let parent = parent_dir(&remote_dest);
        format!(
            "mkdir -p {parent} && base64 -d < {tmp} > {dest} && rm -f {tmp}",
            parent = shell_escape(&parent),
            dest = shell_escape(&remote_dest),
            tmp = shell_escape(&remote_tmp),
        )
    };

    match pod_exec(&client, &pod_id, &finalise_cmd, 120).await {
        Ok(r) if r.exit_code == 0 => {
            let _ = append_transfer_log(&TransferEvent::now(
                "push", &pod_id, &remote_dest, Some(&local), size, true, None,
            ));
            if json {
                println!("{}", serde_json::json!({
                    "ok": true, "verb": "push", "pod": pod_id,
                    "remote": remote_dest, "size_bytes": size,
                }));
            } else {
                eprintln!("pushed {} → pod-{}:{} ({} bytes)", local, pod_id, remote_dest, size);
            }
        }
        Ok(r) => {
            let err = format!(
                "remote finalise failed (exit {}): {}{}",
                r.exit_code,
                r.stderr.clone().unwrap_or_default(),
                r.stdout.clone().unwrap_or_default(),
            );
            log_and_exit("push", &pod_id, &remote_dest, Some(&local), size, false, &err, json);
        }
        Err(e) => log_and_exit("push", &pod_id, &remote_dest, Some(&local), size, false, &e, json),
    }
}

async fn upload_chunked(
    client: &atomek_pods::TytusClient,
    pod_id: &str,
    local: &Path,
    remote_tmp: &str,
    pb: Option<&indicatif::ProgressBar>,
) -> Result<(), String> {
    let mut f = std::fs::File::open(local).map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; CHUNK_PAYLOAD_BYTES];
    let engine = base64::engine::general_purpose::STANDARD;
    let mut first = true;
    loop {
        let n = f.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 { break; }
        let b64 = engine.encode(&buf[..n]);
        let redirect = if first { ">" } else { ">>" };
        let cmd = format!(
            "printf %s {b64} {redir} {tmp}",
            b64 = shell_escape(&b64),
            redir = redirect,
            tmp = shell_escape(remote_tmp),
        );
        let r = pod_exec(client, pod_id, &cmd, 120).await?;
        if r.exit_code != 0 {
            return Err(format!(
                "chunk write failed (exit {}): {}",
                r.exit_code,
                r.stderr.unwrap_or_default(),
            ));
        }
        if let Some(p) = pb { p.inc(n as u64); }
        first = false;
    }
    Ok(())
}

fn pack_dir_to_tarball(dir: &Path) -> std::io::Result<PathBuf> {
    let parent = dir.parent().unwrap_or(Path::new("."));
    let name = dir.file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "bad dir name"))?;
    let tmp = tempfile::Builder::new()
        .prefix("tytus-push-")
        .suffix(".tgz")
        .tempfile()?;
    let tmp_path = tmp.path().to_path_buf();
    // Persist the tempfile so we can re-open it for reading
    // after tar writes to it. NamedTempFile::into_path consumes
    // the guard (caller must delete later).
    let _ = tmp.into_temp_path().keep().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    let status = std::process::Command::new("tar")
        .arg("czf")
        .arg(&tmp_path)
        .arg("-C")
        .arg(parent)
        .arg(name)
        .status()?;
    if !status.success() {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("tar exited with {}", status),
        ));
    }
    Ok(tmp_path)
}

fn resolve_dir_push_destination(
    local_dir: &Path,
    to: Option<&str>,
) -> Result<String, TransferError> {
    // For directory push, `to` is the PARENT on the pod. We
    // untar into it so the local dir name is preserved. Default
    // is the inbox. Explicit `--to` must end with `/` (points at
    // the container dir).
    let base = to.unwrap_or(POD_INBOX);
    validate_pod_path(base)?;
    let parent = if base.ends_with('/') {
        base.to_string()
    } else {
        format!("{}/", base)
    };
    // Sanity: record the eventual remote path for audit log.
    let name = local_dir
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| TransferError::LocalMissing(local_dir.display().to_string()))?;
    Ok(format!("{}{}", parent, name))
}

fn parent_dir(remote: &str) -> String {
    match remote.rfind('/') {
        Some(idx) if idx > 0 => remote[..idx].to_string(),
        _ => "/".to_string(),
    }
}

fn random_nonce() -> String {
    let ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    format!("{:x}-{:x}", ns, pid)
}

// ── PULL ────────────────────────────────────────────────────

pub async fn cmd_pull(
    http: &atomek_core::HttpClient,
    remote: String,
    pod: Option<String>,
    to: Option<String>,
    quiet: bool,
    json: bool,
) {
    if let Err(e) = validate_pod_path(&remote) {
        log_and_exit("pull", "?", &remote, None, 0, false, &e.to_string(), json);
    }

    let Some((state, client)) = bootstrap_client(http).await else {
        std::process::exit(1);
    };
    let pod_id = match resolve_pod(pod.as_deref(), &state) {
        Ok(p) => p,
        Err(e) => log_and_exit("pull", "?", &remote, None, 0, false, &e.to_string(), json),
    };

    // Step 1: figure out whether the remote is a file or dir.
    let stat_cmd = format!(
        "if [ -d {r} ]; then echo dir; elif [ -f {r} ]; then echo file; else echo missing; fi",
        r = shell_escape(&remote),
    );
    let kind = match pod_exec(&client, &pod_id, &stat_cmd, 30).await {
        Ok(r) if r.exit_code == 0 => r.stdout.unwrap_or_default().trim().to_string(),
        Ok(r) => log_and_exit("pull", &pod_id, &remote, None, 0, false,
            &format!("stat failed: {}", r.stderr.unwrap_or_default()), json),
        Err(e) => log_and_exit("pull", &pod_id, &remote, None, 0, false, &e, json),
    };
    if kind == "missing" {
        log_and_exit("pull", &pod_id, &remote, None, 0, false,
            &format!("remote path does not exist: {}", remote), json);
    }
    let is_dir = kind == "dir";

    // Step 2: pack on the pod side into /app/workspace/.tytus-pull-<nonce>.b64
    let nonce = random_nonce();
    let remote_tmp = format!("/app/workspace/.tytus-pull-{}.b64", nonce);
    let pack_cmd = if is_dir {
        let parent = parent_dir(&remote);
        let name = remote.trim_end_matches('/').rsplit('/').next().unwrap_or(".");
        format!(
            "tar cz -C {par} {name} | base64 > {tmp}",
            par = shell_escape(&parent),
            name = shell_escape(name),
            tmp = shell_escape(&remote_tmp),
        )
    } else {
        format!(
            "base64 {r} > {tmp}",
            r = shell_escape(&remote),
            tmp = shell_escape(&remote_tmp),
        )
    };
    if let Err(e) = run_ok(&client, &pod_id, &pack_cmd, 300).await {
        let _ = pod_exec(&client, &pod_id, &format!("rm -f {}", shell_escape(&remote_tmp)), 30).await;
        log_and_exit("pull", &pod_id, &remote, None, 0, false, &e, json);
    }

    // Size-check the base64 blob; raw size = b64 * 3/4 approx.
    let size_cmd = format!("wc -c < {}", shell_escape(&remote_tmp));
    let b64_bytes: u64 = match pod_exec(&client, &pod_id, &size_cmd, 30).await {
        Ok(r) if r.exit_code == 0 => r.stdout.unwrap_or_default().trim().parse().unwrap_or(0),
        _ => 0,
    };
    let raw_estimate = (b64_bytes / 4) * 3;
    if let Err(e) = enforce_size_ceiling(raw_estimate) {
        let _ = pod_exec(&client, &pod_id, &format!("rm -f {}", shell_escape(&remote_tmp)), 30).await;
        log_and_exit("pull", &pod_id, &remote, None, raw_estimate, false, &e.to_string(), json);
    }

    // Step 3: read chunks with dd (base64 block-size = 4, so
    // 4-byte alignment is required per chunk to avoid losing
    // trailing bytes). 262144 / 4 == 65536, divisible. Match
    // CHUNK_PAYLOAD_BYTES so tests and docs agree.
    let chunk_b64 = CHUNK_PAYLOAD_BYTES; // 262144 bytes of base64 per read
    let n_chunks = b64_bytes.div_ceil(chunk_b64 as u64);
    let local_target = resolve_pull_target(&remote, to.as_deref(), is_dir);
    if let Some(dir) = local_target.parent() {
        let _ = std::fs::create_dir_all(dir);
    }

    let pb = make_progress(raw_estimate, quiet);
    let engine = base64::engine::general_purpose::STANDARD;

    // Destination: for dir, write base64 into a .tgz tempfile
    // then untar. For file, decode straight to target.
    let mut tmp_local: Option<tempfile::NamedTempFile> = None;
    let mut out_file: Box<dyn Write> = if is_dir {
        let nt = tempfile::Builder::new().prefix("tytus-pull-").suffix(".tgz").tempfile().unwrap();
        let f = nt.reopen().unwrap();
        tmp_local = Some(nt);
        Box::new(f)
    } else {
        Box::new(std::fs::File::create(&local_target).unwrap_or_else(|e| {
            log_and_exit("pull", &pod_id, &remote, Some(&local_target.display().to_string()),
                raw_estimate, false, &format!("create local: {}", e), json);
        }))
    };

    for i in 0..n_chunks {
        let cmd = format!(
            "dd if={tmp} bs={bs} count=1 skip={i} 2>/dev/null",
            tmp = shell_escape(&remote_tmp),
            bs = chunk_b64,
            i = i,
        );
        let r = match pod_exec(&client, &pod_id, &cmd, 120).await {
            Ok(r) => r,
            Err(e) => {
                let _ = pod_exec(&client, &pod_id, &format!("rm -f {}", shell_escape(&remote_tmp)), 30).await;
                log_and_exit("pull", &pod_id, &remote, None, raw_estimate, false, &e, json);
            }
        };
        if r.exit_code != 0 {
            let _ = pod_exec(&client, &pod_id, &format!("rm -f {}", shell_escape(&remote_tmp)), 30).await;
            log_and_exit("pull", &pod_id, &remote, None, raw_estimate, false,
                &format!("chunk read failed (exit {}): {}", r.exit_code, r.stderr.unwrap_or_default()), json);
        }
        let stdout = r.stdout.unwrap_or_default();
        // Strip whitespace (base64 may wrap); decode sees raw b64 only.
        let compact: String = stdout.chars().filter(|c| !c.is_whitespace()).collect();
        let decoded = match engine.decode(compact.as_bytes()) {
            Ok(d) => d,
            Err(e) => {
                let _ = pod_exec(&client, &pod_id, &format!("rm -f {}", shell_escape(&remote_tmp)), 30).await;
                log_and_exit("pull", &pod_id, &remote, None, raw_estimate, false,
                    &format!("base64 decode: {}", e), json);
            }
        };
        if let Err(e) = out_file.write_all(&decoded) {
            let _ = pod_exec(&client, &pod_id, &format!("rm -f {}", shell_escape(&remote_tmp)), 30).await;
            log_and_exit("pull", &pod_id, &remote, None, raw_estimate, false, &format!("local write: {}", e), json);
        }
        if let Some(ref p) = pb { p.inc(decoded.len() as u64); }
    }
    drop(out_file);
    if let Some(ref p) = pb { p.finish_and_clear(); }

    // Untar if directory.
    if is_dir {
        let tgz = tmp_local.as_ref().map(|t| t.path().to_path_buf()).unwrap();
        let local_dir_parent = local_target.parent().unwrap_or(Path::new("."));
        let _ = std::fs::create_dir_all(local_dir_parent);
        let status = std::process::Command::new("tar")
            .arg("xzf")
            .arg(&tgz)
            .arg("-C")
            .arg(local_dir_parent)
            .status();
        match status {
            Ok(s) if s.success() => {}
            Ok(s) => {
                let _ = pod_exec(&client, &pod_id, &format!("rm -f {}", shell_escape(&remote_tmp)), 30).await;
                log_and_exit("pull", &pod_id, &remote, Some(&local_target.display().to_string()),
                    raw_estimate, false, &format!("local tar xzf exited {}", s), json);
            }
            Err(e) => {
                let _ = pod_exec(&client, &pod_id, &format!("rm -f {}", shell_escape(&remote_tmp)), 30).await;
                log_and_exit("pull", &pod_id, &remote, Some(&local_target.display().to_string()),
                    raw_estimate, false, &format!("local tar: {}", e), json);
            }
        }
    }

    // Cleanup remote tmp.
    let _ = pod_exec(&client, &pod_id, &format!("rm -f {}", shell_escape(&remote_tmp)), 30).await;

    let _ = append_transfer_log(&TransferEvent::now(
        "pull", &pod_id, &remote, Some(&local_target.display().to_string()),
        raw_estimate, true, None,
    ));

    if json {
        println!("{}", serde_json::json!({
            "ok": true, "verb": "pull", "pod": pod_id,
            "remote": remote, "local": local_target, "size_bytes": raw_estimate,
        }));
    } else {
        eprintln!("pulled pod-{}:{} → {} (~{} bytes)", pod_id, remote, local_target.display(), raw_estimate);
    }
}

fn resolve_pull_target(remote: &str, to: Option<&str>, is_dir: bool) -> PathBuf {
    let remote_base = remote
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("out");
    match to {
        Some(p) => {
            let path = PathBuf::from(p);
            if is_dir {
                // For dir pulls, path is the parent we untar into;
                // the final dir path is <to>/<basename>.
                path.join(remote_base)
            } else {
                let p_path = path.clone();
                if p.ends_with('/') || p_path.is_dir() {
                    path.join(remote_base)
                } else {
                    path
                }
            }
        }
        None => PathBuf::from(".").join(remote_base),
    }
}

async fn run_ok(
    client: &atomek_pods::TytusClient,
    pod_id: &str,
    cmd: &str,
    timeout: u32,
) -> Result<(), String> {
    let r = pod_exec(client, pod_id, cmd, timeout).await?;
    if r.exit_code != 0 {
        return Err(format!(
            "remote command failed (exit {}): {}",
            r.exit_code,
            r.stderr.unwrap_or_default(),
        ));
    }
    Ok(())
}

// ── LS ──────────────────────────────────────────────────────

pub async fn cmd_ls(
    http: &atomek_core::HttpClient,
    path: Option<String>,
    pod: Option<String>,
    json: bool,
) {
    let target = path.unwrap_or_else(|| POD_INBOX.to_string());
    if let Err(e) = validate_pod_path(&target) {
        eprintln!("tytus ls: {}", e);
        std::process::exit(1);
    }

    let Some((state, client)) = bootstrap_client(http).await else {
        std::process::exit(1);
    };
    let pod_id = match resolve_pod(pod.as_deref(), &state) {
        Ok(p) => p,
        Err(e) => { eprintln!("tytus ls: {}", e); std::process::exit(1); }
    };

    // Use a machine-parseable format: mode|size|mtime-epoch|name
    // `find -printf` isn't in BusyBox on all pods, but dash pods
    // have coreutils `stat`. Fall back to `ls -la` + best-effort parse.
    let list_cmd = format!(
        "if [ -d {t} ]; then \
            for f in {t}/*; do \
                if [ -e \"$f\" ]; then \
                    stat -c '%a|%s|%Y|%n' \"$f\" 2>/dev/null || ls -la -- \"$f\"; \
                fi; \
            done; \
         elif [ -e {t} ]; then \
            stat -c '%a|%s|%Y|%n' {t} 2>/dev/null || ls -la -- {t}; \
         else \
            echo missing; \
         fi",
        t = shell_escape(&target),
    );
    match pod_exec(&client, &pod_id, &list_cmd, 30).await {
        Ok(r) if r.exit_code == 0 => {
            let out = r.stdout.unwrap_or_default();
            if out.trim() == "missing" {
                eprintln!("tytus ls: no such path: {}", target);
                std::process::exit(1);
            }
            if json {
                let entries: Vec<serde_json::Value> = out
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .filter_map(parse_stat_line)
                    .collect();
                println!("{}", serde_json::json!({
                    "pod": pod_id, "path": target, "entries": entries,
                }));
            } else {
                println!("{:<6} {:>10} {:<25} {}", "mode", "size", "mtime", "name");
                for line in out.lines() {
                    if let Some(v) = parse_stat_line(line) {
                        println!(
                            "{:<6} {:>10} {:<25} {}",
                            v.get("mode").and_then(|x| x.as_str()).unwrap_or(""),
                            v.get("size_bytes").and_then(|x| x.as_u64()).unwrap_or(0),
                            v.get("mtime").and_then(|x| x.as_str()).unwrap_or(""),
                            v.get("name").and_then(|x| x.as_str()).unwrap_or(""),
                        );
                    } else {
                        println!("{}", line);
                    }
                }
            }
        }
        Ok(r) => {
            eprintln!("tytus ls: pod returned non-zero ({}): {}", r.exit_code, r.stderr.unwrap_or_default());
            std::process::exit(1);
        }
        Err(e) => { eprintln!("tytus ls: {}", e); std::process::exit(1); }
    }
}

fn parse_stat_line(line: &str) -> Option<serde_json::Value> {
    // Expect: MODE|SIZE|MTIME_EPOCH|PATH. If we can't parse, return None.
    let mut parts = line.splitn(4, '|');
    let mode = parts.next()?;
    let size: u64 = parts.next()?.parse().ok()?;
    let epoch: i64 = parts.next()?.parse().ok()?;
    let name = parts.next()?;
    let mtime = chrono::DateTime::<chrono::Utc>::from_timestamp(epoch, 0)
        .map(|d| d.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_default();
    Some(serde_json::json!({
        "mode": mode,
        "size_bytes": size,
        "mtime": mtime,
        "name": name,
    }))
}

// ── RM ──────────────────────────────────────────────────────

pub async fn cmd_rm(
    http: &atomek_core::HttpClient,
    remote: String,
    pod: Option<String>,
    recursive: bool,
    json: bool,
) {
    if let Err(e) = validate_pod_path(&remote) {
        log_and_exit("rm", "?", &remote, None, 0, false, &e.to_string(), json);
    }

    let Some((state, client)) = bootstrap_client(http).await else {
        std::process::exit(1);
    };
    let pod_id = match resolve_pod(pod.as_deref(), &state) {
        Ok(p) => p,
        Err(e) => log_and_exit("rm", "?", &remote, None, 0, false, &e.to_string(), json),
    };

    // Check if directory. If so, require --recursive.
    let kind_cmd = format!(
        "if [ -d {r} ]; then echo dir; elif [ -e {r} ]; then echo other; else echo missing; fi",
        r = shell_escape(&remote),
    );
    let kind = match pod_exec(&client, &pod_id, &kind_cmd, 30).await {
        Ok(r) if r.exit_code == 0 => r.stdout.unwrap_or_default().trim().to_string(),
        Ok(r) => log_and_exit("rm", &pod_id, &remote, None, 0, false,
            &format!("stat failed: {}", r.stderr.unwrap_or_default()), json),
        Err(e) => log_and_exit("rm", &pod_id, &remote, None, 0, false, &e, json),
    };
    if kind == "missing" {
        log_and_exit("rm", &pod_id, &remote, None, 0, false,
            &format!("no such path: {}", remote), json);
    }
    if kind == "dir" && !recursive {
        log_and_exit("rm", &pod_id, &remote, None, 0, false,
            "refusing to remove directory without --recursive", json);
    }

    let cmd = if recursive {
        format!("rm -rf {}", shell_escape(&remote))
    } else {
        format!("rm -f {}", shell_escape(&remote))
    };

    match pod_exec(&client, &pod_id, &cmd, 60).await {
        Ok(r) if r.exit_code == 0 => {
            let _ = append_transfer_log(&TransferEvent::now(
                "rm", &pod_id, &remote, None, 0, true, None,
            ));
            if json {
                println!("{}", serde_json::json!({
                    "ok": true, "verb": "rm", "pod": pod_id, "remote": remote,
                }));
            } else {
                eprintln!("removed pod-{}:{}", pod_id, remote);
            }
        }
        Ok(r) => {
            let err = format!("rm failed (exit {}): {}", r.exit_code, r.stderr.unwrap_or_default());
            log_and_exit("rm", &pod_id, &remote, None, 0, false, &err, json);
        }
        Err(e) => log_and_exit("rm", &pod_id, &remote, None, 0, false, &e, json),
    }
}

// ── TRANSFERS (log viewer) ─────────────────────────────────

pub async fn cmd_transfers(tail: usize, pod_filter: Option<String>, json: bool) {
    let path = crate::transfer::transfer_log_path();
    if !path.exists() {
        if json { println!("[]"); }
        else    { eprintln!("no transfers logged yet (log: {})", path.display()); }
        return;
    }
    let contents = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => { eprintln!("tytus transfers: read {}: {}", path.display(), e); std::process::exit(1); }
    };

    let mut rows: Vec<TransferEvent> = contents
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<TransferEvent>(l).ok())
        .collect();

    if let Some(p) = pod_filter.as_ref() {
        rows.retain(|r| r.pod == *p);
    }

    // tail == 0 means "all"
    if tail > 0 && rows.len() > tail {
        let skip = rows.len() - tail;
        rows = rows.into_iter().skip(skip).collect();
    }

    if json {
        for r in &rows {
            println!("{}", serde_json::to_string(r).unwrap_or_default());
        }
    } else {
        println!("{:<26} {:<5} {:<3} {:<10} {:<8} {}", "ts", "verb", "pod", "size", "ok", "remote");
        for r in &rows {
            println!(
                "{:<26} {:<5} {:<3} {:<10} {:<8} {}",
                r.ts,
                r.verb,
                r.pod,
                r.size_bytes,
                if r.ok { "ok" } else { "FAIL" },
                r.remote,
            );
            if !r.ok {
                if let Some(reason) = &r.err_reason {
                    println!("      └ {}", reason);
                }
            }
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────

/// Log failure to the transfer audit log and exit with non-zero.
/// `!` return so it can replace the bail-out arm of a match.
fn log_and_exit(
    verb: &str,
    pod: &str,
    remote: &str,
    local: Option<&str>,
    size: u64,
    ok: bool,
    reason: &str,
    json: bool,
) -> ! {
    let _ = append_transfer_log(&TransferEvent::now(
        verb, pod, remote, local, size, ok, Some(reason),
    ));
    if json {
        println!("{}", serde_json::json!({
            "ok": false, "verb": verb, "pod": pod, "remote": remote, "error": reason,
        }));
    } else {
        eprintln!("tytus {}: {}", verb, reason);
    }
    std::process::exit(1);
}

// ── Unit tests for helpers ─────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transfer::MAX_TRANSFER_BYTES;

    #[test]
    fn parent_dir_extracts_correctly() {
        assert_eq!(parent_dir("/app/workspace/inbox/foo.pdf"), "/app/workspace/inbox");
        assert_eq!(parent_dir("/app/workspace/foo.pdf"), "/app/workspace");
        assert_eq!(parent_dir("/foo"), "/");
    }

    #[test]
    fn parse_stat_line_happy_path() {
        let line = "644|1024|1713964800|/app/workspace/inbox/report.pdf";
        let v = parse_stat_line(line).unwrap();
        assert_eq!(v["mode"], "644");
        assert_eq!(v["size_bytes"], 1024);
        assert_eq!(v["name"], "/app/workspace/inbox/report.pdf");
    }

    #[test]
    fn parse_stat_line_rejects_garbage() {
        assert!(parse_stat_line("ls: no such file").is_none());
        assert!(parse_stat_line("total 0").is_none());
    }

    #[test]
    fn resolve_pull_target_file_default_cwd() {
        let t = resolve_pull_target("/app/workspace/inbox/report.pdf", None, false);
        assert_eq!(t, PathBuf::from("./report.pdf"));
    }

    #[test]
    fn resolve_pull_target_file_explicit() {
        let t = resolve_pull_target("/app/workspace/inbox/report.pdf", Some("/tmp/x.pdf"), false);
        assert_eq!(t, PathBuf::from("/tmp/x.pdf"));
    }

    #[test]
    fn resolve_pull_target_dir_uses_basename() {
        let t = resolve_pull_target("/app/workspace/project", None, true);
        assert_eq!(t, PathBuf::from("./project"));
    }

    #[test]
    fn random_nonce_is_unique_across_calls() {
        let a = random_nonce();
        let b = random_nonce();
        // Same process id but differing nanos should yield
        // different nonces; if they collide, the test is lucky-
        // enough to accept either a different-nonce path OR a
        // re-call after a nanosecond bump.
        if a == b {
            std::thread::sleep(std::time::Duration::from_millis(1));
            let c = random_nonce();
            assert_ne!(a, c);
        } else {
            assert_ne!(a, b);
        }
    }

    #[test]
    fn enforce_size_ceiling_boundary() {
        assert!(enforce_size_ceiling(MAX_TRANSFER_BYTES).is_ok());
        assert!(enforce_size_ceiling(MAX_TRANSFER_BYTES + 1).is_err());
    }
}

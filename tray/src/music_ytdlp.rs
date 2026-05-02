//! Nuclear-derived yt-dlp wrapper for JULI3TA music search/playback.
//!
//! Synchronous on purpose: `tytus-tray` serves requests with `tiny_http`, one
//! thread per request. No shell is involved; all user input is passed as argv.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::RwLock;

static YTDLP_PATH: RwLock<Option<PathBuf>> = RwLock::new(None);

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicStatus {
    pub ready: bool,
    pub installing: bool,
    pub source: String,
    pub version: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicSearchResult {
    pub id: String,
    pub source: String,
    pub title: String,
    pub duration_ms: Option<u64>,
    pub thumbnail_url: Option<String>,
    pub channel: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicStreamInfo {
    pub video_id: String,
    pub stream_url: String,
    pub duration_ms: Option<u64>,
    pub title: Option<String>,
    pub container: Option<String>,
    pub codec: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicPlaylistInfo {
    pub id: String,
    pub title: String,
    pub entries: Vec<MusicSearchResult>,
}

#[derive(Debug, serde::Deserialize)]
struct YtdlpJson {
    id: Option<String>,
    title: Option<String>,
    duration: Option<f64>,
    url: Option<String>,
    thumbnail: Option<String>,
    channel: Option<String>,
    uploader: Option<String>,
    ext: Option<String>,
    acodec: Option<String>,
    playlist_title: Option<String>,
    playlist_id: Option<String>,
}

pub fn set_binary_path(path: PathBuf) {
    if let Ok(mut guard) = YTDLP_PATH.write() {
        *guard = Some(path);
    }
}

pub fn clear_binary_path() {
    if let Ok(mut guard) = YTDLP_PATH.write() {
        *guard = None;
    }
}

fn get_binary_path() -> Result<PathBuf, String> {
    if let Ok(guard) = YTDLP_PATH.read() {
        if let Some(path) = guard.clone() {
            return Ok(path);
        }
    }
    Err("yt-dlp unavailable".to_string())
}

pub fn binary_version(path: &PathBuf) -> Option<String> {
    let out = Command::new(path).arg("--version").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if version.is_empty() { None } else { Some(version) }
}

pub fn configured_status() -> MusicStatus {
    match get_binary_path() {
        Ok(path) => MusicStatus {
            ready: true,
            installing: false,
            source: if path.file_name().and_then(|s| s.to_str()) == Some("yt-dlp") && path.is_relative() {
                "system".to_string()
            } else {
                "bundled".to_string()
            },
            version: binary_version(&path),
            error: None,
        },
        Err(e) => MusicStatus {
            ready: false,
            installing: false,
            source: "none".to_string(),
            version: None,
            error: Some(e),
        },
    }
}

pub fn validate_query(query: &str) -> Result<String, String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err("query is required".to_string());
    }
    if trimmed.chars().count() > 200 {
        return Err("query is too long".to_string());
    }
    Ok(trimmed.to_string())
}

pub fn clamp_limit(limit: Option<u32>) -> u32 {
    limit.unwrap_or(20).clamp(1, 50)
}

pub fn validate_video_id(video_id: &str) -> Result<String, String> {
    let trimmed = video_id.trim();
    if trimmed.len() == 11 && trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        Ok(trimmed.to_string())
    } else {
        Err("invalid videoId".to_string())
    }
}

pub fn validate_playlist_url(url: &str) -> Result<String, String> {
    let trimmed = url.trim();
    if trimmed.len() > 2048 {
        return Err("playlist URL is too long".to_string());
    }
    let parsed = reqwest::Url::parse(trimmed).map_err(|_| "invalid playlist URL".to_string())?;
    if parsed.scheme() != "https" {
        return Err("playlist URL must be https".to_string());
    }
    let host = parsed.host_str().unwrap_or("").to_ascii_lowercase();
    let ok = matches!(host.as_str(), "youtube.com" | "www.youtube.com" | "music.youtube.com" | "youtu.be")
        || host.ends_with(".youtube.com");
    if !ok {
        return Err("playlist URL must be a YouTube URL".to_string());
    }
    Ok(trimmed.to_string())
}

fn run_ytdlp(args: &[&str]) -> Result<String, String> {
    let program = get_binary_path()?;
    let mut cmd = Command::new(&program);
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let output = cmd.output().map_err(|e| format!("failed to execute yt-dlp: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp failed: {}", compact_error(&stderr)));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn compact_error(raw: &str) -> String {
    let s = raw.trim().replace('\n', " ");
    if s.chars().count() > 600 {
        s.chars().take(600).collect::<String>() + "…"
    } else {
        s
    }
}

fn duration_ms(duration: Option<f64>) -> Option<u64> {
    duration.and_then(|d| {
        if d.is_finite() && d >= 0.0 {
            Some((d * 1000.0).round() as u64)
        } else {
            None
        }
    })
}

fn parse_ndjson_entries(stdout: &str) -> Vec<YtdlpJson> {
    stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<YtdlpJson>(line).ok())
        .collect()
}

fn map_entry(entry: YtdlpJson) -> Option<MusicSearchResult> {
    let id = entry.id?;
    Some(MusicSearchResult {
        id,
        source: "youtube".to_string(),
        title: entry.title.unwrap_or_else(|| "Untitled".to_string()),
        duration_ms: duration_ms(entry.duration),
        thumbnail_url: entry.thumbnail,
        channel: entry.channel.or(entry.uploader),
    })
}

pub fn search(query: &str, limit: u32) -> Result<Vec<MusicSearchResult>, String> {
    let query = validate_query(query)?;
    let limit = clamp_limit(Some(limit));
    let search_url = format!("ytsearch{limit}:{query}");
    let stdout = run_ytdlp(&["--dump-json", "--flat-playlist", "--no-warnings", &search_url])?;
    Ok(parse_ndjson_entries(&stdout).into_iter().filter_map(map_entry).collect())
}

pub fn stream(video_id: &str) -> Result<MusicStreamInfo, String> {
    let video_id = validate_video_id(video_id)?;
    let url = format!("https://www.youtube.com/watch?v={video_id}");
    let stdout = run_ytdlp(&[
        "-f",
        "bestaudio[ext=m4a]/bestaudio[ext=webm]/bestaudio",
        "--dump-json",
        "--no-playlist",
        "--no-warnings",
        &url,
    ])?;
    let info: YtdlpJson = serde_json::from_str(&stdout)
        .map_err(|e| format!("failed to parse yt-dlp stream output: {e}"))?;
    let stream_url = info.url.ok_or_else(|| "yt-dlp returned no stream URL".to_string())?;
    Ok(MusicStreamInfo {
        video_id,
        stream_url,
        duration_ms: duration_ms(info.duration),
        title: info.title,
        container: info.ext,
        codec: info.acodec,
    })
}

pub fn playlist(url: &str, limit: u32) -> Result<MusicPlaylistInfo, String> {
    let url = validate_playlist_url(url)?;
    let limit = clamp_limit(Some(limit)) as usize;
    let stdout = run_ytdlp(&["--dump-json", "--flat-playlist", "--no-warnings", &url])?;
    let entries_raw = parse_ndjson_entries(&stdout);
    let title = entries_raw
        .iter()
        .find_map(|e| e.playlist_title.clone())
        .unwrap_or_else(|| "YouTube playlist".to_string());
    let id = entries_raw
        .iter()
        .find_map(|e| e.playlist_id.clone())
        .unwrap_or_default();
    let entries = entries_raw.into_iter().filter_map(map_entry).take(limit).collect();
    Ok(MusicPlaylistInfo { id, title, entries })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ndjson_and_skips_bad_lines() {
        let raw = r#"{"id":"abc12345678","title":"One","duration":12.5,"thumbnail":"http://t","channel":"C"}
not-json
{"id":"def12345678","title":"Two"}
"#;
        let entries = parse_ndjson_entries(raw);
        assert_eq!(entries.len(), 2);
        let mapped: Vec<_> = entries.into_iter().filter_map(map_entry).collect();
        assert_eq!(mapped[0].duration_ms, Some(12_500));
        assert_eq!(mapped[0].channel.as_deref(), Some("C"));
    }

    #[test]
    fn validates_video_id() {
        assert!(validate_video_id("abc_DEF-123").is_ok());
        assert!(validate_video_id("short").is_err());
        assert!(validate_video_id("abc_DEF-123!").is_err());
    }

    #[test]
    fn clamps_limit() {
        assert_eq!(clamp_limit(None), 20);
        assert_eq!(clamp_limit(Some(0)), 1);
        assert_eq!(clamp_limit(Some(99)), 50);
    }

    #[test]
    fn validates_playlist_url_hosts() {
        assert!(validate_playlist_url("https://www.youtube.com/playlist?list=abc").is_ok());
        assert!(validate_playlist_url("https://music.youtube.com/playlist?list=abc").is_ok());
        assert!(validate_playlist_url("http://www.youtube.com/playlist?list=abc").is_err());
        assert!(validate_playlist_url("https://example.com/playlist?list=abc").is_err());
    }
}

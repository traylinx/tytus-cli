//! Nuclear-derived yt-dlp wrapper for JULI3TA music search/playback.
//!
//! Synchronous on purpose: `tytus-tray` serves requests with `tiny_http`, one
//! thread per request. No shell is involved; all user input is passed as argv.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{OnceLock, RwLock};
use std::time::{Duration, Instant};

static YTDLP_PATH: RwLock<Option<PathBuf>> = RwLock::new(None);
static SEARCH_CACHE: OnceLock<RwLock<HashMap<String, CacheEntry<Vec<MusicSearchResult>>>>> =
    OnceLock::new();
static STREAM_CACHE: OnceLock<RwLock<HashMap<String, CacheEntry<MusicStreamInfo>>>> =
    OnceLock::new();

#[derive(Clone, Debug)]
struct CacheEntry<T: Clone> {
    value: T,
    stored_at: Instant,
}

fn cached<T: Clone>(
    cache: &OnceLock<RwLock<HashMap<String, CacheEntry<T>>>>,
    key: &str,
    ttl: Duration,
) -> Option<T> {
    let guard = cache
        .get_or_init(|| RwLock::new(HashMap::new()))
        .read()
        .ok()?;
    let entry = guard.get(key)?;
    if entry.stored_at.elapsed() <= ttl {
        Some(entry.value.clone())
    } else {
        None
    }
}

fn store_cache<T: Clone>(
    cache: &OnceLock<RwLock<HashMap<String, CacheEntry<T>>>>,
    key: String,
    value: T,
) {
    if let Ok(mut guard) = cache.get_or_init(|| RwLock::new(HashMap::new())).write() {
        guard.insert(
            key,
            CacheEntry {
                value,
                stored_at: Instant::now(),
            },
        );
    }
}

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

#[derive(Clone, Debug)]
pub struct MusicReferenceSample {
    pub video_id: String,
    pub wav: Vec<u8>,
    pub duration_sec: f64,
    pub start_sec: f64,
    pub source_duration_sec: Option<f64>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicPlaylistInfo {
    pub id: String,
    pub title: String,
    pub entries: Vec<MusicSearchResult>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicProviderCapability {
    pub search_tracks: bool,
    pub search_albums: bool,
    pub search_artists: bool,
    pub search_playlists: bool,
    pub stream_resolve: bool,
    pub library_metadata: bool,
    pub account_connect: bool,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicProviderStatus {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub state: String,
    pub configured: bool,
    pub needs: Vec<String>,
    pub capabilities: MusicProviderCapability,
    pub load_ms: Option<u64>,
    pub message: String,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedMusicSearchResults {
    pub tracks: Vec<MusicSearchResult>,
    pub albums: Vec<MusicSearchResult>,
    pub artists: Vec<MusicSearchResult>,
    pub playlists: Vec<MusicSearchResult>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedMusicSearchResponse {
    pub provider: String,
    pub results: UnifiedMusicSearchResults,
    pub warnings: Vec<String>,
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
    if version.is_empty() {
        None
    } else {
        Some(version)
    }
}

pub fn configured_status() -> MusicStatus {
    match get_binary_path() {
        Ok(path) => MusicStatus {
            ready: true,
            installing: false,
            source: if path.file_name().and_then(|s| s.to_str()) == Some("yt-dlp")
                && path.is_relative()
            {
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
    if trimmed.len() == 11
        && trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
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
    let ok = matches!(
        host.as_str(),
        "youtube.com" | "www.youtube.com" | "music.youtube.com" | "youtu.be"
    ) || host.ends_with(".youtube.com");
    if !ok {
        return Err("playlist URL must be a YouTube URL".to_string());
    }
    Ok(trimmed.to_string())
}

fn run_ytdlp(args: &[&str]) -> Result<String, String> {
    let program = get_binary_path()?;
    let mut cmd = Command::new(&program);
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let output = cmd
        .output()
        .map_err(|e| format!("failed to execute yt-dlp: {e}"))?;
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
    let thumbnail = entry
        .thumbnail
        .or_else(|| Some(format!("https://i.ytimg.com/vi/{id}/hqdefault.jpg")));
    Some(MusicSearchResult {
        id,
        source: "youtube".to_string(),
        title: entry.title.unwrap_or_else(|| "Untitled".to_string()),
        duration_ms: duration_ms(entry.duration),
        thumbnail_url: thumbnail,
        channel: entry.channel.or(entry.uploader),
    })
}

pub fn search(query: &str, limit: u32) -> Result<Vec<MusicSearchResult>, String> {
    let query = validate_query(query)?;
    let limit = clamp_limit(Some(limit));
    let cache_key = format!("{}:{limit}", query.to_ascii_lowercase());
    if let Some(rows) = cached(&SEARCH_CACHE, &cache_key, Duration::from_secs(15 * 60)) {
        return Ok(rows);
    }
    let search_url = format!("ytsearch{limit}:{query}");
    let stdout = run_ytdlp(&[
        "--dump-json",
        "--flat-playlist",
        "--no-warnings",
        &search_url,
    ])?;
    let rows = parse_ndjson_entries(&stdout)
        .into_iter()
        .filter_map(map_entry)
        .collect::<Vec<_>>();
    store_cache(&SEARCH_CACHE, cache_key, rows.clone());
    Ok(rows)
}

pub fn providers() -> Vec<MusicProviderStatus> {
    let status = configured_status();
    let connector_statuses = crate::music_connectors::statuses()
        .into_iter()
        .map(|s| (s.provider.clone(), s))
        .collect::<HashMap<_, _>>();
    let spotify = connector_statuses.get("spotify");
    let lastfm = connector_statuses.get("lastfm");
    let discogs = connector_statuses.get("discogs");
    vec![
        MusicProviderStatus {
            id: "youtube".to_string(),
            name: "YouTube".to_string(),
            kind: "streaming".to_string(),
            state: if status.ready { "ready" } else { "starting" }.to_string(),
            configured: status.ready,
            needs: if status.ready {
                vec![]
            } else {
                vec!["yt-dlp".to_string()]
            },
            load_ms: None,
            message: if status.ready {
                format!(
                    "yt-dlp {} ready",
                    status.version.unwrap_or_else(|| "installed".to_string())
                )
            } else {
                status
                    .error
                    .unwrap_or_else(|| "yt-dlp is installing".to_string())
            },
            capabilities: MusicProviderCapability {
                search_tracks: true,
                search_albums: false,
                search_artists: false,
                search_playlists: true,
                stream_resolve: true,
                library_metadata: false,
                account_connect: false,
            },
        },
        MusicProviderStatus {
            id: "spotify".to_string(),
            name: "Spotify".to_string(),
            kind: "metadata".to_string(),
            state: spotify
                .map(|s| {
                    if s.oauth_required {
                        "oauth_required"
                    } else if s.connected {
                        "ready"
                    } else {
                        "needs_credentials"
                    }
                })
                .unwrap_or("oauth_required")
                .to_string(),
            configured: spotify.map(|s| s.connected).unwrap_or(false),
            needs: vec!["SPOTIFY_OAUTH_PKCE".to_string()],
            load_ms: None,
            message: spotify.map(|s| s.message.clone()).unwrap_or_else(|| {
                "Spotify account linking requires OAuth PKCE; no token-paste flow is exposed."
                    .to_string()
            }),
            capabilities: MusicProviderCapability {
                search_tracks: true,
                search_albums: true,
                search_artists: true,
                search_playlists: true,
                stream_resolve: false,
                library_metadata: true,
                account_connect: true,
            },
        },
        MusicProviderStatus {
            id: "lastfm".to_string(),
            name: "Last.fm".to_string(),
            kind: "metadata".to_string(),
            state: lastfm
                .map(|s| {
                    if s.connected {
                        "ready"
                    } else {
                        "needs_credentials"
                    }
                })
                .unwrap_or("needs_credentials")
                .to_string(),
            configured: lastfm.map(|s| s.connected).unwrap_or(false),
            needs: lastfm
                .map(|s| {
                    s.credential_specs
                        .iter()
                        .filter(|c| c.required)
                        .map(|c| c.name.clone())
                        .collect()
                })
                .unwrap_or_else(|| vec!["apiKey".to_string()]),
            load_ms: None,
            message: lastfm.map(|s| s.message.clone()).unwrap_or_else(|| {
                "Artist bios, tags and listener metadata need an API key.".to_string()
            }),
            capabilities: MusicProviderCapability {
                search_tracks: false,
                search_albums: false,
                search_artists: true,
                search_playlists: false,
                stream_resolve: false,
                library_metadata: true,
                account_connect: true,
            },
        },
        MusicProviderStatus {
            id: "discogs".to_string(),
            name: "Discogs".to_string(),
            kind: "catalog".to_string(),
            state: discogs
                .map(|s| {
                    if s.connected {
                        "ready"
                    } else {
                        "needs_credentials"
                    }
                })
                .unwrap_or("needs_credentials")
                .to_string(),
            configured: discogs.map(|s| s.connected).unwrap_or(false),
            needs: discogs
                .map(|s| {
                    s.credential_specs
                        .iter()
                        .filter(|c| c.required)
                        .map(|c| c.name.clone())
                        .collect()
                })
                .unwrap_or_else(|| vec!["token".to_string()]),
            load_ms: None,
            message: discogs.map(|s| s.message.clone()).unwrap_or_else(|| {
                "Release/catalog metadata and album artwork need a Discogs token.".to_string()
            }),
            capabilities: MusicProviderCapability {
                search_tracks: false,
                search_albums: true,
                search_artists: true,
                search_playlists: false,
                stream_resolve: false,
                library_metadata: true,
                account_connect: true,
            },
        },
    ]
}

pub fn search_unified(
    query: &str,
    types: &[&str],
    limit: u32,
) -> Result<UnifiedMusicSearchResponse, String> {
    let wants_tracks = types.is_empty() || types.iter().any(|t| *t == "tracks" || *t == "track");
    let wants_playlists = types.iter().any(|t| *t == "playlists" || *t == "playlist");
    let rows = if wants_tracks || wants_playlists {
        search(query, limit)?
    } else {
        Vec::new()
    };
    Ok(UnifiedMusicSearchResponse {
        provider: "youtube".to_string(),
        results: UnifiedMusicSearchResults {
            tracks: if wants_tracks { rows.clone() } else { vec![] },
            // YouTube/yt-dlp has no stable album/artist entity endpoint in this tray layer yet.
            albums: vec![],
            artists: vec![],
            playlists: if wants_playlists { rows } else { vec![] },
        },
        warnings: vec![],
    })
}

pub fn stream(video_id: &str) -> Result<MusicStreamInfo, String> {
    let video_id = validate_video_id(video_id)?;
    if let Some(info) = cached(&STREAM_CACHE, &video_id, Duration::from_secs(90 * 60)) {
        return Ok(info);
    }
    let info = match stream_url_fast(&video_id) {
        Ok(fast_url) => MusicStreamInfo {
            video_id,
            stream_url: fast_url,
            duration_ms: None,
            title: None,
            container: None,
            codec: None,
        },
        Err(_) => stream_with_metadata(&video_id)?,
    };
    store_cache(&STREAM_CACHE, info.video_id.clone(), info.clone());
    Ok(info)
}

// Format chain: prefer audio-only m4a/webm, then any audio-only stream.
// When YouTube's SABR-only experiment leaves only combined formats (e.g.
// format 18), fall back to mp4/best so the user can still play the track —
// HTML <audio> decodes the audio track from an mp4 stream fine.
// See yt-dlp#12482 for the upstream tracking issue.
const STREAM_FORMAT: &str = "bestaudio[ext=m4a]/bestaudio[ext=webm]/bestaudio/best[ext=mp4]/best";

// Player-client chain: yt-dlp's default list leads with web_embedded, which
// gets "playability status: ERROR" on embed-restricted but otherwise-public
// videos (live-event archives, channel uploads with embedding off, …). We
// add `android,ios,tv` as fallbacks so those videos still resolve. Direct
// probe across 4 videos confirmed `android` is the only client that returns
// a playback URL for embed-restricted content; ios/tv cover SABR-only and
// region edge cases that don't always hit android. Keeping `default` first
// preserves yt-dlp's own per-version optimization order.
const PLAYER_CLIENT_ARG: &str = "youtube:player_client=default,android,ios,tv";

fn stream_url_fast(video_id: &str) -> Result<String, String> {
    let url = format!("https://www.youtube.com/watch?v={video_id}");
    let stdout = run_ytdlp(&[
        "-f",
        STREAM_FORMAT,
        "--extractor-args",
        PLAYER_CLIENT_ARG,
        "--get-url",
        "--no-playlist",
        "--no-warnings",
        &url,
    ])?;
    stdout
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .ok_or_else(|| "yt-dlp returned no stream URL".to_string())
        .map(str::to_string)
}

fn duration_hint_from_stream(info: &MusicStreamInfo) -> Option<f64> {
    if let Some(ms) = info.duration_ms {
        return Some(ms as f64 / 1000.0);
    }
    reqwest::Url::parse(&info.stream_url)
        .ok()
        .and_then(|url| {
            url.query_pairs()
                .find(|(k, _)| k == "dur" || k == "duration")
                .and_then(|(_, v)| v.parse::<f64>().ok())
        })
        .filter(|d| d.is_finite() && *d > 0.0)
}

fn ffmpeg_path() -> Result<PathBuf, String> {
    let mut candidates = Vec::new();
    if let Ok(env_path) = std::env::var("FFMPEG_PATH") {
        if !env_path.trim().is_empty() {
            candidates.push(PathBuf::from(env_path));
        }
    }
    candidates.extend([
        PathBuf::from("ffmpeg"),
        PathBuf::from("/opt/homebrew/bin/ffmpeg"),
        PathBuf::from("/usr/local/bin/ffmpeg"),
        PathBuf::from("/usr/bin/ffmpeg"),
    ]);

    for candidate in candidates {
        if Command::new(&candidate)
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return Ok(candidate);
        }
    }
    Err("ffmpeg is required for fast reference samples but was not found".to_string())
}

fn run_ffmpeg_reference(
    ffmpeg: &PathBuf,
    input: &str,
    start_sec: f64,
    duration_sec: f64,
    _timeout: Duration,
) -> Result<Vec<u8>, String> {
    let out_path = std::env::temp_dir().join(format!(
        "tytus-juli3ta-reference-{}-{}.wav",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));

    let output = Command::new(ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-rw_timeout",
            "15000000",
            "-ss",
            &format!("{start_sec:.3}"),
            "-i",
            input,
            "-t",
            &format!("{duration_sec:.3}"),
            "-vn",
            "-ac",
            "1",
            "-ar",
            "24000",
            "-f",
            "wav",
            "-y",
            out_path.to_string_lossy().as_ref(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("failed to execute ffmpeg: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = std::fs::remove_file(&out_path);
        return Err(format!(
            "ffmpeg reference sample failed: {}",
            compact_error(&stderr)
        ));
    }
    let wav = std::fs::read(&out_path)
        .map_err(|e| format!("failed to read ffmpeg reference sample: {e}"));
    let _ = std::fs::remove_file(&out_path);
    let wav = wav?;
    if wav.len() < 44 {
        return Err("ffmpeg produced an empty reference sample".to_string());
    }
    Ok(wav)
}

pub fn reference_sample(
    video_id: &str,
    start_sec: Option<f64>,
    duration_sec: Option<f64>,
) -> Result<MusicReferenceSample, String> {
    let video_id = validate_video_id(video_id)?;
    let info = stream_with_metadata(&video_id).or_else(|_| stream(&video_id))?;
    let source_duration_sec = duration_hint_from_stream(&info);
    let duration_sec = duration_sec
        .filter(|d| d.is_finite() && *d >= 6.0 && *d <= 90.0)
        .unwrap_or(60.0);
    let start_sec = start_sec
        .filter(|s| s.is_finite() && *s >= 0.0)
        .unwrap_or_else(|| {
            source_duration_sec
                .map(|d| {
                    (d * 0.55 - duration_sec / 2.0)
                        .max(0.0)
                        .min((d - duration_sec).max(0.0))
                })
                .unwrap_or(0.0)
        });
    let ffmpeg = ffmpeg_path()?;
    let wav = match run_ffmpeg_reference(
        &ffmpeg,
        &info.stream_url,
        start_sec,
        duration_sec,
        Duration::from_secs(12),
    ) {
        Ok(wav) => wav,
        Err(first_err) => {
            let tmp_base = std::env::temp_dir().join(format!(
                "tytus-juli3ta-reference-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0)
            ));
            let pattern = format!("{}.%(ext)s", tmp_base.to_string_lossy());
            let page_url = format!("https://www.youtube.com/watch?v={video_id}");
            run_ytdlp(&[
                "-f",
                "bestaudio[ext=m4a]/bestaudio[ext=webm]/bestaudio/best[ext=mp4]/best",
                "--extractor-args",
                PLAYER_CLIENT_ARG,
                "--no-playlist",
                "--no-warnings",
                "-o",
                &pattern,
                &page_url,
            ])
            .map_err(|e| {
                format!("yt-dlp reference fallback failed after ffmpeg error ({first_err}): {e}")
            })?;
            let tmp = ["m4a", "webm", "mp4", "opus"]
                .iter()
                .map(|ext| tmp_base.with_extension(ext))
                .find(|p| p.exists())
                .ok_or_else(|| {
                    format!(
                        "yt-dlp reference fallback wrote no audio after ffmpeg error ({first_err})"
                    )
                })?;
            let second = run_ffmpeg_reference(
                &ffmpeg,
                tmp.to_string_lossy().as_ref(),
                start_sec,
                duration_sec,
                Duration::from_secs(15),
            );
            let _ = std::fs::remove_file(&tmp);
            second.map_err(|e| format!("{e}; direct ffmpeg error was: {first_err}"))?
        }
    };

    Ok(MusicReferenceSample {
        video_id,
        wav,
        duration_sec,
        start_sec,
        source_duration_sec,
    })
}

fn stream_with_metadata(video_id: &str) -> Result<MusicStreamInfo, String> {
    let video_id = validate_video_id(video_id)?;
    let url = format!("https://www.youtube.com/watch?v={video_id}");
    let stdout = run_ytdlp(&[
        "-f",
        STREAM_FORMAT,
        "--extractor-args",
        PLAYER_CLIENT_ARG,
        "--dump-json",
        "--no-playlist",
        "--no-warnings",
        &url,
    ])?;
    let info: YtdlpJson = serde_json::from_str(&stdout)
        .map_err(|e| format!("failed to parse yt-dlp stream output: {e}"))?;
    let stream_url = info
        .url
        .ok_or_else(|| "yt-dlp returned no stream URL".to_string())?;
    let info = MusicStreamInfo {
        video_id,
        stream_url,
        duration_ms: duration_ms(info.duration),
        title: info.title,
        container: info.ext,
        codec: info.acodec,
    };
    Ok(info)
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
    let entries = entries_raw
        .into_iter()
        .filter_map(map_entry)
        .take(limit)
        .collect();
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

    // Regression for embed-restricted videos (e.g. live-event archives like
    // the Eindhoven Diving Cup streams Sebastian reported). yt-dlp's default
    // client list starts with web_embedded, which gets "playability status:
    // ERROR" on these videos even though they're public — only `android`
    // returns a playback URL. The PLAYER_CLIENT_ARG explicitly appends
    // android/ios/tv so the extractor walks the list before giving up.
    #[test]
    fn player_client_arg_includes_android_fallback() {
        assert!(
            PLAYER_CLIENT_ARG.starts_with("youtube:player_client="),
            "PLAYER_CLIENT_ARG must be the yt-dlp extractor-args key: {PLAYER_CLIENT_ARG}"
        );
        assert!(
            PLAYER_CLIENT_ARG.contains("default"),
            "PLAYER_CLIENT_ARG must preserve yt-dlp's default order first: {PLAYER_CLIENT_ARG}"
        );
        assert!(
            PLAYER_CLIENT_ARG.contains("android"),
            "PLAYER_CLIENT_ARG must include android to unlock embed-restricted videos: {PLAYER_CLIENT_ARG}"
        );
    }

    // Regression for yt-dlp#12482 (SABR-only experiment): YouTube sometimes
    // exposes only combined audio+video formats, which broke our previous
    // audio-only-only selector. The chain must keep audio-only preference
    // but fall back to combined mp4/best so playback still works.
    #[test]
    fn stream_format_includes_combined_fallbacks() {
        assert!(
            STREAM_FORMAT.starts_with("bestaudio[ext=m4a]"),
            "STREAM_FORMAT must prefer m4a audio-only: {STREAM_FORMAT}"
        );
        assert!(
            STREAM_FORMAT.contains("best[ext=mp4]"),
            "STREAM_FORMAT needs combined-mp4 fallback for SABR-only videos: {STREAM_FORMAT}"
        );
        assert!(
            STREAM_FORMAT.ends_with("/best"),
            "STREAM_FORMAT must end with universal /best fallback: {STREAM_FORMAT}"
        );
    }

    #[test]
    fn validates_playlist_url_hosts() {
        assert!(validate_playlist_url("https://www.youtube.com/playlist?list=abc").is_ok());
        assert!(validate_playlist_url("https://music.youtube.com/playlist?list=abc").is_ok());
        assert!(validate_playlist_url("http://www.youtube.com/playlist?list=abc").is_err());
        assert!(validate_playlist_url("https://example.com/playlist?list=abc").is_err());
    }
}

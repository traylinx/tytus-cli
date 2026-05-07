use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_AUDIO_BYTES: usize = 120 * 1024 * 1024;
const MAX_COVER_BYTES: usize = 16 * 1024 * 1024;
const METADATA_FILE: &str = "metadata.json";

#[derive(Debug, thiserror::Error)]
pub enum LibraryError {
    #[error("invalid request: {0}")]
    Invalid(String),
    #[error("track not found")]
    NotFound,
    #[error("io error: {0}")]
    Io(String),
}

pub fn error_status(err: &LibraryError) -> u16 {
    match err {
        LibraryError::Invalid(_) => 400,
        LibraryError::NotFound => 404,
        LibraryError::Io(_) => 500,
    }
}

impl From<std::io::Error> for LibraryError {
    fn from(value: std::io::Error) -> Self {
        LibraryError::Io(value.to_string())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveTrackRequest {
    pub id: String,
    pub title: String,
    pub style_tags: String,
    pub lyrics_preview: String,
    pub duration_ms: u64,
    pub bitrate: u32,
    pub sample_rate: u32,
    pub size_bytes: u64,
    pub created_at: u64,
    pub audio_data_url: String,
    #[serde(default)]
    pub specs_json: String,
    #[serde(default)]
    pub cover_data_url: String,
    #[serde(default)]
    pub theme: String,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default = "default_audio_kind")]
    pub audio_kind: String,
    #[serde(default)]
    pub external_id: String,
    #[serde(default)]
    pub external_url: String,
    #[serde(default)]
    pub thumbnail_url: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub album: String,
}

fn default_source() -> String {
    "juli3ta".to_string()
}
fn default_audio_kind() -> String {
    "data_url".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryTrack {
    pub id: String,
    pub title: String,
    pub style_tags: String,
    pub lyrics_preview: String,
    pub duration_ms: u64,
    pub bitrate: u32,
    pub sample_rate: u32,
    pub size_bytes: u64,
    pub created_at: u64,
    pub audio_data_url: String,
    pub specs_json: String,
    pub cover_data_url: String,
    pub theme: String,
    pub source: String,
    pub audio_kind: String,
    pub external_id: String,
    pub external_url: String,
    pub thumbnail_url: String,
    pub artist: String,
    pub album: String,
    pub folder_path: String,
    pub audio_path: String,
    pub lyrics_path: String,
    pub metadata_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct MetadataFile {
    id: String,
    title: String,
    style_tags: String,
    lyrics_preview: String,
    duration_ms: u64,
    bitrate: u32,
    sample_rate: u32,
    size_bytes: u64,
    created_at: u64,
    specs_json: String,
    cover_data_url: String,
    theme: String,
    source: String,
    audio_kind: String,
    external_id: String,
    external_url: String,
    thumbnail_url: String,
    artist: String,
    album: String,
    audio_file: String,
    cover_file: String,
    lyrics_file: String,
    saved_at: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryListResponse {
    pub root_path: String,
    pub tracks: Vec<LibraryTrack>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveTrackResponse {
    pub ok: bool,
    pub root_path: String,
    pub track: LibraryTrack,
}

pub fn library_root() -> PathBuf {
    if let Ok(raw) = std::env::var("JULI3TA_MUSIC_DIR") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    let base = dirs::audio_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join("Music")))
        .unwrap_or_else(|| std::env::temp_dir().join("Music"));
    base.join("JULI3TA")
}

pub fn list_tracks() -> Result<LibraryListResponse, LibraryError> {
    let root = library_root();
    fs::create_dir_all(&root)?;
    let mut tracks = Vec::new();
    for entry in fs::read_dir(&root)? {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let meta_path = path.join(METADATA_FILE);
        if !meta_path.is_file() {
            continue;
        }
        let Ok(raw) = fs::read_to_string(&meta_path) else {
            continue;
        };
        let Ok(meta) = serde_json::from_str::<MetadataFile>(&raw) else {
            continue;
        };
        tracks.push(track_from_meta(&root, &path, &meta_path, meta));
    }
    tracks.sort_by(|a, b| {
        b.created_at
            .cmp(&a.created_at)
            .then_with(|| a.title.cmp(&b.title))
    });
    Ok(LibraryListResponse {
        root_path: root.to_string_lossy().to_string(),
        tracks,
    })
}

pub fn save_track(track: SaveTrackRequest) -> Result<SaveTrackResponse, LibraryError> {
    validate_track(&track)?;
    let root = library_root();
    fs::create_dir_all(&root)?;
    let folder = find_track_folder(&root, &track.id)?.unwrap_or_else(|| {
        root.join(format!(
            "{}_{}_{}",
            track.created_at.max(now_ms()),
            slug(&track.title, 60),
            slug(&track.id, 18)
        ))
    });
    fs::create_dir_all(&folder)?;

    let mut audio_file = String::new();
    if !track.audio_data_url.trim().is_empty() {
        let (ext, bytes) = decode_data_url(&track.audio_data_url, MAX_AUDIO_BYTES, "audio")?;
        audio_file = format!("track.{ext}");
        fs::write(folder.join(&audio_file), bytes)?;
    }

    let mut cover_file = String::new();
    if !track.cover_data_url.trim().is_empty() {
        match decode_data_url(&track.cover_data_url, MAX_COVER_BYTES, "cover") {
            Ok((ext, bytes)) => {
                cover_file = format!("cover.{ext}");
                fs::write(folder.join(&cover_file), bytes)?;
            }
            Err(_) => {
                // Cover generation is optional. Keep metadata save robust.
                cover_file.clear();
            }
        }
    }

    let lyrics_file = if track.lyrics_preview.trim().is_empty() {
        String::new()
    } else {
        let name = "lyrics.txt".to_string();
        fs::write(folder.join(&name), track.lyrics_preview.as_bytes())?;
        name
    };

    let metadata = MetadataFile {
        id: track.id,
        title: track.title,
        style_tags: track.style_tags,
        lyrics_preview: track.lyrics_preview,
        duration_ms: track.duration_ms,
        bitrate: track.bitrate,
        sample_rate: track.sample_rate,
        size_bytes: track.size_bytes,
        created_at: track.created_at,
        specs_json: track.specs_json,
        cover_data_url: track.cover_data_url,
        theme: track.theme,
        source: track.source,
        audio_kind: if audio_file.is_empty() {
            "lyrics_only".to_string()
        } else {
            track.audio_kind
        },
        external_id: track.external_id,
        external_url: track.external_url,
        thumbnail_url: track.thumbnail_url,
        artist: track.artist,
        album: track.album,
        audio_file,
        cover_file,
        lyrics_file,
        saved_at: now_ms(),
    };
    let metadata_path = folder.join(METADATA_FILE);
    let metadata_bytes = serde_json::to_vec_pretty(&metadata)
        .map_err(|e| LibraryError::Io(format!("metadata serialize failed: {e}")))?;
    fs::write(&metadata_path, metadata_bytes)?;
    let out = track_from_meta(&root, &folder, &metadata_path, metadata);
    Ok(SaveTrackResponse {
        ok: true,
        root_path: root.to_string_lossy().to_string(),
        track: out,
    })
}

pub fn delete_track(id: &str) -> Result<(), LibraryError> {
    validate_id(id)?;
    let root = library_root();
    if let Some(folder) = find_track_folder(&root, id)? {
        fs::remove_dir_all(folder)?;
    }
    Ok(())
}

pub fn read_audio(id: &str) -> Result<(Vec<u8>, String), LibraryError> {
    validate_id(id)?;
    let root = library_root();
    let folder = find_track_folder(&root, id)?.ok_or(LibraryError::NotFound)?;
    let meta_path = folder.join(METADATA_FILE);
    let raw = fs::read_to_string(meta_path)?;
    let meta: MetadataFile = serde_json::from_str(&raw)
        .map_err(|e| LibraryError::Io(format!("metadata parse failed: {e}")))?;
    if meta.audio_file.is_empty() {
        return Err(LibraryError::NotFound);
    }
    let audio_path = folder.join(&meta.audio_file);
    let mut bytes = Vec::new();
    fs::File::open(audio_path)?.read_to_end(&mut bytes)?;
    let mime = match Path::new(&meta.audio_file)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mp3")
    {
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "m4a" | "mp4" => "audio/mp4",
        "flac" => "audio/flac",
        _ => "audio/mpeg",
    };
    Ok((bytes, mime.to_string()))
}

pub fn open_library_folder() -> Result<String, LibraryError> {
    let root = library_root();
    fs::create_dir_all(&root)?;
    atomek_core::platform::open::open_path(&root)
        .map_err(|e| LibraryError::Io(format!("failed to open folder: {e}")))?;
    Ok(root.to_string_lossy().to_string())
}

fn track_from_meta(
    _root: &Path,
    folder: &Path,
    metadata_path: &Path,
    meta: MetadataFile,
) -> LibraryTrack {
    let audio_data_url = if meta.audio_file.is_empty() {
        String::new()
    } else {
        format!("/api/juli3ta/library/audio?id={}", meta.id)
    };
    LibraryTrack {
        id: meta.id,
        title: meta.title,
        style_tags: meta.style_tags,
        lyrics_preview: meta.lyrics_preview,
        duration_ms: meta.duration_ms,
        bitrate: meta.bitrate,
        sample_rate: meta.sample_rate,
        size_bytes: meta.size_bytes,
        created_at: meta.created_at,
        audio_data_url,
        specs_json: meta.specs_json,
        cover_data_url: meta.cover_data_url,
        theme: meta.theme,
        source: meta.source,
        audio_kind: meta.audio_kind,
        external_id: meta.external_id,
        external_url: meta.external_url,
        thumbnail_url: meta.thumbnail_url,
        artist: meta.artist,
        album: meta.album,
        folder_path: folder.to_string_lossy().to_string(),
        audio_path: if meta.audio_file.is_empty() {
            String::new()
        } else {
            folder.join(meta.audio_file).to_string_lossy().to_string()
        },
        lyrics_path: if meta.lyrics_file.is_empty() {
            String::new()
        } else {
            folder.join(meta.lyrics_file).to_string_lossy().to_string()
        },
        metadata_path: metadata_path.to_string_lossy().to_string(),
    }
}

fn validate_track(track: &SaveTrackRequest) -> Result<(), LibraryError> {
    validate_id(&track.id)?;
    if track.title.trim().is_empty() || track.title.len() > 240 {
        return Err(LibraryError::Invalid(
            "title required and must be <= 240 chars".into(),
        ));
    }
    if track.source != "juli3ta" {
        return Err(LibraryError::Invalid(
            "only generated juli3ta tracks belong in the file library".into(),
        ));
    }
    Ok(())
}

fn validate_id(id: &str) -> Result<(), LibraryError> {
    if id.is_empty()
        || id.len() > 120
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(LibraryError::Invalid("invalid track id".into()));
    }
    Ok(())
}

fn find_track_folder(root: &Path, id: &str) -> Result<Option<PathBuf>, LibraryError> {
    if !root.exists() {
        return Ok(None);
    }
    for entry in fs::read_dir(root)? {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let meta_path = path.join(METADATA_FILE);
        if !meta_path.is_file() {
            continue;
        }
        let Ok(raw) = fs::read_to_string(&meta_path) else {
            continue;
        };
        let Ok(meta) = serde_json::from_str::<MetadataFile>(&raw) else {
            continue;
        };
        if meta.id == id {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

fn decode_data_url(
    input: &str,
    limit: usize,
    label: &str,
) -> Result<(String, Vec<u8>), LibraryError> {
    let Some((prefix, payload)) = input.split_once(',') else {
        return Err(LibraryError::Invalid(format!(
            "{label} must be a base64 data URL"
        )));
    };
    if !prefix.contains(";base64") {
        return Err(LibraryError::Invalid(format!(
            "{label} must be base64 encoded"
        )));
    }
    let ext = if prefix.contains("audio/wav") || prefix.contains("image/wav") {
        "wav"
    } else if prefix.contains("audio/ogg") {
        "ogg"
    } else if prefix.contains("audio/mp4") || prefix.contains("audio/m4a") {
        "m4a"
    } else if prefix.contains("audio/flac") {
        "flac"
    } else if prefix.contains("image/png") {
        "png"
    } else if prefix.contains("image/webp") {
        "webp"
    } else if prefix.contains("image/gif") {
        "gif"
    } else {
        "mp3"
    };
    let bytes = general_purpose::STANDARD
        .decode(payload.as_bytes())
        .map_err(|e| LibraryError::Invalid(format!("invalid {label} base64: {e}")))?;
    if bytes.len() > limit {
        return Err(LibraryError::Invalid(format!(
            "{label} exceeds {} MB",
            limit / 1024 / 1024
        )));
    }
    Ok((ext.to_string(), bytes))
}

fn slug(value: &str, max_len: usize) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for c in value.chars() {
        let next = if c.is_ascii_alphanumeric() {
            c.to_ascii_lowercase()
        } else {
            '-'
        };
        if next == '-' {
            if last_dash {
                continue;
            }
            last_dash = true;
        } else {
            last_dash = false;
        }
        out.push(next);
        if out.len() >= max_len {
            break;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "track".to_string()
    } else {
        trimmed
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    struct EnvGuard {
        _guard: std::sync::MutexGuard<'static, ()>,
        old: Option<String>,
    }

    impl EnvGuard {
        fn new(path: &Path) -> Self {
            let guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
            let old = std::env::var("JULI3TA_MUSIC_DIR").ok();
            std::env::set_var("JULI3TA_MUSIC_DIR", path);
            Self { _guard: guard, old }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(old) = &self.old {
                std::env::set_var("JULI3TA_MUSIC_DIR", old);
            } else {
                std::env::remove_var("JULI3TA_MUSIC_DIR");
            }
        }
    }

    #[test]
    fn save_list_and_read_audio_roundtrip() {
        let root = std::env::temp_dir().join(format!("juli3ta_test_{}", now_ms()));
        let _env = EnvGuard::new(&root);
        let req = SaveTrackRequest {
            id: "t_123_abc".into(),
            title: "My Song!".into(),
            style_tags: "pop".into(),
            lyrics_preview: "hello".into(),
            duration_ms: 1234,
            bitrate: 128000,
            sample_rate: 44100,
            size_bytes: 3,
            created_at: 42,
            audio_data_url: "data:audio/mpeg;base64,AQID".into(),
            specs_json: "".into(),
            cover_data_url: "".into(),
            theme: "theme".into(),
            source: "juli3ta".into(),
            audio_kind: "data_url".into(),
            external_id: "".into(),
            external_url: "".into(),
            thumbnail_url: "".into(),
            artist: "".into(),
            album: "".into(),
        };
        let saved = save_track(req).unwrap();
        assert!(saved.track.audio_path.ends_with("track.mp3"));
        assert!(Path::new(&saved.track.lyrics_path).exists());
        let listed = list_tracks().unwrap();
        assert_eq!(listed.tracks.len(), 1);
        assert_eq!(
            listed.tracks[0].audio_data_url,
            "/api/juli3ta/library/audio?id=t_123_abc"
        );
        let (bytes, mime) = read_audio("t_123_abc").unwrap();
        assert_eq!(bytes, vec![1, 2, 3]);
        assert_eq!(mime, "audio/mpeg");
        fs::remove_dir_all(root).ok();
    }
}

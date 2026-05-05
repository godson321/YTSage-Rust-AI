use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeRequest {
    pub urls: Vec<String>,
    pub generic_mode: bool,
    pub proxy_url: Option<String>,
    pub geo_proxy_url: Option<String>,
    pub cookie_source: Option<String>,
    pub cookie_file_path: Option<String>,
    pub browser_cookies_option: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub id: String,
    pub title: String,
    pub channel: String,
    pub duration_sec: u64,
    pub thumbnail_url: Option<String>,
    pub is_playlist: bool,
    pub playlist_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatItem {
    pub format_id: String,
    pub ext: String,
    pub resolution: Option<String>,
    pub filesize: Option<u64>,
    pub has_audio: bool,
    pub is_audio_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisError {
    pub code: String,
    pub message: String,
    pub detail: Option<String>,
    pub recoverable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisItem {
    pub source_url: String,
    pub normalized_url: String,
    pub status: String,
    pub error: Option<AnalysisError>,
    pub is_playlist: bool,
    pub media_info: Option<MediaInfo>,
    pub formats: Option<Vec<FormatItem>>,
    pub thumbnail_url: Option<String>,
    pub playlist_info: Option<Value>,
    pub playlist_entries: Option<Vec<Value>>,
    pub video_info: Option<Value>,
    pub available_subtitles: Option<Value>,
    pub available_automatic_subtitles: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisResult {
    pub job_id: String,
    pub items: Vec<AnalysisItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadOptions {
    pub output_dir: String,
    pub format_id: String,
    pub audio_only: bool,
    pub merge_subs: bool,
    pub save_description: bool,
    pub save_thumbnail: bool,
    pub embed_chapters: bool,
    pub rate_limit: Option<String>,
    pub playlist_items: Option<String>,
    pub subtitle_langs: Vec<String>,
    pub enable_sponsorblock: bool,
    pub sponsorblock_categories: Vec<String>,
    pub resolution: Option<String>,
    pub download_section: Option<String>,
    pub force_keyframes: bool,
    pub proxy_url: Option<String>,
    pub geo_proxy_url: Option<String>,
    pub force_output_format: bool,
    pub preferred_output_format: Option<String>,
    pub force_audio_format: bool,
    pub preferred_audio_format: Option<String>,
    pub audio_normalization: bool,
    pub filename_format: Option<String>,
    pub cookie_file_path: Option<String>,
    pub browser_cookies_option: Option<String>,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            output_dir: String::new(),
            format_id: String::new(),
            audio_only: false,
            merge_subs: false,
            save_description: false,
            save_thumbnail: false,
            embed_chapters: false,
            rate_limit: None,
            playlist_items: None,
            subtitle_langs: Vec::new(),
            enable_sponsorblock: false,
            sponsorblock_categories: Vec::new(),
            resolution: None,
            download_section: None,
            force_keyframes: false,
            proxy_url: None,
            geo_proxy_url: None,
            force_output_format: false,
            preferred_output_format: None,
            force_audio_format: false,
            preferred_audio_format: None,
            audio_normalization: false,
            filename_format: None,
            cookie_file_path: None,
            browser_cookies_option: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadTask {
    pub task_id: String,
    pub source_url: String,
    pub title: Option<String>,
    pub state: String,
    pub progress: f64,
    pub speed_text: Option<String>,
    pub eta_text: Option<String>,
    pub output_path: Option<String>,
    pub error: Option<String>,
    pub requested_options: DownloadOptions,
}

impl DownloadTask {
    pub fn queued(source_url: String, requested_options: DownloadOptions) -> Self {
        Self {
            task_id: Uuid::new_v4().to_string(),
            source_url,
            title: None,
            state: DownloadTaskState::Queued.as_str().to_string(),
            progress: 0.0,
            speed_text: None,
            eta_text: None,
            output_path: None,
            error: None,
            requested_options,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DownloadTaskState {
    Queued,
    Analyzing,
    Ready,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl DownloadTaskState {
    pub fn as_str(&self) -> &'static str {
        match self {
            DownloadTaskState::Queued => "queued",
            DownloadTaskState::Analyzing => "analyzing",
            DownloadTaskState::Ready => "ready",
            DownloadTaskState::Downloading => "downloading",
            DownloadTaskState::Paused => "paused",
            DownloadTaskState::Completed => "completed",
            DownloadTaskState::Failed => "failed",
            DownloadTaskState::Cancelled => "cancelled",
        }
    }
}

impl From<DownloadTaskState> for String {
    fn from(value: DownloadTaskState) -> Self {
        value.as_str().to_string()
    }
}

impl Default for DownloadTaskState {
    fn default() -> Self {
        DownloadTaskState::Queued
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueState {
    pub tasks: Vec<DownloadTask>,
    pub active_task_id: Option<String>,
    pub retry_count: u32,
}

impl Default for QueueState {
    fn default() -> Self {
        Self {
            tasks: Vec::new(),
            active_task_id: None,
            retry_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadHandle {
    pub task_id: String,
    pub child_process_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessSnapshot {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub progress: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub download_path: String,
    pub generic_mode: bool,
    pub language: String,
    pub proxy_url: Option<String>,
    pub geo_proxy_url: Option<String>,
    pub filename_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub title: String,
    pub url: String,
    pub channel: String,
    pub file_path: String,
    pub download_date: Option<String>,
    pub file_size: Option<u64>,
    pub thumbnail_url: Option<String>,
    pub format_id: Option<String>,
    pub resolution: Option<String>,
    pub is_audio_only: bool,
    pub duration: Option<String>,
    pub download_options: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolStatus {
    pub name: String,
    pub installed: bool,
    pub current_version: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub source: String,
}

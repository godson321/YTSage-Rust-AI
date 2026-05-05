use std::process::Command;

use serde_json::Value;
use uuid::Uuid;

use crate::models::{
    AnalysisError, AnalysisItem, AnalysisResult, AnalyzeRequest, FormatItem, MediaInfo,
};

const YT_DLP_BIN: &str = "yt-dlp";

type RunCandidateFn = fn(&str, &AnalyzeRequest) -> Result<AnalysisItem, AnalysisError>;

pub fn analyze_urls(payload: AnalyzeRequest) -> AnalysisResult {
    analyze_urls_with_runner(payload, run_candidate)
}

fn analyze_urls_with_runner(
    payload: AnalyzeRequest,
    runner: RunCandidateFn,
) -> AnalysisResult {
    let payload_for_runner = payload.clone();
    let items = payload
        .urls
        .into_iter()
        .map(|url| analyze_single_url(&url, &payload_for_runner, runner))
        .collect();

    AnalysisResult {
        job_id: Uuid::new_v4().to_string(),
        items,
    }
}

fn analyze_single_url(url: &str, payload: &AnalyzeRequest, runner: RunCandidateFn) -> AnalysisItem {
    let candidate_urls = normalize_analysis_candidates(url);
    let mut candidate_failures = Vec::new();

    for (index, candidate_url) in candidate_urls.iter().enumerate() {
        match runner(candidate_url, payload) {
            Ok(mut item) => {
                item.source_url = url.to_string();
                return item;
            }
            Err(error) => {
                candidate_failures.push((candidate_url.clone(), error));
                if index + 1 == candidate_urls.len() {
                    let detail = batch_error_detail(&candidate_failures);
                    return failed_item(url, candidate_failures.last().unwrap().1.clone(), detail);
                }
            }
        }
    }

    failed_item(
        url,
        AnalysisError {
            code: "analysis.unknown".to_string(),
            message: "Analysis failed.".to_string(),
            detail: None,
            recoverable: true,
        },
        None,
    )
}

fn normalize_analysis_candidates(url: &str) -> Vec<String> {
    if url.contains("list=") && url.contains("watch?v=") {
        let playlist_id = url
            .split("list=")
            .nth(1)
            .and_then(|rest| rest.split('&').next())
            .unwrap_or_default();
        if !playlist_id.is_empty() {
            return vec![
                format!("https://www.youtube.com/playlist?list={playlist_id}"),
                url.to_string(),
            ];
        }
    }

    vec![url.to_string()]
}

fn run_candidate(url: &str, payload: &AnalyzeRequest) -> Result<AnalysisItem, AnalysisError> {
    let mut command = Command::new(YT_DLP_BIN);
    command
        .arg("--dump-single-json")
        .arg("--flat-playlist")
        .arg("--no-warnings");

    if payload.generic_mode {
        command.arg("--force-generic-extractor");
    }

    if let Some(cookie_file_path) = payload
        .cookie_file_path
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        command.arg("--cookies").arg(cookie_file_path);
    } else if let Some(browser_cookies_option) = payload
        .browser_cookies_option
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        command.arg("--cookies-from-browser").arg(browser_cookies_option);
    } else if let Some(cookie_source) = payload
        .cookie_source
        .as_deref()
        .filter(|value| !value.is_empty() && *value != "none")
    {
        command.arg("--cookies").arg(cookie_source);
    }

    if let Some(proxy_url) = payload.proxy_url.as_deref().filter(|value| !value.is_empty()) {
        command.arg("--proxy").arg(proxy_url);
    }

    if let Some(geo_proxy_url) = payload
        .geo_proxy_url
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        command.arg("--geo-verification-proxy").arg(geo_proxy_url);
    }

    command.arg(url);

    let output = command.output().map_err(|error| AnalysisError {
        code: "analysis.tool_missing".to_string(),
        message: "Failed to launch yt-dlp.".to_string(),
        detail: Some(error.to_string()),
        recoverable: true,
    })?;

    if !output.status.success() {
        return Err(map_ytdlp_error(&String::from_utf8_lossy(&output.stderr)));
    }

    let value: Value = serde_json::from_slice(&output.stdout).map_err(|error| AnalysisError {
        code: "analysis.invalid_json".to_string(),
        message: "yt-dlp returned invalid JSON.".to_string(),
        detail: Some(error.to_string()),
        recoverable: false,
    })?;

    Ok(parse_analysis_item(url, value))
}

fn parse_analysis_item(source_url: &str, value: Value) -> AnalysisItem {
    let is_playlist = value
        .get("_type")
        .and_then(Value::as_str)
        .map(|kind| kind == "playlist")
        .unwrap_or(false);

    let normalized_url = value
        .get("webpage_url")
        .or_else(|| value.get("original_url"))
        .and_then(Value::as_str)
        .or_else(|| value.get("url").and_then(Value::as_str))
        .unwrap_or(source_url)
        .to_string();

    let media_info = MediaInfo {
        id: string_field(&value, "id"),
        title: string_field(&value, "title"),
        channel: string_field_opt(&value, "channel")
            .or_else(|| string_field_opt(&value, "uploader"))
            .or_else(|| string_field_opt(&value, "extractor_key"))
            .unwrap_or_default(),
        duration_sec: value
            .get("duration")
            .and_then(Value::as_f64)
            .map(|duration| duration.max(0.0) as u64)
            .unwrap_or(0),
        thumbnail_url: string_field_opt(&value, "thumbnail"),
        is_playlist,
        playlist_count: value
            .get("playlist_count")
            .and_then(Value::as_u64)
            .map(|count| count as u32),
    };

    let formats = value
        .get("formats")
        .and_then(Value::as_array)
        .map(|items| items.iter().map(parse_format_item).collect())
        .unwrap_or_default();

    let playlist_entries = value.get("entries").and_then(Value::as_array).cloned();
    let playlist_info = if is_playlist { Some(value.clone()) } else { None };
    let video_info = if is_playlist {
        value
            .get("entries")
            .and_then(Value::as_array)
            .and_then(|entries| entries.first())
            .cloned()
    } else {
        Some(value.clone())
    };
    let available_subtitles = video_info
        .as_ref()
        .and_then(|info| info.get("subtitles"))
        .cloned();
    let available_automatic_subtitles = video_info
        .as_ref()
        .and_then(|info| info.get("automatic_captions"))
        .cloned();
    let thumbnail_url = playlist_info
        .as_ref()
        .and_then(|info| string_field_opt(info, "thumbnail"))
        .or_else(|| string_field_opt(&value, "thumbnail"));

    AnalysisItem {
        source_url: source_url.to_string(),
        normalized_url,
        status: "success".to_string(),
        error: None,
        is_playlist,
        media_info: Some(media_info),
        formats: Some(formats),
        thumbnail_url,
        playlist_info,
        playlist_entries,
        video_info,
        available_subtitles,
        available_automatic_subtitles,
    }
}

fn parse_format_item(value: &Value) -> FormatItem {
    FormatItem {
        format_id: string_field(value, "format_id"),
        ext: string_field(value, "ext"),
        resolution: value
            .get("resolution")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .or_else(|| match (value.get("height"), value.get("width")) {
                (Some(height), Some(width)) => Some(format!(
                    "{}x{}",
                    width.as_u64().unwrap_or(0),
                    height.as_u64().unwrap_or(0)
                )),
                _ => None,
            }),
        filesize: value
            .get("filesize")
            .and_then(Value::as_u64)
            .or_else(|| value.get("filesize_approx").and_then(Value::as_u64)),
        has_audio: value
            .get("acodec")
            .and_then(Value::as_str)
            .map(|codec| codec != "none")
            .unwrap_or(false),
        is_audio_only: value
            .get("vcodec")
            .and_then(Value::as_str)
            .map(|codec| codec == "none")
            .unwrap_or(false),
    }
}

fn map_ytdlp_error(stderr: &str) -> AnalysisError {
    let lowered = stderr.to_lowercase();

    if lowered.contains("private video")
        || lowered.contains("login_required")
        || lowered.contains("sign in if you")
        || lowered.contains("members only")
        || lowered.contains("join this channel")
    {
        return analysis_error(
            "analysis.signin_required",
            "Sign-in is required for this item.",
            stderr,
            true,
        );
    }

    if lowered.contains("age restricted") || lowered.contains("confirm your age") {
        return analysis_error(
            "analysis.age_restricted",
            "Age-restricted content.",
            stderr,
            true,
        );
    }

    if lowered.contains("not available in your country")
        || lowered.contains("geo-blocked")
        || lowered.contains("video is not available")
    {
        return analysis_error(
            "analysis.geo_blocked",
            "This item is not available in your region.",
            stderr,
            true,
        );
    }

    if lowered.contains("video unavailable")
        || lowered.contains("this video has been removed")
        || lowered.contains("video does not exist")
    {
        return analysis_error(
            "analysis.video_unavailable",
            "The video is unavailable.",
            stderr,
            false,
        );
    }

    if lowered.contains("live stream") || lowered.contains("is live") {
        return analysis_error(
            "analysis.live_stream",
            "Live streams are not supported here.",
            stderr,
            true,
        );
    }

    if lowered.contains("playlist") || lowered.contains("no entries") {
        return analysis_error(
            "analysis.playlist_error",
            "Playlist analysis failed.",
            stderr,
            true,
        );
    }

    if lowered.contains("network error")
        || lowered.contains("connection")
        || lowered.contains("timeout")
        || lowered.contains("unable to download")
    {
        return analysis_error(
            "analysis.network_error",
            "Network error while analyzing the URL.",
            stderr,
            true,
        );
    }

    if lowered.contains("invalid url")
        || lowered.contains("unsupported url")
        || lowered.contains("no video found")
    {
        return analysis_error(
            "analysis.invalid_url",
            "The URL is invalid or unsupported.",
            stderr,
            false,
        );
    }

    if lowered.contains("youtube premium")
        || lowered.contains("members-only")
        || lowered.contains("channel's members")
    {
        return analysis_error(
            "analysis.premium_content",
            "Premium or members-only content is not accessible.",
            stderr,
            true,
        );
    }

    if lowered.contains("copyright") || lowered.contains("dmca") || lowered.contains("blocked") {
        return analysis_error(
            "analysis.copyright_blocked",
            "The item is blocked.",
            stderr,
            false,
        );
    }

    if lowered.contains("unable to extract") || lowered.contains("extraction failed") {
        return analysis_error(
            "analysis.extraction_failed",
            "Failed to extract media information.",
            stderr,
            true,
        );
    }

    analysis_error(
        "analysis.generic_error",
        "yt-dlp failed to analyze the URL.",
        stderr,
        true,
    )
}

fn analysis_error(code: &str, message: &str, stderr: &str, recoverable: bool) -> AnalysisError {
    AnalysisError {
        code: code.to_string(),
        message: message.to_string(),
        detail: if stderr.trim().is_empty() {
            None
        } else {
            Some(stderr.trim().to_string())
        },
        recoverable,
    }
}

fn failed_item(source_url: &str, error: AnalysisError, detail: Option<String>) -> AnalysisItem {
    AnalysisItem {
        source_url: source_url.to_string(),
        normalized_url: source_url.to_string(),
        status: "failed".to_string(),
        error: Some(AnalysisError {
            detail: detail.or(error.detail),
            ..error
        }),
        is_playlist: false,
        media_info: None,
        formats: None,
        thumbnail_url: None,
        playlist_info: None,
        playlist_entries: None,
        video_info: None,
        available_subtitles: None,
        available_automatic_subtitles: None,
    }
}

fn batch_error_detail(candidate_failures: &[(String, AnalysisError)]) -> Option<String> {
    if candidate_failures.len() <= 1 {
        return candidate_failures
            .first()
            .and_then(|(_, error)| error.detail.clone());
    }

    let details = candidate_failures
        .iter()
        .enumerate()
        .map(|(index, (url, error))| {
            let short_url = if url.len() <= 80 {
                url.clone()
            } else {
                format!("{}...", &url[..77])
            };
            format!(
                "[{}] {}\n{}",
                index + 1,
                short_url,
                error.detail.clone().unwrap_or_else(|| error.message.clone())
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    Some(format!(
        "Batch analysis failed. Tried {} URLs.\n\n{}",
        candidate_failures.len(),
        details
    ))
}

fn string_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn string_field_opt(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MEMBERSHIP_ERROR: &str =
        "This video is members only. Sign in if you want to continue.";

    fn fake_runner_success(url: &str, _: &AnalyzeRequest) -> Result<AnalysisItem, AnalysisError> {
        Ok(AnalysisItem {
            source_url: url.to_string(),
            normalized_url: url.to_string(),
            status: "success".to_string(),
            error: None,
            is_playlist: false,
            media_info: Some(MediaInfo {
                id: "id".to_string(),
                title: "title".to_string(),
                channel: "channel".to_string(),
                duration_sec: 1,
                thumbnail_url: None,
                is_playlist: false,
                playlist_count: None,
            }),
            formats: Some(vec![]),
            thumbnail_url: None,
            playlist_info: None,
            playlist_entries: None,
            video_info: None,
            available_subtitles: None,
            available_automatic_subtitles: None,
        })
    }

    fn fake_runner_for_youtube_regression(
        url: &str,
        _: &AnalyzeRequest,
    ) -> Result<AnalysisItem, AnalysisError> {
        match url {
            "https://www.youtube.com/watch?v=9E9y-suOleI" => fake_runner_success(url, &AnalyzeRequest {
                urls: vec![],
                generic_mode: false,
                proxy_url: None,
                geo_proxy_url: None,
                cookie_source: None,
                cookie_file_path: None,
                browser_cookies_option: None,
            }),
            "https://www.youtube.com/watch?v=oyPhmcgVoSY"
            | "https://www.youtube.com/watch?v=ZX_NwrgmYFk" => Err(map_ytdlp_error(MEMBERSHIP_ERROR)),
            "https://www.youtube.com/playlist?list=PLD3Ywi8n56O7MQLPjxEK52DegGaPFLKcQ" => Ok(AnalysisItem {
                source_url: url.to_string(),
                normalized_url: url.to_string(),
                status: "success".to_string(),
                error: None,
                is_playlist: true,
                media_info: Some(MediaInfo {
                    id: "playlist-id".to_string(),
                    title: "Public playlist".to_string(),
                    channel: "channel".to_string(),
                    duration_sec: 0,
                    thumbnail_url: Some("https://img.example/playlist.jpg".to_string()),
                    is_playlist: true,
                    playlist_count: Some(3),
                }),
                formats: Some(vec![]),
                thumbnail_url: Some("https://img.example/playlist.jpg".to_string()),
                playlist_info: Some(serde_json::json!({
                    "_type": "playlist",
                    "title": "Public playlist",
                    "entries": [
                        {"id": "a", "title": "Episode 1"},
                        {"id": "b", "title": "Episode 2"},
                        {"id": "c", "title": "Episode 3"}
                    ]
                })),
                playlist_entries: Some(vec![
                    serde_json::json!({"id": "a", "title": "Episode 1"}),
                    serde_json::json!({"id": "b", "title": "Episode 2"}),
                    serde_json::json!({"id": "c", "title": "Episode 3"}),
                ]),
                video_info: Some(serde_json::json!({
                    "id": "a",
                    "title": "Episode 1",
                    "subtitles": {},
                    "automatic_captions": {}
                })),
                available_subtitles: Some(serde_json::json!({})),
                available_automatic_subtitles: Some(serde_json::json!({})),
            }),
            _ => Err(map_ytdlp_error("unsupported url")),
        }
    }

    fn fake_runner_fail(url: &str, _: &AnalyzeRequest) -> Result<AnalysisItem, AnalysisError> {
        Err(analysis_error(
            "analysis.invalid_url",
            "The URL is invalid or unsupported.",
            url,
            false,
        ))
    }

    #[test]
    fn parse_analysis_item_maps_basic_fields() {
        let payload = serde_json::json!({
            "id": "abc123",
            "title": "Sample Video",
            "channel": "Sample Channel",
            "duration": 123,
            "thumbnail": "https://img.example/thumb.jpg",
            "_type": "video",
            "webpage_url": "https://example.com/watch?v=abc123",
            "formats": [
                {
                    "format_id": "18",
                    "ext": "mp4",
                    "resolution": "640x360",
                    "filesize": 111,
                    "acodec": "aac",
                    "vcodec": "avc1"
                }
            ],
            "subtitles": {"en": [{"ext": "vtt"}]},
            "automatic_captions": {"zh": [{"ext": "vtt"}]}
        });

        let item = parse_analysis_item("https://example.com/watch?v=abc123", payload);

        assert_eq!(item.status, "success");
        assert_eq!(item.media_info.as_ref().unwrap().title, "Sample Video");
        assert_eq!(item.formats.as_ref().unwrap().len(), 1);
        assert!(item.available_subtitles.is_some());
        assert!(item.available_automatic_subtitles.is_some());
    }

    #[test]
    fn parse_analysis_item_maps_playlist_fields_for_vue_page() {
        let payload = serde_json::json!({
            "_type": "playlist",
            "id": "PLD3Ywi8n56O7MQLPjxEK52DegGaPFLKcQ",
            "title": "Public playlist",
            "thumbnail": "https://img.example/playlist.jpg",
            "webpage_url": "https://www.youtube.com/playlist?list=PLD3Ywi8n56O7MQLPjxEK52DegGaPFLKcQ",
            "playlist_count": 3,
            "entries": [
                {
                    "id": "entry-1",
                    "title": "First video",
                    "url": "https://www.youtube.com/watch?v=entry-1",
                    "thumbnail": "https://img.example/entry-1.jpg",
                    "channel": "Channel 1",
                    "duration": 45,
                    "subtitles": {"en": [{"ext": "vtt"}]},
                    "automatic_captions": {"en": [{"ext": "vtt"}]}
                },
                {
                    "id": "entry-2",
                    "title": "Second video",
                    "url": "https://www.youtube.com/watch?v=entry-2"
                },
                {
                    "id": "entry-3",
                    "title": "Third video",
                    "url": "https://www.youtube.com/watch?v=entry-3"
                }
            ]
        });

        let item = parse_analysis_item(
            "https://www.youtube.com/playlist?list=PLD3Ywi8n56O7MQLPjxEK52DegGaPFLKcQ",
            payload,
        );

        assert!(item.is_playlist);
        assert_eq!(item.thumbnail_url.as_deref(), Some("https://img.example/playlist.jpg"));
        assert_eq!(item.playlist_entries.as_ref().map(Vec::len), Some(3));
        assert_eq!(
            item.video_info
                .as_ref()
                .and_then(|video| video.get("id"))
                .and_then(Value::as_str),
            Some("entry-1")
        );
        assert!(item.available_subtitles.is_some());
        assert!(item.available_automatic_subtitles.is_some());
        assert_eq!(
            item.media_info
                .as_ref()
                .expect("playlist should include media info")
                .playlist_count,
            Some(3)
        );
    }

    #[test]
    fn normalize_playlist_candidates_extracts_playlist() {
        let candidates = normalize_analysis_candidates("https://www.youtube.com/watch?v=abc&list=PL123");

        assert_eq!(candidates[0], "https://www.youtube.com/playlist?list=PL123");
        assert_eq!(candidates[1], "https://www.youtube.com/watch?v=abc&list=PL123");
    }

    #[test]
    fn analyze_single_url_preserves_original_source_url_after_candidate_normalization() {
        let original_url = "https://www.youtube.com/watch?v=abc&list=PL123";
        let payload = AnalyzeRequest {
            urls: vec![original_url.to_string()],
            generic_mode: false,
            proxy_url: None,
            geo_proxy_url: None,
            cookie_source: None,
            cookie_file_path: None,
            browser_cookies_option: None,
        };

        let item = analyze_single_url(original_url, &payload, fake_runner_success);

        assert_eq!(item.source_url, original_url);
        assert_eq!(item.normalized_url, "https://www.youtube.com/playlist?list=PL123");
    }

    #[test]
    fn analyze_urls_with_runner_returns_one_item_per_input() {
        let result = analyze_urls_with_runner(
            AnalyzeRequest {
                urls: vec!["https://example.com/a".to_string(), "https://example.com/b".to_string()],
                generic_mode: false,
                proxy_url: None,
                geo_proxy_url: None,
                cookie_source: None,
                cookie_file_path: None,
                browser_cookies_option: None,
            },
            fake_runner_success,
        );

        assert_eq!(result.items.len(), 2);
        assert_eq!(result.items[0].status, "success");
    }

    #[test]
    fn analyze_urls_with_runner_preserves_failures() {
        let result = analyze_urls_with_runner(
            AnalyzeRequest {
                urls: vec!["https://example.com/a".to_string()],
                generic_mode: false,
                proxy_url: None,
                geo_proxy_url: None,
                cookie_source: None,
                cookie_file_path: None,
                browser_cookies_option: None,
            },
            fake_runner_fail,
        );

        assert_eq!(result.items[0].status, "failed");
        assert_eq!(
            result.items[0].error.as_ref().unwrap().code,
            "analysis.invalid_url"
        );
    }

    #[test]
    fn youtube_membership_regression_matches_original_ytsage_behavior() {
        let urls = vec![
            "https://www.youtube.com/watch?v=oyPhmcgVoSY".to_string(),
            "https://www.youtube.com/watch?v=9E9y-suOleI".to_string(),
            "https://www.youtube.com/watch?v=ZX_NwrgmYFk".to_string(),
        ];

        let result = analyze_urls_with_runner(
            AnalyzeRequest {
                urls,
                generic_mode: false,
                proxy_url: None,
                geo_proxy_url: None,
                cookie_source: None,
                cookie_file_path: None,
                browser_cookies_option: None,
            },
            fake_runner_for_youtube_regression,
        );

        assert_eq!(result.items.len(), 3);
        assert_eq!(result.items[0].status, "failed");
        assert_eq!(
            result.items[0].error.as_ref().unwrap().code,
            "analysis.signin_required"
        );
        assert_eq!(result.items[1].status, "success");
        assert_eq!(
            result.items[1]
                .media_info
                .as_ref()
                .expect("public video should include media info")
                .title,
            "title"
        );
        assert_eq!(result.items[2].status, "failed");
        assert_eq!(
            result.items[2].error.as_ref().unwrap().code,
            "analysis.signin_required"
        );
    }

    #[test]
    fn map_ytdlp_error_marks_members_only_content_as_signin_required() {
        let error = map_ytdlp_error(MEMBERSHIP_ERROR);

        assert_eq!(error.code, "analysis.signin_required");
        assert!(error.recoverable);
    }

    #[test]
    fn youtube_playlist_regression_treats_public_playlist_as_accessible() {
        let result = analyze_urls_with_runner(
            AnalyzeRequest {
                urls: vec![
                    "https://www.youtube.com/playlist?list=PLD3Ywi8n56O7MQLPjxEK52DegGaPFLKcQ"
                        .to_string(),
                ],
                generic_mode: false,
                proxy_url: None,
                geo_proxy_url: None,
                cookie_source: None,
                cookie_file_path: None,
                browser_cookies_option: None,
            },
            fake_runner_for_youtube_regression,
        );

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].status, "success");
        assert!(result.items[0].is_playlist);
        assert_eq!(
            result.items[0]
                .media_info
                .as_ref()
                .expect("playlist should include media info")
                .playlist_count,
            Some(3)
        );
        assert!(result.items[0].playlist_info.is_some());
        assert!(result.items[0].playlist_entries.is_some());
    }
}

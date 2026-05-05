#[cfg(test)]
mod tests {
    use crate::commands;
    use crate::models::{AnalyzeRequest, DownloadOptions, HistoryEntry};
    use crate::services::{analysis_service, history_service, logs_service, storage};
    use crate::state::AppState;
    use serde_json::Value;
    use std::env;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn set_temp_data_dir(prefix: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("ytsage-rust-{prefix}-{}", Uuid::new_v4()));
        storage::set_data_dir_override(dir.clone());
        dir
    }

    fn sample_history_entry(id: &str, title: &str, channel: &str) -> HistoryEntry {
        HistoryEntry {
            id: id.to_string(),
            title: title.to_string(),
            url: format!("https://example.com/{id}"),
            channel: channel.to_string(),
            file_path: format!("C:/tmp/{id}.mp4"),
            download_date: Some("2026-01-01T00:00:00".to_string()),
            file_size: Some(123),
            thumbnail_url: Some("https://img.example/thumb.jpg".to_string()),
            format_id: Some("18".to_string()),
            resolution: Some("720p".to_string()),
            is_audio_only: false,
            duration: Some("1:00".to_string()),
            download_options: Some(Value::Null),
        }
    }

    #[test]
    fn analyze_urls_returns_one_item_per_input() {
        let result = analysis_service::analyze_urls(AnalyzeRequest {
            urls: vec![
                "https://example.com/a".to_string(),
                "https://example.com/b".to_string(),
            ],
            generic_mode: false,
            proxy_url: None,
            geo_proxy_url: None,
            cookie_source: None,
            cookie_file_path: None,
            browser_cookies_option: None,
        });

        assert_eq!(result.items.len(), 2);
        assert!(matches!(result.items[0].status.as_str(), "success" | "failed"));
    }

    #[test]
    fn create_download_task_returns_queued_task_with_real_fields() {
        let state = AppState::default();
        let mut options = DownloadOptions::default();
        options.format_id = "bestvideo+bestaudio".to_string();
        options.output_dir = "C:/Downloads".to_string();
        options.audio_only = true;

        let task = commands::create_download_task_with_state(
            "https://example.com/watch?v=abc123".to_string(),
            options.clone(),
            &state,
        );

        assert!(!task.task_id.is_empty());
        assert_eq!(task.source_url, "https://example.com/watch?v=abc123");
        assert_eq!(task.state, "queued");
        assert_eq!(task.progress, 0.0);
        assert!(task.title.is_none());
        assert_eq!(task.requested_options.format_id, options.format_id);
        assert_eq!(task.requested_options.output_dir, options.output_dir);
        assert!(task.requested_options.audio_only);
    }

    #[test]
    fn search_history_returns_matching_entries_from_command_layer() {
        let _dir = set_temp_data_dir("history-command-search");
        history_service::add_history_entry(sample_history_entry("1", "Alpha", "Video Lab"));
        history_service::add_history_entry(sample_history_entry("2", "Beta", "Audio Lab"));

        let entries = commands::search_history("video".to_string());

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "1");
    }

    #[test]
    fn remove_history_entry_and_clear_history_use_command_layer() {
        let _dir = set_temp_data_dir("history-command-mutate");
        history_service::add_history_entry(sample_history_entry("1", "Alpha", "Video Lab"));
        history_service::add_history_entry(sample_history_entry("2", "Beta", "Audio Lab"));

        assert!(commands::remove_history_entry("1".to_string()));
        assert_eq!(commands::list_history().len(), 1);
        assert_eq!(commands::clear_history(), 1);
        assert!(commands::list_history().is_empty());
    }

    #[test]
    fn tail_logs_returns_latest_lines_from_command_layer() {
        let _dir = set_temp_data_dir("logs-command-tail");
        logs_service::append_log_line("one");
        logs_service::append_log_line("two");
        logs_service::append_log_line("three");

        let lines = commands::tail_logs(2);

        assert_eq!(lines, vec!["two".to_string(), "three".to_string()]);
    }

    #[test]
    fn get_queue_snapshot_returns_current_state_from_command_layer() {
        let state = AppState::default();
        let task = commands::create_download_task_with_state(
            "https://example.com/watch?v=snapshot".to_string(),
            DownloadOptions::default(),
            &state,
        );
        let snapshot = commands::get_queue_snapshot_with_state(&state);

        assert_eq!(snapshot.tasks.len(), 1);
        assert_eq!(snapshot.tasks[0].task_id, task.task_id);
        assert_eq!(snapshot.active_task_id.as_deref(), Some(task.task_id.as_str()));
    }

    #[test]
    fn clear_pause_and_retry_queue_commands_update_state() {
        let state = AppState::default();
        let first = commands::create_download_task_with_state(
            "https://example.com/watch?v=first".to_string(),
            DownloadOptions::default(),
            &state,
        );
        let second = commands::create_download_task_with_state(
            "https://example.com/watch?v=second".to_string(),
            DownloadOptions::default(),
            &state,
        );

        let paused = commands::pause_task_with_state(first.task_id.clone(), &state)
            .expect("first task should pause");
        let retried = commands::retry_task_with_state(first.task_id.clone(), &state)
            .expect("first task should retry");
        let snapshot = commands::get_queue_snapshot_with_state(&state);

        assert_eq!(paused.state, "paused");
        assert_eq!(retried.state, "queued");
        assert_eq!(snapshot.tasks.len(), 2);
        assert_eq!(snapshot.retry_count, 1);
        assert_eq!(snapshot.active_task_id.as_deref(), Some(first.task_id.as_str()));
        assert_eq!(snapshot.tasks[1].task_id, second.task_id);

        let cleared = commands::clear_queue_with_state(&state);

        assert!(cleared.tasks.is_empty());
        assert!(cleared.active_task_id.is_none());
        assert_eq!(cleared.retry_count, 0);
    }

    #[test]
    fn create_download_task_hooks_emit_snapshot_and_schedule_download() {
        let state = AppState::default();
        let emitted = std::sync::Mutex::new(Vec::new());
        let scheduled = std::sync::Mutex::new(Vec::new());

        let task = commands::create_download_task_with_hooks(
            "https://example.com/watch?v=hooked".to_string(),
            DownloadOptions::default(),
            &state,
            |snapshot| {
                emitted
                    .lock()
                    .expect("emitted queue states lock poisoned")
                    .push(snapshot);
            },
            |task_id| {
                scheduled
                    .lock()
                    .expect("scheduled task ids lock poisoned")
                    .push(task_id);
            },
        );

        let snapshots = emitted
            .lock()
            .expect("emitted queue states lock poisoned")
            .clone();
        let scheduled_task_ids = scheduled
            .lock()
            .expect("scheduled task ids lock poisoned")
            .clone();

        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].tasks.len(), 1);
        assert_eq!(snapshots[0].tasks[0].task_id, task.task_id);
        assert_eq!(scheduled_task_ids, vec![task.task_id]);
    }

    #[test]
    fn broadcasting_queue_snapshot_keeps_state_readable() {
        let state = AppState::default();
        let task = commands::create_download_task_with_state(
            "https://example.com/watch?v=emit".to_string(),
            DownloadOptions::default(),
            &state,
        );

        let snapshot = commands::get_queue_snapshot_with_state(&state);

        assert_eq!(snapshot.tasks.len(), 1);
        assert_eq!(snapshot.tasks[0].task_id, task.task_id);
    }
}

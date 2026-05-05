use std::sync::Mutex;

use crate::models::{AppSettings, DownloadHandle, QueueState};

pub const QUEUE_STATE_CHANGED_EVENT: &str = "queue_state_changed";

pub struct AppState {
    pub settings: Mutex<AppSettings>,
    pub queue_state: Mutex<QueueState>,
    pub download_handles: Mutex<Vec<DownloadHandle>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            settings: Mutex::new(AppSettings {
                download_path: String::new(),
                generic_mode: false,
                language: "zh".to_string(),
                proxy_url: None,
                geo_proxy_url: None,
                filename_format: "%(title)s_%(resolution)s.%(ext)s".to_string(),
            }),
            queue_state: Mutex::new(QueueState::default()),
            download_handles: Mutex::new(Vec::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_app_state_starts_with_empty_queue_and_download_handles() {
        let state = AppState::default();
        let queue_state = state
            .queue_state
            .lock()
            .expect("queue state lock poisoned")
            .clone();
        let download_handles = state
            .download_handles
            .lock()
            .expect("download handles lock poisoned")
            .clone();

        assert!(queue_state.tasks.is_empty());
        assert!(queue_state.active_task_id.is_none());
        assert_eq!(queue_state.retry_count, 0);
        assert!(download_handles.is_empty());
    }
}

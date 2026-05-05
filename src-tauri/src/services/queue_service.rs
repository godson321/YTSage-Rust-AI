use std::sync::Mutex;

use crate::models::{DownloadOptions, DownloadTask, QueueState};

pub fn create_queued_task(source_url: String, requested_options: DownloadOptions) -> DownloadTask {
    DownloadTask::queued(source_url, requested_options)
}

pub fn enqueue(task: DownloadTask, queue_state: &Mutex<QueueState>) -> DownloadTask {
    let mut state = queue_state.lock().expect("queue state lock poisoned");
    if state.active_task_id.is_none() {
        state.active_task_id = Some(task.task_id.clone());
    }
    state.tasks.push(task.clone());
    task
}

pub fn snapshot(queue_state: &Mutex<QueueState>) -> QueueState {
    queue_state.lock().expect("queue state lock poisoned").clone()
}

pub fn clear(queue_state: &Mutex<QueueState>) {
    let mut state = queue_state.lock().expect("queue state lock poisoned");
    *state = QueueState::default();
}

pub fn pause_task(queue_state: &Mutex<QueueState>, task_id: &str) -> Option<DownloadTask> {
    let mut state = queue_state.lock().expect("queue state lock poisoned");
    let index = state.tasks.iter().position(|task| task.task_id == task_id)?;
    state.tasks[index].state = "paused".to_string();
    state.active_task_id = None;
    Some(state.tasks[index].clone())
}

pub fn resume_task(queue_state: &Mutex<QueueState>, task_id: &str) -> Option<DownloadTask> {
    let mut state = queue_state.lock().expect("queue state lock poisoned");
    let index = state.tasks.iter().position(|task| task.task_id == task_id)?;
    state.tasks[index].state = "downloading".to_string();
    state.active_task_id = Some(task_id.to_string());
    Some(state.tasks[index].clone())
}

pub fn cancel_task(queue_state: &Mutex<QueueState>, task_id: &str) -> Option<DownloadTask> {
    let mut state = queue_state.lock().expect("queue state lock poisoned");
    let index = state.tasks.iter().position(|task| task.task_id == task_id)?;
    state.tasks[index].state = "cancelled".to_string();
    state.tasks[index].error = None;
    if state.active_task_id.as_deref() == Some(task_id) {
        state.active_task_id = None;
    }
    Some(state.tasks[index].clone())
}

pub fn fail_task(queue_state: &Mutex<QueueState>, task_id: &str, error: &str) -> Option<DownloadTask> {
    let mut state = queue_state.lock().expect("queue state lock poisoned");
    let index = state.tasks.iter().position(|task| task.task_id == task_id)?;
    state.tasks[index].state = "failed".to_string();
    state.tasks[index].error = Some(error.to_string());
    if state.active_task_id.as_deref() == Some(task_id) {
        state.active_task_id = None;
    }
    Some(state.tasks[index].clone())
}

pub fn mark_downloading(queue_state: &Mutex<QueueState>, task_id: &str) -> Option<DownloadTask> {
    let mut state = queue_state.lock().expect("queue state lock poisoned");
    let index = state.tasks.iter().position(|task| task.task_id == task_id)?;
    state.tasks[index].state = "downloading".to_string();
    state.active_task_id = Some(task_id.to_string());
    Some(state.tasks[index].clone())
}

pub fn set_progress(
    queue_state: &Mutex<QueueState>,
    task_id: &str,
    progress: f64,
) -> Option<DownloadTask> {
    let mut state = queue_state.lock().expect("queue state lock poisoned");
    let index = state.tasks.iter().position(|task| task.task_id == task_id)?;
    state.tasks[index].progress = progress.clamp(0.0, 1.0);
    Some(state.tasks[index].clone())
}

pub fn set_download_details(
    queue_state: &Mutex<QueueState>,
    task_id: &str,
    speed_text: Option<String>,
    eta_text: Option<String>,
) -> Option<DownloadTask> {
    let mut state = queue_state.lock().expect("queue state lock poisoned");
    let index = state.tasks.iter().position(|task| task.task_id == task_id)?;
    state.tasks[index].speed_text = speed_text;
    state.tasks[index].eta_text = eta_text;
    Some(state.tasks[index].clone())
}

pub fn mark_completed(
    queue_state: &Mutex<QueueState>,
    task_id: &str,
    output_path: Option<String>,
) -> Option<DownloadTask> {
    let mut state = queue_state.lock().expect("queue state lock poisoned");
    let index = state.tasks.iter().position(|task| task.task_id == task_id)?;
    state.tasks[index].state = "completed".to_string();
    state.tasks[index].progress = 1.0;
    state.tasks[index].output_path = output_path;
    if state.active_task_id.as_deref() == Some(task_id) {
        state.active_task_id = None;
    }
    Some(state.tasks[index].clone())
}

pub fn retry_task(queue_state: &Mutex<QueueState>, task_id: &str) -> Option<DownloadTask> {
    let mut state = queue_state.lock().expect("queue state lock poisoned");
    let index = state.tasks.iter().position(|task| task.task_id == task_id)?;
    state.retry_count += 1;
    state.tasks[index].state = "queued".to_string();
    state.tasks[index].error = None;
    state.active_task_id = Some(state.tasks[index].task_id.clone());
    Some(state.tasks[index].clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_queued_task_uses_real_task_fields() {
        let options = DownloadOptions::default();
        let task = create_queued_task(
            "https://example.com/watch?v=abc123".to_string(),
            options.clone(),
        );

        assert!(!task.task_id.is_empty());
        assert_eq!(task.source_url, "https://example.com/watch?v=abc123");
        assert_eq!(task.state, "queued");
        assert_eq!(task.progress, 0.0);
        assert!(task.title.is_none());
        assert_eq!(task.requested_options.format_id, options.format_id);
    }

    #[test]
    fn enqueue_sets_active_task_for_first_item() {
        let queue_state = Mutex::new(QueueState::default());
        clear(&queue_state);

        let task = create_queued_task(
            "https://example.com/watch?v=abc123".to_string(),
            DownloadOptions::default(),
        );
        let queued = enqueue(task.clone(), &queue_state);
        let snapshot = snapshot(&queue_state);

        assert_eq!(queued.task_id, task.task_id);
        assert_eq!(snapshot.active_task_id.as_deref(), Some(task.task_id.as_str()));
        assert_eq!(snapshot.tasks.len(), 1);

        clear(&queue_state);
    }

    #[test]
    fn pause_task_marks_task_paused_and_clears_active_task() {
        let queue_state = Mutex::new(QueueState::default());
        let task = create_queued_task(
            "https://example.com/watch?v=abc123".to_string(),
            DownloadOptions::default(),
        );
        enqueue(task.clone(), &queue_state);

        let paused = pause_task(&queue_state, &task.task_id).expect("task should pause");
        let snapshot = snapshot(&queue_state);

        assert_eq!(paused.state, "paused");
        assert!(paused.progress >= 0.0);
        assert!(snapshot.active_task_id.is_none());
        assert_eq!(snapshot.tasks[0].state, "paused");
    }

    #[test]
    fn resume_and_cancel_update_real_task_states() {
        let queue_state = Mutex::new(QueueState::default());
        let task = create_queued_task(
            "https://example.com/watch?v=abc123".to_string(),
            DownloadOptions::default(),
        );
        enqueue(task.clone(), &queue_state);
        mark_downloading(&queue_state, &task.task_id).expect("task should exist");
        pause_task(&queue_state, &task.task_id).expect("task should pause");

        let resumed = resume_task(&queue_state, &task.task_id).expect("task should resume");
        assert_eq!(resumed.state, "downloading");

        let cancelled = cancel_task(&queue_state, &task.task_id).expect("task should cancel");
        let snapshot = snapshot(&queue_state);
        assert_eq!(cancelled.state, "cancelled");
        assert!(snapshot.active_task_id.is_none());
    }

    #[test]
    fn retry_task_marks_failed_item_queued_again() {
        let queue_state = Mutex::new(QueueState::default());
        let task = create_queued_task(
            "https://example.com/watch?v=abc123".to_string(),
            DownloadOptions::default(),
        );
        enqueue(task.clone(), &queue_state);
        fail_task(&queue_state, &task.task_id, "boom");

        let retried = retry_task(&queue_state, &task.task_id).expect("task should retry");
        let snapshot = snapshot(&queue_state);

        assert_eq!(retried.state, "queued");
        assert_eq!(retried.error, None);
        assert_eq!(snapshot.retry_count, 1);
        assert_eq!(snapshot.active_task_id.as_deref(), Some(task.task_id.as_str()));
    }

    #[test]
    fn mark_downloading_set_progress_and_mark_completed_update_real_task_states() {
        let queue_state = Mutex::new(QueueState::default());
        let task = create_queued_task(
            "https://example.com/watch?v=abc123".to_string(),
            DownloadOptions::default(),
        );
        enqueue(task.clone(), &queue_state);

        let downloading = mark_downloading(&queue_state, &task.task_id).expect("task should exist");
        assert_eq!(downloading.state, "downloading");

        let progressed = set_progress(&queue_state, &task.task_id, 1.5).expect("task should exist");
        assert_eq!(progressed.progress, 1.0);

        let detailed = set_download_details(
            &queue_state,
            &task.task_id,
            Some("3.20MiB/s".to_string()),
            Some("00:18".to_string()),
        )
        .expect("task should exist");
        assert_eq!(detailed.speed_text.as_deref(), Some("3.20MiB/s"));
        assert_eq!(detailed.eta_text.as_deref(), Some("00:18"));

        let completed = mark_completed(&queue_state, &task.task_id, Some("C:/tmp/file.mp4".to_string()))
            .expect("task should complete");
        let snapshot = snapshot(&queue_state);

        assert_eq!(completed.state, "completed");
        assert_eq!(completed.progress, 1.0);
        assert_eq!(completed.output_path.as_deref(), Some("C:/tmp/file.mp4"));
        assert!(snapshot.active_task_id.is_none());
    }
}

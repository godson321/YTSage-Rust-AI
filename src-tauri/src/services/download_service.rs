use std::process::Command;
use std::sync::Mutex;

use crate::models::{
    DownloadHandle,
    DownloadOptions,
    DownloadTask,
    ProcessSnapshot,
    QueueState,
};
use crate::services::{queue_service, tools_service};

type DownloadExecutor = fn(&DownloadTask) -> Result<ProcessSnapshot, String>;

pub fn create_download_task(
    source_url: String,
    options: DownloadOptions,
    queue_state: &Mutex<QueueState>,
    download_handles: &Mutex<Vec<DownloadHandle>>,
) -> DownloadTask {
    let task = queue_service::create_queued_task(source_url, options);
    let queued_task = queue_service::enqueue(task, queue_state);

    download_handles
        .lock()
        .expect("download handles lock poisoned")
        .push(DownloadHandle {
            task_id: queued_task.task_id.clone(),
            child_process_id: None,
        });

    queued_task
}

pub fn run_download_with_executor(
    task_id: &str,
    queue_state: &Mutex<QueueState>,
    executor: DownloadExecutor,
) -> Result<DownloadTask, String> {
    run_download_with_executor_and_emitter(task_id, queue_state, executor, |_| {})
}

pub fn run_download_with_executor_and_emitter<EmitFn>(
    task_id: &str,
    queue_state: &Mutex<QueueState>,
    executor: DownloadExecutor,
    emitter: EmitFn,
) -> Result<DownloadTask, String>
where
    EmitFn: Fn(QueueState),
{
    let task = queue_service::snapshot(queue_state)
        .tasks
        .into_iter()
        .find(|item| item.task_id == task_id)
        .ok_or_else(|| format!("Unknown task: {task_id}"))?;

    queue_service::mark_downloading(queue_state, task_id)
        .ok_or_else(|| format!("Unknown task: {task_id}"))?;
    emitter(queue_service::snapshot(queue_state));

    match executor(&task) {
        Ok(process) if process.exit_code == Some(0) => {
            queue_service::set_progress(queue_state, task_id, process.progress)
                .ok_or_else(|| format!("Unknown task: {task_id}"))?;
            let completed = queue_service::mark_completed(queue_state, task_id, None)
                .ok_or_else(|| format!("Unknown task: {task_id}"))?;
            emitter(queue_service::snapshot(queue_state));
            Ok(completed)
        }
        Ok(process) => {
            let message = format!(
                "yt-dlp exited with code {}",
                process.exit_code.unwrap_or(-1)
            );
            queue_service::fail_task(queue_state, task_id, &message)
                .ok_or_else(|| format!("Unknown task: {task_id}"))?;
            emitter(queue_service::snapshot(queue_state));
            Err(message)
        }
        Err(error) => {
            queue_service::fail_task(queue_state, task_id, &error)
                .ok_or_else(|| format!("Unknown task: {task_id}"))?;
            emitter(queue_service::snapshot(queue_state));
            Err(error)
        }
    }
}

pub fn execute_yt_dlp(task: &DownloadTask) -> Result<ProcessSnapshot, String> {
    let yt_dlp = tools_service::find_required_tool("yt-dlp")
        .ok_or_else(|| "Failed to launch yt-dlp".to_string())?;

    let mut command = Command::new(yt_dlp);
    command.arg(&task.source_url);

    if !task.requested_options.output_dir.is_empty() {
        command.arg("-P").arg(&task.requested_options.output_dir);
    }

    if task.requested_options.audio_only {
        command.arg("-x");
    }

    if !task.requested_options.format_id.is_empty() {
        command.arg("-f").arg(&task.requested_options.format_id);
    }

    let output = command.output().map_err(|error| error.to_string())?;

    Ok(ProcessSnapshot {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code(),
        progress: if output.status.success() { 1.0 } else { 0.0 },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn create_download_task_returns_queued_task_with_real_fields() {
        let queue_state = Mutex::new(QueueState::default());
        let download_handles = Mutex::new(Vec::<DownloadHandle>::new());

        let task = create_download_task(
            "https://example.com/watch?v=abc123".to_string(),
            DownloadOptions::default(),
            &queue_state,
            &download_handles,
        );

        let snapshot = queue_state.lock().expect("queue state lock poisoned").clone();
        let handles = download_handles
            .lock()
            .expect("download handles lock poisoned")
            .clone();

        assert!(!task.task_id.is_empty());
        assert_eq!(task.source_url, "https://example.com/watch?v=abc123");
        assert_eq!(task.state, "queued");
        assert_eq!(task.progress, 0.0);
        assert!(task.title.is_none());
        assert_eq!(task.requested_options.format_id, "");
        assert_eq!(snapshot.active_task_id.as_deref(), Some(task.task_id.as_str()));
        assert_eq!(handles.len(), 1);
        assert_eq!(handles[0].task_id, task.task_id);
        assert_eq!(handles[0].child_process_id, None);
    }

    #[test]
    fn create_download_task_only_updates_the_provided_queue_state() {
        let queue_state = Mutex::new(QueueState::default());
        let untouched_queue_state = Mutex::new(QueueState::default());
        let download_handles = Mutex::new(Vec::<DownloadHandle>::new());

        let task = create_download_task(
            "https://example.com/watch?v=xyz987".to_string(),
            DownloadOptions::default(),
            &queue_state,
            &download_handles,
        );

        let used_snapshot = queue_state.lock().expect("queue state lock poisoned").clone();
        let untouched_snapshot = untouched_queue_state
            .lock()
            .expect("untouched queue state lock poisoned")
            .clone();

        assert_eq!(used_snapshot.tasks.len(), 1);
        assert_eq!(used_snapshot.tasks[0].task_id, task.task_id);
        assert!(untouched_snapshot.tasks.is_empty());
        assert!(untouched_snapshot.active_task_id.is_none());
    }

    #[test]
    fn start_download_marks_task_failed_when_runner_cannot_launch() {
        let queue_state = Mutex::new(QueueState::default());
        let download_handles = Mutex::new(Vec::<DownloadHandle>::new());
        let task = create_download_task(
            "https://example.com/watch?v=fail".to_string(),
            DownloadOptions::default(),
            &queue_state,
            &download_handles,
        );

        let result = run_download_with_executor(&task.task_id, &queue_state, |_task| {
            Err("Failed to launch yt-dlp".to_string())
        });

        assert!(result.is_err());
        let snapshot = queue_service::snapshot(&queue_state);
        assert_eq!(snapshot.tasks[0].state, "failed");
        assert_eq!(
            snapshot.tasks[0].error.as_deref(),
            Some("Failed to launch yt-dlp")
        );
    }

    #[test]
    fn start_download_marks_task_completed_when_runner_succeeds() {
        let queue_state = Mutex::new(QueueState::default());
        let download_handles = Mutex::new(Vec::<DownloadHandle>::new());
        let task = create_download_task(
            "https://example.com/watch?v=ok".to_string(),
            DownloadOptions {
                output_dir: "C:/Downloads".to_string(),
                ..DownloadOptions::default()
            },
            &queue_state,
            &download_handles,
        );

        run_download_with_executor(&task.task_id, &queue_state, |_task| {
            Ok(crate::models::ProcessSnapshot {
                stdout: "done".to_string(),
                stderr: String::new(),
                exit_code: Some(0),
                progress: 1.0,
            })
        })
        .expect("download should succeed");

        let snapshot = queue_service::snapshot(&queue_state);
        assert_eq!(snapshot.tasks[0].state, "completed");
        assert_eq!(snapshot.tasks[0].progress, 1.0);
    }

    #[test]
    fn start_download_emits_queue_snapshots_for_state_changes() {
        let queue_state = Mutex::new(QueueState::default());
        let download_handles = Mutex::new(Vec::<DownloadHandle>::new());
        let task = create_download_task(
            "https://example.com/watch?v=emit".to_string(),
            DownloadOptions::default(),
            &queue_state,
            &download_handles,
        );
        let emitted = Mutex::new(Vec::<QueueState>::new());

        run_download_with_executor_and_emitter(
            &task.task_id,
            &queue_state,
            |_task| {
                Ok(crate::models::ProcessSnapshot {
                    stdout: "done".to_string(),
                    stderr: String::new(),
                    exit_code: Some(0),
                    progress: 1.0,
                })
            },
            |snapshot| {
                emitted
                    .lock()
                    .expect("emitted queue states lock poisoned")
                    .push(snapshot);
            },
        )
        .expect("download should succeed");

        let snapshots = emitted
            .lock()
            .expect("emitted queue states lock poisoned")
            .clone();
        assert_eq!(snapshots.len(), 2);
        assert_eq!(snapshots[0].tasks[0].state, "downloading");
        assert_eq!(snapshots[1].tasks[0].state, "completed");
        assert_eq!(snapshots[1].active_task_id, None);
    }
}

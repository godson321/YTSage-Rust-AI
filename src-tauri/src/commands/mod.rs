use std::thread;

use tauri::{AppHandle, Emitter, Manager, State};

use crate::models::{
    AnalysisResult,
    AnalyzeRequest,
    AppSettings,
    DownloadOptions,
    DownloadTask,
    HistoryEntry,
    QueueState,
    ToolStatus,
};
use crate::services::{
    analysis_service,
    download_service,
    history_service,
    logs_service,
    queue_service,
    settings_service,
    tools_service,
    update_service,
};
use crate::state::{AppState, QUEUE_STATE_CHANGED_EVENT};

#[tauri::command]
pub fn analyze_urls(payload: AnalyzeRequest) -> AnalysisResult {
    analysis_service::analyze_urls(payload)
}

#[tauri::command]
pub fn create_download_task(
    source_url: String,
    options: DownloadOptions,
    app_handle: AppHandle,
    state: State<AppState>,
) -> DownloadTask {
    create_download_task_with_hooks(
        source_url,
        options,
        &state,
        |snapshot| {
            let _ = app_handle.emit(QUEUE_STATE_CHANGED_EVENT, snapshot);
        },
        {
            let app_handle = app_handle.clone();
            move |task_id| {
                schedule_download(task_id, app_handle.clone());
            }
        },
    )
}

pub fn create_download_task_with_state(
    source_url: String,
    options: DownloadOptions,
    state: &AppState,
) -> DownloadTask {
    create_download_task_with_hooks(source_url, options, state, |_| {}, |_| {})
}

pub fn create_download_task_with_hooks<EmitFn, ScheduleFn>(
    source_url: String,
    options: DownloadOptions,
    state: &AppState,
    emit_snapshot: EmitFn,
    schedule_download_fn: ScheduleFn,
) -> DownloadTask
where
    EmitFn: Fn(QueueState),
    ScheduleFn: Fn(String),
{
    let task = download_service::create_download_task(
        source_url,
        options,
        &state.queue_state,
        &state.download_handles,
    );

    emit_snapshot(queue_service::snapshot(&state.queue_state));
    schedule_download_fn(task.task_id.clone());
    task
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> AppSettings {
    state.settings.lock().expect("settings lock poisoned").clone()
}

#[tauri::command]
pub fn save_settings(settings: AppSettings, state: State<AppState>) -> AppSettings {
    let mut current = state.settings.lock().expect("settings lock poisoned");
    *current = settings.clone();
    settings_service::save_settings(settings)
}

#[tauri::command]
pub fn list_history() -> Vec<HistoryEntry> {
    history_service::list_history()
}

#[tauri::command]
pub fn search_history(query: String) -> Vec<HistoryEntry> {
    history_service::search_history(&query)
}

#[tauri::command]
pub fn remove_history_entry(entry_id: String) -> bool {
    history_service::remove_history_entry(&entry_id)
}

#[tauri::command]
pub fn clear_history() -> usize {
    history_service::clear_history()
}

#[tauri::command]
pub fn tail_logs(lines: usize) -> Vec<String> {
    logs_service::tail_logs(lines)
}

#[tauri::command]
pub fn clear_queue(app_handle: AppHandle, state: State<AppState>) -> QueueState {
    let snapshot = clear_queue_with_state(&state);
    let _ = app_handle.emit(QUEUE_STATE_CHANGED_EVENT, snapshot.clone());
    snapshot
}

pub fn clear_queue_with_state(state: &AppState) -> QueueState {
    queue_service::clear(&state.queue_state);
    state
        .download_handles
        .lock()
        .expect("download handles lock poisoned")
        .clear();
    queue_service::snapshot(&state.queue_state)
}

#[tauri::command]
pub fn pause_task(task_id: String, app_handle: AppHandle, state: State<AppState>) -> Option<DownloadTask> {
    let task = pause_task_with_state(task_id, &state);
    if task.is_some() {
        let _ = app_handle.emit(
            QUEUE_STATE_CHANGED_EVENT,
            queue_service::snapshot(&state.queue_state),
        );
    }
    task
}

pub fn pause_task_with_state(task_id: String, state: &AppState) -> Option<DownloadTask> {
    queue_service::pause_task(&state.queue_state, &task_id)
}

#[tauri::command]
pub fn retry_task(task_id: String, app_handle: AppHandle, state: State<AppState>) -> Option<DownloadTask> {
    let task = retry_task_with_state(task_id, &state);
    if let Some(retried) = &task {
        let _ = app_handle.emit(
            QUEUE_STATE_CHANGED_EVENT,
            queue_service::snapshot(&state.queue_state),
        );
        schedule_download(retried.task_id.clone(), app_handle);
    }
    task
}

pub fn retry_task_with_state(task_id: String, state: &AppState) -> Option<DownloadTask> {
    queue_service::retry_task(&state.queue_state, &task_id)
}

#[tauri::command]
pub fn get_queue_snapshot(state: State<AppState>) -> QueueState {
    get_queue_snapshot_with_state(&state)
}

pub fn get_queue_snapshot_with_state(state: &AppState) -> QueueState {
    queue_service::snapshot(&state.queue_state)
}

#[tauri::command]
pub fn get_tool_status() -> Vec<ToolStatus> {
    tools_service::get_tool_status()
}

#[tauri::command]
pub fn get_update_status(current_version: String) -> crate::models::UpdateStatus {
    update_service::build_update_status(update_service::UpdateCheck {
        current_version,
        latest_version: None,
        source: "local".to_string(),
    })
}

fn schedule_download(task_id: String, app_handle: AppHandle) {
    thread::spawn(move || {
        let state = app_handle.state::<AppState>();
        let _ = download_service::run_download_with_executor_and_emitter(
            &task_id,
            &state.queue_state,
            download_service::execute_yt_dlp,
            |snapshot| {
                let _ = app_handle.emit(QUEUE_STATE_CHANGED_EVENT, snapshot);
            },
        );
    });
}

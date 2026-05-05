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

#[cfg(windows)]
use std::mem::size_of;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
#[cfg(windows)]
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, THREADENTRY32, TH32CS_SNAPTHREAD,
};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{
    OpenThread, ResumeThread, SuspendThread, THREAD_SUSPEND_RESUME, THREAD_QUERY_INFORMATION,
    THREAD_QUERY_LIMITED_INFORMATION,
};

type DownloadExecutor = fn(&DownloadTask) -> Result<ProcessSnapshot, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadControlAction {
    Pause,
    Resume,
    Cancel,
}

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

pub fn set_download_process_id(
    download_handles: &Mutex<Vec<DownloadHandle>>,
    task_id: &str,
    child_process_id: Option<u32>,
) -> Option<DownloadHandle> {
    let mut handles = download_handles
        .lock()
        .expect("download handles lock poisoned");
    let index = handles.iter().position(|handle| handle.task_id == task_id)?;
    handles[index].child_process_id = child_process_id;
    Some(handles[index].clone())
}

pub fn control_download(
    task_id: &str,
    queue_state: &Mutex<QueueState>,
    download_handles: &Mutex<Vec<DownloadHandle>>,
    action: DownloadControlAction,
) -> Result<DownloadTask, String> {
    control_download_with(task_id, queue_state, download_handles, action, control_process)
}

pub fn control_download_with<ControlFn>(
    task_id: &str,
    queue_state: &Mutex<QueueState>,
    download_handles: &Mutex<Vec<DownloadHandle>>,
    action: DownloadControlAction,
    controller: ControlFn,
) -> Result<DownloadTask, String>
where
    ControlFn: Fn(DownloadControlAction, u32) -> Result<(), String>,
{
    let child_process_id = download_handles
        .lock()
        .expect("download handles lock poisoned")
        .iter()
        .find(|handle| handle.task_id == task_id)
        .and_then(|handle| handle.child_process_id)
        .ok_or_else(|| format!("No running process for task: {task_id}"))?;

    controller(action, child_process_id)?;

    let task = match action {
        DownloadControlAction::Pause => queue_service::pause_task(queue_state, task_id),
        DownloadControlAction::Resume => queue_service::resume_task(queue_state, task_id),
        DownloadControlAction::Cancel => {
            let task = queue_service::cancel_task(queue_state, task_id);
            let _ = set_download_process_id(download_handles, task_id, None);
            task
        }
    }
    .ok_or_else(|| format!("Unknown task: {task_id}"))?;

    Ok(task)
}

fn control_process(action: DownloadControlAction, pid: u32) -> Result<(), String> {
    #[cfg(windows)]
    {
        match action {
            DownloadControlAction::Pause => set_process_threads_suspended(pid, true),
            DownloadControlAction::Resume => set_process_threads_suspended(pid, false),
            DownloadControlAction::Cancel => Command::new("taskkill")
                .arg("/F")
                .arg("/T")
                .arg("/PID")
                .arg(pid.to_string())
                .status()
                .map_err(|error| error.to_string())
                .and_then(|status| {
                    if status.success() {
                        Ok(())
                    } else {
                        Err(format!("Failed to cancel process {pid}"))
                    }
                }),
        }
    }

    #[cfg(not(windows))]
    {
        let signal = match action {
            DownloadControlAction::Pause => "-STOP",
            DownloadControlAction::Resume => "-CONT",
            DownloadControlAction::Cancel => "-TERM",
        };
        Command::new("kill")
            .arg(signal)
            .arg(pid.to_string())
            .status()
            .map_err(|error| error.to_string())
            .and_then(|status| {
                if status.success() {
                    Ok(())
                } else {
                    Err(format!("Failed to send {signal} to process {pid}"))
                }
            })
    }
}

#[cfg(windows)]
fn set_process_threads_suspended(pid: u32, suspend: bool) -> Result<(), String> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
        if snapshot == INVALID_HANDLE_VALUE as HANDLE {
            return Err("Failed to snapshot threads".to_string());
        }

        let mut entry = THREADENTRY32 {
            dwSize: size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        };
        let mut found = false;

        if Thread32First(snapshot, &mut entry) != 0 {
            loop {
                if entry.th32OwnerProcessID == pid {
                    found = true;
                    let access = THREAD_SUSPEND_RESUME | THREAD_QUERY_INFORMATION | THREAD_QUERY_LIMITED_INFORMATION;
                    let thread = OpenThread(access, 0, entry.th32ThreadID);
                    if thread != std::ptr::null_mut() {
                        if suspend {
                            SuspendThread(thread);
                        } else {
                            ResumeThread(thread);
                        }
                        CloseHandle(thread);
                    }
                }

                if Thread32Next(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }

        CloseHandle(snapshot);

        if found {
            Ok(())
        } else {
            Err(format!("No threads found for process {pid}"))
        }
    }
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
            queue_service::set_download_details(
                queue_state,
                task_id,
                process.speed_text.clone(),
                process.eta_text.clone(),
            )
            .ok_or_else(|| format!("Unknown task: {task_id}"))?;
            let completed = queue_service::mark_completed(queue_state, task_id, None)
                .ok_or_else(|| format!("Unknown task: {task_id}"))?;
            emitter(queue_service::snapshot(queue_state));
            Ok(completed)
        }
        Ok(process) => {
            let _ = queue_service::set_download_details(
                queue_state,
                task_id,
                process.speed_text.clone(),
                process.eta_text.clone(),
            );
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
        speed_text: None,
        eta_text: None,
    })
}

fn build_yt_dlp_args(task: &DownloadTask) -> Vec<String> {
    let mut args = Vec::new();

    if !task.requested_options.format_id.is_empty() {
        args.push("-f".to_string());
        args.push(task.requested_options.format_id.clone());
    }

    if task.requested_options.audio_only {
        args.push("-x".to_string());
    }

    if !task.requested_options.output_dir.is_empty() {
        args.push("-P".to_string());
        args.push(task.requested_options.output_dir.clone());
    }

    if let Some(playlist_items) = task
        .requested_options
        .playlist_items
        .as_ref()
        .filter(|value| !value.is_empty())
    {
        args.push("--playlist-items".to_string());
        args.push(playlist_items.clone());
    }

    if !task.requested_options.subtitle_langs.is_empty() {
        args.push("--write-subs".to_string());
        args.push("--write-auto-subs".to_string());
        args.push("--sub-langs".to_string());
        args.push(task.requested_options.subtitle_langs.join(","));
        if task.requested_options.merge_subs {
            args.push("--embed-subs".to_string());
        }
    }

    if task.requested_options.enable_sponsorblock
        && !task.requested_options.sponsorblock_categories.is_empty()
    {
        args.push("--sponsorblock-remove".to_string());
        args.push(task.requested_options.sponsorblock_categories.join(","));
    }

    if task.requested_options.save_description {
        args.push("--write-description".to_string());
    }

    if task.requested_options.embed_chapters {
        args.push("--embed-chapters".to_string());
    }

    if let Some(rate_limit) = task
        .requested_options
        .rate_limit
        .as_ref()
        .filter(|value| !value.is_empty())
    {
        args.push("-r".to_string());
        args.push(rate_limit.clone());
    }

    if let Some(download_section) = task
        .requested_options
        .download_section
        .as_ref()
        .filter(|value| !value.is_empty())
    {
        args.push("--download-sections".to_string());
        args.push(download_section.clone());
        if task.requested_options.force_keyframes {
            args.push("--force-keyframes-at-cuts".to_string());
        }
    }

    if let Some(proxy_url) = task
        .requested_options
        .proxy_url
        .as_ref()
        .filter(|value| !value.is_empty())
    {
        args.push("--proxy".to_string());
        args.push(proxy_url.clone());
    }

    if let Some(geo_proxy_url) = task
        .requested_options
        .geo_proxy_url
        .as_ref()
        .filter(|value| !value.is_empty())
    {
        args.push("--geo-verification-proxy".to_string());
        args.push(geo_proxy_url.clone());
    }

    if let Some(cookie_file_path) = task
        .requested_options
        .cookie_file_path
        .as_ref()
        .filter(|value| !value.is_empty())
    {
        args.push("--cookies".to_string());
        args.push(cookie_file_path.clone());
    } else if let Some(browser_cookies_option) = task
        .requested_options
        .browser_cookies_option
        .as_ref()
        .filter(|value| !value.is_empty())
    {
        args.push("--cookies-from-browser".to_string());
        args.push(browser_cookies_option.clone());
    }

    if task.requested_options.force_output_format {
        if let Some(preferred_output_format) = task
            .requested_options
            .preferred_output_format
            .as_ref()
            .filter(|value| !value.is_empty())
        {
            args.push("--merge-output-format".to_string());
            args.push(preferred_output_format.clone());
        }
    }

    if task.requested_options.audio_only && task.requested_options.force_audio_format {
        if let Some(preferred_audio_format) = task
            .requested_options
            .preferred_audio_format
            .as_ref()
            .filter(|value| !value.is_empty())
        {
            args.push("--audio-format".to_string());
            args.push(preferred_audio_format.clone());
        }
    }

    if task.requested_options.audio_normalization {
        args.push("--postprocessor-args".to_string());
        args.push("ExtractAudio:-af loudnorm=I=-16:LRA=11:TP=-1.5".to_string());
    }

    if let Some(filename_format) = task
        .requested_options
        .filename_format
        .as_ref()
        .filter(|value| !value.is_empty())
    {
        args.push("-o".to_string());
        args.push(filename_format.clone());
    }

    args.push(task.source_url.clone());
    args
}

fn parse_download_progress(stdout: &str, stderr: &str, exit_code: Option<i32>) -> ProcessSnapshot {
    let combined = if stderr.trim().is_empty() {
        stdout.to_string()
    } else if stdout.trim().is_empty() {
        stderr.to_string()
    } else {
        format!("{stdout}\n{stderr}")
    };

    let mut progress = if exit_code == Some(0) { 1.0 } else { 0.0 };
    let mut speed_text = None;
    let mut eta_text = None;

    for line in combined.lines() {
        if let Some(percent) = extract_percent(line) {
            progress = (percent / 100.0).clamp(0.0, 1.0);
        }

        if speed_text.is_none() {
            speed_text = extract_speed(line);
        }

        if eta_text.is_none() {
            eta_text = extract_eta(line);
        }
    }

    ProcessSnapshot {
        stdout: stdout.to_string(),
        stderr: stderr.to_string(),
        exit_code,
        progress,
        speed_text,
        eta_text,
    }
}

fn extract_percent(line: &str) -> Option<f64> {
    let marker = "%";
    let index = line.find(marker)?;
    let prefix = &line[..index];
    let number = prefix
        .split_whitespace()
        .last()?
        .trim_matches(|char: char| !char.is_ascii_digit() && char != '.');
    number.parse::<f64>().ok()
}

fn extract_speed(line: &str) -> Option<String> {
    let marker = " at ";
    let start = line.find(marker)? + marker.len();
    let tail = &line[start..];
    let value = tail.split_whitespace().next()?.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn extract_eta(line: &str) -> Option<String> {
    let marker = " ETA ";
    let start = line.find(marker)? + marker.len();
    let value = line[start..].split_whitespace().next()?.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
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
                speed_text: Some("3.2MiB/s".to_string()),
                eta_text: Some("00:00".to_string()),
            })
        })
        .expect("download should succeed");

        let snapshot = queue_service::snapshot(&queue_state);
        assert_eq!(snapshot.tasks[0].state, "completed");
        assert_eq!(snapshot.tasks[0].progress, 1.0);
        assert_eq!(snapshot.tasks[0].speed_text.as_deref(), Some("3.2MiB/s"));
        assert_eq!(snapshot.tasks[0].eta_text.as_deref(), Some("00:00"));
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
                    speed_text: Some("3.2MiB/s".to_string()),
                    eta_text: Some("00:00".to_string()),
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

    #[test]
    fn build_yt_dlp_args_maps_real_download_options() {
        let task = DownloadTask {
            task_id: "task-1".to_string(),
            source_url: "https://example.com/watch?v=abc123".to_string(),
            title: Some("Sample".to_string()),
            state: "queued".to_string(),
            progress: 0.0,
            speed_text: None,
            eta_text: None,
            output_path: None,
            error: None,
            requested_options: DownloadOptions {
                output_dir: "C:/Downloads".to_string(),
                format_id: "137+bestaudio".to_string(),
                audio_only: true,
                merge_subs: true,
                save_description: true,
                save_thumbnail: false,
                embed_chapters: true,
                rate_limit: Some("1048576".to_string()),
                playlist_items: Some("1,3-5".to_string()),
                subtitle_langs: vec!["en".to_string(), "zh-Hans".to_string()],
                enable_sponsorblock: true,
                sponsorblock_categories: vec!["sponsor".to_string(), "intro".to_string()],
                resolution: Some("1080".to_string()),
                download_section: Some("*00:10-00:20".to_string()),
                force_keyframes: true,
                proxy_url: Some("http://127.0.0.1:7890".to_string()),
                geo_proxy_url: Some("http://127.0.0.1:7891".to_string()),
                force_output_format: true,
                preferred_output_format: Some("mp4".to_string()),
                force_audio_format: true,
                preferred_audio_format: Some("mp3".to_string()),
                audio_normalization: true,
                filename_format: Some("%(title)s.%(ext)s".to_string()),
                cookie_file_path: Some("C:/cookies.txt".to_string()),
                browser_cookies_option: None,
            },
        };

        let args = build_yt_dlp_args(&task);

        assert!(args.windows(2).any(|pair| pair == ["-f", "137+bestaudio"]));
        assert!(args.contains(&"-x".to_string()));
        assert!(args.windows(2).any(|pair| pair == ["-P", "C:/Downloads"]));
        assert!(args.windows(2).any(|pair| pair == ["--playlist-items", "1,3-5"]));
        assert!(args.windows(2).any(|pair| pair == ["--sub-langs", "en,zh-Hans"]));
        assert!(args.contains(&"--embed-subs".to_string()));
        assert!(args.windows(2).any(|pair| pair == ["--sponsorblock-remove", "sponsor,intro"]));
        assert!(args.contains(&"--write-description".to_string()));
        assert!(args.contains(&"--embed-chapters".to_string()));
        assert!(args.windows(2).any(|pair| pair == ["-r", "1048576"]));
        assert!(args.windows(2).any(|pair| pair == ["--download-sections", "*00:10-00:20"]));
        assert!(args.contains(&"--force-keyframes-at-cuts".to_string()));
        assert!(args.windows(2).any(|pair| pair == ["--proxy", "http://127.0.0.1:7890"]));
        assert!(args.windows(2).any(|pair| pair == ["--geo-verification-proxy", "http://127.0.0.1:7891"]));
        assert!(args.windows(2).any(|pair| pair == ["--cookies", "C:/cookies.txt"]));
        assert!(args.windows(2).any(|pair| pair == ["--merge-output-format", "mp4"]));
        assert!(args.windows(2).any(|pair| pair == ["--audio-format", "mp3"]));
        assert!(args.windows(2).any(|pair| pair == ["--postprocessor-args", "ExtractAudio:-af loudnorm=I=-16:LRA=11:TP=-1.5"]));
        assert!(args.windows(2).any(|pair| pair == ["-o", "%(title)s.%(ext)s"]));
        assert_eq!(args.last().map(String::as_str), Some("https://example.com/watch?v=abc123"));
    }

    #[test]
    fn parse_download_progress_extracts_percent_speed_and_eta() {
        let snapshot = parse_download_progress(
            "[download]  42.3% of 100.00MiB at 3.20MiB/s ETA 00:18",
            "",
            Some(0),
        );

        assert!((snapshot.progress - 0.423).abs() < f64::EPSILON);
        assert_eq!(snapshot.speed_text.as_deref(), Some("3.20MiB/s"));
        assert_eq!(snapshot.eta_text.as_deref(), Some("00:18"));
    }

    #[test]
    fn control_download_updates_real_task_state_and_process_handle() {
        let queue_state = Mutex::new(QueueState::default());
        let download_handles = Mutex::new(Vec::<DownloadHandle>::new());
        let task = create_download_task(
            "https://example.com/watch?v=control".to_string(),
            DownloadOptions::default(),
            &queue_state,
            &download_handles,
        );
        set_download_process_id(&download_handles, &task.task_id, Some(4242))
            .expect("task handle should exist");
        queue_service::mark_downloading(&queue_state, &task.task_id).expect("task should exist");

        let calls = Mutex::new(Vec::<String>::new());

        let paused = control_download_with(
            &task.task_id,
            &queue_state,
            &download_handles,
            DownloadControlAction::Pause,
            |action, pid| {
                calls
                    .lock()
                    .expect("control call lock poisoned")
                    .push(format!("{action:?}:{pid}"));
                Ok::<(), String>(())
            },
        )
        .expect("pause should succeed");
        let resumed = control_download_with(
            &task.task_id,
            &queue_state,
            &download_handles,
            DownloadControlAction::Resume,
            |action, pid| {
                calls
                    .lock()
                    .expect("control call lock poisoned")
                    .push(format!("{action:?}:{pid}"));
                Ok::<(), String>(())
            },
        )
        .expect("resume should succeed");
        let cancelled = control_download_with(
            &task.task_id,
            &queue_state,
            &download_handles,
            DownloadControlAction::Cancel,
            |action, pid| {
                calls
                    .lock()
                    .expect("control call lock poisoned")
                    .push(format!("{action:?}:{pid}"));
                Ok::<(), String>(())
            },
        )
        .expect("cancel should succeed");

        let snapshot = queue_service::snapshot(&queue_state);
        let handles = download_handles
            .lock()
            .expect("download handles lock poisoned")
            .clone();

        assert_eq!(paused.state, "paused");
        assert_eq!(resumed.state, "downloading");
        assert_eq!(cancelled.state, "cancelled");
        assert_eq!(
            calls.lock().expect("control call lock poisoned").clone(),
            vec![
                "Pause:4242".to_string(),
                "Resume:4242".to_string(),
                "Cancel:4242".to_string()
            ]
        );
        assert_eq!(snapshot.tasks[0].state, "cancelled");
        assert!(snapshot.active_task_id.is_none());
        assert_eq!(handles[0].child_process_id, None);
    }
}

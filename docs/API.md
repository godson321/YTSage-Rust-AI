# API Design

## 1. Boundary Style

Frontend/backend communication should use:

- `invoke(command, payload)` for request/response operations
- Tauri events for async progress, queue changes, and logs

This project should not introduce an internal HTTP server.

## 2. Current Contract Files

- Frontend DTOs: `src/types/models.ts`
- Frontend API wrapper: `src/services/tauri/api.ts`
- Backend DTOs: `src-tauri/src/models/mod.rs`
- Backend commands: `src-tauri/src/commands/mod.rs`

## 3. Current Commands

| Command | Input | Output | Status |
|---|---|---|---|
| `analyze_urls` | `AnalyzeRequest` | `AnalysisResult` | implemented as placeholder |
| `create_download_task` | `sourceUrl`, `DownloadOptions` | `DownloadTask` | implemented as placeholder |
| `get_settings` | none | `AppSettings` | implemented |
| `save_settings` | `AppSettings` | `AppSettings` | implemented in-memory |
| `list_history` | none | `HistoryEntry[]` | implemented as placeholder |
| `get_tool_status` | none | `ToolStatus[]` | implemented as placeholder |

## 4. Recommended Next Commands

### 4.1 Analysis

| Command | Purpose |
|---|---|
| `analyze_urls` | keep as unified single/multi URL analysis entry |
| `get_analysis_job` | optional if analysis becomes queued/async |

### 4.2 Queue and Download

| Command | Purpose |
|---|---|
| `enqueue_downloads` | create one or many download tasks from analyzed items |
| `list_tasks` | get current queue/task snapshot |
| `pause_task` | pause a running task |
| `resume_task` | resume a paused task |
| `cancel_task` | cancel a running or queued task |
| `retry_task` | clone/restart failed task |
| `remove_task` | remove task from UI queue |
| `clear_finished_tasks` | clean completed queue entries |

### 4.3 Settings and Filesystem

| Command | Purpose |
|---|---|
| `get_settings` | keep |
| `save_settings` | keep, but persist to disk |
| `pick_directory` | native folder picker |
| `open_path` | open file or folder in OS shell |

### 4.4 History

| Command | Purpose |
|---|---|
| `list_history` | keep, but add filtering/pagination later |
| `delete_history_entry` | remove history row |
| `redownload_history_entry` | recreate a task from history |

### 4.5 Tools

| Command | Purpose |
|---|---|
| `get_tool_status` | keep |
| `detect_tools` | explicit refresh |
| `update_tool` | update `yt-dlp` / `ffmpeg` / `deno` |

### 4.6 Cookies and Diagnostics

| Command | Purpose |
|---|---|
| `apply_cookie_file` | activate `cookies.txt` |
| `set_browser_cookie_source` | remember browser cookie source |
| `get_cookie_state` | fetch active cookie config |
| `list_recent_logs` | initial log page snapshot |
| `export_diagnostics_bundle` | zip logs/config/task info |

## 5. Recommended Event Surface

| Event | Payload | Purpose |
|---|---|---|
| `analysis:progress` | analysis progress DTO | optional if analysis is async |
| `analysis:completed` | `AnalysisResult` | push analysis completion |
| `analysis:failed` | error DTO | push analysis errors |
| `queue:changed` | queue snapshot DTO | refresh batch/task center |
| `task:created` | `DownloadTask` | task inserted |
| `task:progress` | task progress DTO | live download progress |
| `task:state_changed` | task state DTO | queued/downloading/failed/completed |
| `task:finished` | finished task DTO | completion action |
| `task:failed` | error DTO | show failure details |
| `history:changed` | history summary DTO | refresh history page |
| `tools:status_changed` | tool status DTO | refresh tool page |
| `tools:update_progress` | tool update DTO | show binary updates |
| `log:entry` | log entry DTO | live diagnostics stream |

## 6. DTO Baseline

### 6.1 Existing DTOs

Existing DTO families already defined:

- `AnalyzeRequest`
- `AnalysisResult`
- `AnalysisItem`
- `MediaInfo`
- `FormatItem`
- `DownloadOptions`
- `DownloadTask`
- `AppSettings`
- `HistoryEntry`
- `ToolStatus`

### 6.2 DTO Extensions Needed

| DTO | Needed additions |
|---|---|
| `MediaInfo` | publish date, view count, uploader/channel URL, thumbnail variants |
| `FormatItem` | fps, hdr, codecs, bitrate, note, container detail |
| `AnalysisItem` | subtitles, playlist entries, warning list |
| `DownloadOptions` | subtitle list, sponsorblock, trim section, proxy, output format preference |
| `DownloadTask` | batch ID, timestamps, downloaded bytes, current filename |
| `HistoryEntry` | completed time, file size, format summary, thumbnail path |
| `ToolStatus` | latest version, update available, auto-managed flag |

## 7. Error Model

Recommended normalized error shape:

```json
{
  "code": "analysis.signin_required",
  "message": "Sign-in is required for this item.",
  "detail": "Original yt-dlp stderr or mapped reason",
  "recoverable": true
}
```

Rules:

- backend maps raw stderr into stable codes where possible
- frontend renders user-facing messages from stable codes
- raw tool stderr can still be attached for diagnostics

## 8. Contract Conventions

- JSON field names stay `camelCase`
- Rust structs use `#[serde(rename_all = "camelCase")]`
- frontend types and Rust models must be updated together
- task state enum values stay string-based and backend-owned

## 9. Suggested Sequence for API Implementation

1. complete `analyze_urls`
2. persist `get_settings/save_settings`
3. add `enqueue_downloads` and task events
4. add `list_tasks/pause/resume/cancel/retry`
5. add history mutation commands
6. add tool detection/update commands
7. add cookies/logs/diagnostics commands

# Architecture

## 1. Goal

Rebuild YTSage as a modern desktop application with:

- `Vue 3 + TypeScript + Element Plus` for UI
- `Tauri 2 + Rust` for desktop/backend
- external `yt-dlp`, `ffmpeg`, and `deno` for media work

The rewrite scope is the application shell, task model, state flow, persistence, and UX.
The rewrite scope is not replacing the extractor/downloader core implemented by third-party tools.

## 2. High-Level Architecture

```text
Vue Pages / Pinia Stores
        |
        v
Tauri invoke commands + event listeners
        |
        v
Rust services / state / process orchestration
        |
        v
yt-dlp / ffmpeg / deno / filesystem / OS dialogs
```

## 3. Architectural Rules

- No internal REST server.
- All privileged local capabilities stay in Rust.
- Vue never builds raw `yt-dlp` commands directly.
- Rust is the source of truth for queue state, download state, and tool state.
- Cross-boundary DTOs stay aligned between `src/types/models.ts` and `src-tauri/src/models/mod.rs`.
- Async updates should use Tauri events instead of frontend polling loops where possible.

## 4. Frontend Structure

### 4.1 Current Skeleton

| Route | Current file | Status | Purpose |
|---|---|---|---|
| `/download` | `src/pages/DownloadPage.vue` | Implemented skeleton | URL input and analysis result entry |
| `/batch` | `src/pages/BatchPage.vue` | Implemented skeleton | Placeholder for batch queue |
| `/history` | `src/pages/HistoryPage.vue` | Implemented skeleton | Placeholder history page |
| `/settings` | `src/pages/SettingsPage.vue` | Implemented skeleton | Reads/saves settings |
| `/tools` | `src/pages/ToolsPage.vue` | Implemented skeleton | Placeholder tool management |
| `/logs` | `src/pages/LogsPage.vue` | Implemented skeleton | Placeholder diagnostics page |
| `/about` | `src/pages/AboutPage.vue` | Implemented skeleton | Placeholder about page |

Shared shell:

- `src/components/layout/AppShell.vue`
- `src/router/index.ts`
- `src/main.ts`
- `src/App.vue`

Current frontend stores:

- `src/stores/analysis.ts`
- `src/stores/settings.ts`

### 4.2 Target Frontend Modules

| Module | Responsibility | Notes |
|---|---|---|
| `analysis` | parse URLs, hold analysis result, normalize errors | used by `/download` and `/batch` |
| `queue` | batch tasks, task ordering, retry/remove controls | not built yet |
| `download` | live progress, pause/resume/cancel, active task detail | not built yet |
| `history` | list/search/re-download history | not built yet |
| `settings` | app settings, persistence sync | basic skeleton exists |
| `tools` | binary state, install/update progress | not built yet |
| `cookies` | cookie source state and helper flows | not built yet |
| `logs` | runtime log stream and filtering | not built yet |

## 5. Rust Backend Structure

### 5.1 Current Skeleton

| Module | Current file | Status | Purpose |
|---|---|---|---|
| app entry | `src-tauri/src/lib.rs` | Implemented | registers commands and state |
| command layer | `src-tauri/src/commands/mod.rs` | Implemented skeleton | invoke entrypoints |
| DTO layer | `src-tauri/src/models/mod.rs` | Implemented skeleton | shared backend DTOs |
| state layer | `src-tauri/src/state/mod.rs` | Implemented skeleton | in-memory settings state |
| analysis service | `src-tauri/src/services/analysis_service.rs` | Placeholder | returns mock analysis data |
| history service | `src-tauri/src/services/history_service.rs` | Placeholder | returns empty history |
| tools service | `src-tauri/src/services/tools_service.rs` | Placeholder | returns empty tool state |

### 5.2 Target Backend Services

| Service | Type | Responsibility |
|---|---|---|
| `settings_service` | command style | read/write config and migrate old config |
| `analysis_service` | command + event | call `yt-dlp` for single/batch analysis |
| `queue_service` | long-lived manager | task queue, order, retries, concurrency |
| `download_service` | long-lived manager | create and control download tasks |
| `process_service` | shared infra | subprocess lifecycle and stdout/stderr parsing |
| `history_service` | command style | persist and query download history |
| `tools_service` | command + event | detect/update `yt-dlp`, `ffmpeg`, `deno` |
| `cookie_service` | command style | cookie file/browser source management |
| `log_service` | shared infra | structured logs and UI log streaming |
| `file_service` | command style | open path, choose folder, path validation |
| `update_service` | command style | app update checks and metadata |

## 6. State and Persistence

Current state:

- only `AppState.settings` in memory via `Mutex<AppSettings>`

Target persistence split:

| Data | Proposed storage | Reason |
|---|---|---|
| app settings | config file | simple user configuration |
| history | SQLite | query/filter/re-download support |
| queue snapshot | SQLite or local state file | resume and crash recovery |
| log files | rotating text logs | support diagnostics export |
| thumbnails/cache | filesystem cache directory | avoid bloating database |
| tool paths/versions | config + refresh check | stable startup behavior |

## 7. Interaction Model

### 7.1 Analyze Flow

```text
DownloadPage / BatchPage
  -> invoke analyze_urls(payload)
  -> Rust validates input and prepares yt-dlp args
  -> yt-dlp returns structured JSON
  -> Rust maps JSON to AnalysisResult
  -> Vue store updates result list
```

### 7.2 Download Flow

```text
Vue page submits task intent
  -> invoke create/enqueue command
  -> queue_service creates task state
  -> download_service starts child process
  -> process_service parses progress
  -> Rust emits task progress/state events
  -> Vue updates queue and task views
```

## 8. Why Tauri Command + Event

This project is a local desktop app, not a browser-first SaaS product.

Using Tauri IPC keeps:

- deployment simpler
- local capability boundaries clear
- frontend/backend contracts typed
- future packaging cleaner
- queue/process ownership inside Rust instead of leaking into the UI

## 9. Current Gap Summary

Already in place:

- app shell
- typed contracts baseline
- route layout
- settings load/save path
- placeholder backend services

Still missing:

- real binary invocation
- event-driven progress model
- persistent history/settings
- real queue/download manager
- logs and diagnostics pipeline
- cookie strategy

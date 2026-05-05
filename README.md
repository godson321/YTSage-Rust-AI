# YTSage Rust

YTSage Rust is the planned desktop rewrite of YTSage.

- Frontend: `Vue 3 + TypeScript + Element Plus + Pinia + Vue Router`
- Desktop shell: `Tauri 2`
- Local backend: `Rust`
- Media engine: keep calling external `yt-dlp`, `ffmpeg`, and `deno`

This repository is currently a working skeleton, not a feature-complete replacement for the original Python/PySide app.

## Current Status

Implemented now:

- Vite + Vue + Element Plus frontend skeleton
- Tauri + Rust backend skeleton
- Typed frontend/backend contract baseline
- Primary route shell
- Placeholder analysis/settings/history/tools commands
- Frontend and Rust unit tests
- Basic app icon resources
- Project planning and architecture docs

Not implemented yet:

- Real `yt-dlp` analysis integration
- Real download queue and progress pipeline
- Persistent history/config storage
- Cookie management flow
- Tool install/update flow
- Packaging and release pipeline

## Current App Structure

Frontend routes already present:

- `/download`
- `/batch`
- `/history`
- `/settings`
- `/tools`
- `/logs`
- `/about`

Current Rust command surface:

- `analyze_urls`
- `create_download_task`
- `get_settings`
- `save_settings`
- `list_history`
- `get_tool_status`

Current contract sources:

- `src/types/models.ts`
- `src/services/tauri/api.ts`
- `src-tauri/src/models/mod.rs`
- `src-tauri/src/commands/mod.rs`

## Run

Install dependencies:

```powershell
npm.cmd install
```

Run frontend tests:

```powershell
npm.cmd run test
```

Build frontend:

```powershell
npm.cmd run build
```

Run Rust tests:

```powershell
Set-Location src-tauri
C:\Users\godson321\.cargo\bin\cargo.exe test
```

Run desktop app in dev mode:

```powershell
npm.cmd run tauri:dev
```

## Documents

- [对齐总账](docs/alignment-master.md) - YTSage 功能对齐状态的唯一来源
- [Architecture](docs/ARCHITECTURE.md)
- [Analysis Report](docs/ANALYSIS_REPORT.md)
- [API Design](docs/API.md)
- [Execution Plan](docs/PLAN.md)
- [Task Board](docs/TASKS.md)
- [Progress Log](docs/PROGRESS.md)

## Core Decisions

- Do not build an internal HTTP server for the desktop app.
- Use `Tauri command + event` as the frontend/backend boundary.
- Keep `yt-dlp`, `ffmpeg`, and `deno` as external tools rather than rewriting their core behavior.
- Let Rust own task state, process management, persistence, and filesystem access.
- Keep Vue focused on page state, interaction, and rendering.

## Validation Snapshot

Validated during skeleton setup on `2026-05-05`:

- `npm.cmd install`
- `npm.cmd run build`
- `npm.cmd run test`
- `cargo test` in `src-tauri`

## Near-Term Goal

The next functional milestone is:

1. Connect `analysis_service` to real `yt-dlp --dump-single-json`
2. Persist settings to disk
3. Add queue/download services and task events
4. Make `/download` and `/batch` use real backend state

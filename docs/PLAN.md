# Execution Plan

## 1. Objective

Ship a usable Rust/Tauri replacement for the current YTSage desktop workflow while preserving external `yt-dlp`, `ffmpeg`, and `deno` execution.

## 2. Planning Principles

- keep milestones vertical and testable
- stabilize contracts before feature fan-out
- move task/process ownership into Rust early
- postpone risky cookie/login work until core flows are stable
- prefer shipping working queue/download paths before polishing edge features

## 3. Milestones

| Milestone | Status | Goal | Exit criteria |
|---|---|---|---|
| M0 Foundation | Done | create runnable frontend/backend skeleton | app starts, routes render, tests pass |
| M1 Contracts and Docs | Done | lock architecture and task plan | docs exist and reflect current code |
| M2 Real Analysis | Next | replace placeholder analysis with `yt-dlp` JSON parsing | `/download` can parse real URLs |
| M3 Settings and Tools | Planned | persist settings and detect binaries | settings survive restart, tool page shows real state |
| M4 Queue and Download | Planned | add Rust-owned queue and download pipeline | batch tasks can run with live progress |
| M5 History and Logs | Planned | persist history and expose diagnostics | history/log pages are functional |
| M6 Advanced Options | Planned | subtitles, sponsorblock, trim, proxy, generic mode | advanced options map into task creation |
| M7 Cookie Strategy | Planned | cookie file/browser source baseline | authenticated flows are configurable |
| M8 Packaging | Planned | prepare desktop release artifacts | local build/release pipeline is stable |

## 4. Phase Breakdown

### Phase 1: Real Analysis

Deliverables:

- invoke real `yt-dlp`
- parse JSON into stable DTOs
- preserve one `AnalysisItem` per input URL
- return partial success for mixed batches

Key modules:

- `src-tauri/src/services/analysis_service.rs`
- `src-tauri/src/models/mod.rs`
- `src/stores/analysis.ts`
- `src/pages/DownloadPage.vue`

### Phase 2: Settings and Tool Discovery

Deliverables:

- config file persistence
- real download path and filename template handling
- detect `yt-dlp`, `ffmpeg`, and `deno`
- refresh tool status from UI

Key modules:

- `src-tauri/src/state`
- `src-tauri/src/services/tools_service.rs`
- future `settings_service`
- `src/pages/SettingsPage.vue`
- `src/pages/ToolsPage.vue`

### Phase 3: Queue and Download Core

Deliverables:

- queue manager
- task state machine
- progress events
- pause/resume/cancel/retry

Key modules:

- future `queue_service`
- future `download_service`
- future `process_service`
- `src/pages/BatchPage.vue`
- future queue/download stores

### Phase 4: Persistence and Diagnostics

Deliverables:

- history database
- live log stream
- diagnostics export

### Phase 5: Advanced YTSage Features

Deliverables:

- playlist selection
- subtitle selection/merge
- sponsorblock mapping
- trim/rate-limit/proxy options
- generic mode support

### Phase 6: Cookie and Authentication Strategy

Deliverables:

- stable `cookies.txt` support
- browser-cookie source configuration
- explicit limits documented for member-only/bot-protected content

Embedded login should not block M2-M5.

### Phase 7: Packaging and Release

Deliverables:

- release bundling
- external tool placement strategy
- upgrade story
- release checklist

## 5. Immediate Next Slice

Recommended immediate slice:

1. implement real `analysis_service`
2. extend `MediaInfo` and `FormatItem` DTOs as needed
3. show real errors per URL instead of placeholder success
4. preserve current batch textarea flow in `/download`

## 6. Acceptance Strategy

Each milestone should end with:

- Rust unit tests
- frontend unit tests where applicable
- one manual desktop smoke test
- docs updated if the architecture or API changes

## 7. Deferred Work

Intentionally deferred until later phases:

- embedded browser login
- release updater
- cross-platform distribution polish
- nonessential UI polish beyond the core flows

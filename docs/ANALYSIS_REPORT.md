# Analysis Report

## 1. Inputs Used

This rewrite analysis is based on:

- the original Python project under `F:\Github\YTSage`
- the current Tauri/Vue skeleton under `F:\Github\YTSage-Rust`
- three parallel analysis tracks already consolidated:
  - frontend page decomposition
  - Rust backend service boundaries
  - frontend/backend contract design

## 2. Rewrite Constraints

Hard constraints:

- `yt-dlp`, `ffmpeg`, and `deno` remain external tools
- the rewrite focuses on application architecture and UX
- no internal HTTP API server
- frontend uses `Vue 3 + TS + Element Plus`
- backend uses `Rust` through `Tauri`

Practical implications:

- media parsing/downloading is wrapped, not reimplemented
- anti-bot and membership access cannot be solved purely by UI rewrite
- task state must move out of UI code and into Rust

## 3. Old-to-New Capability Mapping

| Capability | Old project basis | Rewrite feasibility | New home | Notes |
|---|---|---|---|---|
| single URL analysis | `ytsage_gui_analysis.py` | Yes | `analysis_service` + `/download` | high priority |
| multi-URL analysis | main window input flow | Yes | `analysis_service` + `/batch` | should be backend-driven |
| playlist/channel parsing | yt-dlp based | Yes | `analysis_service` | needs richer DTOs |
| format table | GUI table widgets | Yes | Vue table components | straightforward |
| download execution | current Python downloader/thread model | Yes | `download_service` | keep external binaries |
| batch queue | partial/implicit in old app | Yes | `queue_service` + `/batch` | core rewrite value |
| history | history dialog/manager | Yes | `history_service` + `/history` | good Rust fit |
| settings | config manager + dialogs | Yes | `settings_service` + `/settings` | good early milestone |
| tool detection/update | yt-dlp/ffmpeg/deno helpers | Yes | `tools_service` + `/tools` | must stay separate from downloads |
| logs/diagnostics | logger + console output | Yes | `log_service` + `/logs` | needed for supportability |
| cookies.txt import | old custom dialog flow | Yes | `cookie_service` | feasible |
| browser cookie source | old cookie options | Partial | `cookie_service` | platform/browser edge cases remain |
| embedded login | old WebEngine experiments | Partial/high risk | later cookie/login flow | do after core queue/download is stable |
| proxy/generic mode | current settings/options | Yes | settings + request DTOs | easy contract fit |
| subtitles/sponsorblock/trim | yt-dlp/ffmpeg option mapping | Yes | download option DTOs | phase after core download flow |
| localization | old language files | Yes | frontend i18n + backend messages | later phase |
| app packaging | Python packaging today | Yes | Tauri bundle pipeline | separate release milestone |

## 4. What the Rewrite Can and Cannot Solve

### 4.1 Can Solve

- modern page-based UI
- stronger type contracts
- real batch queue ownership
- cleaner process management
- better logs and diagnostics
- cleaner persistence model
- less UI-thread coupling than the Python desktop app

### 4.2 Cannot Solve by Itself

- replacing `yt-dlp` extractor behavior
- bypassing YouTube membership restrictions
- bypassing YouTube bot detection by app UI alone
- guaranteeing browser-cookie extraction on every browser/profile/platform combination

Those remain dependent on:

- user account state
- `yt-dlp` compatibility
- browser storage behavior
- upstream site changes

## 5. Current Skeleton Assessment

Strengths already present:

- frontend route frame is established
- backend DTO contract exists on both sides
- settings and analysis code paths already have stable entrypoints
- tests/build are already green for the skeleton

Current limitations:

- backend services return placeholders only
- no real event model yet
- no queue or process manager yet
- no real persistence yet
- no production error taxonomy yet

## 6. Recommended Build Order

Recommended execution order:

1. make analysis real
2. persist settings and tool paths
3. add queue/download/process services
4. add history/logs/tool management pages
5. add advanced yt-dlp options
6. revisit cookie/login strategy only after core workflows are stable
7. finalize packaging and release

## 7. Main Risks

| Risk | Impact | Mitigation |
|---|---|---|
| `yt-dlp` JSON variance across URL types | wrong analysis mapping | normalize through Rust DTO adapters |
| download progress parsing instability | broken progress UI | centralize parser in `process_service` |
| queue state bugs | failed batch downloads | keep queue as Rust-owned state machine |
| cookie/browser edge cases | login appears flaky | treat as optional module, keep file cookies first |
| packaging external binaries | broken release artifacts | separate tools/bin strategy early |
| contract drift between TS and Rust | runtime failures | treat `models.ts` and `models/mod.rs` as paired files |

## 8. Summary

The rewrite is viable.

The correct scope is:

- rewrite UI, state flow, process orchestration, persistence, and packaging
- do not rewrite the extractor/downloader core provided by third-party binaries

The highest-value differentiator of the Rust rewrite is not cosmetic UI.
It is moving batch analysis, queue state, and process control into a reliable backend layer.

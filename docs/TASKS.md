# Task Board

## Done

- [x] T001 Create `Vue 3 + TS + Element Plus + Pinia + Router` skeleton
- [x] T002 Create `Tauri 2 + Rust` skeleton
- [x] T003 Add shared frontend/backend DTO baseline
- [x] T004 Add initial routes for download, batch, history, settings, tools, logs, about
- [x] T005 Add placeholder analysis/settings/history/tools command layer
- [x] T006 Add frontend and Rust tests for the skeleton
- [x] T007 Add rewrite documentation set

## Next

- [ ] T101 Replace placeholder `analysis_service` with real `yt-dlp` JSON analysis
- [ ] T102 Add structured mapping from raw analysis JSON to `MediaInfo` and `FormatItem`
- [ ] T103 Return per-URL partial failure instead of blanket placeholder success
- [ ] T104 Improve `/download` result table to show richer status and media metadata
- [ ] T105 Add analysis error code normalization

## Queue and Download Core

- [ ] T201 Create `queue_service`
- [ ] T202 Create `download_service`
- [ ] T203 Create `process_service` for child process lifecycle and output parsing
- [ ] T204 Add Tauri task events: progress, state change, finish, fail
- [ ] T205 Add frontend queue store
- [ ] T206 Add frontend download/task center store
- [ ] T207 Turn `/batch` into a real queue page
- [ ] T208 Implement pause/resume/cancel/retry controls

## Settings and Tools

- [ ] T301 Persist settings to disk
- [ ] T302 Add native directory picker
- [ ] T303 Detect actual `yt-dlp`, `ffmpeg`, and `deno` paths/versions
- [ ] T304 Add tool refresh/update command design
- [ ] T305 Expand `/settings` page sections beyond the current basic form
- [ ] T306 Turn `/tools` into a real management page

## History and Logs

- [ ] T401 Add history persistence schema
- [ ] T402 Write successful downloads into history
- [ ] T403 Build `/history` list/search/re-download flow
- [ ] T404 Add structured log file output
- [ ] T405 Stream logs to `/logs`
- [ ] T406 Add diagnostics export

## Advanced Media Features

- [ ] T501 Add subtitle track DTOs and selection flow
- [ ] T502 Add sponsorblock option mapping
- [ ] T503 Add trim/rate-limit/proxy/download-section options
- [ ] T504 Add playlist item selection
- [ ] T505 Add generic mode handling end-to-end

## Cookie Strategy

- [ ] T601 Add `cookies.txt` apply flow
- [ ] T602 Add browser-cookie source config
- [ ] T603 Document authenticated-content limitations
- [ ] T604 Evaluate whether embedded login is still worth implementing after file/browser cookie support is stable

## Packaging

- [ ] T701 Define app data, cache, and tools directory layout
- [ ] T702 Define external binary bundling strategy
- [ ] T703 Enable Tauri bundle pipeline
- [ ] T704 Create local release checklist

## Notes

- `T101-T105` are the highest-value next slice.
- `T201-T208` are the real differentiator versus the old UI-thread-heavy model.
- `T601-T604` are intentionally later than queue/download core.

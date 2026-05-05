# YTSage-Rust Alignment Checklist Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring YTSage-Rust to feature parity with the core YTSage desktop workflow.

**Architecture:** Keep yt-dlp, ffmpeg, and deno external. Move analysis, queue state, settings, history, tool detection, and task control into Rust services, and keep Vue focused on display and user intent. Stabilize contracts first, then add event-driven task flow, then fill in advanced features.

**Tech Stack:** Vue 3, TypeScript, Element Plus, Pinia, Tauri 2, Rust, yt-dlp, ffmpeg, deno, Vitest, cargo test

---

### Task 1: Lock the analysis contract against real YTSage behavior

**Files:**
- Modify: `src-tauri/src/services/analysis_service.rs`
- Modify: `src-tauri/src/models/mod.rs`
- Modify: `src/types/models.ts`
- Modify: `src/pages/DownloadPage.vue`
- Test: `src-tauri/src/services/analysis_service.rs` tests
- Test: `src/tests/url.test.ts`

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn youtube_membership_regression_matches_original_ytsage_behavior() { /* ... */ }

#[test]
fn youtube_playlist_regression_treats_public_playlist_as_accessible() { /* ... */ }
```

```ts
describe("url extraction", () => {
  it("extracts the three youtube regression urls in order", () => {
    expect(extractUrlsFromText(text)).toEqual([
      "https://www.youtube.com/watch?v=oyPhmcgVoSY",
      "https://www.youtube.com/watch?v=9E9y-suOleI",
      "https://www.youtube.com/watch?v=ZX_NwrgmYFk"
    ]);
  });
});
```

- [ ] **Step 2: Run the targeted tests**

Run:
```powershell
npm.cmd run test
cd src-tauri
C:\Users\godson321\.cargo\bin\cargo.exe test
```

Expected: existing analysis tests fail until the runner and DTO mappings match the new regression cases.

- [ ] **Step 3: Implement the minimal contract alignment**

```rust
// keep structured AnalysisError codes
// preserve playlist candidate normalization
// preserve per-URL item results
```

- [ ] **Step 4: Re-run the tests**

Run:
```powershell
npm.cmd run test
cd src-tauri
C:\Users\godson321\.cargo\bin\cargo.exe test
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add src-tauri/src/services/analysis_service.rs src-tauri/src/models/mod.rs src/types/models.ts src/pages/DownloadPage.vue src/tests/url.test.ts
git commit -m "test: lock ytsage analysis regressions"
```

### Task 2: Replace placeholder download creation with a Rust-owned task model

**Files:**
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/models/mod.rs`
- Create: `src-tauri/src/services/queue_service.rs`
- Create: `src-tauri/src/services/download_service.rs`
- Create: `src-tauri/src/services/process_service.rs`
- Modify: `src-tauri/src/services/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/stores/analysis.ts`
- Create: `src/stores/queue.ts`
- Create: `src/stores/download.ts`
- Modify: `src/pages/BatchPage.vue`
- Test: `src-tauri/src/services/*` tests
- Test: `src/stores/*.test.ts`

- [ ] **Step 1: Write the failing task-state test**

```rust
#[test]
fn create_download_task_returns_queued_task_with_real_fields() {
    // assert task_id, source_url, state, progress, title
}
```

- [ ] **Step 2: Run the Rust test**

Run:
```powershell
cd src-tauri
C:\Users\godson321\.cargo\bin\cargo.exe test create_download_task_returns_queued_task_with_real_fields
```

Expected: FAIL until the task model is real.

- [ ] **Step 3: Implement the queue/download/process skeleton**

```rust
pub struct QueueState { /* task list, active task, retry count */ }
pub struct DownloadHandle { /* child process, task id */ }
pub struct ProcessSnapshot { /* stdout, stderr, exit code, progress */ }
```

- [ ] **Step 4: Re-run the Rust test**

Run:
```powershell
cd src-tauri
C:\Users\godson321\.cargo\bin\cargo.exe test
```

Expected: PASS.

- [ ] **Step 5: Wire the frontend stores**

```ts
// queue store: enqueue, remove, retry, pause, resume
// download store: active task, progress, error, completed items
```

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/commands/mod.rs src-tauri/src/models/mod.rs src-tauri/src/services/queue_service.rs src-tauri/src/services/download_service.rs src-tauri/src/services/process_service.rs src-tauri/src/services/mod.rs src-tauri/src/lib.rs src/stores/analysis.ts src/stores/queue.ts src/stores/download.ts src/pages/BatchPage.vue
git commit -m "feat: add rust-owned queue and download skeleton"
```

### Task 3: Persist settings and surface real tool status

**Files:**
- Modify: `src-tauri/src/state/mod.rs`
- Create: `src-tauri/src/services/settings_service.rs`
- Modify: `src-tauri/src/services/tools_service.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src/pages/SettingsPage.vue`
- Modify: `src/pages/ToolsPage.vue`
- Modify: `src/stores/settings.ts`
- Test: `src-tauri/src/services/settings_service.rs` tests

- [ ] **Step 1: Write the failing settings persistence test**

```rust
#[test]
fn save_settings_persists_download_path_and_language() {
    // save, reload, assert values survive
}
```

- [ ] **Step 2: Run the Rust test**

Run:
```powershell
cd src-tauri
C:\Users\godson321\.cargo\bin\cargo.exe test save_settings_persists_download_path_and_language
```

Expected: FAIL until file-backed config exists.

- [ ] **Step 3: Implement file-backed settings**

```rust
// read config on startup
// write config on save
// keep state as a cache, not the source of truth
```

- [ ] **Step 4: Re-run the settings test**

Run:
```powershell
cd src-tauri
C:\Users\godson321\.cargo\bin\cargo.exe test
```

Expected: PASS.

- [ ] **Step 5: Replace tool placeholders**

```rust
// detect yt-dlp, ffmpeg, deno
// report installed state, current version, and path
```

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/state/mod.rs src-tauri/src/services/settings_service.rs src-tauri/src/services/tools_service.rs src-tauri/src/commands/mod.rs src/pages/SettingsPage.vue src/pages/ToolsPage.vue src/stores/settings.ts
git commit -m "feat: persist settings and detect tools"
```

### Task 4: Add history and logs as real product surfaces

**Files:**
- Create: `src-tauri/src/services/history_store.rs`
- Modify: `src-tauri/src/services/history_service.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src/pages/HistoryPage.vue`
- Modify: `src/pages/LogsPage.vue`
- Create: `src/stores/history.ts`
- Create: `src/stores/logs.ts`
- Test: `src-tauri/src/services/history_service.rs` tests

- [ ] **Step 1: Write the failing history test**

```rust
#[test]
fn list_history_returns_saved_entries() {
    // insert one item, list one item
}
```

- [ ] **Step 2: Run the history test**

Run:
```powershell
cd src-tauri
C:\Users\godson321\.cargo\bin\cargo.exe test list_history_returns_saved_entries
```

Expected: FAIL until persistence exists.

- [ ] **Step 3: Implement history persistence**

```rust
// save successful downloads
// query by recency
// support re-download intent
```

- [ ] **Step 4: Add log streaming**

```rust
// write structured logs to file
// expose tail/stream command for UI
```

- [ ] **Step 5: Commit**

```powershell
git add src-tauri/src/services/history_store.rs src-tauri/src/services/history_service.rs src-tauri/src/commands/mod.rs src/pages/HistoryPage.vue src/pages/LogsPage.vue src/stores/history.ts src/stores/logs.ts
git commit -m "feat: add history and log surfaces"
```

### Task 5: Fill in advanced media options end to end

**Files:**
- Modify: `src-tauri/src/models/mod.rs`
- Modify: `src-tauri/src/services/analysis_service.rs`
- Create: `src-tauri/src/services/cookie_service.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src/pages/DownloadPage.vue`
- Modify: `src/pages/BatchPage.vue`
- Modify: `src/pages/SettingsPage.vue`
- Test: `src-tauri/src/services/analysis_service.rs` tests

- [ ] **Step 1: Write the failing option-mapping tests**

```rust
#[test]
fn analysis_payload_preserves_cookie_and_proxy_options() {
    // assert options are forwarded to yt-dlp command construction
}
```

- [ ] **Step 2: Run the option test**

Run:
```powershell
cd src-tauri
C:\Users\godson321\.cargo\bin\cargo.exe test analysis_payload_preserves_cookie_and_proxy_options
```

Expected: FAIL until the option path is wired through all commands.

- [ ] **Step 3: Implement cookie and option plumbing**

```rust
// cookies.txt path
// cookies-from-browser source
// proxy, geo proxy, subtitle, sponsorblock, trim, rate limit
```

- [ ] **Step 4: Re-run the tests**

Run:
```powershell
cd src-tauri
C:\Users\godson321\.cargo\bin\cargo.exe test
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add src-tauri/src/models/mod.rs src-tauri/src/services/analysis_service.rs src-tauri/src/services/cookie_service.rs src-tauri/src/commands/mod.rs src/pages/DownloadPage.vue src/pages/BatchPage.vue src/pages/SettingsPage.vue
git commit -m "feat: wire advanced yt-dlp options"
```

### Task 6: Finalize packaging and release readiness

**Files:**
- Modify: `README.md`
- Modify: `docs/PLAN.md`
- Modify: `docs/TASKS.md`
- Create: `docs/superpowers/plans/2026-05-05-ytsage-rust-alignment-checklist.md`
- Create: release checklist docs as needed

- [ ] **Step 1: Review the completed feature map**

```text
analysis, queue, download, settings, tools, history, logs, cookies, advanced options, packaging
```

- [ ] **Step 2: Add release and validation notes**

```markdown
- build command
- test command
- smoke test command
- packaging prerequisites
```

- [ ] **Step 3: Run full validation**

Run:
```powershell
npm.cmd run test
npm.cmd run build
cd src-tauri
C:\Users\godson321\.cargo\bin\cargo.exe test
```

Expected: PASS.

- [ ] **Step 4: Commit**

```powershell
git add README.md docs/PLAN.md docs/TASKS.md docs/superpowers/plans/2026-05-05-ytsage-rust-alignment-checklist.md
git commit -m "docs: add ytsage alignment implementation plan"
```

## Self-Review

### Coverage
- Analysis parity: covered in Task 1.
- Queue/download core: covered in Task 2.
- Settings/tools persistence: covered in Task 3.
- History/logs: covered in Task 4.
- Advanced options and cookies: covered in Task 5.
- Packaging/release readiness: covered in Task 6.

### Gaps
- Embedded browser login is intentionally deferred until file/browser cookie support is stable.
- Platform-specific packaging details still need a follow-up release plan after the core flows are complete.


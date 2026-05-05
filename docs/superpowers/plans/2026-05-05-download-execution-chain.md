# 真实下载执行链与队列事件同步实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 YTSage-Rust 打通最小真实下载执行链，让任务可以真实启动 `yt-dlp`、推进到 `downloading/completed/failed`，并通过 `queue_state_changed` 事件实时同步到前端队列。

**Architecture:** 本轮先建立最小但完整的闭环：Rust 侧补齐 `get_queue_snapshot` 命令注册、统一队列状态更新函数、后台线程下载执行器和事件广播；前端只在应用启动层注册一次队列同步，统一由 `queue store` 应用快照。下载参数映射只覆盖本轮必需的 `output_dir`、`format_id`、`audio_only`，其余字段继续留在 `requested_options` 中，不扩展范围。

**Tech Stack:** Tauri 2、Rust、Vue 3、TypeScript、Pinia、Vitest、cargo test、yt-dlp

---

## 文件结构

- 修改：`src-tauri/src/lib.rs`
  责任：注册 `get_queue_snapshot` 命令，并为后续事件广播预留完整 command surface。
- 修改：`src-tauri/src/commands/mod.rs`
  责任：补齐命令层暴露，连接 `create_download_task`、`get_queue_snapshot`、队列快照读取和状态流。
- 修改：`src-tauri/src/services/queue_service.rs`
  责任：集中管理队列状态更新，新增 `mark_downloading`、`mark_completed`、`mark_failed`、`set_progress` 等函数。
- 修改：`src-tauri/src/services/download_service.rs`
  责任：新增最小下载执行器、子进程启动逻辑、状态推进与事件广播。
- 修改：`src-tauri/src/services/tools_service.rs`
  责任：如有需要，抽出 `yt-dlp` 路径查找函数，供下载执行器复用。
- 修改：`src-tauri/src/state/mod.rs`
  责任：如需要新增事件广播依赖或共享状态入口，保持 AppState 结构清晰。
- 修改：`src-tauri/src/tests.rs`
  责任：增加命令层集成测试，验证真实状态流和队列快照命令可用。
- 修改：`src/stores/queue.ts`
  责任：新增 `applySnapshot()` 与 `startQueueSync()`，统一应用初始快照和事件更新。
- 修改：`src/services/tauri/api.ts`
  责任：如需补前端事件辅助函数，集中放在 Tauri API 层。
- 修改：`src/main.ts`
  责任：在应用启动时初始化队列同步，只注册一次监听。
- 修改：`src/stores/queue.test.ts`
  责任：验证 `queue store` 的快照应用和事件同步行为。
- 修改：`src/tests/api.test.ts`
  责任：如前端 API 层新增监听包装，补上映射测试。
- 修改：`docs/alignment-master.md`
  责任：本轮验证通过后，回填第 6、7 项的状态、深度、缺口摘要与日期。

### Task 1: 补齐命令注册与队列快照读取

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/tests.rs`
- Test: `src-tauri/src/tests.rs`

- [ ] **Step 1: 先写失败测试**

```rust
#[test]
fn get_queue_snapshot_returns_current_state_from_command_layer() {
    let state = AppState::default();
    let task = commands::create_download_task_with_state(
        "https://example.com/watch?v=snapshot".to_string(),
        DownloadOptions::default(),
        &state,
    );

    let snapshot = commands::get_queue_snapshot(&state);

    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.tasks[0].task_id, task.task_id);
    assert_eq!(snapshot.active_task_id.as_deref(), Some(task.task_id.as_str()));
}
```

- [ ] **Step 2: 运行测试确认失败**

运行：
```powershell
Set-Location F:\Github\YTSage-Rust\src-tauri
C:\Users\godson321\.cargo\bin\cargo.exe test get_queue_snapshot_returns_current_state_from_command_layer
```

预期：失败，原因是 `get_queue_snapshot` 尚未完整注册或当前命令层未形成可验证入口。

- [ ] **Step 3: 写最小实现**

在 `src-tauri/src/lib.rs` 的 `generate_handler![]` 中加入：

```rust
commands::get_queue_snapshot,
```

在 `src-tauri/src/commands/mod.rs` 中把读取队列快照的函数声明为 Tauri command：

```rust
#[tauri::command]
pub fn get_queue_snapshot(state: State<AppState>) -> QueueState {
    queue_service::snapshot(&state.queue_state)
}
```

保留现有测试辅助入口，避免破坏已有命令层测试：

```rust
pub fn get_queue_snapshot_with_state(state: &AppState) -> QueueState {
    queue_service::snapshot(&state.queue_state)
}
```

- [ ] **Step 4: 重新运行测试确认通过**

运行：
```powershell
Set-Location F:\Github\YTSage-Rust\src-tauri
C:\Users\godson321\.cargo\bin\cargo.exe test get_queue_snapshot_returns_current_state_from_command_layer
```

预期：PASS。

- [ ] **Step 5: 提交**

```powershell
if (Test-Path 'F:\Github\YTSage-Rust\.git') {
  git -C F:\Github\YTSage-Rust add src-tauri/src/lib.rs src-tauri/src/commands/mod.rs src-tauri/src/tests.rs
  git -C F:\Github\YTSage-Rust commit -m "feat: register queue snapshot command"
} else {
  Write-Host "Skip commit: no .git metadata in F:\Github\YTSage-Rust"
}
```

### Task 2: 先用 TDD 收敛队列状态更新接口

**Files:**
- Modify: `src-tauri/src/services/queue_service.rs`
- Test: `src-tauri/src/services/queue_service.rs`

- [ ] **Step 1: 先写失败测试**

```rust
#[test]
fn mark_downloading_completed_and_failed_update_real_task_states() {
    let queue_state = Mutex::new(QueueState::default());
    let task = create_queued_task(
        "https://example.com/watch?v=abc123".to_string(),
        DownloadOptions::default(),
    );
    enqueue(task.clone(), &queue_state);

    let downloading = mark_downloading(&queue_state, &task.task_id).expect("task should exist");
    assert_eq!(downloading.state, "downloading");

    let progressed = set_progress(&queue_state, &task.task_id, 0.5).expect("task should exist");
    assert_eq!(progressed.progress, 0.5);

    let completed = mark_completed(&queue_state, &task.task_id, Some("C:/tmp/file.mp4".to_string()))
        .expect("task should complete");
    assert_eq!(completed.state, "completed");
    assert_eq!(completed.output_path.as_deref(), Some("C:/tmp/file.mp4"));

    let failed = fail_task(&queue_state, &task.task_id, "boom").expect("task should fail");
    assert_eq!(failed.state, "failed");
    assert_eq!(failed.error.as_deref(), Some("boom"));
}
```

- [ ] **Step 2: 运行测试确认失败**

运行：
```powershell
Set-Location F:\Github\YTSage-Rust\src-tauri
C:\Users\godson321\.cargo\bin\cargo.exe test mark_downloading_completed_and_failed_update_real_task_states
```

预期：失败，原因是 `mark_downloading`、`set_progress`、`mark_completed` 尚不存在。

- [ ] **Step 3: 写最小实现**

在 `src-tauri/src/services/queue_service.rs` 中新增：

```rust
pub fn mark_downloading(queue_state: &Mutex<QueueState>, task_id: &str) -> Option<DownloadTask> {
    let mut state = queue_state.lock().expect("queue state lock poisoned");
    let index = state.tasks.iter().position(|task| task.task_id == task_id)?;
    state.tasks[index].state = "downloading".to_string();
    state.active_task_id = Some(task_id.to_string());
    Some(state.tasks[index].clone())
}

pub fn set_progress(queue_state: &Mutex<QueueState>, task_id: &str, progress: f64) -> Option<DownloadTask> {
    let mut state = queue_state.lock().expect("queue state lock poisoned");
    let index = state.tasks.iter().position(|task| task.task_id == task_id)?;
    state.tasks[index].progress = progress.clamp(0.0, 1.0);
    Some(state.tasks[index].clone())
}

pub fn mark_completed(
    queue_state: &Mutex<QueueState>,
    task_id: &str,
    output_path: Option<String>,
) -> Option<DownloadTask> {
    let mut state = queue_state.lock().expect("queue state lock poisoned");
    let index = state.tasks.iter().position(|task| task.task_id == task_id)?;
    state.tasks[index].state = "completed".to_string();
    state.tasks[index].progress = 1.0;
    state.tasks[index].output_path = output_path;
    if state.active_task_id.as_deref() == Some(task_id) {
        state.active_task_id = None;
    }
    Some(state.tasks[index].clone())
}
```

- [ ] **Step 4: 重新运行测试确认通过**

运行：
```powershell
Set-Location F:\Github\YTSage-Rust\src-tauri
C:\Users\godson321\.cargo\bin\cargo.exe test mark_downloading_completed_and_failed_update_real_task_states
```

预期：PASS。

- [ ] **Step 5: 提交**

```powershell
if (Test-Path 'F:\Github\YTSage-Rust\.git') {
  git -C F:\Github\YTSage-Rust add src-tauri/src/services/queue_service.rs
  git -C F:\Github\YTSage-Rust commit -m "feat: add queue state transition helpers"
} else {
  Write-Host "Skip commit: no .git metadata in F:\Github\YTSage-Rust"
}
```

### Task 3: 为下载执行器建立可测试的最小真实状态流

**Files:**
- Modify: `src-tauri/src/services/download_service.rs`
- Modify: `src-tauri/src/services/tools_service.rs`
- Modify: `src-tauri/src/state/mod.rs`
- Modify: `src-tauri/src/tests.rs`
- Test: `src-tauri/src/services/download_service.rs`
- Test: `src-tauri/src/tests.rs`

- [ ] **Step 1: 先写失败测试**

在 `src-tauri/src/services/download_service.rs` 测试模块中新增一组基于注入执行器的测试：

```rust
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

    let result = run_download_with_executor(
        &task.task_id,
        &queue_state,
        |_request| Err("Failed to launch yt-dlp".to_string()),
    );

    assert!(result.is_err());
    let snapshot = queue_service::snapshot(&queue_state);
    assert_eq!(snapshot.tasks[0].state, "failed");
    assert_eq!(snapshot.tasks[0].error.as_deref(), Some("Failed to launch yt-dlp"));
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

    run_download_with_executor(
        &task.task_id,
        &queue_state,
        |_request| Ok(ProcessSnapshot {
            stdout: "done".to_string(),
            stderr: String::new(),
            exit_code: Some(0),
            progress: 1.0,
        }),
    ).expect("download should succeed");

    let snapshot = queue_service::snapshot(&queue_state);
    assert_eq!(snapshot.tasks[0].state, "completed");
    assert_eq!(snapshot.tasks[0].progress, 1.0);
}
```

- [ ] **Step 2: 运行测试确认失败**

运行：
```powershell
Set-Location F:\Github\YTSage-Rust\src-tauri
C:\Users\godson321\.cargo\bin\cargo.exe test start_download_marks_task_failed_when_runner_cannot_launch
C:\Users\godson321\.cargo\bin\cargo.exe test start_download_marks_task_completed_when_runner_succeeds
```

预期：失败，原因是下载执行器和注入 runner 尚不存在。

- [ ] **Step 3: 写最小实现**

在 `src-tauri/src/services/download_service.rs` 中增加：

```rust
type DownloadExecutor = fn(&DownloadTask) -> Result<ProcessSnapshot, String>;

pub fn run_download_with_executor(
    task_id: &str,
    queue_state: &Mutex<QueueState>,
    executor: DownloadExecutor,
) -> Result<DownloadTask, String> {
    let snapshot = queue_service::snapshot(queue_state);
    let task = snapshot
        .tasks
        .into_iter()
        .find(|item| item.task_id == task_id)
        .ok_or_else(|| format!("Unknown task: {task_id}"))?;

    queue_service::mark_downloading(queue_state, task_id)
        .ok_or_else(|| format!("Unknown task: {task_id}"))?;

    match executor(&task) {
        Ok(process) if process.exit_code == Some(0) => {
            queue_service::set_progress(queue_state, task_id, process.progress);
            queue_service::mark_completed(queue_state, task_id, None)
                .ok_or_else(|| format!("Unknown task: {task_id}"))
        }
        Ok(process) => {
            let message = format!("yt-dlp exited with code {}", process.exit_code.unwrap_or(-1));
            queue_service::fail_task(queue_state, task_id, &message)
                .ok_or_else(|| format!("Unknown task: {task_id}"))?;
            Err(message)
        }
        Err(error) => {
            queue_service::fail_task(queue_state, task_id, &error)
                .ok_or_else(|| format!("Unknown task: {task_id}"))?;
            Err(error)
        }
    }
}
```

然后用这个执行器包一层真实 runner：

```rust
fn execute_yt_dlp(task: &DownloadTask) -> Result<ProcessSnapshot, String> {
    let yt_dlp = tools_service::find_required_tool("yt-dlp")
        .ok_or_else(|| "Failed to launch yt-dlp".to_string())?;

    let mut command = std::process::Command::new(yt_dlp);
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
```

如果 `tools_service.rs` 目前没有可复用接口，就补一个最小公开函数：

```rust
pub fn find_required_tool(name: &str) -> Option<PathBuf> {
    find_tool_path(name)
}
```

- [ ] **Step 4: 重新运行测试确认通过**

运行：
```powershell
Set-Location F:\Github\YTSage-Rust\src-tauri
C:\Users\godson321\.cargo\bin\cargo.exe test start_download_marks_task_failed_when_runner_cannot_launch
C:\Users\godson321\.cargo\bin\cargo.exe test start_download_marks_task_completed_when_runner_succeeds
```

预期：PASS。

- [ ] **Step 5: 提交**

```powershell
if (Test-Path 'F:\Github\YTSage-Rust\.git') {
  git -C F:\Github\YTSage-Rust add src-tauri/src/services/download_service.rs src-tauri/src/services/tools_service.rs src-tauri/src/state/mod.rs src-tauri/src/tests.rs
  git -C F:\Github\YTSage-Rust commit -m "feat: add minimal real download execution flow"
} else {
  Write-Host "Skip commit: no .git metadata in F:\Github\YTSage-Rust"
}
```

### Task 4: 把队列状态广播成统一事件

**Files:**
- Modify: `src-tauri/src/services/download_service.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/state/mod.rs`
- Test: `src-tauri/src/tests.rs`

- [ ] **Step 1: 先写失败测试**

在 `src-tauri/src/tests.rs` 中新增对广播入口的最小行为测试，至少验证调用广播辅助函数不会破坏当前队列状态：

```rust
#[test]
fn broadcasting_queue_snapshot_keeps_state_readable() {
    let state = AppState::default();
    let task = commands::create_download_task_with_state(
        "https://example.com/watch?v=emit".to_string(),
        DownloadOptions::default(),
        &state,
    );

    let snapshot = commands::get_queue_snapshot_with_state(&state);

    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.tasks[0].task_id, task.task_id);
}
```

- [ ] **Step 2: 运行测试确认失败或缺失约束**

运行：
```powershell
Set-Location F:\Github\YTSage-Rust\src-tauri
C:\Users\godson321\.cargo\bin\cargo.exe test broadcasting_queue_snapshot_keeps_state_readable
```

预期：如果当前状态可读测试已通过，则继续下一步并把它作为回归保护；如果失败，则先修到通过。

- [ ] **Step 3: 写最小实现**

定义统一事件名常量并收敛广播函数：

```rust
pub const QUEUE_STATE_CHANGED_EVENT: &str = "queue_state_changed";
```

在下载服务中增加一个不持锁的广播辅助函数：

```rust
pub fn emit_queue_state(app_handle: &tauri::AppHandle, queue_state: &Mutex<QueueState>) {
    let snapshot = queue_service::snapshot(queue_state);
    let _ = app_handle.emit(QUEUE_STATE_CHANGED_EVENT, snapshot);
}
```

然后在：

- 创建任务入队后
- 任务变成 `downloading` 后
- 任务变成 `completed/failed` 后

都调用该广播函数。

- [ ] **Step 4: 重新运行测试确认通过**

运行：
```powershell
Set-Location F:\Github\YTSage-Rust\src-tauri
C:\Users\godson321\.cargo\bin\cargo.exe test broadcasting_queue_snapshot_keeps_state_readable
```

预期：PASS。

- [ ] **Step 5: 提交**

```powershell
if (Test-Path 'F:\Github\YTSage-Rust\.git') {
  git -C F:\Github\YTSage-Rust add src-tauri/src/services/download_service.rs src-tauri/src/commands/mod.rs src-tauri/src/state/mod.rs src-tauri/src/tests.rs
  git -C F:\Github\YTSage-Rust commit -m "feat: broadcast queue state changes"
} else {
  Write-Host "Skip commit: no .git metadata in F:\Github\YTSage-Rust"
}
```

### Task 5: 前端统一订阅队列事件并应用快照

**Files:**
- Modify: `src/stores/queue.ts`
- Modify: `src/main.ts`
- Modify: `src/stores/queue.test.ts`
- Modify: `src/tests/api.test.ts`
- Test: `src/stores/queue.test.ts`
- Test: `src/tests/api.test.ts`

- [ ] **Step 1: 先写失败测试**

在 `src/stores/queue.test.ts` 中增加：

```ts
it("applies queue snapshots and updates the active task id", () => {
  const store = useQueueStore();

  store.applySnapshot({
    tasks: [
      {
        taskId: "task-1",
        sourceUrl: "https://example.com/watch?v=abc123",
        title: null,
        state: "downloading",
        progress: 0.5,
        speedText: null,
        etaText: null,
        outputPath: null,
        error: null
      }
    ],
    activeTaskId: "task-1",
    retryCount: 0
  });

  expect(store.tasks).toHaveLength(1);
  expect(store.activeTaskId).toBe("task-1");
  expect(store.tasks[0].state).toBe("downloading");
});
```

再增加：

```ts
it("starts queue sync by loading the initial snapshot once", async () => {
  getQueueSnapshot.mockResolvedValue({
    tasks: [],
    activeTaskId: null,
    retryCount: 0
  });

  const store = useQueueStore();
  await store.startQueueSync();

  expect(getQueueSnapshot).toHaveBeenCalledTimes(1);
});
```

- [ ] **Step 2: 运行测试确认失败**

运行：
```powershell
Set-Location F:\Github\YTSage-Rust
npm.cmd run test -- src/stores/queue.test.ts
```

预期：失败，原因是 `applySnapshot`、`startQueueSync` 尚不存在。

- [ ] **Step 3: 写最小实现**

在 `src/stores/queue.ts` 中新增：

```ts
applySnapshot(state: QueueState) {
  this.tasks = state.tasks;
  this.activeTaskId = state.activeTaskId ?? null;
}
```

以及：

```ts
async startQueueSync() {
  const state = await api.getQueueSnapshot();
  this.applySnapshot(state);
}
```

在 `src/main.ts` 中初始化：

```ts
import { useQueueStore } from "./stores/queue";

const pinia = createPinia();
app.use(pinia);

const queueStore = useQueueStore(pinia);
void queueStore.startQueueSync();
```

如果 API 层需要事件监听包装，则在 `src/services/tauri/api.ts` 中新增：

```ts
import { listen } from "@tauri-apps/api/event";
```

以及：

```ts
listenQueueStateChanged(handler: (state: QueueState) => void) {
  return listen<QueueState>("queue_state_changed", (event) => handler(event.payload));
}
```

同时让 `startQueueSync()` 只注册一次监听：

```ts
if (!this._queueSyncStarted) {
  this._queueSyncStarted = true;
  this._unlisten = await api.listenQueueStateChanged((state) => {
    this.applySnapshot(state);
  });
}
```

如果需要新增状态字段，就在 store state 中加入：

```ts
queueSyncStarted: false as boolean,
```

但不要把监听逻辑散落到页面组件中。

- [ ] **Step 4: 重新运行测试确认通过**

运行：
```powershell
Set-Location F:\Github\YTSage-Rust
npm.cmd run test -- src/stores/queue.test.ts
```

预期：PASS。

- [ ] **Step 5: 提交**

```powershell
if (Test-Path 'F:\Github\YTSage-Rust\.git') {
  git -C F:\Github\YTSage-Rust add src/stores/queue.ts src/main.ts src/stores/queue.test.ts src/tests/api.test.ts src/services/tauri/api.ts
  git -C F:\Github\YTSage-Rust commit -m "feat: sync queue state from tauri events"
} else {
  Write-Host "Skip commit: no .git metadata in F:\Github\YTSage-Rust"
}
```

### Task 6: 执行总验证并回填总账

**Files:**
- Modify: `docs/alignment-master.md`
- Test: `src-tauri/src/tests.rs`
- Test: `src-tauri/src/services/download_service.rs`
- Test: `src/stores/queue.test.ts`
- Test: `src/tests/api.test.ts`
- Test: `docs/alignment-master.md`

- [ ] **Step 1: 运行 Rust 全量相关测试**

运行：
```powershell
Set-Location F:\Github\YTSage-Rust\src-tauri
C:\Users\godson321\.cargo\bin\cargo.exe test
```

预期：全部通过。

- [ ] **Step 2: 运行前端全量相关测试**

运行：
```powershell
Set-Location F:\Github\YTSage-Rust
npm.cmd run test
```

预期：全部通过。

- [ ] **Step 3: 运行前端构建**

运行：
```powershell
Set-Location F:\Github\YTSage-Rust
npm.cmd run build
```

预期：构建通过。

- [ ] **Step 4: 回填总账**

在 `docs/alignment-master.md` 中更新至少两行：

```markdown
| 6 | 真实下载执行链 | ... | `部分可用` | 25% | `P0` | 已能真实启动 `yt-dlp` 并把任务推进到 `downloading/completed/failed`，但尚未接入 stdout 进度解析、FFmpeg 后处理和高级参数映射 | 2026-05-05 |
| 7 | 队列状态机与任务控制 | ... | `部分可用` | 40% | `P0` | 已有真实状态推进与前端事件同步，但暂停、取消、重试仍未接入真实子进程控制 | 2026-05-05 |
```

- [ ] **Step 5: 校验总账文本**

运行：
```powershell
rg -n "真实下载执行链|队列状态机与任务控制|部分可用|queue_state_changed" "F:\Github\YTSage-Rust\docs\alignment-master.md"
```

预期：命中更新后的第 6、7 行摘要。

- [ ] **Step 6: 提交**

```powershell
if (Test-Path 'F:\Github\YTSage-Rust\.git') {
  git -C F:\Github\YTSage-Rust add docs/alignment-master.md
  git -C F:\Github\YTSage-Rust commit -m "docs: update alignment ledger for real download flow"
} else {
  Write-Host "Skip commit: no .git metadata in F:\Github\YTSage-Rust"
}
```

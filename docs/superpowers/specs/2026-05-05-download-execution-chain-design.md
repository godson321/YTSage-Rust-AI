# 真实下载执行链与队列事件同步设计

## 背景

根据 [docs/alignment-master.md](F:/Github/YTSage-Rust/docs/alignment-master.md)，当前最高优先级缺口是：

- 第 6 项：真实下载执行链
- 第 7 项：队列状态机与任务控制

当前 Rust 实现只会创建 `queued task` 并写入队列，但不会真正启动 `yt-dlp`，也不会把任务推进到 `downloading`、`completed`、`failed` 等真实状态。前端虽然已经具备队列页面和 `getQueueSnapshot()` API 封装，但 Rust 侧尚未注册该命令，且没有事件推送，导致前端无法实时获取真实下载状态。

## 目标

本轮只打通“最小真实下载闭环”，目标如下：

1. 创建下载任务后，Rust 后端能够真实启动 `yt-dlp` 子进程。
2. 队列中的任务状态能够从 `queued` 推进到 `downloading`、`completed`、`failed`。
3. 每次队列状态变更后，Rust 后端向前端广播统一的 `queue_state_changed` 事件。
4. 前端 `queue store` 在应用启动后统一订阅该事件，并实时刷新 `tasks` 与 `activeTaskId`。
5. 补齐 `get_queue_snapshot` 的 Tauri command 注册，确保前端已有 API 封装可真正调用。

## 不在本轮范围内

为了保证本轮可以稳定落地，本次明确不处理以下内容：

- 字幕下载、字幕合并、字幕嵌入
- SponsorBlock、区间下载、强制关键帧
- Cookies / 浏览器 Cookies / 代理的完整 UI 工作流
- 播放列表选集 UI
- 批量多任务并发下载
- FFmpeg 后处理和音频转码

这些功能仍依赖真实下载执行链，但不与本轮同时展开。

## 方案对比

### 方案 A：只补后端真实下载链，不做事件推送

优点：

- 实现最小
- 风险最低

缺点：

- 前端仍需要手动刷新或增加轮询
- 无法形成完整的实时状态体验

### 方案 B：真实下载链 + 队列状态事件推送

优点：

- 后端和前端闭环完整
- 与后续下载进度、失败回填、批量任务扩展方向一致
- 可以在不引入复杂事件模型的前提下建立统一状态同步机制

缺点：

- 比方案 A 多一层事件桥接
- 需要同时修改 Rust 和前端

### 方案 C：一步到位补齐全部下载参数与后处理

优点：

- 与原版表面更接近

缺点：

- 范围过大
- 风险高
- 很容易再次出现“只完成一部分”的问题

## 结论

采用方案 B。

原因：

- 它是当前阶段最小但完整的端到端闭环。
- 它能直接推动总账第 6、7 项前进，而不是只增加代码骨架。
- 它不会把本轮工作扩展到字幕、SponsorBlock、设置中心等依赖项。

## 架构设计

### 1. Rust 侧下载执行器

在 `src-tauri/src/services/download_service.rs` 中新增一个内部执行器，负责：

- 根据 `DownloadTask.requested_options` 组装最小 `yt-dlp` 命令
- 创建后台线程启动子进程
- 在启动前把任务状态写成 `downloading`
- 在进程退出后，根据退出码写回 `completed` 或 `failed`

本轮命令只要求支持：

- URL
- 输出目录 `output_dir`
- 基本格式 `format_id`
- 音频优先 `audio_only`

其余参数保留在 `requested_options` 中，但不要求本轮全部映射为命令行参数。

### 2. 队列状态更新服务

在 `src-tauri/src/services/queue_service.rs` 中补充集中状态更新函数，避免由多个模块直接改 `tasks[index]`：

- `mark_downloading`
- `mark_completed`
- `mark_failed`
- `set_progress`

这样可以把状态流统一收敛到一个地方，为后续进度解析、暂停、取消打基础。

### 3. 队列广播事件

在 Rust 侧定义统一事件名：

- `queue_state_changed`

每次队列状态更新后，都发送完整 `QueueState` 给前端，而不是只发局部 patch。这样前端同步逻辑最简单，也更适合当前项目的状态模型。

### 4. 前端订阅入口

前端不在页面组件内订阅事件，而是在应用启动层建立一次性订阅，避免：

- 页面切换重复订阅
- 组件卸载导致队列状态丢失
- 事件监听散落在多个页面

具体实现位置优先选：

- `src/main.ts` 负责初始化
- `src/stores/queue.ts` 负责接收并应用状态

### 5. 前端状态更新策略

`queue store` 新增两个能力：

- `applySnapshot(state: QueueState)`：统一替换 `tasks` 和 `activeTaskId`
- `startQueueSync()`：注册 `queue_state_changed` 监听，并在初始化时主动拉一次 `getQueueSnapshot()`

这样前端对“初始加载”和“事件更新”都走同一套状态入口。

## 数据流

1. 前端调用 `create_download_task`
2. Rust 创建 `DownloadTask` 并写入队列
3. Rust 立即广播一次 `queue_state_changed`
4. Rust 后台线程启动 `yt-dlp`
5. 任务状态更新为 `downloading`，再次广播
6. 进程结束后写回 `completed` 或 `failed`
7. Rust 再次广播最新 `QueueState`
8. 前端 `queue store` 收到事件后覆盖本地队列状态

## 错误处理

### `yt-dlp` 不存在

- 任务状态写为 `failed`
- `error` 写入明确提示，例如 `Failed to launch yt-dlp`
- 发送 `queue_state_changed`

### 命令启动失败

- 视为 `failed`
- 保留错误文本到 `task.error`
- 清理 `activeTaskId`

### 进程非零退出

- 任务状态写为 `failed`
- 写入简化后的退出信息

### 事件发送失败

- 不阻塞任务状态更新
- 记录为内部可忽略错误

## 测试设计

### Rust 测试

先写失败测试，至少覆盖：

1. `get_queue_snapshot` 已注册并可通过命令层读取状态
2. 创建任务后，队列能进入真实状态流，而不只是停留在 `queued`
3. 下载命令启动失败时，任务进入 `failed`
4. 成功路径下，任务最终进入 `completed`

测试优先使用可注入 runner 或可替换命令执行器，避免依赖真实网络下载。

### 前端测试

先写失败测试，至少覆盖：

1. `queue store` 可以应用 `QueueState`
2. `startQueueSync()` 会先调用 `getQueueSnapshot()`
3. 收到 `queue_state_changed` 事件后会刷新 `tasks` 和 `activeTaskId`

## 验收标准

满足以下条件才算本轮完成：

1. 创建任务后，不再永久停留在 `queued`
2. 队列状态变化可以被前端自动同步
3. `get_queue_snapshot` 可正常调用
4. Rust 测试和前端测试都有新增覆盖
5. 总账第 6、7 项可以根据真实结果回填，而不是继续停留在纯骨架描述

## 风险与控制

### 风险 1：真实进程调用导致测试不稳定

控制：

- 使用可注入执行器做单元测试
- 不在测试里依赖真实 `yt-dlp` 网络行为

### 风险 2：事件订阅散落导致重复监听

控制：

- 只在应用启动层注册一次
- 状态写入统一收敛到 `queue store`

### 风险 3：本轮再次扩散到下载参数完整度

控制：

- 只映射最小必需参数
- 其他字段继续保留在 `requested_options`，不在本轮展开

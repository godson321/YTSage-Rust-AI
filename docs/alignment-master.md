# YTSage 对齐总账

> 本文件是 `F:\Github\YTSage-Rust` 中 YTSage 功能对齐状态的唯一来源。

## 目的

- 以“用户可感知功能”为单位跟踪 YTSage 与 YTSage-Rust 的对齐状态，而不是只看 DTO、命令名或页面骨架。
- 每次完成对齐工作后，只更新本文件，不再在每个会话里重复做整库盘点。
- 后续实现具体功能时，只回看对应功能行列出的原版代码入口，不再无目标全量重扫旧框架。
- 本文件及后续新增的对齐文档统一使用中文。

## 状态说明

| 状态 | 含义 | 升级条件 |
|---|---|---|
| `未开始` | Rust 侧没有可用实现路径 | 没有可达的前端入口或后端执行链 |
| `骨架已铺` | 模型、命令或页面骨架已存在，但功能不能端到端使用 | 至少存在一个 Rust 代码入口，但用户流程未闭环 |
| `部分可用` | 已有一个或多个真实流程可用，但与 YTSage 仍有明显缺口 | 用户可以走通部分流程，但差异仍较大 |
| `端到端通过` | Rust 侧完整流程可跑通，并且有实际验证证据 | 已执行并记录验证路径 |
| `与原版对齐` | Rust 行为与 YTSage 在该功能范围内一致 | 已完成代码对代码核对与端到端验证 |

## 证据规则

- 不能因为字段、DTO 或页面壳子存在，就把状态提升到 `骨架已铺` 以上。
- 不能在没有验证记录的情况下，把状态提升到 `端到端通过`。
- 不能在没有重新核对该行所列原版代码入口的情况下，把状态提升到 `与原版对齐`。

## 功能总表

| ID | 功能项 | 原版代码入口 | Rust 前端入口 | Rust 后端入口 | 状态 | 深度 | 优先级 | 缺口摘要 | 最后核对 |
|---|---|---|---|---|---|---:|---|---|---|
| 1 | URL 解析与基础分析 | `ytsage/gui/ytsage_gui_analysis.py` | `src/pages/DownloadPage.vue`, `src/stores/analysis.ts` | `src-tauri/src/services/analysis_service.rs` | `部分可用` | 55% | `P1` | 已接 `yt-dlp --dump-single-json`，但分析结果还没完整驱动详情、格式选择和下载准备界面 | 2026-05-05 |
| 2 | 播放列表识别与 URL 归一化 | `ytsage/gui/ytsage_gui_analysis.py` | `src/pages/DownloadPage.vue` | `src-tauri/src/services/analysis_service.rs` | `骨架已铺` | 45% | `P1` | 已返回 `playlistInfo` 与 `playlistEntries`，但前端没有播放列表专用 UI | 2026-05-05 |
| 3 | 播放列表首个可用视频回退探测 | `ytsage/gui/ytsage_gui_analysis.py` | 无 | `src-tauri/src/services/analysis_service.rs` | `未开始` | 10% | `P1` | Rust 版没有原版的多候选条目探测与回退逻辑 | 2026-05-05 |
| 4 | 视频信息 / 缩略图 / 时长 / 频道 / 格式表展示 | `ytsage/gui/ytsage_gui_main.py`, `ytsage/gui/ytsage_gui_video_info.py`, `ytsage/gui/ytsage_gui_format_table.py` | `src/pages/DownloadPage.vue` | `src-tauri/src/models/mod.rs`, `src-tauri/src/services/analysis_service.rs` | `骨架已铺` | 15% | `P0` | 后端 DTO 已有，但下载页仍只显示地址、状态、标题三列 | 2026-05-05 |
| 5 | 下载参数模型完整度 | `ytsage/core/ytsage_downloader.py`, `ytsage/gui/ytsage_gui_dialogs/ytsage_dialogs_settings.py`, `ytsage/gui/ytsage_gui_dialogs/ytsage_dialogs_custom.py` | `src/stores/queue.ts` | `src-tauri/src/models/mod.rs` | `骨架已铺` | 40% | `P1` | 大部分参数字段已建模，但还没形成 UI 与执行链闭环 | 2026-05-05 |
| 6 | 真实下载执行链 | `ytsage/core/ytsage_downloader.py` | `src/pages/BatchPage.vue`, `src/stores/queue.ts`, `src/stores/download.ts` | `src-tauri/src/services/download_service.rs`, `src-tauri/src/services/queue_service.rs` | `部分可用` | 25% | `P0` | 已能真实启动 `yt-dlp` 并把任务推进到 `downloading/completed/failed`，但尚未接入 stdout 进度解析、FFmpeg 后处理和高级参数映射 | 2026-05-05 |
| 7 | 队列状态机与任务控制 | `ytsage/gui/ytsage_gui_main.py`, `ytsage/core/ytsage_downloader.py` | `src/pages/BatchPage.vue`, `src/stores/queue.ts` | `src-tauri/src/commands/mod.rs`, `src-tauri/src/services/queue_service.rs` | `部分可用` | 40% | `P0` | 已有真实状态推进与前端 `queue_state_changed` 事件同步，但暂停、取消、重试仍未接入真实子进程控制与更细粒度进度更新 | 2026-05-05 |
| 8 | 批量任务工作流 | `ytsage/gui/ytsage_gui_main.py` | `src/pages/BatchPage.vue` | `src-tauri/src/services/queue_service.rs` | `骨架已铺` | 20% | `P1` | 多 URL 仅逐个入队，没有批次推进、失败处理和完成收口 | 2026-05-05 |
| 9 | 字幕读取、选择、下载、合并 | `ytsage/gui/ytsage_gui_dialogs/ytsage_dialogs_selection.py`, `ytsage/core/ytsage_downloader.py` | 无 | `src-tauri/src/services/analysis_service.rs`, `src-tauri/src/models/mod.rs` | `未开始` | 10% | `P0` | 只有分析返回的字幕数据，没有选择 UI，也没有下载、合并、嵌入执行 | 2026-05-05 |
| 10 | 播放列表条目选择 | `ytsage/gui/ytsage_gui_dialogs/ytsage_dialogs_selection.py` | 无 | `src-tauri/src/models/mod.rs` | `未开始` | 10% | `P0` | `playlistItems` 已建模，但没有选集界面与实际下载接线 | 2026-05-05 |
| 11 | SponsorBlock / 区间下载 / 强制关键帧 | `ytsage/gui/ytsage_gui_dialogs/ytsage_dialogs_selection.py`, `ytsage/gui/ytsage_gui_dialogs/ytsage_dialogs_custom.py`, `ytsage/core/ytsage_downloader.py` | 无 | `src-tauri/src/models/mod.rs` | `未开始` | 5% | `P0` | 参数字段存在，但没有 UI、命令拼装或执行逻辑 | 2026-05-05 |
| 12 | Cookies / 浏览器 Cookies / 内嵌登录 / 代理 | `ytsage/gui/ytsage_gui_dialogs/ytsage_dialogs_custom.py`, `ytsage/gui/ytsage_gui_analysis.py`, `ytsage/core/ytsage_downloader.py` | `src/pages/SettingsPage.vue` | `src-tauri/src/services/analysis_service.rs`, `src-tauri/src/models/mod.rs` | `骨架已铺` | 15% | `P0` | 分析后端支持参数，但前端没有 Cookies、代理、内嵌登录完整工作流 | 2026-05-05 |
| 13 | 下载设置中心 | `ytsage/gui/ytsage_gui_dialogs/ytsage_dialogs_settings.py` | `src/pages/SettingsPage.vue`, `src/stores/settings.ts` | `src-tauri/src/services/settings_service.rs` | `骨架已铺` | 20% | `P1` | 现在只有下载目录、文件名模板、语言、generic mode，原版的大量设置缺失 | 2026-05-05 |
| 14 | 历史记录 | `ytsage/utils/ytsage_history_manager.py`, `ytsage/gui/ytsage_gui_dialogs/ytsage_dialogs_history.py` | `src/pages/HistoryPage.vue`, `src/stores/history.ts` | `src-tauri/src/services/history_service.rs` | `部分可用` | 40% | `P1` | 已有查删清，但没有缩略图、打开位置、重下、卡片视图和更多元数据 | 2026-05-05 |
| 15 | 日志 / 诊断 | `ytsage/utils/ytsage_logger.py`, `ytsage/gui/ytsage_gui_dialogs/ytsage_dialogs_base.py` | `src/pages/LogsPage.vue`, `src/stores/logs.ts` | `src-tauri/src/services/logs_service.rs` | `骨架已铺` | 20% | `P2` | 只有日志 tail 文本显示，没有诊断包、日志目录入口或分级筛选 | 2026-05-05 |
| 16 | 工具检测与状态展示 | `ytsage/gui/ytsage_gui_dialogs/ytsage_dialogs_base.py` | `src/pages/ToolsPage.vue`, `src/stores/tools.ts` | `src-tauri/src/services/tools_service.rs` | `骨架已铺` | 30% | `P2` | 已能检测 `yt-dlp/ffmpeg/deno`，但没有安装、升级、集成状态和缓存状态 | 2026-05-05 |
| 17 | 更新体系 | `ytsage/gui/ytsage_gui_main.py`, `ytsage/gui/ytsage_gui_dialogs/ytsage_dialogs_update.py`, `ytsage/gui/ytsage_gui_dialogs/ytsage_dialogs_updater.py` | 无 | `src-tauri/src/services/update_service.rs` | `未开始` | 5% | `P2` | 只有本地版本比较器，没有远端检查、UI、YTSage/yt-dlp/ffmpeg/deno 更新流程 | 2026-05-05 |
| 18 | FFmpeg / Deno 安装与集成 | `ytsage/gui/ytsage_gui_dialogs/ytsage_dialogs_ffmpeg.py`, `ytsage/gui/ytsage_gui_dialogs/ytsage_dialogs_updater.py` | `src/pages/ToolsPage.vue` | `src-tauri/src/services/tools_service.rs` | `未开始` | 5% | `P2` | 只有存在性检测，没有自动安装、手动引导、版本检查与升级 | 2026-05-05 |
| 19 | 本地化 / 多语言 / 文案系统 | `ytsage/languages`, `ytsage/utils/ytsage_localization.py` | `src/pages/SettingsPage.vue`, `src/components/layout/AppShell.vue` | `src-tauri/src/services/settings_service.rs` | `未开始` | 10% | `P2` | 只有语言字段，没有真正的 i18n 资源、切换机制和页面文案接线 | 2026-05-05 |
| 20 | About / 系统状态总览 | `ytsage/gui/ytsage_gui_dialogs/ytsage_dialogs_base.py` | `src/pages/AboutPage.vue` | `src-tauri/src/services/tools_service.rs` | `骨架已铺` | 15% | `P2` | About 页仍是静态说明，没有原版的系统状态总览、日志入口、工具细节 | 2026-05-05 |

## 当前优先级顺序

1. 真实下载执行链。
2. 下载页详情、格式表、字幕与播放列表选择。
3. Cookies / 代理 / 设置中心的执行链打通。
4. 批量队列与事件驱动状态回填。
5. 工具安装、更新与诊断能力。

## 更新流程

1. 每次做功能对齐前，先读本文件，再决定是否需要打开原版 YTSage。
2. 先选定要推进的功能行，再去实现，不允许无目标地重新扫全库。
3. 只回看该功能行中列出的原版代码入口，除非它依赖相邻功能。
4. 实现或验证完成后，立刻更新该行的 `状态`、`深度`、`缺口摘要`、`最后核对`。
5. 如果当前实现暴露出一个总表中尚未列出的新功能，先补行，再继续开发。

### 行级更新规则

- `未开始 -> 骨架已铺`：只有在 Rust 侧出现可达代码路径时才能升级。
- `骨架已铺 -> 部分可用`：只有在至少一个真实用户流程可用时才能升级。
- `部分可用 -> 端到端通过`：只有在执行过明确验证路径并留下记录时才能升级。
- `端到端通过 -> 与原版对齐`：只有在重新核对该行列出的原版代码入口后才能升级。
- 除非依赖关系或业务影响发生变化，否则实现过程中不要随意改动 `优先级`。

### 什么时候需要回看原版 YTSage

- 当某一行存在行为歧义、隐藏交互或命令边界条件时。
- 当 Rust 版测试结果与预期不一致，需要回溯确认原版真实行为时。
- 当准备把状态提升到 `与原版对齐` 时，必须重新核对该行原版代码入口。
- 除非确认总账本身已经过期，否则不要重新做整库审计。

### 每次会话清单

- 先读本文件。
- 选择本次要推进的功能行。
- 只打开该行列出的原版代码入口。
- 实现并验证 Rust 侧变更。
- 结束会话前更新本文件。

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AnalysisResult,
  AnalyzeRequest,
  AppSettings,
  DownloadOptions,
  DownloadTask,
  HistoryEntry,
  QueueState,
  ToolStatus,
  UpdateStatus
} from "../../types/models";

export const api = {
  analyzeUrls(payload: AnalyzeRequest) {
    return invoke<AnalysisResult>("analyze_urls", { payload });
  },
  createDownloadTask(sourceUrl: string, options: DownloadOptions) {
    return invoke<DownloadTask>("create_download_task", { sourceUrl, options });
  },
  getSettings() {
    return invoke<AppSettings>("get_settings");
  },
  saveSettings(settings: AppSettings) {
    return invoke<AppSettings>("save_settings", { settings });
  },
  listHistory() {
    return invoke<HistoryEntry[]>("list_history");
  },
  searchHistory(query: string) {
    return invoke<HistoryEntry[]>("search_history", { query });
  },
  removeHistoryEntry(entryId: string) {
    return invoke<boolean>("remove_history_entry", { entryId });
  },
  clearHistory() {
    return invoke<number>("clear_history");
  },
  tailLogs(lines: number) {
    return invoke<string[]>("tail_logs", { lines });
  },
  getToolStatus() {
    return invoke<ToolStatus[]>("get_tool_status");
  },
  getQueueSnapshot() {
    return invoke<QueueState>("get_queue_snapshot");
  },
  clearQueue() {
    return invoke<QueueState>("clear_queue");
  },
  listenQueueStateChanged(handler: (state: QueueState) => void) {
    return listen<QueueState>("queue_state_changed", (event) => handler(event.payload));
  },
  pauseTask(taskId: string) {
    return invoke<DownloadTask | null>("pause_task", { taskId });
  },
  retryTask(taskId: string) {
    return invoke<DownloadTask | null>("retry_task", { taskId });
  },
  getUpdateStatus(currentVersion: string) {
    return invoke<UpdateStatus>("get_update_status", { currentVersion });
  }
};

import { defineStore } from "pinia";
import { api } from "../services/tauri/api";
import type { DownloadOptions, DownloadTask, QueueState } from "../types/models";

const defaultDownloadOptions: DownloadOptions = {
  outputDir: "",
  formatId: "best",
  audioOnly: false,
  mergeSubs: false,
  saveDescription: false,
  saveThumbnail: false,
  embedChapters: false,
  rateLimit: undefined,
  playlistItems: null,
  subtitleLangs: [],
  enableSponsorblock: false,
  sponsorblockCategories: [],
  resolution: null,
  downloadSection: null,
  forceKeyframes: false,
  proxyUrl: null,
  geoProxyUrl: null,
  forceOutputFormat: false,
  preferredOutputFormat: null,
  forceAudioFormat: false,
  preferredAudioFormat: null,
  audioNormalization: false,
  filenameFormat: null,
  cookieFilePath: null,
  browserCookiesOption: null
};

export const useQueueStore = defineStore("queue", {
  state: () => ({
    tasks: [] as DownloadTask[],
    activeTaskId: null as string | null,
    loading: false,
    error: null as string | null,
    queueSyncStarted: false,
    queueSyncUnlisten: null as null | (() => void)
  }),
  getters: {
    activeTask: (state) => state.tasks.find((task) => task.taskId === state.activeTaskId) ?? null
  },
  actions: {
    applySnapshot(state: QueueState) {
      this.tasks = state.tasks;
      this.activeTaskId = state.activeTaskId ?? null;
    },
    async startQueueSync() {
      if (!this.queueSyncStarted) {
        this.queueSyncStarted = true;
        this.queueSyncUnlisten = await api.listenQueueStateChanged((state) => {
          this.applySnapshot(state);
        });
      }

      const snapshot = await api.getQueueSnapshot();
      this.applySnapshot(snapshot);
    },
    async enqueue(sourceUrl: string, options: DownloadOptions = defaultDownloadOptions) {
      this.loading = true;
      this.error = null;
      try {
        const task = await api.createDownloadTask(sourceUrl, options);
        const hasTask = this.tasks.some((item) => item.taskId === task.taskId);
        if (!hasTask) {
          this.tasks = [...this.tasks, task];
        }
        if (!this.activeTaskId) {
          this.activeTaskId = task.taskId;
        }
        return task;
      } catch (error) {
        this.error = error instanceof Error ? error.message : "Failed to enqueue task.";
        throw error;
      } finally {
        this.loading = false;
      }
    },
    async refresh() {
      const snapshot = await api.getQueueSnapshot();
      this.applySnapshot(snapshot);
    },
    async clearRemote() {
      const snapshot = await api.clearQueue();
      this.applySnapshot(snapshot);
    },
    async pause(taskId: string) {
      const task = await api.pauseTask(taskId);
      if (!task) {
        return null;
      }
      this.tasks = this.tasks.map((item) => (item.taskId === taskId ? task : item));
      if (this.activeTaskId === taskId) {
        this.activeTaskId = null;
      }
      return task;
    },
    async resume(taskId: string) {
      const task = await api.resumeTask(taskId);
      if (!task) {
        return null;
      }
      this.tasks = this.tasks.map((item) => (item.taskId === taskId ? task : item));
      this.activeTaskId = task.taskId;
      return task;
    },
    async cancel(taskId: string) {
      const task = await api.cancelTask(taskId);
      if (!task) {
        return null;
      }
      this.tasks = this.tasks.map((item) => (item.taskId === taskId ? task : item));
      if (this.activeTaskId === taskId) {
        this.activeTaskId = null;
      }
      return task;
    },
    async retry(taskId: string) {
      const task = await api.retryTask(taskId);
      if (!task) {
        return null;
      }
      this.tasks = this.tasks.map((item) => (item.taskId === taskId ? task : item));
      this.activeTaskId = task.taskId;
      return task;
    },
    remove(taskId: string) {
      this.tasks = this.tasks.filter((task) => task.taskId !== taskId);
      if (this.activeTaskId === taskId) {
        this.activeTaskId = this.tasks[0]?.taskId ?? null;
      }
    },
    setActiveTask(taskId: string | null) {
      this.activeTaskId = taskId;
    },
    clear() {
      this.tasks = [];
      this.activeTaskId = null;
      this.loading = false;
      this.error = null;
      this.queueSyncStarted = false;
      this.queueSyncUnlisten?.();
      this.queueSyncUnlisten = null;
    }
  }
});

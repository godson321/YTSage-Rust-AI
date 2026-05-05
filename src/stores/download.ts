import { defineStore } from "pinia";
import type { DownloadTask } from "../types/models";

export const useDownloadStore = defineStore("download", {
  state: () => ({
    activeTask: null as DownloadTask | null,
    completedTasks: [] as DownloadTask[],
    failedTasks: [] as DownloadTask[]
  }),
  getters: {
    activeProgress: (state) => state.activeTask?.progress ?? 0,
    hasActiveTask: (state) => state.activeTask !== null
  },
  actions: {
    setActiveTask(task: DownloadTask | null) {
      this.activeTask = task ? { ...task } : null;
    },
    updateActiveTask(patch: Partial<DownloadTask>) {
      if (!this.activeTask) {
        return;
      }

      this.activeTask = { ...this.activeTask, ...patch };
    },
    markCompleted(task: DownloadTask) {
      this.completedTasks = [task, ...this.completedTasks];
      this.activeTask = null;
    },
    markFailed(task: DownloadTask, error: string) {
      this.failedTasks = [{ ...task, state: "failed", error }, ...this.failedTasks];
      this.activeTask = null;
    },
    clear() {
      this.activeTask = null;
      this.completedTasks = [];
      this.failedTasks = [];
    }
  }
});

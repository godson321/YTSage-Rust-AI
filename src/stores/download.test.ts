import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";
import type { DownloadTask } from "../types/models";
import { useDownloadStore } from "./download";

beforeEach(() => {
  setActivePinia(createPinia());
});

describe("download store", () => {
  it("tracks the active task and completion history", () => {
    const store = useDownloadStore();
    const task: DownloadTask = {
      taskId: "task-1",
      sourceUrl: "https://example.com/watch?v=abc123",
      title: null,
      state: "downloading",
      progress: 0.25,
      speedText: "1.2 MB/s",
      etaText: "2m",
      outputPath: null,
      error: null
    };

    store.setActiveTask(task);
    store.updateActiveTask({ progress: 0.5, etaText: "1m" });
    store.markCompleted({ ...task, state: "completed", progress: 1 });

    expect(store.activeTask).toBeNull();
    expect(store.completedTasks).toHaveLength(1);
    expect(store.completedTasks[0].state).toBe("completed");
    expect(store.activeProgress).toBe(0);
  });
});

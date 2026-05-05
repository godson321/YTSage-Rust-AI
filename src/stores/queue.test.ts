import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { DownloadTask, QueueState } from "../types/models";

const createDownloadTask = vi.fn();
const getQueueSnapshot = vi.fn();
const listenQueueStateChanged = vi.fn();
const pauseTask = vi.fn();
const resumeTask = vi.fn();
const cancelTask = vi.fn();
const retryTask = vi.fn();

vi.mock("../services/tauri/api", () => ({
  api: {
    createDownloadTask: (...args: unknown[]) => createDownloadTask(...args),
    getQueueSnapshot: (...args: unknown[]) => getQueueSnapshot(...args),
    listenQueueStateChanged: (...args: unknown[]) => listenQueueStateChanged(...args),
    pauseTask: (...args: unknown[]) => pauseTask(...args),
    resumeTask: (...args: unknown[]) => resumeTask(...args),
    cancelTask: (...args: unknown[]) => cancelTask(...args),
    retryTask: (...args: unknown[]) => retryTask(...args)
  }
}));

import { useQueueStore } from "./queue";

beforeEach(() => {
  setActivePinia(createPinia());
  createDownloadTask.mockReset();
  getQueueSnapshot.mockReset();
  listenQueueStateChanged.mockReset();
  pauseTask.mockReset();
  resumeTask.mockReset();
  cancelTask.mockReset();
  retryTask.mockReset();
});

describe("queue store", () => {
  it("enqueues tasks returned by the backend", async () => {
    const task: DownloadTask = {
      taskId: "task-1",
      sourceUrl: "https://example.com/watch?v=abc123",
      title: null,
      state: "queued",
      progress: 0,
      speedText: null,
      etaText: null,
      outputPath: null,
      error: null
    };
    createDownloadTask.mockResolvedValue(task);

    const store = useQueueStore();
    const result = await store.enqueue(task.sourceUrl);

    expect(result).toEqual(task);
    expect(store.tasks).toHaveLength(1);
    expect(store.activeTaskId).toBe("task-1");
    expect(store.activeTask?.taskId).toBe("task-1");
  });

  it("removes a task and falls back to the next task", () => {
    const store = useQueueStore();
    store.tasks = [
      {
        taskId: "task-1",
        sourceUrl: "https://example.com/a",
        title: null,
        state: "queued",
        progress: 0,
        speedText: null,
        etaText: null,
        outputPath: null,
        error: null
      },
      {
        taskId: "task-2",
        sourceUrl: "https://example.com/b",
        title: null,
        state: "queued",
        progress: 0,
        speedText: null,
        etaText: null,
        outputPath: null,
        error: null
      }
    ];
    store.activeTaskId = "task-1";

    store.remove("task-1");

    expect(store.tasks).toHaveLength(1);
    expect(store.activeTaskId).toBe("task-2");
  });

  it("applies queue snapshots and updates the active task id", () => {
    const store = useQueueStore();
    const snapshot: QueueState = {
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
    };

    store.applySnapshot(snapshot);

    expect(store.tasks).toHaveLength(1);
    expect(store.activeTaskId).toBe("task-1");
    expect(store.tasks[0].state).toBe("downloading");
  });

  it("starts queue sync by loading the initial snapshot once", async () => {
    getQueueSnapshot.mockResolvedValue({
      tasks: [],
      activeTaskId: null,
      retryCount: 0
    });
    listenQueueStateChanged.mockResolvedValue(() => {});

    const store = useQueueStore();
    await store.startQueueSync();

    expect(getQueueSnapshot).toHaveBeenCalledTimes(1);
    expect(listenQueueStateChanged).toHaveBeenCalledTimes(1);
  });

  it("applies pushed queue snapshots after queue sync starts", async () => {
    let handler: ((state: QueueState) => void) | undefined;
    getQueueSnapshot.mockResolvedValue({
      tasks: [],
      activeTaskId: null,
      retryCount: 0
    });
    listenQueueStateChanged.mockImplementation((callback: (state: QueueState) => void) => {
      handler = callback;
      return Promise.resolve(() => {});
    });

    const store = useQueueStore();
    await store.startQueueSync();
    handler?.({
      tasks: [
        {
          taskId: "task-2",
          sourceUrl: "https://example.com/2",
          title: null,
          state: "completed",
          progress: 1,
          speedText: null,
          etaText: null,
          outputPath: "C:/Downloads/file.mp4",
          error: null
        }
      ],
      activeTaskId: null,
      retryCount: 0
    });

    expect(store.tasks).toHaveLength(1);
    expect(store.tasks[0].taskId).toBe("task-2");
    expect(store.tasks[0].state).toBe("completed");
    expect(store.activeTaskId).toBeNull();
  });

  it("pauses resumes and cancels tasks through the backend api", async () => {
    const store = useQueueStore();
    store.tasks = [
      {
        taskId: "task-1",
        sourceUrl: "https://example.com/a",
        title: null,
        state: "downloading",
        progress: 0.4,
        speedText: "1.2MiB/s",
        etaText: "00:10",
        outputPath: null,
        error: null
      }
    ];
    store.activeTaskId = "task-1";

    pauseTask.mockResolvedValue({ ...store.tasks[0], state: "paused" });
    resumeTask.mockResolvedValue({ ...store.tasks[0], state: "downloading" });
    cancelTask.mockResolvedValue({ ...store.tasks[0], state: "cancelled" });

    const paused = await store.pause("task-1");
    expect(paused?.state).toBe("paused");
    expect(store.activeTaskId).toBeNull();

    const resumed = await store.resume("task-1");
    expect(resumed?.state).toBe("downloading");
    expect(store.activeTaskId).toBe("task-1");

    const cancelled = await store.cancel("task-1");
    expect(cancelled?.state).toBe("cancelled");
    expect(store.activeTaskId).toBeNull();
  });
});

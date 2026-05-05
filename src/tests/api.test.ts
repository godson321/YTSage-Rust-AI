import { beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.fn();
const listen = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invoke(...args)
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => listen(...args)
}));

import { api } from "../services/tauri/api";

beforeEach(() => {
  invoke.mockReset();
  listen.mockReset();
});

describe("tauri api wrapper", () => {
  it("maps history and log commands", async () => {
    invoke.mockResolvedValue(undefined);

    await api.listHistory();
    await api.searchHistory("video");
    await api.removeHistoryEntry("history-1");
    await api.clearHistory();
    await api.tailLogs(20);

    expect(invoke).toHaveBeenNthCalledWith(1, "list_history");
    expect(invoke).toHaveBeenNthCalledWith(2, "search_history", { query: "video" });
    expect(invoke).toHaveBeenNthCalledWith(3, "remove_history_entry", { entryId: "history-1" });
    expect(invoke).toHaveBeenNthCalledWith(4, "clear_history");
    expect(invoke).toHaveBeenNthCalledWith(5, "tail_logs", { lines: 20 });
  });

  it("maps queue commands", async () => {
    invoke.mockResolvedValue(undefined);

    await api.getQueueSnapshot();
    await api.clearQueue();
    await api.pauseTask("task-1");
    await api.resumeTask("task-1");
    await api.cancelTask("task-1");
    await api.retryTask("task-1");

    expect(invoke).toHaveBeenNthCalledWith(1, "get_queue_snapshot");
    expect(invoke).toHaveBeenNthCalledWith(2, "clear_queue");
    expect(invoke).toHaveBeenNthCalledWith(3, "pause_task", { taskId: "task-1" });
    expect(invoke).toHaveBeenNthCalledWith(4, "resume_task", { taskId: "task-1" });
    expect(invoke).toHaveBeenNthCalledWith(5, "cancel_task", { taskId: "task-1" });
    expect(invoke).toHaveBeenNthCalledWith(6, "retry_task", { taskId: "task-1" });
  });

  it("maps queue state event subscription", async () => {
    const handler = vi.fn();
    const unlisten = vi.fn();
    listen.mockResolvedValue(unlisten);

    const result = await api.listenQueueStateChanged(handler);
    const payload = { tasks: [], activeTaskId: null, retryCount: 0 };
    const eventHandler = listen.mock.calls[0]?.[1];
    eventHandler?.({ payload });

    expect(listen).toHaveBeenCalledTimes(1);
    expect(listen).toHaveBeenCalledWith("queue_state_changed", expect.any(Function));
    expect(handler).toHaveBeenCalledWith(payload);
    expect(result).toBe(unlisten);
  });
});

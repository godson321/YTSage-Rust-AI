import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { HistoryEntry } from "../types/models";

const listHistory = vi.fn();
const searchHistory = vi.fn();
const removeHistoryEntry = vi.fn();
const clearHistory = vi.fn();

vi.mock("../services/tauri/api", () => ({
  api: {
    listHistory: (...args: unknown[]) => listHistory(...args),
    searchHistory: (...args: unknown[]) => searchHistory(...args),
    removeHistoryEntry: (...args: unknown[]) => removeHistoryEntry(...args),
    clearHistory: (...args: unknown[]) => clearHistory(...args)
  }
}));

import { useHistoryStore } from "./history";

beforeEach(() => {
  setActivePinia(createPinia());
  listHistory.mockReset();
  searchHistory.mockReset();
  removeHistoryEntry.mockReset();
  clearHistory.mockReset();
});

describe("history store", () => {
  it("loads history entries from the backend", async () => {
    const entry: HistoryEntry = {
      id: "history-1",
      title: "Video",
      url: "https://example.com/watch?v=abc123",
      channel: "Channel",
      filePath: "C:/Downloads/video.mp4",
      downloadDate: null,
      fileSize: null,
      thumbnailUrl: null,
      formatId: null,
      resolution: null,
      isAudioOnly: false,
      duration: null,
      downloadOptions: null
    };
    listHistory.mockResolvedValue([entry]);

    const store = useHistoryStore();
    await store.load();

    expect(store.entries).toEqual([entry]);
  });

  it("searches with the backend when the query is not empty", async () => {
    const entry: HistoryEntry = {
      id: "history-1",
      title: "Video",
      url: "https://example.com/watch?v=abc123",
      channel: "Channel",
      filePath: "C:/Downloads/video.mp4",
      downloadDate: null,
      fileSize: null,
      thumbnailUrl: null,
      formatId: null,
      resolution: null,
      isAudioOnly: false,
      duration: null,
      downloadOptions: null
    };
    searchHistory.mockResolvedValue([entry]);

    const store = useHistoryStore();
    await store.load("video");

    expect(searchHistory).toHaveBeenCalledWith("video");
    expect(store.entries).toEqual([entry]);
  });

  it("deletes a history entry and reloads the list", async () => {
    const entry: HistoryEntry = {
      id: "history-1",
      title: "Video",
      url: "https://example.com/watch?v=abc123",
      channel: "Channel",
      filePath: "C:/Downloads/video.mp4",
      downloadDate: null,
      fileSize: null,
      thumbnailUrl: null,
      formatId: null,
      resolution: null,
      isAudioOnly: false,
      duration: null,
      downloadOptions: null
    };
    listHistory.mockResolvedValueOnce([entry]).mockResolvedValueOnce([]);

    const store = useHistoryStore();
    await store.load();
    await store.deleteEntry("history-1");

    expect(removeHistoryEntry).toHaveBeenCalledWith("history-1");
    expect(store.entries).toEqual([]);
  });

  it("clears history and reloads with an empty query", async () => {
    listHistory.mockResolvedValueOnce([]).mockResolvedValueOnce([]);
    clearHistory.mockResolvedValue(3);

    const store = useHistoryStore();
    await store.load("video");
    await store.clear();

    expect(clearHistory).toHaveBeenCalledTimes(1);
    expect(store.query).toBe("");
  });
});

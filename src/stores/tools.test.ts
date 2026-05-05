import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ToolStatus } from "../types/models";

const getToolStatus = vi.fn();

vi.mock("../services/tauri/api", () => ({
  api: {
    getToolStatus: (...args: unknown[]) => getToolStatus(...args)
  }
}));

import { useToolsStore } from "./tools";

beforeEach(() => {
  setActivePinia(createPinia());
  getToolStatus.mockReset();
});

describe("tools store", () => {
  it("loads the current tool status list", async () => {
    const tool: ToolStatus = {
      name: "yt-dlp",
      installed: true,
      currentVersion: "2026.05.05",
      path: "C:/Tools/yt-dlp.exe"
    };
    getToolStatus.mockResolvedValue([tool]);

    const store = useToolsStore();
    await store.refresh();

    expect(store.tools).toEqual([tool]);
    expect(store.loading).toBe(false);
  });
});

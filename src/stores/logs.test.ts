import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { LogEntry } from "../types/models";

const tailLogs = vi.fn();

vi.mock("../services/tauri/api", () => ({
  api: {
    tailLogs: (...args: unknown[]) => tailLogs(...args)
  }
}));

import { useLogsStore } from "./logs";

beforeEach(() => {
  setActivePinia(createPinia());
  tailLogs.mockReset();
});

describe("logs store", () => {
  it("loads and parses recent logs from the backend", async () => {
    const entry: LogEntry = {
      level: "info",
      message: "Started",
      timestamp: "2026-05-05T10:00:00Z"
    };
    tailLogs.mockResolvedValue(["[2026-05-05T10:00:00Z] [INFO] Started"]);

    const store = useLogsStore();
    await store.load();

    expect(store.entries).toEqual([entry]);
    expect(store.rawText).toBe("[2026-05-05T10:00:00Z] [INFO] Started");
  });
});

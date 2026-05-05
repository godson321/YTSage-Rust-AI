import { describe, expect, it } from "vitest";
import type { DownloadTaskState } from "../types/models";

describe("model contracts", () => {
  it("includes expected download task states", () => {
    const states: DownloadTaskState[] = [
      "queued",
      "analyzing",
      "ready",
      "downloading",
      "paused",
      "completed",
      "failed",
      "cancelled"
    ];

    expect(states).toHaveLength(8);
    expect(states).toContain("downloading");
    expect(states).toContain("failed");
  });
});

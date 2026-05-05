import { describe, expect, it } from "vitest";
import type { DownloadTaskState, FormatItem } from "../types/models";

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

  it("supports rich format table fields", () => {
    const item: FormatItem = {
      formatId: "137",
      ext: "mp4",
      resolution: "1920x1080",
      filesize: 104857600,
      hasAudio: false,
      isAudioOnly: false,
      videoCodec: "avc1.640028",
      audioCodec: "none",
      fps: 60,
      dynamicRange: "SDR"
    };

    expect(item.videoCodec).toBe("avc1.640028");
    expect(item.audioCodec).toBe("none");
    expect(item.fps).toBe(60);
    expect(item.dynamicRange).toBe("SDR");
  });
});

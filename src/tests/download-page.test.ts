import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import {
  describeAudioState,
  formatDuration,
  formatFileSize,
  getPrimaryAnalysisItem
} from "../pages/download-page";
import type { AnalysisItem } from "../types/models";

describe("download page helpers", () => {
  it("formats duration for video details", () => {
    expect(formatDuration(95)).toBe("01:35");
    expect(formatDuration(3661)).toBe("1:01:01");
    expect(formatDuration(0)).toBe("00:00");
  });

  it("formats filesize for the formats table", () => {
    expect(formatFileSize(73400320)).toBe("70.0 MB");
    expect(formatFileSize(104857600)).toBe("100.0 MB");
    expect(formatFileSize(undefined)).toBe("-");
  });

  it("describes audio state for format rows", () => {
    expect(
      describeAudioState({
        formatId: "251",
        ext: "webm",
        hasAudio: true,
        isAudioOnly: true
      })
    ).toBe("仅音频");

    expect(
      describeAudioState({
        formatId: "18",
        ext: "mp4",
        hasAudio: true,
        isAudioOnly: false
      })
    ).toBe("含音频");

    expect(
      describeAudioState({
        formatId: "137",
        ext: "mp4",
        hasAudio: false,
        isAudioOnly: false
      })
    ).toBe("需合并音频");
  });

  it("derives the primary analysis summary from a successful item", () => {
    const item: AnalysisItem = {
      sourceUrl: "https://example.com/watch?v=abc",
      normalizedUrl: "https://example.com/watch?v=abc",
      status: "success",
      isPlaylist: true,
      thumbnailUrl: "https://img.example/thumb.jpg",
      playlistEntries: [{ id: "a" }, { id: "b" }, { id: "c" }],
      mediaInfo: {
        id: "abc",
        title: "Sample Video",
        channel: "Sample Channel",
        durationSec: 95,
        thumbnailUrl: "https://img.example/thumb-fallback.jpg",
        isPlaylist: true,
        playlistCount: 3
      },
      formats: [
        {
          formatId: "137",
          ext: "mp4",
          resolution: "1920x1080",
          filesize: 104857600,
          hasAudio: false,
          isAudioOnly: false
        }
      ]
    };

    const summary = getPrimaryAnalysisItem(item);

    expect(summary).not.toBeNull();
    expect(summary?.thumbnailUrl).toBe("https://img.example/thumb.jpg");
    expect(summary?.title).toBe("Sample Video");
    expect(summary?.channel).toBe("Sample Channel");
    expect(summary?.durationText).toBe("01:35");
    expect(summary?.playlistCountText).toBe("播放列表，共 3 项");
    expect(summary?.formats).toHaveLength(1);
  });

  it("returns null for failed items", () => {
    const item: AnalysisItem = {
      sourceUrl: "https://example.com/watch?v=missing",
      normalizedUrl: "https://example.com/watch?v=missing",
      status: "failed",
      isPlaylist: false,
      error: {
        code: "analysis.invalid_url",
        message: "Invalid URL",
        recoverable: false
      }
    };

    expect(getPrimaryAnalysisItem(item)).toBeNull();
  });
});

describe("DownloadPage source", () => {
  it("renders video details and formats table structure", () => {
    const content = readFileSync(resolve(process.cwd(), "src/pages/DownloadPage.vue"), "utf8");

    expect(content).toContain("download-page__details");
    expect(content).toContain("download-page__thumbnail");
    expect(content).toContain('label="频道"');
    expect(content).toContain('label="时长"');
    expect(content).toContain("分析结果详情");
    expect(content).toContain("可用格式");
    expect(content).toContain('field="formatId"');
    expect(content).toContain('field="ext"');
    expect(content).toContain('field="resolution"');
    expect(content).toContain('field="filesize"');
    expect(content).toContain('field="videoCodec"');
    expect(content).toContain('field="audioCodec"');
    expect(content).toContain('field="fps"');
    expect(content).toContain('field="dynamicRange"');
  });
});

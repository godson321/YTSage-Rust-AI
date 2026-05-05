import type { AnalysisItem, FormatItem } from "../types/models";

export interface DownloadPageSummary {
  thumbnailUrl: string | null;
  title: string;
  channel: string;
  durationText: string;
  isPlaylist: boolean;
  playlistCountText: string | null;
  formats: FormatItem[];
}

export function formatDuration(durationSec?: number): string {
  if (!durationSec || durationSec <= 0) {
    return "00:00";
  }

  const totalSeconds = Math.floor(durationSec);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
  }

  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}

export function formatFileSize(filesize?: number): string {
  if (!filesize || filesize <= 0) {
    return "-";
  }

  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = filesize;
  let unitIndex = 0;

  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }

  if (unitIndex === 0) {
    return `${Math.round(value)} ${units[unitIndex]}`;
  }

  return `${value.toFixed(value >= 10 ? 1 : 2)} ${units[unitIndex]}`;
}

export function describeAudioState(format: FormatItem): string {
  if (format.isAudioOnly) {
    return "仅音频";
  }

  if (format.hasAudio) {
    return "含音频";
  }

  return "需合并音频";
}

export function getPrimaryAnalysisItem(item: AnalysisItem | null | undefined): DownloadPageSummary | null {
  if (!item || item.status !== "success" || !item.mediaInfo) {
    return null;
  }

  const playlistCount = item.mediaInfo.playlistCount ?? item.playlistEntries?.length ?? 0;

  return {
    thumbnailUrl: item.thumbnailUrl ?? item.mediaInfo.thumbnailUrl ?? null,
    title: item.mediaInfo.title || "未命名视频",
    channel: item.mediaInfo.channel || "未知频道",
    durationText: formatDuration(item.mediaInfo.durationSec),
    isPlaylist: item.isPlaylist,
    playlistCountText: item.isPlaylist ? `播放列表，共 ${playlistCount} 项` : null,
    formats: item.formats ?? []
  };
}

export type DownloadTaskState =
  | "queued"
  | "analyzing"
  | "ready"
  | "downloading"
  | "paused"
  | "completed"
  | "failed"
  | "cancelled";

export interface AnalyzeRequest {
  urls: string[];
  genericMode: boolean;
  proxyUrl?: string;
  geoProxyUrl?: string;
  cookieSource?: "none" | "file" | "browser" | "embedded";
  cookieFilePath?: string;
  browserCookiesOption?: string;
}

export interface MediaInfo {
  id: string;
  title: string;
  channel: string;
  durationSec: number;
  thumbnailUrl?: string;
  isPlaylist: boolean;
  playlistCount?: number;
}

export interface FormatItem {
  formatId: string;
  ext: string;
  resolution?: string;
  filesize?: number;
  hasAudio: boolean;
  isAudioOnly: boolean;
  videoCodec?: string;
  audioCodec?: string;
  fps?: number;
  dynamicRange?: string;
}

export interface AnalysisItem {
  sourceUrl: string;
  normalizedUrl: string;
  status: "success" | "failed";
  error?: AnalysisError;
  isPlaylist: boolean;
  mediaInfo?: MediaInfo;
  formats?: FormatItem[];
  thumbnailUrl?: string;
  playlistInfo?: unknown;
  playlistEntries?: unknown[];
  videoInfo?: unknown;
  availableSubtitles?: unknown;
  availableAutomaticSubtitles?: unknown;
}

export interface AnalysisResult {
  jobId: string;
  items: AnalysisItem[];
}

export interface AnalysisError {
  code: string;
  message: string;
  detail?: string;
  recoverable: boolean;
}

export interface DownloadOptions {
  outputDir: string;
  formatId: string;
  audioOnly: boolean;
  mergeSubs: boolean;
  saveDescription: boolean;
  saveThumbnail: boolean;
  embedChapters: boolean;
  rateLimit?: string;
  playlistItems?: string | null;
  subtitleLangs: string[];
  enableSponsorblock: boolean;
  sponsorblockCategories: string[];
  resolution?: string | null;
  downloadSection?: string | null;
  forceKeyframes: boolean;
  proxyUrl?: string | null;
  geoProxyUrl?: string | null;
  forceOutputFormat: boolean;
  preferredOutputFormat?: string | null;
  forceAudioFormat: boolean;
  preferredAudioFormat?: string | null;
  audioNormalization: boolean;
  filenameFormat?: string | null;
  cookieFilePath?: string | null;
  browserCookiesOption?: string | null;
}

export interface DownloadTask {
  taskId: string;
  sourceUrl: string;
  title?: string | null;
  state: DownloadTaskState;
  progress: number;
  speedText?: string | null;
  etaText?: string | null;
  outputPath?: string | null;
  error?: string | null;
  requestedOptions?: DownloadOptions;
}

export interface QueueState {
  tasks: DownloadTask[];
  activeTaskId?: string | null;
  retryCount: number;
}

export interface AppSettings {
  downloadPath: string;
  genericMode: boolean;
  language: string;
  proxyUrl?: string;
  geoProxyUrl?: string;
  filenameFormat: string;
}

export interface HistoryEntry {
  id: string;
  title: string;
  url: string;
  channel: string;
  filePath: string;
  downloadDate?: string | null;
  fileSize?: number | null;
  thumbnailUrl?: string | null;
  formatId?: string | null;
  resolution?: string | null;
  isAudioOnly: boolean;
  duration?: string | null;
  downloadOptions?: unknown;
}

export interface ToolStatus {
  name: string;
  installed: boolean;
  currentVersion?: string;
  path?: string;
}

export interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
}

export interface UpdateStatus {
  currentVersion: string;
  latestVersion?: string | null;
  updateAvailable: boolean;
  source: string;
}

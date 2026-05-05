import { defineStore } from "pinia";
import { api } from "../services/tauri/api";
import type { LogEntry } from "../types/models";

function parseLogLine(line: string): LogEntry {
  const match = line.match(/^\[(?<timestamp>[^\]]+)\]\s+\[(?<level>[^\]]+)\]\s*(?<message>.*)$/);
  if (!match?.groups) {
    return {
      timestamp: "",
      level: "info",
      message: line
    };
  }

  return {
    timestamp: match.groups.timestamp,
    level: match.groups.level.toLowerCase(),
    message: match.groups.message
  };
}

export const useLogsStore = defineStore("logs", {
  state: () => ({
    entries: [] as LogEntry[],
    loading: false,
    rawText: ""
  }),
  actions: {
    async load(lines = 200) {
      this.loading = true;
      try {
        const rawLines = await api.tailLogs(lines);
        this.rawText = rawLines.join("\n");
        this.entries = rawLines.map(parseLogLine);
      } finally {
        this.loading = false;
      }
    }
  }
});

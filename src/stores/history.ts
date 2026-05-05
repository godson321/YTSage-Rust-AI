import { defineStore } from "pinia";
import { api } from "../services/tauri/api";
import type { HistoryEntry } from "../types/models";

export const useHistoryStore = defineStore("history", {
  state: () => ({
    entries: [] as HistoryEntry[],
    loading: false,
    query: ""
  }),
  actions: {
    async load(query?: string) {
      this.loading = true;
      const nextQuery = query ?? this.query;
      this.query = nextQuery;
      try {
        const trimmed = nextQuery.trim();
        this.entries = trimmed ? await api.searchHistory(trimmed) : await api.listHistory();
      } finally {
        this.loading = false;
      }
    },
    async deleteEntry(entryId: string) {
      await api.removeHistoryEntry(entryId);
      await this.load();
    },
    async clear() {
      await api.clearHistory();
      await this.load("");
    }
  }
});

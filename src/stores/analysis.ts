import { defineStore } from "pinia";
import type { AnalysisResult, AnalyzeRequest } from "../types/models";
import { api } from "../services/tauri/api";

export const useAnalysisStore = defineStore("analysis", {
  state: () => ({
    loading: false,
    result: null as AnalysisResult | null
  }),
  actions: {
    async analyze(payload: AnalyzeRequest) {
      this.loading = true;
      try {
        this.result = await api.analyzeUrls(payload);
      } finally {
        this.loading = false;
      }
    },
    clear() {
      this.loading = false;
      this.result = null;
    }
  }
});

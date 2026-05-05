import { defineStore } from "pinia";
import { api } from "../services/tauri/api";
import type { ToolStatus } from "../types/models";

export const useToolsStore = defineStore("tools", {
  state: () => ({
    tools: [] as ToolStatus[],
    loading: false
  }),
  actions: {
    async refresh() {
      this.loading = true;
      try {
        this.tools = await api.getToolStatus();
      } finally {
        this.loading = false;
      }
    }
  }
});

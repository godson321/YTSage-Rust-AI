import { defineStore } from "pinia";
import type { AppSettings } from "../types/models";
import { api } from "../services/tauri/api";

export const defaultSettings: AppSettings = {
  downloadPath: "",
  genericMode: false,
  language: "zh",
  proxyUrl: undefined,
  geoProxyUrl: undefined,
  filenameFormat: "%(title)s_%(resolution)s.%(ext)s"
};

export const useSettingsStore = defineStore("settings", {
  state: () => ({
    settings: { ...defaultSettings }
  }),
  actions: {
    async load() {
      this.settings = await api.getSettings();
    },
    async save() {
      this.settings = await api.saveSettings(this.settings);
    }
  }
});

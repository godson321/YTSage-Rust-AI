import { createRouter, createWebHashHistory } from "vue-router";

import DownloadPage from "../pages/DownloadPage.vue";
import BatchPage from "../pages/BatchPage.vue";
import HistoryPage from "../pages/HistoryPage.vue";
import SettingsPage from "../pages/SettingsPage.vue";
import ToolsPage from "../pages/ToolsPage.vue";
import LogsPage from "../pages/LogsPage.vue";
import AboutPage from "../pages/AboutPage.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: "/download" },
    { path: "/download", component: DownloadPage },
    { path: "/batch", component: BatchPage },
    { path: "/history", component: HistoryPage },
    { path: "/settings", component: SettingsPage },
    { path: "/tools", component: ToolsPage },
    { path: "/logs", component: LogsPage },
    { path: "/about", component: AboutPage }
  ]
});

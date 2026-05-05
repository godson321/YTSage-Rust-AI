import { createApp } from "vue";
import { createPinia } from "pinia";
import ElementPlus from "element-plus";
import "element-plus/dist/index.css";
import VxeUITable from "vxe-table";
import "vxe-table/lib/style.css";

import App from "./App.vue";
import { router } from "./router";
import { useQueueStore } from "./stores/queue";

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);
app.use(ElementPlus);
app.use(VxeUITable);

const queueStore = useQueueStore(pinia);
void queueStore.startQueueSync();

app.mount("#app");

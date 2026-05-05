<template>
  <el-card>
    <template #header>Logs</template>
    <el-space>
      <el-button @click="loadLogs">Refresh</el-button>
    </el-space>
    <el-input v-model="store.rawText" type="textarea" :rows="18" readonly />
  </el-card>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { useLogsStore } from "../stores/logs";

const store = useLogsStore();

async function loadLogs() {
  await store.load(200);
}

onMounted(loadLogs);
</script>

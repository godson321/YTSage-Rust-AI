<template>
  <el-card class="batch-page">
    <template #header>批量任务</template>
    <el-space direction="vertical" fill class="batch-page__content">
      <el-input v-model="urlInput" type="textarea" :rows="4" placeholder="Paste one or more URLs" />
      <el-space>
        <el-button type="primary" :loading="queue.loading" @click="enqueueUrls">Enqueue</el-button>
        <el-button @click="queue.clear()">Clear</el-button>
      </el-space>
      <vxe-table :data="queue.tasks" border round stripe auto-resize class="data-table">
        <vxe-column field="sourceUrl" title="Source" min-width="280" />
        <vxe-column field="state" title="State" width="130" />
        <vxe-column title="Progress" width="120">
          <template #default="{ row }">
            {{ Math.round(row.progress * 100) }}%
          </template>
        </vxe-column>
      </vxe-table>
    </el-space>
  </el-card>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useQueueStore } from "../stores/queue";
import { extractUrlsFromText } from "../utils/url";

const urlInput = ref("");
const queue = useQueueStore();

async function enqueueUrls() {
  const urls = extractUrlsFromText(urlInput.value);

  for (const url of urls) {
    await queue.enqueue(url);
  }
}
</script>

<style scoped>
.data-table {
  width: 100%;
}
</style>

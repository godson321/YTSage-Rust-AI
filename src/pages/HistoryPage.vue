<template>
  <el-card>
    <template #header>History</template>
    <el-space>
      <el-input v-model="query" placeholder="Search history" clearable />
      <el-button @click="loadHistory">Refresh</el-button>
      <el-button type="danger" @click="clearEntries">Clear</el-button>
    </el-space>
    <vxe-table :data="store.entries" border round stripe auto-resize class="data-table">
      <vxe-column field="title" title="Title" min-width="220" />
      <vxe-column field="channel" title="Channel" width="160" />
      <vxe-column field="url" title="URL" min-width="260" />
      <vxe-column field="filePath" title="File" min-width="260" />
      <vxe-column title="Actions" width="140">
        <template #default="{ row }">
          <el-button link type="danger" @click="removeEntry(row.id)">Delete</el-button>
        </template>
      </vxe-column>
    </vxe-table>
  </el-card>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { useHistoryStore } from "../stores/history";

const query = ref("");
const store = useHistoryStore();

async function loadHistory() {
  await store.load(query.value);
}

async function clearEntries() {
  await store.clear();
  query.value = "";
}

async function removeEntry(entryId: string) {
  await store.deleteEntry(entryId);
}

onMounted(loadHistory);
watch(query, () => {
  void loadHistory();
});
</script>

<style scoped>
.data-table {
  width: 100%;
  margin-top: 16px;
}
</style>

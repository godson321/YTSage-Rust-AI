<template>
  <el-card>
    <template #header>Tools</template>
    <el-space>
      <el-button @click="loadTools">Refresh</el-button>
    </el-space>
    <vxe-table :data="store.tools" border round stripe auto-resize class="data-table">
      <vxe-column field="name" title="Name" width="120" />
      <vxe-column title="Installed" width="100">
        <template #default="{ row }">
          {{ row.installed ? "Yes" : "No" }}
        </template>
      </vxe-column>
      <vxe-column field="currentVersion" title="Version" width="180" />
      <vxe-column field="path" title="Path" min-width="260" />
    </vxe-table>
  </el-card>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { useToolsStore } from "../stores/tools";

const store = useToolsStore();

async function loadTools() {
  await store.refresh();
}

onMounted(loadTools);
</script>

<style scoped>
.data-table {
  width: 100%;
  margin-top: 16px;
}
</style>

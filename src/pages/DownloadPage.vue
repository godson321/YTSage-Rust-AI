<template>
  <el-card>
    <template #header>下载主页</template>
    <el-space direction="vertical" fill>
      <el-input
        v-model="urlInput"
        type="textarea"
        :rows="4"
        placeholder="输入一个或多个视频/播放列表地址"
      />
      <el-space>
        <el-switch v-model="genericMode" active-text="Generic Mode" />
        <el-button type="primary" :loading="analysis.loading" @click="onAnalyze">分析</el-button>
      </el-space>
      <vxe-table
        v-if="analysis.result"
        :data="analysis.result.items"
        border
        round
        stripe
        auto-resize
        class="data-table"
      >
        <vxe-column field="sourceUrl" title="输入地址" min-width="280" />
        <vxe-column field="status" title="状态" width="120" />
        <vxe-column title="标题" min-width="320">
          <template #default="{ row }">
            {{ row.mediaInfo?.title ?? row.error?.message ?? "-" }}
          </template>
        </vxe-column>
      </vxe-table>
    </el-space>
  </el-card>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useAnalysisStore } from "../stores/analysis";
import { extractUrlsFromText } from "../utils/url";

const analysis = useAnalysisStore();
const urlInput = ref("");
const genericMode = ref(false);

async function onAnalyze() {
  const urls = extractUrlsFromText(urlInput.value);
  if (!urls.length) {
    return;
  }

  await analysis.analyze({
    urls,
    genericMode: genericMode.value,
    cookieSource: "none"
  });
}
</script>

<style scoped>
.data-table {
  width: 100%;
}
</style>

<template>
  <el-card class="download-page">
    <template #header>下载主页</template>
    <el-space direction="vertical" fill class="download-page__content">
      <el-input
        v-model="urlInput"
        type="textarea"
        :rows="4"
        placeholder="输入一个或多个视频/播放列表地址"
      />

      <el-space wrap>
        <el-switch v-model="genericMode" active-text="Generic Mode" />
        <el-button type="primary" :loading="analysis.loading" @click="onAnalyze">分析</el-button>
        <el-button :disabled="!analysis.result" @click="analysis.clear()">清空结果</el-button>
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
        <vxe-column title="详情" width="120">
          <template #default="{ row }">
            <el-button link type="primary" @click="selectedUrl = row.sourceUrl">查看</el-button>
          </template>
        </vxe-column>
      </vxe-table>

      <el-empty
        v-if="analysis.result && !selectedItem"
        description="当前结果没有可展示的视频详情"
      />

      <section v-if="selectedSummary" class="download-page__details">
        <div class="download-page__details-header">
          <h3>分析结果详情</h3>
          <el-tag v-if="selectedSummary.isPlaylist" type="warning">
            {{ selectedSummary.playlistCountText }}
          </el-tag>
        </div>

        <div class="download-page__media">
          <div class="download-page__thumbnail-wrap">
            <img
              v-if="selectedSummary.thumbnailUrl"
              :src="selectedSummary.thumbnailUrl"
              :alt="selectedSummary.title"
              class="download-page__thumbnail"
            />
            <div v-else class="download-page__thumbnail download-page__thumbnail--placeholder">
              暂无缩略图
            </div>
          </div>

          <el-card shadow="never" class="download-page__meta">
            <template #header>{{ selectedSummary.title }}</template>
            <el-descriptions :column="1" border>
              <el-descriptions-item label="频道">
                {{ selectedSummary.channel }}
              </el-descriptions-item>
              <el-descriptions-item label="时长">
                {{ selectedSummary.durationText }}
              </el-descriptions-item>
              <el-descriptions-item label="原始地址">
                <span class="download-page__url">{{ selectedItem?.normalizedUrl }}</span>
              </el-descriptions-item>
            </el-descriptions>
          </el-card>
        </div>

        <el-alert
          v-if="selectedSummary.isPlaylist && selectedSummary.playlistCountText"
          type="info"
          :closable="false"
          show-icon
          :title="selectedSummary.playlistCountText"
        />

        <el-card shadow="never" class="download-page__formats-card">
          <template #header>可用格式</template>
          <vxe-table
            :data="selectedSummary.formats"
            border
            round
            stripe
            auto-resize
            class="data-table"
          >
            <vxe-column field="formatId" title="格式 ID" width="120" />
            <vxe-column field="ext" title="扩展名" width="100" />
            <vxe-column field="resolution" title="分辨率" min-width="140" />
            <vxe-column field="filesize" title="文件大小" width="120">
              <template #default="{ row }">
                {{ formatFileSize(row.filesize) }}
              </template>
            </vxe-column>
            <vxe-column field="videoCodec" title="视频编码" min-width="150">
              <template #default="{ row }">
                {{ row.videoCodec || "-" }}
              </template>
            </vxe-column>
            <vxe-column field="audioCodec" title="音频编码" min-width="150">
              <template #default="{ row }">
                {{ row.audioCodec || "-" }}
              </template>
            </vxe-column>
            <vxe-column title="音频状态" width="120">
              <template #default="{ row }">
                {{ describeAudioState(row) }}
              </template>
            </vxe-column>
            <vxe-column field="fps" title="帧率" width="100">
              <template #default="{ row }">
                {{ row.fps ? `${Math.round(row.fps)} fps` : "-" }}
              </template>
            </vxe-column>
            <vxe-column field="dynamicRange" title="动态范围" width="120">
              <template #default="{ row }">
                {{ row.dynamicRange || "-" }}
              </template>
            </vxe-column>
          </vxe-table>
        </el-card>
      </section>

      <el-alert
        v-else-if="selectedItem?.status === 'failed'"
        type="error"
        :closable="false"
        show-icon
        :title="selectedItem.error?.message ?? '分析失败'"
        :description="selectedItem.error?.detail"
      />
    </el-space>
  </el-card>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useAnalysisStore } from "../stores/analysis";
import { extractUrlsFromText } from "../utils/url";
import {
  describeAudioState,
  formatFileSize,
  getPrimaryAnalysisItem
} from "./download-page";

const analysis = useAnalysisStore();
const urlInput = ref("");
const genericMode = ref(false);
const selectedUrl = ref<string | null>(null);

const selectedItem = computed(() => {
  const items = analysis.result?.items ?? [];
  if (!items.length) {
    return null;
  }

  if (selectedUrl.value) {
    return items.find((item) => item.sourceUrl === selectedUrl.value) ?? items[0];
  }

  return items[0];
});

const selectedSummary = computed(() => getPrimaryAnalysisItem(selectedItem.value));

watch(
  () => analysis.result,
  (result) => {
    selectedUrl.value = result?.items[0]?.sourceUrl ?? null;
  }
);

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
.download-page {
  width: 100%;
}

.download-page__content {
  width: 100%;
}

.data-table {
  width: 100%;
}

.download-page__details {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.download-page__details-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.download-page__details-header h3 {
  margin: 0;
  font-size: 18px;
}

.download-page__media {
  display: grid;
  grid-template-columns: minmax(280px, 360px) minmax(0, 1fr);
  gap: 16px;
  align-items: start;
}

.download-page__thumbnail-wrap {
  width: 100%;
}

.download-page__thumbnail {
  display: block;
  width: 100%;
  aspect-ratio: 16 / 9;
  object-fit: cover;
  border-radius: 12px;
  background: linear-gradient(135deg, #d7dde8 0%, #f4f7fb 100%);
  border: 1px solid #d7deea;
}

.download-page__thumbnail--placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #6b7280;
}

.download-page__meta {
  min-width: 0;
}

.download-page__url {
  word-break: break-all;
}

.download-page__formats-card {
  width: 100%;
}

@media (max-width: 900px) {
  .download-page__media {
    grid-template-columns: 1fr;
  }
}
</style>

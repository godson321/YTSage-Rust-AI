<template>
  <el-card>
    <template #header>设置</template>
    <el-form label-width="140px">
      <el-form-item label="下载目录">
        <el-input v-model="store.settings.downloadPath" />
      </el-form-item>
      <el-form-item label="文件名模板">
        <el-input v-model="store.settings.filenameFormat" />
      </el-form-item>
      <el-form-item label="Generic Mode">
        <el-switch v-model="store.settings.genericMode" />
      </el-form-item>
      <el-form-item label="语言">
        <el-select v-model="store.settings.language">
          <el-option label="中文" value="zh" />
          <el-option label="English" value="en" />
        </el-select>
      </el-form-item>
      <el-form-item>
        <el-button type="primary" @click="store.save()">保存</el-button>
      </el-form-item>
    </el-form>
  </el-card>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { useSettingsStore } from "../stores/settings";

const store = useSettingsStore();

onMounted(() => {
  void store.load();
});
</script>

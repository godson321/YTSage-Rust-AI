<template>
  <el-container class="app-shell">
    <el-aside width="220px" class="sidebar">
      <div class="brand">YTSage Rust</div>
      <el-menu :default-active="route.path" router class="menu">
        <el-menu-item index="/download">下载</el-menu-item>
        <el-menu-item index="/batch">批量任务</el-menu-item>
        <el-menu-item index="/history">历史</el-menu-item>
        <el-menu-item index="/settings">设置</el-menu-item>
        <el-menu-item index="/tools">工具管理</el-menu-item>
        <el-menu-item index="/logs">日志</el-menu-item>
        <el-menu-item index="/about">关于</el-menu-item>
      </el-menu>
    </el-aside>
    <el-container>
      <el-header class="header">
        <span>{{ currentTitle }}</span>
      </el-header>
      <el-main class="main">
        <router-view />
      </el-main>
    </el-container>
  </el-container>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";

const route = useRoute();

const titleMap: Record<string, string> = {
  "/download": "下载主页",
  "/batch": "批量任务",
  "/history": "历史记录",
  "/settings": "设置",
  "/tools": "工具管理",
  "/logs": "日志与诊断",
  "/about": "关于"
};

const currentTitle = computed(() => titleMap[route.path] ?? "YTSage Rust");
</script>

<style scoped>
.app-shell {
  min-height: 100vh;
}

.sidebar {
  background: #111827;
  color: #fff;
}

.brand {
  padding: 20px;
  font-size: 20px;
  font-weight: 700;
}

.menu {
  border-right: none;
}

.header {
  display: flex;
  align-items: center;
  background: #fff;
  border-bottom: 1px solid #e5e7eb;
  font-size: 18px;
  font-weight: 600;
}

.main {
  background: #f5f7fa;
}
</style>

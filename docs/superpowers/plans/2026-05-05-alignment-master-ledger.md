# 对齐总账落地实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在仓库中建立 `docs/alignment-master.md` 作为 YTSage 与 YTSage-Rust 功能对齐状态的唯一总账，并将仓库入口文档接入该总账。

**Architecture:** 本次工作只处理文档基础设施，不直接实现业务功能。先修复现有计划文件编码并统一中文，再创建唯一总账，最后把 `README.md` 接到总账入口，确保后续每次对齐都优先更新总账而不是重新全量盘点旧项目。

**Tech Stack:** Markdown、PowerShell、ripgrep、现有仓库文档

---

## 文件结构

- 创建：`docs/alignment-master.md`
  责任：作为 YTSage 功能对齐状态的唯一来源，维护状态定义、功能总表、优先级和更新流程。
- 修改：`README.md`
  责任：暴露总账入口，并明确声明总账是唯一来源。
- 修改：`docs/superpowers/plans/2026-05-05-alignment-master-ledger.md`
  责任：保存本次“总账落地”的实施计划，供后续子代理按任务执行。

### Task 1: 修复计划文件并统一中文

**Files:**
- Modify: `docs/superpowers/plans/2026-05-05-alignment-master-ledger.md`
- Test: `docs/superpowers/plans/2026-05-05-alignment-master-ledger.md`

- [ ] **Step 1: 先写失败校验**

```powershell
$content = Get-Content 'docs/superpowers/plans/2026-05-05-alignment-master-ledger.md' -Raw
if ($content -notmatch '^# 对齐总账落地实施计划' -or
    $content -notmatch '\*\*Goal:\*\*' -or
    $content -notmatch '### Task 1: 修复计划文件并统一中文') {
  throw 'plan header or task sections are missing'
}
```

- [ ] **Step 2: 运行校验并确认失败**

运行：
```powershell
powershell -NoProfile -Command "$content = Get-Content 'docs/superpowers/plans/2026-05-05-alignment-master-ledger.md' -Raw; if ($content -notmatch '^# 对齐总账落地实施计划' -or $content -notmatch '\*\*Goal:\*\*' -or $content -notmatch '### Task 1: 修复计划文件并统一中文') { throw 'plan header or task sections are missing' }"
```

预期：失败，说明原文件编码或结构有损坏，需要整体重写。

- [ ] **Step 3: 写入中文规范计划**

```markdown
# 对齐总账落地实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在仓库中建立 `docs/alignment-master.md` 作为 YTSage 与 YTSage-Rust 功能对齐状态的唯一总账，并将仓库入口文档接入该总账。

**Architecture:** 本次工作只处理文档基础设施，不直接实现业务功能。先修复现有计划文件编码并统一中文，再创建唯一总账，最后把 `README.md` 接到总账入口，确保后续每次对齐都优先更新总账而不是重新全量盘点旧项目。

**Tech Stack:** Markdown、PowerShell、ripgrep、现有仓库文档
```

- [ ] **Step 4: 重新运行校验并确认通过**

运行：
```powershell
rg -n "^# 对齐总账落地实施计划$|^\*\*Goal:\*\*|^### Task 1: 修复计划文件并统一中文$" "docs/superpowers/plans/2026-05-05-alignment-master-ledger.md"
```

预期：
```text
1:# 对齐总账落地实施计划
5:**Goal:** 在仓库中建立 `docs/alignment-master.md` 作为 YTSage 与 YTSage-Rust 功能对齐状态的唯一总账，并将仓库入口文档接入该总账。
24:### Task 1: 修复计划文件并统一中文
```

- [ ] **Step 5: 提交**

```powershell
if (Test-Path '.git') {
  git add docs/superpowers/plans/2026-05-05-alignment-master-ledger.md
  git commit -m "docs: rewrite alignment ledger plan in chinese"
} else {
  Write-Host "Skip commit: no .git metadata in F:\Github\YTSage-Rust"
}
```

### Task 2: 创建唯一总账文档

**Files:**
- Create: `docs/alignment-master.md`
- Test: `docs/alignment-master.md`

- [ ] **Step 1: 先写失败校验**

```powershell
if (-not (Test-Path 'docs/alignment-master.md')) {
  throw 'docs/alignment-master.md is missing'
}

$content = Get-Content 'docs/alignment-master.md' -Raw
if ($content -notmatch '^# YTSage 对齐总账' -or
    $content -notmatch '^## 功能总表' -or
    $content -notmatch '^## 更新流程') {
  throw 'alignment master headings are missing'
}
```

- [ ] **Step 2: 运行校验并确认失败**

运行：
```powershell
powershell -NoProfile -Command "if (-not (Test-Path 'docs/alignment-master.md')) { throw 'docs/alignment-master.md is missing' }; $content = Get-Content 'docs/alignment-master.md' -Raw; if ($content -notmatch '^# YTSage 对齐总账' -or $content -notmatch '^## 功能总表' -or $content -notmatch '^## 更新流程') { throw 'alignment master headings are missing' }"
```

预期：失败，报错 `docs/alignment-master.md is missing`。

- [ ] **Step 3: 写入首版总账**

```markdown
# YTSage 对齐总账

> 本文件是 `F:\Github\YTSage-Rust` 中 YTSage 功能对齐状态的唯一来源。

## 目的

- 用“用户可感知功能”而不是零散字段或页面骨架跟踪对齐状态。
- 后续每次功能对齐优先更新本文件，不再重复做整库重盘。
- 只有在当前功能行需要核对行为细节时，才回看原版 YTSage 对应代码入口。

## 功能总表

首版写入 20 个功能项，并记录状态、深度、优先级、缺口摘要和最后核对日期。

## 更新流程

1. 先读本文件，再决定是否回看原版代码。
2. 只推进本次目标功能行，不无目标重扫老项目。
3. 实现或验证完成后，立刻回填该行状态和缺口。
```

- [ ] **Step 4: 重新运行校验并确认通过**

运行：
```powershell
rg -n "^# YTSage 对齐总账$|^## 功能总表$|^## 更新流程$" "docs/alignment-master.md"
```

预期：
```text
1:# YTSage 对齐总账
11:## 功能总表
15:## 更新流程
```

- [ ] **Step 5: 提交**

```powershell
if (Test-Path '.git') {
  git add docs/alignment-master.md
  git commit -m "docs: add alignment master ledger"
} else {
  Write-Host "Skip commit: no .git metadata in F:\Github\YTSage-Rust"
}
```

### Task 3: 将总账接入 README 入口

**Files:**
- Modify: `README.md`
- Test: `README.md`

- [ ] **Step 1: 先写失败校验**

```powershell
$content = Get-Content 'README.md' -Raw
if ($content -notmatch '\[对齐总账\]\(docs/alignment-master.md\)') {
  throw 'README is missing alignment master link'
}
if ($content -notmatch 'YTSage 功能对齐状态的唯一来源') {
  throw 'README is missing alignment master description'
}
```

- [ ] **Step 2: 运行校验并确认失败**

运行：
```powershell
powershell -NoProfile -Command "$content = Get-Content 'README.md' -Raw; if ($content -notmatch '\[对齐总账\]\(docs/alignment-master.md\)') { throw 'README is missing alignment master link' }; if ($content -notmatch 'YTSage 功能对齐状态的唯一来源') { throw 'README is missing alignment master description' }"
```

预期：失败，报错 `README is missing alignment master link`。

- [ ] **Step 3: 在 Documents 段增加中文入口**

```markdown
## Documents

- [对齐总账](docs/alignment-master.md) - YTSage 功能对齐状态的唯一来源
- [Architecture](docs/ARCHITECTURE.md)
- [Analysis Report](docs/ANALYSIS_REPORT.md)
- [API Design](docs/API.md)
- [Execution Plan](docs/PLAN.md)
- [Task Board](docs/TASKS.md)
- [Progress Log](docs/PROGRESS.md)
```

- [ ] **Step 4: 重新运行校验并确认通过**

运行：
```powershell
rg -n "对齐总账|YTSage 功能对齐状态的唯一来源" "README.md"
```

预期：
```text
97:- [对齐总账](docs/alignment-master.md) - YTSage 功能对齐状态的唯一来源
```

- [ ] **Step 5: 提交**

```powershell
if (Test-Path '.git') {
  git add README.md docs/alignment-master.md
  git commit -m "docs: expose alignment master in readme"
} else {
  Write-Host "Skip commit: no .git metadata in F:\Github\YTSage-Rust"
}
```

### Task 4: 执行最终文档校验

**Files:**
- Test: `docs/superpowers/plans/2026-05-05-alignment-master-ledger.md`
- Test: `docs/alignment-master.md`
- Test: `README.md`

- [ ] **Step 1: 运行总校验**

运行：
```powershell
rg -n "^# 对齐总账落地实施计划$|^### Task 1: 修复计划文件并统一中文$|^### Task 2: 创建唯一总账文档$" "docs/superpowers/plans/2026-05-05-alignment-master-ledger.md"
rg -n "^# YTSage 对齐总账$|^## 状态说明$|^## 功能总表$|^## 当前优先级顺序$|^## 更新流程$" "docs/alignment-master.md"
rg -n "对齐总账|YTSage 功能对齐状态的唯一来源" "README.md"
```

预期：三份文档都能命中标题、章节和入口链接。

- [ ] **Step 2: 人工复核关键内容**

```text
确认总账中的第 6 项“真实下载执行链”仍然是最高优先级 P0。
确认第 9、10、11、12 项仍受下载执行链未落地阻塞。
确认第 14 项“历史记录”仍保持“部分可用”而非“与原版对齐”。
```

- [ ] **Step 3: 若发现不一致，立即修正**

```markdown
如果总账中的状态、优先级或缺口摘要与当前仓库代码不一致，直接修改 `docs/alignment-master.md` 对应行，再重新执行上一步校验。
```

- [ ] **Step 4: 提交**

```powershell
if (Test-Path '.git') {
  git add docs/superpowers/plans/2026-05-05-alignment-master-ledger.md docs/alignment-master.md README.md
  git commit -m "docs: finalize alignment ledger baseline"
} else {
  Write-Host "Skip commit: no .git metadata in F:\Github\YTSage-Rust"
}
```

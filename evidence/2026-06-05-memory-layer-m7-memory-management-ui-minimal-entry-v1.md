# Evidence：Memory Layer M7 Memory Management UI Minimal Entry v1

日期：2026-06-05

## 结论

M7 记忆管理 UI 最小入口已完成一个前端只读切片。

接受为：

- 全局一级 `记忆` 入口已从 sidecar / revision 工程摘要替换为用户可理解的记忆中心。
- 新增 `MemoryManagementSummary` 前端只读读模型，聚合 FormalMemoryStore、MemoryCandidateStore、ObservationStore、MemoryLintStore 和 workflow task package memory injection summary。
- 正式记忆列表显示正式 / 候选区分、来源、版本、审计、scope、权限策略、模型外发策略、冲突 / lint 摘要和任务包 eligibility。
- 候选列表显示候选状态、风险、确认要求、采纳回链和“不是正式记忆”边界；未采纳候选显示为待审查材料。
- observation 只作为“观察来源”展示，并明确“观察不是正式记忆”。
- 任务包冻结快照摘要显示 included / excluded / review materials / fresh / stale；正式记忆的逐条资格只在数据可证明时显示“任务包冻结快照已引用”。
- 项目相关记忆摘要只保留轻量聚合，不把项目页扩成完整记忆治理后台。
- 离线 UI 测试覆盖正式记忆 / 候选视觉区分、禁止文案、来源 / 版本 / 审计 / lint / 任务包 eligibility。

不接受为：

- 中间版本记忆系统完成。
- 正式记忆生命周期操作完成。
- 知识库 / Obsidian 接口完成。
- 实体关系治理、维护任务、成熟模式或跨项目记忆完成。
- UI 可以直接写正式记忆。
- 真实 worker 或真实 Codex 已执行。
- 真实窗口 / 截图验收完成。

## 关键实现

- 新增 `prototypes/productized-desktop-shell/src/lib/memoryCenter.ts`：
  - 派生 `MemoryManagementSummary`。
  - 生成 `FormalMemoryListItem`、`MemoryCandidateListItem`、`ObservationSourceListItem`、任务包冻结快照摘要和项目相关记忆摘要。
  - 根据 open blocking lint finding、formal memory 状态和 model export policy 派生任务包 eligibility。
- 新增 `prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx`：
  - 全局 `记忆` 入口的只读 UI。
  - 正式记忆、候选记忆、详情、任务包冻结快照、lint、观察来源、项目摘要和最近变化分区。
  - 不渲染编辑、删除、废弃、冻结、归档、合并、拆分、上升全局或下沉项目按钮。
- 更新 `prototypes/productized-desktop-shell/src/App.tsx`：
  - 复用现有一级 `记忆` 导航入口，替换旧 placeholder。
- 更新 `prototypes/productized-desktop-shell/src/styles.css`：
  - 补最小记忆中心布局、详情区和响应式约束。
- 更新 `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`：
  - 新增 M7 记忆中心离线场景。
  - 先跑出缺 `memoryCenter` / `MemoryCenterView` 的失败，再实现通过。

## 验证

红灯记录：

- `npm run test:offline-interaction` 初次失败，原因是 `../src/lib/memoryCenter` 和 `../src/views/MemoryCenterView` 尚未实现。

通过记录：

- `npm run test:offline-interaction`：通过，输出 `offline interaction tests passed: 9`。
- `npm run typecheck`：通过。
- `npm run build`：通过；保留既有 Vite chunk size warning。
- 禁止文案扫描：对新增 `memoryCenter.ts`、`MemoryCenterView.tsx` 和 `App.tsx` 搜索 `已记住`、`系统已长期记住`、`候选已成为正式记忆`、`观察已成为正式记忆`、`worker 已收到记忆包`、`中间版本记忆层已完成`、`编辑正式记忆`、`删除正式记忆`、`归档正式记忆`，无命中。

未运行：

- 未运行 Rust 测试；本轮未修改 Rust 后端。
- 未运行真实 Tauri 窗口验收。

## UI Smoke

- 沙箱内启动 `npm run dev -- --port 4173` 被 `listen EPERM 127.0.0.1:4173` 拒绝。
- 按权限流程在沙箱外启动同一 dev server，Vite 返回 `Local: http://127.0.0.1:4173/`。
- `curl -sS -I http://127.0.0.1:4173/` 返回 `HTTP/1.1 200 OK`。
- 结束前已关闭 dev server。
- 当前线程工具发现未暴露 in-app Browser 导航 / 截图工具；项目 `node_modules` 下无 `playwright` / `@playwright` 包；未下载依赖。
- 因此真实窗口 / 截图验收未完成，不能声称 UI 截图验收通过。

## 边界确认

- 未写 `formal-memories.v1.json`。
- 未写 `memory-candidates.v1.json`。
- 未写 `observations.v1.json`。
- 未写 `memory-lint.v1.json`。
- 未写 `workflow-state.v0.json`。
- 未新增 Tauri 写命令。
- 未新增一级导航入口或右侧顶级入口。
- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未把 candidate、observation、knowledge hit、LLM summary 或 task package content 显示成正式记忆。

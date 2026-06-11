# Root Treatment / R4-A8 ProjectsView Component Extraction v1 Result

日期：2026-06-11

状态：已完成并通过复核线 `STATUS: CLEAR`。

任务包：`tasks/2026-06-11-root-treatment-r4-a8-projects-view-component-extraction-v1.md`

Evidence：`evidence/2026-06-11-root-treatment-r4-a8-projects-view-component-extraction-v1.md`

Planning baseline commit：`a8c5a725dea505e485a90e7d30b08f7650b31549`

Implementation commit：待回填。

Review result：`STATUS: CLEAR`，无 P0 / P1 / P2。

## 1. Result

R4-A8 实现侧已完成：`ProjectsView.tsx` 第一批外围组件已抽到 `src/views/projects/`，主文件行数从 6,069 降到 5,897。

抽取内容：

- `ProjectGallery.tsx`：项目入口方块列表。
- `ProjectReferencePanels.tsx`：交接 / 证据 / 权威面板、资源面板。

## 2. Files

改动文件：

- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectGallery.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectReferencePanels.tsx`
- `tasks/2026-06-11-root-treatment-r4-a8-projects-view-component-extraction-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a8-projects-view-component-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a8-projects-view-component-extraction-v1-result.md`

## 3. Verification

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，14 passed
- `npm run build`，仅既有 Vite chunk size warning
- `node scripts/harness/workbench-shape-gate.js --mode check`，pass，继承 warning `tauri_command_total_increased 97/96`
- `git diff --check`

## 4. Boundary

本轮没有改 UI 视觉风格、CSS、Rust、Tauri command、sidecar、DB、workflow state schema、App 数据加载路径；没有执行真实 Codex、没有发送 prompt、没有读写 `/Users/yoyi/.codex`、没有读取 secret/token/auth/full transcript、没有启动 Tauri/Browser/Chrome/Vite dev/截图工具。

## 5. Review

复核线已回交：

- `STATUS: CLEAR`
- P0：无
- P1：无
- P2：无

复核线建议：

- 可以 checkpoint。
- 主管线继续补齐 evidence / handoff / checkpoint 文档回填。

## 6. Cannot Claim

不能声明：

- R4 完成。
- 项目页 UI 已重做或视觉已验收。
- 页面真实数据来源已迁移。
- `query_workbench_page_read_model` 已被页面真实消费。
- `WorkbenchSnapshot` 已废弃。
- 真实 Tauri / 截图验收完成。
- R3 Level B 或多 agent 并行真实执行已解锁。

# Root Treatment / R4-A8 ProjectsView Component Extraction v1 Evidence

日期：2026-06-11

状态：已完成并通过复核线 `STATUS: CLEAR`。

任务包：`tasks/2026-06-11-root-treatment-r4-a8-projects-view-component-extraction-v1.md`

Planning baseline commit：`a8c5a725dea505e485a90e7d30b08f7650b31549`

Implementation commit：待回填。

Review result：`STATUS: CLEAR`，无 P0 / P1 / P2。

## 1. Scope

R4-A8 只实现 `ProjectsView.tsx` 第一批低风险组件抽取。

本轮接受范围：

- 新增 `prototypes/productized-desktop-shell/src/views/projects/ProjectGallery.tsx`。
- 新增 `prototypes/productized-desktop-shell/src/views/projects/ProjectReferencePanels.tsx`。
- 从 `ProjectsView.tsx` 迁移项目入口方块列表、项目资料索引、项目资源面板。
- 保持 JSX、className、文案和事件语义不变。
- `ProjectsView.tsx` 行数从 6,069 下降到 5,897。

本轮不接受范围：

- 不接受为 R4 完成。
- 不接受为项目页 UI 重做、布局重做或视觉验收。
- 不接受为页面真实数据来源迁移。
- 不接受为 `query_workbench_page_read_model` 被页面真实消费。
- 不接受为 `WorkbenchSnapshot` / `load_workbench_snapshot` 废弃。
- 不接受为真实 Tauri / 截图验收完成。
- 不接受为 R3 Level B、真实执行、多 agent 并行真实执行解锁。

## 2. Changed Files

- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectGallery.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectReferencePanels.tsx`
- `tasks/2026-06-11-root-treatment-r4-a8-projects-view-component-extraction-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a8-projects-view-component-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a8-projects-view-component-extraction-v1-result.md`

## 3. Implementation Notes

抽取到 `ProjectGallery.tsx` 的内容：

- `ProjectGallery`
- `projectInitials`

抽取到 `ProjectReferencePanels.tsx` 的内容：

- `ProjectHandoffEvidencePanel`
- `ProjectResourcesPanel`
- `ProjectFileList`

主管线没有改 CSS、布局 class、页面文案或项目页数据来源。

## 4. Shape Metrics

Baseline：

- `ProjectsView.tsx`：6,069 行
- `ProjectGallery.tsx`：不存在
- `ProjectReferencePanels.tsx`：不存在

After implementation：

- `ProjectsView.tsx`：5,897 行
- `ProjectGallery.tsx`：81 行
- `ProjectReferencePanels.tsx`：97 行

Shape gate：

- `Status: pass`
- `Errors: 0`
- `Warnings: 1`
- 继承 warning：`tauri_command_total_increased 97/96`
- `ProjectsView.tsx: 5897/6069 (decreased)`

## 5. Verification

已运行并通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
  - `offline interaction tests passed: 14`
  - `r4 page read model settings test passed`
  - `r4 page read model query contract test passed`
  - `r4 page selectors test passed`
- `npm run build`
  - 通过，保留既有 Vite chunk size warning。
- `node scripts/harness/workbench-shape-gate.js --mode check`
  - `Status: pass`
  - `Errors: 0`
  - `Warnings: 1`
- `git diff --check`

未运行 Rust：

- 本切片只改前端组件抽取和文档，未改 Rust、Tauri command、sidecar、DB migration 或 workflow state schema。

## 6. Boundary Confirmation

本轮没有：

- 改 UI 视觉风格、CSS、导航入口、布局意图或页面文案。
- 抽取或修改工作流执行、权限、真实 runner、自动编排、记忆写入等高风险区块。
- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 新增 Tauri command、sidecar、DB migration、store kind 或 workflow state 顶层结构。
- 改 `App.tsx` 数据加载路径。
- 把 `query_workbench_page_read_model` 接成页面真实数据源。
- 解冻 Stage L / Stage K / backlog 功能。

## 7. Review Result

复核线回交：

- `STATUS: CLEAR`
- P0：无
- P1：无
- P2：无

复核证据摘要：

- 复核线确认 `ProjectsView.tsx` 只新增抽取组件 import 并在原位置消费组件。
- 复核线确认 `ProjectGallery.tsx` 和 `ProjectReferencePanels.tsx` 只渲染项目入口方块和资料 / 资源面板，没有额外业务逻辑。
- 复核线确认关键 `className` 和文本骨架保持原样。
- 复核线确认未看到 Rust、Tauri command、sidecar、DB、workflow state schema、`App` 数据加载路径相关文件变动。
- 复核线确认新抽取文件没有引入真实执行调用，未见 `invoke(...)`、`query_workbench_page_read_model`、`codex exec` 一类路径。

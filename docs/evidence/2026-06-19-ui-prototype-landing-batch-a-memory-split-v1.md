# UI 原型落地 · 批 A 续做 + 记忆页拆瘦 evidence v1

日期：2026-06-19

关联计划：
- `docs/plans/2026-06-18-ui-prototype-to-frontend-landing-plan-v1.md`
- 延续 evidence：`docs/evidence/2026-06-19-ui-prototype-landing-batch-a-shell-split-v1.md`

## 本轮目标

继续按 2026-06-19 修订后的 UI 原型落地计划推进：

1. 批 A 叶子项继续收口：去掉非知识库页面的可见大标题栏，保留测试 / 辅助技术锚点。
2. 继续拆瘦巨石视图：先拆 `MemoryCenterView.tsx` 的纯展示区域，不改生命周期动作和确认边界。
3. 不碰智能体页、不碰知识库全页方向、不做批 C/D 的结构重构。

## 已改代码

### 批 A 叶子项

- `src/views/ProjectsView.tsx`
  - 空项目态的 `pg-head` 改为 `sr-only` 锚点。
- `src/views/projects/ProjectGallery.tsx`
  - 项目方块入口的可见 `pg-head` 改为 `sr-only` 锚点。
- `src/views/RunningWorkflowsView.tsx`
  - 运行中工作流页的可见 `pg-head` 改为 `sr-only` 锚点。
- `src/views/SettingsView.tsx`
  - 设置页的可见 `pg-head` 改为 `sr-only` 锚点。
- `src/views/MemoryCenterView.tsx`
  - 记忆页的可见 `pg-head` 改为 `sr-only` 锚点。
- `src/components/SourceStylePlaceholder.tsx`
  - 四个源稿风格占位入口去掉可见 `pg-head`，保留 `sr-only` 边界说明。
- `src/views/projects/ProjectOverviewPanels.tsx`
  - 项目总览卡 eyebrow 从“当前工作流”改为“工作流”。

说明：`KnowledgeBaseView.tsx` 仍保留 `pg-head`，因为本计划 2026-06-19 修订明确“知识库整页本期不动”。

### 记忆页拆瘦

- 新增 `src/views/memory/MemoryDetailPanels.tsx`
  - 抽出 `FormalMemoryDetail`
  - 抽出 `CandidateMemoryDetail`
  - 抽出 `operationLabel`
  - 抽出 `sourceText`
  - 保留正式记忆生命周期按钮分组：`编辑提案` / `版本` / `秘书建议` / `更多`
- 新增 `src/views/memory/MemoryWorkbenchSummary.tsx`
  - 抽出 `MemoryCenterStats`
  - 抽出 `MemoryWorkbenchSummary`
  - 抽出 `StatCell` / `MiniMetric` / `memoryWorkbenchActionLabel`

行数变化：
- `src/views/MemoryCenterView.tsx`：约 1391 行 -> 1151 行
- 新增 `src/views/memory/MemoryDetailPanels.tsx`：150 行
- 新增 `src/views/memory/MemoryWorkbenchSummary.tsx`：110 行
- `src/App.tsx` 保持上轮拆分后的 903 行

## 验证

在 `prototypes/productized-desktop-shell` 下执行：

- `npm run typecheck`：通过
- `npm run test:offline-interaction`：通过，15 个 offline interaction tests + r4 page read model / selectors tests 通过
- `git diff --check`：通过
- `curl -sS -I http://127.0.0.1:5173/`：返回 `HTTP/1.1 200 OK`

## 未做 / 暂停

- 未碰智能体页：该页归 `2026-06-19-conversation-shell-codex-layout-refactor-plan-v1.md`。
- 未碰知识库全页方向：计划明确本期不动。
- 未做首页三段式、项目页 3 格状态条、右栏想法箱和右栏按项目分组：这些属于批 B/C，需等进一步拆瘦后再动。
- 未拆 `styles.css`：仍是巨石，后续应单独拆样式层，避免和行为改动混在一起。

## 风险

- 这轮用 `sr-only` 保留旧文本锚点，确保测试和辅助技术可读；视觉上已去掉大标题栏，但文本仍会被静态测试抽取到。
- `MemoryCenterView.tsx` 只拆纯展示组件，未改变读模型、权限弹层、生命周期 preview / request action 链路。

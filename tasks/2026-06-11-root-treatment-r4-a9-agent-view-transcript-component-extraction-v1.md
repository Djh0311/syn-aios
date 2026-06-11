# Root Treatment / R4-A9 AgentView Transcript Component Extraction v1

日期：2026-06-11

状态：已完成并通过复核线 `STATUS: CLEAR`。本文是 Root Treatment / Stage R 的 R4-A9 任务包；R4-A8 已完成并通过复核线 `STATUS: CLEAR`。R4-A9 只接受为 `AgentView.tsx` 第一批低风险 transcript 展示组件抽取；不接受为 R4 完成、智能体页 UI 重做、真实执行控制台改造、页面真实数据来源迁移、真实 Tauri / 截图验收、R3 Level B 或多 agent 并行真实执行解锁。

Planning baseline commit：`a863c61ac272dce9a28baf00a18c9694f9aba422`

Implementation commit：`886d3cf9bf7bb70fb37bedfe6fc7d6ec6be3f347`。

Review result：`STATUS: CLEAR`，无 P0 / P1 / P2。

Checkpoint commit：`efe798b8007ff90e77cb8a67ba3649083eed3dc7`。

## 0. 全局主管理解

已知事实：

- 官方计划 R4-4 是 `AgentView.tsx` 拆分，验收目标是 agent page components、主文件行数下降、对话工作区行为不变。
- 当前 shape gate 通过；`AgentView.tsx` 为 3,360 行，是 ratchet debt 文件。
- `TranscriptTimeline` / `ChatTranscript` / `ChatBubble` / code block / warning strip 属于会话正文展示层，和真实执行 prepare / Phase A / Phase B 控制入口解耦。

核心判断：

```text
R4-A9 先抽取 transcript 展示组件到 `src/views/agent/TranscriptViews.tsx`，保持 JSX、className、文案和行为不变。
```

## 1. Execution Mode

Execution Mode：Supervisor-led implementation with read-only review。

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、证据、handoff 和 checkpoint。
- 复核线复用既有线程，只读审查，不改代码。
- 本切片集中在 transcript 展示组件抽取，不新增开发线程，避免上下文维护成本。

## 2. Read / Write Scope

Allowed read paths：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/*.tsx`
- `prototypes/productized-desktop-shell/src/lib/conversationTurns.ts`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- shape gate output and package scripts as needed.

Allowed write paths：

- `tasks/2026-06-11-root-treatment-r4-a9-agent-view-transcript-component-extraction-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a9-agent-view-transcript-component-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a9-agent-view-transcript-component-extraction-v1-result.md`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/TranscriptViews.tsx`

## 3. Forbidden

R4-A9 禁止：

- 不改 UI 视觉风格、布局意图、CSS、导航入口或页面文案。
- 不抽取或修改 K2/J1 控制入口、真实执行 prepare / confirm / Phase A / Phase B、权限、真实 runner、自动编排、记忆写入等高风险区块。
- 不新增 Tauri command。
- 不新增 sidecar、DB migration、store kind 或 workflow state 顶层结构。
- 不改 `App.tsx` 数据加载路径。
- 不切 `query_workbench_page_read_model` 为页面真实数据源。
- 不废弃或弱化 `WorkbenchSnapshot` / `load_workbench_snapshot`。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 不启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 不解冻 Stage L / Stage K / backlog 功能。

## 4. Implementation Tasks

1. 新增 `src/views/agent/TranscriptViews.tsx`，迁移 `TranscriptTimeline`、`ChatTranscript`、`ChatBubble`、message body / code block、`WarningStrip`、`readbackCountLabel` 和 transcript event helpers。
2. `AgentView.tsx` 改为 import 抽取组件和 helper，移除原内联 transcript renderer。
3. 保持原 JSX、className、文案和事件语义不变。
4. 运行 typecheck / 离线交互 / build / shape gate / diff check。
5. 写 evidence / handoff，记录 `AgentView.tsx` 行数下降、验证结果和禁止声明。

## 5. Acceptance Criteria

R4-A9 可接受条件：

- `AgentView.tsx` 行数下降，shape gate 通过。
- 新文件低于 2,000 行。
- 会话正文、代码块、内部事件详情、warning strip、readback count 展示行为不变。
- `npm run typecheck`、`npm run test:offline-interaction`、`npm run build` 通过。
- 不新增产品能力、不改视觉、不改 Rust、不改数据读取路径、不改真实执行路径。

## 6. Verification Plan

必须运行：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

如未运行 Rust 说明原因；本切片默认不改 Rust。

## 7. Review Plan

实现后复用既有复核线做只读审查。

复核重点：

- `AgentView.tsx` 是否真实下降。
- 抽取组件是否保持原 JSX / 文案 / className / 行为。
- 是否没有改 CSS、Rust、Tauri command、sidecar、DB、workflow state schema、真实执行路径。
- 是否没有改真实执行 prepare / confirm / Phase A / Phase B 控制入口语义。

## 8. 禁止声明

R4-A9 禁止声明：

- R4 完成。
- 智能体页 UI 已重做或视觉已验收。
- 自由 Codex 控制台能力已改变。
- 页面真实数据来源已迁移。
- `query_workbench_page_read_model` 已被页面真实消费。
- `WorkbenchSnapshot` 已废弃。
- 真实 Tauri / 截图验收完成。
- R3 Level B 已执行。
- 多 agent 并行真实执行已解锁。

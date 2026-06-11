# Root Treatment / R4-A9 AgentView Transcript Component Extraction v1 Evidence

日期：2026-06-11

状态：已完成并通过复核线 `STATUS: CLEAR`。

任务包：`tasks/2026-06-11-root-treatment-r4-a9-agent-view-transcript-component-extraction-v1.md`

Planning baseline commit：`a863c61ac272dce9a28baf00a18c9694f9aba422`

Implementation commit：`886d3cf9bf7bb70fb37bedfe6fc7d6ec6be3f347`。

Review result：`STATUS: CLEAR`，无 P0 / P1 / P2。

Checkpoint commit：`efe798b8007ff90e77cb8a67ba3649083eed3dc7`。

## 1. Scope

R4-A9 只实现 `AgentView.tsx` 第一批低风险 transcript 展示组件抽取。

本轮接受范围：

- 新增 `prototypes/productized-desktop-shell/src/views/agent/TranscriptViews.tsx`。
- 从 `AgentView.tsx` 迁移 `TranscriptTimeline`、`ChatTranscript`、`ChatBubble`、message body / code block、transcript event card、`WarningStrip`、`readbackCountLabel`。
- `AgentView.tsx` 保留 `ChatTranscript` / `TranscriptTimeline` re-export，保持既有测试和外部 import 兼容。
- 保持 JSX、className、文案和事件语义不变。
- `AgentView.tsx` 行数从 3,360 下降到 3,118。

本轮不接受范围：

- 不接受为 R4 完成。
- 不接受为智能体页 UI 重做、布局重做或视觉验收。
- 不接受为自由 Codex 控制台能力改变。
- 不接受为页面真实数据来源迁移。
- 不接受为 `query_workbench_page_read_model` 被页面真实消费。
- 不接受为 `WorkbenchSnapshot` / `load_workbench_snapshot` 废弃。
- 不接受为真实 Tauri / 截图验收完成。
- 不接受为 R3 Level B、真实执行、多 agent 并行真实执行解锁。

## 2. Changed Files

- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/TranscriptViews.tsx`
- `tasks/2026-06-11-root-treatment-r4-a9-agent-view-transcript-component-extraction-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a9-agent-view-transcript-component-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a9-agent-view-transcript-component-extraction-v1-result.md`

## 3. Implementation Notes

抽取到 `TranscriptViews.tsx` 的内容：

- `TranscriptTimeline`
- `ChatTranscript`
- `ChatBubble`
- `MessageBody`
- `CodeBlock`
- `TranscriptEventCard`
- `WarningStrip`
- `readbackCountLabel`
- transcript event label / tone / value preview helpers

主管线没有改 CSS、布局 class、页面文案、真实执行 prepare / confirm / Phase A / Phase B 逻辑或智能体页数据来源。

## 4. Shape Metrics

Baseline：

- `AgentView.tsx`：3,360 行
- `TranscriptViews.tsx`：不存在

After implementation：

- `AgentView.tsx`：3,118 行
- `TranscriptViews.tsx`：246 行

Shape gate：

- `Status: pass`
- `Errors: 0`
- `Warnings: 1`
- 继承 warning：`tauri_command_total_increased 97/96`
- `AgentView.tsx: 3118/3365 (decreased)`

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
- 抽取或修改 K2/J1 控制入口、真实执行 prepare / confirm / Phase A / Phase B、权限、真实 runner、自动编排、记忆写入等高风险区块。
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

- 复核线确认 `AgentView.tsx` 只把内联 transcript 组件替换成外部 import，并在原位置继续渲染。
- 复核线确认 `ChatTranscript` / `TranscriptTimeline` re-export 已保留。
- 复核线确认 `TranscriptViews.tsx` 只承载 transcript 展示组件和展示 helper，没有引入真实执行入口。
- 复核线确认关键 className 和展示结构保持原样。
- 复核线确认真实执行 prepare / confirm / Phase A / Phase B 控制入口没有被改语义。
- 复核线确认未看到 Rust、Tauri command、sidecar、DB、workflow state schema、`App` 数据加载相关文件变动。

# Root Treatment / R4-A5 Running Memory Selector Domain And Page Consumption Evidence v1

日期：2026-06-11

状态：已完成并通过复核线 `STATUS: CLEAR`。本文记录 R4-A5 Running Workflows / Memory Center 首批页面 selector 分域和页面最小消费。R4-A5 只接受为前端纯 selector 和两个页面首屏摘要消费完成；不接受为 R4 完成、页面真实数据来源迁移、`query_workbench_page_read_model` 真实消费、UI 重做、真实 Tauri / 截图验收、R3 Level B 或多 agent 并行真实执行解锁。

Planning baseline commit：`930bde34bffe551fb7ec7840313576e1f3ad9493`
Task package commit：`e44cdf4f18e82b25b08d9bbb1d34f33eb4641008`
Implementation commit：`955783f4629176d930fd0b2fb1d881aa6a289c0d`
Review result：`STATUS: CLEAR`，复核线 `019eb51c-61fe-7fc3-8973-b22a4ce58911`，P0/P1/P2 均无。
Checkpoint commit：待回填。

## 1. Scope

本轮改动限定在：

- `prototypes/productized-desktop-shell/src/lib/pageSelectors.ts`
- `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`
- `prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx`
- `prototypes/productized-desktop-shell/tests/r4-page-selectors.test.ts`

未改动：

- Rust / Tauri command / runner。
- sidecar schema、DB migration、workflow state 顶层结构。
- CSS、视觉风格、布局结构或交互入口。
- `App.tsx` 数据加载路径。
- `query_workbench_page_read_model` 页面真实消费路径。

## 2. Implementation Summary

新增前端纯 selector：

- `deriveRunningWorkflowsPageReadModel`
- `deriveRunningWorkflowsPageReadModelFromParts`
- `deriveMemoryCenterPageReadModel`
- `deriveMemoryCenterPageReadModelFromParts`

Running selector 覆盖：

- 工作流数量、运行关注、等待权限、读回异常。
- run queue 数量、等待确认、阻断、失败控制、重复阻断、捕获补偿。
- operation control 计数。
- 记忆待处理、捕获数量、候选 / 正式化待处理。
- 统一执行命令和自动编排摘要计数。
- readback unavailable / failed / timed out 的 unknown result count，保持未知，不转成 0。

Memory selector 覆盖：

- 正式记忆、活跃记忆、候选、观察来源。
- lint open/blocking。
- maintenance blocking / needs review / info。
- mature pattern candidate / confirmation。
- task package snapshot。
- memory workbench action/capture/observation/candidate/formalization/compensation counts。

页面消费：

- `RunningWorkflowsView.tsx` 的页头和首屏 summary tiles 改为读取 `pageReadModel`；详情列表仍使用既有 `runQueue`、workflow、runtime attention、automation 和 product command read model。
- `MemoryCenterView.tsx` 的页头、stat strip 和 memory workbench top numbers 改为读取 `pageReadModel`；详情、按钮和高级治理模块仍使用既有 `summary`。

## 3. Boundary Confirmation

本轮没有：

- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 新增 Tauri command、sidecar、DB migration、store kind 或 workflow state 顶层结构。
- 解冻 Stage L / Stage K / backlog 功能。

## 4. Verification

已运行并通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
  - 输出：`offline interaction tests passed: 14`
  - 包含：`r4 page selectors test passed`
- `npm run build`
  - 通过；保留既有 Vite chunk size warning。
- `node scripts/harness/workbench-shape-gate.js --mode check`
  - 通过。
  - `Errors: 0`
  - `Warnings: 1`
  - 继承 warning：`tauri_command_total_increased`，当前 97 / baseline 96；本轮未改 Rust / Tauri command。
- `git diff --check`

过程说明：

- shape gate 第一次误在 `prototypes/productized-desktop-shell` 子目录运行相对路径，报 `MODULE_NOT_FOUND`。
- 随后在 `/Users/yoyi/workspace/product-line` 根目录重跑同一脚本并通过。
- 该失败是工作目录错误，不是产品代码或 gate 失败。

未运行：

- Rust 测试。本轮未改 Rust、Tauri command、runner、store 或 DB schema。
- 真实 Tauri / 截图验收。R4-A5 任务包禁止启动 Tauri / Browser / Chrome / Vite dev / 截图工具。

## 5. Test Coverage Added

`r4-page-selectors.test.ts` 新增覆盖：

- Running / Memory selector schema 稳定。
- `source_boundary.generated_from = workbench_snapshot_selector`。
- `workbench_snapshot_active = true`。
- `page_ui_migrated = false`。
- `tauri_command_consumed = false`。
- `writes_stores = false`。
- Running selector 统计 workflow、focus、waiting permission、readback issue、unknown readback result、memory confirmations、pending candidates、product commands、automation units。
- Memory selector 统计 formal/candidate/observation，并确认 candidate / observation 不算 formal memory。
- selector 输出不包含 `raw transcript`、`full transcript`、`secret`、`token`、`prompt_body`。

## 6. Acceptance Statement

R4-A5 当前可提交给复核线审查为：

```text
Running Workflows / Memory Center 首批前端纯 selector 分域和页面最小消费已实现并通过本地验证；页面仍以 WorkbenchSnapshot 派生数据为事实来源，不声明真实数据源迁移，不声明 R4 完成。
```

## 7. Deferred / Not Claimed

仍未完成且不得由本轮冒领：

- R4 完成。
- `query_workbench_page_read_model` 页面真实消费。
- `WorkbenchSnapshot` / `load_workbench_snapshot` 废弃。
- Running / Memory 页面完整拆分。
- UI 重做或视觉验收。
- 真实 Tauri / 截图验收。
- R3 Level B。
- 多 agent 并行真实执行。

## 8. Review Result

复核线：`019eb51c-61fe-7fc3-8973-b22a4ce58911`

结论：`STATUS: CLEAR`

发现：

- P0：无。
- P1：无。
- P2：无。

复核确认：

- Running / Memory 页面确实消费新增 selector 输出。
- selector 保持前端纯函数和 source boundary。
- 未新增 Tauri command、sidecar、DB migration、真实执行路径或 workflow state 顶层结构。
- 未改 CSS、视觉风格或页面布局结构。
- readback unavailable / failed / timed out 的 unknown/null 没有被显示或统计成真实 0 条结果。
- candidate / observation 未被当作 formal memory。

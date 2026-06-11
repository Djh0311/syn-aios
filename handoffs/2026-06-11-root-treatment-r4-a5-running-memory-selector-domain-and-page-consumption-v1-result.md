# Root Treatment / R4-A5 Running Memory Selector Domain And Page Consumption Result v1

日期：2026-06-11

结论：已完成并通过复核线 `STATUS: CLEAR`。R4-A5 已实现 Running Workflows / Memory Center 首批页面 selector 分域和页面最小消费，implementation commit 为 `955783f4629176d930fd0b2fb1d881aa6a289c0d`，checkpoint commit 为 `8fb7fa360d6f5074b77728f493fa73eaf68363c3`。

## 必读文件

- `tasks/2026-06-11-root-treatment-r4-a5-running-memory-selector-domain-and-page-consumption-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a5-running-memory-selector-domain-and-page-consumption-v1.md`
- `prototypes/productized-desktop-shell/src/lib/pageSelectors.ts`
- `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`
- `prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx`
- `prototypes/productized-desktop-shell/tests/r4-page-selectors.test.ts`

## 改动摘要

- 新增 `deriveRunningWorkflowsPageReadModel` / `deriveRunningWorkflowsPageReadModelFromParts`。
- 新增 `deriveMemoryCenterPageReadModel` / `deriveMemoryCenterPageReadModelFromParts`。
- Running 页页头和首屏 summary tiles 改为消费 `pageReadModel`。
- Memory 页页头、stat strip 和 memory workbench top numbers 改为消费 `pageReadModel`。
- 详情列表、操作按钮、高级治理模块仍保留原有局部 read model，不做视觉或结构重做。
- `r4-page-selectors.test.ts` 增加 Running / Memory selector coverage。

## 验证结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，14 passed。
- `npm run build`，通过，保留既有 Vite chunk warning。
- `node scripts/harness/workbench-shape-gate.js --mode check`，通过，`Errors: 0`，继承 warning `tauri_command_total_increased` 97 / 96。
- `git diff --check`

未运行：

- Rust 测试，因为本轮未改 Rust / Tauri / runner / store / DB schema。
- 真实 Tauri / 截图验收，因为 R4-A5 禁止启动 Tauri / Browser / Chrome / Vite dev / screenshot。

## 边界确认

本轮没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript，没有新增 sidecar / DB migration / Tauri command / workflow state 顶层结构，没有解冻 Stage L / Stage K / backlog。

## 复核结果

复核线：`019eb51c-61fe-7fc3-8973-b22a4ce58911`

结论：`STATUS: CLEAR`

P0 / P1 / P2：均无。

复核确认：没有越界行为，没有冒领声明；R4-A5 可按当前边界通过只读复核。

## 复核请求原始重点

请复核线只读检查：

- Running / Memory 页面是否确实消费新增 selector。
- selector 是否保持前端纯函数和 source boundary。
- 是否没有新增 Tauri command、sidecar、DB migration、真实执行路径。
- 是否没有视觉 / CSS / 布局重做。
- readback unavailable / failed / timed out 是否仍保持未知/null，不显示成 0。
- candidate / observation / knowledge hit 是否仍不被当成 formal memory。

## 不能声明

不能声明 R4 完成、页面真实数据来源已迁移、`query_workbench_page_read_model` 被真实消费、`WorkbenchSnapshot` 已废弃、Running / Memory 页面已完整拆分、UI 重做完成、真实 Tauri / 截图验收完成、R3 Level B 已执行或多 agent 并行真实执行已解锁。

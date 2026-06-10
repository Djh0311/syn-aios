# Stage J / J4 Run Queue, Failure Control, And User Confirmation Queue v1 Handoff

日期：2026-06-09

状态：已完成，结论为 `accepted_with_deferred_items`。复核线最终确认无 P0/P1/P2。

## 1. 本轮做了什么

J4 把已有 Product Command、自动编排 run unit、runtime attention、workflow fact、J3 memory capture event 和已确认记忆候选汇总成稳定的前端派生读模型：

- `RunQueueItem[]`
- `UserConfirmationQueueItem[]`
- `FailureControlSummary[]`

运行中工作流页现在以“运行队列 / 待确认 / 失败控制”为主视角；右侧 `运行中 / 待办` 使用同一套摘要；秘书只解释 J4 队列风险和查看建议，不生成执行动作。

## 2. 代码落点

- `src/lib/runQueue.ts`
- `src/views/RunningWorkflowsView.tsx`
- `src/components/RightDetailPanel.tsx`
- `src/lib/secretaryReadModel.ts`
- `src/App.tsx`
- `tests/offline-permission-dialog.test.tsx`

## 3. 验证结果

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`：`offline interaction tests passed: 14`
- `npm run build`：通过，仅既有 Vite chunk warning。

未跑 Rust 测试：本轮未改 Rust / Tauri 后端。

## 4. 边界确认

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未新增 Tauri command、后端 sidecar、DB migration 或 workflow state 顶层结构。
- retry / stop / restart / resume 默认只进入确认事项，不调用 runner。
- 已确认候选只进入正式化确认事项，不自动写 FormalMemory。
- 秘书侧已接入 `memoryCaptureStore`，capture compensation 计数可进入秘书风险摘要。
- workflow / runtime attention 的 capture refs 按精确 node 回链，避免把同一 workflow 下无关 capture refs 铺到所有运行项。
- `readback_result_count=null` 继续显示为“未知 / 不可用”，不显示为 0。
- capture / observation / candidate 不冒充 FormalMemory。

## 5. 不能声明

- 不能声明 Stage J 完成。
- 不能声明任意项目无限制自由执行完成。
- 不能声明自动 retry / stop / restart 已无条件可用。
- 不能声明 planned adapters 真实接入。
- 不能声明 provider credential / model verification 完成。
- 不能声明 FormalMemory 自动写入完成。
- 不能声明真实 Tauri / 截图验收完成。

## 6. 下一步

进入 J5：UI 信息层级和真实 Tauri 产品验收。J5 需要保持普通用户视图清晰，把开发者内容继续收进设置 / 开发者区，并对 J1-J4 关键路径做真实 Tauri 手动验收；J4 不等于 Stage J 完成。

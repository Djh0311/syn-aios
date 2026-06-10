# Stage K K5 Running Todo Failure Recovery And Operation Control UX v1 Handoff

日期：2026-06-10

结论：`accepted_non_real_productization_slice`

## 已完成

- K5 任务包已收口为已完成。
- `RunQueueReadModel` 新增 `operation_control_summary` 前端只读派生摘要。
- 运行中工作流页新增“操作控制 / 恢复建议”普通层，展示重试提案、停止请求、重启准备、恢复准备、读回异常、重复阻断、边界阻断和过期清理。
- retry / stop / restart / resume 继续显示为需确认、后续任务或只读 readiness，没有新增真实执行按钮。
- readback unavailable / failed / timed_out / null result count 继续显示为未知 / 不可用，不显示为 0。
- 秘书只读模型改为“运行队列”产品口径，仍不生成 retry / stop / restart / resume / send action proposal。
- 离线测试新增 K5 断言，覆盖 operation summary、readback null、执行边界和误导文案黑名单。

## 验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，14 passed。
- `npm run build`：通过，仅既有 Vite chunk size warning。
- `node scripts/harness/stage-k-architecture-gate.js --target /Users/yoyi/workspace/product-line --strict`：通过，0 error / 0 warning。
- 普通 UI 误导文案扫描无命中：`J4 队列`、`stale cleanup`、`真实 .codex`、`codex exec resume`、`runner_call`、`自动重试中`、`结果数：0`、`已停止`、`已重启`、`已恢复`、`已 resume`。

## 复核结论

复核线最终结论为通过，允许主管线将 K5 本轮收口为 `accepted_non_real_productization_slice`。

无 P0/P1/P2。复核线未发现真实 Codex 执行、prompt 发送、`.codex` 读写、secret/full transcript 读取、K3-B1/K3-B2 冻结突破，或 retry / stop / restart / resume 被 UI / 秘书说成已实现。

## 边界

- 本轮不接受为 K5 全量完成或 Stage K 完成。
- 本轮不接受为 K3-B1 retry 成功或 K3-B2 可开始。
- 本轮不接受为真实 retry / stop / restart / resume 已实现。
- 本轮不接受为真实 Codex 已被再次执行。
- 本轮不接受为自动清理真实 Codex 本地状态完成。
- 后续 K6 才能做真实 Tauri dogfood 和阶段验收收口；K6 不能继承 K5 为真实执行授权。

## 下一步建议

进入 K6：真实 Tauri dogfood 和验收收口。

K6 可以围绕 `mario test`、工作台自身项目和新隔离测试项目做真实桌面验收、截图 / 手动检查和 Stage K 完成项 / deferred 项冻结。若要恢复 K3-B1/K3-B2 真实执行线，必须另行满足授权和安全审查，不能由 K5 或 K6 文档自动继承。

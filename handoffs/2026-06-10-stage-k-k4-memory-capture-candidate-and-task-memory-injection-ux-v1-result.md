# Stage K K4 Memory Capture Candidate And Task Memory Injection UX v1 Handoff

日期：2026-06-10

结论：`accepted_non_real_productization_slice`

## 已完成

- K4 任务包已收口为已完成。
- 新增 `MemoryWorkbenchSummary` / `memory_workbench_summary` 前端只读派生摘要。
- 记忆页普通层新增“捕获 / 候选 / 任务记忆包”摘要，展示捕获、观察、候选、待正式化、补证、任务包入选 / 排除 / 待审材料和行动项。
- 运行中工作流页新增记忆待处理摘要，并明确候选确认、正式化或捕获补证都不会自动写正式记忆。
- 离线测试新增 K4 断言，覆盖候选 / 观察边界、任务记忆包待审材料和误导文案。
- 复核线唯一 P2 已修补：`member refs / signal refs` 改为“关联成员 / 识别信号”。

## 验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，14 passed。
- `npm run build`：通过，仅既有 Vite chunk size warning。
- `node scripts/harness/stage-k-architecture-gate.js --target /Users/yoyi/workspace/product-line --strict`：通过，0 error / 0 warning。

## 复核结论

复核线结论为带 P2 通过，允许主管线将 K4 本轮收口为 `accepted_non_real_productization_slice`。

无 P0/P1。复核线未发现真实 Codex 执行、prompt 发送、`.codex` 读写、secret/full transcript/rollout 读取、K3-B1/K3-B2 冻结突破，或 observation/candidate/capture 被冒充为 FormalMemory。

## 边界

- 本轮不接受为 K4 全量完成或 Stage K 完成。
- 本轮不接受为 K3-B1 retry 成功或 K3-B2 可开始。
- 本轮不接受为真实 Codex 执行后自动生成 observation / candidate 的真实执行验收完成。
- 本轮不接受为用户确认候选后写 FormalMemory 的新能力完成。
- 后续 K5 只能先做运行中 / 待办 / 失败恢复和操作控制的非真实产品化切片；真实 retry / stop / restart 仍必须单独授权并通过审查。

## 下一步建议

进入 K5：运行中、待办、失败恢复和操作控制。

K5 默认不执行真实 Codex，不发送 prompt，不读写 `/Users/yoyi/.codex`；先把 readback failed / unavailable、timed out、duplicate blocked、stale active attempt、retry proposal、stop / restart readiness 和用户确认入口整理成可读产品层级。

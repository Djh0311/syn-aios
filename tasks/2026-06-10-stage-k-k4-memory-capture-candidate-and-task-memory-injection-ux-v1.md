# Stage K / K4 Memory Capture, Candidate, And Task Memory Injection UX v1

日期：2026-06-10

状态：已完成。

结论：`accepted_non_real_productization_slice`。

本任务包用于在 K3-B1 retry 被安全审查拒绝、K3-B2 继续冻结的前提下，推进 Stage K 原目标中的“记忆层记录 / 分析 / 候选化”产品化体验。K4 本轮只做非真实 Codex 产品化切片：把已有 `MemoryCaptureEvent`、Observation、MemoryCandidate、FormalMemory、TaskMemoryPacket 和 run queue 的关系整理成用户可读读模型与 UI 层级。

本文不授权新的真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`，不读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout，不启动 K3-B2。

## 1. 当前事实

- Stage K architecture calibration v2 and gate 已完成，gate strict 通过，0 error / 0 warning。
- K3-B1 已执行但失败分类；retry 申请再次被安全审查拒绝。
- K3-B2 依赖 K3-B1 成功和复核，当前不得启动。
- J3 已有 memory capture bus，J4 已有 run queue 派生确认队列，M1-M13 已有正式记忆、候选、observation、任务记忆包和生命周期能力。
- 当前缺口不是重新造记忆层，而是让用户看懂：这次运行产生了什么、哪些候选需要确认、哪些链路需要补证、下次任务会注入哪些记忆。

## 2. 目标

K4 本轮交付：

1. 记忆中心普通层新增“记忆工作台摘要”，显示捕获、观察、候选、正式化待办、任务包注入和补证状态。
2. 捕获事件明确分类为：已形成观察、已形成候选、待补证、敏感阻断 / 仅审计。
3. 候选记忆明确分类为：待确认、已确认待正式化、已采纳、已拒绝 / 隔离 / 延后。
4. 任务记忆包预览明确显示 included / excluded / review materials，并强调只有正式记忆能进入 included。
5. 运行中工作流页能显示记忆待处理摘要：候选确认、正式化、捕获补证。
6. 普通 UI 不显示 raw sidecar、store revision、raw refs 长列表、阶段术语或开发者边界长文案。

## 3. 非目标

- 不执行真实 Codex。
- 不发送 prompt。
- 不做 K3-B1 retry。
- 不启动 K3-B2。
- 不自动写 FormalMemory。
- 不新增 FormalMemory schema。
- 不新增 provider credential store 或 model verification。
- 不接向量库 / 图数据库 / GraphRAG。
- 不做 Obsidian 原生同步。
- 不把普通浏览器 smoke 当真实 Tauri 验收。

## 4. UI 显示边界确认

本任务会改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端读模型摘要。
- [x] 改已有页面局部 UI。
- [x] 改离线 UI 测试。
- [ ] 新增普通主导航入口。

普通 UI 应显示：

- 本轮捕获多少条、观察多少条、候选多少条、正式记忆多少条。
- 候选待确认、已确认待正式化、捕获补证、任务包待审材料。
- “候选不是正式记忆”“观察不是正式记忆”“正式化仍需确认”。
- 任务记忆包入选 / 排除 / 待审查材料。
- 用户下一步：审查候选、确认正式化、补证、查看任务包注入预览。

普通 UI 不显示：

- raw JSON。
- sidecar 绝对路径。
- store revision。
- prompt body。
- full transcript。
- raw stdout / stderr。
- `/Users/yoyi/.codex` 内部路径内容。
- H/J/K/PCR 阶段术语作为用户操作文案。

## 5. 改动范围

允许改：

- `prototypes/productized-desktop-shell/src/lib/memoryCenter.ts`
- `prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx`
- `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

默认不改：

- Rust runner / Product Command 真实执行语义。
- `workflow-state.v0.json` 顶层结构。
- FormalMemory store schema。
- provider / credential / adapter 真实接入。
- Tauri command wrapper。

## 6. 验收

必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `node scripts/harness/stage-k-architecture-gate.js --target /Users/yoyi/workspace/product-line --strict`

扫描必须确认：

- 不出现“候选已成为正式记忆”“观察已成为正式记忆”“自动记住”等误导文案。
- K3-B1 / K3-B2 旧口径不被改写成已完成或可开始。
- 普通 UI 不暴露 raw sidecar / store revision / prompt body。

## 7. 接受口径

可接受为：

- K4 非真实 Codex 产品化切片完成。
- 记忆捕获、候选、正式化待办、任务包注入预览和补证状态在普通 UI 里可读。
- 不依赖 K3-B1 retry 或 K3-B2。

不接受为：

- 单次真实 Codex 操作后自动生成 observation / candidate 的真实执行验收完成。
- 工作流真实闭环后自动生成候选的真实执行验收完成。
- 用户确认候选后写 FormalMemory 的新能力完成。
- K4 全量完成。
- Stage K 完成。

## 8. 收口记录

本轮实际完成：

- 新增前端只读 `MemoryWorkbenchSummary` / `memory_workbench_summary`，只汇总已有 formal / capture / observation / candidate / lint / task package 读模型，不新增事实源。
- 记忆页普通层新增“捕获 / 候选 / 任务记忆包”摘要，显示捕获、观察、候选、待正式化、需补证、任务包入选 / 排除 / 待审材料和行动项。
- 运行中工作流页新增记忆待处理摘要，明确候选确认、正式化或捕获补证都不会自动写正式记忆。
- 离线测试新增 K4 断言，覆盖补证、待正式化、任务包待审材料、候选 / 观察边界。
- 复核线结论为带 P2 通过；唯一 P2 是普通 UI 中 `member refs / signal refs` 英文内部计数字段，已改为中文产品口径“关联成员 / 识别信号”。

记录：

- `evidence/2026-06-10-stage-k-k4-memory-capture-candidate-and-task-memory-injection-ux-v1.md`
- `handoffs/2026-06-10-stage-k-k4-memory-capture-candidate-and-task-memory-injection-ux-v1-result.md`

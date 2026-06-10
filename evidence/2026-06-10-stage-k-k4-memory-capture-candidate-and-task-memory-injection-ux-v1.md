# Stage K K4 Memory Capture Candidate And Task Memory Injection UX v1 Evidence

日期：2026-06-10

结论：`accepted_non_real_productization_slice`

## 背景

Stage K 原目标不变：自由操控 Codex + 自动化工作流 + 记忆层记录。当前 K3-B1 retry 已被安全审查再次拒绝，K3-B1 未完成，K3-B2 仍不得启动。

K4 本轮只推进不依赖真实 Codex 的记忆层产品化切片：把已有 `MemoryCaptureEvent`、Observation、MemoryCandidate、FormalMemory、TaskMemoryPacket、lint finding 和 run queue 摘要整理成用户可读的普通 UI 层级。

本轮不授权新的真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`，不读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout，不启动 K3-B2。

## 改动范围

- `tasks/2026-06-10-stage-k-k4-memory-capture-candidate-and-task-memory-injection-ux-v1.md`
- `prototypes/productized-desktop-shell/src/lib/memoryCenter.ts`
- `prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx`
- `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 已完成

1. 新增 `MemoryWorkbenchSummary` / `memory_workbench_summary` 前端只读派生摘要，只汇总已有 formal / capture / observation / candidate / lint / task package，不创建新事实源。
2. 记忆页普通层新增“捕获 / 候选 / 任务记忆包”摘要，显示捕获、观察、候选、待正式化、需补证、任务包入选 / 排除 / 待审材料和行动项。
3. 运行中工作流页新增记忆待处理摘要，提示候选确认、正式化或捕获补证都不会自动写正式记忆。
4. 离线测试新增 K4 断言，覆盖补证、待正式化、任务包待审材料、候选 / 观察边界。
5. 复核线发现的 P2 文案债已修补：`member refs / signal refs` 改为中文产品口径“关联成员 / 识别信号”。

## 关键边界

- Observation 不是 FormalMemory。
- Candidate 不是 FormalMemory。
- Capture event 不是 FormalMemory。
- TaskMemoryPacket included 仍只允许正式记忆；candidate / observation 只能作为待审材料。
- `memory_workbench_summary` 是前端只读读模型，不新增 sidecar，不写 FormalMemory，不触发 runner。
- 普通 UI 不展示 raw sidecar、store revision、prompt body、full transcript 或 raw stdout / stderr。
- `store revision` / `sidecar path` 仍仅允许出现在已折叠开发者详情内。

## 复核线结论

复核线结论：带 P2 通过，允许主管线将 K4 本轮收口为 `accepted_non_real_productization_slice`。

复核线确认：

- 无 P0/P1。
- 未发现真实 Codex 执行、prompt 发送、`.codex` 读写、secret/full transcript/rollout 读取，或 K3-B1/K3-B2 冻结被突破。
- 未发现 observation/candidate/capture/knowledge hit 被冒充为 FormalMemory。
- Task memory packet included 仍只允许正式记忆，candidate/observation 只作为待审材料。
- 普通 UI 未暴露 raw sidecar / store revision / prompt / raw refs 长文案；已知 `store revision / sidecar path` 在折叠开发者详情内。

复核线唯一 P2：

- `MemoryCenterView.tsx` 成熟模式候选卡片中 `member refs / signal refs` 偏内部。主管线已修补为“关联成员 / 识别信号”。

## 验证

主管线验证：

```bash
npm run typecheck
```

结果：通过。

```bash
npm run test:offline-interaction
```

结果：通过，`offline interaction tests passed: 14`。

```bash
npm run build
```

结果：通过，仅保留既有 Vite chunk size warning。

```bash
node scripts/harness/stage-k-architecture-gate.js --target /Users/yoyi/workspace/product-line --strict
```

结果：通过，0 error / 0 warning。

## 不接受为

- K3-B1 retry 成功。
- K3-B2 可开始。
- 真实 Codex 执行后自动生成 observation / candidate 的真实执行验收完成。
- 工作流真实闭环后自动生成候选的真实执行验收完成。
- 用户确认候选后写 FormalMemory 的新能力完成。
- K4 全量完成。
- K5/K6 或 Stage K 完成。
- 任意项目无限制自由控制台、自动 retry / stop / restart、planned adapters 真实接入、provider credential / model verification、GraphRAG / 向量库 / 图数据库 / Obsidian 原生同步完成。

## 边界确认

本轮没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout，没有启动 Tauri / Browser / Chrome / Vite dev / screenshot。

本轮没有修改 Rust runner / Product Command 真实执行语义，没有修改 `workflow-state.v0.json` 顶层结构，没有新增 FormalMemory schema，没有新增 provider / credential / adapter 真实接入。

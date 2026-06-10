# Stage K K5 Running Todo Failure Recovery And Operation Control UX v1 Evidence

日期：2026-06-10

结论：`accepted_non_real_productization_slice`

## 背景

Stage K 原目标不变：自由操控 Codex + 自动化工作流 + 记忆层记录。当前 K3-B1 retry 已被安全审查再次拒绝，K3-B1 未完成，K3-B2 仍不得启动。

K5 本轮只推进不依赖真实 Codex 的运行状态产品化切片：把已有 run queue、user confirmation queue、failure control、readback boundary、duplicate guard、stale cleanup 和 operation readiness 整理成普通用户可读的运行中 / 待办 / 失败恢复 / 操作控制层级。

本轮不授权新的真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`，不读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout，不启动 K3-B1 retry，不启动 K3-B2。

## 改动范围

- `prototypes/productized-desktop-shell/src/lib/runQueue.ts`
- `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `tasks/2026-06-10-stage-k-k5-running-todo-failure-recovery-and-operation-control-ux-v1.md`

## 已完成

1. `RunQueueReadModel` 新增前端只读 `operation_control_summary`，schema 为 `operation_control_summary.v1`。
2. 操作控制摘要汇总 retry proposals、stop requests、restart / resume readiness、readback issues、duplicate blocked、guard blocked、stale cleanup、manual review 和 confirmation required。
3. `operation_control_summary.true_operation_available` 固定为 `false`；边界文案明确不会自动重试、不会 kill Codex、不会真实重启、不会执行真实恢复命令、读回 unknown 不等于 0、过期状态清理只处理工作台自有状态。
4. 运行中工作流页新增“操作控制 / 恢复建议”普通层，只展示摘要卡和只读边界卡，不新增执行按钮，不新增 Tauri invoke，不展示真实命令串。
5. 秘书只读模型把普通用户可见的“J4 队列”改成“运行队列”产品口径，仍不生成 retry / stop / restart / resume / send action proposal。
6. 离线测试新增 K5 断言，覆盖 operation summary schema、`true_operation_available=false`、readback null、无 `runner_call`、无真实恢复命令串、无“自动重试中 / 已停止 / 已重启 / 已恢复 / 结果数：0”等误导文案。

## 关键边界

- `operation_control_summary` 是前端派生读模型，不是新事实源。
- retry / stop / restart / resume 仍只是需确认、后续任务或只读 readiness，不是真实操作能力。
- `readback_unavailable` / `readback_failed` / `timed_out` / null result count 继续显示为未知 / 不可用，不显示为 0。
- duplicate blocked 和 guard blocked 只作为阻断 / 需人工查看，不触发 runner。
- stale cleanup 只作为工作台自有状态边界展示，不清理真实 Codex 本地状态。
- 普通 UI 不展示 raw JSON、sidecar 绝对路径、store revision、prompt body、full transcript、raw stdout / stderr、真实 `codex exec` / `codex exec resume` 命令串。
- 开发者字段如 `store revision` / `sidecar path` 仍仅允许出现在已折叠开发者详情内。

## 复核线结论

复核线最终结论：通过，允许主管线将 K5 本轮收口为 `accepted_non_real_productization_slice`。

复核线确认：

- 无 P0/P1/P2。
- 未发现真实 Codex 执行、prompt 发送、`.codex` 读写、secret/full transcript 读取，或 K3-B1/K3-B2 冻结被突破。
- `operation_control_summary` 仍是派生读模型，没有变成事实源或执行入口。
- retry / stop / restart / resume 仍只是需确认 / 后续任务 / 只读 readiness，未被 UI 或秘书说成已实现。
- readback unavailable / failed / timed_out / result_count null 仍显示为未知 / 不可用，不是 0。
- 普通 UI 未暴露 raw sidecar / store revision / prompt / raw refs 长文案。
- 上一版复核发现的两个 P2 已关闭：普通 UI 的“J4 队列”已改为“运行队列”，`stale cleanup` 已改为“过期清理 / 过期状态清理”。

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

扫描：

```bash
rg -n "J4 队列|stale cleanup|真实 \\.codex|codex exec resume|runner_call|自动重试中|已自动修复|已写正式记忆|结果数：0|已停止|已重启|已恢复|已 resume" \
  product-line/prototypes/productized-desktop-shell/src/lib/runQueue.ts \
  product-line/prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx \
  product-line/prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts \
  product-line/prototypes/productized-desktop-shell/src/styles.css
```

结果：无命中。

## 不接受为

- 真实 retry / stop / restart / resume 已实现。
- 真实 Codex 已被再次执行。
- 自动清理真实 Codex 本地状态完成。
- K3-B1 retry 成功。
- K3-B2 可开始。
- K5 全量完成。
- K6 或 Stage K 完成。
- 任意项目无限制自由控制台、自动 retry / stop / restart、planned adapters 真实接入、provider credential / model verification、GraphRAG / 向量库 / 图数据库 / Obsidian 原生同步完成。

## 边界确认

本轮没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout，没有启动 Tauri / Browser / Chrome / Vite dev / screenshot。

本轮没有修改 Rust runner / Product Command 真实执行语义，没有修改 Tauri command wrapper，没有修改 `workflow-state.v0.json` 顶层结构，没有新增 FormalMemory schema，没有新增 provider / credential / adapter 真实接入。

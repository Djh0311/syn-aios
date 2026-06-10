# Stage K Architecture Calibration v2 And Gate v1 Evidence

日期：2026-06-10

结论：`accepted_architecture_gate_added`

## 背景

Stage K 原目标不变：自由操控 Codex + 自动化工作流 + 记忆层记录。当前 K3-B1 retry 已被安全审查再次拒绝，K3-B1 未完成，K3-B2 不得启动。

本轮不继续真实执行线，而是补一轮架构校准：

- 写入 Stage K 架构校准补充计划 v2。
- 新增只读 Stage K architecture gate。
- 用 gate 复核真实执行归口、legacy / sealed runner、prompt body、readback null、fixture 常量和 FormalMemory 误称风险。

## 新增 / 修改

- `docs/plans/2026-06-10-stage-k-architecture-calibration-plan-v2.md`
- `scripts/harness/stage-k-architecture-gate.js`
- `docs/plans/2026-06-10-stage-k-daily-use-codex-workbench-productization-plan-v1.md`
- `CURRENT.md`
- `tasks/README.md`

## Gate 覆盖项

`scripts/harness/stage-k-architecture-gate.js` 是只读扫描脚本，不执行 Codex、不发送 prompt、不读写 `/Users/yoyi/.codex`。

当前覆盖：

- 裸 `Command::new("codex")` 只能在批准 runner 或文档说明中出现。
- 普通前端不能直接调用 legacy workflow dispatch / workflow machine / sealed canvas real-run wrapper。
- `prompt_body` 只能在 Phase B runtime input、类型 / UI 入参边界、测试断言或文档边界出现。
- `result_count=0` 在测试 fixture 中只作为 info；产品 unknown-result 状态仍必须归一为 `null`。
- K2 / J2 / K3 / H5 / PCR9 等阶段常量必须留在 fixture、测试、任务或文档边界。
- 候选 / observation / knowledge hit 不能误写成自动进入 FormalMemory；否定句会分类为 info。

## 验证

### 语法检查

```bash
node --check scripts/harness/stage-k-architecture-gate.js
```

结果：通过。

### Gate 普通模式

```bash
node scripts/harness/stage-k-architecture-gate.js --target /Users/yoyi/workspace/product-line
```

结果：

- Status: `pass`
- Errors: `0`
- Warnings: `0`
- Info: `33`

### Gate strict 模式

```bash
node scripts/harness/stage-k-architecture-gate.js --target /Users/yoyi/workspace/product-line --strict
```

结果：

- Status: `pass`
- Errors: `0`
- Warnings: `0`
- Info: `33`

## 关键判断

- K3-B1 真实 retry 仍未完成。
- K3-B2 仍冻结。
- Stage K 原目标不缩小。
- 当前可继续推进的是不依赖真实 Codex 的架构校准、UI 信息层级、memory consistency、运行队列和 K4/K5 非真实产品化切片。
- 任何新的真实 `codex exec` / `codex exec resume` 仍必须单独执行点授权，并通过安全审查。

## 边界确认

本轮没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout，没有启动 Tauri / Browser / Chrome / Vite dev / screenshot。

本轮只新增文档和只读扫描脚本，不改变真实执行 runner 语义，不推进 K3-B2。

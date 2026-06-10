# Root Treatment R0 Shape Gate And Governance Task Package Rule v1

日期：2026-06-10

状态：R0 制度说明。本文服务 `tasks/2026-06-10-root-treatment-r0-shape-gate-task-template-and-governance-package-type-v1.md`，不是产品功能任务包，不授权真实 Codex 执行，不读写 `/Users/yoyi/.codex`，不修改业务逻辑。

## 1. Workbench Shape Gate

权威脚本：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
```

脚本边界：

- 只读扫描源码和 git 元数据。
- 不执行 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 不读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。

模式：

- `baseline`：输出当前形状指标，不因既有巨石文件或历史债失败。
- `check`：执行 R0 棘轮规则；硬错误返回非 0。
- `--json`：输出机器可读报告。
- `--strict`：把 warning 也作为失败，默认不启用。

当前硬规则：

- `lib.rs` 水位线只降不升，R0 初始水位线为 `25,925` 行。
- `lib.rs` 内 `#[tauri::command]` 数量必须保持 `0`。
- 新增生产 sidecar JSON 种类必须命中 R0 允许清单；新 sidecar 必须先有用户确认和 `decisions/**` 记录。
- 新建 Rust 文件不得超过 `3,000` 行。
- 新建 TS / TSX 文件不得超过 `2,000` 行。
- 存量超限文件进入 ratchet list，只降不升。

当前 P2：

- sidecar 扫描基于源码字符串和 R0 允许清单，能阻断常规新增 `*.vN.json` 名称；如果后续出现动态拼接 sidecar 名称，需要在 R2/R3 前进一步收紧扫描或迁入统一存储。
- Tauri command 总数增加目前为 warning；硬阻断范围先限定为“不得新增到 `lib.rs`”。后续 R2 命令注册拆分后可把 command surface 规则收紧。

## 2. 任务包形状影响节

所有新任务包必须包含 `形状影响`，并至少说明：

- 任务类型。
- 新增代码落点。
- 是否触碰棘轮文件。
- 预计行数变化。
- 是否新增 Tauri command。
- 是否新增 sidecar JSON 种类。
- 是否需要 shape gate 豁免和对应 decision。
- 本任务基线 commit / 完成 commit。

`TASK_TEMPLATE.md` 是当前模板入口。

## 3. 治理任务包类型

治理任务包的验收口径：

```text
行为不变 + 形状指标改善 + evidence 记录前后指标。
```

治理任务包也走既有任务包、evidence、handoff 和回收流程，不另起制度。治理任务包不得借“重构”夹带产品功能。

## 4. 解冻后治理配额

治理冻结期内，R0-R5 默认都是治理任务包。解冻恢复功能开发后，配额为：

```text
每 3 个功能任务包至少配 1 个治理任务包。
```

跑一个 Stage 后可复盘调整。配额例外必须写入 `decisions/**`，不能沉默跳过。

## 5. Evidence / Handoff Commit Hash

每个治理任务包 evidence / handoff 必须记录：

- start commit。
- end commit。
- shape gate baseline 摘要。
- shape gate check 摘要。
- `git diff --check` 结果。
- 如无 git，必须标记 `no_git_blocked_for_r2_r3`，并阻断 R2/R3。

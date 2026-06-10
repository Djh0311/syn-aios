# Root Treatment / R1 Workflow State Lock And Backup Retention v1

日期：2026-06-10

状态：待执行。本文是 Root Treatment / Stage R 的 R1 任务包，用于给 `workflow-state.v0.json` 写入路径补 StoreLock 和备份保留策略，先降低 JSON 事实层的并发写、备份膨胀和半写风险。

R1 是“立即止血”任务，不是 SQLite 迁移，不改 workflow state 顶层 schema，不做 UI，不执行真实 Codex。

## 0. 全局主管理解

已知事实：

- Root Treatment 正式计划要求 R1 可与 R0 并行，但代码合入必须受 shape gate 约束；如果 R0 尚未完成，R1 evidence 至少手工记录形状指标。
- R-Preflight 已建立 git baseline。
- R0 将建立通用 shape gate；R1 如果先于 R0 完成，必须在 evidence 中标明“R0 gate 未完全上线时的手工指标”。
- `workflow-state.v0.json` 当前仍是 v0 JSON 事实层，R3 才迁移 SQLite。
- 治本方案指出当前 backups 只增不清，存在长期膨胀风险。

R1 的核心判断：

```text
不等 R3，先把当前 JSON 写入路径加锁、把备份增长控制住。
```

## 1. 权威依据

必须读取并服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `decisions/2026-06-10-stage-l-root-treatment-freeze-relationship-v1.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/plans/2026-06-10-root-treatment-plan-v1.md`
- `evidence/2026-06-10-root-treatment-r-preflight-authority-sync-and-git-baseline-v1.md`
- `handoffs/2026-06-10-root-treatment-r-preflight-authority-sync-and-git-baseline-v1-result.md`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`

## 2. 目标

R1 必须完成：

- 审计 `workflow_state_store.rs` 当前所有写路径：
  - load / save。
  - atomic temp + rename。
  - backup 创建。
  - revision conflict / corrupt JSON 处理。
  - 上层调用是否已有互斥。
- 如果没有完整上层互斥，给 workflow state 写入补 StoreLock。
- StoreLock 必须：
  - lock path 稳定。
  - 写入前 acquire。
  - Drop 释放。
  - lock busy 可分类为明确错误。
  - 不能因 lock 失败覆盖原文件。
  - 测试中可模拟 lock busy。
- 保持现有 temp + rename 原子替换语义。
- 增加备份保留策略函数：
  - 默认保留最近 30 份。
  - 同时保留每日 1 份。
  - 测试中可控时间 / 文件名。
- R1 默认只在测试夹具中验证 prune，不对用户真实历史 backups 做不可逆清理。
- 补 Rust 单测覆盖：
  - 并发写 / lock busy。
  - corrupt JSON 不覆盖原文件。
  - revision conflict 不覆盖原文件。
  - backup retention 策略。
- 写 R1 evidence / handoff，记录形状指标、测试命令、完成 commit。

## 3. 允许修改

允许写入：

- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- 如确有必要，可新增小型 Rust helper 模块，但必须说明为什么不能放在 `workflow_state_store.rs` 内。
- 如测试需要，可修改或新增 Rust 单测。
- `evidence/2026-06-10-root-treatment-r1-workflow-state-lock-and-backup-retention-v1.md`
- `handoffs/2026-06-10-root-treatment-r1-workflow-state-lock-and-backup-retention-v1-result.md`
- R1 完成 checkpoint 的入口文档同步：`CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`。

允许读取：

- 项目源码和文档。
- git 元数据。
- 测试夹具和本项目 `tmp/**`。

## 4. 禁止事项

R1 禁止：

- 不迁移 SQLite。
- 不改 workflow state 顶层 schema。
- 不新增 sidecar store。
- 不新增 Tauri command。
- 不改真实 Codex runner。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 不启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 不对真实历史 backups 做不可逆清理；真实清理必须另起任务，先 dry-run 再用户确认。
- 不启动 Stage L / K3-B1 / K3-B2。
- 不做 UI、无限画布、MCP 视觉工具或 backlog 功能。

## 5. 形状影响

预期修改：

- 主要修改 `workflow_state_store.rs`。
- 可能新增小型测试 helper。
- 不新增 command。
- 不新增 sidecar。
- 不新增 UI。

行数约束：

- `workflow_state_store.rs` 是治理目标文件之一，允许小幅增加以换取写入安全，但 evidence 必须记录前后行数。
- 如新增 Rust 文件，必须低于 3,000 行，且建议低于 500 行。
- 禁止把新增逻辑塞回 `lib.rs`。

Git 要求：

- R1 开始基线 commit：`b409ab92d36b44f63911a4f12b057e5577f8aeb5` 或 R0 完成后的最新治理 commit。
- R1 完成后必须形成独立 commit。
- evidence / handoff 必须记录 start commit / end commit。

## 6. 验收标准

R1 可接受为：

- 写路径审计完成并写入 evidence。
- workflow state 写入路径具备 StoreLock 或明确证明已有互斥覆盖全部写入。
- lock busy / corrupt JSON / revision conflict 不覆盖原文件。
- 备份保留策略在测试夹具中通过。
- 不对真实历史 backups 做不可逆清理。
- `cargo test --lib workflow_state` 或更精确相关测试通过。
- `cargo fmt -- --check` 或 `rustfmt --check` 相关文件通过。
- 如 R0 shape gate 已完成，必须跑 `node scripts/harness/workbench-shape-gate.js --mode check`。
- `git diff --check` 针对 R1 增量通过。

R1 不接受为：

- R0 完成。
- R2 lib.rs 解体完成。
- R3 SQLite 迁移完成。
- workflow state schema 迁移完成。
- 真实 backups 已清理完成。
- Stage L / K3-B1 / K3-B2 恢复。

## 7. 建议验证命令

必须跑：

```bash
cargo test --lib workflow_state
cargo fmt -- --check
git diff --check
git status --short
```

如果测试名不匹配，应在 evidence 中记录实际可用的聚焦测试名称，并补跑：

```bash
cargo test --lib
```

如 R0 已完成，补跑：

```bash
node scripts/harness/workbench-shape-gate.js --mode check
```

如果只改 Rust 后端，默认不需要跑 `npm run build`；如触碰前端类型或 UI，则必须补跑 `npm run typecheck`、`npm run test:offline-interaction`、`npm run build`。

## 8. 必须回传

开发线回传必须包含：

1. 做了什么。
2. 改了哪些文件。
3. 写路径审计结果。
4. StoreLock 实现边界。
5. backup retention 策略说明。
6. 测试清单和结果。
7. shape 指标前后值。
8. start commit / end commit。
9. 是否触碰真实 backups。
10. 是否触碰任何禁止项。

## 9. 总指导回收动作

总指导回收时必须判断：

- `accepted`
- `accepted_with_p2`
- `needs_changes`
- `blocked`

P0/P1 示例：

- 没有 StoreLock 且没有证明上层互斥。
- lock busy 会覆盖原文件。
- corrupt JSON / revision conflict 会覆盖原文件。
- retention 测试没覆盖。
- 对真实历史 backups 做了不可逆清理。
- 新增 command 或 sidecar。

P2 示例：

- 只实现进程内 lock，跨进程 lock 需要 R3 / 后续任务继续强化。
- retention 当前只在新写入后触发，不做全量历史清理。

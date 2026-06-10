# Root Treatment / R0 Shape Gate, Task Template, And Governance Package Type v1

日期：2026-06-10

状态：已完成，结论为 `accepted_with_p2`。本文是 Root Treatment / Stage R 的 R0 任务包，用于建立工作台开发的“形状治理”制度前置：shape gate、任务包“形状影响”必填节、治理任务包类型、解冻后 `1:3` 治理配额和 evidence / handoff 的 commit hash 记录要求。完成 commit：`7563e6a9d11a92217e1baf34ed71b70722bbc17c`。

R0 不改业务行为，不执行真实 Codex，不读写 `/Users/yoyi/.codex`，不启动 Stage L / K3-B1 / K3-B2，不解冻 backlog 功能。

## 0. 全局主管理解

已知事实：

- Root Treatment 正式计划已创建：`docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`。
- R-Preflight 已完成权威入口同步和 git baseline 建立。
- 治理前 baseline commit：`ed01c6f281e3fd7a38548da948046e8366cc368d`。
- R-Preflight 收口 commit：`b409ab92d36b44f63911a4f12b057e5577f8aeb5`。
- 当前目录此前没有 git；现在已经建立可审查 diff 和可回滚提交。
- `src-tauri/src/lib.rs`、`real_execution_command.rs`、`ProjectsView.tsx`、`types.ts`、`styles.css`、离线测试主文件等存在巨石化和棘轮治理需求。
- 首次 baseline 前 `git diff --cached --check` 暴露大量历史 trailing whitespace / EOF blank line 债务；R0 只记录和 gate，不批量清理历史债。

R0 的核心判断：

```text
先让后续任务停止继续长歪，再进入 R1/R2/R3/R4 的具体治理。
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
- `handoffs/2026-06-10-root-treatment-plan-claude-to-codex-kickoff-v1.md`
- `evidence/2026-06-10-root-treatment-r-preflight-authority-sync-and-git-baseline-v1.md`
- `handoffs/2026-06-10-root-treatment-r-preflight-authority-sync-and-git-baseline-v1-result.md`
- `scripts/harness/stage-k-architecture-gate.js`
- `TASK_TEMPLATE.md`
- `templates/docs/**`

## 2. 目标

R0 必须完成：

- 新增通用 shape gate 脚本：`scripts/harness/workbench-shape-gate.js`。
- 建立当前形状基线输出，至少覆盖：
  - `src-tauri/src/lib.rs` 行数。
  - `real_execution_command.rs` 行数。
  - `ProjectsView.tsx` 行数。
  - `AgentView.tsx` 行数。
  - `types.ts` 行数。
  - `styles.css` 行数。
  - 离线测试主文件行数。
  - Tauri command 总量。
  - `lib.rs` 内 `#[tauri::command]` 数量。
  - 生产 sidecar JSON 种类扫描结果。
  - 新建文件上限检查：Rust 3,000 行、TS / TSX 2,000 行。
- 支持至少两个模式：
  - `--mode baseline`：输出当前指标，不因既有债失败。
  - `--mode check`：执行当前棘轮规则，能返回可读 findings。
- 将 `lib.rs` 初始水位线冻结为当前扫描值；R0 不需要直接降行数。
- 禁止新增 `#[tauri::command]` 到 `lib.rs` 的规则必须可检测。
- 禁止新增生产 sidecar JSON 种类的规则必须可检测；如现阶段只能生成 baseline 清单，必须把缺口写入 evidence，不得冒充强制完成。
- 更新 `TASK_TEMPLATE.md`，新增“形状影响”必填节，要求说明：
  - 新增代码落点。
  - 是否触碰棘轮文件。
  - 预计行数变化。
  - 是否新增 command。
  - 是否新增 sidecar。
  - 是否需要 gate 豁免和 decision。
  - 本任务基线 commit / 完成 commit。
- 定义治理任务包类型：验收 = 行为不变 + 形状指标改善 + evidence 记录前后指标。
- 将解冻后治理配额写入模板 / 文档：每 3 个功能任务包至少配 1 个治理任务包，跑一个 Stage 后可复盘调整。
- 写 R0 evidence / handoff，记录 baseline、执行命令、结果、剩余 P2 和完成 commit。

## 3. 允许修改

允许写入：

- `scripts/harness/workbench-shape-gate.js`
- `TASK_TEMPLATE.md`
- 必要的模板 / 说明文档，例如 `templates/docs/**` 或 `docs/plans/**` 中与任务包模板、治理任务包类型、shape gate 使用说明直接相关的文件。
- `evidence/2026-06-10-root-treatment-r0-shape-gate-task-template-and-governance-package-type-v1.md`
- `handoffs/2026-06-10-root-treatment-r0-shape-gate-task-template-and-governance-package-type-v1-result.md`
- 当前权威入口在 R0 完成 checkpoint 时的状态同步：`CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`。

允许读取：

- 全部项目源码和文档。
- git 元数据。

## 4. 禁止事项

R0 禁止：

- 不改产品业务逻辑。
- 不改 Rust/Tauri runner 真实执行语义。
- 不改 workflow state JSON schema。
- 不新增 sidecar store。
- 不迁移 SQLite。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 不启动 Tauri / Browser / Chrome / 截图工具。
- 不启动 Stage L / L1-L6。
- 不启动 K3-B1 retry 或 K3-B2。
- 不做 UI 布局重做、无限画布、UI 视觉反馈 MCP 工具或 backlog 功能。
- 不批量清理历史 whitespace 债，除非单独拆治理任务包。

## 5. 形状影响

预期新增：

- 新增脚本 1 个：`scripts/harness/workbench-shape-gate.js`。
- 修改任务包模板 1 个：`TASK_TEMPLATE.md`。
- 可能新增少量文档说明。

预期不新增：

- 不新增 Tauri command。
- 不新增 Rust 生产模块。
- 不新增 sidecar。
- 不新增数据库。
- 不新增 UI 页面。

文件上限：

- 新增 JS 脚本应控制在 500 行以内；如超过 800 行必须拆模块或写明原因。
- 新增文档不设硬上限，但必须服务 R0 验收，不写泛化长文。

Git 要求：

- R0 开始基线 commit：`b409ab92d36b44f63911a4f12b057e5577f8aeb5`。
- R0 完成后必须形成独立 commit。
- evidence / handoff 必须记录完成 commit hash。

## 6. 验收标准

R0 可接受为：

- `node scripts/harness/workbench-shape-gate.js --mode baseline` 可运行并输出指标。
- `node scripts/harness/workbench-shape-gate.js --mode check` 可运行并输出 findings / pass-fail 结果。
- baseline 输出包含 `lib.rs` 水位线和棘轮文件清单。
- gate 能识别新增 command 进入 `lib.rs` 的风险。
- gate 能识别新增生产 sidecar JSON 种类的风险，或明确标为 R0 P2。
- `TASK_TEMPLATE.md` 已新增“形状影响”必填节。
- 治理任务包类型和解冻后 `1:3` 治理配额已写入模板 / 文档。
- evidence / handoff 记录 start commit、end commit、执行命令和结果。
- `git diff --check` 针对 R0 增量通过；历史 baseline whitespace 债不要求本轮清零。

R0 不接受为：

- R1 完成。
- R2 lib.rs 解体完成。
- R3 SQLite 迁移完成。
- R4 前端瘦身完成。
- Stage L 恢复。
- backlog 功能解冻。

## 7. 建议验证命令

必须跑：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
git diff --check
git status --short
```

如脚本使用 Node.js 标准库且未改产品代码，可以不跑 `npm run build` / `cargo test --lib`；但 evidence 必须说明原因。

如果 R0 修改了任何前端 / Rust 产品代码，必须补跑相关 `npm` / `cargo` 验证。

## 8. 必须回传

开发线回传必须包含：

1. 做了什么。
2. 改了哪些文件。
3. shape gate baseline 输出摘要。
4. shape gate check 输出摘要。
5. `TASK_TEMPLATE.md` 新增字段摘要。
6. 治理任务包类型和 `1:3` 配额落点。
7. start commit / end commit。
8. 未解决的 P2。
9. 是否触碰任何禁止项。

## 9. 总指导回收动作

总指导回收时必须判断：

- `accepted`
- `accepted_with_p2`
- `needs_changes`
- `blocked`

P0/P1 示例：

- gate 脚本不能运行。
- 模板没有形状影响节。
- 治理配额没有写入。
- R0 顺手改了产品业务逻辑。
- 未记录 commit hash。

P2 示例：

- sidecar 种类扫描只能 baseline，尚不能强制阻断。
- command 分类需要 R2 后进一步精确。

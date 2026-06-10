# 任务包：final-skeleton-00-current-facts-freeze-v1

## 目标

冻结当前原型架构事实，给后续最终工作台骨架小切片作为基线。

## 依据

- `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md`
- `CURRENT.md`
- `AUTHORITY.md`
- `tasks/README.md`
- `docs/workbench-system-architecture-v1.md`
- 已完成的架构相关 evidence / handoff

## 允许

- 只读复核当前入口、架构文档、最近 evidence / handoff、原型代码模块。
- 新增本任务 evidence。
- 新增本任务 handoff。
- 更新 `CURRENT.md` 和 `tasks/README.md` 的当前事实。

## 禁止

- 不改代码。
- 不改 workflow state。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 auth、token、`.env`、密钥、完整 transcript 或 rollout JSONL 正文。
- 不迁移数据库。
- 不写真实业务项目目录。

## 执行步骤

1. 列出当前已完成骨架能力。
2. 列出当前未完成骨架能力。
3. 列出当前主要代码模块和职责。
4. 列出 `lib.rs`、`ProjectsView.tsx`、工作流状态、黑板、控制核心、读模型的剩余风险。
5. 输出最终骨架完成度矩阵。

## 验收

- 只新增 evidence / handoff。
- 可以更新 `CURRENT.md` 和 `tasks/README.md`。
- 不需要跑代码测试。

## 输出

- `evidence/2026-06-01-final-skeleton-00-current-facts-freeze-v1.md`
- `handoffs/2026-06-01-final-skeleton-00-current-facts-freeze-v1-result.md`

## 完成后

普通小任务，不必停；继续执行 `final-skeleton-01-audit-helper-slice-v1`，除非发现权威冲突。

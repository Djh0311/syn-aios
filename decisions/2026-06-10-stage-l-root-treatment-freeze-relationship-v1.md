# 决策：Stage L 与治理阶段 R 的冻结关系 v1

日期：2026-06-10
拍板：用户已要求按治本全计划开发推进；全局主管复核后执行。

## 影响面

- `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 的当前入口口径。
- `docs/plans/2026-06-10-stage-l-post-k-deferred-closure-and-daily-use-hardening-plan-v1.md` 的执行状态解释。
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md` 的 R-Preflight 执行依据。
- Stage L / L1-L6 任务包调度顺序。
- Stage K / K3-B1 / K3-B2 后续恢复顺序。

## 结论

治理阶段 R 插队执行。Stage L 剩余 L1-L6 在治理冻结期内暂挂为 `deferred_during_root_treatment`，不在当前治理期内开工。

Stage L 暂挂不等于完成，也不等于取消。治理收口后，仍回到 Stage L / Stage K 继续处理 K3-B1、K3-B2、真实恢复、操作控制、记忆闭环和日常硬化。

当前下一步进入 R-Preflight：同步权威入口、建立版本控制前置、拆 R0 / R1 任务包。

## 大白话

先把施工队的脚手架补好，再继续加楼层。

Stage L 处理的是很急的产品恢复问题，但它会继续新增状态、UI、读模型、运行记录和记忆捕获。如果在 shape gate、写入锁、版本控制和后续存储治理之前继续堆功能，老问题会被放大。所以治理阶段 R 先插队，把“代码形状、写入安全、证据可回滚”补上。

这不是放弃 K3-B1 / K3-B2。只是把它们从“马上继续写功能”改成“治理后按更安全的机制继续”。

## 允许范围

治理冻结期允许：

- 执行 R-Preflight、R0、R1、R2、R3、R4、R5。
- 同步权威入口和任务队列口径。
- 创建治理任务包。
- 建立 shape gate、任务包形状影响节、治理任务包类型和解冻后治理配额。
- 建立版本控制 baseline 或等价可回滚方案。
- 做 R1 workflow state 写入锁和备份保留策略。
- 做 R2/R3/R4/R5 的治理型重构、迁移和文档对齐。

## 暂挂范围

治理冻结期暂挂：

- Stage L / L1-L6 产品代码任务。
- K3-B1 retry。
- K3-B2 isolated workspace-write execution。
- 新的真实 `codex exec` / `codex exec resume` 执行点。
- planned adapters 真实接入。
- provider credential store、真实 token 读取或 model verification。
- backlog 中的解冻后用户功能：前端布局重做、无限画布、UI 视觉反馈 MCP 工具、秘书型 AI、记忆时效标注等。

## 不接受为

- 不接受为 Stage L 已完成。
- 不接受为 Stage L 被取消。
- 不接受为 K3-B1 / K3-B2 被取消。
- 不接受为允许绕过安全审查执行真实 Codex。
- 不接受为治理阶段 R 已完成。
- 不接受为 R0 / R1 已完成。
- 不接受为 backlog 功能解冻。

## 与现有计划的关系

- `docs/plans/2026-06-10-stage-l-post-k-deferred-closure-and-daily-use-hardening-plan-v1.md`：保持为 Stage L 后续计划，但当前执行状态改为治理期 paused/deferred。
- `docs/plans/2026-06-10-root-treatment-plan-v1.md`：用户已确认的治本方案，是本决策的来源之一。
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`：本决策补齐其 R-Preflight 前置。
- `handoffs/2026-06-10-root-treatment-plan-claude-to-codex-kickoff-v1.md`：本决策响应其“必须复核 Stage L 与治理冻结关系”的要求。

## 依据

- 用户已确认治本方案核心决策：冻结新功能，集中治理；R3 SQLite 收口前不开多 agent 并行真实执行；解冻后每 3 个功能任务包至少 1 个治理任务包。
- 当前权威入口此前仍指向 Stage L / L1 待执行，和治本方案“治理插队”的启动要求存在当前入口冲突。
- Stage L L1 属于产品路径 / UI / 状态恢复工作，不是纯治理任务；继续推进会绕开 shape gate 和写入治理。
- K3-B1 retry 仍受安全审查阻断，K3-B2 仍依赖 K3-B1 成功或等价替代路径。

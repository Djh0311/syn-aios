# Stage H / H2.1 Real Resume Authorization Matrix And Execution Decision Freeze v1

日期：2026-06-07

状态：已完成，等待全局主管和用户基于本矩阵决定是否进入 H2 真实执行。

用途：把 H2 通用真实 resume 产品化的执行前授权项整理成可逐项确认的决策工作表。H2.1 不执行真实 `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`，不授权 H3。

## 1. 权威依据

本任务包依据：

- `tasks/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md`
- `evidence/2026-06-07-stage-h-h2-0-real-resume-preflight-authorization-guard-v1.md`
- `handoffs/2026-06-07-stage-h-h2-0-real-resume-preflight-authorization-guard-v1-result.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `tasks/2026-06-07-stage-h-h0-safety-boundary-and-task-package-freeze-v1.md`
- `tasks/2026-06-07-stage-h-h1-codex-local-runner-architecture-and-data-contract-v1.md`

## 2. 当前事实

- H0 已冻结阶段 H 安全边界和 H1-H7 顺序。
- H1 已完成 CodexLocalRunner 契约、guard、fake dry-run、结构化 argv 和 stdin prompt ref/hash 边界。
- H2 任务包已创建，但仍是待执行前授权。
- H2.0 已完成授权预检 guard，授权矩阵完整时也只返回 `complete_but_not_executed`，不调用真实 runner。
- E5 Level B 只证明 `/Users/yoyi/Documents/mario test` 指定 session 的一次健康探针可行，不能作为 H2 默认授权。

## 3. H2.1 接受范围

H2.1 接受为：

- H2 真实 resume 执行前授权工作表完成。
- 推荐默认方案、待用户确认项、禁止假设项和停止条件已冻结。
- 后续 H2 真实执行前需要确认的问题已明确。

H2.1 不接受为：

- H2 通用真实 resume 产品化完成。
- 真实 `codex exec resume` 已执行。
- prompt 已发送。
- `/Users/yoyi/.codex` 已读写。
- H3 通用真实 send / 新会话可开始。
- 项目工作流真实派发、planned adapters 真实接入或 provider credential / model verification 完成。

## 4. 推荐授权矩阵草案

| 授权项 | 推荐默认值 | 必须由用户 / 全局主管确认 | 说明 |
| --- | --- | --- | --- |
| 操作类型 | `resume` | 是 | H2 只做 `codex-local` resume，不做 H3 send / new session。 |
| 测试项目 | 新建隔离 fixture 项目 | 是 | 推荐不要默认复用 `mario test`，避免把历史健康探针误当通用能力。 |
| fixture 路径 | `/Users/yoyi/workspace/product-line/tmp/h2-real-resume-fixture` | 是 | 位于 workspace 内，可控、可删除、低风险。创建前仍需确认。 |
| project root | 与 fixture 路径一致 | 是 | 必须是绝对路径，不能含 `..`。 |
| target cwd | 与 fixture 路径一致 | 是 | 必须在 project root 或 allowed write roots 内。 |
| target session | 待用户指定或工作台绑定 | 是 | 不能由主管线猜测，也不能读取 `.codex` 搜索完整 transcript。 |
| prompt summary | `H2 real resume safe probe` | 是 | 只作为摘要；完整 prompt 不进入 argv，不进任务包正文。 |
| prompt hash / ref | 执行前生成 | 是 | 执行前必须固定 hash/ref；不在 H2.1 伪造。 |
| allowed write roots | fixture 项目目录 | 是 | 不允许默认写真实业务项目。 |
| `.codex` 范围 | 仅限 Codex CLI resume 必需最小范围 | 是 | 禁止 auth/token/secret/full transcript/provider credential。 |
| sandbox | 受控 sandbox，禁止 dangerous bypass | 是 | 明确禁止 `--dangerously-bypass-approvals-and-sandbox`。 |
| timeout | 建议 120000 ms | 是 | 超时必须写 failure reason，不自动重试。 |
| readback plan | workbench-managed last message + status 分类 | 是 | unavailable / failed / timed out 不得显示为 0 条结果。 |
| evidence path | `evidence/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md` | 是 | H2 真正执行后才写真实结果。 |
| handoff path | `handoffs/2026-06-07-stage-h-h2-general-real-resume-productization-v1-result.md` | 是 | H2 真正执行后才写真实结果。 |
| rollback / 降级 | 执行前后 hash、runtime log、audit、readback 分类 | 是 | H2 如果失败，记录为失败分类，不包装成通过。 |

## 5. 用户确认问题

进入 H2 真实执行前，必须得到用户明确回答：

1. 是否同意新建或使用隔离 fixture 项目。
2. 是否确认 fixture 路径为 `/Users/yoyi/workspace/product-line/tmp/h2-real-resume-fixture`，或指定其他路径。
3. 目标 Codex session 是哪个；如果还没有绑定 session，是否先单独创建 / 绑定低风险测试 session。
4. 是否允许本次 H2 真实执行调用 `codex exec resume`。
5. 是否允许本次 H2 真实执行触碰 `/Users/yoyi/.codex` 的 resume 必需最小范围。
6. 是否接受执行前后项目文件 hash / 差异记录写入 evidence。
7. 如果 readback failed / unavailable / timed out，是否停止在 H2.x 修补，而不是进入 H3。

## 6. 禁止假设项

- 不能默认使用 `mario test`。
- 不能默认选择某个 target session。
- 不能默认读取 `/Users/yoyi/.codex` 来寻找完整 transcript。
- 不能默认允许真实 `codex exec resume`。
- 不能默认允许 prompt 发送。
- 不能默认允许写真实业务项目。
- 不能默认把 readback unavailable 当成 0 条结果。
- 不能默认进入 H3、H4 或 H5。

## 7. H2 执行停止条件

遇到以下任一情况，H2 真实执行必须停止：

- 用户未确认 target session。
- 用户未确认 `.codex` 最小触碰范围。
- fixture 项目路径不在允许写入范围内。
- prompt summary/hash/ref 缺失。
- readback plan 缺失。
- 发现需要读取 auth、token、secret、`.env`、keychain、OAuth、provider credential 或完整 transcript。
- 需要使用 shell 字符串拼接或 dangerous sandbox bypass。
- 出现 queued/running duplicate attempt。
- 全局主管未复核通过。

## 8. 后续执行建议

建议下一步不是直接进入 H3，而是进入 H2 真实执行准备或 H2.x 修补：

1. 如果用户确认授权矩阵：进入 H2 真实 runner 实现 / 执行任务。
2. 如果用户不确认 target session：先拆 H2.a target session 绑定准备任务。
3. 如果用户不确认 `.codex` 范围：保持 H2 blocked，不做真实执行。
4. 如果 H2 真实执行失败或 readback 不可信：先拆 H2.x failure/readback 修补任务，再判断是否进入 H3。

## 9. 回交要求

H2.1 完成后必须新增：

- `evidence/2026-06-07-stage-h-h2-1-real-resume-authorization-matrix-and-execution-decision-freeze-v1.md`
- `handoffs/2026-06-07-stage-h-h2-1-real-resume-authorization-matrix-and-execution-decision-freeze-v1-result.md`

并同步：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`

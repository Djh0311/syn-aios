# Stage H / H2.4 Real Resume Execution Authorization And Fixture Freeze v1

日期：2026-06-07

状态：已完成，等待用户 / 全局主管基于本授权包决定是否进入 H2 真实执行。

用途：把 H2 通用真实 resume 产品化从“可预检、可构建 request、可跑 guard”推进到“真实执行前可审批”的状态。H2.4 只冻结执行授权包、fixture 建议、证据路径、回滚策略和停止条件；不执行真实 `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`，不创建 fixture，不授权 H3。

## 1. 权威依据

本任务包依据：

- `tasks/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md`
- `tasks/2026-06-07-stage-h-h2-1-real-resume-authorization-matrix-and-execution-decision-freeze-v1.md`
- `tasks/2026-06-07-stage-h-h2-2-real-resume-authorization-readiness-read-model-and-readonly-ui-v1.md`
- `tasks/2026-06-07-stage-h-h2-3-real-resume-request-builder-and-codex-local-guard-bridge-v1.md`
- `evidence/2026-06-07-stage-h-h2-3-real-resume-request-builder-and-codex-local-guard-bridge-v1.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`

## 2. 当前事实

- H2 主任务包允许后续在明确授权后实现受控真实 `codex exec resume`，但任务包创建本身不授权执行。
- H2.0 已完成执行前授权预检 guard。
- H2.1 已冻结授权矩阵和主管决策材料。
- H2.2 已把授权缺口显示到只读 UI。
- H2.3 已在授权矩阵完整时构建 H1 `CodexLocalExecutionRequest` 并跑 H1 guard inspection，但仍只返回 `complete_but_not_executed`。
- 当前仍没有用户确认的 fixture、target session、`.codex` 最小范围、prompt hash/ref 或真实执行批准。

## 3. 接受范围

H2.4 接受为：

- H2 真实执行前授权包完成。
- 隔离 fixture 推荐方案冻结。
- 用户 / 全局主管必须确认项冻结。
- prompt summary/hash/ref 生成规则冻结。
- allowed write roots、`.codex` 最小范围、readback plan、runtime log、audit、evidence 和 rollback 要求冻结。
- 停止条件和不得执行项冻结。
- 权威入口同步到 H2.4 已完成、H2 真实 resume 仍待用户明确授权。

H2.4 不接受为：

- H2 通用真实 resume 产品化完成。
- 真实 `codex exec resume` 已执行。
- prompt 已发送。
- `/Users/yoyi/.codex` 已读写。
- fixture 项目已创建。
- target session 已确认。
- H3 通用真实 send / 新会话可开始。
- 项目工作流真实派发、planned adapters 真实接入或 provider credential / model verification 完成。

## 4. 推荐执行授权包

推荐默认执行包如下；所有“待确认”项必须由用户 / 全局主管在真实执行前明确确认。

| 授权项 | 推荐值 / 待确认值 | 是否阻断真实执行 | 说明 |
| --- | --- | --- | --- |
| operation | `resume` | 是 | H2 只允许 `codex-local` resume。 |
| adapter | `codex-local` | 是 | planned adapters 仍不可执行。 |
| fixture project | `/Users/yoyi/workspace/product-line/tmp/h2-real-resume-fixture` | 是 | 推荐隔离 fixture；H2.4 不创建该目录。 |
| project root | 同 fixture project | 是 | 必须为绝对路径，不能含 `..`。 |
| target cwd | 同 fixture project | 是 | 必须在 project root / allowed write roots 内。 |
| target session | 待用户指定或工作台绑定 | 是 | 不能读取 `.codex` 搜索完整 transcript 来猜测。 |
| allowed write roots | fixture project | 是 | 不默认写真实业务项目。 |
| prompt summary | `H2 real resume safe probe` | 是 | 只记录摘要，不记录完整 prompt。 |
| prompt ref | `workbench-managed:h2-real-resume-safe-probe:v1` | 是 | 真实执行前必须由执行包或产品路径生成。 |
| prompt hash | 真实执行前由完整 prompt 计算 SHA-256 | 是 | H2.4 不伪造 hash。 |
| `.codex` 范围 | 仅限 Codex CLI resume 必需最小范围 | 是 | 禁止 auth/token/secret/full transcript/provider credential。 |
| sandbox | 受控 sandbox，禁止 dangerous bypass | 是 | 禁止 `--dangerously-bypass-approvals-and-sandbox`。 |
| timeout | 建议 120000 ms | 是 | 超时写 failure reason，不自动重试。 |
| readback plan | workbench-managed last message + attempt/runtime refs | 是 | unavailable / failed / timed out 不得显示为 0 条结果。 |
| runtime log | 必须写脱敏 runtime log ref | 是 | 不把 raw stdout/stderr 当 audit。 |
| audit | 必须写用户确认、执行开始、执行结束 / 失败 | 是 | audit 不替代 runtime log。 |
| evidence path | `evidence/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md` | 是 | 真实执行结果只能写 H2 主 evidence。 |
| handoff path | `handoffs/2026-06-07-stage-h-h2-general-real-resume-productization-v1-result.md` | 是 | 真实执行结果只能写 H2 主 handoff。 |
| rollback | 执行前后 hash + diff + failure classification | 是 | 失败不能包装成通过。 |

## 5. 用户批准前必须回答

进入 H2 真实执行前，用户 / 全局主管必须明确回答：

1. 是否使用推荐 fixture：`/Users/yoyi/workspace/product-line/tmp/h2-real-resume-fixture`。
2. 是否授权创建或使用该 fixture 项目。
3. target session 是哪个；如果没有，是否先另拆 target session 绑定准备任务。
4. 是否授权本次执行真实 `codex exec resume`。
5. 是否授权本次执行触碰 `/Users/yoyi/.codex` 的 resume 必需最小范围。
6. 是否确认 allowed write roots 只限 fixture project。
7. 是否确认 prompt summary/ref/hash 规则，并接受完整 prompt 不进入任务包 / argv / shell string。
8. 是否确认 readback unavailable / failed / timed out 保持状态，不显示为 0 条结果。
9. 是否确认执行前后 hash / diff、runtime log、audit、readback 和 failure classification 写入 H2 evidence/handoff。
10. 如果 guard blocked、execution failed、timeout 或 readback 不可信，是否停止在 H2.x 修补，不进入 H3。

## 6. 停止条件

出现以下任一情况，必须停止，不得执行真实 resume：

- 未确认 target session。
- 未确认 `.codex` 最小范围。
- 未确认 allowed write roots。
- 未确认 prompt summary/ref/hash 规则。
- 需要读取 auth/token/secret/.env/keychain/OAuth/provider credential/full transcript。
- 需要使用 shell 字符串拼接或 dangerous sandbox bypass。
- 发现 duplicate queued/running attempt。
- 需要写真实业务项目而非隔离 fixture。
- runtime log、audit、readback 或 rollback 方案缺失。
- 用户 / 全局主管未明确授权真实 `codex exec resume`。

## UI 显示边界确认

本任务是否改前端：

- [x] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [ ] 改读模型摘要或状态显示。
- [ ] 改已有页面局部 UI。
- [ ] 新增入口、面板、tab、按钮或确认动作。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`

本任务不新增 UI。后续真实执行任务如果改权限弹层、运行状态或 readback UI，必须重新补 UI 显示边界确认和对应验收。

## 7. 验收

H2.4 为文档 / 授权包任务，验收为：

- 新增 H2.4 task。
- 新增 H2.4 evidence。
- 新增 H2.4 handoff。
- 同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`docs/plans/README.md`、`docs/plans/middleware-version-stage-plan-v1.md` 和 H-I 阶段计划。
- 扫描确认没有把 H2.4 写成真实执行完成。

不要求运行 `npm` / `cargo`，因为 H2.4 不改产品代码。

## 8. 下一步

H2.4 完成后，下一步只能是二选一：

- 用户明确批准真实执行授权包后，进入 H2.5 / H2-real-runner-execution 任务，执行前再次确认真实 `codex exec resume`、`.codex` 最小范围、target session 和 fixture。
- 用户未批准前，H2 保持待授权；不得进入 H3 / H4 / H5。

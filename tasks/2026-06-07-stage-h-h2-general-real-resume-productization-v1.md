# Stage H / H2 General Real Resume Productization v1

日期：2026-06-07

状态：H2.5 Phase A 已完成；H2 Phase B `mario test` 真实 resume 产品化探针已完成并回收，记录见 `evidence/2026-06-08-stage-h-h2-phase-b-mario-test-real-resume-productization-probe-v1.md` 与 `handoffs/2026-06-08-stage-h-h2-phase-b-mario-test-real-resume-productization-probe-v1-result.md`。  
用途：把 E5 Level B 的单 session 健康探针升级为工作台受控的通用真实 resume 产品能力。H2 允许真实 `codex exec resume`，但只有在本任务包的授权矩阵逐项确认后才能执行；任务包创建本身不授权执行。2026-06-08 已在用户授权的测试范围内对 `/Users/yoyi/Documents/mario test` session `019e798a-6ce5-76c3-b8ee-33bd0fda841f` 完成一次真实 Phase B probe；该结果不等于 H3、H5 或阶段 H 完成。

## 1. 权威依据

本任务包依据：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `tasks/2026-06-07-stage-h-h0-safety-boundary-and-task-package-freeze-v1.md`
- `tasks/2026-06-07-stage-h-h1-codex-local-runner-architecture-and-data-contract-v1.md`
- `evidence/2026-06-07-stage-h-h1-codex-local-runner-architecture-and-data-contract-v1.md`
- `handoffs/2026-06-07-stage-h-h1-codex-local-runner-architecture-and-data-contract-v1-result.md`
- E4 / E5 Level A / E5 Level B / E6 / G1 / G2 相关 task、evidence 和 handoff。

## 2. 前置事实

- H0 已通过全局主管复核，阶段 H 安全边界、测试项目原则、`.codex` 授权矩阵和 H1-H7 顺序已冻结。
- H1 已通过全局主管复核，`CodexLocalRunner` 架构、类型、guard、fake dry-run、结构化 argv、stdin prompt ref/hash、runtime log / audit / readback 分离边界已完成。
- E5 Level B 只证明指定 `/Users/yoyi/Documents/mario test` “总指导” session 的一次最小真实 resume 健康探针可行，不能直接证明通用产品化完成。
- H2.5 Phase A 已完成：已实现并测试可替换 runner 边界、非执行 Phase A attempt / audit / readback 分类和 duplicate guard；未真实执行 `codex exec resume`，未发送 prompt，未读写 `/Users/yoyi/.codex`。
- H2 Phase B `mario test` 真实 resume 产品化探针已完成：真实执行 `codex exec resume`，发送固定安全 probe prompt，写入 `/Users/yoyi/.codex`，readback 返回 `H2_PHASE_B_MARIO_TEST_REAL_RESUME_OK_2026_06_08`，并写入 continuation / runtime log / audit / readback 记录。
- H2 Phase B 只接受为一次受控 `mario test` 真实 resume 产品化探针完成；不接受为 H3 真实新会话、H5 项目工作流真实派发、任意项目无限制执行或阶段 H 完成。

## 3. H2 目标

H2 目标：

- 从项目、工作流、任务包、目标 session 和 H1 `CodexLocalExecutionRequest` 派生真实 resume request。
- 通过后端 `codex-local` runner 执行受控真实 `codex exec resume`。
- 使用结构化 argv 和 stdin prompt，不使用 shell 字符串拼接。
- 写入 continuation record、runtime log、audit event 和 readback result。
- 支持 success、guard blocked、user rejected、execution failed、timeout、readback unavailable、readback failed、duplicate blocked。
- 保证 readback unavailable / failed / timed out 不能显示为真实 0 条结果。

H2 不目标：

- 不做 H3 通用真实 send / 新会话。
- 不做自由聊天式会话控制器。
- 不做项目工作流真实派发集成，H5 另拆。
- 不接入 Claude Code / OpenClaw / OpenCode / OpenCode-like planned adapters。
- 不做 provider credential store、model verification、外部模型调用。
- 不做自动重试产品化；重试必须后续单独 preview 和用户确认。

## 4. 执行前授权矩阵

执行 H2 前必须由全局主管和用户逐项确认：

| 项 | 待确认值 | 默认要求 |
| --- | --- | --- |
| 操作类型 | `resume` | 只能是 `codex-local` resume |
| 测试项目 | 待确认 | 默认隔离测试项目；不能默认 `mario test` 或真实业务项目 |
| project root | 待确认 | 必须是绝对路径，不能含 `..`，必须可备份或可安全丢弃 |
| target cwd | 待确认 | 必须在 project root 或 allowed write roots 内 |
| target session | 待确认 | 必须是用户明确指定或工作台已绑定 session |
| prompt 来源 | 待确认 | 必须有 prompt summary、prompt hash、prompt ref；不得把完整 prompt 写进命令字符串 |
| allowed write roots | 待确认 | 必须逐项列明且足够窄 |
| denied paths | `.codex` 外的 secret / auth / token / .env / credential / full transcript 等 | 永远禁止作为项目写入目标或 prompt source |
| `/Users/yoyi/.codex` 读写范围 | 待确认 | 仅限 resume 必需的 Codex 自身最小读写；不得读取 auth/token/secret/完整 transcript |
| sandbox | 待确认 | 禁止 `--dangerously-bypass-approvals-and-sandbox` |
| timeout | 待确认 | 必须设置；超时要写 failure reason |
| readback plan | 待确认 | 必须声明 expected sources、unavailable behavior 和 trust policy |
| evidence path | 待确认 | 必须写入 product-line evidence/handoff |
| rollback / 降级 | 待确认 | 至少包含项目文件 hash 前后对比、runtime log、audit 和 readback 分类 |

未确认任一项时，H2 只能停在 `blocked_waiting_authorization`，不得执行真实 resume。

## 5. 建议测试项目

默认建议新建或使用隔离测试项目，而不是默认使用 `mario test`：

- 路径候选：由全局主管执行前确认，例如 `/Users/yoyi/workspace/product-line/tmp/h2-real-resume-fixture` 或用户指定的独立目录。
- 目标 session：必须由用户明确指定或通过工作台绑定到该隔离项目。
- 项目文件：建议最少包含 `README.md` 和一个小型文本 / 代码文件，方便 hash 前后对比。
- 任务目标：优先使用低风险、可验证、可回滚的小改动或 no-op safe probe。

如果用户坚持复用 `mario test`：

- 必须单独确认它不再只是 E5 Level B 历史健康探针。
- 必须重新记录文件 hash、允许写入路径、目标 session 和回滚策略。
- 不能把复用 `mario test` 的成功解释为所有项目通用成功。

## 6. 后端实现要求

必须实现或收敛：

- H2 request builder：从 project / workflow / task package / target session / memory packet 生成 H1 request。
- H2 guard：复用并扩展 `inspect_codex_local_execution_guard`，加入真实执行所需的 execution authorization。
- H2 real runner：结构化 argv + stdin prompt；不得使用 `sh -c` 或拼接 shell 字符串。
- H2 attempt store：写 continuation attempt，不把 stdout/stderr/raw transcript 直接塞进 audit。
- H2 runtime log：记录脱敏 status、duration、exit code、timeout、failure category 和 refs。
- H2 audit：记录用户确认、执行开始、执行结束、失败/超时/拒绝，不替代 runtime log。
- H2 readback：读取允许范围内的 last message / workbench-managed output；不可用时 status 必须是 `readback_unavailable`，`result_count` 必须为 null / None。
- H2 duplicate guard：同一 continuation / session / work item 有 queued / running attempt 时阻断。

必须禁止：

- prompt 进入 argv。
- prompt 进入 shell string。
- `--dangerously-bypass-approvals-and-sandbox`。
- 读取 auth、token、`.env`、secret、keychain、OAuth、provider credential。
- 读取完整 transcript / rollout 作为默认 readback。
- 把 readback unavailable / failed 写成 0 条结果。
- 前端直接调用 CLI 或直接拼命令。

## 7. UI 显示边界确认

本任务预计涉及 UI：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper，但不新增裸执行按钮。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [ ] 新增一级入口、主导航或右侧入口。

必须遵守：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

允许显示：

- resume request preview。
- 权限确认弹层：目标 session、project root、cwd、allowed write roots、prompt summary、task memory packet 摘要、风险、readback 预期、可能写入 `.codex` 的说明。
- 执行状态：waiting authorization、queued、running、succeeded、failed、timed out、readback unavailable、readback failed、duplicate blocked。
- runtime log / audit / readback refs 的用户可理解摘要。

禁止显示：

- “Codex 已收到任务”，除非真实 runner 已成功启动并写入 attempt。
- “结果 0 条”，如果实际是 readback unavailable / failed。
- planned adapters 可执行。
- provider credential / model verified。
- 原始 secret、raw transcript、完整 prompt、完整 stdout/stderr。
- 绕过权限的自由发送按钮。

UI 验收：

- 如改可见 UI，必须补离线交互测试。
- 如改真实执行 UI / 权限弹层 / 运行状态，必须安排真实 Tauri 或明确记录 screenshot incomplete；普通浏览器 smoke 不能冒充真实 Tauri 验收。

## 8. 验收要求

代码验证建议：

- `cargo test --lib codex_local`
- `cargo test --lib session_continuation`
- `cargo test --lib runtime_log`
- `cargo test --lib runtime_session_attention`
- `cargo test --lib`
- `rustfmt --check ...`
- 如改前端：`npm run typecheck`
- 如改 UI：`npm run test:offline-interaction`
- 如改 build 相关：`npm run build`

真实执行验收必须包含：

- 执行前项目文件 hash / 目录快照。
- 真实 `codex exec resume` exit code。
- workbench-managed last message / readback result。
- `/Users/yoyi/.codex` 读写范围说明。
- continuation record。
- runtime log。
- audit event。
- readback result。
- 失败 / 超时 / readback unavailable 分类。
- 执行后项目文件 hash / 差异说明。

固定扫描：

- 不得出现 H3 已完成、H5 已完成、阶段 H 已完成。
- 不得出现 planned adapters 已接入、provider credential 已验证。
- 不得出现未分类的真实 `codex exec resume` 成功宣称。
- 不得把 E5 Level B 或 H2 单次成功写成通用 send / resume 全完成。

## 9. 接受范围

H2 完成后可接受为：

- 工作台受控通用真实 resume 最小产品能力完成。
- 至少一个隔离测试项目的真实 resume 成功或明确失败分类完成。
- continuation / runtime log / audit / readback 真实链路可追溯。
- duplicate guard、timeout、user rejection、guard blocked、readback unavailable / failed 至少有测试或 fixture 覆盖。

H2 不接受为：

- H3 通用真实 send / 新会话完成。
- H5 项目工作流真实派发完成。
- H 阶段完成。
- planned adapters 真实接入。
- provider credential / model verification。
- 自动重试产品化。
- 完整多 agent / 多模型协作抽象。

## 10. 回交要求

完成 H2 后必须新增：

- `evidence/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md`
- `handoffs/2026-06-07-stage-h-h2-general-real-resume-productization-v1-result.md`

必须同步：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`

回交中文必须写清：

- 是否真实执行了 `codex exec resume`。
- 是否读写了 `/Users/yoyi/.codex`，范围是什么。
- 是否发送了 prompt。
- 哪个测试项目 / session 被使用。
- 哪些文件发生变化或保持不变。
- runtime log / audit / readback 的记录位置。
- H2 接受范围和不接受范围。
- 下一步是否可以进入 H3 / H4，或是否需要 H2.x 修补。

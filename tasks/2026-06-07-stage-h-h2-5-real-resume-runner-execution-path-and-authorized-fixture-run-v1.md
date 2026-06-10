# Stage H / H2.5 Real Resume Runner Execution Path And Authorized Fixture Run v1

日期：2026-06-07

状态：Phase A 已完成；Phase B 未授权、未执行。  
用途：把 H2.0-H2.4 的 real resume 预检、授权矩阵、request builder、guard bridge 和执行授权包推进到 H2 主任务的真实 runner 产品路径。H2.5 分为两个明确阶段：先实现可测试但不执行真实 Codex 的产品代码路径；再在用户 / 全局主管再次明确授权后，才允许对隔离 fixture 执行一次真实 `codex exec resume`。

## 1. 权威依据

本任务包依据：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `tasks/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md`
- `tasks/2026-06-07-stage-h-h2-4-real-resume-execution-authorization-and-fixture-freeze-v1.md`
- `evidence/2026-06-07-stage-h-h2-4-real-resume-execution-authorization-and-fixture-freeze-v1.md`
- `handoffs/2026-06-07-stage-h-h2-4-real-resume-execution-authorization-and-fixture-freeze-v1-result.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

## 2. 当前事实

- H1 只有 `FakeCodexLocalRunner` dry-run runner、结构化 argv plan 和 guard；没有真实 `codex exec resume` runner。
- E5 Level A 已有 `session-continuations.v1.json`、confirmation、stub attempt 和 readback unavailable 边界。
- H2.0-H2.3 已能把授权矩阵完整的 continuation 构建成 H1 `CodexLocalExecutionRequest` 并跑 guard，但仍只返回 `complete_but_not_executed`。
- H2.4 已冻结执行授权包、推荐 fixture、`.codex` 最小范围、prompt summary/ref/hash、allowed write roots、readback plan、runtime log、audit、evidence 和 rollback 要求，但未创建 fixture、未选择 target session、未授权真实执行。
- H2.5 Phase A 已完成：已实现并测试非执行产品路径、可替换 runner 边界、attempt / audit / readback / duplicate guard 链路。
- H2.5 Phase B 仍未授权、未执行；真实 `codex exec resume`、prompt 发送和 `/Users/yoyi/.codex` 最小读写必须等用户 / 全局主管再次明确授权。

## 3. 目标

H2.5 目标：

- 新增受控真实 resume runner 产品路径，但不使用 shell 字符串拼接。
- 将 H2.3 生成的 `CodexLocalExecutionRequest` 作为唯一真实执行输入。
- 在真实执行前写入 running / started attempt 和 audit，结束后写 succeeded / failed / timed_out / readback_unavailable / readback_failed 分类。
- 将 continuation record、attempt、runtime log、audit event 和 readback result 形成可追踪链路。
- 保持 duplicate queued/running attempt 阻断。
- 保证 readback unavailable / failed / timed out 的 `result_count` 为 `null` / `None`，不能显示为真实 0 条结果。
- 支持 Phase A 的非执行 runner adapter / fake process 测试，覆盖成功、guard blocked、user rejected、timeout、execution failed、readback unavailable、duplicate blocked。
- 支持 Phase B 在明确授权后，对隔离 fixture 做一次真实 `codex exec resume`。

H2.5 不目标：

- 不做 H3 通用真实 send / 新会话。
- 不做 H5 项目工作流真实派发集成。
- 不做自动重试、取消、恢复产品化；这些属于 H4 或后续任务。
- 不接 planned adapters 真实执行。
- 不读取 auth/token/secret/.env/keychain/OAuth/provider credential。
- 不读取完整 transcript 作为默认 readback。
- 不新增自由聊天式 send/resume 控制器。

## 4. 分段执行规则

### Phase A：非执行产品路径实现

Phase A 可在不触碰 `/Users/yoyi/.codex`、不发送 prompt、不执行真实 `codex exec resume` 的前提下开发。

必须实现：

- `CodexLocalRunner` 增加真实 runner trait / implementation 边界，真实执行实现必须可由测试 runner 替换。
- 结构化 `Command` / process runner：program + argv + stdin prompt，不使用 `sh -c`，不拼 shell string。
- H2 real attempt command：从 `CodexLocalExecutionRequest` 派生 argv，prompt 通过 stdin / prompt ref 边界传递。
- continuation store 写入真实执行 attempt 生命周期：queued / running / succeeded / failed / timed_out / readback_unavailable / readback_failed / blocked_by_guard / user_rejected / duplicate_blocked。
- audit event：用户确认、执行开始、执行结束、失败 / 超时 / readback 分类。
- runtime log：脱敏 status、duration、exit code、failure category、source refs、audit refs。
- readback result：只读 workbench-managed last message 或明确授权来源；不可用时保持 unavailable，不写 0 条。
- duplicate guard：同 continuation / session / work item 存在 queued / running / running_real attempt 时阻断。
- Rust 单测覆盖 guard、command safety、attempt state、audit refs、runtime log refs、readback unavailable、duplicate blocking、timeout / failed 分类。

Phase A 禁止：

- 执行真实 `codex exec`。
- 执行真实 `codex exec resume`。
- 发送真实 prompt。
- 读写 `/Users/yoyi/.codex`。
- 创建真实 fixture session。
- 启动 Tauri / GUI / 截图。

### Phase B：授权后隔离 fixture 真实执行

Phase B 只有在用户 / 全局主管再次明确批准后才能执行。

执行前必须再次确认：

- 是否使用推荐 fixture：`/Users/yoyi/workspace/product-line/tmp/h2-real-resume-fixture`。
- 是否授权创建或使用该 fixture。
- target session 是哪个；若没有 target session，必须先另拆 session 绑定 / 创建准备任务。
- 是否授权真实 `codex exec resume`。
- 是否授权 Codex CLI resume 必需的 `/Users/yoyi/.codex` 最小读写。
- allowed write roots 是否只限 fixture project。
- prompt summary/ref/hash 和完整 prompt 不进入 argv / shell string / evidence。
- timeout、sandbox、readback plan、runtime log、audit、evidence、rollback 和 failure classification。

Phase B 成功也只接受为 H2 最小真实 resume fixture run 完成，不接受为 H3、H5 或阶段 H 完成。

## 5. 推荐执行包

默认推荐：

- fixture project：`/Users/yoyi/workspace/product-line/tmp/h2-real-resume-fixture`
- operation：`resume`
- adapter：`codex-local`
- allowed write roots：仅 fixture project
- sandbox：受控 sandbox，禁止 dangerous bypass
- timeout：120000 ms
- prompt summary：`H2 real resume safe probe`
- prompt ref：`workbench-managed:h2-real-resume-safe-probe:v1`
- readback：workbench-managed last message + attempt / runtime refs
- evidence：`evidence/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md`
- handoff：`handoffs/2026-06-07-stage-h-h2-general-real-resume-productization-v1-result.md`

## UI 显示边界确认

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

- H2 real resume request preview。
- 权限弹层：project、workflow、node、target session、cwd、allowed write roots、sandbox、prompt summary/hash/ref、任务记忆包摘要、`.codex` 最小副作用说明、readback plan、timeout、duplicate guard、failure handling、runtime log refs、audit refs。
- 执行状态：waiting authorization、queued、running、succeeded、failed、timed out、readback unavailable、readback failed、duplicate blocked、user rejected。
- runtime log / audit / readback 的用户可理解摘要和 refs。

禁止显示：

- 未执行前显示“Codex 已收到任务”。
- readback unavailable / failed / timed out 显示为“真实 0 条结果”。
- 完整 prompt、raw transcript、secret、完整 stdout/stderr。
- planned adapters 可执行。
- provider credential / model verified。
- 绕过权限的自由发送按钮。

如改可见 UI，必须补离线交互测试。真实执行 UI / 权限弹层 / 运行状态若进入 Phase B，必须安排真实 Tauri 或明确记录 screenshot incomplete。

## 6. 验收要求

Phase A 验收建议：

- `cargo test --lib codex_local`
- `cargo test --lib session_continuation`
- `cargo test --lib runtime_log`
- `cargo test --lib runtime_session_attention`
- `cargo test --lib`
- `rustfmt --check ...`
- 如改前端：`npm run typecheck`
- 如改 UI：`npm run test:offline-interaction`
- 如改 build 相关：`npm run build`
- 禁止文案扫描：不能出现 H2 / H3 / H5 / 阶段 H 已完成，不能出现 planned adapters 已接入或 provider credential 已验证。

Phase B 真实执行验收必须包含：

- 执行前 fixture 文件 hash / 目录快照。
- 真实 `codex exec resume` exit code。
- prompt_sent / real_codex_executed / writes_codex_home 的真实值。
- `/Users/yoyi/.codex` 最小读写范围说明。
- continuation record。
- attempt record。
- runtime log。
- audit event。
- readback result 或 readback unavailable / failed 分类。
- 执行后 fixture 文件 hash / diff。
- failure / timeout / duplicate / user rejection / guard blocked 的分类证据或测试覆盖。

## 7. 接受范围

H2.5 Phase A 完成后可接受为：

- H2 real resume runner 产品路径实现完成，但未真实执行。
- command safety、attempt lifecycle、runtime log、audit、readback 和 duplicate guard 的非执行测试覆盖完成。

H2.5 Phase B 完成后可接受为：

- 经用户 / 全局主管明确授权后，隔离 fixture 的一次 H2 真实 resume 执行完成或明确失败分类完成。
- continuation / runtime log / audit / readback 真实链路可追溯。

H2.5 不接受为：

- H3 通用真实 send / 新会话完成。
- H5 项目工作流真实派发完成。
- H 阶段完成。
- planned adapters 真实接入。
- provider credential / model verification。
- 自动重试产品化。
- 完整多 agent / 多模型协作抽象。

## 8. 回交要求

完成 H2.5 后必须新增或更新：

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

回交必须写清：

- 是否真实执行了 `codex exec resume`。
- 是否读写了 `/Users/yoyi/.codex`，范围是什么。
- 是否发送了 prompt。
- 使用的 fixture / project / session。
- 哪些文件发生变化或保持不变。
- runtime log / audit / readback 的记录位置。
- H2.5 接受范围和不接受范围。
- 是否可以继续 H2.x 修补、H3 或必须等待授权。

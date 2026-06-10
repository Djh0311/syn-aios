# Stage H / H2.6 Phase B Readiness, Fixture Session Binding, And Runtime Log Hardening v1

日期：2026-06-07

状态：已完成；不授权真实 `codex exec` / `codex exec resume`。  
用途：在 H2.5 Phase A 非执行产品路径完成后，补齐 H2.5 Phase B 真实 resume 前的可复核前置条件，避免在缺 fixture、缺 target session、缺显式 runtime log 写入口径时请求或执行真实 Codex。

## 1. 权威依据

本任务包依据：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `tasks/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md`
- `tasks/2026-06-07-stage-h-h2-5-real-resume-runner-execution-path-and-authorized-fixture-run-v1.md`
- `evidence/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md`
- `handoffs/2026-06-07-stage-h-h2-general-real-resume-productization-v1-result.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

## 2. 当前事实

- H2.5 Phase A 已完成，只接受为非执行 runner 产品路径、attempt / audit / readback 分类和 duplicate guard 完成。
- H2.5 Phase B 未授权、未执行；真实 `codex exec resume`、prompt 发送和 `/Users/yoyi/.codex` 最小读写仍必须等用户 / 全局主管再次明确授权。
- 当前没有实际创建推荐 fixture：`/Users/yoyi/workspace/product-line/tmp/h2-real-resume-fixture`。
- 当前没有可用于 H2 Phase B 的已确认 target session。
- 当前没有可作为 H2 Phase B 证据的真实 workbench continuation sidecar 实例。
- 当前 Phase A 的 runtime log 口径主要是 `runtime_log_ref` / 派生摘要；尚未证明 H2 attempt 会显式写入 runtime log sidecar。

全局主管判断：

- 不能直接进入 Phase B。
- 不能直接进入 H3 通用真实 send / 新会话。
- H2.6 的核心是把 Phase B 前置条件变成可验证对象，而不是抢跑真实执行。

## 3. 目标

H2.6 目标：

- 明确 H2 Phase B 真实 resume 进入授权请求前必须满足的状态机。
- 准备或冻结隔离 fixture 的项目目录、文件 hash / rollback / allowed write roots 规则。
- 准备 target session 绑定策略：用户明确提供 existing session，或停在 `blocked_waiting_target_session`。
- 明确不使用 H3/new session 绕过 H2 Phase B；如果没有 existing session，只能另拆 H3 或 target session 绑定准备任务。
- 将 H2 attempt 的 runtime log 从仅有 ref / 派生摘要推进到显式 sidecar 写入或明确冻结为阻断项。
- 输出 Phase B 授权请求草案，但不触发真实 `codex exec resume`。

H2.6 不目标：

- 不执行真实 `codex exec`。
- 不执行真实 `codex exec resume`。
- 不发送真实 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 auth、token、secret、`.env`、keychain、OAuth、provider credential 或完整 transcript。
- 不接 planned adapters。
- 不做 H3 通用真实 send / 新会话。
- 不做 H5 项目工作流真实派发。
- 不做自动重试、取消或恢复产品化。

## 4. 必须解决的前置项

### 4.1 Fixture readiness

必须明确：

- 是否使用推荐 fixture：`/Users/yoyi/workspace/product-line/tmp/h2-real-resume-fixture`。
- 是否允许创建该 fixture。
- fixture 内初始文件清单。
- 执行前 hash / 目录快照方案。
- rollback / cleanup 方案。
- allowed write roots 是否仅限该 fixture。
- denied paths 是否包含 secret、credential、`.env`、完整 transcript 和用户真实业务目录。

如果 fixture 未确认，本任务输出：

```text
h2_phase_b_readiness = blocked_waiting_fixture
```

### 4.2 Target session readiness

必须明确：

- target session 由用户明确提供，或由工作台已有绑定证明。
- target session 对应 project root / cwd / workflow / node binding。
- 不通过读取 `/Users/yoyi/.codex` 搜索完整 transcript 来猜测 target session。
- 不把 E5 Level B 的 mario test session 默认复用为 H2 fixture session。

如果 target session 未确认，本任务输出：

```text
h2_phase_b_readiness = blocked_waiting_target_session
```

### 4.3 Runtime log hardening

H2 Phase B 前默认要求：

- H2 attempt 必须有显式 runtime log sidecar 写入能力，或全局主管明确接受派生 runtime log 作为 Phase B 最小证据。
- runtime log 不能替代 audit event。
- audit event 不能替代 runtime log。
- runtime log 不得包含完整 prompt、raw transcript、secret、完整 stdout/stderr。
- readback unavailable / failed / timed out 不能写成真实 0 条结果。

如果仍只有 `runtime_log_ref` / 派生摘要，本任务必须明确写出：

```text
h2_phase_b_readiness = blocked_waiting_runtime_log_writer
```

除非全局主管单独冻结为：

```text
h2_phase_b_readiness = authorization_ready_with_derived_runtime_log_only
```

### 4.4 Permission envelope readiness

必须准备 Phase B 授权请求草案，包含：

- operation: `resume`
- adapter: `codex-local`
- project root
- target cwd
- target session
- allowed write roots
- denied paths
- sandbox
- timeout
- prompt summary
- prompt hash
- prompt ref
- task memory packet summary
- `.codex` 最小副作用说明
- readback plan
- runtime log plan
- audit plan
- evidence / handoff path
- rollback / failure classification

授权请求草案只能作为待用户确认材料；不能解释为用户已批准真实执行。

## 5. UI 显示边界确认

本任务可能涉及 UI：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 可改前端类型 / Tauri wrapper，但不新增裸执行按钮。
- [x] 可改读模型摘要或状态显示。
- [x] 可改已有页面局部 UI。
- [ ] 不新增一级入口、主导航或右侧入口。

必须遵守：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

允许显示：

- Phase B readiness status。
- 缺失项：fixture / target session / runtime log writer / permission envelope / rollback。
- authorization preview。
- blocked reason。
- runtime log 与 audit 的区别。

禁止显示：

- “Codex 已收到任务”。
- “真实 resume 已执行”。
- “prompt 已发送”。
- “H2 已完成”。
- readback unavailable / failed / timed out 显示为真实 0 条结果。
- planned adapters 可执行。
- provider credential / model verified。
- 完整 prompt、raw transcript、secret、完整 stdout/stderr。
- 绕过权限的自由发送 / resume 按钮。

如果改可见 UI，必须补离线交互测试。真实 Tauri 截图验收不属于本任务默认范围，若未做必须写为未完成。

## 6. 建议实现范围

允许：

- 新增或补强 H2 Phase B readiness 读模型。
- 新增或补强 H2 Phase B authorization preview。
- 新增或补强 runtime log sidecar append / write helper，前提是不触发真实 Codex。
- 创建 workbench 自有 fixture metadata 或 readiness record，前提是写入范围仅限 `product-line` workspace。
- 在没有 target session 时输出 blocked，而不是猜测。
- 更新 evidence / handoff / 权威入口。

禁止：

- 直接执行 Phase B。
- 通过 shell 字符串拼接构造 Codex 命令。
- 让前端直接调用 CLI。
- 读取 `/Users/yoyi/.codex` 来发现 session。
- 将 mario test E5 Level B 结果当成 H2 通用证据。
- 自动创建新 Codex session 来绕过 target session 缺失。

## 7. 验收要求

如果本任务只写文档 / readiness 包：

- 固定扫描当前入口不应宣称 H2 / H3 / H 阶段完成。
- 固定扫描不应宣称真实 resume 已执行、prompt 已发送或 `.codex` 已读写。

如果改后端：

- `cargo test --lib codex_local`
- `cargo test --lib session_continuation`
- `cargo test --lib runtime_log`
- `cargo test --lib`
- `rustfmt --check` 指定修改文件

如果改前端 / UI：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`

必须新增或更新：

- `evidence/2026-06-07-stage-h-h2-6-phase-b-readiness-fixture-session-and-runtime-log-hardening-v1.md`
- `handoffs/2026-06-07-stage-h-h2-6-phase-b-readiness-fixture-session-and-runtime-log-hardening-v1-result.md`

## 8. 接受范围

H2.6 完成后可接受为：

- H2 Phase B 真实执行前置条件闭合或阻断原因冻结完成。
- fixture readiness、target session readiness、runtime log hardening 和 permission envelope readiness 已可被全局主管复核。
- 如果缺 target session，则明确停在 `blocked_waiting_target_session`。
- 如果缺显式 runtime log writer，则明确停在 `blocked_waiting_runtime_log_writer`，或由全局主管明确接受派生 runtime log 最小证据。

H2.6 不接受为：

- H2 通用真实 resume 产品化完成。
- H2.5 Phase B 已授权或已执行。
- 真实 `codex exec resume` 已执行。
- prompt 已发送。
- `/Users/yoyi/.codex` 已读写。
- H3 通用真实 send / 新会话完成。
- H5 项目工作流真实派发完成。
- 阶段 H 完成。
- planned adapters 真实接入。
- provider credential / model verification。

## 10. 回收记录

本任务已回收：

- `evidence/2026-06-07-stage-h-h2-6-phase-b-readiness-fixture-session-and-runtime-log-hardening-v1.md`
- `handoffs/2026-06-07-stage-h-h2-6-phase-b-readiness-fixture-session-and-runtime-log-hardening-v1-result.md`

最终状态：

```text
h2_phase_b_readiness = blocked_waiting_fixture_and_target_session
runtime_log_writer = explicit_sidecar_writer_ready
phase_b_authorization_request = not_ready
```

## 9. 回交要求

回交必须中文写清：

- 是否创建 fixture。
- 是否确认 target session。
- 是否读写 `/Users/yoyi/.codex`。
- 是否执行真实 `codex exec` / `codex exec resume`。
- 是否发送 prompt。
- runtime log 是显式 sidecar 写入，还是仅 ref / 派生摘要。
- readiness 最终状态。
- 是否可以进入 Phase B 授权请求。
- 为什么不能进入 H3 / H5。

# Unified Product Command Routing PCR9A Product Command Phase B Bridge v1

日期：2026-06-09

状态：已通过主管复核，带 P2 后续注意项。

PCR9A 是 PCR9 的执行前 P1 修补任务。开发线预检确认：当前代码只有统一 Product Command 的 Phase A no-op API，真实 Phase B runner 仍停留在 controlled session continuation 层，不能作为 PCR9 完成证据。PCR9A 的目标是补齐统一 product command 命名空间内的 Phase B 桥，让后续 PCR9 Level B 真实探针能够从 product command 链路触发并写回 product command attempt / read model。

PCR9A 不执行真实 Codex，不发送真实 prompt，不读写 `/Users/yoyi/.codex`。真实执行仍保留给 PCR9，且必须等待用户二次授权。

## 0. 先说薄弱点

- 这是把真实执行能力挂回统一 product command 命名空间的关键桥，风险高于普通 read model 修补。
- 底层 `run_controlled_session_continuation_real_resume_phase_b` 已能调用真实 runner，但它只写 continuation attempt；直接用它作为 PCR9 完成证据会绕过 product command attempt。
- Phase B 桥必须记录 product command 层 attempt，否则 UI / audit / readback 仍无法证明真实执行是由统一 product command 发起。
- 不能为了测试而执行真实 `codex exec` / `codex exec resume`。测试必须使用 fake Phase B runner 或 ignored real probe。
- 不能把 prompt body 持久化到 product command sidecar、runtime log、audit、普通 evidence 或 UI；只记录 prompt summary / ref / sha256。

## 1. 前置依据

- PCR8 已完成，Level A checkpoint 通过。
- PCR9 授权准备任务包已通过复核，但执行前阻断为缺少统一 product command Level B execute API。
- 当前 product command Tauri / TS wrapper 已有：
  - `preview_real_execution_product_command`
  - `prepare_real_execution_product_command`
  - `record_real_execution_product_command_decision`
  - `confirm_real_execution_product_command`
  - `run_real_execution_product_command_phase_a`
- 当前底层真实 runner 入口为：
  - `run_controlled_session_continuation_real_resume_phase_b`
- PCR9A 必须新增 product command 层 Phase B，不得把底层 continuation Phase B 直接当完成路径。

## 2. 本任务目标

PCR9A 必须完成：

1. 新增 Rust DTO：
   - `RunRealExecutionProductCommandPhaseBInput`
   - `RealExecutionProductCommandPhaseBOutput`
2. 新增后端服务函数：
   - `run_real_execution_product_command_phase_b_at(...)`
   - 内部测试可用的 fake runner 变体或等价 seam。
3. 新增 Tauri command：
   - `run_real_execution_product_command_phase_b`
4. 新增 TS 类型和 wrapper：
   - `RunRealExecutionProductCommandPhaseBInput`
   - `RealExecutionProductCommandPhaseBOutput`
   - `runRealExecutionProductCommandPhaseB(...)`
5. Phase B 桥必须走 product command gate：
   - 校验 product command exists。
   - 校验 store revision。
   - 校验 user decision 已 approved。
   - 校验 `confirmed_by == "user"`。
   - 校验 prompt sha256 与 prompt body 一致。
   - 校验 adapter 为 `codex-local`。
   - 校验 operation 为 `resume`。
   - 校验 continuation id / session id / project root 绑定。
   - 校验 duplicate / guard / diagnostics / stale memory / readback boundary。
6. Phase B 桥必须调用 controlled session continuation Phase B，但 product command 层必须写自己的 attempt：
   - `prompt_sent`
   - `real_codex_executed`
   - `writes_codex_home`
   - `writes_project_files`
   - `continuation_attempt_id`
   - `runtime_log_ref`
   - `audit_refs`
   - `readback_summary`
   - `failure_reason`
   - `warnings`
7. Phase B 桥不得持久化 prompt body。
8. 前端普通 UI 不新增真实执行按钮，不接 wrapper 调用；只同步类型和 wrapper。
9. 更新 PCR9A 任务包执行结果为“待主管复核”，不要同步权威入口。

## 3. 本任务不做

- 不执行 PCR9 B1/B2 真实探针。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送真实 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 `.codex/plugins/cache`。
- 不读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 不启动 Browser/Chrome/Tauri/Vite/截图工具。
- 不新增自由聊天输入框、通用终端、任意项目执行按钮。
- 不修改 `workflow-state.v0.json` 顶层结构。
- 不同步 `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`。
- 不把 PCR9A 说成 PCR9 真实探针完成。

## 4. 允许修改范围

预计允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`，仅允许暴露 fake-runner seam 或最小 bridge helper，不允许重构真实 runner。
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`，仅当类型 fixture 必需。
- 本任务包。

不允许修改：

- 真实 UI 入口布局。
- 普通产品页面新增执行按钮。
- authority/current 入口文档。
- `.codex` 或插件缓存。

## 5. 输入合同建议

`RunRealExecutionProductCommandPhaseBInput` 至少包含：

```text
product_command_id
expected_product_command_store_revision
expected_session_continuation_store_revision
actor_role
execution_decision
authorization
prompt_body
requested_at
```

说明：

- `authorization` 可以复用 `H2RealResumeAuthorizationMatrix`，但 product command 层必须额外校验它与 product command preview / request 的 project、session、sandbox、prompt summary、prompt ref、prompt sha256 一致。
- `prompt_body` 只用于运行时 stdin，不得写入 sidecar / runtime / audit / ordinary evidence。
- 如果 prompt sha256 不一致，必须阻断，不得调用 runner。

## 6. 输出合同建议

`RealExecutionProductCommandPhaseBOutput` 至少包含：

```text
status
product_command_id
product_command_attempt
read_model
product_command_store_revision
product_command_sidecar_path
continuation_id
continuation_attempt_id
session_continuation_store_revision
runtime_log_ref
audit_refs
readback_summary
runner_call_allowed
prompt_sent
real_codex_executed
writes_codex_home
writes_project_files
writes_product_command_sidecar
writes_continuation_sidecar
writes_runtime_log
blocked_reasons
warnings
```

建议与 `RealExecutionProductCommandPhaseAOutput` 保持字段结构相近，方便 UI / tests / evidence 复用。

## 7. 关键实现要求

### 7.1 Product command 层校验

真实 runner 前必须先完成：

- 找到 `product_command_id` 对应 request / preview。
- 存在 approved decision。
- 高影响真实执行必须由用户确认，项目主管确认不能替代用户确认。
- 不允许 rejected / expired / duplicate / stale / guard blocked / diagnostics blocked。
- 不允许 planned adapter。
- 不允许 operation 非 `resume`。
- 不允许 prompt hash mismatch。
- 不允许 source_kind 绕到 legacy/H5/direct CLI 完成证据。

### 7.2 Continuation Phase B 调用

允许复用 `session_continuation_store::run_real_resume_phase_b` 的授权、guard、runtime log 和 audit 写入能力。

但 PCR9A 必须保证：

- product command 层创建自己的 attempt。
- product command attempt 引用 continuation attempt。
- product command attempt 写入真实执行 flags。
- read model 能看到 Phase B attempt。
- failure / readback unavailable 保持 `result_count = null`，不能显示为 0。

### 7.3 Fake runner seam

必须提供不执行真实 Codex 的测试路径：

- 可以把 `run_real_resume_phase_b_with_runner` 调整为 `pub(crate)` 或新增更窄 helper。
- 只允许 Rust 单元测试使用 fake runner。
- 生产 Tauri command 仍使用真实 runner，但测试不调用真实 runner。

### 7.4 Prompt 安全

必须证明：

- prompt body 不进入 product command sidecar。
- prompt body 不进入 continuation sidecar。
- prompt body 不进入 runtime log。
- prompt body 不进入 audit event。
- prompt body 不进入普通 UI / evidence。
- 只保留 prompt summary / ref / sha256。

## 8. 验收标准

PCR9A 可接受为完成，当且仅当：

- 新 product command Phase B API / Tauri wrapper / TS wrapper 存在。
- fake runner 单测能证明 product command Phase B 成功路径写入 product attempt，并设置：
  - `prompt_sent=true`
  - `real_codex_executed=true`
  - `writes_codex_home=true`
  - `writes_project_files` 按 sandbox/allowed write roots 正确表达
- 阻断测试覆盖：
  - no approved user decision
  - non-user confirmer
  - prompt hash mismatch
  - duplicate attempt
  - continuation Phase B blocked
  - unsupported operation / planned adapter
- prompt body 不持久化测试通过。
- 普通 UI wrapper 扫描无命中；没有新增执行按钮。
- 真实 runner 测试保持 ignored 或需要显式 env/用户授权。
- PCR9 任务包仍保持“真实执行待用户二次授权”。

## 9. 验证命令

至少运行：

```text
cargo test --lib real_execution_command
cargo test --lib session_continuation
cargo test --lib codex_local_runner
cargo test --lib runtime_log
cargo test --lib diagnostic
cargo test --lib workflow_authorization
cargo test --lib
cargo fmt -- --check
npm run typecheck
npm run test:offline-interaction
npm run build
```

扫描：

```text
rg -n "runRealExecutionProductCommandPhaseB|run_real_execution_product_command_phase_b" prototypes/productized-desktop-shell/src/App.tsx prototypes/productized-desktop-shell/src/views prototypes/productized-desktop-shell/src/components
rg -n "Command::new\\(\"codex\"\\)|codex exec|codex exec resume|prompt_body" prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs
rg -n "\\.codex/plugins/cache|auth/token|secret|keychain|OAuth|provider credential|full transcript|rollout" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src tasks
```

扫描命中必须分类。`prompt_body` 在输入 DTO / runtime-only runner 参数中可接受，但不得出现在持久化 store 字段、audit 文案、runtime log 文案、普通 UI 文案中。

## 10. 分线职责

### 主管线

- 创建任务包。
- 派发开发线实现。
- 不执行真实 Codex。
- 收开发线回交后派复核线只读审查。
- 决定是否允许回到 PCR9 授权等待。

### 开发线

- 实现 PCR9A 代码和测试。
- 不启动真实 runner。
- 不读写 `.codex`。
- 不同步权威入口。
- 完成后任务包状态写“待主管复核”。

### 复核线

- 只读复核 PCR9A 结果。
- 重点看真实执行入口是否只在 product command wrapper 下，普通 UI 是否未接执行按钮，prompt body 是否未持久化。

## 11. 完成后口径

PCR9A 完成后可以说：

- 统一 product command Phase B 桥已具备可测试产品路径。
- PCR9 真实探针的代码前置阻断已解除。

仍不能说：

- PCR9 B1/B2 真实探针已完成。
- 用户已授权真实执行。
- 任意项目自由执行完成。
- planned adapters 真实接入。
- provider/model verification 完成。
- 自动 retry / stop / restart 完成。

## 12. 开发线执行结果

状态：已完成，已由主管线和只读复核线复核通过，带 P2 后续注意项。

本轮实现：

- 新增 product command 命名空间 Phase B 后端 API：`run_real_execution_product_command_phase_b_at` / `run_real_execution_product_command_phase_b_with_runner` / Tauri command `run_real_execution_product_command_phase_b`。
- 新增 TS type / wrapper：`RunRealExecutionProductCommandPhaseBInput`、`RealExecutionProductCommandPhaseBOutput`、`runRealExecutionProductCommandPhaseB`。
- Product command request 增加 `sandbox` 字段，Phase A continuation preview 改为继承 product request sandbox，供 Phase B 授权矩阵绑定校验。
- Phase B product gate 校验 command / store revision / approved user decision / adapter / operation / prompt sha256 / continuation session-project-sandbox-prompt 绑定 / duplicate / preview guard-diagnostics-stale boundary。
- Phase B bridge 调用 controlled session continuation Phase B，并把 continuation attempt 映射回 product command attempt，记录 continuation/runtime/audit/readback refs 与真实执行 flags。
- 测试 seam：`session_continuation_store::run_real_resume_phase_b_with_runner` 暴露为 `pub(crate)`；PCR9A 单测只使用 fake / panic runner，不启动真实 Codex。
- Prompt body 只作为 runtime input 进入底层 Phase B；新增单测断言 product command sidecar、continuation sidecar、runtime log sidecar 均不包含 runtime prompt 文本。
- 普通 UI 未新增真实执行按钮；离线测试增加 Phase B wrapper / command 名称不得出现在普通 UI markup 的断言。

验证结果：

- `cargo test --lib real_execution_command`：通过，31 passed。
- `cargo test --lib session_continuation`：通过，17 passed，4 ignored real-probe tests。
- `cargo test --lib codex_local_runner`：通过，11 passed。
- `cargo test --lib runtime_log`：通过，6 passed。
- `cargo test --lib diagnostic`：通过，4 passed。
- `cargo test --lib workflow_authorization`：通过，1 passed。
- `cargo test --lib`：通过，300 passed，5 ignored。
- `cargo fmt -- --check`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，offline interaction tests passed: 13。
- `npm run build`：通过；保留既有 Vite chunk size warning。

扫描分类：

- `rg -n "runRealExecutionProductCommandPhaseB|run_real_execution_product_command_phase_b" prototypes/productized-desktop-shell/src/App.tsx prototypes/productized-desktop-shell/src/views prototypes/productized-desktop-shell/src/components`：无命中，普通 UI 未接 Phase B wrapper / command。
- 同名 wrapper 全局命中只在 `src/lib/tauri.ts`、Rust command/service、invoke handler、PCR9A 单测与离线禁止暴露断言中。
- `Command::new("codex")|codex exec|codex exec resume|prompt_body` 扫描：`Command::new("codex")` 无命中；`codex exec*` 命中为既有 redacted command preview / ignored real-probe 文案；`prompt_body` 命中为 DTO/runtime stdin 参数、hash 校验、fake runner 测试、既有“not persisted”文案，未新增持久化 runtime prompt 文本。
- 敏感词扫描命中大量历史任务包边界文案、既有 rollout/session read model 字段、secret/keychain/provider credential deny-list/只读状态文案；本轮未新增读取 secret/token/.env/keychain/OAuth/provider credential/full transcript/rollout 的代码路径，未读写 `/Users/yoyi/.codex` 或 `.codex/plugins/cache`。

不能声明完成事项：

- 不能声明 PCR9 B1/B2 真实探针已完成。
- 不能声明用户已授权真实执行。
- 不能声明真实 Codex 已执行。
- 不能声明 planned adapters 真实接入、provider/model verification 完成、自动 retry / stop / restart 完成。

## 13. 主管复核记录

结论：带 P2 通过。PCR9A 可接受为“统一 product command Phase B 桥已具备可测试产品路径，PCR9 真实探针代码前置阻断已解除”。仍不得声明 PCR9 B1/B2 真实探针完成或用户已授权真实执行。

主管线验证：

- `cargo test --lib real_execution_command`：通过，31 passed。
- `cargo test --lib`：通过，300 passed，5 ignored。
- `cargo fmt -- --check`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，offline interaction tests passed: 13。
- `npm run build`：通过，仅保留既有 Vite chunk size warning。
- 普通 UI Phase B wrapper / command 窄扫无命中。
- `.codex/plugins/cache` 窄扫只命中本任务包禁止 / 扫描说明。
- 敏感路径读取窄扫无命中。
- 入口文档 `CURRENT.md` / `AUTHORITY.md` / `STAGE_PLAN.md` / `README.md` / `tasks/README.md` 未同步 PCR9A 状态。

只读复核线结论：

- P0/P1：无。
- P2：既有底层 continuation Phase B Tauri/API 面仍存在，后续 PCR9 evidence 必须明确不能用它冒充统一 product command 路径。真实探针验收必须以 `run_real_execution_product_command_phase_b`、product command attempt、runtime/audit/readback refs 为完成证据。

关键边界：

- 本轮未执行真实 `codex exec` / `codex exec resume`。
- 未发送真实 prompt。
- 未读写 `/Users/yoyi/.codex`，未读取 `.codex/plugins/cache`。
- 未读取 auth/token/secret/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未启动 Browser/Chrome/Tauri/Vite preview/截图工具。
- 未同步权威入口；后续入口同步必须等 PCR checkpoint。

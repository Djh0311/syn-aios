# Stage H / H2.3 Real Resume Request Builder And Codex Local Guard Bridge v1 Evidence

日期：2026-06-07

结论：H2.3 接受为 H2 非执行 request builder 和 CodexLocal guard bridge 完成；不接受为 H2 通用真实 resume 产品化完成。

## 1. 范围

本轮完成：

- H2 real resume preflight 输出新增 `codex_local_request`。
- H2 real resume preflight 输出新增 `codex_local_guard`。
- 授权矩阵完整时，H2 preflight 会构建 H1 `CodexLocalExecutionRequest`。
- H2 preflight 会调用 H1 `inspect_codex_local_execution_guard` 做 guard 复核。
- H1 guard 阻断时，H2 preflight 返回 `blocked_by_codex_local_guard`，并把 guard reason 合并进缺失 / 无效项。
- 授权矩阵不完整时，不构建 request / guard。
- 授权矩阵完整且 guard 允许时，只返回 `complete_but_not_executed`，仍不调用真实 runner。
- 同步 H2.3 任务包、evidence、handoff 和权威入口。

## 2. 边界

本轮没有：

- 执行真实 `codex exec`。
- 执行真实 `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 auth/token/.env/secret/keychain/OAuth/provider credential/full transcript。
- 创建 fixture 项目。
- 启动 Tauri、GUI 或截图。
- 新增执行按钮、发送按钮、resume 按钮、确认按钮、授权按钮或重试按钮。
- 把 H2 标记为完成。
- 进入 H3/H4/H5。

## 3. 新增 / 更新文件

新增：

- `tasks/2026-06-07-stage-h-h2-3-real-resume-request-builder-and-codex-local-guard-bridge-v1.md`
- `evidence/2026-06-07-stage-h-h2-3-real-resume-request-builder-and-codex-local-guard-bridge-v1.md`
- `handoffs/2026-06-07-stage-h-h2-3-real-resume-request-builder-and-codex-local-guard-bridge-v1-result.md`

更新：

- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`

## 4. 关键实现证据

`InspectControlledSessionContinuationRealResumeOutput` 现在包含：

- `codex_local_request: Option<CodexLocalExecutionRequest>`
- `codex_local_guard: Option<CodexLocalExecutionGuard>`

`inspect_real_resume_authorization` 现在的行为：

- 先校验 continuation、adapter、operation、duplicate running attempt 和 H2 授权矩阵。
- 授权矩阵不完整：保持 `blocked_waiting_authorization`，不构建 request / guard。
- 授权矩阵完整：通过 `build_codex_local_request_for_h2` 构建 H1 request。
- 对 request 调用 `codex_local_runner::inspect_codex_local_execution_guard`。
- guard 阻断：返回 `blocked_by_codex_local_guard`。
- guard 允许：返回 `complete_but_not_executed`，但 attempt 仍为 `h2_real_resume_preflight_no_execution`。

`build_codex_local_request_for_h2` 生成的 request 包含：

- `adapter_id`
- `operation_id`
- `project_id`
- `project_root`
- `workflow_id`
- `node_id`
- `session_id`
- `continuation_id`
- `target_cwd`
- `allowed_write_roots`
- `sandbox`
- `prompt_source_kind`
- `prompt_summary`
- `prompt_sha256`
- `prompt_ref`
- `readback_plan`
- `authorization_scope_id`
- `runtime_log_refs`
- `audit_refs`
- `active_attempts`
- 非执行 warning：`h2_request_builder_only`、`codex_local_guard_only_no_runner_call`、`prompt_not_sent`、`codex_home_not_touched`

## 5. 测试覆盖

已覆盖的关键断言：

- incomplete matrix 返回 `blocked_waiting_authorization`。
- incomplete matrix 不生成 `codex_local_request`。
- incomplete matrix 不生成 `codex_local_guard`。
- complete matrix 返回 `complete_but_not_executed`。
- complete matrix attempt status 为 `ready_for_real_resume_authorization`。
- attempt `prompt_sent=false`。
- attempt `real_codex_executed=false`。
- attempt `writes_codex_home=false`。
- `readback_summary.result_count=None`，不会把 readback unavailable 写成 0。
- complete matrix 生成 H1 request。
- request operation 为 `resume`。
- request 使用授权矩阵里的 target session、target cwd 和 prompt hash。
- complete matrix 生成 H1 guard。
- guard 不阻断 dry-run inspection。
- command plan `program="codex"`。
- command plan 非 shell invocation。
- command plan 不把 prompt 放进命令。
- command argv 包含 `resume`。
- command argv 不包含 prompt hash。

## 6. 验证

已通过：

```text
rustfmt --check src/session_continuation_store.rs src/types.rs
cargo test --lib session_continuation
cargo test --lib codex_local
npm run typecheck
```

最终回收前已重新执行并通过上述命令。

已知非阻断提示：

- Rust 仍保留既有 `JsonRpcError::invalid_params` unused warning。

## 7. 复核扫描

已完成权威入口同步扫描：

```text
rg -n 'H2\.3|h2-3|2026-06-07-stage-h-h2-3' CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans/README.md docs/plans/middleware-version-stage-plan-v1.md docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md
```

结果：H2.3 已出现在 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`docs/plans/README.md`、`docs/plans/middleware-version-stage-plan-v1.md` 和 H-I 阶段计划中。

已完成误导口径扫描：

```text
rg -n 'H2 已完成|H2 通用真实 resume 产品化完成|H3 可开始|H3 .*可直接开始|真实 resume 已执行|真实 `codex exec resume` 已执行|prompt 已发送|`\.codex` 已读写|\.codex.*已读写|Codex 已收到任务|planned adapters 已接入|provider credential 已验证' ...
```

分类结果：

- H2.3 新任务包 / evidence / handoff 中的命中均位于“不接受为”“禁止显示”“禁止说”或“未完成 / 保留项”语境。
- 权威入口中的命中均位于“不代表”“不接受为”“不能直接进入”“未获授权前不得执行”等安全语境。
- 旧历史命中如 `2026-05-30-workflow-state-closure-real-dispatch-retest-v1` 明确是历史真实派发记录，不是 H2.3 新增执行。
- 未发现 H2.3 被写成真实 resume 已执行、prompt 已发送、`.codex` 已读写、H2 已完成、H3 可开始、planned adapters 已接入或 provider credential 已验证。

## 8. 未完成 / 保留项

- H2 真实 resume 仍未执行。
- prompt 仍未发送。
- `/Users/yoyi/.codex` 仍未读写。
- H2 仍缺真实执行前任务包级明确授权。
- H3 通用真实 send / 新会话仍不可开始。
- 真实 Tauri / 截图验收未执行；本轮无可见 UI 改动，因此不把它作为 H2.3 阻断项。

## 9. 全局主管复核结论

H2.3 可以接受为 H2 非执行 request builder 和 CodexLocal guard bridge 完成。

H2 仍处于真实执行前准备阶段。下一步不能直接进入 H3；必须先由用户和全局主管明确授权真实 H2 resume 的测试项目、target session、`.codex` 最小范围、prompt hash/ref、allowed write roots、readback plan、runtime log、audit、evidence 和 rollback。

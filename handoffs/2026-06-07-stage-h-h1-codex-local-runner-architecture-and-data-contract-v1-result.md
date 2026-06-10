# Handoff: Stage H / H1 CodexLocalRunner Architecture And Data Contract v1

日期：2026-06-07

状态：已完成，并已通过全局主管复核。

## 结果摘要

H1 已完成 `CodexLocalRunner` 架构和数据契约。实现只包含 Rust / TS 类型、后端 guard、`CodexLocalRunner` trait、`FakeCodexLocalRunner` dry-run 和单测；没有真实执行、没有 prompt 发送、没有 `/Users/yoyi/.codex` 读写、没有 UI 执行入口。

## 可复用对象

- E4：`SessionContinuationRequest`、`SessionContinuationPreview`、`SessionContinuationGuardResult` 的 preview / guard 边界。
- E5：`SessionContinuationStoreV1`、`ControlledSessionContinuation`、`SessionContinuationAttempt`、`SessionContinuationReadbackSummary`、`SessionContinuationAuditEvent` 的 confirmation / attempt / audit 记录边界。
- E6：`RuntimeSessionAttention`、`SessionRunStatusSummary`、`ReadbackBoundaryStatus` 的 unavailable / failed 用户可见边界。
- G1：`RuntimeLogStoreV1`、`RuntimeLogEntry`、`RuntimeLogBoundary` 的 runtime log / audit 分离边界。
- G2：diagnostic / degraded state 只读摘要边界。

## 新增契约

- Rust：`CodexLocalExecutionRequest`、`CodexLocalExecutionGuard`、`CodexLocalExecutionAttempt`、`CodexLocalReadbackPlan`、`CodexLocalReadbackResult`、`CodexLocalFailureReason`、`CodexLocalRuntimeLogRef`、`CodexLocalAuditRef`。
- Rust：`CodexLocalCommandPlan` 和 `CodexLocalActiveAttempt` 作为 H1 guard 支撑类型。
- Rust：`CodexLocalRunner` trait 和 `FakeCodexLocalRunner`。
- TS：镜像上述 H1 数据类型，不新增 Tauri wrapper。

## Guard 流程

1. 校验 adapter / operation。
2. 校验 project / workflow / node / session 或 work item 绑定。
3. 校验 cwd、project root、allowed write roots，路径必须是绝对路径且不能包含 `..` 逃逸。
4. 校验 secret deny list。
5. 校验 prompt summary、prompt hash、prompt ref。
6. 校验 readback plan。
7. 校验 duplicate running / queued attempt。
8. 校验 user confirmation、authorization scope 和 audit refs。
9. 生成结构化 command plan。
10. H1 fake runner 只记录 dry-run attempt 和 readback unavailable。

## 安全边界

- CLI 计划是 `program + argv`，不拼 shell 字符串。
- prompt 不进入 argv，只通过 stdin prompt ref/hash 表示。
- H1 fake runner 不调用 `Command::new`、不 spawn、不中转 shell。
- readback unavailable / failed 不等于 0 条结果。
- runtime log / audit / readback 分离：runtime log 只记录脱敏运行状态，audit 只记录确认和权限，readback 只记录可信读回状态。
- 全局主管复核后补强：readback plan 和可选绑定字段也纳入敏感片段扫描，避免 `.codex`、secret、token、credential、full transcript 等敏感引用进入 H1 request。

## 验证

- `rustfmt --check src/codex_local_runner.rs src/types.rs src/lib.rs`：通过。
- `cargo test --lib codex_local -- --nocapture`：开发线通过 3 passed；主管复核补强后通过 4 passed。
- `cargo test --lib`：开发线通过 235 passed, 1 ignored；主管复核补强后通过 236 passed, 1 ignored；仅有既有 `JsonRpcError::invalid_params` dead_code warning。
- `npm run typecheck`：通过。

## 风险

- H1 只是契约和 fake dry-run，不证明真实 `codex exec` / `codex exec resume` 可产品化。
- 现有 `RealCodexResumeRunner` 仍是历史真实 runner 参考，H1 未接入它。
- H2 需要重新定义真实执行落账、读回和回滚验证，不能复用 H1 dry-run 结论替代真实验收。

## 下一步建议

H1 已通过全局主管复核。下一步可准备 H2 任务包；H2 任务包必须逐项授权测试项目、目标 session、allowed write roots、`.codex` 读写范围、用户确认弹层、runtime log / audit / readback 写入和失败回滚。未获授权前不得执行真实 resume。

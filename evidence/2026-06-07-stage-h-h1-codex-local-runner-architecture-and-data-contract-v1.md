# Evidence: Stage H / H1 CodexLocalRunner Architecture And Data Contract v1

日期：2026-06-07

状态：已完成，并已通过全局主管复核。

## 本轮改动

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`。
- 更新 `prototypes/productized-desktop-shell/src-tauri/src/types.rs`。
- 更新 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`。
- 更新 `prototypes/productized-desktop-shell/src/lib/types.ts`。
- 新增本任务包和 handoff，并同步当前权威入口。

## 实现证据

Rust H1 数据契约已新增：

- `CodexLocalExecutionRequest`
- `CodexLocalExecutionGuard`
- `CodexLocalExecutionAttempt`
- `CodexLocalReadbackPlan`
- `CodexLocalReadbackResult`
- `CodexLocalFailureReason`
- `CodexLocalRuntimeLogRef`
- `CodexLocalAuditRef`
- `CodexLocalCommandPlan`
- `CodexLocalActiveAttempt`

Rust H1 runner 契约已新增：

- `CodexLocalRunner`
- `FakeCodexLocalRunner`
- `inspect_codex_local_execution_guard`

Guard 覆盖：

- `adapter_id` 只能是 `codex-local`。
- `operation_id` 只能是 `send_message` / `resume`。
- `resume` 必须绑定 session。
- 必须绑定 project / workflow / node，并绑定 session 或 work item。
- `project_root`、`target_cwd` 和 allowed write roots 必须是绝对路径，且不能包含 `..` 逃逸。
- `target_cwd` 必须落在 project root 或 allowed write roots。
- allowed write roots 必须落在 project root 内。
- secret deny list 覆盖 `.codex`、`.env`、auth、token、secret、keychain、OAuth、provider credential、full transcript。
- 必须有脱敏 prompt summary、64 位 hex prompt hash 和 prompt ref。
- readback plan 必须 required，且必须声明 expected sources 和 unavailable behavior。
- 会阻断 duplicate running / queued attempt。
- H1 dry-run 也要求用户确认状态和 authorization scope。
- audit ref 必须存在；runtime log ref 缺失只给 warning，dry-run attempt 会生成脱敏 runtime log ref。

主管复核补充修补：

- 路径 guard 增加绝对路径和 `..` 逃逸阻断。
- secret deny list 增加 readback plan、session / work item / continuation / authorization scope 等可选绑定字段扫描。
- 新增单测覆盖 `target_cwd` 路径逃逸和 readback source 引用 `.codex` 时必须阻断。

CLI 安全计划：

- 只构造 `program: "codex"` 和 `argv: string[]`。
- prompt 只以 `stdin_prompt_ref` 和 `stdin_prompt_sha256` 表示。
- `prompt_in_command=false`。
- `shell_invocation=false`。
- 不调用 `Command::new`、不 spawn、不中转 shell、不写 stderr/stdout 文件。

Readback 边界：

- H1 fake runner 固定 `readback_result.status=readback_unavailable`。
- `attempted=false`。
- `real_readback_performed=false`。
- `result_count=None`。
- warning 包含 `readback_unavailable_is_not_zero_results`。

## 验证命令

`rustfmt --check src/codex_local_runner.rs src/types.rs src/lib.rs`

结果：通过。

`cargo test --lib codex_local -- --nocapture`

开发线结果：通过，3 passed。

主管复核后结果：通过，4 passed。

`cargo test --lib`

开发线结果：通过，235 passed, 1 ignored。

主管复核后结果：通过，236 passed, 1 ignored。仅有既有 warning：`src/mcp/protocol.rs` 的 `JsonRpcError::invalid_params` 未使用。

`npm run typecheck`

结果：通过。

固定口径扫描：

- 覆盖 H2/H3 完成、真实 Codex 执行完成、planned adapters 接入完成、provider credential 验证完成等七类越界短语。
- 本轮新增和同步文件未出现越界完成宣称。
- 唯一命中为历史 G4 任务包中的“禁止出现”自检短语，不是完成声明。

## 未执行事项

- 未执行真实 `codex exec`。
- 未执行真实 `codex exec resume`。
- 未发送真实 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 auth / token / `.env` / full transcript / secret / keychain / OAuth / provider credential。
- 未启动 Tauri、未截图、未做 GUI 操作。
- 未接入 planned adapters 真实执行。

## 接受结论

H1 可接受为 CodexLocalRunner 架构和数据契约、guard、fake dry-run 和 TS 类型镜像完成，并已通过全局主管复核。H1 仍不授权 H2/H3 真实执行。

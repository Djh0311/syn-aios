# Stage H / H1 CodexLocalRunner Architecture And Data Contract v1

日期：2026-06-07

状态：已完成，并已通过全局主管复核。

## 目标

完成 H1：把 E4/E5/E6/G1/G2 的 continuation preview、stub attempt、runtime attention、runtime log 和 diagnostics 边界收敛为 `CodexLocalRunner` 架构和数据契约。

H1 只做类型、guard、runner 契约、fake runner / dry-run、单测和必要只读类型镜像；不执行真实 Codex，不发送 prompt，不读写 `/Users/yoyi/.codex`。

## 必须产出

- Rust 类型：`CodexLocalExecutionRequest`、`CodexLocalExecutionGuard`、`CodexLocalExecutionAttempt`、`CodexLocalReadbackPlan`、`CodexLocalReadbackResult`、`CodexLocalFailureReason`、`CodexLocalRuntimeLogRef`、`CodexLocalAuditRef`。
- Rust runner 契约：`CodexLocalRunner` trait 或等价应用服务契约。
- H1 fake runner / dry-run：只能产出结构化 attempt，不 spawn 进程、不写 sidecar、不发送 prompt。
- Guard：校验 adapter、operation、项目 / 工作流 / 节点 / session 或 work item 绑定、cwd / allowed write roots、secret deny list、prompt summary/hash/ref、readback plan、duplicate running attempt、用户确认和 authorization scope。
- CLI 计划：只用 `program + argv` 和 stdin prompt ref/hash 表达；禁止 shell 字符串拼接、禁止 `sh -c`、禁止把 prompt 放进 command string。
- Runtime log / audit / readback 关系：audit 记录确认和权限，runtime log 记录脱敏运行状态，readback 记录可信结果状态；readback unavailable / failed 不能变成 0 条。
- TS 类型镜像：只做类型，不新增真实执行按钮或 Tauri invoke。

## UI 显示边界确认

本任务没有新增可见 UI、按钮、导航入口或真实执行入口。前端改动只镜像 TypeScript 数据类型，不改变页面展示和交互。

## 接受范围

- 接受为 H1 CodexLocalRunner 架构和数据契约完成。
- 接受为 H1 guard / fake dry-run 单测覆盖完成。
- 接受为结构化 CLI argv 计划和 stdin prompt ref/hash 边界完成。
- 接受为 readback unavailable 不等于 0 条结果的 H1 契约完成。

## 不接受范围

- 不接受为 H2 通用真实 resume 产品化。
- 不接受为 H3 通用真实 send / 新会话产品化。
- 不接受为真实 `codex exec` 或真实 `codex exec resume` 已执行。
- 不接受为 prompt 已发送、真实 readback 已完成或 worker 已执行。
- 不接受为 planned adapters 真实接入。
- 不接受为 provider credential / model verification。

## 实现记录

- 新增 `src-tauri/src/codex_local_runner.rs`：纯内存 `CodexLocalRunner` trait、`FakeCodexLocalRunner`、guard 和 dry-run attempt 构造。
- 更新 `src-tauri/src/types.rs`：新增 H1 CodexLocal 数据契约类型。
- 更新 `src-tauri/src/lib.rs`：挂载 `codex_local_runner` 模块。
- 更新 `src/lib/types.ts`：镜像 H1 类型，不新增 invoke。

## 验证

- `rustfmt --check src/codex_local_runner.rs src/types.rs src/lib.rs`：通过。
- `cargo test --lib codex_local -- --nocapture`：通过，3 passed。
- `cargo test --lib`：通过，235 passed, 1 ignored。
- `npm run typecheck`：通过。

## 全局主管复核

复核结论：通过。

主管复核补充修补：

- H1 guard 已硬化 `project_root`、`target_cwd` 和 `allowed_write_roots` 路径契约：必须是绝对路径，且不能包含 `..` 逃逸。
- H1 secret deny list 已扩展到 readback plan 和可选绑定字段，避免 readback source / warning / authorization scope 等字段携带 `.codex`、secret、token、credential、full transcript 等敏感引用。

主管复核后验证：

- `rustfmt --check src/codex_local_runner.rs src/types.rs src/lib.rs`：通过。
- `cargo test --lib codex_local -- --nocapture`：通过，4 passed。
- `cargo test --lib`：通过，236 passed, 1 ignored。
- `npm run typecheck`：通过。

## 下一步

H1 已通过全局主管复核。下一步可准备 H2 通用真实 resume 产品化任务包，但 H2 真实执行前仍必须单独批准测试项目、目标 session、允许写入路径、`.codex` 读写范围、用户确认和回滚策略。

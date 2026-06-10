# Stage I / I5 Adapter SDK CLI Parity And Diagnostics Reservation

状态：已完成  
结论：accepted

## 目标

在不接入 planned adapters、不执行真实 worker 的前提下，把未来 adapter 接入需要遵守的 SDK / CLI parity / diagnostics 契约纳入 `WorkbenchSnapshot.worker_protocol` 中立读模型，并在智能体页做只读可见化。

## 范围

- 新增 adapter contract checklist。
- 新增 controlled API / CLI semantics。
- 新增 diagnostic event schema descriptor。
- 新增 adapter health summary。
- 新增 degraded mode。
- 新增 data location / persistence descriptor。
- UI 只显示 I5 契约和诊断预留，不新增执行按钮。

## 验收

- `codex-local` 可显示为 guarded contract，但仍要求 control core / permission / audit / runtime log。
- planned adapters 必须保持 blocked / reserved / unavailable，不得显示为已接入或可执行。
- CLI parity 不能成为 universal app API backdoor。
- diagnostics schema 必须声明脱敏，不包含 secret、raw transcript 或 provider payload。

## 边界

- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 auth/token/secret/`.env`/keychain/OAuth/provider credential/full transcript。
- 不新增 store，不迁移数据库，不接 planned adapters 真实执行。
- 不接受为阶段 I 完成；下一步仍需 I6 最终验收和后续 adapter 路线冻结。

## 记录

- Evidence：`../evidence/2026-06-08-stage-i-i5-adapter-sdk-cli-parity-and-diagnostics-reservation-v1.md`
- Handoff：`../handoffs/2026-06-08-stage-i-i5-adapter-sdk-cli-parity-and-diagnostics-reservation-v1-result.md`

# M2a · T4 Grant Ledger 与 Code Map 验收记录 · 2026-08-04

任务：`tasks/2026-08-04-syn-m2a-remaining-one-shot-package-v1.md`

结论：**grant ledger focused path PASS；Code Map staged/managed-installation gates HOLD；Station 3b 仍为 M3 边界 HOLD。** 本记录不把其中任一局部结果提升为 M2 完成。

## 已验证的 M2 grant ledger 子路径

生产路径只从 active `PlanAuthorizationStoreV1` source 铸造 grant，并在 runner 前将不可变 grant/attempt/binding 写入 `workflow_node_dispatches`。调用方不能把 `dispatch_id` 当 grant 或 attempt，也不能提供 wildcard scope：

- `GrantMintInput` 与 `mint_grant` 为 `#[cfg(test)]`；生产只使用 `mint_dispatch_grant`。
- grant/attempt 均为服务端随机标识；grant 精确绑定 authorization revision、project、workflow、node、work item、dispatch、attempt、binding、principal 和排序后的 scope fingerprint。
- worker execution report 会 fresh-load persisted dispatch 与 active authorization source；伪造 grant、错 subject、非终态 attempt、撤销后回传均拒绝且不写 report。
- Tauri 公共 ingress 只接受 manual/offline report；`report_kind=execution` 在任何 state I/O 前拒绝，真实 execution report 只可走服务器验证路径。
- `s3_director_dispatch_integration_stub` 的 DB-primary variant 会用新的只读 SQLite connection 读取 `workflow_node_dispatches.record_json`，与 JSON projection 相等后，反序列化 grant 并逐字段 verify；这证明了该受控 fixture 的 mint → persist → fresh DB load → verify 闭环。

本轮聚焦命令与结果：

| 命令 | exit | 结果 |
| --- | ---: | --- |
| `cargo test --lib mcp::execution_grant::tests -- --nocapture` | 0 | `5 passed; 0 failed; 0 ignored` |
| `cargo test --lib worker_report::tests -- --nocapture` | 0 | `24 passed; 0 failed; 0 ignored` |
| `cargo test --lib m2a_execution_report_ingress_tests -- --nocapture` | 0 | `1 passed; 0 failed; 0 ignored`；forged Tauri execution ingress 零 state write |
| `cargo test --lib fnd006_acceptance -- --nocapture` | 0 | `10 passed; 0 failed; 0 ignored` |
| `cargo test --lib s3_director_dispatch_integration_stub -- --nocapture` | 0 | `1 passed; 0 failed; 0 ignored`；含 DB-primary fresh read-only ledger 校验 |
| `cargo check --lib` | 0 | `694 warnings`；未以 warnings=0 声称清零 |

安全措辞限制：`grant_hash` 是完整性 checksum，不是外部签名；该机制是 **M2 legacy-dispatch grant ledger adapter**，不是 M3 的 `RoleSession` 或 canonical `PreparedAttempt`。生产 report verifier 仍从 canonical dispatch projection 读取后再验证，不能声称每次生产 report 直接从 SQLite 查询。

## forged / mismatch 拒绝范围

被聚焦测试覆盖的拒绝包括：缺失/畸形/伪造 canonical grant id、伪造 report subject、mid-flight attempt state、撤销 authorization 后回传、过期/篡改 grant、wildcard 或重复 scope，以及公开 Tauri ingress 提交伪 execution report。拒绝路径的测试均断言没有 report/state 写入；这不等于真实外部 provider 或 M3 session 运行证据。

## Code Map 结算（R2 更正）

本节原先记录的是本包临时修改 managed Code Map 实现时的观察，不能作为现役 installed Code Map 的验收。该 managed 实现已按 R2 边界撤回，关联的未跟踪临时测试 `scripts/harness-v2/code-map-model.test.js` 亦已撤回；它不是正式受管测试，也不再构成证据。以下为撤回后的事实边界：

| 检查 | exit | 事实 |
| --- | ---: | --- |
| `node scripts/harness-v2/config-check.js --target . --strict --json` | 0 | managed installation digest 一致。 |
| `node scripts/harness-v2/active-path-audit.js --target . --strict --json` | 0 | 仅证明受管 active path 完整（不是 Code Map 语义验证）。 |
| `node scripts/harness-v2/codebase-map.js query --query workflow --target . --json` | 0 | `INCOMPLETE_SHADOW`；现役受管模型不能读取当前结构化 provenance。 |
| `node scripts/harness-v2/codebase-map.js overlay --target . --json` | 0 | `INCOMPLETE_SHADOW`；同一 managed schema 能力缺口。 |
| `node scripts/harness-v2/codebase-map.js check --staged --shadow --target . --json` | 0 | `WARN / MAP_UPDATE_REQUIRED`；本轮禁止 staging，不能把 index 基线当作 dirty candidate 验收。 |

本包没有 staging 权限；因此不能把工作树的 candidate 偷换成 `--staged --shadow` result，也不能改检查器来掩盖该 advisory。已撤回的临时 managed 变更不构成现役语义工作；“相关 advisory 清零 / Code Map query/overlay/model”均为 **HOLD / HARNESS_CAPABILITY_GAP**，需要单独授权的正式 managed-pack/schema 升级。

## Station 3b 边界

`fnd006_acceptance` 的 fixture 覆盖了 Station 3b write rejection；没有新增 `RoleSession`、canonical `PreparedAttempt` 或真实 Station 3b session/attempt runtime。因此没有 M2 的 Station 3b 运行 PASS。按 M2 计划，该缺口是 **M3 RoleSession 边界 HOLD**，M3 仍不得激活。

## 本记录的非主张

- 未接真实外部 provider。
- 未宣称单机/off-machine 验证。
- 未宣称 M2 已完成、T2 已接受或 M3 已开始。

# M5R07 项目 UI、隔离 App 与阶段候选报告

- 日期：2026-08-17
- 阶段：stage-14 / leaf M5R07
- 状态：**`AWAITING_INDEPENDENT_ACCEPTANCE`**
- 不宣布 M5 完成，不关闭 stage-14，不激活 M6 / F2

本包是对总线 REJECT `d31885e` 的最窄 correction，不是 closeout。

## 候选载体

| 项 | 值 |
|---|---|
| 当前 exact implementation candidate | `df11a4a3fa19c6d91c8aaa006e395f83c155e772` |
| 被拒 predecessor | `d31885ec878aca64fb98dfe00550419501b91e2a` |
| 更早被拒 authority 路径 | `faa6ed191f6bef29ddd03b74b4369c4b4e6445fd` |
| 基线（M3O03 evidence tip） | `288fd3dc6e9b8a2438a4094af63f838c839e834f` |
| Disposable checkout | `/tmp/m5r07-disposable-df11a4a` |
| Disposable opening SHA | `df11a4a3fa19c6d91c8aaa006e395f83c155e772` |
| Disposable final SHA | `df11a4a3fa19c6d91c8aaa006e395f83c155e772` |
| Isolated launcher receipt | `docs/harness/reports/M5R07-isolated-app-launcher-receipt.json` |
| Isolated UI receipt | `docs/harness/reports/M5R07-isolated-ui-unavailable-receipt.json` |
| Disposable receipt | `docs/harness/reports/M5R07-disposable-checkout-receipt.json` |

## 被拒原因（d31885e）

1. `run_m5_authorized_runtime_with_state` 在任何 durable operation/effect/receipt 之前只复核了 ProjectSupervisor；执行用的是 M5 grant 上的 `worker_role_session_id`，没有经 AppState M3 authority `load` 当前 Worker view。approve 之后 Worker 变 inactive、binding drift 或 permission drift 仍会执行。
2. `ensure_supervisor_schema` 仍 `CREATE TABLE IF NOT EXISTS m5_role_sessions`，与“删除创建/真源化该表、但不 DROP 旧库已有表”冲突。

`faa6ed1` 旧 isolated Tauri full-loop **不再是** M1/M3 authority PASS；它绕过已安装端口，不能作为本 candidate 的权威闭环证据。

## 本 correction 产品结果

1. Runtime 在任何 workcell / retry / formal-progress receipt 写入前，用 `require_binding` 返回的 typed `M1ProjectId` 调用 `load_project_role(Worker)`；M3 stable error 原样返回。成功后精确核验 Worker project、role、`role_session_id` 与 grant；`Workcell.actor_binding` 只用刚加载 view 的 `role_session_id`。
2. 攻击测试：approve 后把 Worker 置 `SUSPENDED`，runtime 得到 `m3_project_role_session_inactive`，durable operation/receipt/progress 计数不变。permission snapshot 漂移得到 `m3_project_role_session_permission_drift`，同样零写。
3. Fresh M5 store 不再 CREATE `m5_role_sessions`。旧库若已有该表，不会被 INSERT/SELECT。

保持 d31885e 其余边界：M1 无 fallback；三角色只消费 M3 view；open DTO 无 caller `role_session_id`；shared isolated constructor 保持 M1/M3 未安装。

## Isolated composition gap

冻结合同 `m1-m3-shared-appstate-acceptance-profile-isolation-v1.md` 要求 `try_new_with_isolated_product_profile` 不安装 M1/M3。当前 leaf 原先希望同一 shared isolated Tauri 走完整闭环，这在冻结合同下不可能。本包把它记为独立组合前置缺口：isolated profile 只证明 unavailable，不得冒充 full-loop 或 authority PASS。普通产品闭环只在 disposable ordinary AppState 测试中证明。

## 验证

- Disposable `cargo check --lib --offline`：PASS
- Disposable `cargo test --lib --offline -- m5_ -- --test-threads=1`：90 passed / 0 failed
- Disposable `npm run typecheck`：PASS
- Disposable `npm run build`：PASS（310 modules）
- Isolated launcher（disposable checkout，隔离 fake profile）：exit 0；`open_available=false`；`full_loop_claimed=false`；`m1_authority_installed=false`；`m3_authority_installed=false`；scene A/B/resume 均未声称 PASS
- 全库 `cargo test`：不需要、不宣称 PASS

## 交给总线

- Git：本地新增 correction `df11a4a`，未 push / merge / rebase。既有 WIP 仍原位未 add。
- Harness：唯一 current leaf = M5R07；authorization closed；stage-14 仍开。
- 请总线只读复核 exact candidate `df11a4a` 及其后续 evidence-binding SHA。不要把本报告当成 M5 完成或 isolated full-loop PASS。

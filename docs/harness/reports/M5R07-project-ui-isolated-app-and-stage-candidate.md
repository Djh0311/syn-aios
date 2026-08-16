# M5R07 项目 UI、隔离 App 与阶段候选报告

- 日期：2026-08-17
- 阶段：stage-14 / leaf M5R07
- 状态：**`AWAITING_INDEPENDENT_ACCEPTANCE`**
- 不宣布 M5 完成，不关闭 stage-14，不激活 M6

本包是对独立验收失败的最窄返修，不是 closeout。

## 候选载体

本工作树相对 `04200bb` 只新增 M5 返修提交。七个未跟踪 M6 文件、`linux-schema.json` 与 2026-08-17 壳方向文档仍原位未 add。

| 项 | 值 |
|---|---|
| 基线 HEAD | `04200bb57e20240edc04f582f21ccf1ec0ed61d1` |
| 先前失败实现 candidate | `65413a2d32830e689a6dc73ae34f75c4efbf223f` |
| 新实现 candidate | `faa6ed191f6bef29ddd03b74b4369c4b4e6445fd` |
| Isolated launcher receipt | `docs/harness/reports/M5R07-isolated-app-launcher-receipt.json` |
| Disposable checkout | `/tmp/m5r07-disposable-faa6ed1` |
| Disposable receipt | `docs/harness/reports/M5R07-disposable-checkout-receipt.json` |

## 返修产品结果

1. **M3-owned RoleSession + 服务器项目身份**  
   普通产品 `open_m5_project_supervisor` 经 `m5_m3_identity` 打开/恢复 M3 ordinary RoleSession；不再发明 `m5:project-supervisor:{id}` 或把 `m5_role_sessions` 当产品身份根。项目身份由 index / isolated profile 服务器解析。调用方自造 `role_session_id` 被拒绝。隔离 Tauri 恢复到同一 `session:sha256:…` 与同一 binding。

2. **渲染器不得选 Grant command/scope**  
   前端不再发送 `allowed_command`。提案提交时服务器写入 `authorized_action`；批准只绑定该存储动作与 `pol:m5r07:{action}`。扩权 command 被拒（`renderer_grant_scope_rejected`）。

3. **普通 UI 正式执行链**  
   项目面板用正式 command：runtime receipt、worker report、independent review、result decision、summary。隔离 helper 仅 `SYN_M5R07_ISOLATED_ACCEPTANCE=1` 可写 receipt。隔离 DOM 驱动点击正式按钮，不再调用 isolated followthrough command。

4. **Summary ACL / source / deep-link**  
   consumer 来自 M3 RoleSession。source refs 是事实/claim/run 的真实 id。deep-link `syn://m5/{type}/{id}` 经 `resolve_source_ref` 回源。

5. **隔离 UI receipt 后端派生**  
   `write_m5_isolated_ui_receipt` 只收 phase，从 store 派生 grant 计数、join、stale、deep-link、binding。Scene A：`grants=0`、`spawned=false`、拒绝提案。Scene B：exact grant/dispatch/claim/review/fact join、`stale=true`、deep-link 可解析。Resume：`same_binding=true`、`same_role_session=true`。

6. **Disposable checkout 后真实隔离 Tauri**  
   实现提交后在 `/tmp/m5r07-disposable-faa6ed1` checkout `faa6ed1` 跑 cargo/typecheck/build 与 `node scripts/run-m5-isolated-app-acceptance.mjs`。opening=final=`faa6ed1`。

## 未记录 delta 说明

`commands.rs` 与 `lib_read_model_boundary_tests.rs` 先前各加 `m5_store_path: None`：**仅因** `AppState` 新增该字段后既有测试字面量 E0063；不改 command 语义或读模型边界。本返修未再改这两文件。

## 隔离场景事实

- Scene A：只读 chat + REJECTED，`grants=0`，`spawned=false`。
- Scene B：APPROVED 后经正式 runtime/report/review/result/summary；exact join 成立；deep-link 可解析；stale 可观察。
- Resume：同一 binding + 同一 M3 role_session。
- 窗口截图：`DISPLAY=:0` 写出 1920×1080 PPM。选中 drawable 是 X11 root（xid=1080，空 title），两帧 SHA 相同，**不得写成面板像素 PASS**。
- 旧入口：仍 guarded-legacy，未物理删除。

## 验证

- `cargo check --lib --offline`（disposable）：PASS
- `cargo test --lib --offline -- m5_`（disposable）：83 passed / 0 failed
- 完整 `cargo test --lib --offline`：不宣称 PASS。权威树先前观察到既有非 M5 失败：`conversation_transport_command_tests`、`exec_process_registry` reaper、`fix9_tests`、`manual_relay::conversation_transport`。本轮未复跑全库。
- `npm run typecheck`（disposable）：PASS
- `npm run build`（disposable）：PASS（310 modules）
- 隔离 Tauri 交互 / deep-link / 强退恢复：PASS（disposable checkout JSON receipts）
- 窗口截图：`EXECUTED_ROOT_ONLY / NOT_CLAIMED_PANEL_PIXELS`

## 交给总线

- Git：本地 `main` 相对 `origin/main` 超前；未 push、未 merge、未 rebase。
- Harness：唯一 current leaf = M5R07；authorization closed；stage-14 仍开。
- M6 / D0C04 / D0C05 / M1–M4 冻结合同未动。壳方向文档与七个 m6_*.rs 未跟踪文件仍原位保全。
- 请总线只读复核新的 M5-only candidate SHA `faa6ed1` 及其证据绑定提交，不要把本报告当成 M5 完成。

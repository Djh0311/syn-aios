# L3 `knowledge_open` host-owned relay 恢复后 R1-R3 离线验证 v2

- 日期：2026-07-23
- 对应任务包：`tasks/2026-07-23-l3-knowledge-open-host-owned-relay-and-real-app-acceptance-package-v1.md`
- 前置返工：`tasks/2026-07-23-l3-knowledge-open-relay-secret-sink-closure-repair-package-v1.md` 已获指导线离线验收。
- 执行线结论：**R3 必跑离线门已通过；shape 仍为历史聚合债，R4 依 §4.3 保持 HOLD，等待指导线独立验收。**
- 说明：这是执行线的实际验证/停点记录，不替代指导线验收。

## 1. 本恢复轮完成的最小接线

- host-owned relay 仍保持唯一数据流：固定 supervisor MCP argv 的短期 relay context → host 复核 Active binding 与固定 vault 的已验证 Markdown → 仅向 `main` Webview 发固定意图 → 原生工作区 typed read、选中和实际焦点完成后，以同一 intent/path ack → `knowledge_open` 才可返回 `opened=true`。
- `mcp/mod.rs` 现在拒绝四个 relay 参数的任一重复出现；红测先证明旧解析会以后一值覆盖前一值，修补后重复 endpoint、grant、turn-id、project-id 均 fail-closed。
- 追加 relay wire 的额外字段/非法值、run/project mismatch、错误 intent、replay、timeout、run revoke 与无 listener 的失败矩阵。失败的 `knowledge_open` 不产生 `opened=true`，不改变 binding lifecycle，也不创建卡、chain 或 worker。
- 原生工作区在“读入、选中、焦点”未完整达成时，明确给出中立失败提示并发送 `rejected`，不把该状态展示为已打开。

## 2. 安全复核

- 未新增 intent/path 的 durable binding、DB、JSON、sidecar 或静态全局真相源；relay pending 状态仍在 host 进程内。
- 未新增 supervisor 写能力、非空写根、shell/filesystem、wildcard/default allow-all 或知识写 MCP 工具；工具面仍为 `submit_proposal + knowledge_search/read/open/cite`，本轮没有启动真实工具调用。
- 已获指导验收的 secret-sink/cleanup 返工未被放宽：本恢复轮没有改 Cargo、capability/profile/allowlist、真实 store/vault 或 raw capture 路径。

## 3. 已跑离线验证

| 检查 | 实际结果 |
| --- | --- |
| `cargo test supervisor_relay_arguments_must_be_complete_and_nonduplicated --lib --quiet` | 红测先失败（旧重复 endpoint 可覆盖）；修补后通过，§4.3 格式返工后复跑仍为 1 passed。 |
| `cargo test knowledge_open_relay --lib --quiet` | 11 passed。 |
| `cargo test knowledge_ --lib --quiet` | 54 passed。 |
| `cargo test shared_supervisor --lib --quiet` | 13 passed，1 ignored。 |
| capability registry / binding / transport / orchestrator / registry / manual relay 定向回归 | 分别 4/4、8/8、30/30、59/59（1 ignored）、13/13、54/54（2 ignored）通过。 |
| `cargo check --lib` | exit 0；实际仍报告 598 条项目 aggregate warnings，未写成绿色。 |
| `npm run typecheck` | exit 0。 |
| `node scripts/run-offline-interaction-test.mjs` | exit 0；15 项通过，含 relay UI 合同。 |
| 完整目标 Rust `rustfmt --check --config skip_children=true` | exit 0；只检查任务包列出的 13 个目标，避免递归触及第三文件。 |
| `git diff --check` | exit 0。 |
| `git diff --cached --name-only` | 空。 |

## 4. §4.3 一次性格式返工与精确 diff

使用 `rustfmt 1.9.0-stable (59807616e1 2026-04-14)`，以冻结检查相同的 `--edition 2021 --config skip_children=true` 只写入两份授权文件。

| 文件 | 冻结前 SHA-256 | 格式后 SHA-256 | 精确输出 |
| --- | --- | --- | --- |
| `src/index_host_app_entrypoints.rs` | `011fe5ad6b440d340de50e512e5c99b1f17d5da11e74de048fc61b8e9d94e7d0` | `5ae1f355bc5f0c3f07c24bfa91be7479728affd275dc5259d328b5bf68182a8e` | 2 个 unified diff block：`rows.into_iter().map(...).collect()` 链，以及启动段 3 个相邻 `if let Err(...)`；共 4 个表达式级换行。 |
| `src/lib.rs` | `f74794452496f994220c290fa9cdd111e47262d2e2881ca7f123028e881bdd15` | `828667f3f8631d3d6a9932f3abb93b9101e79c94817ff9b3e24f7167a9abf3ff` | 6 个测试代码 block：`assert_eq!`、bindings `all(...)`、两处 authorization、`audit_events`、`matches!`。 |

精确 pre/post unified diff（除下列 8 个 block 外没有源码变化）：

```diff
--- a/src/index_host_app_entrypoints.rs
+++ b/src/index_host_app_entrypoints.rs
@@ -24,7 +24,10 @@
-            let sessions = rows.into_iter().map(session_record_from_codex_thread).collect();
+            let sessions = rows
+                .into_iter()
+                .map(session_record_from_codex_thread)
+                .collect();
@@ -589,17 +592,21 @@
-    if let Err(error) = crate::exec_process_registry::reap_registered_orphans(&state.workflow_state_path) {
+    if let Err(error) =
+        crate::exec_process_registry::reap_registered_orphans(&state.workflow_state_path)
+    {
-    if let Err(error) = crate::supervisor_session_launcher::reap_supervisor_resident_stale_sessions_at(
-        &state.workflow_state_path,
-    ) {
+    if let Err(error) =
+        crate::supervisor_session_launcher::reap_supervisor_resident_stale_sessions_at(
+            &state.workflow_state_path,
+        )
+    {
-    if let Err(error) = crate::workbench_sqlite_storage_mode::initialize_for_startup(
-        &state.workflow_state_path,
-    ) {
+    if let Err(error) =
+        crate::workbench_sqlite_storage_mode::initialize_for_startup(&state.workflow_state_path)
+    {
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -8484,16 +8484,22 @@
-        assert_eq!(outcome.prepared_count, 2, "StubDirector 拆的两项任务都应真正 prepare");
+        assert_eq!(
+            outcome.prepared_count, 2,
+            "StubDirector 拆的两项任务都应真正 prepare"
+        );
-            bindings.iter().all(|binding| optional_string_from(binding, "native_thread_id")
-                .as_deref()
-                == Some(fresh_thread_id)),
+            bindings
+                .iter()
+                .all(
+                    |binding| optional_string_from(binding, "native_thread_id").as_deref()
+                        == Some(fresh_thread_id)
+                ),
@@ -8617,10 +8623,7 @@
-        let authorization = confirmed
-            .plan_authorization
-            .clone()
-            .expect("authorization");
+        let authorization = confirmed.plan_authorization.clone().expect("authorization");
@@ -8790,10 +8793,7 @@
-        let authorization = confirmed
-            .plan_authorization
-            .clone()
-            .expect("authorization");
+        let authorization = confirmed.plan_authorization.clone().expect("authorization");
@@ -9119,7 +9119,10 @@
-        let audit_events = state["audit_events"].as_array().cloned().unwrap_or_default();
+        let audit_events = state["audit_events"]
+            .as_array()
+            .cloned()
+            .unwrap_or_default();
@@ -9144,7 +9147,8 @@
-                    Some("workflow_node_dispatch_prepared") | Some("workflow_node_dispatch_started")
+                    Some("workflow_node_dispatch_prepared")
+                        | Some("workflow_node_dispatch_started")
```

为排除并行漂移，已把格式后文件在临时只读副本中仅反向复原上述 2 + 6 block；两份反向副本的 SHA-256 分别精确回到冻结前 SHA。pre/post unified diff 没有第 9 个 block或第三个文件。`KnowledgeOpenRelayState::new()`、`.start(...)`、`.shutdown()` 与 `knowledge_open_relay: None` 均位于 formatter block 外，写后 token 仍在。

## 5. Shape 与工作树事实

- shape baseline 实测为 **17 errors / 5 warnings / 5 info**；check 同样 exit 1。此前最后已知读数为 **16/5/5**。授权 formatter 使 `lib.rs` 的物理行数从 14795 变为 14799，但 finding 三数未变化；这不支持“绝对零净增”或把历史聚合债写成绿色。
- HEAD 冻结为 `e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`；暂存区在本轮检查时为空。
- 既有大范围脏改保持原样；未 reset、clean、stash、stage、commit 或 push。

## 6. R4 与真实资源状态

- 没有启动 Syn、Codex CLI/MCP server、Obsidian 或真实 App；没有访问真实 store、vault、消息、卡、chain 或 worker。
- R3 已复跑通过，但 §4.3 明确要求先回交指导线；fresh Gate 0、工具列表、四个只读 knowledge 调用与十二项真实 App 场景均未执行。
- 末次 scoped `pgrep` 受本机 `sysmond service not found` 限制而无法读取进程表；这不能证明系统绝无相关进程，但本执行线没有启动任何真实进程，也没有以此错误伪称清理完成。
- 未创建 `evidence/raw/2026-07-23-l3-native-knowledge-real-app/`，因为没有可安全保存的真实截图、日志或 manifest。

## 7. 最小下一步

等待指导线独立验收本次 2 + 6 formatter diff、SHA 和第 7 节结果。即使 R3 离线门已通过，本线也不自行重新冻结或进入 Gate 0；不得以真实 App、扩大写面或放宽 supervisor 权限绕过此停点。

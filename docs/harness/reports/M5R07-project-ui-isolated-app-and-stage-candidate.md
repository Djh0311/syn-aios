# M5R07 项目 UI、隔离 App 与阶段候选报告

- 日期：2026-08-17
- 阶段：stage-14 / leaf M5R07
- 状态：**`AWAITING_INDEPENDENT_ACCEPTANCE`**
- 不宣布 M5 完成，不关闭 stage-14，不激活 M6

本包是对独立验收失败的最窄修正，不是 closeout。

## 候选载体

本工作树相对 `7a6598a` 只含 M5 修正。七个未跟踪 M6 文件与 `linux-schema.json` 仍原位未 add。

| 项 | 值 |
|---|---|
| 先前失败实现 candidate | `20740a8654ddddea08717800d9be0536d4b0021d` |
| 当前 main（修正前） | `7a6598a7ba58cecc5cbc61f228a3e512ff65b0b1` |
| 新实现 candidate | `65413a2d32830e689a6dc73ae34f75c4efbf223f` |
| Isolated launcher receipt | `docs/harness/reports/M5R07-isolated-app-launcher-receipt.json` |
| Disposable checkout | `/tmp/m5r07-disposable-65413a2` |
| Disposable receipt | `docs/harness/reports/M5R07-disposable-checkout-receipt.json` |

## 六个缺口的产品结果

1. **真实隔离 AppState / command / 项目壳**  
   `AppState` 在普通、隔离 profile 与验收构造路径安装 `{app_data}/m5/orchestration.sqlite`。六个正式 command 加四个隔离辅助 command 登记在 `command_registry.rs`。现有项目壳总览页挂 `ProjectSupervisorPanel`，经 `invoke` 调用，不是 crate 内测试直调。

2. **正式持久 RoleSession**  
   `m5_role_sessions` 持久化 M3 形 ID。turn / decision 经 `load_binding_by_id` 精确校验 binding + project + role_session + actor。调用方自造 `role_session_id` 被拒绝。

3. **正式授权链**  
   公开执行入口只有 `record_user_authorization_decision`：DRAFT proposal → 用户 `APPROVED`/`REJECTED` → Authorization → Grant → Dispatch。隔离场景与 UI 不再直接调用 `prepare_and_dispatch`。拒绝零 Grant。

4. **ProjectSummary 语义**  
   读取不重建。`SummaryStale` 返回已持久摘要且 `stale=true`。DTO 保留 `source_refs` 与 `syn://` deep link。

5. **两场景隔离 Tauri 交互**  
   `scripts/run-m5-isolated-app-acceptance.mjs` 铸造 R4 隔离 profile（隔离 app-data / scratch / fake runtime），拉起 debug 二进制 + 现有项目壳。`main.tsx` 驱动真实 DOM：只读对话、提出、拒绝、批准、摘要、只读 advice、deep-link 点击；强退后同 profile 恢复同一 binding / role_session。

6. **Receipt**  
   见本目录 JSON。disposable checkout 绑定命令、环境、退出码、opening/final hash 与 candidate SHA。

## 隔离场景事实

- Scene A：只读 chat + 用户 REJECTED，`spawned=false`，无 grant。
- Scene B：用户 APPROVED 后走正式链；followthrough 只在已有 Grant/Dispatch 上做 timeout→retry→echo、独立 review、重复 claim 幂等、摘要、只读 advice、deep-link 点击。
- Resume：`same_binding=true`，`same_role_session=true`。
- 窗口截图：在 `DISPLAY=:0` + `GDK_BACKEND=x11` 下写出 1920×1080 PPM。选中 drawable 是 X11 root（xid=1080，空 title），两帧 SHA 相同，**不得写成面板像素 PASS**。UI 交互证据是运行中 webview 经正式 command 写出的 scene/resume JSON。
- 旧入口：仍 guarded-legacy，未物理删除。

## 验证

- `cargo check --lib --offline`：PASS
- `cargo test --lib --offline -- m5_`：80 passed / 0 failed
- 完整 `cargo test --lib --offline`：不宣称 PASS。本环境观察到既有非 M5 失败：`conversation_transport_command_tests`（6）、`exec_process_registry` reaper（1）、`fix9_tests`（5）、`manual_relay::conversation_transport`（3）。另有 `manual_relay` 关机/杀进程测试在本环境无法跑完，不能冒充 M5 回归。
- `npm run typecheck`：PASS
- `npm run build`：PASS（310 modules）
- 隔离 Tauri 交互 / deep-link / 强退恢复：PASS（JSON receipts）
- 窗口截图：`EXECUTED_ROOT_ONLY / NOT_CLAIMED_PANEL_PIXELS`

## 交给总线

- Git：本地 `main` 相对 `origin/main` 超前；未 push、未 merge、未 rebase。
- Harness：唯一 current leaf = M5R07；authorization closed；stage-14 仍开。
- M6 / D0C04 / D0C05 / M1–M4 冻结合同未动。
- 请总线只读复核新的 M5-only candidate SHA，不要把本报告当成 M5 完成。

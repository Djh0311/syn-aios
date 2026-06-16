# Stage L / L3 Operation Control Hardening Result Handoff v1

日期：2026-06-16

状态：实现、本地验证与独立复核完成；Aquinas 复审 `STATUS: CLEAR`；提交前停止，不 `git add` / `git commit`。

## 当前状态

L3 已把 retry / stop / restart / resume 四个操作做成产品化控制面：

- 运行中工作流页显示四个 L3 操作卡片。
- 用户点击后进入 PermissionDialog 风险确认。
- 确认后调用 `record_operation_control_decision`，只向 workflow-state `audit_events` 写入 `operation_decision_recorded`，并登记为 `confirmed_recorded`。
- 不调用 runner、不执行真实 Codex、不停止 / 重启真实进程、不解锁 K3-B2。

## 四操作产品状态

- retry：可发起确认；真执行需另窗授权；L3 确认后只记录重试请求。
- stop：可发起确认；当前无 runtime handle；L3 确认后只记录停止请求，不 kill 进程。
- restart：可发起确认；重启语义未冻结；L3 确认后只记录重启意图。
- resume：可发起确认；real-resume 仍限既有 mario test / J1-B / J2-B 门；L3 不进入 phase B。

## 关键边界

- `session_continuation_store.rs::run_real_resume_phase_b_with_runner()` 未改。
- 未新增 `Command::new("codex")`、runner 调用、真实 process-control path。
- 新增 1 个 Tauri 命令，命令总数 97→98；该命令只写 workflow-state audit，不在 `lib.rs`，shape-gate warning 已分类。
- 未读写 `/Users/yoyi/.codex`，未读 secret/token/.env/keychain/OAuth/provider credential/full transcript/rollout/prompt body。
- readback 结果数保持 `null`，UI 显示未知 / 不可用，不显示 0 条。
- runtime log helper 定义了 operation decision 安全记录形状，但确认动作不自动写 runtime sidecar。
- memory capture 允许形成 observation/candidate 来源，但不自动写 FormalMemory。
- L3 离线交互断言已拆到 `tests/helpers/offlineL3OperationControlScenario.tsx`，主测试文件从超水线降到 3395/3404。

## 验证

已通过：

- `cargo test --lib operation_control`（6 passed / 0 ignored）
- `cargo test --lib workflow_audit`
- `cargo test --lib runtime_log_store`
- `cargo test --lib memory_capture_bus`
- `cargo test --lib page_read_model`
- `cargo test --lib`（516 passed / 21 ignored）
- `cargo fmt -- --check`
- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- shape gate（0 errors / 1 warning；warning 为 Tauri command 总数 97→98，已分类）
- `git diff --check`（空输出）
- capability scan（7 PASS / 0 FAIL）与 state-file guard（19 PASS）
- checkpoint-audit：`VERDICT: PASS`；因提交前停点无 commit / 无 CURRENT.md 条目，commit/current/allow-list 项为 NA，dirty tree 为预期 warning；review STATUS 解析为 `CLEAR`。
- 独立复核线 Aquinas（`019ece6b-4b39-7830-9553-86b979ec322c`）复审 `STATUS: CLEAR`，P0/P1/P2/P3 均 none；初审 P1 已修。

残余风险：

- 真实浏览器 smoke 未完成：Vite 需提升权限后可启动，但 Playwright 自带 Chromium 未安装，系统 Google Chrome headless 以 SIGABRT 退出。当前以 offline React render + typecheck + build 覆盖主路径，建议复核线按 P2 / residual risk 处理。

待执行于最终收尾：

- 主管线核实物并授权后再 `git add` / `git commit`；子线当前不提交。

## 下一步建议

当前可交主管线核实物。若主管线接受并授权提交，再 `git add` / `git commit`。若后续用户要真的执行 retry / stop / restart / resume，必须另开独立授权任务包，重新列 execution point、权限 envelope、审计、runtime log、readback、rollback 和用户确认。

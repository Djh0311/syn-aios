# Stage L / L1 K3-B1 Blocked Recovery Product Path Evidence v1

日期：2026-06-16

状态：独立复核线 Aquinas 复审 `STATUS: CLEAR_WITH_P2`。本文记录 `tasks/2026-06-10-stage-l-l1-k3-b1-blocked-recovery-product-path-v1.md` 的实现与验证证据。提交前停止，不 `git add` / `git commit`。

## 1. 执行摘要

L1 将 K3-B1 当前状态产品化为 `blocked_by_safety_review_again`：用户能看到阻断原因、合法恢复路径、手动回交边界、重新授权边界和 K3-B2 仍阻断。L1 不执行真实 Codex、不发送 prompt、不读写 `/Users/yoyi/.codex`、不启动 K3-B1 retry / K3-B2，也不把手动回交自动当成功。

一句话判据：如果 K3-B1 在产品层显示为安全审查再次阻断、只提供手动回交 / 重新授权申请 / 更窄桥设计三条恢复路径、K3-B2 仍阻断，并且验证证明没有真实执行路径被接通，则 L1 可进入复核。

## 2. 实际改动范围

产品代码：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/k3_b1_recovery.rs`：K3-B1 blocked recovery 读模型、9 个状态、K3-B2 gate 约束、手动回交不自动成功、敏感材料 guard、单测。
- 补充 `manual_exact_command` 契约：展示 K3-B1 冻结 env/test 命令、执行目录、prompt ref/hash 和“用户手动终端执行”边界；不包含 prompt body，不接工作台执行。
- 更新 snapshot / page read model：`WorkbenchSnapshot.k3_b1_recovery`、项目 / 智能体 / 运行中页字段覆盖。
- 更新 `workflow_audit.rs`：新增 `k3_b1_recovery_decision_recorded` audit 事件构造器，测试证明不存 prompt body / secret / `.codex` 原始内容。
- 更新前端类型与空 snapshot：增加 `K3B1RecoveryReadModel` 和两类非执行 pending action。
- 更新项目工作流右侧面板：新增 K3-B1 恢复卡片、手动回交待复核入口、重新授权说明入口；详情层只显示引用 / hash / 边界字段。
- 更新 `PermissionDialog.tsx` 与 `App.tsx`：两个 L1 action 只显示/记录 notice，不调用 Tauri command、不启动 runner。
- 更新 offline fixture 和 offline 交互测试：覆盖卡片主文案、禁用文案、result_count=null 显示为未知/不可用。

治理/测试结构：

- 新增 `prototypes/productized-desktop-shell/tests/helpers/offlineK3B1RecoveryScenario.tsx`，避免继续扩大 `offline-permission-dialog.test.tsx` 棘轮。
- `lib.rs` 维持 5,567 行水线；`ProjectsView.tsx` 降到 337 行；`offline-permission-dialog.test.tsx` 降到 3,403 行。

## 3. 边界确认

- 真实 Codex 执行：未执行。
- `codex exec` / `codex exec resume`：未执行；新增 L1 文案只声明“不执行 codex exec/resume”。`manual_exact_command` 只展示已冻结 env/test 命令供用户另行手动执行，`workbench_executes_in_l1=false`。
- prompt 发送：未发送。
- `/Users/yoyi/.codex`：未读写；新增模型只包含风险说明和禁止/边界文字，不读取内容。
- secret / token / `.env` / keychain / OAuth / provider credential / 完整 transcript / rollout / prompt body：未读取；模型和 audit 测试拒绝或不存这类材料。
- K3-B1 retry / K3-B2：未启动；K3-B2 gate 仍 blocked。
- 手动回交：只进入 `manual_recovery_needs_review`，不自动 accepted。
- FormalMemory：未自动写入；只给 candidate 文案边界。
- 真实浏览器验证：尝试启动本地 Vite 服务成功；Playwright 浏览器二进制缺失，系统 Chrome headless 在当前环境 SIGABRT，无法完成真实浏览器验收。已保留为残余风险，使用 offline React render + typecheck + build 替代覆盖 UI 文案和交互。

## 4. 验证原始输出摘录

Preflight：

```text
node scripts/harness/capability-scan.js --target .
PASS (7)
WARN (10)
FAIL (0)
```

```text
node scripts/harness/guard-state-files.js --target .
PASS (19)
envFiles: []
```

前端：

```text
npm run typecheck
> codex-governance-workbench@0.1.0 typecheck
> tsc --noEmit
```

```text
npm run test:offline-interaction
> codex-governance-workbench@0.1.0 test:offline-interaction
> node scripts/run-offline-interaction-test.mjs

offline interaction tests passed: 15
r4 page read model settings test passed
r4 page read model query contract test passed
r4 page read model runtime test passed
r4 page selectors test passed
```

```text
npm run build
> codex-governance-workbench@0.1.0 build
> tsc --noEmit && vite build

vite v7.3.3 building client environment for production...
✓ 252 modules transformed.
dist/index.html                   0.59 kB │ gzip:   0.42 kB
dist/assets/index-Cq18P1uG.css  145.61 kB │ gzip:  24.83 kB
dist/assets/index-C3PP4V6V.js   983.47 kB │ gzip: 268.75 kB
(!) Some chunks are larger than 500 kB after minification.
✓ built in 2.70s
```

Rust：

```text
cargo fmt -- --check
exit code 0
```

```text
cargo test --lib k3_b1_recovery
running 5 tests
test k3_b1_recovery::tests::l1_manual_exact_command_is_visible_without_prompt_body_or_workbench_execution ... ok
test k3_b1_recovery::tests::l1_recovery_model_covers_all_states_and_keeps_k3_b2_blocked_until_acceptance ... ok
test k3_b1_recovery::tests::l1_manual_submission_stays_pending_review_with_unknown_readback ... ok
test k3_b1_recovery::tests::l1_recovery_boundaries_reject_sensitive_material_and_never_store_prompt_or_codex_home ... ok
test workflow_audit::tests::k3_b1_recovery_audit_event_records_choice_without_sensitive_payloads ... ok
test result: ok. 5 passed; 0 failed; 0 ignored
```

```text
cargo test --lib workflow_audit
test result: ok. 3 passed; 0 failed; 0 ignored; 525 filtered out
```

```text
cargo test --lib page_read_model
test result: ok. 7 passed; 0 failed; 0 ignored; 521 filtered out
```

```text
cargo test --lib runtime_log_store
test result: ok. 1 passed; 0 failed; 0 ignored; 527 filtered out
```

```text
cargo test --lib memory_capture_bus
test result: ok. 7 passed; 0 failed; 0 ignored; 521 filtered out
```

```text
cargo test --lib real_execution_command
test result: ok. 36 passed; 0 failed; 7 ignored; 485 filtered out
```

```text
cargo test --lib project_workflow_automation
test result: ok. 15 passed; 0 failed; 4 ignored; 509 filtered out
```

```text
cargo test --lib
running 529 tests
test result: ok. 508 passed; 0 failed; 21 ignored; 0 measured; 0 filtered out; finished in 9.78s
```

Gates：

```text
node scripts/harness/workbench-shape-gate.js --mode check
Status: pass
Errors: 0
Warnings: 0
lib.rs: 5567/5567 (same)
offline-permission-dialog.test.tsx: 3403/3404 (decreased)
ProjectsView.tsx: 337/378 (decreased)
Tauri commands: 97 total; 0 in lib.rs
```

```text
git diff --check
exit code 0
```

## 5. 扫描分类

扫描命令按 `git status --short` 中的 21 个 modified / untracked 文件显式列出执行；此处修正了初审指出的 P2：不能只用 `git diff --name-only`，否则会漏掉未跟踪新增文件。

```text
rg -n --fixed-strings \
  -e 'K3-B1 retry 成功' -e 'K3-B2 可开始' -e '自动重试已启用' \
  -e '安全审查已绕过' -e 'result_count: 0' -e 'codex exec resume' \
  -e 'codex exec' -e '/Users/yoyi/.codex' \
  <explicit git-status modified and untracked file list>
```

分类结果：

- `K3-B1 retry 成功` / `K3-B2 可开始` / `自动重试已启用` / `安全审查已绕过`：命中来自 `offlineK3B1RecoveryScenario.tsx` 禁止文案断言，以及 handoff 的不可声称提醒；产品 UI 正向文案未把这些状态当成已发生。
- `result_count: 0`：当前改动文件无命中。
- `codex exec` / `codex exec resume`：命中来自既有 PermissionDialog 历史/权限文案、既有 lib 测试目标、L1 新增的“不执行 codex exec/resume”边界提示；未新增 `Command::new("codex")` 或真实 runner 调用路径。
- `/Users/yoyi/.codex`：命中来自既有权限文案 / guard、L1 风险说明、fixture 边界和 audit 不包含原始路径的断言；未新增读取或写入 `.codex` 的代码路径。

## 6. TDD / 验证说明

本轮接手时已有上一模型形成的红绿证据：

- Rust red：`cargo test --lib k3_b1_recovery` 因新符号缺失失败。
- Rust green：新增 read model 后 `cargo test --lib k3_b1_recovery` 通过。
- Frontend red：`npm run test:offline-interaction` 因缺少 `K3-B1 被安全审查再次阻断` 文案失败。
- Frontend green：新增卡片和 fixture 后 offline interaction 通过。

接手后又修复 shape gate ratchet：将 L1 offline 场景搬入 helper，保持断言不删；`ProjectsView.tsx` 改成 prop-through；`lib.rs` 保持水线。

## 7. 复核状态

独立复核文件：

- `evidence/2026-06-10-stage-l-l1-k3-b1-blocked-recovery-product-path-review-aquinas-v1.md`

Aquinas 初审结论为 `STATUS: FINDINGS`：

- P1：手动回交路径没有展示实际 exact command 或可执行命令引用。已修：新增 `manual_exact_command` 契约、UI 展示、Rust 单测和 offline 断言。
- P2：扫描命令不覆盖 untracked 新文件。已修：改为按 `git status --short` 中 modified / untracked 文件显式扫描并重新分类。
- P2：真实浏览器验证未完成。保留为 residual risk；offline render / typecheck / build 已覆盖主文案和入口，真实浏览器仍受当前环境阻断。

复审结论为 `STATUS: CLEAR_WITH_P2`：

- P0：none。
- P1：none；初审 P1 已修。
- P2：真实浏览器验证仍未完成，评为 residual risk，不阻断 L1 收口。
- P3：none。

主管线交回重点：

- 9 状态是否完整，K3-B2 是否仍 blocked。
- 手动回交是否只到 needs_review，不自动 accepted。
- UI 主层是否用户可懂，详情层是否不展开 prompt body / full transcript / `.codex` 原文。
- runtime / audit / readback / memory capture 边界是否不保存敏感材料。
- 是否未新增真实 Codex 执行、prompt 发送、`.codex` 读写、K3-B1 retry 或 K3-B2 启动。
- 验证与 shape gate 是否足够；浏览器验证环境阻断是否可接受为 P2 / residual risk。

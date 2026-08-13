# M4R07 普通产品隔离验收、全量回归与收口

阶段：stage-07 阶段7 M4 独立修正与再验收
目标：只做普通产品 composition 集成、故障恢复、可携带范围/receipt 证据、新鲜全量回归、独立审查和 M4 修正文档收口。
干完的标准：全新隔离 root 无 repository 预灌；普通 AppState/registry/dispatcher/scheduler/resolver/transport/readers 覆盖全部对象与五项 P1；SIGKILL/restart/duplicate/failure/rollback 无重复 effect；第 8 次 UI/Computer Use 验证按用户当前范围明确记为不执行；全量回归通过；stage-07 关闭后等待总线独立复核。

完成标记：仅仓库内 `docs/harness/reports/M4R07-isolated-product-reacceptance-closeout-behavior-receipt.json` 通过 v2 完整合同校验，且 v2 closeout manifest 精确绑定 portable receipt SHA 与 `launch_8_ui_validation` canonical SHA 时，才可判定本包在当前后端/产品链范围内 `PASS`。隔离 root 不发布正式 R07 PASS receipt；PNG、raw attestation、capture signal、root ready/ack 必须全部不存在；单独遗留的 manifest、临时文件或 stdout 均不是完成标记。该完成标记不是第 8 次 UI、截图、Accessibility 或 Computer Use 通过证据，也不得满足旧 v1/UI attestation 合同。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/m4_acceptance.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4r06_ordinary_legacy_read_driver.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4r03_ordinary_clock_driver.rs
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs
- prototypes/productized-desktop-shell/src-tauri/src/commands.rs
- prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs
- prototypes/productized-desktop-shell/src/App.tsx
- prototypes/productized-desktop-shell/src/main.tsx
- prototypes/productized-desktop-shell/src/components/WorkbenchShell.tsx
- prototypes/productized-desktop-shell/src/components/SecretaryBoardView.tsx
- prototypes/productized-desktop-shell/src/lib/tauri.ts
- prototypes/productized-desktop-shell/src/lib/types/
- prototypes/productized-desktop-shell/tests/
- prototypes/productized-desktop-shell/scripts/
- prototypes/productized-desktop-shell/package.json
- docs/current-state.md
- docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md
- docs/plans/2026-08-01-syn-stage-4-secretary-attention-and-daily-rhythm-plan-v1.md
- docs/plans/2026-08-11-syn-m4-independent-remediation-and-reacceptance-plan-v1.md
- docs/plans/README.md
- docs/task-queue.md
- docs/harness/
- handoffs/2026-08-10-syn-m4-to-m5-m6-m7-handoff-v1.md
- prototypes/productized-desktop-shell/dist/
- prototypes/productized-desktop-shell/src-tauri/target/

## 步骤

1. 用全新隔离 root 和普通产品 composition 覆盖全部 M4 对象、五项 P1 和旁路禁令。
2. 覆盖 transaction interruption、SIGKILL/restart、duplicate event/tick/message、fake failure 与 guarded rollback。
3. 保存脱敏可携带 v2 receipt 与范围 manifest，完成 Git/Harness、代码测试、文档下游三条独立复核。
4. 在新专用 TMPDIR 跑完整 Rust，并跑 typecheck、离线入口、production build、launcher syntax、rustfmt 与冻结 hash exact。
5. 只按实际证据同步 current-state/计划/交接，精确提交、归档叶与 stage，停止等待总线复核。

## 第 8 次 UI validation scope v1

- 固定 12 次与 `retry=0` 不变。第 8 次仍是普通 `recovery_timer`：完成真实 98 秒等待、后端 OPEN/FIRED 验证并写入该次 receipt；之后才允许第 9 次。取消的只是第 8 次 UI/Computer Use/截图/attestation gate，不取消产品恢复验证。
- portable v2 顶层只保存 exact `launch_8_ui_validation`：ordinal `8`、phase `recovery_timer`、`required_by_current_contract=false`、`execution_status=NOT_EXECUTED`、`acceptance_result=NOT_APPLICABLE`、reason `USER_SCOPE_EXCLUDED_LAUNCH_8_UI_VALIDATION`，并绑定第 8 次 receipt SHA 与真实等待秒数；Computer Use 次数为 0，截图、attestation、capture signal 均为 false。
- admission、12 次结束和 publication 三处都必须证明 repo portable/PNG/raw/manifest/fixed signal 以及 fresh-root ready/ack 不存在；当前 formal mode 不创建 capture contract、UI capture 子产物或握手文件，通用 evidence 目录仅用于成功后的 v2 closeout manifest。
- publication 使用 `syn.m4r07.closeout-evidence-manifest.v2`，先发布只含 schema、portable receipt SHA、`launch_8_ui_validation` canonical SHA 的 manifest，最后发布 portable receipt。旧 composite v1、旧 `ui_evidence`、旧 UI manifest v1、字段篡改或任何被取消的 UI artifact 一律拒绝。
- 因此当前合同的 `PASS` 只陈述隔离普通产品的后端/产品链结果；它明确不是第 8 次页面可见性、Accessibility、截图质量或 Computer Use 的 `PASS`。

## 2026-08-13 收口结果

- 正式 v2 receipt SHA-256 为 `854008d7fc304721f26fc3ffebf775424ffea43d234758721e87d6b87c8f30c8`，v2 manifest SHA-256 为 `1c6290cce97777a13403e0fbfad3dd0c11fb94a464f809da556b050694acd2e7`；manifest 精确绑定 receipt 与 `launch_8_ui_validation` canonical SHA。
- 新鲜隔离产品运行完成 12/12 次物理启动；第 8 次 `recovery_timer` 真实等待 98 秒并完成后端恢复，第 9 次只在其 PASS 后启动。UI / Computer Use / PNG / attestation 为 `NOT_EXECUTED / NOT_APPLICABLE`，Computer Use 次数为 0。
- `typecheck`、15-entrypoint offline interaction、production build、launcher/offline runner syntax、M4R07 focused contract、三份本轮 Rust 文件 `rustfmt --check`、五项 P1 green probe 与六份冻结合同 exact 均通过。production build 只保留既有大 chunk warning。
- 完整 Rust 在新专用 `TMPDIR` / `CARGO_TARGET_DIR` 中运行：沙箱内 1763 passed / 1 PID `lstart` 权限失败 / 45 ignored；同一唯一失败测试在主机权限下精确复跑 1/1 PASS，因此全部非 ignored 断言均通过。181 条仓库既有 warning 如实保留，不写成零 warning。
- 完整 Rust 留下的 fixture 仅位于本轮独占临时 root；确认无进程占用后已删除该精确 root。R01–R06 的 12 份历史报告/receipt 逐文件 bytes、SHA、mode、nlink 与正式 receipt `after` 投影一致。

## 实际改动与遗留

- 产品/验收字节实际修改：`scripts/run-offline-interaction-test.mjs`、`scripts/run-r4-isolated-app-preflight.mjs`、`src-tauri/src/m4_secretary_repository.rs`、`src-tauri/src/m4r03_ordinary_clock_driver.rs`、`src-tauri/src/m4r06_ordinary_legacy_read_driver.rs`、`src/main.tsx`，以及 M4R03/M4R04/M4R06/M4R07 四份 runner/contract test。
- 文档实际同步：`docs/current-state.md`、`docs/task-queue.md`、master/M4/remediation 三份计划、`docs/plans/README.md`、M4→M5/M6/M7 handoff、本 leaf；新增正式 v2 receipt 与 manifest。没有修改冻结合同、旧 R01–R06 证据或 stage-06 归档。
- 第 8 次 UI / Accessibility / screenshot / Computer Use 仍是当前用户范围排除项，不属于本次 PASS；真实资料、真实 provider/connector、远端、部署、发布、长期日用和 M5–M10 仍未验或未激活。
- 全仓 `cargo fmt --all --check` 仍会命中本 leaf 之外的既有格式债；本轮三份实际修改 Rust 文件的定向 `rustfmt --check` 已通过。`docs/workbench-system-architecture-v1.md` 仍有一处修正前的五缺口旧描述，但不在本 leaf 写面，作为 P2 文档漂移留给后续具名文档包。
- 5600X / WSL / Tailscale 只是用户提出的后续迁移方向；5600X 侧改稿尚未纳入本仓、未建立新 stage/leaf，也不由本次 closeout 自动执行。

# M2C01 M2 边界冻结与干净候选提交

日期：2026-08-08

结论：PASS。M2 的主线收口候选已从共享 dirty worktree 精确提取到独立分支；原工作树、index、分支头和 13 项战略 WIP 均未改变。

## 冻结输入

- 来源 worktree：`/Users/yoyi/workspace/product-line-syn-fnd-002`
- 来源分支/HEAD/tree/index：`syn-fnd-002-dev` / `2a7229bde7f0b5bb6701f4a7aa21944973f1881f` / `81528b95a8ac50416ab09f5a77a044601c48341f` / 同 tree
- 来源状态：64 tracked modified、14 untracked、0 staged
- status SHA-256：`60f1395f2df3b0e7355952b65f23bae4c121fbe63090802d8c4793178385f399`
- tracked binary diff SHA-256：`996452ac558c938d8558bd77c6d233be0d26f00d4989aa67a0dc773475086b2e`
- 当前 main 起点：`43578fd845f43e87154a04c0791bb25babea31e5`
- R4 原始 receipt：`acc05a13c791717b83d90ddd714717d8f4fc78121b46eb07579737ae999f876a`，schema `syn_m2_r4_reference_slice_launcher_receipt.v1`，7 个场景。

13 项战略 WIP 的 SHA-256 与 2026-08-05 / 2026-08-08 冻结记录逐项一致；其中 `docs/plans/README.md` 保持已知异常值 `4fa7337f8c979996dffb10983791c7c61131a5efe7a94e636a576d15ccb2c5cd`，未覆盖也未归责。

## 提取结果

候选 worktree：`/Users/yoyi/workspace/product-line-syn-m2-closeout`，分支 `codex/syn-m2-closeout`。

纳入 56 个实现前路径：

- 3 个 Code Map 文件：移除已退役 Harness 路径、登记 M2 grant ledger、修正 index 键；`development-harness.json` 使用 main 的 Lite 版本，没有从 dirty worktree 覆盖。
- 44 个已跟踪产品/测试文件：M2 reference slice、SQLite port/schema/storage、grant/report 边界、R4 profile/runner、必要的 legacy caller 与类型适配。
- 2 个新增 Rust 文件：`m2_clock.rs`、`m2_r4_reference_slice_driver.rs`。
- 1 份 DAT-001B 合同、1 份 umbrella task 和 9 份 value-free / isolated acceptance 证据。

明确排除：

- 13 项战略愿景与状态 WIP，包括 dirty worktree 中的旧 `docs/harness/AUTHORITY.md`、`docs/harness/CURRENT.md`。
- `docs/code-map/domains/development-harness.json` 的旧 Harness 改动。
- `codex_db.rs`、`codex_local_runner.rs`、`mcp/event_audit_boundary.rs`、`mcp/identity_kernel.rs`、`mcp/path_guard.rs`、`mcp/storage.rs`、`mcp/supervisor_conversation_binding.rs` 的纯格式化相邻改动。
- live Workbench、`~/.codex`、provider、M3/M5 语义和旧 Adaptive Harness runtime。

generic `m2_ports` / `m2_outbox` / `m2_projector` / `m2_legacy_adapter` / `m2_domain_cutover` 仍为私有、非权威候选，不计作完成积分；它们只随 M2 内部类型一致性进入本提交。唯一具名完成 port 仍为 `workflow-state-sidecar.repository.m2.v1` 的 concrete SQLite implementation。

## 候选验证

- `cargo check --lib --quiet`：exit 0。
- `cargo test --lib m2_reference_slice --quiet`：11 passed / 0 failed。
- `cargo test --lib worker_report --quiet`：31 passed / 0 failed。
- `cargo test --lib workbench_sqlite_schema_m2::tests --quiet`：7 passed / 0 failed。
- `cargo test --lib m2a_execution_report_ingress_tests --quiet`：2 passed / 0 failed。
- 借用来源 worktree 已安装的 `node_modules` 运行 `npm run typecheck`：exit 0；临时依赖符号链接已移出候选 worktree。
- `git diff --cached --check`：PASS。

这些检查只证明候选提取没有破坏 M2 直接路径；完整库测、真实 R4 重跑、Harness task/quick 和 main 集成属于 M2C02。

## 保全

提取后再次复核来源 worktree：HEAD/tree/index、64+14 计数、status/diff SHA-256 和 13 项 WIP 哈希均与开工冻结一致。未在来源 worktree stage、commit、reset、clean、stash 或改 ref；未 push。

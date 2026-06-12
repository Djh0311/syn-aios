# 复核结论：Root Treatment / R2-T12 Rust Task Package Preview Binding Read Model Test Extraction v1

日期：2026-06-12

Reviewer：Claude（claude-opus-4-8，复核线临时代班，依据 `handoffs/2026-06-12-review-line-temporary-takeover-claude-v1.md`）

复核对象：

- 任务包：`tasks/2026-06-12-root-treatment-r2-t12-rust-task-package-preview-binding-read-model-test-extraction-v1.md`
- 实现 evidence：`evidence/2026-06-12-root-treatment-r2-t12-rust-task-package-preview-binding-read-model-test-extraction-v1.md`
- 结果 handoff：`handoffs/2026-06-12-root-treatment-r2-t12-rust-task-package-preview-binding-read-model-test-extraction-v1-result.md`

复核基线：工作区未提交改动（Implementation commit 待用户放行后回填）；Planning baseline `435c21471ad056bd7ed1b44681ad52f285883b5c`（= 当前 git HEAD）。

性质：只读独立复核。本结论文件是复核线唯一产出；复核线不改产品代码/任务包/evidence/权威文档，不跑 `git commit`；发现问题只列不修。

---

## STATUS: CLEAR

- P0：无
- P1：无
- P2：无

R2-T12 的 11 个 preview/binding/read-model inline tests 迁移为行为保持的纯搬运：脚本门全绿、删除块与新 include 字节级一致、迁移名单无多无少、禁迁清单零命中、产品代码零改动、waterline 正确锁到历史新低 6006。未发现任何 P 级问题。

---

## 1. 独立重跑的脚本门（本地全量复跑，非转述 evidence）

- `cargo test --lib`：`471 passed; 0 failed; 16 ignored`——与 R2-T11 收口基线一致，无测试丢失或新增。
- 定向切片（与 evidence §5 每条申报逐一吻合）：`work_item_state_update` 3 passed、`workflow_node_session_binding` 2 passed、`task_package_preview` 6 passed、`workflow_task_package_read_model` 1 passed、`project_blackboard_read_model` 1 passed；全部 0 failed。
- `cargo fmt --manifest-path prototypes/productized-desktop-shell/src-tauri/Cargo.toml -- --check`：通过（FMT_CLEAN）。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings；`lib.rs: 6006/6006 (same)`，ratchet policy `historical_lowest_closed_value`。
- `git diff --check`：干净（exit 0）。
- 方法注：`cargo fmt --check` 与 `git diff --check` 都不覆盖未跟踪的新 include 文件（rustfmt 不跟进 `include!`；diff-check 不扫未跟踪文件）。新文件清洁性由第 2、3 节字节对账 + 尾随空白扫描独立确认。

## 2. 对账（任务包声明 vs 真实工作区 diff）

- 行数收益：`git diff --numstat` 显示 `lib.rs` 仅 `1 added / 539 deleted`（净 −538），无散落改动；`wc -l lib.rs` = 6006。声明 6544→6006 降 538 行，一致。
- 新文件规模：`wc -l` 新 include = 539，低于 `.rs` 新文件 3000 上限，一致。
- waterline：shape gate diff 仅 `lib.rs` 一行 `6544 → 6006`；实测 6006/6006，已锁历史新低、未放松，一致。
- 字节对账：从 `git diff` 提取的 539 行删除块与新 include 文件 `diff` 结果为 `BYTE_EXACT_MATCH`。直接坐实"行为保持纯搬运"——新文件逐字节等于被删块。
- include 锚点：`lib.rs:3411` 为 `include!("lib_task_package_preview_binding_read_model_tests.rs");`，与 §6 声明的 3411-3949 起点吻合。

## 3. 迁移纯度

- 迁移名单：新文件 11 个 `fn`（`#[test]` 计数 = 11），逐一核对 = 任务包 §2 白名单 11 个，无多无少。
- `#[test]` 守恒：HEAD `lib.rs` 55 个 → 工作区 `lib.rs` 44 个 + 新文件 11 个；55 − 11 = 44，守恒。
- 禁迁/helper 扫描：对新文件扫 `workflow_node_dispatch_(prepare|started|execute|readback|failure|timeout|permission)`、`workflow_execution_runner`、`workflow_machine`、`memory_candidate_adoption`、`memory_candidate_rejection`、`formal_memory_store`、`formal_memory_adoption`、`k3_b_`、`reads_real_static`、`prepare_offline_role_dispatch`、`record_offline_role`、`fn fixture_`、`mark_task_package_fixture_ready`、`append_fixture_dispatch`、`stub_runner|StubRunner|RunnerFake`——零命中。T0"不得迁移"清单未触碰；共享 helper 仍留在 `lib.rs`。
- 新文件清洁：无尾随空白、无 `#[ignore]`、EOF 单一换行（字节对账已涵盖）；R2-T11 同类 EOF P2 在本包未复现。
- 产品语义：numstat 证 `lib.rs` 仅"删测试块 + 插 include 行"，产品函数签名/可见性/语义零改动；shape gate `Tauri commands: 0 in lib.rs`、`Sidecar JSON: 0 unknown`，无 schema/command/sidecar 变动。

## 4. §6 候选评估对账（按当前磁盘核对，非开机快照）

- 当前任务包 §6：选中 11 / 禁迁 33 / deferred 11 = 55，与 HEAD `lib.rs` 实测 55 个 inline `#[test]` 完全吻合。
- 留痕：复核线开机快照读到的 §6 曾为"禁迁 共 32"。按 takeover 约定"收口前重读磁盘"复核当前状态，§6 已为 33，且任务包正文 / evidence / result 三处一致——主管线自查抓出的 32→33 计数笔误确已落到任务包正文（并在任务包 line 131 留痕）。故此项无 P 级问题。

## 5. 边界与流程确认（据 evidence/result 留痕，复核侧以工作区改动面交叉验证、无反证）

- 工作区改动面仅为：Rust 测试搬运（`lib.rs` + 新 include）+ shape gate waterline + R2-T12 文档（task/evidence/result）+ 接管文档 + `CLAUDE.md`。无 `~/.codex` 写入、无 sidecar/schema/Tauri command 改动、无 UI/CSS/TS 改动。与"无真实 Codex 执行、无 prompt、无 secret 读取、无 Tauri/截图"的边界声明一致。
- 一次接管文档入库的 `git commit` 被 harness 权限层按 AGENTS.md 拦截、无仓库变更——符合"commit 需用户放行"。
- 主管线在档案更新前自派的同模型只读审查已显式降级为"主管线自查"、不充当正式复核——无职位越界。正式复核即本文件（独立复核线会话，claude-opus-4-8，相对档案设想的 fable-5 主管线为跨模型）。

## 6. 复核边界声明

- 本文件为复核线唯一产出。未改任何产品代码/任务包/evidence/权威文档，未跑 `git commit`。
- 第 1 节五门均为复核线本地独立重跑结果，非转述 evidence。
- 结论仅针对 R2-T12 任务包 §2–§5 的验收口径；不接受为任务包 §5 列明的任何外延（`lib.rs <= 3,000`、R2 完成、R3 Level B、生产 SQLite 迁移/read-cut/stop-write、多 agent 并行真实执行解锁、真实 Codex 执行、产品语义变更等）。
- 本结论将作为 Codex 额度恢复后"换脑抽查"事后复检的输入之一。

# M6P00 canonical ProjectId 消费扩面与 relation owner 类型化前置

阶段：stage-15 M6 全局主管与内部组织（域层先行，UI 验收载体为新壳）

状态：`CURRENT` / `NOT_ACCEPTED` / `NOT_CLOSEOUT`。stage-15 于 2026-08-18 建立并 active；authorization closed；M6 域层各叶、F2/F3/F5 与壳采纳均未激活。

来源收据：`M5R09-20260818-1836.verdict.md` 欠账 2、4 与 2026-08-18 18:40 用户 closeout 纪律把本项定为 M6 前置（当时判为“不修也不至于对真实用户不可用”，因此没进 M5 返修）；用户 2026-08-18 21:49 明确“接下来就是 M6”，并批准总指导的排法：M6 域层施工前先做本前置。交接输入见 `handoffs/2026-08-18-syn-m5-to-m6-and-shell-deferred-debts-v1.md` 第 1 节。

目标：让 canonical `ProjectId` 成为项目身份在正式读写入口上的唯一命名空间，并让 memory relation 的 source owner 校验建立在可判别类型上，而不是字符串巧合。本叶只补前置，不做 M6 域层实现，不改 M5 已接受的执行合同语义。

已核实的起点事实（施工前须自行复核，不得照抄）：

- canonical 消费面目前只落在 `mature_pattern_governance.rs`、`memory_entity_relation_governance.rs` 与 `commands.rs` 的部分入口（`TrustedCanonicalProject`）；
- `global_supervisor_agent.rs` 仍有多处 `crate::project_id(project_root)` 路径派生（约 144、485、814、908、968、1396 行），是 M6 跨项目查询会直接消费的面；
- `m2_r4_reference_slice_driver.rs` 有一处路径派生比较；
- `memory_entity_relation_governance.rs` 内的路径派生部分属 legacy 兼容比较，是否保留须逐处判定并说明理由，不得一律替换。

做完的标准：

1. canonical `ProjectId` 的消费面从现有面扩到 workflow、项目编排与执行链的正式读写入口；扩到哪些文件、哪些入口、为什么这些是“正式读写入口”，须在本叶报告里逐条列出并与代码位置对应；
2. M6 跨项目查询路径不得同时消费 canonical id 与 path-derived id 两套命名空间；仍需保留的 legacy 比较必须显式标注为 legacy 兼容并给出保留理由与失效条件；
3. memory relation / relation candidate 的 `source_id` 建立可判别的 source kind / owner 类型边界：只对明确属于 project owner 的 source 执行 canonical/legacy 校验；foreign project owner 在业务写前 fail-closed；合法 doc/tool/session source 不被误拒；
4. 定向测试覆盖迁移、重启后同一解析、跨项目拒绝、mixed owner 零部分写，以及一个 M6 ProjectSummary 查询反例；
5. 不改 M1–M5 冻结合同正文与旧 hash；解释或扩展语义只能新建增补合同；
6. `cargo check --lib --offline` 与本叶相关定向测试在 disposable checkout 上通过，证据绑定候选 SHA；
7. 独立内容提交，写域精确，`git diff --check` 通过；
8. 到此停下写节点请求文件，等独立验收，不自行进入 M6 域层第一叶。

证据：只在 disposable checkout 上产出定向证据，绑定候选 SHA。本叶不做 GUI、不做窗口截图、不做 computer use、不接真实 provider 或账号。

载体：本叶新增的增补合同（如需要）、下列写域内的源码改动、任务包与本叶报告；一次独立内容提交加一次记账提交。

允许动：

- `docs/contracts/`（仅新增增补合同，不改冻结合同正文与旧 hash）
- `prototypes/productized-desktop-shell/src-tauri/src/m1_project_index.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/global_supervisor_agent.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_governance.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_governance.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/m2_r4_reference_slice_driver.rs`（仅路径派生比较必须同步时）
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`（仅 canonical 解析与校验接线）
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`（仅 canonical 解析接线与必要 `AppState` 接线）
- workflow / 编排入口文件中经本叶报告点名的具体文件（`workflow_execution_entrypoints.rs`、`workflow_run_dispatch_entrypoints.rs`、`workflow_read_model_entrypoints.rs`、`project_workflow_automation.rs`、`m5_orchestration_store.rs`、`m5_orchestration_schema.rs`），**仅** canonical ProjectId 的解析与校验接线；不改执行合同语义、不改 Grant/receipt/audit/quarantine 边界
- `tasks/2026-08-18-*`、`tasks/2026-08-19-*`
- `docs/harness/stages/stage-15.md`、`docs/harness/leaves/`、`docs/harness/unfinished/`、`docs/harness/audit/2026-08.jsonl`、`docs/harness/reports/M6P00-*`、`docs/harness/plan.md`、`docs/current-state.md`

不许动：

- M5 已接受的执行合同语义：ExecutionGrant、WorkerReport、receipt/audit/quarantine 不得放宽；`m5_runner_entry_registry` 的 `new-grant / guarded-legacy / blocked` 分类不得改判，guarded legacy 不得升格
- M1–M5 冻结合同正文与 hash
- 6 个未跟踪 `m6_*.rs`（含 `.bak`）与 `gen/schemas/linux-schema.json`：只读保全，不暂存、不清理、不作为实现输入
- 页面布局、旧壳 UI、`syn-shell` 仓库、F2/F3/F5、壳采纳
- stage-12、`unfinished/D0C04`、`unfinished/D0C05`
- OSS-01 与用户自有门面载体（已由 `c1025ba` 独立提交）
- 自行关闭 stage-15、宣布 M6 完成或进入 M6 域层第一叶
- 伪造 receipt、authorization、stage/leaf、测试或 App 证据
- push、merge、rebase、部署、发布

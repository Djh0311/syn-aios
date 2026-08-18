# M6P00 candidate and evidence v1

状态：`SUPERVISOR_SELF_REVIEW_PASS / CHECKPOINT_PENDING / NOT_STAGE_ACCEPTED / NOT_RELEASED`

## Harness

- stage：`stage-15`；leaf：`M6P00 canonical ProjectId 消费扩面与 relation owner 类型化前置`；执行来源 receipt：`u-bf5cbe4117b9c087c7f1`。
- 内容候选：`4147454bc046d5a5d3047799725d9e77ed086179`；tree：`69816100d15c449b16faef08deda1fc37af48df5`。产品实现由 Grok 优先、Codex 接管不收敛部分并完成复核；提交按事实同时署名。记账提交只由 Codex 完成，其精确 SHA/tree 写入仓外交包。
- 本叶已原子归档到 `docs/harness/done/2026-08/`；`docs/harness/leaves/` 留空；`authorization.json` 为精确 closed 两字段。M6D01 在独立检查点 PASS 前仍留在 `unfinished/`。
- 内容提交实际写域共 12 路径：7 个允许的产品源文件 `global_supervisor_agent.rs`、`memory_entity_relation_governance.rs`、`project_workflow_automation.rs`、`workflow_execution_entrypoints.rs`、`workflow_run_dispatch_entrypoints.rs`、`commands.rs`、`c4_c6_workflow_governance_entrypoints.rs`，以及 4 个 `tasks/2026-08-19-*` 与 1 个 `tasks/2026-08-18-*` 任务包。未新增或修改冻结合同。
- 第一次检查点 verdict 对产品与第 2–7 项全部判为 PASS，只因初版记账 `6459b30` 带入未列入 allowlist 的 supervisor handoff 而判 FAIL。返修采用路线 A：保持内容 tree 不变，把 handoff 从本叶净记账中完全摘出，并纠正本报告的 warning、dead wrapper 与任务日期表述；新的记账 SHA/tree 以重交包为准。
- 自复核结论：七项全部放行；这里只签发主管自复核，不代替 Cursor Opus 检查点 verdict，不关闭 stage-15。

## 产品

正式读写入口及 canonical 接线如下。它们是注册的 Tauri command 或其普通产品调用链上的实际授权、查询、持久化与派发路径，不是只在 `#[cfg(test)]` 中可达的 helper：

- `global_supervisor_agent.rs:566`、`:904`：B1/B2 全局主管查询从 M1 index 解析 canonical project owner，再进入 supervisor read model；M6 ProjectSummary 反例证明同 alias 的 path-derived owner 不会被误选。
- `commands.rs:1757`：supervisor conversation context；`:5922`：plan user confirmation；`:6131`：project workflow Phase A；`:7106`：dispatch readiness；`:7351`：session bind；`:7538`：workflow dispatch execute；`:8546`：workflow list；`:8610`：draft submit；`:8897`：workflow nodes；`:9023`：director review；`:9086`：offline prepare；`:9125`：offline handoff；`:9168`：offline review。以上 wrapper 都先以 AppState 的 M1 index 做 exact alias canonical resolution，再把 canonical id 传入下层读写。
- `memory_entity_relation_governance.rs:1350`：relation source owner classifier 建立 `project_owner` 可判别边界。明确 project owner 才接受 canonical id 或精确 legacy-v1 兼容；foreign project owner 在业务写前 fail-closed；knowledge document、tool 与 session source 保持合法。
- `project_workflow_automation.rs:110`：Phase A core 只接受上层解析出的 canonical id，拒绝 caller claim 不一致且零写入。
- `c4_c6_workflow_governance_entrypoints.rs:575`：project director process-fact observation 的正式 command 路径增加 canonical project owner exact join；旧 C4 preview/prepare 行为按本叶写域约束保持不变。
- `workflow_execution_entrypoints.rs:1483`：canonical workflow/node/work-item/binding/dispatch/task-package selector；同 key foreign owner 不被选择，实际 mutation 与 readback 均绑定 canonical owner。

仍保留的路径派生都被分类而非冒充 canonical 正式面：

- `commands.rs:7807` 的 `execute_project_workflow_node` 明确标为 guarded legacy，只能命中固定 workflow-engine 测试项目；其下游临时 work item、guard 与 prepared dispatch 的 path-derived owner 仅维持旧 fixture 兼容。失效条件是该命令要面向普通项目解封，届时必须先改为 M1 canonical resolution。
- workflow execution/run-dispatch 中仍被引用的未带 canonical 参数 wrappers 只供既有测试或 guarded legacy caller；正式 Tauri wrappers 已改走 `_with_canonical_project_id`。另有 16 个被 canonical 版本取代的旧 wrapper 已完全无调用者并产生 `dead_code` warning，作为 ENG-01/cutover 后清理欠账记录，不在 M6P00 返修中删除。
- relation 的历史 v1 project source 接受精确 `legacy_project_id` 只为已有记录迁移；未知/foreign owner 不做 fallback。失效条件是历史 source migration 完成并切断 v1 reader。
- C4 preview/prepare 的旧 path-derived 比较属于 M5 既有项目主管入口，本叶只获准给 process-fact observation 增加必要相邻接线；它不是 M6 跨项目查询输入，旧行为保持并在后续 command cutover 时迁移。

## 证据

原始证据根：`/home/synadmin/workspace/.syn-gates/evidence/M6P00-4147454/`。全部产品验证在 detached candidate `/tmp/syn-verify-4147454` 上运行，clean baseline 在 `/tmp/syn-m6p00-baseline`（原 HEAD `8e73ff4`）上运行。

- `cargo check --lib --offline`：exit 0；clean HEAD `8e73ff4` 为 883 warnings，候选为 897，本候选净增 14 条 `never used`/dead wrapper warnings；未写成零 warning。
- `cargo test --lib m6p00_ --offline`：21 passed / 0 failed，exit 0。
- `cargo test --lib global_supervisor_ --offline`：33 passed / 0 failed / 2 ignored，exit 0。
- `cargo test --lib memory_entity_relation_ --offline`：19 passed / 0 failed，exit 0。
- `cargo test --lib project_workflow_ --offline`：51 passed / 0 failed / 6 ignored，exit 0。
- `cargo test --lib workflow_node_dispatch_ --offline`：14 passed / 0 failed，exit 0。
- `cargo test --lib offline_role_ --offline`：3 passed / 0 failed，exit 0。
- `git diff --check`：exit 0。
- 补充回归 `conversation_transport_`：候选与 clean HEAD 都是 22 passed / 6 failed、exit 101，且失败的 6 个测试名与消息完全相同。这是新鲜记录的既有基线红灯，不计作本候选引入，也不伪装成绿色测试。

主管七项判据：

1. 写域：内容提交的 12 个路径逐项落在 leaf allowlist。修正后的净记账只含 7 个允许路径：`docs/current-state.md`（stage 与 leaf 均允许）、`docs/harness/audit/2026-08.jsonl`（stage 与 leaf 均允许）、`docs/harness/authorization.json`（stage 与 leaf 均允许）、leaf 从 `docs/harness/leaves/` 到 `docs/harness/done/2026-08/` 的原子移动（stage 与 leaf 的 lifecycle 写域）、`docs/harness/plan.md`（stage 与 leaf 均允许）、`docs/harness/reports/M6P00-*`（stage 与 leaf 均允许）、`docs/harness/stages/stage-15.md`（stage 与 leaf 均允许）。supervisor handoff 相对内容父提交零 delta。
2. 冻结物：内容提交未改 `docs/contracts/`，M1–M5 冻结合同正文与旧 hash 无 delta；ExecutionGrant、WorkerReport、receipt/audit/quarantine 与 guarded-legacy 分类未放宽。
3. WIP 保全：主工作树 status 只观察未归属 WIP；6 个 `m6_*.rs`（含 `.bak`）和 `gen/schemas/linux-schema.json` 仍未跟踪、未暂存、未提交、未作为实现输入。disposable build 生成的 schema 只存在于验证载体，不归入候选。
4. 独立重跑：候选 SHA 与 tree 写入证据根，所有上列定向命令在 detached checkout 重跑；clean HEAD 单独复现 conversation baseline 红灯。
5. 实质：普通 AppState/Tauri command 调用链真实消费 M1 exact-alias resolution；canonical id 进入实际查询、授权、写入、派发和 readback；foreign same-key 反例验证零误选/零部分写，不是测试专用空壳。
6. 不越级：证据只到 WSL 本地、synthetic/disposable 产品链；没有 GUI、窗口像素、Computer Use、真实资料/provider/账号、外部业务写、部署或发布结论。
7. 欠账：conversation transport 既有 6 个失败留作独立后续基线债；16 个完全无调用者的旧 wrappers 与其新增 14 条 warning 随测试迁移/command cutover 进入 ENG-01；隔离档未安装 M1 authority 会 fail-closed，须在 ORG-007 新壳验收前裁定；M6D08 纳入 M1 restart 回归；固定测试项目 guarded legacy 与旧 C4 preview/prepare 随后续 cutover 处理。M6D01–M6D08、M6 UI 与 ORG-007 均未开始。

## 载体

- 产品内容载体是候选 `4147454`；本报告、归档 leaf、stage/plan/current-state、audit、authorization 与 stage 检查点协议修订属于独立记账提交，不改变产品行为。supervisor handoff 不在本叶净记账写域。
- 证据日志位于仓外 `.syn-gates/evidence/M6P00-4147454/`；检查点交包位于 `.syn-gates/open/`，独立 verdict 只能由 Cursor Opus 验收官写入 `.syn-gates/verdicts/`。
- 当前结论为 `M6P00 SUPERVISOR SELF-REVIEW PASS / CHECKPOINT PENDING`，不是检查点 PASS、M6 域层完成、stage-15 closeout、GUI/新壳验收、真实运行、部署或发布。

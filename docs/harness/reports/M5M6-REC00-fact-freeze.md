# M5/M6 REC-00 事实冻结与路由报告

- 日期：2026-08-16
- 阶段：stage-14（真实活动阶段），唯一 current leaf：REC-00-m5-fact-freeze-git-and-harness-rebuild
- 计划：`docs/plans/2026-08-16-syn-m5-m6-fact-reconciliation-and-product-closure-plan-v1.md`
- 状态：`FACT_FREEZE_COMPLETE / M5R00=NOT_NEEDED / ROUTE=M5R01`

## 1. R0 恢复载体

- 工作根：`/home/synadmin/workspace/.m5-rec00-work`（独立临时目录，位于仓库外）
- `r0b-archive.tar`：215,818,240 bytes，SHA-256 `77c2837ef967a99298113c7496c1cccd5965316d7186d3b59a4875ebd10629a7`，3609 项常规文件
- `r0b-manifest.tsv`：3609 行 sha256/mode/size/path
- 独立验证目录全量解包后 `sha256sum -c` 全部一致（ALL HASHES MATCH）
- 排除项只记元数据：`.git`（67M）、Rust `target/`（7.2G）、`.claude/.harness-lite.rollback-Go9ehz/`（旧 rollback carrier，29 文件）
- 现场无 `.env`/secret/credential/DB/node_modules/symlink/special file；tracked symlink 0，untracked 无特殊文件

## 2. 锚点与工作树事实

- branch=`main`，HEAD=`9103c3b26b060e854be119a8cedaa856a2a900ce`，index=`4080362684d53e6ccc845de87827ec722490efae`，index 相对 HEAD 无 staged delta；本地 `origin/main` 与 HEAD 一致（未联网，不 fetch）
- tracked 3581 / untracked 54 / ignored 7458；工作树变更 37 modified、26 deleted、54 untracked
- 所有 M5/M6 候选字节（`m5_*.rs`、`m6_*.rs`、`worker_report.rs` 扩展、`lib.rs` mod 声明、`tasks/2026-08-16-syn-prj-001-*.md`）均为 uncommitted 候选 WIP

## 3. bootstrap transaction

- 保存 preimage：authorization.json、plan.md、current-state.md、audit jsonl、usage/.turn（SHA-256 见 provenance）
- 旧 malformed authorization（legacy 8-field，`authorized:true`）字节与 SHA 已冻结并标注“无效控制状态”，不冒充历史授权
- authorization 恢复为 Lite 0.8 精确 closed 两字段，runtime `read()` 验证 `{kind:"closed"}`
- `docs/harness/plan.md` 撤销 stage-14/15 假完成勾选，57/33 测试改写为候选原型单测基线（未绑定阶段退出）
- `docs/current-state.md` 统一回写为 `M5/M6 CANDIDATE WIP / NOT_ACCEPTED / NOT_MAINLINE`
- 真实 `stage-14` 与唯一 REC-00 current leaf 建立（closed auth）；runtime `status` 验证：`currentCount=1`、唯一 leaf `stageId=stage-14`、selected stage=stage-14、stage-12 与 stage-14 并存、D0C04/D0C05 仍在 unfinished 且 `stageId=stage-12`
- 未手填 executionReceipt/session/turn/expiresAt；未执行任何 Git add/commit/push/merge/rebase/reset/stash/clean
- 全程未读 secret/credential 内容；stage-12、D0C04/D0C05、M1–M4 冻结合同零修改

## 4. 分层归责

初版分类（详见 `M5M6-REC00-provenance.json`）：L2=76（Harness Lite 0.8、Hook、D0 closeout、控制文件）、L3=14（D1A1 + 架构/计划文档）、L4=6（M5 候选）、L5=6（M6 候选）、LX=15（共享/重叠，需 hunk 级投影）。LR=无。L1–L3 commit/tree 载体在 3.4 阶段形成后以 finalized 补入，初版不预填。共享文件（lib.rs、worker_report.rs、Cargo.toml/lock、plan.md、current-state.md、docs/*）在候选 commit 前必须按 hunk 精确投影。

## 5. 前置矩阵（fresh, static, 判定不施工）

| 项 | 判定 | 证据 |
|---|---|---|
| M1：Project/scope/workflow owner 精确 | PASS | `stable_id` 确定性 `project:{id}`；`list_project_workflows` 只认 `project_id` 精确匹配；node read model 按精确 project_id 解析 project_root；plan_authorization_store 强制 scope 的 project_id/workflow_id 与授权对象一致 |
| M1：`list_project_workflows` 不再用 slug `contains` 猜归属 | PASS | commands.rs SYN-FND-004A 注释确认 contains 兼容判定已删除；存在 `list_project_workflows_excludes_old_record_without_project_id` 测试 |
| M1：node query 不丢 project root | PASS | workflow_read_model_entrypoints `project_workflow_summaries` 精确 join project_id→project_root |
| M1：Proposal/Authorization/ExecutionGrant/Report owner 唯一 | PASS（基础）/ 精确 join 冻结归属 M5R01 | M1 store 有 owner/scope 强制；WorkerReport 完整 exact-join 缺口是 M5R01 修正对象，不构成前置阻断 |
| M2：UoW 可供 M5 production adapter 使用 | PASS | m2_ports `UnitOfWork` + WorkbenchSqliteRepository 事务化 mutation（2485/2508/2574 行） |
| M2：outbox 可供使用 | PASS | m2_ports `OutboxRepository`/`OutboxClaimer` + m2_outbox `OutboxProcessorImpl`（claim/release/lease/retry/poison） |
| M2：effect/readback 可供使用 | PASS | m3 `provider_effect_attempts` + M2 effect ledger + readback 路径存在 |
| M2：失败原子性 | PASS | repository 事务操作与 rollback 语义；M2 验收已证明幂等与失败原子性 |
| M3：RoleSession/repository/Handoff 唯一身份真源 | PASS | m3_role_session_schema.rs 生产 SQLite schema（11 表）+ m3_role_session.rs 域 + m3_handoff.rs；M3 stage-05 已主线验收 |

结论：无前置 GAP → **M5R00 = NOT_NEEDED**（不制造空施工）。M5R01 将冻结完整 identity owner/join 并对现有原型做 KEEP/REWRITE/QUARANTINE 裁决。

## 6. 事实与证据上限

- 本次冻结只建立“可恢复、可归责、可逐层提交”的事实基线；M5/M6 候选仍为 `CANDIDATE WIP / UNIT-LEVEL PROTOTYPE / NOT_ACCEPTED / NOT_MAINLINE`
- 本报告所有结论为静态合同/代码证据，不等于 production service 已接入
- 下一步：REC-00 退场（closed auth）→ 建立 M5R01 current leaf（closed auth 起步）→ 顺序 M5R01–M5R07 → 候选 → 独立验收

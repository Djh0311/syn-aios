# SYN M2 总派发 Kickoff · v1

date: 2026-08-03
status: ACTIVE — 新会话拿本文件即可执行整个 M2，不需要其他上下文。
author: 总指导线（经 M1 全程验收）

---

## 0. 你是谁、先读什么

你是 Syn（个人 AI 工作台）M2 阶段的执行线。权威顺序：

1. 当前用户指令
2. 本文件
3. `docs/harness/CURRENT.md`（活正本，每次工作完必回写）
4. M2 阶段计划：`docs/plans/2026-08-01-syn-stage-2-fact-event-audit-transaction-foundation-plan-v1.md`（切片/顺序/退出的最终依据，本文件是它的执行化摘要；冲突时以它为准）
5. master plan：`docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md`
6. 项目规则：`AGENTS.md`（全程中文；commit 问一次；commit message 必带 `catch:` 标记——hook 机械强制）

工作目录：`/Users/yoyi/workspace/product-line-syn-fnd-002`，分支 `syn-fnd-002-dev`（tip `a5e0235`）。
**不要切分支、不要 merge、不要 push。** 若 harness 引导你新建 M2 worktree，基座必须是 `syn-fnd-002-dev`（理由见 §2）。

## 1. M2 要造什么（一句话）

给全工作台建一套可复用的事务底座：**每条 command 原子提交 domain state + typed event + scrubbed audit + outbox**，带 receipt、lease、projector checkpoint；旧 store 逐域经 shadow/parity/rollback 迁移。**不做**大一统重构、不做全量 event sourcing、不接真实外部 provider/凭据。

## 2. 现状事实（都核实过，可直接采信）

- **M1 已关闭**（2026-08-03 用户拍板）：`decisions/2026-08-03-syn-m1-closure-acceptance-v1.md`。六个安全切片全部接线，FND-006 真机验收通过（`test-fixtures/fnd-006-acceptance/acceptance-record-2026-08-03.md`）。
- **M1 代码只在 `syn-fnd-002-dev`**（10 commits），FND-001 合同 commit `0b257db` 不在 integration main。用户已定基座策略：**M2 直接从 `syn-fnd-002-dev` 干，main 集成留到 M2 收口。**不要以 main 为基。
- **M1 合同与迁移输入**在 `docs/contracts/`：10 份版本化合同（identity-scope/command/event-audit-outbox/role-session/handoff/attention-decision/project-orchestration/memory-personal-model/connector-capability/object-ref-navigation）+ 入口/存储/迁移 inventory + `m2-shadow-write-parity-rollback-input-v1.json`。DAT-001 的输入齐全。
- **验证基线**（改动前先复跑确认）：`cargo check --lib` exit 0 / **599 warnings**；`cargo test --lib` **1304 passed / 2 failed / 45 ignored**。两个失败是既有的、与 M2 无关：`workbench_sqlite_production_apply::...sqlite_production_preflight...`（稳定失败，属 M2 §0.4 承接项）+ 进程夹具环境族（codex_local_runner/obsidian/manual_relay 在本沙箱轮流翻，环境性失败，勿当回归）。
- **工作目录**：`prototypes/productized-desktop-shell` 已 `npm ci`；tauri CLI 在 npm 全局（`tauri-cli 2.11.4`）。

## 3. 授权（用户 2026-08-03 拍板，`decisions/2026-08-03-syn-m2-blanket-authorization-v1.md`）

**M2 计划 §8 的独立授权项已整体预授权，不再逐包请示**：schema migration、live-manifest preflight、真实 store shadow/parity、逐域主读/主写切换、DB/JSON reconcile、真实 App 强退、adapter、旧写路关闭、工作台自有数据（真实 HOME 应用目录 + 仓内 store）任意改删。

**兜底纪律**：动真实 HOME 工作台 store 前先 `cp -R` 留副本到仓外 temp，位置写进 evidence。

**仍是硬线（碰了即停）**：git push / 对外发布 / 合并共享分支；写 `~/.codex` 或读凭据；codex 在工作台以外的真实项目目录真执行（固定测试项目 `/Users/yoyi/codex-workflow-mario-test` 是轻档可随便跑）；删工作台自有数据以外的不可恢复数据；接真实外部 provider。

## 4. M1 残留承接（M2 计划 §0.4，归你管）

| 项 | 落在 |
|---|---|
| grant 校验仅格式级（无 grant store，grant_id=dispatch_id） | DAT-002/003 建真 grant mint/load/verify；接上之前任何文档不得把 grant 当真防御 |
| FND-006 场景 3/4（伪造 report/grant 全链运行时） | DAT-008，需 fake runner 夹具 |
| FND-006 场景 5（Station 3b 写入运行时） | DAT-008；supervisor 机制 M2 未成则顺延 M3 并显式标注 |
| `sqlite_production_preflight...` 稳定失败 | DAT-002 期间定性修复 |
| 进程夹具族环境性失败 | DAT-002 期间并案排查 |
| code-map advisory（MAP_UPDATE_REQUIRED） | 首个 DAT 提交批顺手处理 |
| FND-001 合同未进 main | M2 收口随 main 集成一起 |

## 5. 切片序列与铁律（M2 计划 §4/§5 摘要）

```
DAT-001(文档:机制合同+逐域迁移清单) → DAT-002(additive schema+repository ports)
  → DAT-003(首个 vertical slice: policy→UoW→state→event→audit→receipt→snapshot)
  → DAT-004(transactional outbox+结果 command)
  └→ DAT-005(deterministic projector+shadow/parity) → DAT-006(legacy adapter+quarantine) → DAT-008(隔离 App 崩溃恢复验收)
                                                └→ DAT-007(逐域真实切换, 每域一包)
DAT-001B(只读 live-manifest preflight) → DAT-005/007 用真实 store 时的前置
```

铁律：

- **DAT-001 必须冻结一个具名 `reference_slice_id`**，DAT-003—006 全用同一片；不同样本各自通过不得拼成"切片通过"。
- 每次只迁一个 domain；`DAT-007(domain X)` 硬依赖 `DAT-001B(domain X)`。
- additive schema 先行；旧 store 观察窗内可读可导出；不得物理删旧数据；rollback 只切回已验证旧读主。
- policy-denied command 走独立 append-only scrubbed denial receipt，零 domain/event/outbox mutation。
- DB-primary blocked 时必须合同化 fail closed，不得让 JSON 静默领先。
- payload 只存 summary/ref/hash；raw transcript/prompt/tool output/secret 机械拒绝。
- 同一 Rust 承重文件单写者；`commands.rs`/`command_registry.rs`/`types.rs`/`c4_c6_workflow_governance_entrypoints.rs`/AppState 是公共承重面。
- DAT-003 后可收集 M3/M7 需求，但**不得激活 M3/M7 实现包**。

## 6. 每片验收口径（M2 计划 §7）

Contract lint 证明合同一致 ≠ 已实现；单测证明纯函数 fail closed ≠ 入口全接；temp SQLite/fixture 证明崩溃点/parity/重建 ≠ live store 已迁；non-test build 证明可构建 ≠ App 行为对；隔离 Tauri（scratch）证明崩溃/重启可见 ≠ 真实数据通过。**含 Rust production 路径的包必须同时跑 `cargo check --lib`（non-test build）+ `cargo test --lib`，全量前台落盘。**

## 7. 环境手册（都是踩过的坑，直接照用）

- **隔离跑 App**：`HOME=/tmp/xxx RUSTUP_HOME=/Users/yoyi/.rustup CARGO_HOME=/Users/yoyi/.cargo tauri dev`（rustup 按 `$HOME/.rustup` 找 toolchain，不指回必炸）。`SYN_ISOLATED_PROFILE` 不存在，别用。
- **控制台 invoke 桥**：`tauri dev --config <override.json>`，override 内容 `{"app":{"withGlobalTauri":true}}`；repo 配置保持 false。控制台里 `await window.__TAURI__.core.invoke('命令',{camelCase参数})`。
- **重启前清端口**：`lsof -ti :5173 | xargs kill`。
- **本沙箱教训**：批量验证一律前台 `> file 2>&1` 落盘再 grep（后台管道会报假 exit 0）；commit 后**另起一条命令**核 `git log` + `rev-parse HEAD^{tree}` 与提交前 `git write-tree` 逐位比对；`git add` 显式列文件，禁止 `git add -A`。
- **harness CURRENT.md 合同**（写错会 DEGRADED）：mode ∈ QUICK/PLAN/GUIDANCE/DEVELOPMENT；work-state ∈ READY/IN_PROGRESS/WAITING_EXTERNAL_CONDITION/BLOCKED/COMPLETE；必填 goal；STATUS 1-5 条、BLOCKERS ≤3、NEXT_ACTION 恰好 1 条、SAFETY ≤2。自检：`node scripts/harness-v2/project-context.js --target .` 报 OK。
- **立卡（可选）**：`scripts/harness-v2/task.js propose/start` 可用，但 start 要求 proposal 显式 `unknowns:[]` + 有效 `parent.id/digest` + declaration 含 `localCommitAllowed/pushAllowed`。嫌麻烦可像 M1 一样不立卡：用户直接指令 + CURRENT.md 记录授权即合法。

## 8. 每个任务包交付纪律

- 完成必附"怎么验的 + 真证据"（命令输出/文件/hash）；没验就写"已实现，未验证"。
- commit message 必带 `catch:` 标记；收口必回写 `docs/harness/CURRENT.md`；catch 记 `docs/harness-catch-log.md`。
- 报状态先核实物（git log + grep + 真机），不照搬 plan 的 ✅/⏳；聚合数字必须能由明细逐行加出。
- 注释不写"已校验/不信任"之类找不到对应代码的安全断言；新增必填字段逐个构造点按真实语义填，None 必须有注释。
- 共享 worktree 上验证 staged 树用 `git archive <tree> | tar -x -C <仓外>`，不用 `git worktree add`。

## 9. M2 退出门（计划 §9，全部满足才向用户申请关阶段）

同一 reference slice 完整通过 UoW/denial audit/snapshot/outbox/projector/shadow/parity/recovery；公共 ports/schema/receipt/禁止字段冻结；每个已触 domain 有 exact migration state；隔离 App 崩溃/重启证据；旧数据未物理删除、rollback/export 可执行；CURRENT 回写；用户显式激活 M3 前不得自动进入。

## 10. 开工第一步

**SYN-DAT-001（纯文档，不改生产 schema）**：消费 M1 合同，冻结 command receipt / UoW / event envelope / audit record / outbox item / current snapshot / projection checkpoint 的持久化与运行时状态机、FK/unique/index、receipt 丢失、lease、quarantine、重建、rollback；冻结具名 `reference_slice_id`；冻结安全 payload storage / payload-ref 完整性 / retention / scrub 规则；用 conversation、workflow、memory、knowledge 四类现状路径走纸面追踪（每类答出 owner、事务边界、外部 effect、失败残留、恢复动作）。交付进 `docs/contracts/` 与本阶段迁移清单。参考输入：`docs/contracts/m2-shadow-write-parity-rollback-input-v1.json`、`storage-opening-inventory-v1.json`、`legacy-migration-inventory-v1.json`、`entrypoint-inventory-v1.json`。

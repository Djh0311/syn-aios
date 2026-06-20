# 结果 · 全项目「事实 ⇄ 文档」核实与正本收口（v1 · 2026-06-21）

> 交回**主导线**。本文是 `handoffs/2026-06-21-full-project-fact-reconciliation-task-package-v1.md` 的执行结果。
> 一句话结论：**散在文档里的状态确实和代码打架，但打架的方向被任务包说反了一半——核心底座（R3/记忆层）不是「没做」，也不是「做完上线」，而是「建好+预演+用户拍板『先不翻闸』」。CURRENT ④ 把这种状态压成「deferred」一个词，才让主导线误报。修法是把这一格说精确，不是翻成「done」。**
> 纪律：本文每条状态都附「怎么核的 + 证据（`file:line` + commit + 真机/测试观察）」。**没在本仓 git 里的，明确标 UNVERIFIABLE，不替它圆。**

---

## 0. 整体怎么核的（方法 + 我顺手纠的错）

**核法**：HEAD `866a7d5`（main）。三路交叉——(1) `git log`/`git rev-list` 查 provenance；(2) `grep` + 亲读代码定 `file:line`；(3) 真机观察：跑 `cargo test --lib`、读线上 App Support 状态目录、查 Tauri 命令注册表。读代码、不改代码、不 commit、不解封、不真跑 codex、不碰 `~/.codex`。

**这次连任务包和我自己派的子核查都纠了错（事实优先于任何上文，包括本任务的描述）：**

1. **任务包 §2-C 说「`commands.rs:612` 默认路径走 `temp_dir`」——错。** `commands.rs:612` 在 `#[cfg(test)]` 的 `temp_codex_home` 测试辅助里，不是生产路径。生产工作台状态路径是 `default_workflow_state_path()`（`lib.rs:1284-1288`）= `~/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`，一个**真实持久的 JSON 文件**，既不是 temp、也不是 sqlite。
2. **任务包暗示「R3 走完+拍板 ⇒ CURRENT 说 deferred 是错的」——只对一半。** git 确实显示 R3 Level A/B 跑完、`ac7813b`「close stage r — user-ratified」。但**拍板拍的是「ready_but_not_executed / 先不翻闸」这个决定本身**，不是「生产已切 sqlite」。所以「deferred」对**生产结果**不算假，错在**精度**（见 §2-C、§3）。
3. **本仓 git 历史只到 `2026-06-11`（`ed01c6f` root treatment baseline，共 420 commit）。Stage E/F/G/H/I/J/K 在本仓 git 里一条 commit 都没有。** 任何给 E-K 配 commit hash 的「✅+证据」都**无法在本仓核实**——这是 §三「核实物」纪律的一个结构性洞（见 §6）。
4. 子核查线把 product-command Phase A/B 说成「UI 不可达/orphaned」——**偏强**。前端确有视图引用（`AgentView/ProjectsView/ProjectWorkspaceShell/...`），但 Phase A 明确「不真执行」、Phase B 真执行被授权门挡在真实项目外。净效果一样：**自动编排不是线上默认**，但措辞我按代码收紧了。

**真机/测试观察**：`cargo test --lib` → `555 passed; 0 failed; 24 ignored`（10s，本机现跑）。`cargo build` 成功但 **591 条 dead-code warning + 9 条 unused-import**（绝大多数来自 R3 sqlite 孤岛，见 §5）。线上 `formal-memories.v1.json` 实存 **3 条真实正式记忆**（Jun 17 写）。

---

## 1. 状态矩阵（能力/阶段 × 五类 × 证据）

五类：① 真·上线在用 ② 只预演/探针跑过 ③ 故意锁着 ④ 没建/空壳 ⑤ 文档声称✅但代码对不上。

| 能力 / 阶段 | 类 | 证据（`file:line` + commit + 观察） | 一句话 |
|---|---|---|---|
| **甲·手动中转 relay** | **①** | `manual_relay.rs`；命令注册 `command_registry.rs:13-16`；里程碑 `9b7360a`；沙箱闸 `--sandbox workspace-write`+`--add-dir`（`manual_relay.rs:2135/2142`、`codex_local_runner.rs:1321/1324/1892`）；拒审批绕过 `manual_relay.rs:2150-2152/2192-2194`；真发钥匙 `MANUAL_RELAY_REAL_CODEX_CONFIRM`（`:17-18,1040`） | **唯一真能指挥 codex 的路径。** CURRENT ① 对它的描述属实。真发受在场 env 钥匙门控。 |
| **会话列表数据源（codex sqlite 只读）** | **①** | `codex_db.rs:63-68`（`~/.codex/state_5.sqlite`）、`:127` `SQLITE_OPEN_READ_ONLY`；transcript 命令注册 `command_registry.rs:17-19` | 仍成立、在用。只读，未写 `.codex`。 |
| **transcript 读取（静态索引→sqlite/rollout 回退）** | **①** | `lib.rs:168` `..._from_sqlite_row`、`:202` rollout 回退 | 记忆 `codex-workbench-session-data-sources` 里「2026-06-02 修」的回退已在码、已接。 |
| **生产持久化（工作台自身状态）** | **①（JSON）** | `lib.rs:1284-1288` → `…/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`；真机：该文件存在(2MB，**末改 May 31**) | 线上用 **JSON 文件**，**不是 temp、不是 sqlite**。这是 §2-C 第一题的答案。 |
| **R3 真库切换（JSON→sqlite 迁移机制）** | **②（fixture/Level-B 预演+拍板）+ 生产翻闸 deferred** | 机制：`workbench_sqlite_*`（13 文件，`lib.rs:42-53`）；闸内证据 `evidence/2026-06-11-...r3-a13-...-gap-matrix-v1.md`（Level A complete / 生产项全 `deferred`）；Level B 收口 `handoffs/2026-06-16-...r3-b4b-...result.md`（`ready_but_not_executed`）；commit `370acd3→57da3d0`、`ac7813b`；**真机：0 个 tauri 命令、0 个生产调用方**（`command_registry.rs` 无 sqlite cutover 命令） | **建好+在 fixture/temp 全流程演过+用户拍板「先不翻闸」。生产没切，仍走 JSON。** 既非「没做」也非「上线」。 |
| **统一记忆层（存储+治理）** | **①（已建已接已存真数据）+ over-built 已拍板裁剪待办** | 存储=JSON sidecar：`formal_memory_store.rs:14`、`memory_candidate_store.rs:15`、`memory_capture_bus.rs:11`、`observations.v1.json`；命令注册 `command_registry.rs:59-82`；决策 `decisions/2026-06-16-memory-layer-form-...-trim-v1.md`（用户拍板：终局成立但当前过度、不返工、观察+裁剪、DB 切 deferred、多 agent 部分挂起）；真机：线上 5 个 memory sidecar 都在(Jun 17) | 17 表/七层全建好、命令接通、**线上实存 3 条真实记忆**。不是「deferred」。被拍板冻结的只有「切 DB」+「多 agent 专属」。 |
| **真攒记忆（capture→candidate→formal 日循环）** | **①（已起，量小）** | `memory_capture_bus.rs`（`capture_memory_event` 注册 `command_registry.rs:60`）、`memory_daily_loop.rs`（commit `512c047` L5）；真机：`formal-memories.v1.json` **3 条**（grep `memory_id`=3，Jun 17）；首条里程碑 `ece4d77` | 真在攒，只是少。决策时说「0 条」，现 3 条。 |
| **工作流执行引擎·多节点编排（run_workflow_machine）** | **③ 故意锁着（impl 完整但生产 dead）** | 前端拦 `App.tsx:547`；后端 `commands.rs:1789-1798` → `legacy_product_command_blocked`；完整实现 `workflow_execution_entrypoints.rs:1489` `run_workflow_machine_at`（`max_rounds` 多轮、四角色 `round_steps`、`execute_workflow_machine_step`→runner 真 spawn、失败/acceptance 处理）；测试到底 `lib.rs:4355 workflow_machine_runs_four_role_loop_to_acceptance`（含在 555 pass 内） | **任务包说的「闸后面的多节点编排实现」属实且完整**——director→dev→…→acceptance 真环、真 spawn，**但只被测试触达，生产里是死的**。⚠️ **正在变动**：主导线按 `decisions/2026-06-21-next-step-unseal-workflow-engine-for-test-project-v1.md` 并行解封到固定测试项目。本线只读未碰。 |
| **节点派发 execute / 读回 readback** | **③ 锁着（impl present）** | `commands.rs:1651-1659`/`1674-1683` → blocked；真实现 `workflow_run_dispatch_entrypoints.rs:841/900` 仍在，仅测试调用 | 同上「被 block 但代码仍在」。`prepare_workflow_node_dispatch`（`commands.rs:1631`）**未锁**（只准备不执行）。 |
| **MCP canvas 执行（canvas_start_run/tick）** | **③ sealed** | `mcp/commands.rs:49/76` → `mcp_canvas_real_execution_blocked`；`mcp/orchestrator.rs` start/tick「Sealed legacy experiment」；前端 `src/lib/tauri.ts` 标 `@deprecated` | legacy 封存。 |
| **product-command 真执行 Phase A / Phase B** | **Phase A ④(no-op) / Phase B ③(授权门挡真项目)** | 前端 `tauri.ts:334-352` 接、视图引用；Phase A 决策枚举 `agentSession.ts:416 "phase_a_runner_path_recorded_no_real_execution"`；Phase B `:444 "phase_b_real_resume_executed"`；K3-B 真发被拒 `commands.rs:879-892` | UI 接了，但 **Phase A 明说不真执行**、Phase B 真执行对真实项目封。所以「自动跑」不是线上默认。 |
| **会话续跑 real resume（受控）Phase A/B** | **②（受控探针）** | 注册 `command_registry.rs:51-56`；真 runner `session_continuation_store.rs:581`；stub 并存 `run_controlled_session_continuation_stub`（`commands.rs:1014`） | 受控真 resume 探针，非线上默认。 |
| **Stage E/F/G（适配描述/会话边界/可用性/画布读模/运行日志/诊断）·读模与 UI 部分** | **①** | `session_continuation_store.rs`、`page_read_model.rs`、`workflow_read_model_entrypoints.rs`、`runtime_log_store.rs`；命令接通且视图在用；**commit UNVERIFIABLE（早于本仓 git）** | 读模/边界/预览/日志这些「只读面」是活的。G3 真机截图自认未完（`docs/plans/2026-06-06-stage-e-f-g-...:503` 10/13）。 |
| **Stage H/I（CodexLocalRunner/真 resume/适配中性 WorkerAdapter）** | **② / 抽象层** | `real_execution_command.rs`、`h5_project_dispatch_bridge.rs`、`worker_protocol.rs`（编译在 `lib.rs`）；真执行=Phase B 探针，H5 桥**只用于 preview**（`commands.rs:734`）；**commit UNVERIFIABLE** | 码在，作探针在 mario test 上跑过；非线上能力。H3-B 真新会话计划自认失败未重试。 |
| **Stage J/K（控制平面/工作流自动化/记忆 UX/操作控制/日用工作台）** | **混合：①(记忆+UI) / ②(探针) / ③(真派发锁)** | 记忆捕获 `capture_memory_event`（`commands.rs:1112`）①；自动化 UI 只调 `…PhaseA`(no-op) ；真 `execute_workflow_node_dispatch` 锁 `commands.rs:1651`；**commit UNVERIFIABLE** | 「日用工作台/真派发闭环」的**读模/记忆/UI 半边是活的，真执行半边是 no-op 或锁的**。 |
| **Stage K5 操作控制（retry/stop/restart/resume）** | **④ decision-only** | `operation_control.rs`；commit `224c9cb`(L3) 自述「**decision-only** control face」；计划自认「仍只是需确认…不是真实操作能力」 | 是「确认面」，不是真停/真重跑 codex。 |
| **Stage L1（K3-B1 blocked recovery 产品面）** | **①** | commit `69f89e8`；`k3_b1_recovery.rs`、读模 `lib.rs:1698`、UI `PermissionDialog.tsx`/`ProjectWorkflowSidePanel.tsx` | **git 可核+线上活。** 是「解释被挡状态」的面，不解封真执行。 |
| **Stage L5（记忆候选日 inbox）** | **①** | commit `512c047`；`memory_daily_loop.rs`、`DailyMemoryCandidateInbox.tsx`（`RunningWorkflowsView.tsx:260` 渲染） | git 可核+线上活（Jun-20 画布重写后仍在）。 |
| **Stage L「RU」dogfood（ru_dogfood）** | **④ dead/blocked** | commit `ece4d77`；`ru_dogfood.rs`(497 行) 仅 `memory_context_entrypoints.rs:5` 有 `mod` 声明，**`ru_dogfood::` 零调用点**；commit 标题自述「RU1 GUI 被 .codex 封印阻断」 | 提交了但是死码。 |
| **Stage L2/L4/L6** | **④/UNVERIFIED** | 无对应 commit、无独立模块 | 无证据落地；计划也没声称做完。 |
| **Stage R / R2 / R4 / R-U（root treatment 系列）** | **①（已收口，git 可核）** | R 收口 `ac7813b`；R2 lib 拆分/治理边界（`R2-T11..T14` 系列 commit）；R4 前端拆分（`b9491c8` 等）；R-U dedup `e6325e8/1ba8f01/bc436dd/c4335e1/16e96bd` | 这些 6-11 后的阶段 git 可核、已落。R-U dedup **漏掉 sqlite 孤岛**（见 §5）。 |
| **前端 B 线（拆瘦/渐进披露/会话流/画布重画/智能体页）** | **①** | `App.tsx`=695 行、`RunningWorkflowsView.tsx`（画布 P1-P4 `866a7d5`）、`@xyflow/react ^12.10.2`、`ProjectWorkflowReactFlowCanvas`(`src/lib/projectCanvas.ts`) | CURRENT ① 对前端的行数/收口描述与码一致。**唯一没验**：真机对图（需起 Tauri，未做）。 |
| **乙·自动连环 / 多项目接力** | **④ 没建（终局）** | 无实现；`autonomy gate` 仅在计划里 | 终局，确实没开。CURRENT ④ 这条对。 |

---

## 2. §2 四块悬题——钉死

**A. 工作流执行引擎**：当前**前后端双重硬 block**（`App.tsx:547`/`commands.rs:1789`）。闸后的多节点编排**实现完整且测试到 acceptance**（`workflow_execution_entrypoints.rs:1489`），但生产里是死码（仅测试触达）。**正在变动**（主导线并行解封到固定测试项目，`decisions/2026-06-21-...`）。本线只读、未碰、未解封、未真跑。

**B. 甲·手动中转**：成立。GUI 真发里程碑 `9b7360a`；沙箱+拒绕过闸齐全；env 钥匙 `MANUAL_RELAY_REAL_CODEX_CONFIRM` 与 GUI 产品路径是两条，没混。**这是唯一真执行路径**。

**C. 线上数据存哪 + R3 翻没翻闸**：存 **JSON**（`workflow-state.v0.json`，App Support）。**R3 没翻到 sqlite。** Level A=fixture 全演过、Level B=受控决策跑到 `ready_but_not_executed`、用户拍板关闭**验证阶段**（不是关闭「翻闸」动作）。sqlite 机制 0 命令、0 生产调用方。→ **「deferred」对生产结果不假，但把「建好+演过+拍板暂缓」压成一个词，是误报之源。**

**D. 记忆层真实存储 + 捕获到哪步**：存储=**JSON sidecar**（`formal-memories.v1.json` 等，App Support）。命令全接通。**真攒**：线上实存 **3 条真实正式记忆**，日 inbox 已接（`512c047`）。被拍板冻结的只有「切 DB」+「多 agent 专属门」。CURRENT ④ 说「deferred」同样**误把已建已用的压成没做**。

---

## 3. `CURRENT.md` 修订草案（**不 commit**；交主导线核后做）

只动 ④ 块 + ① 加一句、② 注一句。**正本只一只手动，故此处给草案，未改文件本身。**

**④ 块——改前：**
```
- **真跑 codex 进真实项目**（非 temp）：用户在场明确授权那一下，不可省。
- **乙·自动连环 / 多项目接力**：终局，没开（风险到这才真大）。
- **底座**：R3 真库切换、统一记忆层、真攒记忆 —— deferred，各需另窗另批。
```

**④ 块——改后（建议）：**
```
> ④ 区分三种「不在线上默认」：未建 / 故意锁(可解条件) / 已建已演已拍板但暂不翻闸。别压成一个词。

- **故意锁（impl 完整、可解条件明确）**
  - 工作流多节点编排引擎：前后端双锁（`App.tsx:547`、`commands.rs:1789`→legacy blocked）；
    完整四角色真环实现在 `workflow_execution_entrypoints.rs:1489`，测试到 acceptance。
    ⚠️ 主导线正按 `decisions/2026-06-21-next-step-unseal-workflow-engine...` 解封到固定测试项目。
  - product-command 真执行 Phase B / 受控 real resume：对真实项目封，探针跑过。
  - 真跑 codex 进真实项目（非 temp）：用户在场明确授权那一下，不可省。
- **已建+已演+用户拍板「先不翻闸」（不是没做）**
  - R3 真库切换（JSON→sqlite）：迁移机制 13 模块全建、fixture+Level-B 演完、`ac7813b` 拍板，
    结论 `ready_but_not_executed`。**线上仍走 JSON（`workflow-state.v0.json`）**；翻闸是被拍板暂缓的一步。
  - 统一记忆层 + 真攒记忆：存储/命令全接通，线上实存 3 条真实记忆；
    `decisions/2026-06-16-memory-layer-...-trim-v1` 拍板「过度、不返工、观察后裁剪」。
    **冻结的只有「切 DB」+「多 agent 专属门」**，不是整块 deferred。
- **没建（终局）**
  - 乙·自动连环 / 多项目接力：风险到这才真大，没开。
```

**① 块——建议加一句**（事实校准）：
```
- 后端测试基线：`cargo test --lib` = 555 passed / 0 failed / 24 ignored（2026-06-21 本机）。
```

---

## 4. 打架文档清单（建议处置）

| 文档 | 判定 | 据 |
|---|---|---|
| `CURRENT.md` ④ | **改**（见 §3） | 唯一被规则信任的正本却把「已建已演已拍板」压成「deferred」，是本次误报根因。 |
| `STAGE_PLAN.md`（44KB） | **归档** | 自带「状态过期」横幅但体量+✅ 最易被照搬；Stage A-R 世界已被 master-roadmap+CURRENT 取代。最大单点过期面。 |
| `docs/plans/2026-06-10-stage-l-post-k-deferred-closure-...` | **标过期/改** | 声称「L1-L6 deferred during root treatment」，但 **L1/L3/L5 已 commit 落地+线上活**（`69f89e8`/`224c9cb`/`512c047`）。⑤ 类，文档与 git 直接矛盾。 |
| `docs/plans/stage-j-...`、`stage-k-...(v1/v2/v3,daily-use)` | **降附件/归档** | 「真派发闭环/日用工作台」实为：读模+记忆+UI 活、真执行 no-op/锁。已被 master-roadmap §6 降为附件。 |
| `docs/plans/2026-06-18-master-roadmap-phased-v1.md` | **保留**（自认过期） | §1 状态是 6-18 快照、line 125 自标过期、指向 CURRENT；结构正本仍有效。§2 附件指针有 `tasks/→handoffs/` 轻微 rot。 |
| `README.md` | **标过期** | 「当前定位=Stage R 治理主线」落后约 5 周（漏 relay+画布转向）；顶有免责。 |
| `DEV_LINES.md`/`PROTOTYPE_WORK_LINES.md`/`principles.md` | **归档/改** | 仨都把「当前方向权威」路由到已非正本的 `tasks/README.md`；Jun-1 期框架。 |
| `docs/plans/README.md` | **改/删** | 模板样板，引用不存在的 `requirements-matrix.md`/`task-queue.md`/`decisions.md`，点名一个过期 active plan。 |
| `docs/plans/*` Stage E-L/K-calib/PCR/middleware 簇(~30, 06-01→06-13) | **批量归档** | 早于 6-17 relay 转向+6-18 master-roadmap，已被降为「附件」。 |
| `handoffs/`（398 文件）/`tasks/`(280)/`evidence/`(419) | **批量归档 6-15 前** | append-only 审计流水，非维护正本；体量本身妨碍「看最近几份」。 |
| **保留勿动** | — | R3 计划契约（fixture-only 范围与码一致，是设计源）；`decisions/**`（拍过的板，未被码推翻）。 |

---

## 5. 代码事实问题清单（**本任务不清理**，供后续）

`cargo build` 成功但 **591 dead-code warning + 9 unused-import**。重点：

1. **`workbench_sqlite_*` 孤岛（13 文件，~480 条 dead warning）** — 0 个 `#[tauri::command]`、0 个生产调用方，仅自身测试触达。`write_json_file` 复制 6 份（5 份字节相同）、`write_report` 5 份等。**R-U dedup 漏了这片。删它能一把清掉后端 ~80% dead 噪声。** 最大单点。
2. **`workflow_execution_entrypoints.rs` `run_workflow_machine_at` 整条链 dead**（~21 函数）——因命令被硬 block。完整实现，但生产不可达。
3. **`real_execution_command.rs` K2「mario test」脚手架 dead**（~42 项）——硬编码 `/Users/yoyi/Documents/mario test` + 写死 session id/prompt hash（`:34-66`）。
4. **`ru_dogfood.rs`（497 行）dead** —— 零调用点（`ece4d77`，被 `.codex` 封印阻断）。
5. **blocked-but-present**：5 个 tauri 命令 + 2 个 mcp canvas + 1 个 cli，闸后 impl 仍在（dead）。注册表 `command_registry.rs:97-113`。
6. **命名≠行为**：`real_execution_command`（线上是 Phase A no-op）、`workflow_execution_entrypoints`（半数 dead）、`workbench_sqlite_production_apply` 家族（fixture-only）、`mcp/orchestrator`（sealed no-op）。
7. `worker_protocol.rs:2418` `_readback_plan`、`workbench_sqlite_exporter.rs:346` `manifest_counts`、`mcp/tools.rs:488` `_silence_unused` —— `#[allow(dead_code)]` 盖着的真死码。
8. 无 `todo!()`/`unimplemented!()`/`FIXME`（前后端皆 0）；`unreachable!()` 几处是合法枚举兜底，非 bug。

---

## 6. 机制反思（给 `mistake-ledger` / §三·§五 补丁）

**正本为什么跟代码漂这么远——两个真洞，都这次真发生了：**

1. **「除 CURRENT 外按过期对待」假设 CURRENT 自己对，但 CURRENT ④ 只有一个桶装三种状态。** ④ 用同一套词（「deferred / 各需另窗另批 / 锁着」）同时盖：(a) 真没建（乙）、(b) 故意锁可解（引擎/Phase B）、(c) 已建+已演+拍板暂不翻闸（R3 翻闸、记忆切 DB）。把 (c) 说成 (a) 的话术，主导线一照搬就报「没做」。
   **补丁**：凡是**靠『决策』收口（而非『完成』收口）的能力，CURRENT 必须同时记『建/演/拍了什么』和『故意没执行什么』，禁止压成「deferred」一个词。** ④ 分三栏：未建 / 故意锁(条件) / 已建已演已拍板但暂不翻闸。

2. **`git log` 核实物的前提——历史能够到那段工作——这次不成立。** 本仓 git 从 `2026-06-11` 起，Stage E-K 一条 commit 没有。任何「E-K ✅ + commit hash」都不可在本仓核，却一直被当已核。
   **补丁**：`§三` 的「核实物（git log + grep + 真机）」加一条——**引用 commit hash 当证据前，先确认该 hash 在本仓 `git cat-file -e` 得到；够不到的（如 baseline 前）明确标 UNVERIFIABLE，不得当已核传递。**

**附带**：本次还印证「不信子核查自报」——我派的子线把 Phase B 说成 UI 不可达、把 612 行误读这类都按码纠了。**核实物纪律对子 agent 同等适用。**

---

## 7. 边界声明（我没做什么）

只读代码；**未改任何代码、未 commit**；未碰/未解封工作流引擎闸；未真跑 codex；未写/未读 `~/.codex` 凭据。`cargo test`/`cargo build` 为只读式编译验证（fixture/temp，无副作用）。读了线上 App Support 工作台状态目录（只数条数，未导出记忆正文，避免外泄）。本文是**草案/报告**，落在 `handoffs/`，等主导线核实物 + commit（带 CURRENT ④ 回写）。

> 交回主导线：据本矩阵**核实物 + commit（带 CURRENT 回写）**，并据「故意锁 / 已建暂不翻闸 / 没建」三分重排下一步。**别据本文把 R3/记忆翻成「done」——那是反方向的虚高。**

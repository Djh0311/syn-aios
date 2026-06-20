# 任务包 · 全项目「事实 ⇄ 文档」全面整理与正本收口（v1 · 2026-06-21）

> 交给**另一个独立对话**执行。本包**自带全部背景**，不依赖任何上文。
> 一句话目标：**把这个项目散在十几份文档里、且与代码/ git 互相打架的状态，核成一份「代码为准」的单一真相**，并改对正本、标过期。

---

## 0. 给接手对话的话（必读）

- 你接手的是一个**只读核实 + 正本收口**的任务。你没有上文，所有需要的信息都在本包里。
- **项目**：`codex-governance-workbench` —— 一个治理 Codex 的桌面工作台，Tauri（React+TS 前端 / Rust 后端），代码在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/`（前端 `src/`、后端 `src-tauri/src/`）。仓库根 `/Users/yoyi/workspace/product-line/`。
- **规则正本**：先读根目录 `AGENTS.md`（精简版 v2）+ `CLAUDE.md`。再读 `CURRENT.md`。
- **角色**：两条线——**主导线**（统筹 / 核实物 / 对接用户）、**执行线**（Codex 干活）。你干的是**主导线的核实物那部分**：只读代码、核对文档、产出真相。
- **最高纪律（AGENTS.md §三，本任务的全部理由）**：报任何「做没做 / 到哪步」**必须先核实物**（`git log` + 代码 grep + 真机），**严禁照搬任何 plan / roadmap / STAGE_PLAN / 甚至 CURRENT 的 ✅/⏳**。本任务存在的原因就是：**这些文档的状态已被证明不可信，包括 CURRENT 本身**。

---

## 1. 为什么有这个任务（问题陈述）

1. **计划散成「一边一套」**：根目录就有 `AUTHORITY.md`、`CURRENT.md`、`STAGE_PLAN.md`、`DEV_LINES.md`、`PROTOTYPE_WORK_LINES.md`、`RESULT_REVIEW.md`、`backlog.md`、`README.md`、`principles.md`，外加 `docs/plans/*`、`decisions/*`、`handoffs/*`。它们互相、且与代码打架。
2. **连唯一指定正本 `CURRENT.md` 自己都错了**：它第④块把「**R3 真库切换、统一记忆层**」列为 `deferred / 锁着`，但 `git log` 显示这俩是**走完且用户拍板关掉**的阶段（见 §2 证据）。主导线照搬了 CURRENT 的错误状态，向用户误报，才触发本任务。
3. **用户判断：连代码事实都是乱的**——存在「被 block 但代码仍在的引擎」「dead 变量」「重复 / 孤儿实现」等。需要一次彻底的、以**代码为准**的整理。

---

## 2. 已确认的事实（**别重做，但要复核 + 补全**）

下面是触发本任务的对话里已核到的点，给你做起点。**你要做的不是信它，是验它 + 把它扩成全项目**。

**A. 工作流执行引擎 = 当前被硬 block（已亲验）**
- `run_workflow_machine` → 返回 `legacy_product_command_blocked`，`src-tauri/src/commands.rs:1790-1798`
- `execute_workflow_node_dispatch` → 同上，`commands.rs:1651-1659`
- `read_workflow_node_dispatch_result` → 同上，`commands.rs:1674-1683`
- `canvas_start_run` / `canvas_tick_run`（MCP）→ legacy sealed，`src-tauri/src/mcp/commands.rs:45-52, 72-77`
- block 原因：不满足「H5 统一产品命令边界」（permission/continuation/runtime log/audit/readback），`src-tauri/src/real_execution_command.rs:144-177`。替代命令名：`preview_h5_project_workflow_dispatch` + `controlled_session_continuation`（**是否接通、到哪步，待你核**）。
- 闸**后面**的多节点编排实现据称在 `src-tauri/src/workflow_execution_entrypoints.rs:1489+`（director→dev→验证→审查 循环、真 spawn codex）——**此处只经二手扫描，未亲读，务必亲核其完整度**。
- ⚠️ **协调注意**：**工作流引擎的「解封」由主导线在另一个对话并行进行。本任务对这块只读、如实报状态、不改代码、不解封、不真跑。** 把它当「正在变动」标注即可。

**B. 唯一真能执行的是「甲·手动中转」（单条手动转发，已上线）**
- `run_manual_codex_relay_gui_direct` 等，`commands.rs:60-85`；后端 `manual_relay.rs`。GUI 真发 codex 成功的里程碑提交 `9b7360a`。
- 另有一条 env-gated 模式要环境变量钥匙 `MANUAL_RELAY_REAL_CODEX_CONFIRM`，`manual_relay.rs:1039-1044`（与 GUI 产品路径是两条，别混）。

**C. R3 真库切换 ——「锁着」是错的，它是走完+用户拍板的大阶段**
- git 轨迹（节选）：`370acd3 enable r3 b1 confirmed production apply` → `97ec465 r3 b1 apply hash mismatch stop` → `789949c r3 b1 retry` → `48513d5/9edc2a7 r3 b2 read-cut（"source/db unchanged"）` → `1a2db17/11beb3b r3 b3 observation` → `26744ad/6824402 r3 b4 stop-write（"ready_but_not_executed; no stop-write"）` → `57da3d0 close r3 level b (b5 final matrix; user-ratified)` → `ac7813b close stage r — user-ratified`。
- ❓**关键悬而未决**：阶段走完+拍板是真的，但「**线上工作台数据现在到底存哪**」是糊的。`commands.rs:612` 显示默认路径走 `std::env::temp_dir()`、`:620` `load_workflow_state_snapshot` 读的是**文件路径**（疑似还在 temp 文件），而 b4 写的是 `ready_but_not_executed`。**到底真翻闸到 sqlite 了没？这是必须钉死的第一题。**
- 相关 decision：`decisions/2026-06-10-stage-l-root-treatment-freeze-relationship-v1.md`、`decisions/2026-06-13-root-treatment-r2-late-stage-closure-track-v1.md`。

**D. 统一记忆层 / 真攒记忆 —— 也动过、也拍过板**
- `decisions/2026-06-16-memory-layer-form-acknowledgement-and-use-driven-trim-v1.md`（结论：**over-built-for-stage，按使用裁剪**）。
- git：`bb4d1dc ratify memory-layer form decision`、`4d5c958 consolidate memory-layer research`、`512c047 stage l l5 memory capture-to-candidate daily loop`、`f69922e retire agentmemory`。
- ❓**必须钉死**：真实记忆存储是什么、捕获循环到哪步、CURRENT④说的「deferred」错在哪。

---

## 3. 任务本体：产出一份「代码核实」的单一真相

对**每一个**主要能力 / 阶段，判定其**真实状态**，归入下列**五类之一**，每条**必附证据**（代码 `file:line` + commit hash +（涉及运行时就）一次真机/测试观察）：

| 类 | 含义 |
|---|---|
| ① 真·上线在用 | 代码接通 + 能跑 + 是线上默认 |
| ② 只预演/探针跑过 | 受控跑过一次（fixture/probe），但**不是**线上默认 |
| ③ 故意锁着 | 有意的闸（写明解锁条件 / 为什么锁） |
| ④ 没建/空壳 | no-op / placeholder / TODO / 搜不到实现 |
| ⑤ 文档声称✅但代码对不上 | **重点标出**——正本漂移的根源 |

**要覆盖的清单（至少）：**
- 各 Stage：E / F / G / H / I / J / K / L / R、R2、R3（A 线 a6-a13、B 线 b0-b5）的真实收口状态 vs 各文档声称。
- §2 的 A/B/C/D 四块，把❓那几个悬题钉死。
- 工作流编排执行（只报状态，**不改**，见 §2-A 协调注意）。
- 真实数据库 / 持久化：线上到底用 temp 文件还是 sqlite。
- 记忆层：真实存储 + 捕获状态。
- 会话数据源：会话列表读 codex sqlite（只读）这条是否仍成立、在用。
- **代码层的「乱」**：dead 变量、重复/孤儿实现、blocked-but-present 残留、命名与实际不符——单列一张问题清单。

**必须核的「文档源」（逐一对照代码）：**
`CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`DEV_LINES.md`、`PROTOTYPE_WORK_LINES.md`、`RESULT_REVIEW.md`、`backlog.md`、`docs/plans/*`（尤其 `2026-06-18-master-roadmap-phased-v1.md`）、`decisions/*`、`handoffs/*`。对每份给一句「与代码一致 / 哪条过期」。

---

## 4. 产出物（交回主导线）

1. **状态矩阵**：`能力/阶段 × 五类之一 × 证据(file:line + commit + 观察)`。这是新正本的依据。
2. **`CURRENT.md` 修订草案**（尤其④块）：改到与代码一致；**以 diff 或清楚标注的草案给出，先不 commit**（commit 由主导线核后做，保证正本只有一只手在动）。
3. **打架文档清单**：哪些 plan/roadmap/authority 与事实冲突 → 建议「标过期 / 归档 / 删」。
4. **代码事实问题清单**：dead/重复/孤儿/blocked 残留，供后续清理（**本任务不清理代码**）。
5. **一条机制反思**（给 `mistake-ledger` 或 §三 补丁）：为什么正本会跟代码漂这么远、怎么堵——因为「§三 说『除 CURRENT 外都按过期对待』，但 CURRENT 自己没维护对」这个洞这次真的发生了。

---

## 5. 边界（必守）

- **只读代码**；**可改文档**（产出草案/报告），但 **不 commit**（执行子线不 commit；交回主导线核 + commit）。
- **不碰安全闸 / 不解封任何 blocked 命令 / 不真跑 codex / 不写 `~/.codex` / 不读其凭据**。
- **工作流引擎那块代码不要动**——主导线在并行解封，你只读、只报状态。
- 范围超出预期 → 停下说一声，别擅自扩。
- 报状态一律附「怎么核的 + 证据」，不照搬文档 ✅。

---

## 6. 交接回主导线

完成后交回：**状态矩阵 + CURRENT 修订草案(diff) + 打架文档清单 + 代码问题清单 + 机制反思**，外加一句「整体怎么核的」。主导线据此**核实物 + commit（带 CURRENT 回写）**，并据矩阵刷新真正的下一步排布。

> 落点建议：结果写到 `handoffs/2026-06-21-full-project-fact-reconciliation-result-v1.md`。

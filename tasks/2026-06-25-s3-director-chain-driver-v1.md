# 实现任务包：S3 主管→worker 薄链驱动（LM 多步自包含计划 → worker 链真跑出结果）· 主导线 → 执行线 v1

日期：2026-06-25　性质：**高危#4-轻档**（固定测试项目内的自动连环——多 worker 按依赖顺序跑；前提 4 护栏在 + 每节点过 S1 闸 + path-lock；真链跑用户在场）。

## 0. 接手须知（冷启即读，本包自包含）

- 你是**执行线**（新开的、干净上下文）。子线不 `git add`/`commit`。
- **前置已 done（committed `75ab2c0`）**：主管 agent（`director_agent.rs` 的 `CliDirectorAgent`）已能就着已授权方案产**自包含** `planned_tasks`（带 `depends_on`、scope 从授权派生、worker 结构化返回）——真跑验过 LM 现产自包含、残留引用词 0。**本包只差最后一步：把这串任务按依赖顺序真跑成一条 worker 链。**
- **为什么要新薄驱动（核实物结论，别再走弯路）**：`workflow_chain_controller` 跑的是**工作流节点**（按 `edges` 拓扑序），而 `prepare_authorized_auto_dispatch` 建 **per-role 节点**（`c4_node_id(wf,role)`，同 role 多任务被 `ensure_node` 合并成 1 节点）——所以"核对→创建→回读"3 个 codex-dev 任务只产 **1 个节点**，chain controller 装不下 director 的 planned_tasks。**「喂现成 chain controller」不成立。**
- 先读：① `director_agent.rs`（`CliDirectorAgent.plan()` → planned_tasks）② `prepare_authorized_auto_dispatch_for_index_at`（planned_tasks → work_items/PreparedAutoDispatchReadModel）③ **§6/S1-③ 真跑路** `s2_3_real_run_role_loop_through_gate`（lib.rs：`execute_project_workflow_node_at` 过 S1 闸真跑 worker）④ `workflow_chain_controller.rs`（4 护栏/拓扑序参考；私有 `workflow_chain_topological_order` 可复用）⑤ `tasks/2026-06-25-s3-director-worker-chain-fix-v1.md` §2②（本包的来源）⑥ 记忆 `tier1-codex-exec-no-ondemand-read-inject` / `real-codex-run-flaky-verify-by-artifact`。
- **全程中文。子线不 commit。** 一句话：**写一个薄 planned-task 链驱动——按 `depends_on` 拓扑序遍历 director 的 planned_tasks，每个 `execute_project_workflow_node_at(work_item=该任务的)` 过 S1 闸真跑 worker、失败即停+runaway 上限；圈固定测试项目 + 4 护栏；复用现成 execute/prepare/沙箱，闸/沙箱/chain controller 本体 0-diff。**

## 1. 拍板摘要

- **要做的事**：建薄链驱动，让主管 LM 拆的**多步自包含计划**（核对→创建→回读）**按序真驱动 worker 链**，每步过 S1 闸真 codex 跑，**最终在测试项目跑出结果**（建 proof + 回读核验）。补全闭环：「对话→AI 编排→审→**跑→看结果**」。
- **代价**：一轮——薄驱动（拓扑遍历 + 逐步 execute + 失败即停/runaway）+ 真链跑验证。做完后**第一次看见 LM 多步计划真驱动一条 worker 链干成事**。
- **关键澄清**：复用现成 `execute_project_workflow_node_at`（S1 闸/沙箱）+ prepare 的 work_items；圈**固定测试项目**；**不放开**非测试/多项目/auto-approve（高危#4 那几条仍锁）。

## 一句话判据

判改动在不在本包——问：**「是不是建薄 planned-task 链驱动、按 depends_on 逐步过 S1 闸真跑 worker、圈测试项目 + 4 护栏，且没改闸/沙箱/chain controller 本体/prepare、没放开非测试/多项目、没绕 S1?」** 是 → 做；否（尤其放开非测试连环/多项目、改闸沙箱、绕 S1）→ **停、回主导线。**

## 2. 建什么

**薄 planned-task 链驱动**（新函数/小模块）：
1. 入参：director 的 `planned_tasks`（带 `depends_on`）+ 已授权方案/已 prepare 的 dispatches。
2. 按 `depends_on` **拓扑排序**（复用 `workflow_chain_topological_order`——可开 `pub(crate)`；或自写小拓扑）。
3. 逐任务：`execute_project_workflow_node_at(work_item=该任务的 work_item, …)` **过 S1 闸**真跑 worker（同-role 任务共享 1 节点没关系——每次用**该任务的 work_item**、objective 各异 → 跑出 N 步）。worker prompt = 该任务的**自包含 objective**（①③ 已保证自包含）。
4. **失败即停**（某步 worker failed/exit≠0 → 停链）+ **runaway 上限**（如 `min(task_count, 50)`）。
5. （可选）result-forward：仅当某任务真需上一步**产出**才接最小 result→下一步 prompt 注入；自包含任务大多只需顺序+失败即停。

## 3. 安全死线（本包死线·必须成立）

- **圈固定测试项目**：每节点 worker `authorized ⟹ path-lock 命中测试项目`（S1 铁律）；非测试 root → 闸拦。正反钉死。
- **4 护栏在**：失败即停 / runaway 上限（driver 实现）/ 可中断 / 审计（走现成 dispatch 记录）；**path-lock/沙箱靠每节点 S1 闸**（现成）。**不放开非测试连环 / 多项目接力 / auto-approve**（高危#4 仍锁）。
- **scope 不扩**：worker 写范围 = 主管从授权 `scope_draft` 派生（测试项目）。
- **0-diff**：S1 闸 `decide_real_execution_command` / `command_plan_for` 沙箱 / `workflow_chain_controller` **本体** / prepare 派发机器 **不改逻辑**（只新增薄驱动 + 至多把拓扑 fn 开 `pub(crate)`；若必须改本体先回主导线）。
- **自动测试不真起 codex**（stub runner）；真链跑仅 `#[ignore]` + 用户在场。
- **碰线就停**：放开非测试/多项目 / 改闸沙箱/chain 本体 / 绕 S1 → 停、回主导线。

## 4. TDD 验收门

- **链 stub 全链**：planned_tasks(多·带 depends_on) → 薄驱动按拓扑序 → 每步 `execute`(stub) 过 S1 闸、**依赖序对**、某步失败 → **失败即停**拦、超 runaway 上限 → 拦。
- **path-lock 正反**：节点非测试 root 拦；测试项目+授权 放。
- **regression**：S1 闸/沙箱/chain controller 本体/prepare 0-diff（或经主导线批）、既有 §6/director/dispatch 测试全绿、`cargo test --lib` 计数不降。
- **全量**：`cargo test --lib` / `cargo fmt -- --check` / `git diff --check`。

## 5. 真跑验证（单独步·`#[ignore]`/用户在场·固定测试项目·圈 4 护栏）

stub 验通 + 主导线核实物**后**：测试项目造「proof-goal 已授权方案」→ 主管真 LM 拆**自包含**任务链（核对→创建→回读）→ 薄驱动按序跑 → **多 worker 真 codex 接连跑** → 最终**真建出 proof + 回读核验**。验：**LM 多步计划真驱动 worker 链干成事**（proof 在测试项目、内容对、链按序跑完）、每节点过 S1 闸、path-lock 命中、**沙箱只动测试目录、`.codex`/auth 没碰**、4 护栏在（失败即停/runaway/可中断）。flake → retry。`#[ignore]` 默认不跑。

## 6. 本包不做（deferred）

- 非测试真实项目真跑（高危#1）、多项目接力 / auto-approve（高危#4）。
- 改 S1 闸 / 沙箱 / chain controller 本体 / prepare 逻辑（只新增薄驱动；必须改先回主导线）。
- tier-2 / API / 别角色（秘书/全局主管）/ 前端（看结果 UI 归布局重做）。

## 7. 回交

- 跑 §4；回交：薄驱动 diff（确认闸/沙箱/chain 本体/prepare 0-diff 或经批）+ 链 stub 证据（拓扑序/失败即停/runaway）+ path-lock 正反 + 真跑证据（proof + 链按序跑完 + 对得上计划）+ confinement 实物（沙箱只动测试目录、auth 没碰）+ 4 护栏证据 + 计数 → 主导线核实物（重跑 + 扫 diff + 真链跑用户在场看 LM 计划真驱动 worker 链 + 核 confinement）。子线不 commit。

## 8. 不接受为

- 不接受为：改了闸/沙箱/chain 本体/prepare 没经批 / worker 跑了非测试项目 / 放开了多项目或 auto-approve / 绕了 S1 闸 / scope 被扩 / 失败不停或无 runaway 上限 / 自动测试真起 codex / 链没真跑出结果。
- 不接受为 S3 主管整体完成（本包只到「LM 多步计划真驱动 worker 链在测试项目跑出结果」；非测试/多项目/UI/别角色另说）。

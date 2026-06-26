# 实现任务包：S3 主管→worker 链修复（自包含任务 + 多任务依赖链）· 主导线 → 执行线 v1

日期：2026-06-25　性质：**高危#4-轻档**（固定测试项目内的自动连环——多 worker 按依赖跑；4 护栏在 + S1 闸每节点 + path-lock；真跑用户在场）。
上游：单任务派发接线已建·**未 commit**（stub 在 lib.rs，真跑 FAILED）；失败根因 + 修法见 Agent SDK 深挖（主导线核过官方文档）。**本包修对它 + 上链。**

## 0. 接手须知 + 为什么修

- 你是**执行线**。子线不 `git add`/`commit`。
- **真跑为什么挂（主导线核过）**：`s3_director_dispatch_real_run` FAILED——主管 LM 拆得对（核对→创建→回读），但 **worker 在干净隔离上下文里、只看任务 prompt**，拿不到任务引用的"已注入方案"。worker 诚实拒绝（没瞎编）。
- **Agent SDK 铁律（深挖确认）**：subagent **不继承父上下文，唯一信道是 prompt 字符串**；**编排者是集成层，别指望 subagent 自己集成/共享**。这正是 codex 一次性 exec 的行为（记忆 `tier1-codex-exec-no-ondemand-read-inject`）。
- 先读：① `director_agent.rs`（档案 + planned_tasks 产出）② `workflow_chain_controller.rs`（拓扑序跑节点 + 4 护栏——**看它喂不喂上一节点结果进下一节点**）③ 单任务那 2 个 stub 测试 + `s3_director_dispatch_real_run`（在 lib.rs，接着改）④ `prepare_authorized_auto_dispatch`（planned_tasks→work_item/node）。
- **全程中文。子线不 commit。** 一句话：**主管档案改产「自包含」任务（具体解析进 objective、不引用方案）+ 把多任务按 depends_on 喂进现成 chain controller 跑（每节点 = S1 闸 worker 真跑）+ worker 结构化返回；圈测试项目 + 4 护栏。**

## 1. 拍板摘要

- **要做的事**：修主管→worker 交接——主管产**自包含**任务（worker 不需再看方案就能干）+ 上**多任务依赖链**（核对→创建→回读 按序跑、每步过 S1 闸真 worker）。补全闭环：主管 LM 的多步计划**真驱动 worker 链跑出结果**。
- **代价**：一轮——档案改 + 链接线（复用 chain controller）+ worker 返回格式。做完后**第一次看见 LM 计划驱动一条 worker 链干成事**。
- **关键澄清**：复用现成 chain controller（4 护栏）/ S1 闸 / 沙箱（0-diff）；圈**固定测试项目**；不放开非测试/多项目（高危#4 那几条仍锁）。

## 一句话判据

判改动在不在本包——问：**「是不是在改主管产自包含任务 + 把 planned_tasks 喂现成 chain controller 按 depends_on 跑、每节点过 S1 闸、圈测试项目 + 4 护栏，且没改闸/沙箱/chain 本体、没放开非测试/多项目?」** 是 → 做；否（尤其放开非测试连环/多项目、改闸沙箱、绕 S1）→ **停、回主导线。**

## 2. 建什么（三处修，深挖定的）

**① 主管档案改产「自包含」任务**（核心修）
- 档案里**硬性要求**：每个任务的 `objective` **把具体解析进去**——目标文件**完整路径**、**要写的内容**、验收标准、scope——**不许说"按已注入方案/参见上文"**（worker 看不到）。主管已拿到方案,它的职责就是把方案**翻译成 worker 能独立执行的自包含任务**。
- 测试钉死：产出任务 objective **不含**"已注入方案/参见/上文/上一步"这类引用词;含具体路径/内容。

**② 多任务依赖链（薄 planned-task 链驱动·执行线 recon 已纠正 2026-06-25）**
- **⚠️ 纠正（不是"喂现成 chain controller"）**：核实物发现 `workflow_chain_controller` 跑的是**工作流节点**（按 `edges` 拓扑序），且 `prepare` 建 **per-role 节点**（`c4_node_id(wf,role)`，同 role 多任务被 ensure_node 合并成 1 节点）——所以"核对→创建→回读"3 个 codex-dev 任务只产 1 节点，chain controller 装不下 director 的 planned_tasks。**「喂现成 chain controller」对同-role 多任务不成立。**
- **正解 = 写一个薄 planned-task 链驱动**：按 `depends_on` 拓扑序遍历 director 的 planned_tasks，每个 `execute_project_workflow_node_at(work_item=该任务的)` 过 **S1 闸**真跑、**失败即停 + runaway 上限**；同-role 任务共享 1 节点没关系（每次用**该任务的 work_item**、objective 各异 → 按序跑出 N 步）。
- **复用 execute（S1 闸/沙箱）+ prepare 的 work_items**；`workflow_chain_controller` **本体 0-diff**（复用拓扑可把私有 `workflow_chain_topological_order` 开 `pub(crate)`，或 driver 自写小拓扑——实现时定）。4 护栏：失败即停/runaway 在 driver 实现，审计走现成 dispatch 记录，path-lock/沙箱靠每节点 S1 闸（现成）。
- **result-forward**：自包含任务大多只需顺序+失败即停；仅当某任务真需上一步**产出**才接最小 result→prompt 注入。

**③ worker 结构化返回**
- worker 任务里要它**按结构返回**结果（做了啥/产出/成败），主管/链能 parse 了决策或往下喂（接现成 C5 worker 汇报）。

## 3. 安全死线（本包死线·必须成立）

- **圈固定测试项目**：每节点 worker `authorized ⟹ path-lock 命中测试项目`（S1 铁律）;非测试 root → 闸拦。
- **4 护栏在**（chain controller 现成）：runaway 上限 / 可中断 / 审计 / 失败即停。**不放开非测试连环 / 多项目接力 / auto-approve**（高危#4 仍锁）。
- **scope 不扩**：worker 写范围 = 主管从授权 `scope_draft` 派生（测试项目）。
- **0-diff**：S1 闸 `decide_real_execution_command` / `command_plan_for` 沙箱 / `workflow_chain_controller` **本体**/ prepare 派发机器 **不改逻辑**（只接线 + 喂 planned_tasks;若 ② result-forward 必须改 controller,先回主导线确认）。
- **自动测试不真起 codex**；真链跑仅 `#[ignore]` + 用户在场。
- **碰线就停**：放开非测试/多项目 / 改闸沙箱 / 绕 S1 → 停、回主导线。

## 4. TDD 验收门

- **自包含任务**：主管产出任务 objective 含具体（路径/内容）、**不含引用词**（钉死）。
- **链 stub 全链**：director→planned_tasks(多·带 depends_on)→prepare→chain controller 按序→execute(stub) 每节点过 S1 闸、依赖序对、失败即停拦。
- **path-lock 正反**：节点非测试 root 拦;测试项目+授权 放。
- **regression**：S1 闸/沙箱/chain 本体/prepare 0-diff（或 result-forward 改动经主导线批）、既有 §6/director/dispatch 测试全绿、`cargo test --lib` 计数不降。
- **全量**：`cargo test --lib` / `cargo fmt -- --check` / `git diff --check`。

## 5. 真跑验证（单独步·`#[ignore]`/用户在场·固定测试项目·圈 4 护栏）

stub 验通 + 主导线核实物**后**：测试项目造「proof-goal 已授权方案」→ 主管真 LM 拆**自包含**任务链 → 喂 chain controller → **多 worker 按序真 codex 跑** → 最终**真建出 proof + 回读核验**。验：**LM 计划真驱动 worker 链干成事**（proof 在测试项目、内容对、链按 核对→创建→回读 跑完）、每节点过 S1 闸、path-lock 命中、**沙箱只动测试目录、`.codex`/auth 没碰**、4 护栏在（失败即停/可中断）。flake → retry。`#[ignore]` 默认不跑。

## 6. 本包不做（deferred）

- 非测试真实项目真跑（高危#1）、多项目接力 / auto-approve（高危#4）。
- 改 S1 闸 / 沙箱 / chain controller 本体逻辑（只接线;result-forward 若必须改先回主导线）。
- tier-2 / API / 别角色 / 前端（看结果 UI 归布局重做）。

## 7. 回交

- 跑 §4；回交：实现 diff（确认闸/沙箱/chain 本体/prepare 0-diff 或改动经批）+ 自包含任务断言 + 链 stub 证据 + path-lock 正反 + 真跑证据（proof + 链跑完 + 对得上计划）+ confinement 实物 + 4 护栏证据 + 计数 → 主导线核实物（重跑 + 扫 diff + 真链跑你在场看 LM 计划真驱动 worker 链 + 核 confinement）。子线不 commit。

## 8. 不接受为

- 不接受为：任务还引用方案（worker 拿不到）/ 放开非测试或多项目连环 / 改了闸沙箱 chain 本体没经批 / worker 跑非测试 / scope 被 LM 扩 / 绕 S1 闸 / 自动测试真起 codex / 链没真跑出结果。
- 不接受为 S3 主管整体完成（本包只到「LM 计划真驱动 worker 链在测试项目跑出结果」;非测试/多项目/UI/别角色另说）。

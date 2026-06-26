# 实现任务包：S3 主管→worker 链 — 修缺陷 + 补可中断 + 真链跑（**链已存在·本包非建链**）· 主导线 → 执行线 v1

日期：2026-06-26　性质：**高危#4-轻档**（固定测试项目内自动连环——多 worker 按依赖跑；前提 4 护栏在 + 每节点过 S1 闸 + path-lock；真链跑用户在场）。

## 0. 接手须知（冷启即读，本包自包含）

- 你是**执行线**（新开、干净上下文）。**子线不 `git add`/`commit`。** 全程中文。
- **核实物头条（别再走弯路）**：薄链驱动 `run_director_task_chain` **已经存在** —— `director_agent.rs:214`，committed in `75ab2c0`，2 个 stub 测试 `s3_director_chain_runs_all_prepared_tasks_topo` / `s3_director_chain_fail_stop_halts` **已绿（主导线刚重跑过 2 passed）**。**本包不是建链**——是**修它的 2 个缺陷 + 补「可中断」护栏 + 钉死断言 + 接真多-worker 链跑出 proof**。
- **本包取代** `tasks/2026-06-25-s3-director-worker-chain-fix-v1.md` 与 `tasks/2026-06-25-s3-director-chain-driver-v1.md`——那两个把链说成「待建」，是 canon 漂移（链就在它们引用为前置的 `75ab2c0` 里）。**以本包为准，别按那两个从头建。**
- **先读（核过的实物坐标）**：
  - ① `director_agent.rs:201-296`（`run_director_task_chain` 现状 + `DirectorChainOutcome`）。
  - ② `workflow_chain_controller.rs` 停链机制：`chain_run_stop_requested:106` / `ensure_chain_run_record:121` / `set_chain_node_state:186` / `finalize_chain_run:223` / `append_chain_audit:240` / `stop_project_workflow_chain_at:548` / `run_project_workflow_chain_at:296`（边界查 flag 的范本）/ `start_project_workflow_chain`（async + spawn_blocking，注释解释为什么）。
  - ③ `lib.rs`：2 个 director chain stub（`s3_director_prepared_chain` helper + 上面两测试）/ `project_workflow_chain_interrupts_at_node_boundary_on_stop_flag:6050`（**可中断 stub 范本**）/ `s3_director_dispatch_real_run`（真跑范本，但只跑第一个任务、不走链）。
  - ④ `c4_c6_workflow_governance_entrypoints.rs`：`annotate_project_director_planned_tasks:1788`（**看懂为什么所有任务都带 work_item/node**——1805/1807 行无条件设，blocked/needs_binding 也设）/ `prepare_authorized_auto_dispatch_for_index_at:51`。
  - ⑤ 记忆 `tier1-codex-exec-no-ondemand-read-inject` / `real-codex-run-flaky-verify-by-artifact`。
- **关键结构事实**：这些文件全 `include!` 进 crate root 同一模块（`lib.rs:125` commands / `:272` controller / `:278` director_agent）→ `director_agent.rs` 现在就直接调 controller 的「私有」fn（`workflow_chain_topological_order` 等）且能编译。**所以复用 controller 停链 helper 无需改可见性 → `workflow_chain_controller.rs` 必须保持 byte-0-diff。**
- **一句话**：修 filter（尊重授权状态）+ 补可中断（复用现成链记录+停链命令，链记录可被找到、节点边界查 flag）+ 钉死多任务断言 + 真多-worker 链跑出 proof；圈测试项目，闸/沙箱/execute/prepare/controller 本体 0-diff。

## 1. 拍板摘要

- **要做的事**：把这条**已存在**的薄链驱动，从「骨架对、安全核稳，但有 1 bug + 缺 1 条决策要的护栏 + 没真跑验」修到「**达 4 护栏门槛 + 真跑验过**」。具体五项：
  - **B1 · 修 filter（中危·撒谎注释）**：现 filter（`director_agent.rs:243-248`）只查 `work_item_id`/`node_id` 在不在 → 跳过 `_`。但 `annotate`（`c4_c6_...:1805/1807`）给**所有**任务（含 blocked / needs_binding）都设了 work_item+node → `_ => continue` **永不命中**，注释「被 guard 拦/needs_binding 的跳过」**是假的**。**改为只派 `status == "prepared"` 的任务**，其余跳过 + 记录（blocked/needs_binding 不进 execute）。
  - **B2 · 补可中断（头条·决策要求的第④护栏）**：复用现成链记录 + `stop_requested` + 审计。驱动**起链时建一条 running `workflow_chain_runs` 记录**（`ensure_chain_run_record`，order = 拓扑序的 planned_task_id 列表，max_nodes = runaway 上限）；**每任务边界 read-fresh state → `chain_run_stop_requested(chain_run_id)`** → 真则 `finalize_chain_run("stopped")` + 返回 `stopped_reason="user_stop_requested"`（已完成任务保留、断点续友好）；每任务 `set_chain_node_state`（running→completed/failed/skipped）；末尾 `finalize_chain_run`。**沿用现成 `stop_project_workflow_chain` 命令**（按 `project_id+workflow_id+state==running` 找 → 会找到本驱动建的记录，**0 新停链命令**）。**这同时补上 F3 链级审计**（链起/每节点/链停经 `append_chain_audit`）。
  - **F4 · 钉死多任务断言（低危·防退化）**：`s3_director_chain_runs_all_prepared_tasks_topo` 现只断言 `completed == prepared_count` 且 `prepared_count >= 1`（**只 1 任务也过**）。补 `assert!(prepared_count >= 2)` + **依赖序断言**（「搭骨架」先于「接业务」——可查 chain_run 记录 node 顺序或 dispatch 时序）。
  - **F5 · 健壮（低危）**：拓扑边按 title 建——**重复 title 报错**（防 `find` 取第一个、后一个永不跑）；**depends_on 指向不存在 title 记录/警告**（不静默丢）。
  - **C2 · §5 真跑（里程碑）**：真 LM 拆自包含多步链（核对→创建→回读）→ 驱动按序 → 多 worker 真 codex 接连跑 → proof 建出 + 回读核验。
- **代价**：一轮。做完链才算「达 4 护栏门槛 + 真驱动 worker 链干成事」。
- **关键澄清**：复用现成 `execute_project_workflow_node_at`（S1 闸/沙箱）+ prepare 的 work_items + controller 的停链/审计 helper；圈**固定测试项目**；**不放开**非测试/多项目/auto-approve（高危#4 那几条仍锁）。

## 一句话判据

判改动在不在本包——问：**「是不是在修这条已存在的薄链驱动（filter / 可中断 / 断言 / 健壮）、复用现成 execute+controller 停链机制、圈测试项目 + 4 护栏、真跑多-worker 链，且没改闸/沙箱/execute/prepare/controller **本体**、没放开非测试/多项目、没绕 S1?」** 是 → 做；否（尤其放开非测试连环/多项目、改闸沙箱、绕 S1、把链当没建从头重写）→ **停、回主导线。**

## 2. 建什么（对**现有** `run_director_task_chain` 的增改）

1. **B1 filter**：循环里把 `match (workflow_node_id, work_item_id) { (Some,Some)=>…, _=>continue }` 改为**先判 `task.status`**——只 `"prepared"` 进 execute；`"blocked"`/`"needs_binding"`/其它 → `set_chain_node_state(..,"skipped")` + 记一笔，不派。**同步把那句撒谎注释改成实话**。
2. **B2 可中断 + 链记录**：
   - 起链：read state → `ensure_chain_run_record(value, project_id, workflow_id, order=拓扑 planned_task_id, max_nodes=cap, ts)` → write。`chain_run_id` 进 `DirectorChainOutcome`。
   - 每步**前** read-fresh state → `chain_run_stop_requested(chain_run_id)`？真 → `finalize_chain_run("stopped")` + audit + return（`stopped_reason="user_stop_requested"`）。
   - 每步：`set_chain_node_state(running)` → `execute_project_workflow_node_at(...)`（**不动**）→ 据结果 `set_chain_node_state(completed/failed)`；failed → finalize("failed") + 失败即停（保留现有语义）。
   - 末：`finalize_chain_run("completed")`。
   - **state 交错铁律**：驱动写 `workflow_chain_runs`、execute 写 `work_items/dispatches/bindings`——**每次更新链记录前必 read-fresh**（别缓存 execute 前的 `value`），各改各的顶层 key、不互相 clobber。
3. **DirectorChainOutcome 增字段**：`chain_run_id` / `skipped` 计数（B1）/ 透出每步成败（worker 结构化返回③已由自包含 objective + execute readback 覆盖，确认 outcome 能透出）。
4. **F4/F5**：见 §1。
5. **runaway 上限**：确认 `max_tasks` 传入有 `min(task_count, 50)` 的实际取值（现是裸 param；调用处/测试给 cap）。

## 3. 安全死线（本包死线·必须成立）

- **圈固定测试项目**：每节点 worker `authorized ⟹ path-lock 命中测试项目`（S1 铁律，execute 现成）；非测试 root → 闸拦。**正反钉死。**
- **4 护栏全在（本包补齐第②条）**：① runaway 上限（driver·确认 cap）② **可中断（本包新增·复用 stop 机制）** ③ 审计（本包新增链级 + execute per-dispatch）④ 可回滚（测试项目 git + execute backup）。**不放开非测试连环 / 多项目接力 / auto-approve。**
- **0-diff**：`decide_real_execution_command` / `command_plan_for` 沙箱 / `execute_project_workflow_node_at` **本体** / prepare 派发机器 / **`workflow_chain_controller.rs` 本体（byte-0-diff——只调不改，include! 同模块无需改可见性）** 不改逻辑。**改动集中在 `director_agent.rs`（+ `lib.rs` 测试）。** 若发现必须改 controller 本体 / execute / prepare → **停、回主导线**。
- **scope 不扩**：worker 写范围 = 主管从授权 `scope_draft` 派生（测试项目）。
- **自动测试不真起 codex**（stub runner）；真链跑仅 `#[ignore]` + 用户在场。
- **碰线就停**：放开非测试/多项目 / 改闸沙箱 / 改 controller·execute·prepare 本体 / 绕 S1 → 停、回主导线。

## 4. TDD 验收门

- **B1 跳过**：造「1 个 blocked + 1 个 prepared」任务集 → 断言**只 prepared 跑**（blocked 不进 execute、记 skipped）。
- **B2 可中断**（仿 `project_workflow_chain_interrupts_at_node_boundary_on_stop_flag`）：runner 跑完第一步顺手置该 running 链 `stop_requested=true` → 断言链在**下个边界停**、`completed==1`、`stopped_reason` 含 user_stop、chain_run `state=="stopped"`、审计有 `workflow_chain_stop_requested`；**+ 现成 `stop_project_workflow_chain_at` 能找到并停本驱动的 running 记录**（证「停」命令 0-diff 就管用）。
- **链 stub 全链（F4 加强）**：`prepared_count >= 2` + **依赖序**（搭骨架 先于 接业务）+ 全完成。`s3_director_chain_fail_stop_halts` 仍绿。runaway 上限拦。
- **F5**：重复 title → 报错；dangling dep → 记录不静默丢。
- **path-lock 正反**：节点非测试 root 拦；测试项目+授权 放。
- **regression**：闸/沙箱/execute/prepare 0-diff + **controller `git diff` 字节为 0**（执行线自证）；既有 §6/director/dispatch/chain 测试全绿；`cargo test --lib` 计数不降。
- **全量**：`cargo test --lib` / `cargo fmt -- --check` / `git diff --check`。

## 5. 真跑验证（单独步·`#[ignore]`/用户在场·固定测试项目·圈 4 护栏）

stub 验通 + 主导线核实物**后**：仿 `s3_director_dispatch_real_run` **扩成链**——测试项目造「proof-goal 已授权方案」（多步：建文件→回读核验）→ 真 director LM 拆**自包含**任务链 → 驱动按序跑 → **多 worker 真 codex 接连跑** → 最终**真建出 proof + 回读核验**。验：**LM 多步计划真驱动 worker 链干成事**（proof 在测试项目、内容对、链按 核对→创建→回读 跑完）、每节点过 S1 闸、path-lock 命中、**沙箱只动测试目录、`.codex`/auth 没碰**、4 护栏在（失败即停/runaway 真跑证；可中断由 §4 stub 证）。真 codex flake → retry。`#[ignore]` 默认不跑。

## 6. 本包不做（deferred·**显式·不静默**）

- **C1 · 生产 async 起链命令（「按计划开干」入口）**：现 `run_director_task_chain` **没接进任何 Tauri 命令**（只测试调）。app 内真正可起/可中断整条链，需一个 **async + spawn_blocking** 起链命令（范本 `start_project_workflow_chain`——注释明说同步链跑主线程会冻 UI、连停链都点不动、**可中断形同虚设**）；它还要 LM+prepare+chain 编排 + UI，是「按计划开干」入口的一部分，归后续包 / 布局重做。**本包交付：可中断在驱动层 + stub 验过（链记录可被现成 stop 命令找到），但 app 端起/停整链端到端要 C1 才通。** 明记于此、不当已完成。
- 非测试真实项目真跑（高危#1）、多项目接力 / auto-approve（高危#4）。
- 改 S1 闸 / 沙箱 / `execute` / prepare / controller **本体**逻辑。
- tier-2 / API / 别角色（秘书/全局主管）/ 前端看结果 UI。

## 7. 回交

- 跑 §4；回交：**实现 diff**（确认闸/沙箱/execute/prepare 0-diff + **controller `git diff` 字节 0** 自证）+ B1 跳过证据 + **B2 可中断 stub 证据**（停在边界/已完成保留/审计/stop 命令找到）+ F4 多任务+依赖序断言 + F5 + path-lock 正反 + **真跑证据**（proof + 链按序跑完 + 对得上计划）+ **confinement 实物**（沙箱只动测试目录、auth 没碰）+ 4 护栏证据 + 计数 → **主导线核实物**（重跑 + 扫 diff + 真链跑你在场看 LM 计划真驱动 worker 链 + 核 confinement）。**子线不 commit。**

## 8. 不接受为

- 不接受为：把链**当没建从头重写** / filter 仍不尊重授权状态（注释还撒谎）/ 可中断没真接（链记录 stop 命令找不到，或边界不查 flag）/ 改了闸·沙箱·execute·prepare·controller 本体没经批 / worker 跑了非测试项目 / 放开多项目或 auto-approve / 绕 S1 闸 / scope 被扩 / 失败不停或无 runaway 上限 / 自动测试真起 codex / 链没真跑出结果。
- **不接受为 S3 主管整体完成**（本包只到「已存在的链**修对 + 达 4 护栏门槛 + 真跑出结果**」；C1 起链命令 / 非测试 / 多项目 / UI / 别角色 另说）。

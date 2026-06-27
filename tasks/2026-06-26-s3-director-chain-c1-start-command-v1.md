# 实现任务包：S3 主管→worker 链 · C1 生产起链命令（app 内 async 起/停整链入口）· 主导线 → 执行线 v1

日期：2026-06-26　性质：**高危#4-轻档**（固定测试项目内的自动连环——把已建的主管链驱动接成 app 命令；前提 4 护栏在 + 每节点过 S1 闸 + path-lock；真跑用户在场）。

## 0. 接手须知（冷启即读，本包自包含）

- 你是**执行线**（新开、干净上下文）。**子线不 `git add`/`commit`。** 全程中文。
- **核实物头条**：主管链驱动 `run_director_task_chain`（`director_agent.rs:231`，`pub(crate)`）+ 4 护栏（含可中断）+ §5 真跑**都已建并 committed（`db394be`）**，cargo 608/0/32 绿。**本包只差一件事：把这个驱动接成一个 app 能调的 async 命令**，让界面能「按计划开干」起整条主管链、并能中途停。
- **为什么 C1 很小（B2 的红利）**：B2 已让主管链驱动建一条标准 `workflow_chain_runs` 记录（按 project_id+workflow_id），所以——
  - **停链**：现成 `stop_project_workflow_chain` 命令（`workflow_chain_controller.rs:625`）按 (project_id, workflow_id, running) 找记录置 stop_requested → **不改一字就能停主管链**（B2 的 stub `s3_director_chain_interrupts_at_task_boundary_on_stop` 已证它找得到）。
  - **进度**：现成 `get_project_workflow_chain_status`（`workflow_chain_controller.rs:655`）读最新链记录 → 主管链进度**免费复用**。
  - 所以 **C1 = 只加一个 async 起链命令**，停/进度/审计全现成。
- **为什么前端回传 planned_tasks（不是从 state 重载、也不重跑 LM）**：`depends_on`/planned_tasks **没持久化进 workflow-state**（核过：dispatch 记录只存 node_id/work_item_id/c4_planned_task_id/prompt_preview，无拓扑信息；读模型 `PreparedAutoDispatchReadModel` 从内存 plan 现搭、不含 depends_on）。产品流是 `preview_project_director_task_plan`（LM 拆·命令在）→ 用户**审** → `prepare_authorized_auto_dispatch`（命令在，返回 `AuthorizedPreparedDispatchResult.plan.planned_tasks`，**已含 depends_on/已 annotate**）。**前端手里就有这份已审计划** → 起链时回传给 C1。**关键产品正确性：跑的必须是用户审过的那份计划**——绝不在起链时重跑 LM（会拆出没审过的计划=跑了用户没批的东西）。
- **先读**：① `director_agent.rs:201-296`（`run_director_task_chain` 签名/`DirectorChainOutcome`）② `workflow_chain_controller.rs`：`start_project_workflow_chain`（**async + spawn_blocking 范本**·注释解释为什么同步链会冻 UI+停不动）/ `stop_project_workflow_chain` / `get_project_workflow_chain_status` / `ProjectWorkflowChainRunRequest`（请求范本，字段 project_root/workflow_id/max_nodes）③ `commands.rs:748-763`（`preview_project_director_task_plan`/`prepare_authorized_auto_dispatch` 两命令，看 planned_tasks 怎么从前端流过来）④ `lib.rs` 的 `s3_director_chain_real_run`（§5 真跑·C1 真跑照它改成走命令路径）⑤ 记忆 `real-codex-run-flaky-verify-by-artifact`。
- **一句话**：加一个 async `start_project_director_chain` 命令——收前端回传的已审 planned_tasks → `spawn_blocking` 调 `run_director_task_chain` → 返回 outcome；停/进度复用现成命令；圈测试项目、4 护栏、gate/沙箱/prepare/controller/driver 本体 0-diff。

## 1. 拍板摘要

- **要做的事**：把已建的主管链驱动接成 **app 能调的 async 起链命令**，补全「对话→AI 编排→审→**跑（界面里点）→看结果**」最后那一下的**后端入口**。停链/进度复用现成命令。
- **代价**：一轮——一个 async 命令 + 注册 + 请求/返回类型 + `DirectorChainOutcome` 加 `Serialize` + 测试。做完**主管链在 app 层可起/可停/可查进度**（按钮摆哪是 S2 角色循环布局的事，见 §6）。
- **关键设计（已定·可在审时翻）**：**A·前端回传已审 planned_tasks**（prepare 0-diff、跑的就是用户审过的、no LM 重跑）。**不选 B/C**（从 state 重建/持久化 plan）——depends_on 没存、要么重跑 LM（错·跑没审的）要么改 prepare 加持久化（surface 大）；**C「持久化 plan + 起链时按 authorization_id 从 state 重载」= 到乙/生产档的稳健升级，本包 deferred**（§6）。

## 一句话判据

判改动在不在本包——问：**「是不是只加一个 async 主管起链命令、收前端已审 planned_tasks、spawn_blocking 调现成 `run_director_task_chain`、停/进度复用现成命令、圈测试项目 + 4 护栏，且没改 gate/沙箱/prepare/controller/driver **本体**、没放开非测试/多项目、没重跑 LM、没绕 S1?」** 是 → 做；否（尤其重跑 LM、放开非测试、改 gate/沙箱、新写停链/状态机）→ **停、回主导线。**

## 2. 建什么

1. **请求/返回类型**（`lib.rs` 或就近）：
   ```
   struct StartProjectDirectorChainRequest {
       project_root: String,
       workflow_id: String,
       planned_tasks: Vec<ProjectDirectorPlannedTask>, // 前端从 prepare 返回回传（已审·含 depends_on）
       max_nodes: Option<usize>,                        // 同节点链：只能更小、越不过 min(任务数,50)
   }
   ```
   - `DirectorChainOutcome` + `DirectorChainStep` 加 `#[derive(Serialize)]`（现是 `Debug`；命令要返回它）。**只加 derive、字段不动。**
2. **async 命令**（**严格照 `start_project_workflow_chain` 范本**）：
   ```
   #[tauri::command]
   async fn start_project_director_chain(request, state) -> Result<DirectorChainOutcome, String> {
       let path = state.workflow_state_path.clone();
       let index = read_index(&state)?;              // await 前取（State 不跨 'static 闭包）
       tauri::async_runtime::spawn_blocking(move || {
           let readback_db_path = codex_db::default_state_db_path();
           let runner = codex_local_runner::RealWorkflowNodeCodexRunner;
           run_director_task_chain(&path, &index, &readback_db_path, &runner,
               &request.project_root, &request.workflow_id,
               &request.planned_tasks, request.max_nodes.unwrap_or(50))
       }).await.map_err(|e| format!("主管链执行线程异常：{e}"))?
   }
   ```
   - **async + spawn_blocking 是死线**（不是可选）：主管链每节点真起 codex、长耗时；同步命令跑主线程会冻 UI + 让停链命令抢不到线程 → **可中断形同虚设**（范本注释原话）。
3. **注册命令**：加进 `lib.rs` 的 `tauri::generate_handler!` 列表。
4. **停/进度 = 0 新命令**：前端停链按钮调现成 `stop_project_workflow_chain`、进度轮询调现成 `get_project_workflow_chain_status`（都按 project_root+workflow_id 找主管链建的记录）。**本包不新写停链/状态命令**。

## 3. 安全死线（本包死线·必须成立）

- **圈固定测试项目**：`run_director_task_chain` 入口已有 `require_test_project_path_lock`（非测试 root 直接拒、连链记录都不建）+ 每节点 execute 的 S1 闸——C1 命令**不得绕过、不得自建更松的入口判定**。正反钉死（命令收非测试 root → Err）。
- **4 护栏在**：失败即停/runaway/可中断/审计——全在 driver（已建），C1 不削。
- **不重跑 LM**：C1 跑前端回传的 planned_tasks，**不调 `CliDirectorAgent::plan()`**（重跑=跑用户没审的计划）。
- **0-diff**：`decide_real_execution_command` / `command_plan_for` 沙箱 / `execute_project_workflow_node_at` / `prepare_authorized_auto_dispatch` 派发机器 / `workflow_chain_controller`（停/状态/记录）/ `run_director_task_chain` **本体**——**全不改逻辑**。本包只新增命令 + 请求类型 + 给 outcome 加 `Serialize` + 注册 + 测试。若发现要改上面任何本体 → **停、回主导线**。
- **不放开**非测试连环 / 多项目接力 / auto-approve（高危#4 仍锁）。
- **碰线就停**：重跑 LM / 放开非测试 / 改 gate·沙箱·prepare·controller·driver 本体 / 新写停链状态机 / 绕 S1 → 停、回主导线。

## 4. TDD 验收门（自动测试·stub·不真起 codex）

- **命令驱动链**：用 stub runner（`PermissiveExperimentRunner`）走 `start_project_director_chain` 的**内层逻辑**（`run_director_task_chain` 调用，async 壳可不进 tokio、直接测内层或用 `tauri::test`）：多 prepared 任务 → 按拓扑序全跑、outcome 对。
- **停链复用**（仿 B2 `s3_director_chain_interrupts_at_task_boundary_on_stop`）：跑中调现成 `stop_project_workflow_chain_at` → 边界停、`stopped_reason=user_stop_requested`。证 C1 建的记录被现成停命令认得。
- **进度复用**：跑后 `get_project_workflow_chain_status` 读得到该主管链最新状态。
- **path-lock 正反**：命令收非测试 root → Err（入口 path-lock 生效）；测试项目+prepared → 跑。
- **不重跑 LM**：断言命令路径不调 `plan()`（结构上 C1 不持有 director、只收 planned_tasks）。
- **regression**：gate/沙箱/execute/prepare/controller/driver **本体 0-diff**（扫 diff 自证）；`DirectorChainOutcome` 加 `Serialize` 后既有 608 测试全绿、计数不降。
- **全量**：`cargo test --lib` / `cargo fmt -- --check`（**只本包改的文件**·别 `--write` 碰预存偏差的 codex_db/codex_local_runner/mcp）/ `git diff --check`。

## 5. 真跑验证（单独步·`#[ignore]`/用户在场·固定测试项目·圈 4 护栏）

stub 验通 + 主导线核实物**后**：照 §5 `s3_director_chain_real_run` 改成**走命令路径**——preview→prepare 拿到 planned_tasks → **经 `start_project_director_chain`（async）** 起链 → 多 worker 真 codex 接连跑 → proof 建出 + 回读核验；**跑中用现成 `stop_project_workflow_chain` 真停一次**验 app 层可中断真生效（已完成保留、断点续）。验：proof 在测试项目内容对、链按序、每节点 S1 闸 path-lock 命中、沙箱只动测试目录 `.codex`/auth 没碰、停链在边界生效。flake → retry。`#[ignore]` 默认不跑。

## 6. 本包不做（deferred·**显式·不静默**）

- **UI 按钮的最终摆位**：「按计划开干」按钮归 **S2 角色循环接前端 = 工作流+记忆中心布局重做**（CURRENT ③·3）——那里才是方案授权→拆任务→开干的完整界面流。本包交付**后端命令**（app 可调、可停、可查进度）；最终 clickable 入口随 S2 布局落。**本包后主管链在命令层已可起/停/查，缺的只是按钮摆哪。**
- **Design C（持久化 plan + 起链时按 authorization_id 从 state 重载）**：到乙/生产档的稳健升级（state 作单一真相源、抗前端漂移）；本包用 A（前端回传），prepare 0-diff。
- 非测试真实项目真跑（高危#1）、多项目接力/auto-approve（高危#4）、改 gate/沙箱/prepare/controller/driver 本体、tier-2/别角色。

## 7. 回交

- 跑 §4；回交：**实现 diff**（确认 gate/沙箱/execute/prepare/controller/driver 本体 0-diff、只 +命令/类型/derive/注册/测试）+ 命令驱动链证据 + 停链复用证据 + 进度复用证据 + path-lock 正反 + 「不重跑 LM」证据 + 计数（608→新计数）→ **主导线核实物**（重跑 + 扫 diff + 真跑用户在场走命令路径起+停 + 核 confinement）。**子线不 commit。**

## 8. 不接受为

- 不接受为：C1 起链时**重跑 LM**（跑了用户没审的计划）/ 自建更松的入口绕过 `require_test_project_path_lock` / 新写停链或状态命令（该复用现成）/ 同步命令跑主线程（停不动=可中断形同虚设）/ 改 gate·沙箱·prepare·controller·driver 本体没经批 / 放开非测试或多项目 / 绕 S1。
- **不接受为 S3 主管整体完成 / 不接受为 UI 已接**（本包只到「主管链在**命令层**可起/可停/可查进度 + 真跑走命令路径验过」；按钮摆位归 S2 布局、非测试/多项目/别角色另说）。

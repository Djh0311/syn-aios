# 回交：S3 主管→派发联调（LM planned_tasks → prepare → S1 闸 → worker 真跑）· 执行线 → 主导线 v1

日期：2026-06-25　性质：**轻档**（圈固定测试项目·复用现成 prepare+S1 闸+worker 真跑路）　任务包：`tasks/2026-06-25-s3-director-dispatch-integration-v1.md`　上游：主管 agent 首切 `f5e2d22`

## 0. 一句话结论

主管 LM 的 planned_tasks 接进现成派发链跑通：`director.plan()` → `prepare_authorized_auto_dispatch`（喂显式 planned_tasks）→ prepared dispatch（过授权 guard·映射到 codex-dev 节点）→ `execute_project_workflow_node_at`（S1 闸）→ worker completed。**纯接线、零 glue、零代码改动**——闸/派发/沙箱/chain/agent 全 **0-diff**，只往 lib.rs 加了 2 个测试。**执行线未 commit。**

## 1. 链路（复用现成·无 glue）

`s3_director_dispatch_integration_stub`：
1. `create_active_*` 出已授权方案 + 绑 codex-dev 节点。
2. `StubDirector.plan()` → `planned_tasks`（target_role=codex-dev·scope 取自授权 scope_draft）。
3. `prepare_authorized_auto_dispatch_for_index_at(planned_tasks)` —— 喂 **director 的显式 planned_tasks**（非 vec![] 兜底）→ 过授权 guard → prepared dispatch。
4. `prep.workflow_node_id` == `{workflow_id}:node:codex-dev`（已绑节点）—— 因 `c4_node_id(wf, role) = "{wf}:node:{role}"`，target_role=codex-dev 正好映射到默认节点，**无需 glue**。
5. `execute_project_workflow_node_at`（S1 闸·复用 §6 路）→ stub worker → **completed**。

**为什么零 glue**：任务包预留了「planned_task → work_item composition gap 接最小 glue」——但实测 director 的 planned_tasks（scope 从授权派生、target_role=codex-dev）**直接被 prepare 接受、guard 授权、映射到已绑节点**，链路本就对得上。无任何命令本体/闸改动。

## 2. 安全死线（§3·全守住）

- **复用现成·0-diff** ✅：`prepare_authorized_auto_dispatch` / `execute_project_workflow_node_at` / S1 闸 `decide_real_execution_command` / `command_plan_for` 沙箱 / `workflow_chain_controller` / director_agent / consultant_agent **全 0 行 diff**。本任务只往 lib.rs 加测试。
- **worker 锁测试项目** ✅：worker 步走 `execute_project_workflow_node_at`（S1 闸·path-lock）；非测试 root → 闸拦（同闸，由 `s2_3_role_loop_worker_blocked_at_nontest_root` / `s1_step3_nontest_root_blocked_before_runner` 覆盖）。
- **scope 不扩** ✅：worker 写范围 = director 从授权 scope_draft 派生的（director_agent 已钉死）。
- **第一刀单任务** ✅：只派 `prepared_dispatches[0]`（多任务 depends_on 自动连环 = 后续走现成 chain controller，本包不做）。
- **不自动放开** ✅：没开非测试连环/多项目/auto-approve；没绕 S1 闸。
- **自动测试不真起 codex** ✅：stub 用 `PermissiveExperimentRunner`；真 codex 仅 `#[ignore]`。

## 3. 验收门

| 门 | 结果 |
|---|---|
| stub 集成全链 | ✅ `s3_director_dispatch_integration_stub`（director→prepare→prepared dispatch→execute completed·按序）|
| path-lock 正反 | ✅ 正向：stub 用测试项目 root 过闸；负向：worker 步同 `execute_project_workflow_node_at` 闸，由既有 nontest 测试覆盖 |
| regression | ✅ `cargo test --lib` = **601 passed / 0 failed / 31 ignored**（+stub 集成 +#[ignore]）；§6/S1-③/director/consultant 全绿 |
| 0-diff | ✅ 闸/prepare/派发/沙箱/chain/两个 agent 模块 全 0 行 |
| fmt / git diff --check | ✅ 干净 |
| 范围 | **只 lib.rs**（+2 测试，纯接线）|

## 4. §5 真跑（单独步·用户在场·固定测试项目）

写了 `#[ignore]` `s3_director_dispatch_real_run`：自定义 proof-goal 方案（"创建 s3-director-dispatch-proof.txt 写 token"）→ **真 director LM 拆** planned_tasks → prepare → 派第一个任务 → **worker 真 codex 跑出 proof 文件**。显式跑：`cargo test --lib s3_director_dispatch_real_run -- --ignored --nocapture`。验：打印 director 拆的任务 + worker 结果、proof 文件含 token（**LM 计划真驱动了 worker**）、confinement（测试项目只动 proof、auth 没碰）。真 codex flake → retry。
> 注：worker prompt = director 任务 objective（prepare 落进 work_item 任务包）；proof 能否出取决于 director 把 proof-goal 拆成了写文件任务——proposal goal 已写明确，但 LM 拆解有变动性，跑时你核 director 拆的任务 + worker 结果。

## 5. 主导线核实物 + 收口
- 重跑 `cargo test --lib`（601/31）；扫 diff 确认**只 lib.rs 加测试、闸/派发/沙箱/chain 0-diff、没绕 S1 闸、没自动放开多任务/多项目**。
- §5 真跑你在场：`s3_director_dispatch_real_run -- --ignored` 看 LM 计划真驱动 worker + proof + confinement。
- 执行线不 commit；你核实物 + commit + CURRENT 回写。

## 6. 本包没做（deferred）
多任务 depends_on 自动连环（走现成 `workflow_chain_controller` + 4 护栏）/ 非测试真跑(高危#1)/多项目接力(高危#4)/ 前端看结果 UI。本包只到「LM 计划真驱动一个 worker 跑出结果」。

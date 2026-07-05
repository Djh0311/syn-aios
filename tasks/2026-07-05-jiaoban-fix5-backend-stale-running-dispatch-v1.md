# 实现任务包:交办 fix5 后端(残料终章:中断遗留的 running 派发记录标结)· 主导线 → 执行线 v1

日期:2026-07-05　性质:**轻档**(编排侧数据接管;S1 闸/`has_inflight_dispatch`/execute 全 0-diff)。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**(后端)。**子线不 `git add`/`commit`。** 全程中文。
- **背景 = 用户真机[接着跑]第三撞(主导线已钉死,别再猜)**:
  - 报错:`fail_stop:node_error:…:real_execution_gate_blocked:duplicate_blocked:duplicate active attempt blocks real runner call`;
  - 盘上实况:**仅 1 条** `workflow_node_dispatches` 记录 state=="running"(07-03 崩掉那次·node=`{mario-wf}:default:node:codex-dev`·dispatch_id 尾 `1783010639745`);
  - 机制:`commands.rs::has_inflight_dispatch`(~2363)按 **(workflow_id, node_id) + state=="running"** 查重喂 `duplicate_blocked` → S1 拒起真跑。它的注释明说 "running 是同一次调用里推进到 completed/failed 的窗口、不残留"——**进程中途死掉是设计外**,留下永久墓碑;所有任务共用角色节点 `node:codex-dev` → 该工作流一切新派发全撞;
  - 定性:**闸是对的**(防两个真跑并行=四护栏),残料是错的。这是崩溃残料的**第三层也是最后一层**(work items/链记录 fix4 已治;execution_attempts 盘上全终态、不在此路径)。
- **前情**:fix4 的 `reconcile_stale_work_items_for_plan`(prepare 前·两分支合流处)就是本包的挂载点样板。
- **先读**:`commands.rs:2289-2385`(gate 输入装配 + `has_inflight_dispatch`——**只读懂、不改**)、`director_agent.rs` fix4 reconcile 与调用点、`c4_node_id`(`{wf}:node:{role}` 推导)、worker 派发的超时上限(codex_local_runner 的 timeout 配置——它决定"活派发最长能跑多久")。
- **一句话**:prepare 前,把本轮要用的 (workflow, 角色节点) 上**确证已死**的 running 派发记录标结(failed + 人话说明 + 审计);"确证已死"用**超龄判据**(> 活派发物理上限),没超龄的**绝不碰**(可能真在跑·留给闸拦=保护原语义)。

## 1. 拍板摘要

- **要做的事**:中断遗留的 running 派发墓碑不再永久卡死工作流;真在飞的照旧被闸保护。
- **为什么**:用户第三撞;不修则 fix4 疏通的路在执行那步又死。
- **代价**:一轮·后端·集中 `director_agent.rs`(+ lib.rs 测试)。

## 一句话判据

判改动在不在本包——问:**「是不是只在编排侧把『本轮节点上、超龄(>物理上限)的 running 派发记录』标结为 failed+审计,而 `has_inflight_dispatch`/S1 闸/execute 一字不动、未超龄的一概不碰、只碰本轮 (wf,node) 匹配的记录?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 超龄判据(先算清·写成常量+注释)
- 读 worker 真跑的超时上限(runner timeout,含刀1 retry=同任务再来一次):活派发的**物理最长窗口 ≈ timeout × 2 + 余量**。定常量如 `STALE_RUNNING_DISPATCH_MS`(建议 ≥ 20 分钟,注释写清推导:超过它还 running 的派发**物理上不可能还活着**——runner 到时必杀,唯一解释是进程中途死了)。

### 2.2 接管函数
- 新 `reconcile_stale_running_dispatches(path, planned_tasks, now_ms) -> Vec<String>`,挂在 fix4 reconcile 同一位置(两分支合流后·prepare 前):
  - 本轮节点集 = planned_tasks 的 target_role(fix3 钳位后)经 `c4_node_id(wf, role)` 推导去重;
  - 扫 `workflow_node_dispatches`:同 (workflow_id, node_id) 且 state=="running":
    - **超龄** → 标结:state→"failed" + 追加说明字段(如 message/warnings:「上次运行进程中断遗留,已由新一轮标结」)+ append 审计事件(带 dispatch_id);outcome warnings 加人话「已标结 1 条上次中断遗留的执行记录」;
    - **未超龄** → **不碰**;outcome warnings 加人话「该节点可能仍有一次执行没结束(X 分钟前起),本轮若被拦请稍后再试」——闸随后拦=原保护语义,但用户看得懂。
  - 派发记录没有正式迁移表(状态由 execute 流内联写)——本次是**受控的定点数据接管**:只允许改匹配记录的 state/说明字段,别的字段、别的记录一律不动;写法照链记录既有的 JSON 帮手风格,**必须带审计**。

### 2.3 范围纪律
- 只 (wf,node) 匹配本轮;canvas 或别的工作流的记录不扫;"prepared" 状态(合法残留·闸不数它)不碰;failed/completed/timed_out 终态不碰。

## 3. 安全死线

- **S1 闸 0-diff**:`commands.rs`(含 `has_inflight_dispatch`、gate 装配、execute)整文件不动;`codex_local_runner`/`real_execution_command`/`control_core`/`c4_c6`/`workflow_chain_controller`/两 store——全 0-diff;
- **防误杀真活**:未超龄绝不碰(宁可被闸拦+人话提示);超龄常量必须 > 物理上限并写清推导;
- 人闸/档位/path-lock 增删 0 命中;接管必带审计、只 append 审计 + 定点两字段。

## 4. 验收(执行线自己验)

- **单测·复刻撞死**:预置线上同款(node:codex-dev·running·老时间戳)→ re-plan 跑 → 标结 + 审计在 + 后续 execute 不再 duplicate_blocked → ran;
- **单测·防误杀**:预置**新鲜** running(now-1min)→ 不被碰 + warnings 有"可能仍在执行"人话 +(若走到 execute)仍被闸拦=原语义;
- **单测·范围**:别的 workflow / canvas 形状 node / prepared / 终态记录全不被碰(逐类断言原样);
- **单测·approved 路**:approved 分支同样接管(挂载点在合流后·两路同享);
- **regression**:`cargo test --lib` 计数不降;§3 全 0-diff 扫 diff 自证;fmt(只本包文件·skip_children)。

## 5. 本包不做(deferred)

- 改 `has_inflight_dispatch` 让它自带超龄判断(= 改 S1 闸·不做);进程活性心跳/注册表(治本·另议);canvas-run 历史工作项清理(卫生包);UI(fix3-UI 呈现已够——warnings 走现有通道)。

## 6. 回交

- 跑 §4;回交列:常量与推导、接管函数与挂载点、定点写的字段清单(自证只动 state+说明)、审计样例、防误杀/范围/approved 各测试证据、0-diff 自证、计数 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 不接受为:动了 `has_inflight_dispatch`/gate/execute / 未超龄也标结 / 扫了本轮之外的记录 / 改了 state+说明以外的字段 / 无审计的静默改 / 超龄常量小于物理上限。

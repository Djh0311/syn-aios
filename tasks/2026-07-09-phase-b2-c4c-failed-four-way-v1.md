# 实现任务包:B2·C4c failed 四选一(接现成机器·不造新)· 主导线 → 执行线 v1

日期:2026-07-09　性质:**中等**(接线为主·复用现成机器·人闸不动)。主导线已 measure-first + **防重造 grep 全仓**(见 §0·四块机器大半现成)。正本:节点状态机 §5.2 硬规则 + 七拍拍6。**C4 最后一片(C4a 终标✅+C4b 总结✅+本片)。**

## 0. 接手须知(冷启即读·前提+防重造已核)

- 你是**执行线**(后端)。**子线不 commit。** 全程中文。
- **现状缺口**:任务 failed → 现状**只「结束」一条路**(director_agent 各 failed 分支 `finalize_chain_run("failed")`·如 1446);蓝图 §5.2 硬规则要「failed 后由项目主管选**重试/退回/换会话/结束**」。
- **防重造 grep 结论(主导线 07-09·四块大半现成·别重造)**:
  1. **退回 = 复用 C4a**:`reset_work_item_for_director_rework` + `director_rework_attempts` + `DIRECTOR_FINAL_REWORK_BUDGET`(C4a 已造·needs_rework 预算制)——退回直接用这套,**别新造第二套 rework**;
  2. **换会话 = 复用 C1**:`create_and_bind_task_session` / `run_director_task_chain_with_session_creator`(C1 先生后绑)——换会话=给该任务建新会话再跑,**别新造建会话逻辑**;
  3. **结束 = 现成**:转移表 `("failed","archived")`(workflow_read_model:1377)——用它;
  4. **重试 = 接现成规则**:`workflow_transition_allowed`(read_model:1398·`failed→running` 需 `explicit_retry_or_reopen`)+ `workflow_node_transition_allowed`(1407·`project_director` 门)**声明了没接线**——C4c 把它接进真实动作。
- **人闸铁律**:重试/换会话会**重新真跑 codex**——必须走**现有授权/人闸**(不新开免用户路径·定稿铁律)。

## 1. 拍板摘要

- **做什么**:failed 节点从「死路一条(只结束)」→ 暴露**四个可选处置**(重试/退回/换会话/结束),每个**接现成机器**(退回=C4a·换会话=C1·结束=转移表·重试=接规则),迁移经现成 `workflow_(node_)transition_allowed` 校验、`project_director` 门。
- **谁选**:主管**可给建议**(advisory·可选·延续两钩点"意见不是闸");**真执行重试/换会话(重跑 codex)走现有人闸**——不自动连环。
- **不做**:新造 rework/建会话/转移机器(全复用)/ C5 词表 / 审查智能体。

## 一句话判据

**「是不是只:failed 节点接四个处置动作(退回→复用 C4a rework·换会话→复用 C1 create_and_bind·结束→现成 archived·重试→接现成 failed→running 规则)+ 经现成 transition_allowed 校验+project_director 门——而人闸/授权/沙箱 0-diff、重跑类走现有授权不自动、不新造任何第二套机器?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 failed 节点四处置(接线·复用)

- failed 节点/链暴露四个动作(命令或既有交办面动作·按现状 UI 机器就近挂,别新造面):
  - **重试**:`failed→running`·置 `explicit_retry_or_reopen=true`·经 `workflow_node_transition_allowed`(project_director 门)校验通过→复用现成派发把该任务重跑(**走现有授权/人闸**);
  - **退回**:置 `needs_rework`·**复用 C4a** `reset_work_item_for_director_rework`+预算(`director_rework_attempts`·耗尽不无限)→ 反馈边回 worker;
  - **换会话**:**复用 C1** `create_and_bind_task_session` 给该任务建新会话→绑→重跑(**走现有授权/人闸**·cwd 仍测试项目);
  - **结束**:`failed→archived`(现成转移表)·链终态。
- **迁移合法性**:全经现成 `workflow_transition_allowed`/`workflow_node_transition_allowed`(project_director 门)——**别绕校验自己改状态**;失败态→四态的合法出边补进允许表(现状 failed 只有 archived 出边·加 running[需 explicit]/needs_rework)。

### 2.2 主管建议(可选·advisory·cheap)

- 主管**可**对 failed 出一句建议动作(如"看着像供给类失败·建议重试"·advisory·不拦);**成本**:失败才可能调·或纯确定性提示(如供给类失败→建议重试·断供→建议结束)——**别每 failed 都烧 LM**;可先做确定性建议、LM 留后。

### 2.3 明确不做

新造 rework/建会话/转移机器(§0 全复用)/ 自动连环(重试/换会话走人闸·不自动接力)/ C5 词表对齐 / failed 之外的态。

## 3. 安全死线

- **人闸/授权铁律**:重试/换会话重跑 codex → **走现有授权+人闸·不新开免用户路径·不自动连环**(定稿+高危#4);
- **复用不重造**:退回用 C4a 的、换会话用 C1 的、转移用现成校验——**新造第二套=红线**(防重造);
- 沙箱/path-lock/prepare needs_binding/execute/worker_report/c4_c6 record — **0-diff**;换会话 cwd 写死测试项目;memories 观察模式不加旗。

## 4. 验收

- **单测**:①failed 节点四动作各自迁移合法(重试→running 需 explicit+project_director·退回→needs_rework·换会话→新会话绑定·结束→archived)·非 project_director 拒;②退回**复用 C4a** 预算(stub 证调的是 reset_work_item_for_director_rework·非新造)·耗尽不无限;③换会话**复用 C1** create_and_bind(stub 证);④非法迁移(如 failed→completed)仍拒;
- **人闸**:重试/换会话真跑路径经现有授权校验(require_active_authorization 类)·无免用户旁路(测证);
- **回归**:C4a/C4b 的 746 全绿(证没碰终标/总结);现有 failed 测按新增出边调整处说明;
- 三闸绿 + 死线 0-diff + 复用自证(git grep 证没新造 rework/建会话 fn)+ 计数不降 + fmt `cargo fmt --check`。

## 5. 回交

- §4 证据(尤其「退回/换会话复用现成·没新造」+「重跑走人闸」)+ 死线 0-diff + 复用自证 → 主导线核实物(**重点核:没重造机器、人闸没绕、迁移经校验**）。**子线不 commit。**

## 7. 不接受为

- 新造第二套 rework/建会话/转移机器(复用 C4a/C1/现成·防重造红线)/ 重试或换会话绕人闸/开免用户自动连环(高危#4)/ 绕 transition_allowed 自己改状态 / 碰沙箱/授权/execute/worker_report/c4_c6 record / 每 failed 都烧 LM(建议先确定性)/ 提前做 C5 / 自报 fmt 或 ad-hoc rustfmt。

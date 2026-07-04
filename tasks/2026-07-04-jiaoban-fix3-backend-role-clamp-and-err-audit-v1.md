# 实现任务包:交办 fix3 后端(角色钳位 + 失败留档 + 接续告知)· 主导线 → 执行线 v1

日期:2026-07-04　性质:**轻档**(装配/审计加缝;闸/判决体/controller 本体不碰)。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**(后端)。**子线不 `git add`/`commit`。** 全程中文。
- **背景 = 用户真机复验的实况(主导线已读线上 store 定根因,别再猜)**:
  1. 审计实锤(07-03):任务被拦 `blocked_reasons: ["目标角色不在授权范围内"]`——**主管 LM 给任务自由编角色**(如审查类),而授权 `allowed_role_ids = ["codex-dev","project_director"]`(刀1 档位),编出界外名字 → guard 拦 → stage=blocked。方案开了 `suggest_workflow` 后拆得更多步,**中招率更高**,是"基本上都卡住"的最大一类。
  2. 今晚(07-04 23:22)审计只有 `role_loop_auto_advance_started`,**之后一片空白**——合流/auto_advance 的**失败走 Err 早返回,不写审计**;原因只活在前端内存,重开 app 即失踪。
  3. 07-03 有条链记录**至今挂在 running**(第二个任务 dispatch_started 后进程没了)。已核:`ensure_chain_run_record` 断点续会复用它、卡 running 的任务**会被重派**(只有 completed 才跳)——**不挡路,但用户不知情**。
- **先读**:`director_agent.rs`(`director_task_scope_from_proposal` ~102-127 / `parse_director_plan` / 预拆命令的 warnings 通道 / `run_confirm_and_start_authorized_run_inner` 与 `run_auto_advance_authorized_role_loop` 的各 Err 返回点 / `run_director_task_chain` 起链处)。
- **一句话**:LM 编界外角色 → 自动钳成 codex-dev + 记警告;确认之后的失败一律写审计再返回;接续未收尾旧链时给用户一句告知。

## 1. 拍板摘要

- **要做的事**:把"莫名被拦"最大类(角色漂移)从源头归一化;失败原因永久留档;接续旧链可见化。
- **为什么**:用户真机复验"基本上都卡住、有反馈不知道怎么办"——三处全是可修的确定性根因。
- **代价**:一轮·后端·集中 `director_agent.rs`(+ lib.rs 测试)。

## 一句话判据

判改动在不在本包——问:**「是不是只做了『拆解产物的角色归一到已授权集合』『确认后失败路径补审计』『接续旧链加警告』,而 guard/控制核心/controller 本体/人闸/档位全 0-diff?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 角色钳位(核心)
- `director_task_scope_from_proposal`(或其调用处):LM 给的 `target_role` **不在 `proposal.scope_draft.allowed_role_ids` 里** → 钳成 `"codex-dev"`,并把一条警告带出:「任务『X』的角色『Y』不在授权名单,已按 codex-dev 执行」。空 → codex-dev(现状保留)。
- **两条产出路都要钳**:`plan`(auto_advance/合流用)与 `plan_preview`(刀2 预拆)——预拆给用户看的图必须已是钳后的(所见即所跑)。警告走预拆已有的 warnings 通道 / auto_advance 的 outcome warnings。
- **辨析(写给复核者)**:这不是放权——codex-dev 本就在授权名单;是把 LM 的自由文本归一到**已授权集合内**,scope 其余字段(读写/工具/检查)不动。`validate_approved_planned_tasks` **不钳**(回传数据不静默改;界外照 guard 拦,安全兜底不变)。

### 2.2 确认后失败留档
- `run_confirm_and_start_authorized_run_inner`:**record_decision(确认)成功之后**的任何失败(边界复核/授权/拆任务[含 retry 后仍败]/prepare/起链的 Err)→ 先 append 一条审计(`role_loop_auto_advance_stopped`,payload 带失败阶段+错误文本人话)再返回 Err。
- `run_auto_advance_authorized_role_loop` 同待遇(它的早期"无 active 授权"参数类拒可不记;**拆任务失败起必须记**)。
- 复用现成 audit append 帮手;**只 append、不改任何既有状态字段**。

### 2.3 接续告知
- `run_director_task_chain` 起链前(**director_agent 侧、只读检测**):该 workflow 已有 `state ∈ {running,stopped}` 的链记录 → outcome/chain warnings 加一条「已接续上次中断的运行,中断处的任务会重跑」。**`workflow_chain_controller.rs` 本体 0-diff**(检测放调用方,别改 ensure_chain_run_record)。

## 3. 安全死线

- **0-diff**:`control_core`(guard)/ `c4_c6` 判决体 / `workflow_chain_controller.rs` 本体 / 档位函数 `profile_edit_test_project_scope` / 人闸校验(PendingUserConfirmation)/ 执行闸/沙箱/path-lock 全套。
- 钳位**只收不放**:目标集合 = 已授权 `allowed_role_ids` 内的 codex-dev;绝不把界外角色加进授权、绝不动 write/tools。
- 审计**只 append**;失败仍失败(Err 语义不变,别吞错)。

## 4. 验收(执行线自己验)

- **单测·钳位**:stub 主管吐 target_role="reviewer" → scope.target_role=="codex-dev" + 警告在;plan 与 plan_preview 两路都断言;"codex-dev"/"project_director" 原样不动;`validate_approved_planned_tasks` 对界外角色**不改**(guard 兜底路径照 blocked——现有测试口径别破)。
- **单测·留档**:注入拆步两次假早退(retry 后仍败)→ 返回 Err 且审计里有 stopped 事件带阶段+人话;确认前参数拒 → 不多写。
- **单测·接续**:预置一条 running 旧链记录 → 再跑 → warnings 含「已接续」+ 卡 running 任务被重派(现行为断言住,防回归)。
- **regression**:`cargo test --lib` 计数不降;§3 全部 0-diff 扫 diff 自证;fmt(只本包文件·skip_children);真跑不强制(改动无新执行路径),若跑则照核实物纪律。

## 5. 本包不做(deferred)

- UI(接着跑按钮/停因动作映射)= 姊妹包 `2026-07-04-jiaoban-fix3-ui-actionable-stops-v1.md`。
- 方案 a(新会话)下一阶段;刀2-UI;LM 多角色真支持(reviewer 真派发)= 未来角色扩展,不在本包。

## 6. 回交

- 跑 §4;回交列:钳位落点(两路)+ 警告样例、留档的失败点清单(逐个 Err 返回点标记了/没标记哪些+为什么)、接续告知落点、0-diff 自证、计数 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 不接受为:把界外角色加进授权名单 / 钳了回传数据 / 改了 controller 本体或 guard / 吞错(失败变静默成功)/ 审计写了非 append 的状态改动。

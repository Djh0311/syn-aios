# 实现任务包:needs_rework 处置面(四选一放宽 + [接着跑]上交办主界面)· 主导线 → 执行线 v1(可派)

日期:2026-07-10 · 性质:**轻档**(处置机器放宽一态 + 前端呈现;人闸/冻结核 0-diff)。来源:B2 真机验收 bug④(`CURRENT.md` §三.1)。

## 00. 主导线勘察记录(派前核实物)

- **案发**:任务 1 被主管终标 `needs_rework`(打回·「已复位为可重做」)后,**主界面零处置按钮**;用户在**历史左栏**卡住单行内才找到 [接着跑] 点了复跑。
- **后端硬闸**:`director_agent.rs:993`「failed 四选一只能处置 failed 节点」——`apply_project_director_failed_action`(request :820·action = `"retry"|"rework"|"change_session"|"archive"`)只收 failed。
- **四选一机器现状**(retry/change_session 分支 :999-1035):transition 检查(`workflow_transition_allowed`/`ensure_failed_node_transition`)+ `require_test_project_path_lock` + **`require_active_authorization`(重跑走人闸·保留)** + change_session 走 C1 `create_and_bind_task_session`(:1032·不许绕)。
- **复位机制现成且幂等取向**:`:1272-1293` 注释——重试前把 work_item 走**现成合法跳转**逐步 fire(running→failed→needs_changes→ready_to_dispatch·**非法跳转各步自忽略**·以末步到位为准)。**needs_rework 终标已把 work_item 复位到 ready_to_dispatch**(:1148「主管显式选择 rework:复用 C4a reset」同源)——放宽后 retry 的复位调用预期幂等直过,**执行线必须亲核**这条(防双重复位/非法跳转)。
- **[接着跑] 现有实现**:前端 `continueRun()`(`ProjectJiaobanPanel.tsx:652`·方案已确认时直调 auto_advance 不走合流命令·:657 明确「用现有链不新建会话」);挂点 :875/:980(onContinueRun)+ 历史左栏行内。**needs_rework 停链后主界面为何没露它 = 执行线先核**:chainStatus→五态脸分类对「needs_rework 停链」落到哪个脸、该脸为何没接处置按钮组。
- **词表**:`humanizeVerdict` 已有 needs_rework=「要返工」;死配对纪律 = 绝不零按钮(classifyBlocked 同哲学)。

## 0. 接手须知(自包含)

- 执行线(后端放宽 + 前端呈现)。**子线不 commit。** 全程中文。
- 与包 1(复核输入修正)可能并行在树——**别碰 `global_supervisor_agent.rs`**,撞了停手报回。
- 硬语义:**重跑走人闸不自动连环**(B2 原纪律)——放宽只是让 needs_rework 可被**用户**处置,不是自动重跑。

## 1. 拍板摘要

- **做什么**:① 四选一机器放宽收 `needs_rework` 节点(与 failed 同一套四动作);② needs_rework 停链的**交办主界面脸**给处置按钮组(人话四选:[接着跑(按原样重做)] [换个新会话重做] [退回主管重拆] [结束这单]),[接着跑] 不再只藏历史左栏。
- **为什么**:主管打回是每单都会发生的常态路径,现状用户面对退回无处可点(bug④·真机实撞)。
- **不做**:不自动重跑;不改终标判定;不改 failed 原路径行为;不动人闸。

## 一句话判据

**「是不是只:放宽处置机器多收一个 needs_rework 态(四动作语义与 failed 对齐·复位幂等)+ 主界面补处置按钮组——而终标/链驱动/人闸(`require_active_authorization` 照旧)/C1 会话机制/冻结核 0-diff?」** 是 → 做;否 → 停。

## 2. 建什么

### 2.1 后端:四选一放宽收 needs_rework

- `:993` 判改为 `failed | needs_rework` 白名单(其余态照拒·错误话术同步:「只能处置 failed / needs_rework 节点」);
- transition 检查:needs_rework→running(retry/change_session)、needs_rework→(rework/archive 目标态)进白名单——**用现成 `workflow_transition_allowed`/`ensure_failed_node_transition` 的扩展点**,别旁路新写一套;函数名带 failed 的,若只差命名可加别名/参数,不复制粘贴第二套;
- **复位幂等亲核**:needs_rework 已被终标复位(work_item=ready_to_dispatch)→ retry 分支的逐步 fire 复位(:1272)对已复位者应「各步自忽略·末步已到位」直过——写一条单测钉死(已复位+retry=不炸不重复位);
- 四动作对 needs_rework 的语义(与 failed 完全同构):retry=原会话重跑 / change_session=C1 新会话重跑(`create_and_bind_task_session` 复用·不许绕) / rework=打回主管重拆 / archive=结束归档。

### 2.2 前端:主界面处置按钮组

- 先核 needs_rework 停链在五态脸的落点(00 节遗留问题),把处置按钮组接到**该主界面脸**上:显示终标退回理由(人话·现有字段)+ 四按钮(词表:接着跑(按原样重做) / 换个新会话重做 / 退回主管重拆 / 结束这单);
- [接着跑(按原样重做)] 可复用现有 `continueRun()` 语义(方案已确认·auto_advance 直推·它会捡起 ready_to_dispatch);其余三个走 2.1 放宽后的四选一命令;
- 历史左栏行内 [接着跑] **保留**(它是旧单浏览的入口·不删);主界面是主入口;
- 死配对纪律:绝不零按钮;按钮说人话不露 `planned_task_id` 等黑话(词表死线)。

## 3. 安全死线

- **0-diff**:人闸(`require_active_authorization` 一字不动)/ 终标机器(七查/判定)/ 链自动驱动(不加任何自动重跑)/ C1 `create_and_bind_task_session` 本体 / `global_supervisor_agent.rs`(包 1 地盘)/ runner/沙箱/冻结核;
- path-lock 检查照旧(`require_test_project_path_lock`);
- 放宽仅此一态:**不许**顺手把其它态(completed/superseded/…)放进处置机器。

## 4. 验收

- 单测:needs_rework×四动作各一条(retry 幂等复位 / change_session 走 C1 / rework / archive)+ failed 原路径回归不变 + 其它态仍拒;
- 前端:离线 DOM——needs_rework 停链主界面脸出四按钮+退回理由人话;真机一眼(用户下一单自然验);
- 三闸绿 + `cargo test --lib` 计数不降(基线 764/0/43+包1增量)+ 权威 `cargo fmt --check` 新改行净;
- 0-diff 自证:diff 不含 §3 清单文件。

## 5. 回交

改动清单 + §4 证据 + needs_rework 停链脸落点勘察结论(00 遗留)→ 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 自动重跑(处置必须用户点) / 绕 `require_active_authorization` / 绕 C1 建会话 / 复制第二套 transition 检查 / 动终标判定 / 碰包 1 地盘 global_supervisor_agent.rs / 删历史左栏入口 / 按钮露黑话。

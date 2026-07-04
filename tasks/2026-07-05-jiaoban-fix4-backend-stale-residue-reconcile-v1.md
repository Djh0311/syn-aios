# 实现任务包:交办 fix4 后端(残料接管:上一轮遗留工作项合法复位 + 重拆即新链)· 主导线 → 执行线 v1

日期:2026-07-05　性质:**轻档**(编排侧状态机合法调用;C4 保护规则/controller 本体/闸 全 0-diff)。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**(后端)。**子线不 `git add`/`commit`。** 全程中文。
- **背景 = 用户真机走[接着跑]撞死(主导线已读线上 store + 代码钉死根因,别再猜)**:
  - 报错:`已存在 work item 且状态为 ready_for_review,C4 拒绝覆盖:…planned-task-…-default-1`。
  - 盘上实况:07-03 那轮跑挂,遗留 `…default-1` 工作项 **ready_for_review**(活完成、审查没记、进程死)、`…default-2` **running**(派发后进程死)、链记录挂 running;
  - 机制:planned_task_id 是**工作流+序号**定址(`planned-task:{wf}:{n}`)→ 任何新一轮拆任务撞同名旧工作项;C4 的保护(`c4_c6:2277-2283`:existing ∈ {running, ready_for_review, accepted, failed, timed_out, cancelled} → 拒)是**对的、不改**;
  - 后果:**该工作流上一切新方案/重跑全被卡死**,不修不能走。
- **手里已有的合法工具(复用,别造新的)**:
  - `director_agent.rs::reset_work_item_for_retry`(~275):合法状态机行走器 `running→failed→needs_changes→ready_to_dispatch`(非法步自动忽略);
  - `update_work_item_state_at`:合法跳转的执行器(先去读它的**合法迁移表**,确认 `ready_for_review→needs_changes` 是否合法——按产品语义"审查打回"应合法;若不合法,回主导线,别硬写状态);
  - controller 的 `finalize_chain_run`(链收尾 helper·刀1 起就在调)。
- **先读**:上面三处 + `run_auto_advance_authorized_role_loop`(re-plan 的 None 分支与 approved 的 Some 分支·prepare 调用点)+ `run_confirm_and_start_authorized_run_inner` + `ensure_chain_run_record` 的续跑判据(workflow_id+project_id+state∈{running,stopped})。
- **一句话**:prepare 之前,把**本轮 planned_task_ids 对应的**遗留工作项走**合法路径**复位到可重派;重拆(re-plan)路径把未收尾旧链记录**正式标结(superseded)后起新记录**,不再跨轮乱续;所批即所跑(approved/C1)路径的续跑语义**不动**。

## 1. 拍板摘要

- **要做的事**:让"重跑/新方案"不再被上一轮残料卡死——残料被**合法接管**(每步留审计),旧链正式收尾,新一轮干干净净;按钮上"从拆任务重来"的承诺变成真的。
- **为什么**:用户真机被完全卡死;这是"跑挂了再来"这一日常路径的必修,不修交办没法用。
- **代价**:一轮·后端·集中 `director_agent.rs`(+ lib.rs 测试)。

## 一句话判据

判改动在不在本包——问:**「是不是只在编排侧(director_agent)prepare 前用合法状态跳转复位本轮任务的遗留工作项、并在 re-plan 路径把旧链标结后起新链,而 C4 保护规则/controller 本体/guard/人闸/档位/path-lock 全 0-diff、没有任何直接改 state 字段的写法?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 残料接管(pre-prepare reconcile·核心)
- 新帮手(如 `reconcile_stale_work_items_for_plan(path, project_root, planned_tasks) -> Vec<String>`),在 **auto_advance 的两条分支(re-plan 与 approved)** prepare 之前调:
  - 只扫**本轮 planned_task_ids 派生的 work_item_id**(定址规则照 c4 现有推导;**别扫别人**——canvas-run 等残料不碰);
  - 已存在且 state ∈ **{running, ready_for_review, failed, timed_out}** → 走合法行走器复位到 `ready_to_dispatch`:
    - 扩展 `reset_work_item_for_retry` 的行走序(或加姊妹函数):`running→failed`(已 failed 忽略)→ `ready_for_review→needs_changes`(审查打回语义)→ `failed/timed_out→needs_changes` → `needs_changes→ready_to_dispatch`。**每步只经 `update_work_item_state_at`(合法迁移·自带审计),严禁直接改 JSON state 字段**;
    - **先核实合法迁移表**:若 `ready_for_review→needs_changes` 不在表里 → 停、回主导线(别自作主张加迁移——那是改状态机本体)。
  - state ∈ **{accepted, cancelled}** → **不接管**(终态/人工态·罕见):留给 C4 照拒,报错文案已可行动(fix3-UI 配对表兜底)。记 warning 说明即可。
  - 复位发生过 → outcome warnings 加一条人话:「已接管上一轮遗留的 N 个工作项(打回重派)」。
### 2.2 重拆即新链(fresh-run 语义·只动 re-plan 路径)
- **re-plan(None 分支)**:起链前若该 workflow 有未收尾链记录(state ∈ {running,stopped}·判据镜像 ensure 的)→ 用现成 `finalize_chain_run` 把它标结(failed/superseded 语义、message 写「被新一轮重拆取代」)→ 之后 ensure 走"新建记录"分支。fix3 的「已接续」warning 在此路径改为「上一轮未收尾的运行已标结,本轮从头重跑」(别再说"接续"——re-plan 语义就是重来)。
- **approved / C1(Some 分支与 start_project_director_chain)**:**续跑语义不动**(同计划中途停→续是既有已验特性),不 finalize、不动 warning。
### 2.3 测试夹具贴线上形状
- stub 测试要造出线上同款残料(ready_for_review + running + 挂 running 的链记录),别只造理想态。

## 3. 安全死线(0-diff / 不碰 / 不绕)

- **C4 保护规则原样**:`c4_c6` 整文件 **0-diff**(2277-2283 那段一个字不动——接管发生在它之前、用合法跳转让残料离开被保护状态);
- **controller 本体 0-diff**(`finalize_chain_run`/`ensure_chain_run_record` 只调不改);
- **状态机本体 0-diff**(`update_work_item_state_at` 的合法迁移表**不加不改**;走不通=回主导线);
- 人闸 / 档位 / path-lock / guard / 沙箱 / 执行闸——增删 0 命中;
- 接管**只针对本轮 planned ids**(不许全库扫荡清理);accepted/cancelled 不碰。

## 4. 验收(执行线自己验)

- **单测·接管**:预置 ready_for_review 残料(线上同款 id 形状)→ re-plan 跑 → prepare 过、任务重派、warnings 含「已接管」;预置 running 残料 → 同上;预置 accepted → 照拒(现有报错·断言不被接管);**canvas-run 形状的 id 不被碰**(断言原样)。
- **单测·新链**:预置挂 running 旧链 → re-plan → 旧记录被标结(state 非 running/stopped、message 在)+ 新记录新 id + 不再出现「已接续」字样;**approved/C1 路径**:同计划中途停→再跑 = 仍续(completed 跳过·记录复用)——现有测试口径别破。
- **单测·合法性**:接管全程无直接 JSON state 写(代码审读自证)+ 每步经 update_work_item_state_at(审计事件在)。
- **真跑**(`#[ignore]`·可选但建议·temp 夹具复刻线上残料形状):残料在 → 合流/接着跑 → proof 出、warnings 对。
- **regression**:`cargo test --lib` 计数不降;§3 全 0-diff 扫 diff 自证;fmt(只本包文件·skip_children)。

## 5. 本包不做(deferred)

- canvas-run 那 ~20 条历史残料的清理(不同 id 空间·不挡交办·另议卫生包);
- run-scoped 任务编址(治本改造·牵 chain 定址/source_ref 配对/C1,Phase A 不动);
- accepted/cancelled 的接管语义(真撞到再议);UI(fix3-UI 的按钮/警告显示已够)。

## 6. 回交

- 跑 §4;回交列:接管函数与调用点(两分支)、行走器扩展(合法迁移表核实结论)、finalize 落点、warnings 样例、0-diff 自证、计数 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 不接受为:改了 C4 保护/状态机迁移表/controller / 直接改 state 字段 / 全库扫荡清理 / 动了 approved·C1 的续跑语义 / 接管了 accepted·cancelled / 把「已接续」和「重来」两种语义混着报。

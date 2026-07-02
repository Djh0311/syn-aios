# 实现任务包:交办·刀2 后端(批前预拆 + 所批即所跑 + 任务级节点/依赖边落画布 + 拆步 retry)· 主导线 → 执行线 v1

日期:2026-07-02　性质:**轻档**(只读预拆命令 + 数据装配/落盘;**prepare 判决体/闸逻辑不碰**,见 §3 精确圈界)。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**(后端·新开干净上下文)。**子线不 `git add`/`commit`。** 全程中文。
- **决策正本**:`decisions/2026-07-02-project-jiaoban-tab-final-design-v1.md` §条件式工序图 + §地基 4/5——**已拍**:批前预拆(只读·人闸仍锁执行)/ 所批即所跑(批的图原样执行不重拆)/ 图落成真节点进「工作流」tab / 任务节点不放运行按钮。
- **可行性主导线已核过实物(要点直接给你,省你重查)**:
  - `CliDirectorAgent::plan` 只吃 ctx+proposal、**自己不查授权状态**、结构性只读(readonly_codex_consult);对 pending 方案跑它机制上零障碍,缺的只是一个薄命令。方案的 `scope_draft` 在确认前就存在(pending 时即有),拆解的钳制数据源在。
  - `start_project_director_chain`(C1)**本来就收前端回传 planned_tasks、不重跑 LM**;`prepare_authorized_auto_dispatch` 的 planned_tasks 也是调用方传入。「所批即所跑」的执行侧现成,缺的是"把预拆结果带进合流"。
  - 节点/边写入通道现成:node/edge 是无 schema 约束 JSON、`node_id` 自由字符串(按 `(workflow_id,node_id)` 查重);read model `derive_workflow_nodes` 对新节点原样透传、`depends_on` 会从 edges 按 `to_node_id` 反推带出。反解站点对任务级 id 兼容(前缀仍是 `{wf}`)。
  - **坑(逐条防)**:① `{wf}:node:task` 是 bootstrap 保留节点(精确匹配),任务级 id 必须带后缀 `{wf}:node:task:{n}`;② planned task 的 `workflow_node_id` 字段语义**不能改**(仍指 role 节点·有测试断言·binding 靠它)——任务级节点是**加法**,配对存 node 的 `source_ref`=planned_task_id;③ 写节点 position 要错开(现硬编码 x:560,y:120 会全叠);④ `depends_on` 是按 **title** 连的,写边前做 title→任务节点 id 映射,悬空依赖照现有先例记 warning 不建边;⑤ 主管 LM 偶发吐残图(重复 title 现有逻辑拒起链、悬空 warning)——预拆返回时把 warnings 一并给前端。
- **今晚真机实况(动机)**:用户批到旧方案撞停,UI 丢消息(UI 线在修·fix2);同时暴露 **director 拆任务那步没 retry**(刀1 的 retry 只包 worker 链;拆步 flaky 早退要手动重跑——记忆 `jiaoban-retry-gap-director-consult-step`,本包 §2.4 补)。
- **先读**:`director_agent.rs`(plan/auto_advance 内层/合流 inner/C1)、`consultant_agent.rs`(ConsultationProposal/parse/prompt——刀1 重塑后的现状)、`c4_c6_workflow_governance_entrypoints.rs` 的 `ensure_project_director_worker_node`(~2099)与 prepare 主循环(节点写入的现成通道)、`workflow_read_model_entrypoints.rs:739-746`(edges→depends_on 反推)。
- **一句话**:加一个只读「预拆」命令(pending 方案 → 主管 LM 拆图返回·零写盘);合流命令可收「用户批过的那份图」原样执行(不重拆);prepare 时**加法**写任务级节点+依赖边(老 role 节点/绑定原样);director 拆步补 retry 一次。

## 1. 拍板摘要

- **要做的事**:复杂活批前能看图、批的图就是跑的图、跑的图真实出现在画布上。
- **为什么**:决策已拍;这补的是人闸的真窟窿——现在用户批的是文字,真跑的图是批后才拆、没人见过,且每次重拆可能不同。
- **代价**:一轮·后端。1 个新只读命令 + 合流收图参数 + prepare 加法写节点/边 + 咨询加一个"建议按工作流"轻标记 + 拆步 retry。

## 一句话判据

判改动在不在本包——问:**「是不是只加了『只读预拆』『批过的图带进合流原样跑』『prepare 加法写任务级节点+依赖边』『拆步 retry 一次』,而 prepare 的 guard/判决路径字节级不变、无新执行口、预拆零写盘、任务级节点纯加法不改老编址?」** 是 → 做;否 → **停、回主导线。**

## 2. 建什么

### 2.1 只读预拆命令(批前看图)
- 新命令(如 `preview_pending_proposal_director_plan`):输入 project_root + proposal_id → 从 proposal store 按 id 取方案(**接受 PendingUserConfirmation**)→ `load_project_context` → `CliDirectorAgent::plan`(全现成)→ **原样返回** `Vec<ProjectDirectorPlannedTask>` + warnings(悬空依赖等)。**零写盘、不 annotate、不 prepare、不碰链**。
- async + spawn_blocking(真 LM 420s 级·同 P2 范本);**别复用/别撞名** `preview_project_director_task_plan_for_index_at`(那是确定性单任务压缩拆·闸在 UserConfirmed,完全另一物)。
- prompt:`director_build_prompt` 现文案写死「已授权方案」——加 variant/参数,预拆时说「待确认方案·仅预览」(别对 LM 撒谎)。
- path-lock:**不加**(与 `run_project_consultation` 同款先例——只读咨询形,读非测试项目不碰高危#1,决策 `2026-06-25-consultant-readonly-guard-exemption`);但**预拆产物不得直通任何执行**(执行只能走 2.2 的人闸合流)。

### 2.2 所批即所跑(合流收图)
- `ConfirmAndStartAuthorizedRunRequest` 加**可选** `approved_planned_tasks: Option<Vec<ProjectDirectorPlannedTask>>`:
  - `Some` → 合流内层跳过 `director.plan`,直接用它进 prepare(prepare 对每个 task 照常过 guard——**只能钳/拒、不能扩**,越界照 blocked,安全不降);
  - `None` → 现状(批后 LM 拆·简单活路径不变)。
- 校验:传入 tasks 的 workflow_id/project 必须与方案一致;空数组拒。C1 已有的"重复 title 拒/悬空 warning"逻辑照常兜底。
- 这样「预拆给用户看的那份」=「批后真跑的那份」,重拆不确定性消除;UI(刀2-UI 包·另开)负责把预拆结果回传。

### 2.3 任务级节点 + 依赖边落画布(prepare 加法)
- prepare 主循环里,对**每个过了 guard 的** planned task,**额外**写一个任务级节点:
  - `node_id = {workflow_id}:node:task:{序号或 slug(planned_task_id)}`(**绝不能恰好等于** `{wf}:node:task`);`title` 用任务 title;`source_kind` 沿用 director 来源标记;`source_ref = planned_task_id`(配对锚·别动 planned task 的 `workflow_node_id` 语义);position 按 index 错开(比如 x 递增一格、y 分行)。
  - `node_exists` 查重幂等(重跑 prepare 只刷状态不重复建)。
- 同批把 `depends_on`(title 引用)翻成 edges 写入:title→任务节点 id 映射;`edge_type="depends_on"`;`edge_exists` 查重;悬空依赖记 warning 不建边。
- **老的 role 节点、work_item、binding、`workflow_node_id` 锚全部原样**——本节是纯加法。blocked 的任务**不建节点**(现行"没过 guard 不建 node"的边界语义不动)。
- 链跑时对任务级节点的状态回写:链驱动已按 planned_task_id 记链节点;**本包只需**在链每任务完成/失败处,顺带把对应任务级节点(按 source_ref 配对)state 刷成 completed/failed(加法小缝·别动链判决)。做不进就 deferred 声明,别硬塞。

### 2.4 拆步 retry(补今晚的缺口)
- `director.plan` 的调用处(auto_advance 内层 / 合流 inner / 2.1 预拆命令):失败特征 = tier-1 偶发早退(照刀1 `is_tier1_early_exit` 的思路对 consult 错误形状判,如 `consult_last_message_read_failed`/exit≠0 无输出)→ **原地重试一次**,warnings 记「主管拆任务已自动重试」;仍败照常报错。**gate/解析类错误不 retry。**

### 2.5 咨询轻标记(AI 判定"要不要按工作流")
- `ConsultationProposal` 加 `#[serde(default)] suggest_workflow: bool`(或 Option<bool>);prompt 让咨询判「这活是否值得拆成多步工作流」;缺省/老输出 → false(**向后兼容**)。map 透传进 proposal(找个不破 schema 的位置,如 proposed_steps 首行标记或新字段——执行线按 store schema 兼容性定,别破老数据反序列化)。UI 用它决定"图区要不要出现"(用户手动开关兜底·UI 包的事)。

## 3. 安全死线(0-diff / 不碰 / 不绕——本包最要紧的一节)

- **prepare 判决体不碰**:`c4_c6` 文件会 diff(2.3 加法),但 **guard 评估、authorized/blocked 分流、C4 校验、`project_director_authorization_context`、确定性兜底拆——这些函数字节级不变**;新代码只在节点物化区(`ensure_project_director_worker_node` 旁)加。回交时贴这些函数的 diff 为空的自证。
- **预拆零写盘、不通执行**:2.1 无任何 store 写、无 annotate、无 prepare;执行唯一入口仍是合流(人闸 PendingUserConfirmation 校验在)。
- **「没过 guard 不建 node」边界语义不动**(blocked 任务不上画布——要"全图含 blocked 预览"走 2.1 的返回值给前端画,不落盘)。
- **任务级节点无执行口**:不给它建 binding/work_item;它没绑定 → 现行 run readiness 天然不就绪。UI 侧不放运行按钮(UI 包);后端不为它开任何派发路径。
- 死线本体 0-diff:`decide_real_execution_command` / `command_plan_for` / `execute_project_workflow_node_at` / `workflow_chain_controller` 本体 / `readonly_codex_consult` / 两 store 校验。retry 是调用处加缝。
- 不放开非测试项目真执行(预拆只读可任意项目·**执行仍 path-lock**);不新增 auto-approve;方案授权人闸不省。

## 4. 验收(两条线自己验)

- **单测·预拆**:stub 主管 → pending 方案能拆、返回含 depends_on + warnings;**确认后的方案也能拆**(幂等·预览无状态);全程零写盘(store 文件 mtime/内容断言)。
- **单测·所批即所跑**:合流传 `approved_planned_tasks` → prepare/链用的就是它(objective 原样·不重跑 LM[stub 主管置炸弹断言没被调]);越界 task 照 blocked;workflow 不匹配拒。
- **单测·节点/边**:prepare 后任务级节点一任务一个、id 形状对、position 错开、edges 按 depends_on 建、悬空 warning 不建边、重跑幂等;**老 role 节点/binding/work_item 断言原样**(现有测试全绿);`{wf}:node:task` 保留节点不受扰(bootstrap 测试不炸)。
- **单测·retry**:注入拆步假早退 → 重试一次成功 + warning;两次败 → 报错;解析错不 retry。
- **单测·咨询标记**:带/不带 suggest_workflow 的输出都能解析(老样本 false)。
- **真跑**(`#[ignore]`·测试项目):预拆一份真方案出图(≥2 任务含依赖)→ 合流带图跑 → proof + 画布读模型里任务级节点/边在、链完成后节点状态刷了。核实物纪律(读 proof/state,别只信 exit)。
- **regression**:`cargo test --lib` 计数不降;§3 圈界函数 diff 为空自证;fmt(只本包文件);`git diff --check`。

## 5. 本包不做(deferred)

- UI 全部(卡上出图/图后浮现/去工作流看/任务节点挡运行按钮/suggest_workflow 开关)= **刀2-UI 包**(fix2 落地后开,避免同文件撞车)。
- 画布可编辑工序图再跑(推后·决策已圈外);方案 a 新会话(下一阶段);blocked 任务上画布(只在预拆返回值里给前端画)。
- 停因/图跨 app 重启的持久恢复。

## 6. 回交

- 跑 §4;回交列:新命令签名、合流收图参数与校验、节点/边写入点与配对方式(source_ref)、retry 落点、§3 圈界函数 diff-空自证、单测/真跑证据、计数 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 不接受为:预拆写了盘/接通了执行 / 合流不带图时行为变了 / guard·判决路径被改 / 任务级节点改了老编址或 `workflow_node_id` 语义 / blocked 任务落了盘 / retry 循环化 / 为"图好看"碰 binding·work_item。
- 不接受为工序图整体完成(UI 呈现另包;本包只到「预拆可看、批的图原样跑、图和进度真实落在 workflow-state 里」)。

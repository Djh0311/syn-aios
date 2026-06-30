# 实现任务包：S2 前端第一薄切 · 主管链「按计划开干」按钮（C1 接上界面）· 主导线 → 执行线 v1

> 🅿️ **状态（2026-06-27·用户拍 B·挂起）**：前端**已实现·机器侧过**（typecheck 0 / `src-tauri` 0-diff / 按钮逻辑+「prepared 非 preview」双护栏 主导线核过），但 **真机 BLOCKED·未 commit**。
> **为什么 BLOCKED**：查 live app 状态（`~/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`）——测试项目**整套角色循环数据是空的**（0 个 consultation proposal / plan authorization / `authorized_prepared_auto_dispatch`；live app 还跑老 workflow-machine 模型，新角色循环从没在 app 里真跑过）。所以 `plan.prepared_dispatch_count==0`、**按钮永远 disabled、点不到**。
> **结论**：真正缺口不是这个按钮，是**上游角色循环 UI（咨询→方案→授权→边界复核→拆任务→准备派发）在 live app 里从未走通**。本刀**挂起**，其真机+commit **并进下一刀「上游角色循环 UI 走通」**一起做（这按钮是那刀的收尾）。
> **未提交工作树**：3 前端文件（`lib/tauri.ts` / `lib/types/workflow.ts` / `views/projects/ProjectWorkflowGovernancePanels.tsx`）+ 本任务包 untracked——别清、并进上游那刀。
> **追加（2026-06-27·真机暴露的上游 bug·同挂起包·typecheck0·真机待验）**：`ProjectWorkflowGovernancePanels.tsx` 里另修了 3 处角色循环 UI bug——① 第四步「生成拆任务草案」死按钮上方加可见原因（需先确认方案+边界复核）；② 方案 rejected/changes_requested 终态加「重新创建方案草案」出口（原来拒绝后死胡同）；③ needs_binding 提示改成可操作（指向执行面板「节点会话绑定」）。**这些是 blind 能改的上限**：真"绑会话→prepared 出来"+ 散面板观感 仍需真机。

日期：2026-06-27　性质：**轻档（纯前端接线）**；但**按钮一点 = 测试项目真起 codex 链**（高危#1+#4 轻档）——真机点那一下用户在场。

## 0. 接手须知（冷启即读，本包自包含）

- 你是**执行线**（新开、干净上下文）。**子线不 `git add`/`commit`。** 全程中文。
- **背景**：主管链后端 **C1 已建并 committed（`9ebd090`）**——`start_project_director_chain`（async·收前端回传已审 `planned_tasks`·spawn_blocking 调 `run_director_task_chain`），停/进度复用现成 `stop_project_workflow_chain`/`get_project_workflow_chain_status`。**但前端零接**：没 wrapper、没按钮。本包**只做前端接线**：把 C1 接成执行流里的「按计划开干（整条主管链）」按钮。**后端一行不改（C1 已完）。**
- **核过的集成点（直接用）**：
  - wrapper 模板：`src/lib/tauri.ts:797` 的 `startProjectWorkflowChain`（节点链·照抄改名）；停/进度 `stopProjectWorkflowChain`/`getProjectWorkflowChainStatus` **已存在、主管链复用同一种链记录、不用新写**。
  - 类型：`src/lib/types.ts`（`ProjectWorkflowChainRunResult/Status` 在此）。
  - 已审计划来源：`prepareAuthorizedAutoDispatch`（`tauri.ts:299`）返回 `AuthorizedPreparedDispatchResult`，调用在 **`App.tsx:445`**；`plan.planned_tasks` 在 **`ProjectWorkflowGovernancePanels.tsx:84/192`** 已握/已展示。
  - 进度显示范本：`src/views/WorkflowCommandConsoleView.tsx`（轮询 `getProjectWorkflowChainStatus`、显示 `chainStatus.state` + nodes·line 72/201-204）。
- **⚠️ 关键正确性点**：按钮传给 C1 的必须是 **prepared 那份 `planned_tasks`（`status=="prepared"`，来自 `prepareAuthorizedAutoDispatch` 的返回）**——**不是** preview 那份。传 preview（无 status）→ C1 的 B1 filter 把全部任务当非授权 skip 掉、跑个空链。**这条钉死。**
- **一句话**：加 `startProjectDirectorChain` wrapper + 类型 → 在已有 prepared 计划的执行流里加「按计划开干」按钮（传 prepared `planned_tasks`）→ 进度/停复用现成 → 真机点一次看主管链真跑/真停。

## 1. 拍板摘要

- **要做的事**：把刚做完的主管链从"命令层能跑"接成"界面里能点"。**第一次让用户点一下、看 LM 计划真驱动 worker 链在 app 里跑起来 + 能停。**
- **代价**：一轮·**纯前端**·几乎全是复用（wrapper 照抄、停/进度现成、进度显示有范本）。不碰后端、不碰闸。
- **关键澄清**：前端**只造请求 + 发**，闸在后端（C1 入口 `require_test_project_path_lock`）——非测试项目前端造不了钥匙、按钮单独开不了闸（同节点链发令台口径）。圈固定测试项目；**不放开**非测试/多项目。

## 一句话判据

判改动在不在本包——问：**「是不是只在前端加 C1 的 wrapper/类型 + 一个传 prepared planned_tasks 的『按计划开干』按钮 + 复用现成停/进度，没碰后端/闸、没传 preview 计划、没放开非测试?」** 是 → 做；否（改后端/闸、传错计划、自写停/进度命令、放开非测试）→ **停、回主导线。**

## 2. 建什么（纯前端）

1. **`src/lib/types.ts`**：加 3 个类型，**逐字对 Rust**（`director_agent.rs:599` 的 `StartProjectDirectorChainRequest` + `DirectorChainOutcome`/`DirectorChainStep`）：
   ```ts
   export type StartProjectDirectorChainRequest = {
     project_root: string;
     workflow_id: string;
     planned_tasks: ProjectDirectorPlannedTask[]; // 复用现有 ProjectDirectorTaskPlan 里的任务类型
     max_nodes?: number;
   };
   export type DirectorChainStep = { planned_task_id: string; title: string; state: string };
   export type DirectorChainOutcome = {
     total: number; dispatched: number; completed: number; skipped: number;
     chain_run_id: string; steps: DirectorChainStep[];
     warnings: string[]; stopped_reason: string | null;
   };
   ```
2. **`src/lib/tauri.ts`**：加 wrapper（照 `:797` 的 `startProjectWorkflowChain`）：
   ```ts
   export function startProjectDirectorChain(
     request: StartProjectDirectorChainRequest,
   ): Promise<DirectorChainOutcome> {
     ensureTauriRuntime();
     return invoke<DirectorChainOutcome>("start_project_director_chain", { request });
   }
   ```
3. **按钮 + 进度/停**（落点 = 已握 prepared 计划的执行流；优先 `ProjectWorkflowGovernancePanels.tsx`／或其上游 `App.tsx:445` 的 prepare 结果所在处）：
   - 「**按计划开干（整条主管链）**」按钮：`onClick` → `startProjectDirectorChain({ project_root, workflow_id, planned_tasks: <prepared 那份>, max_nodes })`。
   - **进度**：复用 `getProjectWorkflowChainStatus(projectRoot, workflowId)` 轮询（主管链建同种链记录、读得到），显示 `state` + 每步（范本 `WorkflowCommandConsoleView`）。
   - **停**：复用 `stopProjectWorkflowChain({ project_root, workflow_id })` —— 一个「停链」按钮。
   - 按钮区**写清「在固定测试项目真起 codex 链」**（这是真执行、不是预览）。
4. **不新写**停/进度命令或 wrapper（已现成）。

## 3. 安全死线

- **圈固定测试项目**：真执行的闸在后端 C1 入口 `require_test_project_path_lock`（非测试 root 拒）。前端只造请求+发、**不判闸、不自建更松路径**。
- **传 prepared 计划**（§0 ⚠️）：按钮源数据 = `status=="prepared"` 那份，传错=空跑。
- **后端 0-diff**：本包**不碰 `src-tauri/` 任何文件**——C1/run_director_task_chain/闸/沙箱已完、已 commit。若发现要改后端才能接 → **停、回主导线**（多半是数据没接对、不是真要改后端）。
- **不放开**非测试/多项目/auto-approve。
- **真跑用户在场**：点按钮 = 真 codex 链（高危轻档），真机验那一下用户在。

## 4. 验收

- **机器**：`npm run typecheck`（或项目对应）0 错、offline/单测（若有前端测）不降、`build` 绿。**只动 `src/lib/types.ts` + `src/lib/tauri.ts` + 落点面板**（扫 diff 自证没碰 `src-tauri/`）。
- **真机（用户在场·固定测试项目）**：走到有 prepared 主管计划的执行流 → 点「按计划开干」→ 看主管链真跑（多 worker 真 codex 接连）+ 进度更新 + 点「停」能在边界停。**机器绿 ≠ 真机能用**（记忆：画布/UX 类必真机过；computer-use 抓不到 Tauri dev 二进制，这步用户做）。

## 5. 本包不做（deferred·显式）

- **执行面板整体 UX 重做 / "裸协议面板"人话化**：归 S2「可用性」单独一刀（方案授权 UI/收字段/编辑 UX）。本包只接通「能点能跑能停」，**不重做面板观感**——别膨胀。
- 别角色（秘书/全局主管）前端、NL 一句话启动、套模版。
- 非测试真实项目（高危#1）、多项目（高危#4）、改后端任何东西。

## 6. 回交

- 跑 §4 机器侧；回交：前端 diff（确认只 types/tauri/落点面板、`src-tauri/` 0 碰）+ typecheck/build 结果 + 截图或说明按钮接到哪 → 主导线核（扫 diff 碰没碰后端/闸）。**真机点按钮看主管链真跑/停 = 用户在场单独验**（执行线给步骤、用户做）。**子线不 commit。**

## 7. 不接受为

- 不接受为：改了 `src-tauri/`（后端 C1 已完、不该碰）/ 按钮传了 preview（非 prepared）计划 = 空跑 / 自写停链或进度命令（该复用）/ 前端绕过后端闸自判 / 放开非测试或多项目 / 把执行面板整体 UX 重做塞进来。
- 不接受为 S2 前端整体完成（本包只到「主管链一个按钮能点能跑能停」；面板 UX/别角色/NL 启动 另算）。

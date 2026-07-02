# 实现任务包：一键自动推进「停因」在工作台**真机核显 + 可操作化** · UI 专线 v1

日期：2026-07-01　性质：**轻档·前端**（呈现 + 一个可操作按钮；不碰任何后端闸）。

> ⚠️ **先读这条·省你半天**：停因**显示码已经存在**，别重复造。核实物：`outcome.message` 已在 [ProjectWorkflowGovernancePanels.tsx:364](prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowGovernancePanels.tsx:364) 以 `state-warning` 渲染、`outcome.stop_reason` 在 :370 渲染，且该卡挂在**默认展开的一等区**（[ProjectWorkflowSidePanel.tsx:106](prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowSidePanel.tsx:106)，section `defaultOpen={true}`）、**没埋在折叠抽屉里**。所以用户"八成没显示"**极可能是旧编译**（后端停因文案 c52ab14 刚改、要重编 + app 内 Cmd+R 强刷，见记忆 `tauri-dev-frontend-stale-and-uncapturable`），不是缺显示码。**本包第一步是真机证实/证伪，不是加显示。**

## 0. 接手须知（冷启即读，本包自包含）

- 你是 **UI 专线**。全程中文。改前端，**不碰后端**（`src-tauri/**` 一字不动）。
- **背景**：用户实测「一键自动推进」停在 blocked（停因"授权写入范围为空"），反馈"八成没显示"停因。主导线核实物发现**显示码已在**（见顶部⚠️）。后端已把停因文案改细（[director_agent.rs:778](prototypes/productized-desktop-shell/src-tauri/src/director_agent.rs:778) 现在带「具体：授权写入范围为空」+「请重新让 AI 出方案(把这些写进去)」）。姊妹后端包在让咨询方案自带写范围（根治），本包只管**把停因在工作台真机核清 + 顺手做成可操作**。
- **`AutoAdvanceRoleLoopOutcome` 字段**（已存在·别改后端类型）：`stage`(ran/needs_binding/blocked/no_dispatchable) / `planned_task_count` / `prepared_count` / `needs_binding_count` / `blocked_count` / `message` / `stop_reason` / `chain_outcome`。渲染入口 [ProjectWorkflowGovernancePanels.tsx:355-393](prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowGovernancePanels.tsx:355)。
- **一句话**：先真机确认停因到底显不显示（多半重编就好）；确认后把 blocked/no_dispatchable 的停因做得**更显眼 + 给一个直达"重新出方案"的按钮**，让用户不用去别处找路。

## 1. 拍板摘要

- **要做的事**：① 真机核实停因显示（重编 + Cmd+R 后跑一次 blocked 场景，截图证明 `message`/`stop_reason` 出现）；② 若确显示 → 把它**升级成可操作**：blocked（尤其"写范围为空"）时，在停因下给一个按钮直达同卡的「说目标 → 让 AI 出方案」输入，闭环不用用户翻找。
- **为什么**：用户点了自动推进、卡住了、要**一眼看懂为什么 + 一键知道下一步**。显示已有但可能被旧编译挡住、且不够可操作。
- **代价**：一轮·前端·小改（多半是"确认已工作 + 加一个按钮/文案"，不是新建显示）。

## 一句话判据

判改动在不在本包——问：**「是不是只在前端把已有的停因显示核实/强化、并加一个直达重新出方案的操作，没碰任何后端（src-tauri）？」** 是 → 做；否（改后端闸/类型、改 outcome 语义、动 path-lock/授权）→ **停、回主导线。**

## 2. 建什么

1. **真机核实（第一步·必做·别跳）**：重编前端（`npm run tauri dev` 或既有起法）→ app 内 **Cmd+R 强刷** → 在测试项目工作流上造一次 blocked（用一份只读方案跑一键自动推进）→ **截图**证明 `outcome.message`（带"具体：授权写入范围为空"）和/或 `stop_reason` 在卡里出现。
   - 若**已正常显示** → 记进回交（"显示码本就在·旧编译问题"），做第 2 步强化。
   - 若**真没显示** → 那才是真 bug：查 §0 渲染块的条件（`outcome` 为真？`stage` 分支？CSS 把 `state-warning` 藏了？）、按真机现象定位改，**别猜 CSS**（记忆 `ux-render-bugs-measure-before-guessing`：先量运行时真实态再改）。
2. **可操作化**（确认显示后）：`stage==="blocked"`（或含"写范围"关键词）时，在停因文案下加一个按钮，如「重新让 AI 出方案（把写范围写进去）」，`onClick` 滚动/聚焦到同侧栏「方案与授权」区的说目标输入（`ProjectConsultationProposalCard` 的 goal 输入·[ProjectWorkflowGovernancePanels.tsx:483+](prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowGovernancePanels.tsx:483)）。纯前端锚点/聚焦，不发新后端调用。
3. **文案对齐**：确保停因用的是后端给的 `message`（已含"请重新让 AI 出方案"），前端别再包一层含糊话盖住它。needs_binding 分支已有专门提示（:365-368），保留。

## 3. 安全死线

- **后端 0-diff**：`src-tauri/**` 一字不改。outcome 字段/语义不动。
- **不新增后端调用**：可操作按钮只做前端聚焦/滚动，不触发执行、不改授权。
- **真机为准**：这是渲染类改动——**必须真机过**（重编+Cmd+R）才算完成，别只信代码读通（记忆 `ux-render-bugs-measure-before-guessing` / `tauri-dev-frontend-stale-and-uncapturable`）。

## 4. 验收（UI 线自己真机验）

- **核显**：截图 —— blocked 时停因（含"具体：…"）在自动推进卡内可见。
- **可操作**：点"重新出方案"按钮 → 聚焦/滚到说目标输入（截图或录屏）。
- **不回退**：ran/needs_binding/no_dispatchable 四态文案照常（各截一眼或说明）。
- **构建**：前端 `npm run build` / lint 过；`git diff` 只含前端文件。

## 5. 本包不做（deferred）

- 用户在方案卡里**手改 write_roots** 的编辑器（另议）。
- 咨询让方案自带写范围 = **后端姊妹包**（`2026-07-01-consultant-propose-execution-scope-v1.md`）。
- 停因的历史/审计视图。

## 6. 回交

- 跑 §4；回交列：**真机核显结论**（本就显示 / 是真 bug·如何定位修）+ 截图/录屏 + 可操作按钮证据 + `git diff` 仅前端 → 主导线核（重点：后端 0-diff）。

## 7. 不接受为

- 不接受为：改了 `src-tauri/**` / 加了新后端调用 / 只读代码没真机就报"修好了"（渲染类必须真机）/ 用含糊前端话盖掉后端已给的具体停因。

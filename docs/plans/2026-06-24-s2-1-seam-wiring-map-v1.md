# S2-1 接缝勘测 · wiring map v1（2026-06-24）

> 状态：**主导线只读勘测产出**（S2 子计划的 S2-1）。服务于写 S2-2..S2-5 任务包。全部读码所得、未改任何码。
> 结论一句话：**角色循环 UI 没丢、是被我这轮的 HUD 重做当「过程内容/诊断」埋进了画布详情抽屉。S2 的本质比想的更「整理非重建」——把埋着的、真功能但 raw 的角色循环面板提升为一等产品 UI + 用人话呈现。**

## 1. 现状接缝（3 条实测）

**① 顶层 nav `proposal`/`ideas` 视图 = 占位空壳**
- `ActiveWorkbenchView.tsx:203/241`：两者都是 `SourceStylePlaceholder`（只读摘要）。proposal 页自己写明「**真实方案确认仍在项目页权限弹层完成**、本页不批准范围、不创建授权」。→ 真正的角色循环不在这俩入口。

**② 真正的方案/授权/治理/执行/记忆 A 面板 = 在项目页 SidePanel，但被埋**
- `ProjectWorkflowSidePanel.tsx` 装配：Governance（治理四环）/ Execution（统一执行状态）/ Memory（候选治理条）/ Derived / Recovery / RunCheck。
- 但 SidePanel 被 `ProjectWorkflowCanvasView.tsx` 埋进**详情抽屉**：`detailDrawerOpen = useState(typeof window === "undefined")` → **真机 webview（有 window）默认收起**，要点顶边「详情」按钮（:773）才出；HUD 注释（:222-225）明写「过程内容…默认隐，日常视图只剩画布 + 四边」。
- 面板内部还套 fold「统一执行状态…**诊断，默认收起**」（SidePanel:120）。→ **双层埋**。
- **根因**：本会话的全屏 HUD 重做（commit `e19218e`）把角色循环面板归类成「过程内容/开发者字段」收起了——正是用户说的「UI 不行 / 怪」。

**③ A 面板是真功能、但呈现 raw**
- `ProjectWorkflowGovernancePanels.tsx` 有 preview / `prepare-authorized-auto-dispatch` 按钮（:179）+ 渲染 proposal / `linked_plan_authorization` 状态 / `proposal.plan_authorization_id`，经 `lib/tauri.ts` 调真命令（`create_project_consultation_proposal` / `plan_authorization` / …）。→ **功能在、流程通**。
- 但呈现是 `DetailLine` 摊字段（`prepared_dispatch_count`、`linked_plan_authorization status`…）= 开发者向，不是人话。→ 需 C① 方案授权制 UI（人话讲干啥）+ 收字段。

## 2. 对 S2 本质的修正
不是「建新角色循环 UI」，是：**把埋在画布详情抽屉里、真功能但 raw 的角色循环面板，提升为一等产品 UI + 用人话呈现**。后端命令/读模型现成（S1 已把执行闸合一），改的主要是**前端的「框架位置」+「呈现」**，不是后端、不是从零。

## 3. 逐子阶段影响（钉死改哪里）
- **S2-2 方案+授权**：把 GovernancePanels 的「咨询出方案 + 方案授权」段**从详情抽屉提出来**做成一等流程 + 人话呈现（C①）。主改：`ProjectWorkflowCanvasView`（抽屉框架/入口）+ `ProjectWorkflowGovernancePanels`（呈现）+ 可能新增方案/授权主面布局。轻档。
- **S2-3 派发+汇报+复核**：`ProjectWorkflowExecutionPanels`（worker orchestration）同样提升 + 接 S1 闸真跑。碰高危#1。
- **S2-4 记忆全救**：`ProjectWorkflowMemoryPanels` + `MemoryCenterView` + memory/ 子树提升/接全。轻档。
- **S2-5 整理+UX**：清纯协议裸面板（`AgentExecutionPanels` 那种 prepare/confirm 摊给用户的）、收字段（§6）、编辑 UX。轻档。

## 4. 待用户拍的 UX 决策（影响 S2-2 框架）
**角色循环的入口放哪？** 现在它寄生在「项目页画布 HUD 的详情抽屉」里。两条路：
- **(a) 就地提升**：还在画布里，但把角色循环（方案/授权/派发/复核）从「默认收起的诊断抽屉」改成**默认可见的一等区**（画布旁常驻，不是点详情才出）。改动小、不挪架构。
- **(b) 一等入口**：给角色循环一个项目页的**一等位置**（如项目页一个 tab：工作流 / **运转** / 智能体 / 记忆…），画布只管编排、运转循环单独一面。更贴「产品=对话→方案→跑」的主路径，但改动大些。

我倾向 **(a) 先就地提升**（最快让埋着的东西可用、符合「整理非重建/前端不重做」），(b) 留作后续；但这是你的产品手感,你定。

## 同步
- S2-1 完成 → 据本 map + 用户的 §4 决策，写 **S2-2 任务包** → 派执行线。
- 关联：S2 子计划 `2026-06-24-s2-revive-and-unify-subplan-v1.md` / 审查广度目录（A 面板清单）/ 显示边界 §6。

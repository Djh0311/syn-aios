# 工作流画布 · 全屏画布 + 四周悬浮 HUD · 重设计方案 v1（2026-06-23）

> 状态：**待开发**（用户拍板方向：画布占满整窗 · 无标题 · 控件四周悬浮 · 砍过程内容/开发者字段）。
> **纯前端 UI 重构，不碰后端 / 双闸 / 沙箱 / manual_relay。** 画布/渲染类——**必须真机过才算完成**（记忆 `ux-render-bugs-measure-before-guessing`；机器绿 ≠ 真机）；computer-use 抓不到 Tauri dev 二进制（记忆 `tauri-dev-frontend-stale-and-uncapturable`），**主导线看不到渲染、真机由用户逐阶段验**。
> 先读：`CURRENT.md`、上述两条记忆、架构方案 `docs/plans/2026-06-21-workflow-canvas-two-surfaces-one-engine-v1.md`。

## 0. 一句话
项目工作流界面改成「**全屏画布 + 四周悬浮 HUD**」：除一圈细边框，整窗都是可平移缩放的画布；所有控件作为悬浮 overlay 分布在四边；删标题/头部，把「过程内容」与「开发者字段」收进按需面板。

## 1. 现状（核实物 2026-06-23）
- **项目面 `ProjectWorkflowCanvasView`** 纵向堆叠：顶部状态条 `ProjectRuleStatusBar` →（编辑态）`WorkflowCanvasEngine` /（只读态）头部 `workflow-orchestration-head`（eyebrow「项目工作流主入口」+ h3 标题 + path + `workflow-state-actions`：工作流选择器 / 新建 / 编辑 / ▶运行 / 状态徽章）+ `project-canvas-shell`（`ProjectWorkflowReactFlowCanvas` + `renderSidePanel` 侧栏）。
- **共享引擎 `WorkflowCanvasEngine`**（实验 + 项目编辑都用）：节点调色板（`节点调色板` legend）、保存 / 清空 / 重置 / 绑项目按钮、节点编辑器（名称 / 种类 / 提示词 / 自定义字段 + `<details>接执行（真跑用）`：sandbox / 会话 / work_item / ▶运行此节点）。
- **杂在三处**：① 标题/头部（多块 eyebrow + h3：主入口 / 画布状态原因 / 草案 / 受控编辑边界）② 过程内容（状态条、侧栏 audit/dispatch/attention/读回、各 `canvas-hint` 引导语）③ 开发者字段（`接执行` 折叠 + 自定义字段 / 种类选择器）。

## 2. 目标布局（四边 HUD）
- **画布**：`position:absolute; inset:0` 占满整窗（除细边框），React Flow 平移缩放照旧。
- **顶边**：工作流选择器 + 新建/编辑工作流 + 紧凑「运行性」灯（`ProjectRuleStatusBar` 压成一个 pill，不占整条）。
- **左边**：节点调色板（竖排图标栏，编辑态出现）。
- **右边**：**选中节点才出现**的节点面板（名称/种类/提示词）；开发者字段折进**默认收起**的「接执行 / 高级」。
- **底边**：▶运行选中节点 + 保存（编辑态）+ 紧凑状态/提示。
- 四边用 **React Flow `<Panel position="top/left/right/bottom">`** 或绝对定位 overlay；`pointer-events` 只在控件上、画布在底下可操作。
- 边的分配（顶=选择器 / 左=调色板 / 右=节点 / 底=运行）是**默认提议，可调**。

## 3. 砍 / 收（清理）
- **删**：`workflow-orchestration-head`（eyebrow + h3 + path）、「画布状态原因」「受控编辑边界」标题块、各 `canvas-hint` 引导语。项目名顶多做角落小标 / hover 提示，不占头部。
- **收进按需「详情抽屉」（默认隐，点开才看）**：侧栏 audit / dispatch / attention / 读回、状态原因长文。
- **收进右边节点面板折叠区（默认收起）**：`接执行`（sandbox / 会话 / work_item）、自定义字段、种类选择器。日常用看不到开发者字段。

## 4. 机制 / 风险
- **高度链重做**：现「头部 + 壳」纵向分高 → 画布吃满 viewport。⚠️ 记忆 #004「高度塌 0」坑：`canvas-view` / `canvas-flow` / `running-canvas-stage-wrap .project-flow-stage` 的定高那套**别照搬**，按「画布 = viewport」重定。
- **overlay 不挡画布操作**：HUD 容器 `pointer-events:none`、内部控件 `pointer-events:auto`。
- **纯前端**：动 `WorkflowCanvasEngine` + `ProjectWorkflowCanvasView` + CSS；状态条 / 侧栏改 overlay/抽屉。**不碰**后端命令 / 双闸 / 沙箱 / manual_relay。

## 5. 分期（每阶段真机验，机器绿 ≠ 真机）
- **P1 全屏壳**：删头部/标题、画布铺满整窗、现有动作条挪顶边悬浮。真机：全屏 + 可平移 + 高度没塌。
- **P2 四周分布**：调色板 → 左、节点面板 → 右（选中才出）、运行/状态 → 底。真机：每边对位、不挡画布、四面都够得着。
- **P3 砍杂项**：开发者字段折叠、过程内容进抽屉、删余下标题/引导。真机：日常视图只剩画布 + 四边、干净。

## 6. 验证
- **机器**：`typecheck` / `offline` / `build` 全绿（offline 若断言了旧布局结构，需同步改）。
- **真机（必做 · 用户）**：每阶段 `Cmd+R` 看实画面——全屏 / 可平移 / 不挡 / 高度不塌 / 日常干净。**主导线看不到渲染**（computer-use 抓不到 dev 二进制），靠用户逐阶段报。
- 流程：主导线改 → 用户 `Cmd+R` 报画面 → 迭代；阶段过 → 主导线提交（带 CURRENT 回写、问一次）。

## 7. 不在本方案
- 后端 / 闸 / 沙箱 / 真跑逻辑（纯 UI）。
- 乙·自动连环（北极星，另线）。
- 「只读走引擎」引擎统一（已评估不做，CURRENT ④d）——本方案在现有渲染器上做 UI 布局，不动渲染器归属。

## 同步
- 落地后回写 `CURRENT.md`（界面重构从「在做」挪入①，标 P1/P2/P3 真机进度）。
- 关联：`docs/plans/2026-06-21-workflow-canvas-two-surfaces-one-engine-v1.md`；记忆 `ux-render-bugs-measure-before-guessing` / `tauri-dev-frontend-stale-and-uncapturable` / `running-workflows-view-test-load-bearing`。

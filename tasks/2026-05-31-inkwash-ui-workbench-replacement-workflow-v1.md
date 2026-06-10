# 任务包：水墨 HTML 原型替换工作台真实界面 v1

## 任务名

水墨 HTML 原型替换工作台真实界面 v1。

## 所属开发线

桌面应用线 / 前端 UI 线 / 工作流机器线。

总指导线回收。

## 当前判断

用户希望使用四角色工作流机器，把已有 HTML UI 原型接进真实工作台界面。

依据：

- 用户指定源 HTML：`/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/ui-prototype/inkwash-full.html`
- 当前工作台工程：`/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell`
- `CURRENT.md` 记录当前主线为 Codex 角色编排闭环，已完成四角色工作流机器真实闭环。
- `tasks/README.md` 记录 `workflow-machine-real-director-natural-accepted-v1` 已完成，一轮真实四角色 run 已自然收口为 `accepted`。

大白话：

这次不是重新设计 UI，而是让工作流把已经有的水墨 HTML 原型尽量原模原样搬进真实 React/Tauri 工作台，同时保住现有工作流功能。

## 薄弱点

- “原模原样”不能只靠复制 HTML。真实工作台有 React 状态、Tauri 命令、确认弹层、workflow state 和角色编排数据。
- HTML 是 3593 行单文件静态原型，直接塞进 `App.tsx` 会让后续维护困难。
- 当前原型里大量数据是写死的，接入真实工作台时必须区分“视觉占位”和“真实数据”。
- 若一次性替换全部视图，风险是功能按钮丢失、状态入口丢失、确认弹层失效。
- 真实工作流执行会调用绑定 Codex 会话，可能写 `/Users/yoyi/.codex`，必须在执行前再次获得用户明确批准。

## 目标

通过四角色工作流机器完成真实工作台 UI 替换：

1. 以 `inkwash-full.html` 为视觉源稿建立基准。
2. 把水墨壳布局接进 `productized-desktop-shell`。
3. 保留现有工作台真实功能入口和确认机制。
4. 用截图和功能检查回收，判断是否达到“原模原样 + 功能不退化”。

目标工程：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell`

源稿：

- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/ui-prototype/inkwash-full.html`

优先替换范围：

- `App.tsx` 主框架
- `src/styles.css`
- 首页 / 项目 / 工作流 / Agent 会话 / Skills / Harness 相关视图外观
- `ProjectsView.tsx`
- `OfflineRoleOrchestrationPanel.tsx`
- `WorkflowStatePanel.tsx`
- `PermissionDialog.tsx`

## 非目标

- 不重新设计一套和源稿不同的 UI。
- 不引入新运行时 UI 框架。
- 不安装 GSAP 或其他动画库。
- 不修改后端状态机语义。
- 不修改真实 workflow state，除非运行工作流机器自身需要记录 run，且用户明确批准。
- 不改 `/Users/yoyi/Documents/mario test`。
- 不读取 `.env`、`auth.json`、密钥、token 或授权文件。
- 不读取完整 transcript。
- 不把 UI 替换说成复杂业务自动编排已完成。

## 必须保留的功能

替换 UI 后必须还能操作或看见：

- 项目列表和当前项目路径。
- Codex 会话列表和原始会话名。
- 四角色编排：总指导、开发线、验证线、回收线。
- 工作项状态。
- 绑定状态。
- 派发记录。
- 工作流机器运行入口。
- 工作流机器运行结果摘要。
- evidence / handoff 入口或摘要。
- 总指导 review / accepted 结论入口。
- 权限确认弹层。
- 动作确认弹层。
- 错误和 warning 提示。

## 视觉验收基准

执行前必须先生成源稿截图基准。

建议视口：

- 桌面：`1440x900`
- 宽桌面：`1728x1117`
- 窄屏：`390x844`

源稿截图对象：

- `inkwash-full.html`
- 默认首页。
- 至少点击并截图这些视图：`首页`、`项目`、`工作流`、`智能体`、`技能`、`harness`、`设置`。

必须记录：

- 截图路径。
- 视口尺寸。
- 是否出现横向/纵向溢出。
- 是否有文字重叠。
- 是否有外链资源依赖。

已知只读观察：

- HTML 使用内联 CSS。
- HTML 主要交互是左侧 `.rail-btn` 切换 `.view-*`。
- HTML 有多个 data URL SVG 背景。
- 没发现必须依赖外部脚本才能切换页面。
- 主要页面结构包含：`topbar`、`rail-left`、`stage`、`rail-right`、`dock`。

## 建议实施拆分

### 第 1 步：基准冻结

只读完成：

- 读取源 HTML 结构。
- 用浏览器生成基准截图。
- 标注主要视图结构和关键 CSS 变量。

输出：

- 源稿截图路径。
- 结构摘要。
- 风险清单。

### 第 2 步：React 壳替换

改造目标：

- 把 `App.tsx` 的外层布局替换成水墨壳结构。
- 增加左侧 rail、顶部栏、右侧状态栏、底部 dock。
- 保留现有 `activeView` 和确认弹层逻辑。

不得做：

- 不删除 `PermissionDialog`。
- 不删除 `pendingAction` 确认链路。
- 不删除 `runWorkflowMachine` 调用链路。

### 第 3 步：样式迁移

改造目标：

- 从源 HTML 迁移核心 CSS 到 `src/styles.css`。
- 保留 CSS 变量体系：宣纸底、浓墨、朱砂、苔绿、陶土等。
- 处理 React className 命名冲突。
- 保证构建后无 TypeScript / CSS 基础错误。

注意：

- 不要把所有 3593 行机械复制进一个组件。
- 允许先迁移首屏和关键视图样式，再逐步补齐。

### 第 4 步：真实数据挂接

把源稿里的静态文本换成真实数据摘要：

- 项目数来自 `snapshot.projects.length`。
- 会话数来自 `snapshot.sessions.length`。
- 工作流状态来自 `workflowState`。
- 四角色状态来自当前项目 workflow nodes / bindings / machine runs。
- Skills / Harness 页面继续使用现有视图数据。

保留可接受占位：

- 未实现的知识库 / 记忆 / 模型凭据页面可以先显示水墨占位，但必须标明“未接真实数据”。

### 第 5 步：功能回归

必须验证：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- 如改到 Rust / Tauri 命令，再跑 `cargo test --offline`

浏览器验证：

- 打开本地构建或 dev server。
- 截图当前工作台首页和项目页。
- 对比源稿基准，列出差异。
- 检查确认弹层仍能打开。
- 检查工作流机器入口仍在。

## 建议四角色分工

### 总指导

输入用户目标和本任务包，输出执行计划。

必须要求：

- 不重新设计。
- 先冻结截图基准。
- 分阶段替换，优先保功能。

### 开发线

负责代码修改：

- React 组件化。
- CSS 迁移。
- 数据挂接。
- 构建修复。

### 验证线

负责检查：

- 截图对比。
- 类型检查。
- 离线交互测试。
- 构建。
- 功能入口是否还在。

### 回收线

负责判断：

- 是否接近源稿。
- 是否保住工作流机器入口。
- 是否有功能退化。
- 是否需要下一轮修正。

### 总指导结论

只允许以下结论：

- `accepted`
- `needs_changes`
- `paused`
- `rejected`

## 建议工作流机器目标 prompt

```text
目标：把 /Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell 的工作台界面按 /Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/ui-prototype/inkwash-full.html 进行替换，要求尽量原模原样，同时保留现有工作流机器、四角色编排、确认弹层、项目和会话数据入口。

执行原则：
1. 先建立源稿截图基准，不要凭感觉改。
2. 不重新设计，不改成其他风格。
3. 不删除现有业务能力。
4. 不读取 auth.json、.env、密钥、token、授权文件。
5. 不读取完整 transcript。
6. 不修改 /Users/yoyi/Documents/mario test。
7. 不安装新依赖。
8. 如必须分轮，先完成主框架、首页、项目/工作流入口，再回传下一轮计划。

完成标准：
- npm run typecheck 通过。
- npm run test:offline-interaction 通过。
- npm run build 通过。
- 能打开工作台页面并看到水墨壳布局。
- 工作流机器入口仍存在。
- 权限/动作确认弹层仍存在。
- 回传改了哪些文件、截图路径、未完全还原处、下一轮建议。
```

## 允许读取

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- 本任务包
- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/ui-prototype/inkwash-full.html`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/**`
- `/Users/yoyi/.codex/skills/ui-ux-pro-max/SKILL.md`
- `/Users/yoyi/.codex/skills/awesome-design-md/SKILL.md`
- `/Users/yoyi/.codex/skills/awesome-design-md/references/design-md/linear.app/DESIGN.md`
- `/Users/yoyi/.codex/skills/awesome-design-md/references/design-md/cursor/DESIGN.md`
- `/Users/yoyi/.codex/skills/awesome-design-md/references/design-md/raycast/DESIGN.md`

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件
- 完整 transcript 正文
- rollout JSONL 正文

## 允许写入

用户明确批准工作流执行后，允许开发线写：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/**`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/**`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/dist/**`，仅因 `npm run build` 生成
- `/Users/yoyi/workspace/product-line/evidence/2026-05-31-inkwash-ui-workbench-replacement-workflow-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-31-inkwash-ui-workbench-replacement-workflow-v1-result.md`

允许更新：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`

如果通过四角色工作流机器真实执行，允许工作流机器写：

- 真实 workflow state 中的 `workflow_machine_runs[]`、work item 状态、audit events 和必要备份。
- `/Users/yoyi/.codex`，仅限 `codex exec resume` 产生的会话运行写入。

## 必须先获得用户明确批准

真实执行四角色工作流机器前，必须再次让用户明确同意：

- 执行真实 `codex exec resume`。
- 写 `/Users/yoyi/.codex`。
- 修改 `productized-desktop-shell` 项目文件。
- 写真实 workflow state。

如果没有明确批准，只能停留在任务包和只读前置检查。

## 禁止事项

- 禁止直接开始改代码，除非用户明确批准执行本任务包。
- 禁止未建立源稿截图基准就宣称“原模原样”。
- 禁止把水墨原型改成其他 UI 风格。
- 禁止删除现有工作流机器入口。
- 禁止删除确认弹层和权限提示。
- 禁止读取敏感文件。
- 禁止读取完整 transcript。
- 禁止修改 `/Users/yoyi/Documents/mario test`。
- 禁止联网安装依赖。
- 禁止使用 `--dangerously-bypass-approvals-and-sandbox`。
- 禁止在 shell 双引号里写未转义反引号模式；搜索包含反引号文本时必须使用单引号或 `rg -F`。

## 验收标准

必须满足：

- 源稿截图基准已生成并记录。
- 工作台页面使用水墨壳布局：顶部栏、左侧 rail、中央 stage、右侧状态栏、底部 dock。
- 首页视觉接近 `inkwash-full.html` 默认首页。
- 项目 / 工作流视图仍可进入。
- 四角色工作流机器入口仍可见。
- 动作确认弹层仍可用。
- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过。
- 浏览器截图无明显重叠、空白、爆版。
- 回传差异清单，不能笼统写“基本一致”。

## 必须回传

回传必须包含：

1. 薄弱点。
2. 是否执行真实 `codex exec resume`。
3. 是否写 `/Users/yoyi/.codex`。
4. 是否写真实 workflow state。
5. 是否修改工作台项目文件。
6. 是否读取敏感文件或完整 transcript。
7. 源稿截图路径。
8. 目标工作台截图路径。
9. 改了哪些文件。
10. 验证命令和结果。
11. 与源稿仍不一致的地方。
12. 下一轮建议。

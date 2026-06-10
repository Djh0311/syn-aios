# Workbench UI Redesign Brief For External AI v1

日期：2026-06-08

状态：交给外部 AI 的 UI 改造说明书。本文不是权威入口，不改变产品事实和阶段结论；它只用于让不了解项目的 AI 快速理解当前工作台、改 UI 时不能碰的边界、应该优先改善哪些界面。

## 1. 你接手的是什么项目

这是一个本地桌面 AI 工作台原型，路径：

```text
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell
```

技术栈：

- Tauri 2
- Rust 后端
- React + TypeScript + Vite 前端
- React Flow / xyflow 用于项目工作流画布
- 当前事实层主要来自本地 JSON sidecar、workflow state、runtime log、audit、diagnostics 和 Codex 会话索引

当前项目不是普通 SaaS 管理后台。它的产品定位是：

- 用户管理项目。
- 工作台调度本地 / 外部 agent。
- `codex-local` 已有受控真实执行链路。
- 记忆系统、知识库、项目工作流、运行日志、诊断和审计都已经有产品化底座。
- 未来会接入 Claude Code / OpenClaw / OpenCode / OpenCode-like 等多种 agent，但当前不能显示成已真实接入。

你要做的是提升 UI / UX，不是重写架构、改事实模型、绕过权限或新增后端能力。

## 2. 当前 UI 的主要问题

当前 UI 功能很多，但视觉和信息组织很差，典型问题：

- 信息密度失控：大量治理状态、任务包、审计、候选、readback、runtime、adapter 边界堆在页面上，用户很难知道下一步该看哪里。
- 页面像内部调试台，不像产品工作台：raw 状态、边界说明、长文案和卡片堆叠太多。
- 主导航过载：目前 `App.tsx` 里存在首页、项目、想法箱、建议方案、实验画布、智能体、知识库、记忆、技能、harness、工具、模型 / 凭据、设置等入口；但当前权威 UI 边界要求普通一级入口应收敛到项目 / 智能体 / 画布 / 记忆 / 知识库 / 设置。
- 右侧入口还不够产品化：秘书 / 通知 / 待办 / 运行中 / 管理应该清晰分工，不能混成杂项面板。
- 项目页容易变成任务包管理器：项目工作流画布应是主视觉，任务包、权限、readback、记忆包、audit/evidence 应进入节点详情或侧栏摘要。
- 智能体页承载内容过多：会话列表、transcript、adapter、provider、operation boundary、send/resume preview、runtime attention、worker protocol 都在一个页面内，层级需要重新设计。
- 记忆中心和知识库需要更像用户能理解的产品，而不是 store viewer。
- 视觉语言不统一：当前 CSS 偏朴素后台风格，缺少明确层级、留白、动线和状态语义。

## 3. 最重要的边界

改 UI 必须读并遵守：

```text
/Users/yoyi/workspace/product-line/docs/workbench-frontend-display-boundary-v1.md
/Users/yoyi/workspace/product-line/docs/plans/task-package-ui-display-boundary-rule-v1.md
/Users/yoyi/workspace/product-line/CURRENT.md
/Users/yoyi/workspace/product-line/AUTHORITY.md
```

硬边界：

- 不要把 UI 做成任务包管理器。
- 不要把模型、adapter、凭据、能力声明做成普通一级入口。
- 不要把审计、日志、schema、raw event、数据库状态、文件路径大表铺进普通主界面。
- 不要把通知、待办、运行中混成一个列表。
- 不要把秘书塞进项目画布右侧详情；秘书是独立入口 / 悬浮形态。
- 不要恢复底部常驻聊天框。
- 不要在画布主区域铺候选治理、权限、readback、审计、任务包。
- 不要把 observation、candidate、knowledge hit、LLM summary 写成“已记住”“正式事实”“已注入任务包”。
- 不要显示未实现能力按钮。
- 不要用假数据表现已经完成的后端能力。
- 不要把 planned adapters 显示成可执行。Claude Code / OpenClaw / OpenCode / OpenCode-like 目前只能显示为 planned / unavailable / no credential / model unverified。
- 不要执行真实 `codex exec` / `codex exec resume`。
- 不要读写 `/Users/yoyi/.codex`。
- 不要读取 auth/token/secret/`.env`/keychain/OAuth/provider credential/full transcript。

如果你只是改前端 UI，请不要动 Rust 后端、sidecar schema、workflow state 结构或权限控制逻辑。

## 4. 你应该优先改哪些文件

优先关注：

```text
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/App.tsx
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/styles.css
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/AgentView.tsx
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/KnowledgeBaseView.tsx
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/CanvasView.tsx
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/components/SecretaryBrief.tsx
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/components/Badge.tsx
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/components/Metric.tsx
```

建议不要第一轮碰：

```text
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/**
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts
```

除非你已经确认只是为了展示适配，且不会改变数据含义。

## 5. UI 改造目标

### 5.1 总体目标

把工作台从“工程调试面板”改成“用户能实际使用的 AI 项目工作台”。

用户打开后应该立刻知道：

- 当前有哪些项目。
- 哪些任务正在运行。
- 哪些事项需要我批准。
- 哪些结果已经回来。
- 哪些记忆被使用或需要确认。
- `codex-local` 是否可用。
- 哪些 adapter 只是计划中。
- 出错时应该看哪里。

### 5.2 信息层级

每个页面按三层组织：

- 第一层：用户当前要做的事，少量关键状态和主操作。
- 第二层：解释为什么、影响哪里、风险是什么。
- 第三层：审计、日志、raw id、evidence path、debug 信息，默认折叠或放进管理 / 详情。

不要把三层混在同一个卡片里。

### 5.3 视觉方向

建议方向：本地 AI 工作室 + 操控台 + 清晰信息卡片。

不要做：

- 普通后台模板。
- 大面积浅灰卡片堆。
- 紫白 AI SaaS 套壳。
- 只靠 emoji 当图标。
- 深色模式优先。

可以做：

- 更强的空间层级：左侧导航、主内容、右侧状态区各有清楚边界。
- 更少但更有意义的色彩：执行中、待确认、阻断、完成、planned、degraded 用稳定语义色。
- 更清晰的 typography scale：页面标题、状态数字、卡片标题、说明文本层级分明。
- 更强的主画布表达：项目工作流画布要成为项目页视觉中心。
- 适度动效：页面切换、卡片展开、权限弹层出现、运行状态变化可以有 150-300ms 的有意义 motion。

## 6. 页面级改造要求

### 6.1 全局导航和壳层

目标：

- 收敛普通用户一级入口为：项目、智能体、画布、记忆、知识库、设置。
- 其他入口如技能、工具、模型 / 凭据、harness、任务证据应降级到设置、管理、开发者模式或隐藏入口。
- 右侧固定入口为：秘书、通知、待办、运行中、管理。

注意：

- 如果暂时不删除入口，至少在视觉上区分“主入口”和“开发 / 内部入口”。
- 不要把 `模型 / 凭据` 做得像已经可配置 provider；它目前主要是只读边界和 planned 状态。

### 6.2 首页

目标：

- 首页不要堆所有系统状态。
- 首页应该是“今天应该看什么”的仪表盘。

建议模块：

- 当前项目 / 最近项目。
- 正在运行。
- 待我确认。
- 最近结果 / readback。
- 系统健康摘要。
- 记忆待处理摘要。

禁止：

- raw store 大表。
- 长篇阶段说明。
- 把 evidence / handoff 路径作为首页主要内容。

### 6.3 项目页

目标：

- 左侧项目列表只显示项目名和少量状态。
- 中间主区域是项目工作空间。
- 项目内 tab 收敛为：工作流 / 智能体 / 文档 / 记忆 / 设置。
- 工作流 tab 中，画布为主，节点详情为辅。

工作流画布：

- 节点只显示摘要：角色、状态、是否阻断、是否需要确认、最近结果。
- 节点详情再显示任务包摘要、任务记忆包、权限、readback、失败、audit/evidence/handoff 引用。
- 候选治理、lint、observation、process fact 不应铺在画布主区域。

### 6.4 智能体页

目标：

- 接近 Codex 原生会话体验，但要展示工作台的安全边界。
- 优先让用户看懂会话、运行状态和可用 agent。

建议布局：

- 左侧：agent / software 分组和 session list。
- 中间：会话阅读区。
- 右侧：能力和执行边界摘要。

必须清楚显示：

- `codex-local` 是当前唯一可用真实执行 adapter。
- planned adapters 不可执行。
- send / resume 需要权限预览和执行点授权。
- readback unavailable / failed / timed out / blocked_by_guard 不能显示成“0 条结果”。

禁止：

- 显示 Claude Code / OpenClaw / OpenCode 的执行按钮。
- 显示自由输入框并暗示可以直接发给任意 agent。
- 把 provider availability 写成 credential / model 已验证。

### 6.5 记忆中心

目标：

- 让用户理解“工作台记住了什么、候选是什么、为什么会被用于任务”。

建议分区：

- 正式记忆。
- 候选记忆。
- 最近变化。
- 冲突 / 过期 / lint。
- 使用记录 / 任务包引用。
- 生命周期操作入口。

必须区分：

- observation 不是正式记忆。
- candidate 不是正式记忆。
- knowledge hit 不是正式记忆。
- mature pattern 需要 gate 和用户确认。

### 6.6 知识库

目标：

- 知识库是资料和笔记空间，不是记忆中心的重复页面。

建议分区：

- 资料列表。
- 文档引用。
- 关联记忆。
- 可从明确资料提出候选记忆。
- Obsidian-compatible 边界说明。

禁止：

- 直接把知识库资料写成正式记忆。
- 暗示已经有 Obsidian 原生同步、vault 自动扫描或 GraphRAG。

### 6.7 管理

目标：

- 管理是系统健康、日志、诊断、审计、权限、数据位置的集合。
- 普通用户默认只看健康摘要和最近错误。

建议：

- 默认显示健康状态、degraded state、最近错误、数据位置摘要。
- raw log、audit event、internal id、evidence path 放在折叠详情或开发者模式。

## 7. 权限弹层要求

权限弹层是高优先级页面。

用户必须能看懂：

- 这次要做什么。
- 为什么需要做。
- 会写哪里。
- 会不会写 `/Users/yoyi/.codex`。
- 会不会发送 prompt。
- 影响哪些项目文件。
- 风险是什么。
- 批准后下一步是什么。
- 拒绝后会怎样。

按钮至少保持：

- 允许一次。
- 拒绝。
- 查看详情。

不要把权限弹层做成工程日志面板；详情可以折叠。

## 8. 文案规则

允许说：

- 预览。
- 待确认。
- 已记录 observation。
- 候选记忆。
- 正式记忆。
- readback unavailable / failed / timed out。
- planned adapter。
- 未配置凭据。
- 模型未验证。
- 需要执行点授权。

禁止说：

- 已记住，除非是正式记忆。
- 已注入任务包，除非 included list 真的包含正式记忆。
- worker 已执行，除非真实执行记录存在。
- Codex 已收到任务，除非 prompt_sent=true 且真实执行已完成记录。
- Claude Code / OpenClaw / OpenCode 已接入。
- provider 已验证。
- 自动重试已完成。
- 通用自由 send / resume 已完成。

## 9. 技术实现建议

第一轮建议只做前端 UI 重构，不改后端：

- 建立统一 design tokens：颜色、字体、间距、阴影、radius、状态色。
- 抽出可复用布局组件：Shell、SideNav、RightRail、PageHeader、StatusCard、SectionCard、EmptyState、Disclosure、RiskPill。
- 减少每个页面内重复的卡片结构。
- 把长说明默认折叠。
- 保留现有 Tauri wrapper 和读模型，不伪造数据。
- 对大列表做搜索 / 过滤 / 折叠，而不是一次铺满。
- 移动端至少不能横向溢出；桌面是主目标。

如果要大改 `ProjectsView.tsx` 或 `AgentView.tsx`，建议先抽子组件，避免继续扩大单文件复杂度。

## 10. 验收标准

至少运行：

```text
cd /Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell
npm run typecheck
npm run test:offline-interaction
npm run build
```

如果改了布局、导航、右侧入口、项目页、智能体页、记忆页、知识库页或权限弹层，必须做真实窗口或浏览器截图验收。

截图至少覆盖：

- 首页。
- 项目页。
- 项目工作流画布。
- 工作流节点详情。
- 智能体页。
- send / resume 或权限边界区域。
- 记忆中心。
- 知识库。
- 管理 / diagnostics。
- 权限弹层。

如果没有截图工具，必须在 handoff / evidence 写清：

```text
真实窗口 / 截图验收未完成。
```

## 11. 外部 AI 交付物要求

请交付：

- 改了哪些文件。
- UI 结构做了什么调整。
- 哪些入口被收敛、隐藏或保留。
- 哪些文案改了，为什么。
- 是否遵守 `docs/workbench-frontend-display-boundary-v1.md`。
- 是否有未完成截图验收。
- 测试命令结果。
- 仍然看起来差的地方和建议下一轮怎么做。

不要只说“已优化 UI”。必须说明具体优化点和验证结果。

## 12. 给外部 AI 的一句话任务

请在不改变后端事实模型、不新增真实执行能力、不伪造 planned adapter 状态的前提下，把当前 Tauri 工作台前端从工程调试式界面，重构成清晰、可信、用户能理解的 AI 项目工作台 UI。优先改善全局导航、首页、项目工作流画布、智能体页、记忆中心、知识库、管理入口和权限弹层；所有 UI 调整必须遵守现有显示边界和安全边界，并通过类型检查、离线交互测试、构建和截图验收。

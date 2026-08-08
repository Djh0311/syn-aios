# Task Package UI Display Boundary Rule v1

> **状态校正（2026-08-09）：HISTORICAL RULE SOURCE / NOT CURRENT HARD RULE。** 本文中的桌面端边界和“普通界面不暴露内部治理细节”等原则仍可作为来源，但本文件不再自行要求所有任务包引用。当前产品与界面权威看 `../product/syn-product-canon-v1.md`、`../product/authority-register-v1.md` 和当前任务的明确边界。

日期：2026-06-04

状态：当前任务包写作硬规则。凡是任务包可能改到前端、读模型展示、导航入口、右侧面板、项目页、智能体页、画布、记忆、知识库、秘书、管理、通知、待办或运行中状态，都必须引用并落实本文。

## 0. 平台边界硬规则

后续所有 UI 任务包必须默认声明并遵守：

```text
平台边界：本任务只面向桌面端 Tauri 工作台；不做手机端 UI，不做移动端适配，不做 mobile-first responsive 设计，不以手机浏览器或移动视口作为验收目标。
```

固定解释：

- 本项目当前 UI 目标是桌面工作台，不是移动应用。
- 允许保留历史遗留的基础窄屏回退，避免极窄窗口完全不可访问。
- 不允许新增移动端导航、移动端页面、手机专属布局、触控优先交互或移动端验收项。
- 不允许因为 CSS media query 或普通浏览器 smoke，把任务扩展成手机端 UI 设计。
- 涉及真实 UI 验收时，默认验收对象是真实 Tauri 桌面窗口；普通浏览器 smoke 只算辅助加载检查。

## 1. 先说薄弱点

- 现在很多任务名义上是后端、记忆层或工作流层，但实际会改前端类型、读模型、按钮、摘要、面板和文案。
- 如果任务包只写“读 `docs/workbench-frontend-display-boundary-v1.md`”，执行者可能仍会把内部治理信息、schema、raw event、日志、adapter 细节或候选状态铺进普通 UI。
- `docs/workbench-frontend-display-boundary-v1.md` 包含最终形态、中间版本、后端依赖和后置能力，不能被一次性解释成当前任务全部要做。
- 所以任务包必须把 UI 影响显式写出来，而不是只把 UI 文档放进“必读”清单。

## 2. 适用范围

只要任务包满足任一条件，就必须包含第 4 节的固定章节：

- 改 `src/views/**`、`src/components/**`、`src/App.tsx`、`src/styles.css`。
- 改 `src/lib/types.ts`、`src/lib/tauri.ts` 或任何会影响前端读模型的类型。
- 新增、移动、隐藏或重命名导航入口、右侧入口、项目页 tab、画布节点详情、记忆/知识库/智能体显示内容。
- 新增按钮、确认动作、状态摘要、候选治理、观察摘要、正式记忆摘要、秘书提示、通知、待办、运行中、管理或日志入口。
- 改 UI 文案，即使不改布局。

如果任务包确认完全不影响前端，也必须写一句：

```text
UI 显示边界：本任务不改前端、不改读模型、不改 UI 文案；因此不需要 UI 验收。
```

## 3. 必须读取

所有涉及 UI 的任务包必须读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `CURRENT.md`
- `tasks/README.md`

如果任务涉及项目页、画布、记忆、知识库、智能体或秘书，还必须读取相关设计文档：

- 项目页 / 工作流画布：`docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md`
- 记忆：`docs/memory-layer-design-v1.md`、`docs/plans/memory-layer-implementation-slice-v1.md`
- 知识库：当前还未形成完整实施方案，不能假装已经定型。
- 智能体：`decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md` 和当前会话中心任务记录。
- 秘书：`docs/workbench-frontend-display-boundary-v1.md` 第 5 节。

## 4. 任务包必须包含的固定章节

涉及 UI 的任务包必须加入以下章节，不能只写“遵守 UI 文档”：

```md
## UI 显示边界确认

本任务是否改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [ ] 改读模型摘要或状态显示。
- [ ] 改已有页面局部 UI。
- [ ] 新增入口、面板、tab、按钮或确认动作。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `CURRENT.md`
- `tasks/README.md`

平台边界：

- 本任务只面向桌面端 Tauri 工作台。
- 本任务不做手机端 UI、不做移动端适配、不做 mobile-first responsive 设计。
- 本任务不以手机浏览器或移动视口作为验收目标。

本任务允许显示：

- ...

本任务禁止显示：

- ...

显示位置：

- 一级入口：
- 右侧入口：
- 项目页：
- 画布：
- 记忆入口：
- 知识库入口：
- 智能体入口：
- 管理入口：

中间版本范围：

- 本轮必须落地：
- 本轮只做读模型 / 摘要：
- 本轮后置：

后端和数据依赖：

- 需要后端正式读模型：
- 需要审计 / 日志 / 权限 / 状态机：
- 不能用假数据伪装：

UI 文案边界：

- 禁止说：
- 允许说：

验收：

- 类型检查：
- 离线交互测试：
- 构建：
- 真实窗口 / 截图验收：
- 未验收项必须写入 evidence / handoff：
```

## 5. 默认禁止

除非任务包明确授权，否则 UI 任务默认禁止：

- 不做手机端 UI，不做移动端适配，不做 mobile-first responsive 设计。
- 不新增一级入口。
- 不把模型、adapter、凭据、能力声明做成一级入口。
- 不把审计、日志、schema、raw event、数据库状态、路径大表放进普通主界面。
- 不把通知、待办、运行中混成一个列表。
- 不把审计和日志作为右侧顶级图标散开；它们进入管理。
- 不把秘书塞进项目画布右侧详情；秘书是独立入口 / 悬浮形态。
- 不恢复底部常驻聊天框。
- 不把项目工作流页变成任务包管理器。
- 不在画布主区域铺候选治理、权限、readback、审计、任务包。
- 不把 observation、candidate、knowledge hit、LLM summary 文案写成“已记住”“正式事实”“已注入任务包”。
- 不显示未实现能力按钮。
- 不用假数据表现已经完成的后端能力。

## 6. 常用入口边界

一级入口：

- 项目
- 智能体
- 画布
- 记忆
- 知识库
- 设置

右侧入口：

- 秘书
- 通知
- 待办
- 运行中
- 管理

管理内部：

- 审计
- 日志
- 权限
- 健康状态
- 数据位置

项目页：

- 工作流 tab 只显示项目工作流画布。
- 智能体 tab 只显示项目相关智能体会话。
- 文档 tab 只显示项目知识库资料。
- 记忆 tab 只显示项目相关记忆。
- 设置 tab 只显示项目设置。

## 7. 验收规则

涉及 UI 的任务至少运行：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

如果改变布局、导航、右侧入口、画布、项目页、智能体页、记忆页或秘书入口，必须做真实窗口或浏览器截图验收。

如果当前对话没有可用截图工具，不能声称 UI 验收完成，必须在 evidence / handoff 中明确写：

```text
真实窗口 / 截图验收未完成。
```

## 8. 对既有任务包的处理

本文对未来任务包立即生效。

已经完成的历史任务包不需要逐个回改；M1、M1.1、M2 和 M3 先不追溯调整。

从 M4 开始，后续任务包必须先判断是否会改到 UI、前端读模型、UI 文案或显示入口。会改 UI 的任务包必须补入第 4 节固定章节，并按 `docs/workbench-frontend-display-boundary-v1.md` 已确认的 UI 方案执行；确认完全不改 UI 的任务包也必须写明“不改前端、不改读模型、不改 UI 文案”。

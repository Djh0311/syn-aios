# Evidence：task package UI display boundary rule v1

日期：2026-06-04

## 背景

用户指出：现在执行任务时经常会改到前端内容，之前确认过的 UI 显示方案需要能被正确读取并写入每个任务包。

复核结论：

- `docs/workbench-frontend-display-boundary-v1.md` 已是前端显示边界权威文档。
- 但仅把该文档放进“必读”清单不够，执行者仍可能在任务中把内部治理信息、schema、raw event、日志、adapter 细节或候选状态铺进普通 UI。
- 当前 M3 任务包已经读取了 UI 显示边界并有 UI / 读模型要求，但缺少固定格式的“UI 显示边界确认”章节。

## 已完成

新增：

- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

该文档规定：

- 凡是任务包可能改前端、读模型展示、UI 文案、导航入口、右侧入口、项目页、画布、记忆、知识库、智能体、秘书或管理入口，都必须包含“UI 显示边界确认”固定章节。
- 如果任务不改前端，也必须写明“不改前端、不改读模型、不改 UI 文案”。
- 涉及 UI 的任务包必须区分允许显示、禁止显示、显示位置、中间版本范围、后端和数据依赖、UI 文案边界和验收。

更新：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `README.md`
- `tasks/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`

M3 任务包已补入：

- `docs/plans/task-package-ui-display-boundary-rule-v1.md` 到必读清单。
- “UI 显示边界确认”固定章节。
- 明确 M3 允许显示工作流观察只读摘要、状态计数、candidate link。
- 明确 M3 禁止新增一级入口、右侧顶级入口、画布主区域 observation 面板，以及“已记住 / 正式事实 / 已注入任务包”等越界文案。

## 未做

- 未改产品代码。
- 未执行测试，因为本轮只改文档和任务包。
- 未读取 `/Users/yoyi/.codex`。
- 未执行真实 Codex。
- 未创建新的 UI 实现任务。

## 结论

从本轮开始，前端显示方案不再只是“有一份文档可读”，而是成为任务包写作硬规则。后续涉及 UI 的任务包必须显式写入 UI 显示边界确认，否则执行前需要先补任务包。

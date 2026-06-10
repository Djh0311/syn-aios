# Frontend Display Boundary v1 Result

时间：2026-06-03 20:15 CST

## 本轮做了什么

已把用户问答确认的前端显示边界拆解后落成权威文档：

- `docs/workbench-frontend-display-boundary-v1.md`

并已更新当前入口：

- `CURRENT.md`
- `AUTHORITY.md`
- `README.md`
- `tasks/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`

## 关键口径

- 这份文档不是视觉稿，也不是实现任务包。
- 它管“前端应该显示什么、不显示什么、放在哪里显示、哪些内容进详情 / 管理 / 开发者模式”。
- 它明确拆成最终产品边界、中间版本必须落地、后端和数据依赖、后置能力。
- 后续 UI 任务不能把该文档全部内容一次性当成当前开发任务。

## 当前权威入口

- 当前事实：`CURRENT.md`
- 权威索引：`AUTHORITY.md`
- 阶段计划：`STAGE_PLAN.md`
- 中间版本整体计划：`docs/plans/middleware-version-stage-plan-v1.md`
- 前端显示边界：`docs/workbench-frontend-display-boundary-v1.md`
- 任务队列：`tasks/README.md`

## 未验证项

- 未做真实 Tauri 验收。
- 未做浏览器截图。
- 未跑 npm / cargo。

原因：本轮只改文档和入口，没有产品代码改动。

## 后续建议

下一个 UI 或后端任务包在开始前，应先引用 `docs/workbench-frontend-display-boundary-v1.md`，并明确本任务属于：

- 中间版本必须落地。
- 后端和数据依赖。
- 最终形态后置。

不能再把治理后台、schema、raw event、adapter 细节、日志和 evidence 路径直接铺进普通主界面。

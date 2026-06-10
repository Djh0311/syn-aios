# 计划与架构更新交接

## 结论

本轮完成项目计划方案更新。

当前阶段仍以治理 Codex 为主。个人知识库、多 agent、模型调度和超级工作台方向写入后续扩展，不进入当前阶段验收。

## 先说薄弱点

- 技术栈决策是架构方向，不是全部能力已验证。
- React Flow、SQLite FTS、个人知识库还没有原型。
- Tauri 探针仍缺 UI 正文和按钮点击稳定验证。
- 这轮没有新增产品代码。

## 做了什么

- 新增技术栈与扩展架构决策。
- 更新阶段计划总原则。
- 更新 README 当前定位和长期方向。
- 更新工作线控制，防止“超级工作台”提前膨胀。
- 更新任务队列，把个人知识库、多 agent、向量搜索、模型调度列为暂不派发。

## 当前技术栈方向

- Tauri 2
- Rust
- React + TypeScript + Vite
- React Flow
- SQLite
- SQLite FTS
- 向量库后置评估

## 当前阶段目标

只做 Codex 治理闭环：

- 本地只读索引。
- 桌面应用壳。
- 低风险路径动作。
- 任务线、handoff、evidence 管理入口。
- 不写 Codex 状态库。
- 不展示密钥和正文类敏感内容。

## 后续方向

后续再做：

- 本地个人知识库。
- 多 agent 接入。
- 模型辅助调度。
- 复杂画布编排。
- 本地向量搜索。
- 可控写入。

## 改了哪些文件

- `product-line/decisions/2026-05-27-technical-stack-and-expansion-architecture.md`
- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/DEV_LINES.md`
- `product-line/tasks/README.md`
- `product-line/evidence/2026-05-27-plan-and-architecture-refresh.md`
- `product-line/handoffs/2026-05-27-plan-and-architecture-refresh-result.md`

## 下一步建议

继续走原队列：

- 派验证线补测 Tauri 探针 UI 与按钮行为。
- 是否真实点击本机动作，需要用户确认。

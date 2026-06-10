# 计划与架构更新 evidence

## 对象

- 产品线：`product-line`
- 执行线：总指导线
- 记录时间：2026-05-27 21:49:56 CST

## 先说薄弱点

- 这轮没有做新功能实现，只更新计划和架构文档。
- 技术栈方向已经写入决策，但 React Flow、SQLite FTS、个人知识库还没有原型验证。
- 当前仍没有完整桌面应用验收；Tauri 探针还缺 UI 正文和按钮行为稳定验证。
- “超级工作台”只是长期方向，不进入当前阶段目标。

## 这轮做了什么

- 把阶段总原则更新为“当前仍以治理 Codex 为主”。
- 新增技术栈与扩展架构决策。
- 明确第一版推荐技术栈：
  - Tauri 2
  - Rust
  - React + TypeScript + Vite
  - React Flow
  - SQLite
  - SQLite FTS
  - 向量库后置评估
- 把个人知识库、多 agent、模型调度、复杂画布编排明确放到后续版本。
- 同步 README、阶段计划、工作线控制、开发线分工、任务队列。

## 改了哪些文件

- `product-line/decisions/2026-05-27-technical-stack-and-expansion-architecture.md`
- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/DEV_LINES.md`
- `product-line/tasks/README.md`
- `product-line/evidence/2026-05-27-plan-and-architecture-refresh.md`
- `product-line/handoffs/2026-05-27-plan-and-architecture-refresh-result.md`

## 当前权威变化

新增当前权威：

- `product-line/decisions/2026-05-27-technical-stack-and-expansion-architecture.md`

继续有效：

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/DEV_LINES.md`
- `product-line/tasks/README.md`
- `product-line/decisions/2026-05-27-desktop-container-route.md`
- `product-line/handoffs/2026-05-27-tauri-min-prototype-after-cli-review.md`

## 没有改变的边界

- 不新增常设开发线。
- 数据盘点线仍然已完成并封存。
- 当前阶段仍只治理 Codex。
- 不写 `/Users/yoyi/.codex`。
- 不做个人知识库正文入库。
- 不做多 agent 接入。
- 不做向量搜索。
- 不做模型辅助调度。

## 下一步

建议继续派验证线：

- Tauri 探针 UI 与按钮行为验证。
- 是否真实点击“打开目录”“定位文件”“复制路径”需要用户确认，因为会打开 Finder 或改系统剪贴板。

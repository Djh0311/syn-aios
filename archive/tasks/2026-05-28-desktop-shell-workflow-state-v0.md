# 任务包：桌面壳实现工作流事实层 v0 最小读写

## 任务名

在产品化桌面壳中实现工作流事实层 v0 的最小读写。

## 所属开发线

桌面应用线。

当前先派给桌面应用线，因为任务涉及 Tauri / Rust 后端、前端状态展示和用户确认初始化。后续如果工作流事实层需要长期独立推进，可以按开发线治理规则新增或拆分开发线。

## 背景

阶段 3 本地工作流事实层 v0 存储决策已接受。

已知：

- v0 存储方式是 JSON。
- 真实运行路径是 `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`。
- 备份路径是 `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.<timestamp>.json`。
- v0 schema 已定义。
- 索引候选不能自动升级成本地事实。
- 初始化、状态转换、接受候选等写入都必须用户确认并追加 audit event。

依据：

- `product-line/decisions/2026-05-28-workflow-state-storage-v0.md`
- `product-line/handoffs/2026-05-28-workflow-state-storage-v0-review.md`
- `product-line/handoffs/2026-05-28-workflow-state-storage-v0-result.md`
- `product-line/evidence/2026-05-28-workflow-state-storage-v0.md`
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`
- `product-line/decisions/2026-05-28-extensible-first-development-rule.md`

## 目标

- Rust 后端实现读取 v0 状态文件。
- 状态文件不存在时返回空状态和 `exists=false`，不自动创建。
- Rust 后端实现用户确认后的初始化写入。
- 初始化写入创建最小 v0 JSON：
  - `schema_version = workflow_state_v0`
  - `workflow_version = 1`
  - `workspace_id`
  - `projects`
  - `agent_adapters`
  - `workflows`
  - `nodes`
  - `edges`
  - `work_items`
  - `artifacts`
  - `reviews`
  - `audit_events`
  - `capabilities`
  - `harness_resources`
- 写入必须追加 audit event。
- 写入必须使用临时文件 + 原子替换。
- 如果旧状态文件存在，写入前必须备份到 backups；如果不存在，要在 audit event 或返回结果中说明是首次初始化无旧文件。
- 前端显示本地事实层状态：
  - 是否存在状态文件。
  - 状态文件路径。
  - schema / workflow version。
  - workflows / nodes / edges / reviews / audit events 数量。
  - 是否仍处于未初始化。
- 前端提供“初始化工作流事实层”动作，并复用或新增确认弹层；确认文案必须显示目标路径和写入边界。
- 初始化后重新读取并展示状态。
- 不做复杂画布编辑，不做工作项状态转换，不接受 harness 候选成为事实。

## 允许读取

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/DEV_LINES.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/tasks/README.md`
- `product-line/tasks/2026-05-28-desktop-shell-workflow-state-v0.md`
- `product-line/decisions/2026-05-28-workflow-state-storage-v0.md`
- `product-line/handoffs/2026-05-28-workflow-state-storage-v0-review.md`
- `product-line/handoffs/2026-05-28-workflow-state-storage-v0-result.md`
- `product-line/evidence/2026-05-28-workflow-state-storage-v0.md`
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`
- `product-line/decisions/2026-05-28-extensible-first-development-rule.md`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/prototypes/productized-desktop-shell/`

## 允许写入

- `product-line/prototypes/productized-desktop-shell/`
- `product-line/evidence/`
- `product-line/handoffs/`

运行应用时允许在用户确认后写入：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`

测试优先使用临时目录夹具，不要为了单元测试写真实应用数据目录。

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不写项目业务目录。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 不读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不自动创建真实状态文件。
- 不绕过用户确认写状态文件。
- 不自动运行 harness。
- 不把索引候选自动升级成本地事实。
- 不接入非 Codex agent。
- 不做知识库、向量搜索、LM 调度。
- 不做 release 打包。

## 验收标准

- 有 evidence 和 handoff。
- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过，或更新测试后通过。
- `npm run build` 通过。
- `cargo test --offline` 通过。
- 缺状态文件时，后端返回 `exists=false`，不创建文件。
- 初始化必须经前端确认。
- 初始化成功后状态文件符合 v0 schema。
- 初始化写入包含 audit event。
- 已存在状态文件时写入前会备份。
- 写入使用临时文件和原子替换。
- 前端能显示状态文件路径、存在状态、schema version、workflow version、对象数量。
- 不写 `/Users/yoyi/.codex`。
- 不写 Codex 状态库。
- 不自动运行 harness。
- 不展示正文或敏感内容。
- 验证后 5173 无监听残留。

## 必须回传

1. 做了什么
2. 改了哪些文件
3. 新增或修改了哪些测试
4. v0 状态文件路径如何计算
5. 状态文件不存在时如何处理
6. 初始化确认流程是什么
7. 写入、备份、audit、原子替换如何实现
8. 前端如何展示本地事实层状态
9. 是否创建了真实状态文件；如果创建，路径和触发依据是什么
10. 是否触碰禁止事项
11. 验证命令和结果
12. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否没有自动创建状态文件。
- 是否初始化必须用户确认。
- 是否写入路径只在应用数据目录。
- 是否不写 `.codex` 和项目业务目录。
- 是否有备份、audit、原子替换和重新读取校验。
- 是否没有把索引候选自动变成本地事实。
- 是否能作为后续可编辑工作流的事实底座。

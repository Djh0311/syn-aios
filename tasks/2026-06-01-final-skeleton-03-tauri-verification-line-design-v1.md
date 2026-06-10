# 任务包：final-skeleton-03-tauri-verification-line-design-v1

## 目标

设计真实 Tauri 窗口验收线，不直接改 UI，不把普通浏览器截图冒充完整验收。

## 允许

- 只读复核当前 Tauri 启动方式、package scripts、测试工具、既有 evidence。
- 新增验收线设计 evidence。
- 新增 handoff。
- 如需要，新增后续实现任务包草案。
- 更新 `CURRENT.md` 和 `tasks/README.md`。

## 禁止

- 不改业务代码。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 auth、token、`.env`、密钥、完整 transcript 或 rollout JSONL 正文。
- 不把 Chrome headless 当完整 Tauri 验收。
- 不启动 MCP canvas run。

## 执行步骤

1. 复核当前如何启动 Tauri 应用。
2. 复核是否已有截图或 browser 自动化工具。
3. 设计验收对象：首页、项目页、工作流画布、会话页、右侧栏、权限确认弹层。
4. 设计截图路径和命名。
5. 设计 evidence 模板。
6. 写后续实现任务包草案。

## 验收

- 只交设计和下一步实现任务包。
- 如果设计不要求新增高风险权限或外部工具，可以后续继续实现验收线。
- 如果需要新增工具链、权限或读写敏感目录，必须先问用户。

## 输出

- `evidence/2026-06-01-final-skeleton-03-tauri-verification-line-design-v1.md`
- `handoffs/2026-06-01-final-skeleton-03-tauri-verification-line-design-v1-result.md`

## 完成后

普通小任务，不必停；除非实现需要新增工具链、权限或读写敏感目录。

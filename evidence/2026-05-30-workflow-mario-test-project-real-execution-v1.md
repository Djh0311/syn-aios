# 工作流创建测试项目真实执行 v1 evidence

## 结论

工作流测试项目第三次尝试完成：绑定 Codex 会话在 `/Users/yoyi/codex-workflow-mario-test` 创建了四个静态网页文件。

这证明当前工作流可以在用户确认后，通过 `codex exec resume` 派发一个真实写入任务并回收结果。

但这不等于桌面壳业务派发接口已经完整可用。依据：当前 `execute_workflow_node_dispatch` 后端仍只放行 `safe_probe`，真实业务 prompt 本轮走的是可控执行协议记录 + CLI resume。

## 薄弱点

- 前两次尝试失败，说明权限策略没有一次到位。
- 第一次失败原因：目标路径不在被派发会话的可写范围内。
- 第二次失败原因：仅设置工作根仍不够，被派发会话看到的是只读沙箱。
- 第三次成功依赖显式 `codex exec -C /Users/yoyi --sandbox workspace-write ... resume ...`。
- 过程中出现插件目录鉴权、插件同步、MCP shutdown warning。
- 没有做浏览器实机运行，只做了文件级自检和本地文件清单复核。

## 目标

用一个测试专用项目验证真实工作流能不能跑通：

- 目标目录：`/Users/yoyi/codex-workflow-mario-test`
- 任务：创建一个无依赖静态网页横版跳跃小游戏
- 派发对象：绑定业务 thread `019e76d9-0f67-7433-81eb-72da585d28a4`
- 禁止碰：`/Users/yoyi/gameai/agent world`、`/Users/yoyi/workspace/product-line`、敏感文件、完整 transcript

## 执行尝试

### 第一次

- control id：`control:workflow-mario-test-project:1780112518043`
- attempt id：`attempt:workflow-mario-test-project:1780112518043`
- 结果：`needs_changes`
- 原因：目标路径 `/Users/yoyi/codex-workflow-mario-test` 不在当前可写范围内。
- 创建文件数：0

### 第二次

- control id：`control:workflow-mario-test-project:1780112784013`
- attempt id：`attempt:workflow-mario-test-project:1780112784013`
- 结果：`needs_changes`
- 原因：被派发会话仍处在只读沙箱，不能创建目录或写文件。
- 创建文件数：0

### 第三次

- control id：`control:workflow-mario-test-project:1780112848434`
- attempt id：`attempt:workflow-mario-test-project:1780112848434`
- 结果：`completed`
- 执行参数：`codex exec -C /Users/yoyi --sandbox workspace-write ... resume ...`
- 创建文件数：4
- 允许范围外文件数：0

## 创建结果

被派发会话创建了：

- `/Users/yoyi/codex-workflow-mario-test/index.html`
- `/Users/yoyi/codex-workflow-mario-test/styles.css`
- `/Users/yoyi/codex-workflow-mario-test/game.js`
- `/Users/yoyi/codex-workflow-mario-test/README.md`

本地只读复核：

- `index.html`：21 行
- `styles.css`：35 行
- `game.js`：109 行
- `README.md`：13 行
- 总计：178 行

## 最终回复摘要

绑定会话回传：

```text
薄弱点：未做浏览器实机运行，只做了文件级自检；游戏是小体量测试版本。
创建文件：index.html、styles.css、game.js、README.md。
允许范围外文件：否。
读取敏感文件：否。
运行方式：直接用浏览器打开 /Users/yoyi/codex-workflow-mario-test/index.html。
自检：HTML 引用了本地 styles.css 和 game.js；没有外部资源；包含左右移动、跳跃、金币、障碍、计分、生命、开始/重开按钮。
```

## 写入情况

- 是否写 `/Users/yoyi/.codex`：是，通过 `codex exec resume`。
- 是否写真实 workflow state：是。
- 是否写业务测试项目：是，第三次由被派发 Codex 会话写入。
- 是否手工创建小游戏文件：否。本轮开始时误创建过临时草稿目录，已删除；最终项目文件由绑定会话创建。
- 是否读取敏感文件：没有发现。
- 是否读取完整 transcript：否。
- 是否修改 `/Users/yoyi/gameai/agent world`：否。
- 是否修改 `/Users/yoyi/workspace/product-line`：是，只写 evidence / handoff / 临时执行脚本和当前权威文档。

## Workflow State 写入

写入字段类型：

- `workflow_execution_controls[]`
- `execution_attempts[]`
- `audit_events[]`
- 顶层 `updated_at`

备份路径：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780112518043.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780112585415.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780112784013.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780112825638.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780112848434.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780112971435.json`

## 审计事件

- `audit:workflow-mario-test-started:1780112518043`
- `audit:workflow-mario-test-finished:1780112585415`
- `audit:workflow-mario-test-started:1780112784013`
- `audit:workflow-mario-test-finished:1780112825638`
- `audit:workflow-mario-test-started:1780112848434`
- `audit:workflow-mario-test-finished:1780112971435`

## 验证

- 目标目录只读复核：四个目标文件存在。
- `wc -l`：四文件总计 178 行。
- workflow state 只读复核：最近三条 control 分别为 `needs_changes`、`needs_changes`、`completed`。
- workflow state 只读复核：最近三条 attempt 分别为 `needs_changes`、`needs_changes`、`completed`。

## 下一步

需要把这次学到的权限参数固化到桌面壳真实业务派发：

1. `user_reviewed_instruction` 不能再被后端一律拒绝。
2. 派发请求必须携带工作根、额外可写目录、沙箱模式。
3. UI 需要把这些权限显示给用户确认。
4. 结果回收需要区分：权限不足、只读沙箱、执行成功、允许范围外写入。

# 工作流创建测试项目真实执行 v1 result

## 薄弱点

- 这次不是桌面壳业务派发接口的完整胜利。后端正式 `execute_workflow_node_dispatch` 仍只放行 `safe_probe`。
- 前两次尝试失败，说明权限策略需要成为工作流协议的一等字段。
- 成功依赖第三次显式传 `-C /Users/yoyi --sandbox workspace-write`。
- 没有做浏览器实机验证。

## 做了什么

- 通过绑定 Codex 会话 `019e76d9-0f67-7433-81eb-72da585d28a4` 执行三次工作流测试。
- 目标是在 `/Users/yoyi/codex-workflow-mario-test` 创建测试专用静态网页小游戏。
- 第三次成功创建四个文件。
- 写入真实 workflow state 的 control、attempt、audit events。
- 写入 evidence。

## 结果

创建成功：

- `/Users/yoyi/codex-workflow-mario-test/index.html`
- `/Users/yoyi/codex-workflow-mario-test/styles.css`
- `/Users/yoyi/codex-workflow-mario-test/game.js`
- `/Users/yoyi/codex-workflow-mario-test/README.md`

运行方式：

- 直接用浏览器打开 `/Users/yoyi/codex-workflow-mario-test/index.html`

## 边界

- 是否写 `/Users/yoyi/.codex`：是，通过 `codex exec resume`。
- 是否写真实 workflow state：是。
- 是否写 `/Users/yoyi/codex-workflow-mario-test`：是，由被派发会话写入。
- 是否手工创建小游戏文件：否。
- 是否读取敏感文件：没有发现。
- 是否读取完整 transcript：否。
- 是否修改 `/Users/yoyi/gameai/agent world`：否。

## 关键教训

路径已经不是主要问题；权限才是。

前两次失败说明：

- 仅指定目标路径不够。
- 仅指定工作根也不够。
- 真实写入任务必须显式把沙箱模式、工作根、允许写入目录纳入用户审核和派发请求。

## 下一步建议

把 `user_reviewed_instruction` 真实业务派发接入桌面壳后端：

- 支持 `project_root` / `execution_cwd` / `allowed_write_roots` / `sandbox_mode`。
- UI 确认弹层展示这些字段。
- 后端调用 `codex exec` 时传入 `-C` 和 `--sandbox workspace-write`，必要时传 `--add-dir`。
- 回收时记录权限不足、只读沙箱、成功写入、允许范围外写入。

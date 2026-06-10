# 第一条极小业务只读体检执行 v1 result

## 结论

已执行第一条极小业务试跑，但不能回收为干净成功。

结果显示目标目录顶层为空；同时业务会话执行了 `find/stat` 目录元数据命令，违反了“需要执行命令时先停止并回传”的候选权限规则。

## 薄弱点

- 目录为空，无法判断真实业务项目结构。
- `codex exec resume` 已执行并写 `/Users/yoyi/.codex`。
- 出现插件目录鉴权 warning、curated plugin sync warning、MCP shutdown warning。
- 命令边界需要重新确认。

## 做了什么

- 向业务 thread `019e76d9-0f67-7433-81eb-72da585d28a4` 发送只读体检指令。
- 读回最终回复。
- 写入真实 workflow state 的 execution control、attempt、permission request 和 audit events。

## 边界

- 是否写真实 workflow state：是。
- 是否写 `/Users/yoyi/.codex`：是，通过 `codex exec resume`。
- 是否执行 `codex exec resume`：是。
- 是否发送 Codex 消息：是。
- 是否修改业务文件：没有发现。
- 是否读取文件正文：没有发现。
- 是否读取敏感文件：没有发现。

## 写入标识

- control id：`control:first-readonly-project-check:1780111503495`
- attempt id：`attempt:first-readonly-project-check:1780111503495`
- permission request id：`permission:first-readonly-project-check-command-boundary:1780111503495`
- backup：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780111503495.json`

## 验证结果

- `workflow_execution_controls[]` 长度：1，最后状态 `needs_changes`。
- `permission_requests[]` 长度：1，最后状态 `pending`。
- `execution_attempts[]` 长度：1，最后状态 `needs_changes`。
- `python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，`validation_ok`。

## 下一步建议

先确认目录路径是否正确，以及后续是否允许只读目录元数据命令。

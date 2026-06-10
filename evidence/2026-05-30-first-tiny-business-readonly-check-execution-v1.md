# 第一条极小业务只读体检执行 v1 evidence

## 范围

- 目标项目：`/Users/yoyi/gameai/agent world`
- 业务 thread：`019e76d9-0f67-7433-81eb-72da585d28a4`
- 指令类型：用户审核候选的极小只读业务试跑。

## 薄弱点

- 目标目录顶层为空，业务价值有限。
- 本轮执行了 `codex exec resume`，因此写了 `/Users/yoyi/.codex`。
- 业务会话执行了 `find/stat` 目录元数据命令；虽然没有读文件正文、没有修改文件，但这违反了候选指令中的“需要执行命令时停止并回传”权限规则。
- 因此本轮不能回收为干净成功，只能记录为 `needs_changes`。

## 执行结果

最终回复摘要：

```text
1. 薄弱点：目标目录第一层没有返回任何文件或目录名。只能说明“顶层看起来为空”，不能判断项目真实业务状态。
2. 读取了哪些范围：只读取了 /Users/yoyi/gameai/agent world 目录第一层的名称和元数据。没有读取任何文件正文。
3. 项目结构摘要：当前未发现顶层业务文件、目录、配置文件或入口文件。
4. 入口猜测和依据：暂无可用入口猜测。依据：顶层目录没有返回任何条目。
5. 风险点：可能是目录确实为空；也可能是项目在其他路径、还没同步、或被隐藏在不允许读取的更深层位置。当前没有证据判断。
6. 需要用户确认的问题：请确认 /Users/yoyi/gameai/agent world 是否就是目标项目根目录。
7. 下一步建议：如果路径确认无误，下一步可以让你指定允许读取哪些非敏感文件名或允许递归列出目录名，但仍不读正文。
```

## 边界

- 是否写真实 workflow state：是。
- 是否写 `/Users/yoyi/.codex`：是，通过 `codex exec resume`。
- 是否执行 `codex exec resume`：是，用户要求“直接开始做”后执行。
- 是否发送 Codex 消息：是，发送极小只读体检指令。
- 是否读取文件正文：没有发现。
- 是否修改业务文件：没有发现。
- 是否读取敏感文件：没有发现。
- 是否运行 harness：否。

## 观察到的命令边界问题

业务会话执行了只读命令：

```text
find '/Users/yoyi/gameai/agent world' -maxdepth 1 -mindepth 1 -exec stat ...
```

这类命令只读取目录元数据，但候选指令要求“需要执行命令时停止并回传”。所以本轮产生权限策略问题。

## 写入 workflow state

写入字段类型：

- `workflow_execution_controls[]`
- `execution_attempts[]`
- `permission_requests[]`
- `audit_events[]`
- 顶层 `updated_at`

写入标识：

- control id：`control:first-readonly-project-check:1780111503495`
- attempt id：`attempt:first-readonly-project-check:1780111503495`
- permission request id：`permission:first-readonly-project-check-command-boundary:1780111503495`
- audit event ids：
  - `audit:workflow_user_reviewed_instruction_dispatched:1780111503495`
  - `audit:workflow_execution_attempt_recorded:1780111503495`
  - `audit:workflow_permission_requested:1780111503495`
- backup：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780111503495.json`

## 只读复核

- `workflow_execution_controls[]` 长度：1，最后状态 `needs_changes`。
- `permission_requests[]` 长度：1，最后状态 `pending`。
- `execution_attempts[]` 长度：1，最后状态 `needs_changes`。
- 备份存在：是。

## 验证

- `python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，`validation_ok`。

## 下一步

需要总指导和用户确认：

1. `/Users/yoyi/gameai/agent world` 是否就是目标项目根目录。
2. 后续是否允许使用 `find/stat` 这类只读命令读取目录元数据。
3. 如果允许，是否继续做递归目录名扫描；如果不允许，应修改执行器策略。

# 第一条用户审核极小业务试跑指令设计 v1 result

## 结论

已产出一条候选指令：只读体检 `/Users/yoyi/gameai/agent world` 的目录顶层结构，不修改业务文件。

这只是设计，不是执行。候选指令未写入真实 workflow state，也没有发送到任何 Codex 会话。

## 薄弱点

- 用户尚未批准这个候选目标。
- 目标项目目录存在，但顶层没有普通文件或子目录。
- 当前 active binding 是测试会话，不是业务会话；真实试跑前必须重新确认或重新绑定。

## 候选指令摘要

- 目标路径：`/Users/yoyi/gameai/agent world`
- 目标：只读确认目录结构、入口线索和明显风险。
- 允许读取：目录元数据和顶层文件/目录名称。
- 允许写入：无。
- 超时：600 秒。
- 最大重试：0。
- 权限规则：遇到敏感文件、读取正文、写入、执行命令、绑定会话需求时停止并回传。

## 边界

- 是否写真实 workflow state：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否执行 `codex exec resume` 或任何 `codex exec`：否。
- 是否发送 Codex 消息：否。
- 是否读取敏感文件：否。
- 是否读取完整 transcript：否。
- 是否修改真实业务项目文件：否。

## 新增文件

- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-first-user-reviewed-tiny-business-instruction-design-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-first-user-reviewed-tiny-business-instruction-design-v1-result.md`

## 验证结果

- `python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，`validation_ok`。
- `rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md`：通过。

## 需要用户确认的问题

1. 是否接受“只读项目目录体检”作为第一条极小业务试跑？
2. 是否先重新绑定一个 cwd 为 `/Users/yoyi/gameai/agent world` 的业务 Codex 会话？
3. 如果后续目录中出现 `package.json`、`README.md` 等非敏感 manifest，是否允许读取正文？
4. 是否允许把候选指令写入真实 workflow state 的 `workflow_execution_controls[]`？


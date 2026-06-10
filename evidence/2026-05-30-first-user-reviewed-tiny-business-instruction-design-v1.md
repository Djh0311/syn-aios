# 第一条用户审核极小业务试跑指令设计 v1 evidence

## 范围

- 任务包：`/Users/yoyi/workspace/product-line/tasks/2026-05-30-first-user-reviewed-tiny-business-instruction-design-v1.md`
- 开发线：总指导线 / 桌面应用线 / Codex 会话线
- 本轮只设计候选指令和审核边界，不执行真实业务任务。

## 薄弱点

- 当前没有用户确认的具体业务目标，所以不能把候选方向写成已批准事实。
- 目标目录 `/Users/yoyi/gameai/agent world` 存在，但顶层没有普通文件或子目录；只读体检可能只能回传“目录当前为空或没有顶层条目”。
- 当前 workflow state 里的 active binding 指向测试会话 `019e7389-349a-7f02-aa31-a4a90b24e865`，并带有 `confirmed_test_session_not_business_session` 和 cwd 不一致 warning；真实业务试跑前必须重新确认目标业务会话或重新绑定。
- 本轮没有把候选指令写入真实 workflow state。

## 只读依据

- 真实 workflow state：
  - schema：`workflow_state_v0`
  - workflows：1
  - work_items：1
  - `workflow_execution_controls[]`：长度 0
  - `permission_requests[]`：长度 0
  - `execution_attempts[]`：长度 0
  - reviews：1
- 目标项目路径：
  - `/Users/yoyi/gameai/agent world` 存在。
  - 顶层普通文件：0。
  - 顶层子目录：0。
- 当前绑定：
  - thread id：`019e7389-349a-7f02-aa31-a4a90b24e865`
  - binding source：`user_confirmed_test_session`
  - warnings：`session_not_found_in_current_index`、`session_cwd_differs_from_project_root`、`test_session_cwd:/private/tmp/codex-control-probe-v2`、`confirmed_test_session_not_business_session`

## 候选指令

状态：候选，未获用户批准，未写入真实 workflow state。

```json
{
  "instruction_id": "user-reviewed-instruction:first-tiny-business-readonly-project-check-v1",
  "project_root": "/Users/yoyi/gameai/agent world",
  "summary": "只读体检目标项目目录顶层结构，不修改业务文件。",
  "objective": "确认目标项目目录是否已有可用业务文件、入口线索和明显风险，为后续真实业务任务做准备。",
  "target_session": {
    "status": "needs_user_confirmation_or_rebind",
    "reason": "当前 active binding 是测试会话，cwd 为 /private/tmp/codex-control-probe-v2，不应直接作为业务会话。"
  },
  "allowed_read": [
    "/Users/yoyi/gameai/agent world 的目录元数据和顶层文件/目录名称",
    "允许读取非敏感的 manifest 文件名和文件大小；是否读取文件正文需用户另行确认"
  ],
  "allowed_write": [],
  "forbidden": [
    "不修改 /Users/yoyi/gameai/agent world 下任何文件",
    "不读取 .env、auth.json、密钥、token、授权文件或疑似凭据文件",
    "不读取完整 transcript",
    "不执行 codex exec 或 codex exec resume",
    "不发送 Codex 消息",
    "不写 /Users/yoyi/.codex",
    "不运行 harness",
    "不把目录为空解释成业务失败，只如实回传"
  ],
  "permission_policy": "遇到敏感文件、需要读取正文、需要写入、需要执行命令、需要绑定新会话时停止并回传请求，不自行继续。",
  "timeout_seconds": 600,
  "cancel_policy": "用户取消或发现越界风险时立即停止，只回传已读取范围和原因。",
  "failure_policy": "失败时不重试真实执行；只回传失败原因、已读取范围、是否触及敏感边界。",
  "max_retries": 0,
  "return_format": [
    "薄弱点",
    "读取了哪些范围",
    "项目结构摘要",
    "入口猜测和依据",
    "风险点",
    "需要用户确认的问题",
    "下一步建议"
  ]
}
```

## Prompt 预览

```text
你将执行一条用户审核过的极小业务试跑指令。只允许做只读项目目录体检，不允许修改任何业务文件。

目标项目路径：
/Users/yoyi/gameai/agent world

目标：
确认目标项目目录是否已有可用业务文件、入口线索和明显风险，为后续真实业务任务做准备。

允许读取：
1. /Users/yoyi/gameai/agent world 的目录元数据和顶层文件/目录名称。
2. 非敏感 manifest 文件名和文件大小。读取文件正文需要用户另行确认。

禁止事项：
1. 不修改 /Users/yoyi/gameai/agent world 下任何文件。
2. 不读取 .env、auth.json、密钥、token、授权文件或疑似凭据文件。
3. 不读取完整 transcript。
4. 不执行 codex exec 或 codex exec resume。
5. 不发送 Codex 消息。
6. 不写 /Users/yoyi/.codex。
7. 不运行 harness。
8. 不把目录为空解释成业务失败，只如实回传。

权限规则：
遇到敏感文件、需要读取正文、需要写入、需要执行命令、需要绑定新会话时，停止并回传请求，不自行继续。

超时和重试：
超时 600 秒；max_retries 为 0；失败时不重试真实执行，只回传失败原因。

回传格式：
1. 薄弱点
2. 读取了哪些范围
3. 项目结构摘要
4. 入口猜测和依据
5. 风险点
6. 需要用户确认的问题
7. 下一步建议
```

## 边界

- 是否写真实 workflow state：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否执行 `codex exec resume` 或任何 `codex exec`：否。
- 是否发送 Codex 消息：否。
- 是否读取敏感文件：否。
- 是否读取完整 transcript：否。
- 是否修改真实业务项目文件：否。

## 验证

- `python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，输出 `validation_ok`。
- `rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md`：通过，固定字符串搜索完成。

## 需要用户确认的问题

1. 是否接受这个“只读项目目录体检”作为第一条极小业务试跑？
2. 是否需要先重新绑定一个 cwd 等于 `/Users/yoyi/gameai/agent world` 的业务 Codex 会话？
3. 是否允许读取非敏感 manifest 文件正文，例如 `package.json`、`README.md`，如果后续目录中出现这些文件？
4. 是否允许把候选指令写入真实 workflow state 的 `workflow_execution_controls[]`？


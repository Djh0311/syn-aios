# 任务包：第一条用户审核极小业务试跑指令设计 v1

## 任务名

第一条用户审核极小业务试跑指令设计 v1。

## 所属开发线

总指导线 / 桌面应用线 / Codex 会话线。

验证线按需复核。

## 当前判断

工作流已经跑通无业务 safe probe，并且真实 workflow state 已经有可控执行协议空队列。

下一步仍不能直接执行真实业务派发。必须先设计第一条用户明确审核过的极小业务指令，把目标、允许读写、禁止事项、权限、超时、失败回收和输出格式说清楚。

依据：

- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1-final-review.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-controlled-execution-protocol-v1-incident-guardrail-review.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-protocol-empty-queues-real-state-v1-review.md`

## 薄弱点

- 真实业务自动编排仍未开始。
- 目前没有用户确认的业务目标，不能编造业务任务。
- 目标测试会话 cwd 曾是 `/private/tmp/codex-control-probe-v2`，不是业务项目目录；真实业务试跑必须重新确认目标会话、项目路径和允许写入范围。
- 本任务只设计指令和审核边界，不执行 `codex exec resume`。

## 目标

产出第一条极小业务试跑指令的候选方案，并让它能被用户审核：

1. 明确候选业务目标。
2. 明确目标项目路径。
3. 明确目标 Codex 会话或说明需要重新绑定。
4. 明确允许读取范围。
5. 明确允许写入范围。
6. 明确禁止事项。
7. 明确权限确认规则。
8. 明确超时、取消、失败、重试规则。
9. 明确回传格式。
10. 生成用户可审核的 prompt 预览。

大白话目标：

先把“要让 Codex 做哪一件非常小的真事”写清楚，让用户能看懂、能拒绝、能修改；还不真正派出去。

## 非目标

- 不执行真实业务任务。
- 不执行 `codex exec resume`。
- 不执行任何 `codex exec`。
- 不发送 Codex 消息。
- 不写 `/Users/yoyi/.codex`。
- 不读取完整 transcript。
- 不读取授权、密钥、`.env`、token。
- 不运行 harness。
- 不修改真实业务代码，除非用户另行确认执行试跑。
- 不把指令设计说成真实业务自动工作流完成。

## 允许读取

允许读取项目内：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-protocol-empty-queues-real-state-v1-review.md`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/`

允许只读真实 workflow state 的必要结构：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

允许只读目标业务项目的顶层结构，但不得读取敏感文件：

- `/Users/yoyi/gameai/agent world`

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件内容
- 完整 transcript 正文

## 允许写入

允许写项目内：

- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-first-user-reviewed-tiny-business-instruction-design-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-first-user-reviewed-tiny-business-instruction-design-v1-result.md`

如需要补 UI 预览或 schema 支持，允许写：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/`

本任务默认不写真实 workflow state。

如要把候选指令草案写入真实 workflow state 的 `workflow_execution_controls[]`，必须单独获得用户确认。

## 禁止事项

- 禁止执行 `codex exec resume`。
- 禁止执行任何 `codex exec`。
- 禁止发送任何消息到 Codex 会话。
- 禁止写 `/Users/yoyi/.codex`。
- 禁止读取完整 transcript。
- 禁止读取授权、密钥、`.env`、token。
- 禁止运行 harness。
- 禁止修改真实业务项目文件。
- 禁止把未获用户确认的业务目标写成事实。
- 禁止在 shell 双引号里写未转义反引号模式；搜索包含反引号的文本时必须使用单引号或 `rg -F`。

## 候选指令原则

第一条真实业务试跑必须足够小：

- 优先只读分析，避免直接改业务文件。
- 如必须写入，优先写项目内临时 evidence / report 文件，不改源码。
- 目标必须能在 5 到 10 分钟内完成。
- 输出必须结构化，方便总指导回收。
- 遇到权限、敏感文件、`.env`、密钥、token，必须停止并回传。

## 建议候选方向

如果没有更明确的用户业务目标，建议候选只做只读项目体检：

- 读取 `/Users/yoyi/gameai/agent world` 的顶层文件结构。
- 不读取 `.env`、密钥、token、授权文件。
- 输出项目结构摘要、可运行入口猜测、风险点、下一步建议。
- 不改任何业务文件。

注意：这只是候选，不代表用户已经批准。

## 建议 user_reviewed_instruction schema

```json
{
  "instruction_id": "user-reviewed-instruction:<timestamp>",
  "project_root": "/Users/yoyi/gameai/agent world",
  "summary": "只读体检目标项目顶层结构，不修改业务文件。",
  "objective": "确认项目结构、入口和明显风险，为后续真实业务任务做准备。",
  "allowed_read": [
    "/Users/yoyi/gameai/agent world"
  ],
  "allowed_write": [],
  "forbidden": [
    "不读取 .env、密钥、token、授权文件",
    "不修改业务文件",
    "不执行 codex exec 或 codex exec resume",
    "不发送 Codex 消息"
  ],
  "timeout_seconds": 600,
  "max_retries": 0,
  "permission_policy": "遇到敏感文件或写入需求必须停止并回传",
  "return_format": [
    "薄弱点",
    "读取了哪些范围",
    "项目结构摘要",
    "入口猜测和依据",
    "风险点",
    "下一步建议"
  ]
}
```

## 验收标准

必须满足：

- 产出一条用户可审核的极小业务指令候选。
- 明确这只是设计，不是执行。
- 明确目标项目路径、读写范围、禁止事项。
- 明确权限、超时、取消、失败、重试规则。
- 不执行任何 `codex exec`。
- 不发送 Codex 消息。
- 不写 `/Users/yoyi/.codex`。
- 不读取完整 transcript。
- 不读取敏感文件。

建议验证命令：

```bash
python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json
rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md
```

安全搜索要求：

- 搜索固定文本使用 `rg -F '固定文本' ...`。
- 搜索包含反引号的文本必须使用单引号或 `rg -F`。
- 禁止用 shell 双引号包住未转义反引号。

## 必须回传

回传必须包含：

1. 薄弱点。
2. 候选指令内容。
3. 候选指令是否写入真实 workflow state。
4. 是否写 `/Users/yoyi/.codex`。
5. 是否执行 `codex exec resume` 或任何 `codex exec`。
6. 是否发送 Codex 消息。
7. 是否读取敏感文件。
8. 新增 evidence / handoff。
9. 验证命令和结果。
10. 需要用户确认的问题。

## 总指导回收重点

总指导回收时必须判断：

- 是否编造业务目标。
- 是否越过只设计不执行的边界。
- 是否读了敏感文件。
- 是否能让用户一眼看懂并决定是否批准。

通过后，再由用户明确确认是否执行这条极小业务试跑。

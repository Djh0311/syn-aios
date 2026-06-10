# 工作流可控执行协议 v1 事故防护小修 evidence

## 范围

- 任务包：`/Users/yoyi/workspace/product-line/tasks/2026-05-30-workflow-controlled-execution-protocol-v1-incident-guardrail.md`
- 开发线：桌面应用线 / 验证线
- 本轮只补文档护栏，不改协议功能代码。

## 薄弱点

- 这是流程事故防护，不是功能完成证明。
- 不能用“没有 prompt”抹掉上一轮 `codex exec resume` 被执行过的事实。
- 本轮验证必须使用安全搜索写法；不能再用 shell 双引号包住含反引号的模式。

## 做了什么

- 在 `2026-05-30-workflow-controlled-execution-protocol-v1.md` 中补充禁止事项：
  - 禁止在 shell 双引号里写未转义反引号模式。
  - 搜索包含反引号的文本时必须使用单引号或 `rg -F`。
- 在同一任务包的验证说明中补充安全搜索要求。
- 在 `tasks/README.md` 当前任务限制中补充同样规则。
- 在 `CURRENT.md` 当前边界中补充同样规则。

## 写入边界

- 是否写真实 workflow state：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否执行 `codex exec resume`：否。
- 是否执行任何 `codex exec`：否。
- 是否发送 Codex 消息：否。
- 是否读取完整 transcript：否。
- 是否读取授权、密钥、`.env`、token：否。
- 是否运行 harness：否。

## 验证

- `rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md`：通过，固定字符串搜索完成，没有触发命令替换。
- `python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，输出 `validation_ok`。

## 后续要求

- 以后搜索 Markdown 中带反引号的文本，必须使用单引号或 `rg -F`。
- 不要使用 shell 双引号包住未转义反引号模式。


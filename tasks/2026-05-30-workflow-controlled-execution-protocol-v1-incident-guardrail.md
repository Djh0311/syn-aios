# 任务包：工作流可控执行协议 v1 事故防护小修

## 任务名

工作流可控执行协议 v1 事故防护小修。

## 所属开发线

桌面应用线 / 验证线。

## 当前判断

`2026-05-30-workflow-controlled-execution-protocol-v1.md` 方向正确，但不能直接接受。

原因是开发线在自检搜索时误把反引号放进 shell 双引号，触发了命令替换，意外执行 `codex exec resume`。输出显示没有 prompt，但这仍违反任务包禁止项。

依据：

- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-controlled-execution-protocol-v1-review.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-controlled-execution-protocol-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-controlled-execution-protocol-v1-result.md`

## 薄弱点

- 这是流程事故防护，不是功能大改。
- 不能用“没有 prompt”来抹掉 `codex exec resume` 被执行过的事实。
- 不能为了证明没问题再运行任何 `codex exec resume`。

## 目标

补上最小防护，防止后续自检搜索再次误触发命令替换：

1. 在任务包或验证说明中明确：搜索包含反引号的文本时必须用单引号或 `rg -F`。
2. 给当前任务 evidence/handoff 的自检建议补一条安全写法说明。
3. 如适合，在项目文档中加入一条短规则：禁止在 shell 双引号里写未转义反引号模式。
4. 不改协议功能代码，除非发现 UI 文案里有必须修的小问题。

## 非目标

- 不重做工作流可控执行协议。
- 不写真实 workflow state。
- 不执行 `codex exec resume`。
- 不发送 Codex 消息。
- 不写 `/Users/yoyi/.codex`。
- 不读取完整 transcript。
- 不读取授权、密钥、`.env`、token。
- 不运行 harness。
- 不执行真实业务任务。

## 允许读取

允许读取项目内：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/tasks/2026-05-30-workflow-controlled-execution-protocol-v1.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-controlled-execution-protocol-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-controlled-execution-protocol-v1-result.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-controlled-execution-protocol-v1-review.md`

## 允许写入

允许写入项目内：

- `/Users/yoyi/workspace/product-line/tasks/2026-05-30-workflow-controlled-execution-protocol-v1.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-controlled-execution-protocol-v1-incident-guardrail.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-controlled-execution-protocol-v1-incident-guardrail-result.md`

## 禁止事项

- 禁止执行 `codex exec resume`。
- 禁止执行任何 `codex exec`。
- 禁止发送任何消息到 Codex 会话。
- 禁止写 `/Users/yoyi/.codex`。
- 禁止读取完整 transcript。
- 禁止读取授权、密钥、`.env`、token。
- 禁止用双引号包住含反引号的 shell 搜索模式。

## 验收标准

必须满足：

- 文档中有明确的安全搜索写法。
- evidence/handoff 记录本轮没有再次执行 `codex exec resume`。
- 不改动真实 workflow state。
- 不写 `/Users/yoyi/.codex`。
- 不执行 `codex exec resume`。

建议验证命令：

```bash
rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md
python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json
```

注意：如果要搜索带反引号的文本，使用单引号或 `rg -F`，不要使用 shell 双引号。

## 必须回传

回传必须包含：

1. 薄弱点。
2. 做了什么。
3. 改了哪些文件。
4. 是否写真实 workflow state。
5. 是否写 `/Users/yoyi/.codex`。
6. 是否执行 `codex exec resume`。
7. 是否发送 Codex 消息。
8. 新增 evidence / handoff。
9. 验证命令和结果。

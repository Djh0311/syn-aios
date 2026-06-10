# 任务包：创建 workflow mario test Codex 会话 v1

## 任务名

创建 workflow mario test Codex 会话 v1。

## 所属开发线

Codex 会话线 / 索引内核线。

总指导线回收。

## 当前判断

README smoke 不能继续准备 workflow state，因为当前索引里没有 `/Users/yoyi/codex-workflow-mario-test` 对应 Codex thread。

依据：

- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-prepare-workflow-state-for-readme-smoke-v1-review.md`
- 当前 `codex-index.json` 无 `/Users/yoyi/codex-workflow-mario-test` thread。
- 任务包 `2026-05-30-prepare-workflow-state-for-readme-smoke-v1.md` 明确要求：没有合适 thread 时停止，不得擅自创建。

## 薄弱点

- 这一步会执行真实 `codex exec`。
- 这一步会写 `/Users/yoyi/.codex`。
- 这一步只创建测试会话并刷新索引，不修改 README。
- 这一步不写真实 workflow state。

## 目标

创建一个 cwd / project_root 为 `/Users/yoyi/codex-workflow-mario-test` 的无业务测试 Codex 会话，并刷新索引。

测试会话只用于后续 workflow node active binding。

建议创建 prompt：

```text
请只回复这一句：WORKFLOW_MARIO_TEST_SESSION_READY_2026_05_30。不要读取、列出、修改任何文件，不要运行任何命令。
```

## 非目标

- 不执行 `codex exec resume`。
- 不发送 README smoke 指令。
- 不修改 `/Users/yoyi/codex-workflow-mario-test/README.md`。
- 不写真实 workflow state。
- 不读取完整 transcript。
- 不读取 `auth.json`、`.env`、密钥、token、授权文件。
- 不运行 harness。
- 不删除、移动、归档任何 Codex 会话。

## 必须先获得用户明确批准

执行前必须让用户明确同意：

- 执行真实 `codex exec`。
- 写 `/Users/yoyi/.codex`。
- 创建 cwd 为 `/Users/yoyi/codex-workflow-mario-test` 的测试会话。
- 刷新 `codex-index.json`。

没有明确批准，只能做只读检查。

## 允许读取

允许读取：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json`
- `/Users/yoyi/codex-workflow-mario-test/README.md`，只用于确认目标项目存在，不读取敏感内容

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件内容
- 完整 transcript 正文

## 允许写入

用户明确批准后，允许写：

- `/Users/yoyi/.codex`，通过 `codex exec`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-create-codex-workflow-mario-test-session-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-create-codex-workflow-mario-test-session-v1-result.md`

允许更新：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`

## 禁止事项

- 禁止执行 `codex exec resume`。
- 禁止修改 README。
- 禁止写真实 workflow state。
- 禁止读取完整 transcript。
- 禁止读取敏感文件。
- 禁止运行 harness。
- 禁止使用 `--dangerously-bypass-approvals-and-sandbox`。
- 禁止在 shell 双引号里写未转义反引号模式；搜索包含反引号文本时必须使用单引号或 `rg -F`。

## 验收标准

必须满足：

- 创建一个新 Codex thread。
- 新 thread 的 cwd / project_root 是 `/Users/yoyi/codex-workflow-mario-test`。
- 最终回复完全匹配 `WORKFLOW_MARIO_TEST_SESSION_READY_2026_05_30`。
- 刷新后的 `codex-index.json` 能查到该 thread。
- `rollout_exists = true`。
- 不执行 `codex exec resume`。
- 不修改 README。
- 不写真实 workflow state。
- 不读取敏感文件或完整 transcript。

## 必须回传

回传必须包含：

1. 薄弱点。
2. 是否获得用户明确批准。
3. 是否执行 `codex exec`。
4. 是否写 `/Users/yoyi/.codex`。
5. 新 thread id。
6. cwd / project_root。
7. 最终回复摘要。
8. 是否刷新 `codex-index.json`。
9. 目标 thread 是否进入索引。
10. rollout 是否存在。
11. 是否修改 README。
12. 是否写真实 workflow state。
13. 新增 evidence / handoff。
14. 验证命令和结果。

## 总指导回收重点

总指导回收时必须判断：

- 是否只创建测试会话，没有发送 README smoke。
- 新 thread 是否真的属于 `/Users/yoyi/codex-workflow-mario-test`。
- 是否可以进入下一轮准备 workflow state。

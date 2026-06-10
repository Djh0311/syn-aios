# 创建 workflow mario test Codex 会话 v1 证据

## 结论

薄弱点：

- 本轮执行了真实 `codex exec`。依据：CLI 输出 `thread.started`，并返回新 thread id。
- 本轮写了 `/Users/yoyi/.codex`。依据：`codex exec` 创建了新的 rollout 文件。
- 本轮只创建测试会话并刷新索引，不代表 README smoke 已执行。
- CLI 过程中出现插件鉴权 warning 和 MCP 关闭 warning；最终回复匹配，但这些 warning 仍要在后续真实派发稳定性里继续观察。
- 第一次刷新索引用系统 `python3` 失败。依据：系统 Python 是 3.9.6，脚本使用 `datetime.UTC`，该属性不可用。后续改用 `/Users/yoyi/miniconda3/bin/python3` 成功。

可用结果：

- 已获得用户明确批准。
- 已创建 cwd 为 `/Users/yoyi/codex-workflow-mario-test` 的测试 Codex 会话。
- 新 thread id：`019e7738-5e29-74e0-a22f-5c2481b64c38`。
- 最终回复完全匹配：`WORKFLOW_MARIO_TEST_SESSION_READY_2026_05_30`。
- 已刷新 `prototypes/index-kernel/codex-index.json`。
- 新 thread 已进入当前索引。
- 索引中 `project_root=/Users/yoyi/codex-workflow-mario-test`。
- 索引中 `rollout_exists=true`。

## 用户批准

用户回复：

```text
同意
```

本轮按任务包解释，这个同意覆盖：

- 执行真实 `codex exec`。
- 写 `/Users/yoyi/.codex`。
- 创建 cwd 为 `/Users/yoyi/codex-workflow-mario-test` 的测试会话。
- 刷新 `codex-index.json`。

## 执行对象

- 项目路径：`/Users/yoyi/codex-workflow-mario-test`
- 任务包：`tasks/2026-05-30-create-codex-workflow-mario-test-session-v1.md`
- 期望回复：`WORKFLOW_MARIO_TEST_SESSION_READY_2026_05_30`

执行 prompt：

```text
请只回复这一句：WORKFLOW_MARIO_TEST_SESSION_READY_2026_05_30。不要读取、列出、修改任何文件，不要运行任何命令。
```

## 执行记录

执行命令：

```bash
codex exec --skip-git-repo-check --json --output-last-message /private/tmp/codex-workflow-mario-test-session-last-message.txt "请只回复这一句：WORKFLOW_MARIO_TEST_SESSION_READY_2026_05_30。不要读取、列出、修改任何文件，不要运行任何命令。"
```

执行 cwd：

```text
/Users/yoyi/codex-workflow-mario-test
```

CLI 返回：

```json
{"type":"thread.started","thread_id":"019e7738-5e29-74e0-a22f-5c2481b64c38"}
```

最终 agent message：

```text
WORKFLOW_MARIO_TEST_SESSION_READY_2026_05_30
```

last message 文件：

```text
/private/tmp/codex-workflow-mario-test-session-last-message.txt
```

读取结果：

```text
WORKFLOW_MARIO_TEST_SESSION_READY_2026_05_30
```

## CLI warning

执行过程中出现 warning：

- remote plugin catalog 鉴权 warning。
- MCP process group terminate permission warning。
- MCP shutdown initialize warning。

判断：

- warning 没有阻止本次测试会话创建。
- warning 没有改变最终回复匹配事实。
- 后续长任务和真实业务派发仍要继续观察这些 warning 是否影响稳定性。

## 索引刷新

第一次刷新命令：

```bash
python3 prototypes/index-kernel/build_index.py --pretty
```

结果：失败。

原因：

```text
AttributeError: module 'datetime' has no attribute 'UTC'
```

版本依据：

```text
Python 3.9.6
```

改用 Python 3.13：

```bash
/Users/yoyi/miniconda3/bin/python3 prototypes/index-kernel/build_index.py --pretty
```

结果：

```json
{"memory_count": 11, "output": "/Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json", "plugin_count": 11, "project_count": 34, "rollout_checked": 329, "rollout_existing": 329, "skill_count": 51, "thread_count": 329, "warning_count": 0}
```

结构校验：

```bash
/Users/yoyi/miniconda3/bin/python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json
```

结果：

```text
validation_ok
```

warning summary：

```bash
/Users/yoyi/miniconda3/bin/python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json --warning-summary
```

结果：

```json
{"entrypoints_truncated": 4, "handoff_candidates_truncated": 1, "harness_candidates_truncated": 1, "missing_entrypoints": 6, "missing_manifest": 13, "missing_readme": 12, "missing_version": 15, "project_root_missing": 2, "title_truncated": 89, "weak_harness_signal": 1}
```

## 索引复核

查询命令：

```bash
jq '.threads | map(select(.thread_id == "019e7738-5e29-74e0-a22f-5c2481b64c38")) | {count: length, items: map({thread_id, title, project_root, cwd, rollout_exists, rollout_path, thread_source})}' prototypes/index-kernel/codex-index.json
```

结果：

```json
{
  "count": 1,
  "items": [
    {
      "thread_id": "019e7738-5e29-74e0-a22f-5c2481b64c38",
      "title": "请只回复这一句：WORKFLOW_MARIO_TEST_SESSION_READY_2026_05_30。不要读取、列出、修改任何文件，不要运行任何命令。",
      "project_root": "/Users/yoyi/codex-workflow-mario-test",
      "cwd": null,
      "rollout_exists": true,
      "rollout_path": "/Users/yoyi/.codex/sessions/2026/05/30/rollout-2026-05-30T12-50-43-019e7738-5e29-74e0-a22f-5c2481b64c38.jsonl",
      "thread_source": "user"
    }
  ]
}
```

项目复核：

```bash
jq '.projects | map(select(.project_root == "/Users/yoyi/codex-workflow-mario-test" or .root_path == "/Users/yoyi/codex-workflow-mario-test" or .path == "/Users/yoyi/codex-workflow-mario-test")) | {count: length, items: .}' prototypes/index-kernel/codex-index.json
```

结果：`count = 1`，`thread_count = 1`，`active_thread_count = 1`。

## 测试项目边界

只读复核文件：

```text
/Users/yoyi/codex-workflow-mario-test/game.js
/Users/yoyi/codex-workflow-mario-test/index.html
/Users/yoyi/codex-workflow-mario-test/styles.css
/Users/yoyi/codex-workflow-mario-test/README.md
```

README smoke 目标行：

```bash
rg -n -F 'Workflow dispatch smoke passed.' /Users/yoyi/codex-workflow-mario-test/README.md
```

结果：exit 1，无匹配。

文件 hash 只读复核：

```text
6f9cc4be0f3ad0cdf7926af9bcbbd747a383ce6d3e2085a9322786b8176811db  /Users/yoyi/codex-workflow-mario-test/README.md
35ecf58229427d00a3087b729995adf263aeefefbda79bb3bae7b288c3fbcaa8  /Users/yoyi/codex-workflow-mario-test/index.html
7c866d69c5d6c52d69ede7e14803b5b6eb2fdacca3b7ef76917ac061b65bde1e  /Users/yoyi/codex-workflow-mario-test/styles.css
a794af1e4116edf3cdf456c14f20da36394de8bb989c0799a3b017bf48c7f2ee  /Users/yoyi/codex-workflow-mario-test/game.js
```

判断：

- 本轮没有执行 README smoke。
- README 目标行仍不存在。
- 本轮没有修改 README 或测试项目文件。

## workflow state 边界

是否写真实 workflow state：否。

依据：

- 本轮没有执行写 state 的脚本或后端命令。
- 只读 `stat` 显示 state 文件修改时间仍是本轮之前：

```text
May 30 11:49:31 2026 /Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json
```

## 安全边界

本轮做了：

- 执行真实 `codex exec`。
- 写 `/Users/yoyi/.codex`。
- 刷新 `codex-index.json`。
- 写本 evidence 和 handoff。

本轮没有做：

- 没有执行 `codex exec resume`。
- 没有发送 README smoke 指令。
- 没有修改 README。
- 没有写真实 workflow state。
- 没有读取完整 transcript。
- 没有读取 `/Users/yoyi/.codex/auth.json`。
- 没有读取 `.env`、密钥、token、授权文件内容。
- 没有运行 harness。

补充说明：

- 本轮只用 `find` 验证 rollout 文件路径存在，没有打开 rollout JSONL 正文。
- 本轮读取的是 CLI 输出和 last-message 文件，不是完整 transcript。

## 下一步

可以回到：

```text
tasks/2026-05-30-prepare-workflow-state-for-readme-smoke-v1.md
```

原因：

- 该任务上一轮暂停的主要阻塞是没有 `/Users/yoyi/codex-workflow-mario-test` 对应 thread。
- 现在阻塞已消除。

但下一步会写真实 workflow state，仍需要用户明确批准。

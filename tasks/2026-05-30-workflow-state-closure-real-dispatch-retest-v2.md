# 任务包：工作流状态收口真实派发复测 v2

## 任务名

工作流状态收口真实派发复测 v2。

## 所属开发线

桌面应用线 / Codex 会话线 / 工作流状态线。

总指导线回收。

## 当前判断

v1 真实复测没有通过，原因只能确认到 300 秒超时。v2 retry work item 已准备好，但还没有执行真实重试派发。

依据：

- `handoffs/2026-05-30-workflow-state-closure-real-dispatch-retest-v1-review.md`：v1 回收为需要修改。
- `handoffs/2026-05-30-workflow-dispatch-timeout-diagnosis-and-retry-prep-v1-review.md`：v2 retry work item 已准备完成。
- 真实 workflow state 中 `state-closure-retest-v2` 为 `ready_to_dispatch`，active binding 指向 thread `019e7738-5e29-74e0-a22f-5c2481b64c38`。

大白话：

上次派发是真的跑了，但 300 秒超时，README 没写成。现在换一个新的 v2 工作项重试，旧超时工作项不回滚、不复用。

## 薄弱点

- 这一步会真实执行 `codex exec resume`，会写 `/Users/yoyi/.codex`。
- 这一步会修改测试项目 README。
- 这一步会写真实 workflow state。
- 上次超时根因仍不确定；v2 只是降低变量后重试，不能保证成功。
- 这仍然只是极小复测，不证明复杂业务自动编排完成。

## 目标

通过用户审核业务派发路径，对测试项目 README 追加复测标记：

- 目标文件：`/Users/yoyi/codex-workflow-mario-test/README.md`
- 目标追加行：`Workflow dispatch state closure retest passed.`
- work item id：`workflow:users-yoyi-codex-workflow-mario-test:default:state-closure-retest-v2`
- 派发方式：`prompt_kind = user_reviewed_instruction`
- 执行目录：`/Users/yoyi/codex-workflow-mario-test`
- 沙箱：`workspace-write`
- timeout：600 秒
- 最大重试：0

核心验收：

- README 追加目标行。
- `index.html`、`styles.css`、`game.js` hash 不变。
- completed dispatch `exit_code=0`。
- v2 work item 进入 `ready_for_review`。
- v2 work item current node 进入 review。
- 实际派发节点 codex-dev 不再是 `running`，应收口为 `ready_for_review`。
- 真实 workflow state 写 dispatch / execution control / execution attempt / audit。

## 非目标

- 不复用旧 `state-closure-retest` work item。
- 不把旧 `timed_out` work item 改回 `ready_to_dispatch`。
- 不修改 `index.html`、`styles.css`、`game.js`。
- 不修改 `/Users/yoyi/gameai/agent world`。
- 不读取完整 transcript。
- 不读取 rollout JSONL 正文。
- 不读取 `auth.json`、`.env`、密钥、token、授权文件。
- 不运行 harness。
- 不联网安装依赖。
- 不做复杂业务自动编排。

## 前置要求

执行前必须只读确认：

- `/Users/yoyi/codex-workflow-mario-test/README.md` 存在。
- 目标行 `Workflow dispatch state closure retest passed.` 尚不存在；如果已存在，停止并回传，不重复追加。
- `workflow:users-yoyi-codex-workflow-mario-test:default:state-closure-retest-v2` 存在且为 `ready_to_dispatch`。
- v2 work item 的 `assigned_role_id` 为 `codex-dev`。
- v2 active binding 存在，node id 为 `workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev`。
- binding thread 为 `019e7738-5e29-74e0-a22f-5c2481b64c38`。
- thread 在 `codex-index.json` 中，`project_root=/Users/yoyi/codex-workflow-mario-test`，rollout 存在。
- 旧 `state-closure-retest` work item 仍为 `timed_out`，不得回滚。

## 必须先获得用户明确批准

执行真实派发前必须让用户明确同意：

- 执行真实 `codex exec resume`。
- 写 `/Users/yoyi/.codex`。
- 修改 `/Users/yoyi/codex-workflow-mario-test/README.md`。
- 写真实 workflow state。

没有明确批准，只能做只读前置检查，不得派发。

## 允许读取

允许读取：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/tasks/2026-05-30-workflow-state-closure-real-dispatch-retest-v2.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-dispatch-timeout-diagnosis-and-retry-prep-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-dispatch-timeout-diagnosis-and-retry-prep-v1-result.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-dispatch-timeout-diagnosis-and-retry-prep-v1-review.md`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `/Users/yoyi/codex-workflow-mario-test/README.md`
- `/Users/yoyi/codex-workflow-mario-test/index.html`
- `/Users/yoyi/codex-workflow-mario-test/styles.css`
- `/Users/yoyi/codex-workflow-mario-test/game.js`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json`
- 真实 workflow state 的必要结构：
  - `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件内容
- 完整 transcript 正文
- rollout JSONL 正文

## 允许写入

用户明确批准后，允许写：

- `/Users/yoyi/codex-workflow-mario-test/README.md`
- `/Users/yoyi/.codex`，通过 `codex exec resume`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-state-closure-real-dispatch-retest-v2.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-state-closure-real-dispatch-retest-v2-result.md`

允许更新：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`

## 禁止事项

- 禁止未获用户确认就执行 `codex exec resume`。
- 禁止执行新的 `codex exec`。
- 禁止修改允许范围外文件。
- 禁止读取敏感文件和完整 transcript。
- 禁止读取 rollout JSONL 正文。
- 禁止运行 harness。
- 禁止联网安装依赖。
- 禁止使用 `--dangerously-bypass-approvals-and-sandbox`。
- 禁止把复测说成复杂业务自动编排完成。
- 禁止在 shell 双引号里写未转义反引号模式；搜索包含反引号文本时必须使用单引号或 `rg -F`。

## 建议用户审核业务指令

```json
{
  "instruction_id": "user-reviewed-instruction:state-closure-retest-v2",
  "summary": "向测试项目 README 追加一行状态收口 v2 复测标记。",
  "objective": "验证修复后的工作流状态收口路径在真实 completed 派发后不会让 codex-dev 节点残留 running。",
  "execution_cwd": "/Users/yoyi/codex-workflow-mario-test",
  "sandbox_mode": "workspace-write",
  "allowed_write_roots": [
    "/Users/yoyi/codex-workflow-mario-test"
  ],
  "allowed_reads": [
    "/Users/yoyi/codex-workflow-mario-test/README.md"
  ],
  "allowed_writes": [
    "/Users/yoyi/codex-workflow-mario-test/README.md"
  ],
  "forbidden_actions": [
    "不读取 auth.json、.env、密钥、token、授权文件",
    "不读取完整 transcript",
    "不读取 rollout JSONL 正文",
    "不修改 index.html、styles.css、game.js",
    "不修改 /Users/yoyi/gameai/agent world",
    "不运行 harness",
    "不联网安装依赖"
  ],
  "timeout_seconds": 600,
  "max_retries": 0,
  "required_return": [
    "薄弱点",
    "是否修改 README",
    "是否修改允许范围外文件",
    "是否读取敏感文件",
    "最终文件摘要",
    "自检结果"
  ]
}
```

## 建议真实派发 prompt

保持短指令：

```text
请只做这一件事：在 /Users/yoyi/codex-workflow-mario-test/README.md 末尾追加一行：
Workflow dispatch state closure retest passed.

边界：
- 不修改 index.html、styles.css、game.js。
- 不读取 auth.json、.env、密钥、token、授权文件。
- 不读取完整 transcript 或 rollout JSONL。
- 不运行 harness。
- 不联网安装依赖。

完成后用最短文字回传：README_UPDATED_STATE_CLOSURE_RETEST_V2
```

## 验收标准

必须满足：

- README 追加目标行。
- `index.html`、`styles.css`、`game.js` hash 不变。
- completed dispatch `exit_code=0`。
- v2 work item state 为 `ready_for_review`。
- v2 work item current node 为 review。
- codex-dev node state 不是 `running`，应为 `ready_for_review`。
- 写入 execution control / execution attempt。
- 写入 audit event。
- 不保存完整 transcript。
- 不读取敏感文件。

如果超时：

- v2 work item 应收口为 `timed_out`。
- codex-dev node 应收口为 `timed_out`，不能残留 `running`。
- execution control / attempt 必须记录 `timeout`。
- README 若未修改，必须如实回传未修改。

## 建议验证

只读复核：

```bash
rg -n -F 'Workflow dispatch state closure retest passed.' /Users/yoyi/codex-workflow-mario-test/README.md
shasum -a 256 /Users/yoyi/codex-workflow-mario-test/README.md /Users/yoyi/codex-workflow-mario-test/index.html /Users/yoyi/codex-workflow-mario-test/styles.css /Users/yoyi/codex-workflow-mario-test/game.js
/Users/yoyi/miniconda3/bin/python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json
rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md
```

workflow state 摘要复核：

- v2 work item state。
- v2 dispatch state / exit code。
- codex-dev node state。
- execution control / attempt。
- audit event。
- 旧 `state-closure-retest` 是否仍为 `timed_out`。

## 必须回传

回传必须包含：

1. 薄弱点。
2. 是否获得用户明确批准。
3. 是否执行真实 `codex exec resume`。
4. 是否写 `/Users/yoyi/.codex`。
5. 是否写真实 workflow state。
6. 是否修改 README。
7. 是否修改允许范围外文件。
8. 是否读取敏感文件或完整 transcript。
9. completed 后 codex-dev node 是否仍为 `running`。
10. 如果超时，codex-dev node 是否收口为 `timed_out`。
11. workflow state 写入字段类型和 audit id。
12. 新增 evidence / handoff。
13. 验证命令和结果。

## 总指导回收重点

总指导回收时必须判断：

- 新代码路径是否真实避免 codex-dev 残留 `running`。
- 是否只修改了允许范围内 README。
- 是否没有读取敏感文件或完整 transcript。
- 是否把 completed 成功和 timed_out 失败区分清楚。
- 是否仍然不能把本轮说成复杂业务自动编排完成。

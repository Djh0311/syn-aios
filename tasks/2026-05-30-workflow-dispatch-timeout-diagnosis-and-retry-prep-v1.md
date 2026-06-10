# 任务包：工作流派发超时诊断与重试准备 v1

## 任务名

工作流派发超时诊断与重试准备 v1。

## 所属开发线

桌面应用线 / Codex 会话线 / 工作流状态线。

总指导线回收。

## 当前判断

上一轮真实状态收口复测没有通过。

依据：

- `handoffs/2026-05-30-workflow-state-closure-real-dispatch-retest-v1-review.md` 回收为需要修改。
- README 目标行 `Workflow dispatch state closure retest passed.` 没有写入。
- retest work item 和 codex-dev 节点均已收口为 `timed_out`。

大白话：

上次不是代码继续挂在“执行中”，而是派发出去以后 300 秒没回来。现在不能拿已经超时的旧工作项硬推，要先查清楚卡在哪里，再准备一个新的合法重试工作项。

## 薄弱点

- 当前只知道结果是 `timed_out`，还不知道根因。
- 不能断言是 Codex 会话问题、插件启动问题、沙箱参数问题、runner 等待策略问题，还是 prompt 太复杂。
- 不能直接复用旧 work item。依据：旧 work item 已经是 `timed_out`，不是 `ready_to_dispatch`。
- 本任务不执行真实重试派发，所以不能证明 README 会写成功。
- 如果准备新的 retry work item，会写真实 workflow state，必须单独获得用户明确批准。

## 目标

分两段完成。

第一段：只读诊断。

- 复核上次超时派发的 evidence / handoff / review。
- 复核真实 workflow state 中 retest work item、dispatch、execution control、execution attempt、codex-dev node 的摘要。
- 复核 README 目标行仍不存在。
- 复核 `index.html`、`styles.css`、`game.js` hash 未变。
- 复核目标 thread 仍在 `codex-index.json`，project root 仍为 `/Users/yoyi/codex-workflow-mario-test`，rollout 存在。
- 给出初步根因分类，必须标明依据和不确定项。

第二段：准备合法重试状态。

只有用户明确批准写真实 workflow state 后，才允许创建新的 retry work item 和 active binding。

建议新 work item：

- work item id：`workflow:users-yoyi-codex-workflow-mario-test:default:state-closure-retest-v2`
- workflow id：`workflow:users-yoyi-codex-workflow-mario-test:default`
- assigned role：`codex-dev`
- current node：`workflow:users-yoyi-codex-workflow-mario-test:default:node:director`
- state：`ready_to_dispatch`
- active binding thread：`019e7738-5e29-74e0-a22f-5c2481b64c38`

重试目标仍是：

- 目标文件：`/Users/yoyi/codex-workflow-mario-test/README.md`
- 目标追加行：`Workflow dispatch state closure retest passed.`

## 非目标

- 不执行 `codex exec resume`。
- 不写 `/Users/yoyi/.codex`。
- 不修改 README。
- 不修改 `index.html`、`styles.css`、`game.js`。
- 不修改 `/Users/yoyi/gameai/agent world`。
- 不读取完整 transcript。
- 不读取 rollout JSONL 正文。
- 不读取 `auth.json`、`.env`、密钥、token、授权文件。
- 不运行 harness。
- 不联网安装依赖。
- 不把本任务说成真实重试派发成功。
- 不把本任务说成复杂业务自动编排完成。

## 前置检查

必须先只读确认：

- `/Users/yoyi/codex-workflow-mario-test/README.md` 存在。
- 目标行 `Workflow dispatch state closure retest passed.` 尚不存在；如果已存在，停止并回传，不准备新重试。
- 当前旧 work item `workflow:users-yoyi-codex-workflow-mario-test:default:state-closure-retest` 为 `timed_out`。
- codex-dev node 不为 `running`。
- 目标 thread `019e7738-5e29-74e0-a22f-5c2481b64c38` 在索引中，且 `project_root=/Users/yoyi/codex-workflow-mario-test`。
- 新 work item id `workflow:users-yoyi-codex-workflow-mario-test:default:state-closure-retest-v2` 尚不存在；如果已存在，停止并回传。

## 必须先获得用户明确批准

准备新 retry work item 前必须让用户明确同意：

- 写真实 workflow state。
- 写 workflow state 备份。
- 追加 audit event。
- 新建 retry work item。
- 新建 active binding。

没有明确批准，只能做只读诊断和写 evidence / handoff，不得写真实 workflow state。

本任务即使获得批准，也禁止执行真实 `codex exec resume`。

## 允许读取

允许读取：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/tasks/2026-05-30-workflow-state-closure-real-dispatch-retest-v1.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-state-closure-real-dispatch-retest-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-state-closure-real-dispatch-retest-v1-result.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-state-closure-real-dispatch-retest-v1-review.md`
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

无需额外批准可写：

- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-dispatch-timeout-diagnosis-and-retry-prep-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-dispatch-timeout-diagnosis-and-retry-prep-v1-result.md`

允许更新：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`

用户明确批准后才允许写：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`

## 禁止事项

- 禁止执行 `codex exec resume`。
- 禁止执行任何新的 `codex exec`。
- 禁止写 `/Users/yoyi/.codex`。
- 禁止修改 README。
- 禁止修改允许范围外文件。
- 禁止读取敏感文件和完整 transcript。
- 禁止读取 rollout JSONL 正文。
- 禁止运行 harness。
- 禁止联网安装依赖。
- 禁止使用 `--dangerously-bypass-approvals-and-sandbox`。
- 禁止直接把旧 `timed_out` work item 改回 `ready_to_dispatch`，除非后续有单独决策明确允许这种状态回滚策略。
- 禁止在 shell 双引号里写未转义反引号模式；搜索包含反引号文本时必须使用单引号或 `rg -F`。

## 诊断输出要求

必须输出：

1. 上次超时的事实链。
2. 已知原因。
3. 未知原因。
4. 不能断言的内容。
5. 下一轮重试应该调整什么。
6. 是否建议增加 timeout。
7. 是否建议简化 prompt。
8. 是否建议把 cwd 固定为 `/Users/yoyi/codex-workflow-mario-test`。
9. 是否建议继续使用同一个 thread。
10. 是否需要新建 thread。

每个判断必须标明依据；没有依据就写“不确定”。

## 建议重试策略

初步建议，不是执行指令：

- 使用新 work item，不复用旧 `timed_out` work item。
- 继续使用同一个测试 thread，除非只读检查发现 thread / rollout / project root 异常。
- 下轮真实派发 prompt 要比上轮更短，只要求追加一行 README，并回传最小结果。
- 下轮 timeout 可从 300 秒提高到 600 秒，但必须在任务包里说明理由和风险。
- 下轮执行 cwd 优先使用 `/Users/yoyi/codex-workflow-mario-test`，避免 `/Users/yoyi` 写入范围过宽。

## 建议验证

只读复核：

```bash
rg -n -F 'Workflow dispatch state closure retest passed.' /Users/yoyi/codex-workflow-mario-test/README.md
shasum -a 256 /Users/yoyi/codex-workflow-mario-test/README.md /Users/yoyi/codex-workflow-mario-test/index.html /Users/yoyi/codex-workflow-mario-test/styles.css /Users/yoyi/codex-workflow-mario-test/game.js
/Users/yoyi/miniconda3/bin/python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json
rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md
```

workflow state 摘要复核：

- 旧 retest work item state。
- 新 v2 work item 是否存在。
- codex-dev node state。
- 旧 dispatch / control / attempt 摘要。
- 目标 thread binding 状态。
- 新 audit event 是否存在。

## 必须回传

回传必须包含：

1. 薄弱点。
2. 是否执行 `codex exec` 或 `codex exec resume`。
3. 是否写 `/Users/yoyi/.codex`。
4. 是否写真实 workflow state。
5. 是否修改 README。
6. 是否读取敏感文件或完整 transcript。
7. 超时诊断结论和依据。
8. 新 work item 是否创建。
9. 新 binding 是否创建。
10. 备份路径和 audit id。
11. 新增 evidence / handoff。
12. 验证命令和结果。

## 总指导回收重点

总指导回收时必须判断：

- 是否把超时失败和成功完成区分清楚。
- 是否没有直接复用旧 `timed_out` work item。
- 是否没有执行新的真实派发。
- 如果写了真实 workflow state，是否有备份和 audit。
- 是否仍然不能把本轮说成复杂业务自动编排完成。

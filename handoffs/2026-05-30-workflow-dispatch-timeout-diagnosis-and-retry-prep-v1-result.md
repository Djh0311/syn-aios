# Handoff：工作流派发超时诊断与重试准备 v1

## 结论

本轮只读诊断已完成。用户回复“批准”后，已准备新的 retry work item。

上一轮失败的直接原因是 `timeout_seconds=300` 到期后未拿到有效完成结果。当前只能确认超时事实和状态收口结果，不能确认具体卡点。

## 薄弱点

- 仍不知道真实卡点在 prompt 投递、模型执行、插件启动、MCP、权限等待、输出回收，还是 runner 等待策略。
- 本轮没有读取完整 transcript，不能还原 Codex 会话内部全过程。
- 上轮出现插件同步、GitHub rate limit、MCP warning，但这些只能列为可能相关，不能定为根因。
- 没有执行新的真实重试，所以不能证明 README 追加会成功。
- 新 retry work item 已创建，但真实派发仍需要另行明确批准，因为会执行 `codex exec resume`、写 `/Users/yoyi/.codex` 并修改 README。

## 边界

- 是否执行 `codex exec`：否。
- 是否执行 `codex exec resume`：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否写真实 workflow state：是。
- 是否修改 README：否。
- 是否读取敏感文件或完整 transcript：否。
- 是否读取 rollout JSONL 正文：否。
- 是否运行 harness：否。

## 诊断摘要

- 旧 work item：`workflow:users-yoyi-codex-workflow-mario-test:default:state-closure-retest`
- 旧 work item state：`timed_out`
- codex-dev node：`timed_out`
- dispatch：`failed`
- execution control：`timed_out`
- execution attempt：`timed_out`
- timeout：`300` 秒
- README 目标行：不存在
- 新 work item：`workflow:users-yoyi-codex-workflow-mario-test:default:state-closure-retest-v2`，状态为 `ready_to_dispatch`
- target thread：`019e7738-5e29-74e0-a22f-5c2481b64c38`
- thread project root：`/Users/yoyi/codex-workflow-mario-test`
- rollout：存在

## 判断

已知原因：

- 300 秒超时。依据：workflow execution control / attempt。
- 业务结果未完成。依据：README 目标行无命中，last-message 为空。

未知原因：

- 不确定是不是插件同步、GitHub rate limit、MCP warning、权限等待或 runner 策略导致。

不能断言：

- 不能断言 thread 损坏。
- 不能断言 README 权限失败。
- 不能断言 sandbox 是唯一原因。
- 不能断言下次 600 秒一定成功。

## 下一轮建议

- 新建 `state-closure-retest-v2` work item，不复用旧 `timed_out` work item。
- 继续使用 thread `019e7738-5e29-74e0-a22f-5c2481b64c38`，除非下一轮只读检查发现 thread / rollout 异常。
- 不需要新建 thread；当前没有证据支持必须换 thread。
- cwd 固定为 `/Users/yoyi/codex-workflow-mario-test`。
- prompt 简化到只追加 README 一行并回传最小结果。
- timeout 可提高到 600 秒，理由是上轮 300 秒超时且存在插件/同步 warning；风险是失败反馈变慢。

## 写入情况

- 新 work item 是否创建：是。
- 新 binding 是否创建：是。
- 备份路径：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780131018244.json`
- audit id：
  - `audit:workflow-state-closure-retest-v2-work-item-ready:1780131018244`
  - `audit:workflow-state-closure-retest-v2-session-bound:1780131018244`
  - `audit:workflow-state-closure-retest-v2-node-ready:1780131018244`

写入字段类型：

- `work_items[]`
- `workflow_node_session_bindings[]`
- `nodes[]` 中 codex-dev 节点摘要状态
- `audit_events[]`
- 顶层 `updated_at`

新 work item：

- work item id：`workflow:users-yoyi-codex-workflow-mario-test:default:state-closure-retest-v2`
- state：`ready_to_dispatch`
- current node：`workflow:users-yoyi-codex-workflow-mario-test:default:node:director`
- assigned role：`codex-dev`

新 binding：

- binding id：`binding:workflow-users-yoyi-codex-workflow-mario-test-default:workflow-users-yoyi-codex-workflow-mario-test-default-node-codex-dev:workflow-users-yoyi-codex-workflow-mario-test-default-state-closure-retest-v2`
- lifecycle：`active`
- node id：`workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev`
- native thread id：`019e7738-5e29-74e0-a22f-5c2481b64c38`
- rollout_exists：true

## 新增 evidence

- `evidence/2026-05-30-workflow-dispatch-timeout-diagnosis-and-retry-prep-v1.md`

## 验证命令和结果

- `rg -n -F 'Workflow dispatch state closure retest passed.' /Users/yoyi/codex-workflow-mario-test/README.md`：无命中。
- `shasum -a 256 /Users/yoyi/codex-workflow-mario-test/README.md /Users/yoyi/codex-workflow-mario-test/index.html /Users/yoyi/codex-workflow-mario-test/styles.css /Users/yoyi/codex-workflow-mario-test/game.js`：README / index / styles / game hash 与上轮记录一致。
- `/Users/yoyi/miniconda3/bin/python3 /Users/yoyi/workspace/product-line/prototypes/index-kernel/build_index.py --check /Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json`：`validation_ok`。
- workflow state 摘要复核：旧 work item / execution control / attempt 均保持 `timed_out`；新 v2 work item 为 `ready_to_dispatch`；新 v2 binding 为 `active`；codex-dev node 为 `ready_to_dispatch`。

## 待用户确认

若要继续真实重试派发，需要再次明确批准，因为下一步会执行真实 `codex exec resume`、写 `/Users/yoyi/.codex`、写真实 workflow state，并修改 README。

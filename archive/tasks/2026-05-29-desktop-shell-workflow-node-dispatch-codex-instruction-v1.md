# 任务包：桌面壳工作流节点派发 Codex 指令 v1

## 所属开发线

桌面应用线。

## 关联口径来源

- `product-line/decisions/2026-05-29-codex-session-plan-retained-workflow-first.md`
- `product-line/decisions/2026-05-29-codex-session-workflow-route-correction.md`
- `product-line/decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`
- `product-line/tasks/2026-05-29-desktop-shell-workflow-orchestration-min-loop-v1.md`
- `product-line/tasks/2026-05-29-desktop-shell-workflow-node-session-binding-v1.md`
- `product-line/handoffs/2026-05-29-codex-bound-session-dispatch-probe-v1-result.md`

## 后续验证

本任务完成后另派验证线。验证线不是本任务的共同执行线。

## 背景

当前已经完成：

- 工作流能做状态流转。
- 工作流节点能绑定已有 Codex 会话。
- Codex 会话线已证明无业务测试会话可以通过 `codex exec resume` 接收第二轮 prompt、等待完成、写最终回复，并被 transcript reader 读回。

但这些能力还没有接进桌面工作流。当前工作台还不能从工作流节点点击“派发”，让绑定会话接收一条受控指令并把结果写回工作台状态。

## 薄弱点

- 这轮仍不等于真实业务自动开发。依据：已验证的是无业务短 prompt，不覆盖长任务、工具调用、权限确认和失败重试。
- 真实派发会写 `/Users/yoyi/.codex`。依据：`codex exec resume` 会向目标会话追加消息和结果。
- 如果不设计派发记录，后续总指导无法知道哪次派发对应哪次回传。
- 如果直接允许自由文本派发真实业务任务，风险过高；v1 必须先做受控指令和用户确认。

## 已知、未知和假设

已知：

- `codex exec resume --skip-git-repo-check --json --output-last-message <file> <thread_id> <prompt>` 已在无业务测试会话上跑通。
- transcript reader 能读回第二轮目标文本。
- 工作流节点绑定状态中已经有 `native_thread_id`。
- 工作台状态可写 `work_items[]`、`audit_events[]` 和扩展字段。

未知：

- 真实业务长任务是否稳定。
- 工具调用、多步修改、权限确认会怎样反映到 JSON 事件流。
- 派发失败、超时、半完成时应如何恢复。
- 回收判断应由总指导自动做还是先让用户审核。

假设：

- v1 只做“单节点、单会话、单次派发”。
- v1 不做并发。
- v1 只支持绑定已有会话。
- v1 不新建业务会话。
- v1 不直接跑真实开发长任务。
- v1 派发前必须用户确认。

## 目标

把“工作流节点绑定会话”推进到“工作流节点可以派发一条受控 Codex 指令”：

1. 用户在工作流节点上点击“派发指令”。
2. 系统根据当前工作项生成一条受控 prompt 预览。
3. 用户确认后，后端调用绑定会话的 `codex exec resume`。
4. 系统等待命令完成。
5. 系统保存最终回复路径或摘要。
6. 系统用 transcript reader 读回本轮结果统计。
7. 工作项状态从 `ready_to_dispatch` 推进到 `running`，完成后推进到 `ready_for_review`。
8. 写入派发记录和审计事件。

大白话目标：

让工作流节点真的能“给它绑定的 Codex 会话发一条任务”，然后把回复带回工作台。

## 非目标

- 不做真实业务长任务自动执行。
- 不做并发调度。
- 不做失败重试自动恢复。
- 不做总指导自动回收判断。
- 不运行 harness。
- 不创建新 Codex 业务会话。
- 不删除、移动、归档 Codex 会话。
- 不读取 `auth.json`、`.env`、密钥或授权文件。
- 不保存完整 transcript 到工作台状态。
- 不把工作流派发做成任务包文件管理。

## 受控派发模式

v1 必须提供两种模式：

### 安全测试模式

默认用于验证实现。

prompt 固定为无业务内容，例如：

```text
请只回复这一句：WORKFLOW_NODE_DISPATCH_OK_2026_05_29
```

用途：

- 验证桌面后端调用链。
- 验证绑定会话接收指令。
- 验证最终回复文件和 transcript 读回。
- 验证状态写回和审计。

### 用户审核模式

显示派发 prompt 预览，由用户确认后才允许执行。

prompt 必须来自工作项字段和工作流上下文，不能补编业务内容。

v1 可以先只支持非常短的审核后指令，例如：

- 目标
- 允许读取
- 允许写入
- 禁止事项
- 完成后必须回传

如果字段缺失，必须显示缺口并阻止真实业务派发。

## 建议数据模型

在工作台状态里新增或复用派发记录。

建议字段：

- `dispatch_id`
- `project_id`
- `workflow_id`
- `node_id`
- `work_item_id`
- `binding_id`
- `native_thread_id`
- `prompt_preview`
- `prompt_kind`
- `state`
- `started_at_ms`
- `ended_at_ms`
- `exit_code`
- `last_message_path`
- `last_message_summary`
- `transcript_event_count`
- `transcript_target_hits`
- `warnings`

建议 `prompt_kind`：

- `safe_probe`
- `user_reviewed_instruction`

建议 `state`：

- `prepared`
- `running`
- `completed`
- `failed`
- `blocked`

建议审计事件：

- `workflow_node_dispatch_prepared`
- `workflow_node_dispatch_started`
- `workflow_node_dispatch_completed`
- `workflow_node_dispatch_failed`
- `workflow_node_dispatch_readback_completed`

## 建议后端命令

在 Tauri Rust 后端新增：

- `prepare_workflow_node_dispatch`
- `execute_workflow_node_dispatch`
- `read_workflow_node_dispatch_result`

要求：

- 非索引项目拒绝。
- 缺 workflow 拒绝。
- 缺 work item 拒绝。
- 缺节点绑定会话拒绝。
- 绑定会话不在索引中时拒绝或 blocked。
- 真实业务 prompt 缺字段时拒绝。
- 派发前必须写入 prepared 记录或确认上下文。
- 执行时必须走确认弹层。
- 执行命令只能调用 `codex exec resume`，不能调用 `codex fork`。
- 执行输出写入 `/tmp` 或工作台自己的 dispatch 临时目录。
- 执行完成后用 transcript reader 读回统计。
- 不保存完整 transcript 到工作台状态。
- 写状态前备份，临时文件原子替换，写后校验。

## 建议前端改动

项目工作流页：

- 当前节点详情区增加“派发指令”区域。
- 显示绑定会话。
- 显示当前工作项状态。
- 显示 prompt 预览。
- 显示缺字段 warning。
- 显示最近一次派发结果。
- 提供“安全测试派发”按钮。
- 提供“审核后派发”入口，但缺字段时禁用。

确认弹层必须说明：

- 将向绑定的 Codex 会话发送一条消息。
- 会写 `/Users/yoyi/.codex`。
- 会写工作台自己的 workflow state。
- 不会读取授权或密钥。
- 不会运行 harness。
- 不会删除、移动、归档会话。

派发完成后：

- 显示最终回复摘要。
- 显示 transcript 读回统计。
- 工作项进入 `ready_for_review`。
- 最近审计事件显示派发和读回。

## 允许读取

允许读取项目内：

- `product-line/decisions/2026-05-29-codex-session-plan-retained-workflow-first.md`
- `product-line/decisions/2026-05-29-codex-session-workflow-route-correction.md`
- `product-line/decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`
- `product-line/handoffs/2026-05-29-codex-bound-session-dispatch-probe-v1-result.md`
- `product-line/prototypes/productized-desktop-shell/src/`
- `product-line/prototypes/productized-desktop-shell/src-tauri/`
- `product-line/prototypes/productized-desktop-shell/tests/`
- `product-line/prototypes/index-kernel/transcript_reader.py`
- `product-line/prototypes/index-kernel/build_index.py`
- `product-line/prototypes/index-kernel/codex-index.json`

允许读取工作台状态文件必要结构：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

执行受控派发时允许只读：

- `/Users/yoyi/.codex/state_5.sqlite` 的统计和线程元数据。
- `/Users/yoyi/.codex/sessions/` 中目标绑定会话的 JSONL。
- `/Users/yoyi/.codex/archived_sessions/` 的文件清单。

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 授权文件内容
- 密钥文件内容
- 与当前派发无关的业务会话正文

## 允许写入

允许写入项目内：

- `product-line/prototypes/productized-desktop-shell/src/`
- `product-line/prototypes/productized-desktop-shell/src-tauri/`
- `product-line/prototypes/productized-desktop-shell/tests/`
- `product-line/evidence/2026-05-29-desktop-shell-workflow-node-dispatch-codex-instruction-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-workflow-node-dispatch-codex-instruction-v1-result.md`

允许在用户通过 UI 确认时写入：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`
- 绑定目标 Codex 测试会话的 rollout 追加记录

允许写临时输出：

- `/tmp/codex-workflow-node-dispatch-v1/`

## 禁止事项

- 禁止未确认就执行派发。
- 禁止运行 `codex fork`。
- 禁止向未绑定会话派发。
- 禁止向真实业务会话发送未审核指令。
- 禁止读取 `auth.json`、`.env`、授权文件、密钥文件。
- 禁止保存完整 transcript 到工作台状态。
- 禁止把完整 transcript 或完整事件流写进 evidence / handoff。
- 禁止运行 harness。
- 禁止删除、移动、归档 Codex 会话。
- 禁止绕过工作台状态审计。

## 验收标准

必须满足：

- 无绑定会话时不能派发。
- 能生成 safe probe prompt 预览。
- 派发前必须出现确认弹层。
- 确认后能调用绑定会话的 `codex exec resume`。
- 能等待完成并拿到最终回复。
- 能用 transcript reader 读回本轮结果统计。
- 工作项状态能从 `ready_to_dispatch` 进入 `running`，完成后进入 `ready_for_review`。
- 派发记录写入工作台状态。
- 审计事件写入工作台状态。
- evidence / handoff 不包含完整 transcript、密钥或授权内容。

验证命令：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --offline
```

如果执行真实 safe probe 派发，必须明确记录：

- 使用的绑定测试会话。
- 用户确认方式。
- 是否写了 `/Users/yoyi/.codex`。
- 是否额外产生 guardian / auto-review 线程。
- transcript 读回统计。

## 必须回传

回传时必须说明：

1. 薄弱点。
2. 做了什么。
3. 改了哪些文件。
4. 新增了哪些 evidence / handoff。
5. 是否执行真实派发。
6. 如果执行，使用哪个 thread id。
7. 是否写了 `/Users/yoyi/.codex`。
8. 是否写了真实 workflow state。
9. 写入了哪些状态字段，不要打印完整状态正文。
10. 是否读取授权、密钥、`.env`。
11. 是否触碰真实业务会话。
12. 测试命令和结果。
13. 当前距离真实业务自动工作流还缺什么。

## 总指导回收重点

回收时重点看：

- 是否真的从工作台工作流节点触发派发。
- 是否只向绑定会话派发。
- 是否有用户确认和审计。
- 是否完成最终回复和 transcript 读回。
- 是否没有污染真实业务会话。
- 是否没有把 safe probe 包装成真实业务自动执行。


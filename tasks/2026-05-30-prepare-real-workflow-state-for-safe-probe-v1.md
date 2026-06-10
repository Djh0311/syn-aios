# 任务包：准备真实 workflow state 以便 safe probe v1

## 任务名

准备真实 workflow state 以便 safe probe v1。

## 所属开发线

桌面应用线 / 工作流运行线。

## 当前判断

真实 safe probe 派发被正确阻塞，不是因为代码路径缺失，而是因为真实 workflow state 没有满足前置条件。

当前必须先准备状态，再派发。

依据：

- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1-result.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1-review.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1.md`

## 薄弱点

- 当前真实 workflow state 中 `workflow_node_session_bindings[]` 数量是 `0`，无法派发。
- 当前真实 workflow state 中唯一工作项状态是 `draft`，不是 `ready_to_dispatch`。
- 当前真实 workflow state 指向 `/Users/yoyi/gameai/agent world`，不是当前 `product-line`。必须确认这是不是本轮 safe probe 的目标项目。
- 绑定测试会话和推进工作项都会写真实 workflow state，不能默认执行。
- 本任务如果顺手执行 safe probe，会越界；派发必须留给下一任务。

## 背景

上一轮 safe probe 任务按规则停止：

- 没有绑定测试会话。
- 工作项还在 `draft`。
- 没有获得真实派发批准。
- 没有执行 `codex exec resume`。

这说明当前缺的是“派发前状态准备”：

1. 选定目标项目。
2. 选定目标工作项。
3. 选定目标工作流节点。
4. 选定目标 Codex 测试会话。
5. 写入节点会话绑定。
6. 把工作项推进到 `ready_to_dispatch`。

## 目标

在用户明确确认后，把真实 workflow state 准备到可以执行 safe probe 的状态：

1. 只读展示当前真实 workflow state 摘要。
2. 只读列出当前可选的 Codex 候选会话摘要。
3. 明确目标项目、目标工作项、目标节点和目标测试会话。
4. 用户确认后写入 `workflow_node_session_bindings[]`。
5. 用户确认后将目标工作项从 `draft` 推进到 `ready_to_dispatch`。
6. 写入审计事件。
7. 输出 evidence 和 handoff。

大白话目标：

把“可以派发”的账本准备好，但不真正发消息。

## 非目标

- 不执行 `codex exec resume`。
- 不发送 safe probe。
- 不写 `/Users/yoyi/.codex`。
- 不触碰真实业务会话。
- 不读取完整会话正文。
- 不读取 `auth.json`、`.env`、密钥、token 或授权文件内容。
- 不运行 harness。
- 不创建新 Codex 会话。
- 不删除、移动、归档 Codex 会话。
- 不把状态准备说成真实派发完成。

## 已知、未知和假设

已知：

- 真实 workflow state 文件存在。
- 当前真实 workflow state 有 1 个项目、1 个 workflow、7 个节点、1 个 work item。
- 当前唯一 work item 是 `draft`。
- 当前没有节点会话绑定。
- 后端已有 `bind_workflow_node_codex_session`。
- 后端已有 `update_work_item_state`，并允许 `draft -> ready_to_dispatch`。

未知：

- 用户是否要以 `/Users/yoyi/gameai/agent world` 为本次 safe probe 目标项目。
- 应绑定哪一条 Codex 测试会话。
- 当前索引里是否存在项目归属匹配的可用测试会话。
- 目标测试会话是否应绑定到工作项级别，还是节点级别。

假设：

- 优先绑定到 `codex-dev` 节点。
- 优先绑定到当前 work item 级别，而不是纯节点级别。
- 如果没有明确测试会话，停止并回传，不创建新会话。
- 如果用户没有明确确认写真实 workflow state，只做只读检查。

## 建议目标对象

当前只读复核看到：

- 项目路径：`/Users/yoyi/gameai/agent world`
- workflow：`workflow:users-yoyi-gameai-agent-world:default`
- 开发节点：`workflow:users-yoyi-gameai-agent-world:default:node:codex-dev`
- work item：`work-item:workflow:users-yoyi-gameai-agent-world:default:1780032043420`
- work item 当前状态：`draft`

这些只是当前事实，不等于用户已确认目标。

## 执行前确认

本任务包本身不构成写入授权。

执行真实 workflow state 写入前，必须再次获得用户明确批准。确认内容必须包含：

- 将写真实 workflow state。
- 将绑定指定 Codex 测试会话到指定工作流节点。
- 将指定工作项推进到 `ready_to_dispatch`。
- 不会执行 `codex exec resume`。
- 不会写 `/Users/yoyi/.codex`。
- 不会读取授权、密钥或 `.env`。
- 不会触碰真实业务会话。

## 允许读取

允许读取项目内：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/STAGE_PLAN.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1-result.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1-review.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1.md`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json`

允许读取真实 workflow state 必要结构：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

允许读取 Codex 索引中的会话摘要：

- 线程 id
- 标题
- 项目路径
- 更新时间
- rollout 是否存在
- 索引 warning

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件内容
- 完整会话正文
- 与目标测试会话选择无关的业务正文

## 允许写入

允许写入项目内交付物：

- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-prepare-real-workflow-state-for-safe-probe-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-prepare-real-workflow-state-for-safe-probe-v1-result.md`

用户明确批准后，允许写真实工作台状态：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`

## 禁止事项

- 禁止未获用户明确批准就写真实 workflow state。
- 禁止执行 `codex exec resume`。
- 禁止写 `/Users/yoyi/.codex`。
- 禁止发送任何消息到 Codex 会话。
- 禁止绑定真实业务会话作为测试会话。
- 禁止读取完整 transcript。
- 禁止读取授权、密钥或 `.env`。
- 禁止运行 harness。
- 禁止删除、移动、归档 Codex 会话。
- 禁止把索引推断当作用户确认事实。

## 实施要求

执行时按以下顺序：

1. 只读输出当前 workflow state 摘要，不打印完整状态。
2. 只读列出候选 Codex 会话摘要，不打印完整 transcript。
3. 判断是否存在明确测试会话。
4. 如果目标项目、工作项、节点或测试会话不明确，停止并请求用户确认。
5. 用户确认后，调用现有绑定路径写入节点会话绑定。
6. 用户确认后，调用现有状态推进路径将工作项推进到 `ready_to_dispatch`。
7. 写 evidence。
8. 写 handoff。

如果出现以下情况，必须停止：

- 找不到目标项目。
- 找不到目标 workflow。
- 找不到目标节点。
- 找不到目标 work item。
- 找不到明确测试会话。
- 用户没有批准写真实 workflow state。
- 写入后 schema 校验失败。

## 验收标准

必须满足：

- evidence 中记录写入前的结构摘要。
- evidence 中记录用户确认方式。
- workflow state 中存在 active `workflow_node_session_bindings[]`。
- 绑定对象是明确测试会话，不是真实业务会话。
- 目标 work item 状态为 `ready_to_dispatch`。
- 审计事件包含绑定和状态推进。
- 没有执行 `codex exec resume`。
- 没有写 `/Users/yoyi/.codex`。
- 没有读取授权、密钥、`.env`。
- 没有保存完整 transcript。

建议验证命令：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --offline
```

如果只做状态准备且未改代码，可以说明验证命令是否复用上一轮结果；但不能把未运行的命令写成已通过。

## 必须回传

开发线回传必须包含：

1. 薄弱点。
2. 做了什么。
3. 是否写了真实 workflow state。
4. 是否写了 `/Users/yoyi/.codex`。
5. 是否执行 `codex exec resume`。
6. 用户确认方式。
7. 目标项目路径。
8. 目标 workflow id。
9. 目标 node id。
10. 目标 work item id。
11. 目标 thread id。
12. 写入的字段类型，不打印完整状态正文。
13. 新增 evidence / handoff。
14. 测试命令和结果。
15. 是否已经满足下一步 safe probe 前置条件。

## 总指导回收动作

总指导回收时必须判断：

- 接受。
- 需要修改。
- 暂停。
- 废弃。

回收重点：

- 是否只准备状态，没有派发。
- 是否有明确用户确认。
- 是否绑定了正确测试会话。
- 是否把工作项推进到 `ready_to_dispatch`。
- 是否没有写 `/Users/yoyi/.codex`。
- 是否没有读取授权或密钥。

## 完成后的下一步

如果本任务通过，再重试：

- `tasks/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1.md`

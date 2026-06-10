# 任务包：准备 README smoke 测试 workflow state v2

## 任务名

准备 README smoke 测试 workflow state v2。

## 所属开发线

桌面应用线 / Codex 会话线。

总指导线回收。

## 当前判断

README smoke 仍不能直接执行。

依据：

- 真实 workflow state 目前只有 1 个 project、1 个 workflow、1 个 work item。
- 当前真实 workflow state 还没有 `/Users/yoyi/codex-workflow-mario-test` 的 project / workflow / work item / active binding。
- README smoke 目标行 `Workflow dispatch smoke passed.` 还没有写入。
- 现在已具备一个可绑定测试 thread：`019e7738-5e29-74e0-a22f-5c2481b64c38`。
- 索引中该 thread 的 `project_root=/Users/yoyi/codex-workflow-mario-test`，`rollout_exists=true`。

大白话：

现在不缺 Codex 测试会话了，缺的是把这个测试项目登记进真实工作流账本，并创建一个“可以派发 README smoke”的工作项。

## 薄弱点

- 本任务会写真实 workflow state，必须先获得用户明确批准。
- 本任务不执行 README 修改，只准备派发前置状态。
- 本任务不执行 `codex exec resume`，所以不能证明 README smoke 会成功。
- 如果写入结构和桌面壳现有 schema 不一致，后续派发会被后端拒绝，所以写前必须对照现有字段。
- 当前已有 `/Users/yoyi/gameai/agent world` 的 workflow 和 binding，不能误改或替换它。

## 已知

- 目标项目路径：`/Users/yoyi/codex-workflow-mario-test`
- 目标 README：`/Users/yoyi/codex-workflow-mario-test/README.md`
- README smoke 目标行：`Workflow dispatch smoke passed.`
- 绑定 thread id：`019e7738-5e29-74e0-a22f-5c2481b64c38`
- thread 标题：`请只回复这一句：WORKFLOW_MARIO_TEST_SESSION_READY_2026_05_30。不要读取、列出、修改任何文件，不要运行任何命令。`
- rollout 路径：`/Users/yoyi/.codex/sessions/2026/05/30/rollout-2026-05-30T12-50-43-019e7738-5e29-74e0-a22f-5c2481b64c38.jsonl`
- rollout 存在：是

## 未知

- 新 project / workflow / node / work item 的最终 id。执行时可以按现有 id 规则生成。
- 当前真实 workflow state 在执行前是否又被其他线修改。执行前必须重新只读检查。

## 假设

- 目标是准备 README smoke 的派发前置状态，不是直接执行 README smoke。
- 继续使用 v0 JSON workflow state。
- 新工作项初始状态应为 `ready_to_dispatch`。
- 绑定节点应使用 Codex dev 类节点，名称可以沿用现有节点模型，但 id 必须属于 mario test workflow。

## 目标

把真实 workflow state 准备成可以派发 README smoke 的状态：

1. 创建或登记 `/Users/yoyi/codex-workflow-mario-test` project。
2. 创建该项目默认 workflow。
3. 创建 Codex dev 节点。
4. 创建 README smoke work item。
5. work item 状态设为 `ready_to_dispatch`。
6. 创建 active binding：
   - node id 属于 mario test workflow。
   - work item id 属于 README smoke。
   - thread id 为 `019e7738-5e29-74e0-a22f-5c2481b64c38`。
   - 记录 `project_root=/Users/yoyi/codex-workflow-mario-test`。
7. 写入备份。
8. 写 audit events。
9. 只读复核写入结果。

## 非目标

- 不执行真实 `codex exec resume`。
- 不发送 README smoke 指令。
- 不执行任何新的 `codex exec`。
- 不写 `/Users/yoyi/.codex`。
- 不修改 `/Users/yoyi/codex-workflow-mario-test/README.md`。
- 不读取完整 transcript。
- 不读取 `auth.json`、`.env`、密钥、token、授权文件。
- 不运行 harness。
- 不删除、移动、归档任何 Codex 会话。
- 不改动 `/Users/yoyi/gameai/agent world` 现有 project / workflow / work item / binding，除非只追加全局审计事件。
- 不把“准备 workflow state”说成“README smoke 已完成”。

## 必须先获得用户明确批准

执行写入前，必须让用户明确同意：

- 写真实 workflow state。
- 写 workflow state 备份。
- 创建 `/Users/yoyi/codex-workflow-mario-test` project / workflow / work item / binding。
- 绑定 thread `019e7738-5e29-74e0-a22f-5c2481b64c38`。

没有明确批准，只允许做只读检查和写本任务包。

## 允许读取

允许读取：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/tasks/2026-05-30-workflow-user-reviewed-business-dispatch-real-readme-smoke-v1.md`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json`
- `/Users/yoyi/codex-workflow-mario-test/README.md`
- 真实 workflow state 必要结构：
  - `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

允许读取 Codex 索引中的线程元数据。

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件内容
- 完整 transcript 正文
- rollout JSONL 正文

## 允许写入

用户明确确认后，允许写真实 workflow state：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`

允许写 evidence / handoff：

- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-prepare-workflow-state-for-readme-smoke-v2.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-prepare-workflow-state-for-readme-smoke-v2-result.md`

允许更新：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`

## 禁止事项

- 禁止未获用户确认就写真实 workflow state。
- 禁止执行 `codex exec resume`。
- 禁止发送 README smoke 指令。
- 禁止执行新的 `codex exec`。
- 禁止写 `/Users/yoyi/.codex`。
- 禁止修改 `/Users/yoyi/codex-workflow-mario-test/README.md`。
- 禁止读取完整 transcript。
- 禁止读取敏感文件。
- 禁止运行 harness。
- 禁止改写或删除现有 `/Users/yoyi/gameai/agent world` 相关记录。
- 禁止在 shell 双引号里写未转义反引号模式；搜索包含反引号文本时必须使用单引号或 `rg -F`。

## 建议执行顺序

1. 只读检查 README 目标行是否仍不存在。
2. 只读检查 `codex-index.json` 中 thread `019e7738-5e29-74e0-a22f-5c2481b64c38`：
   - `project_root=/Users/yoyi/codex-workflow-mario-test`
   - `rollout_exists=true`
3. 只读检查真实 workflow state 当前结构和数量。
4. 用户确认写 state 后：
   - 备份 workflow state。
   - 创建或登记 project。
   - 创建 workflow。
   - 创建 node。
   - 创建 README smoke work item。
   - work item 设为 `ready_to_dispatch`。
   - 写 active binding。
   - 写 audit events。
   - 更新 `updated_at`。
5. 只读复核：
   - project 存在。
   - workflow 存在。
   - node 存在。
   - work item 存在且 `ready_to_dispatch`。
   - active binding 存在且 thread id 正确。
   - thread 在索引内。
   - rollout 存在。
   - README 目标行仍不存在。

## 建议 id 口径

可采用确定性 id，便于后续派发：

```text
project:users-yoyi-codex-workflow-mario-test
workflow:users-yoyi-codex-workflow-mario-test:default
workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev
work-item:workflow:users-yoyi-codex-workflow-mario-test:default:readme-smoke
```

如果现有代码要求时间戳型 work item id，可以使用时间戳，但必须在 evidence / handoff 中回传最终 id。

## 建议 audit event 类型

至少追加：

- `workflow_project_registered_for_readme_smoke`
- `workflow_readme_smoke_work_item_ready_to_dispatch`
- `workflow_node_session_bound_for_readme_smoke`

audit event 里必须包含：

- workflow id
- work item id
- node id
- thread id
- project root
- 操作时间

## 验收标准

必须满足：

- 真实 workflow state 有 `/Users/yoyi/codex-workflow-mario-test` 对应 project / workflow。
- 有 README smoke work item。
- work item 状态是 `ready_to_dispatch`。
- 有 active binding。
- active binding 指向 thread `019e7738-5e29-74e0-a22f-5c2481b64c38`。
- 绑定 thread 在索引中存在且 rollout 存在。
- 写入前有备份。
- 有 audit events。
- 未执行 `codex exec resume`。
- 未执行新的 `codex exec`。
- 未写 `/Users/yoyi/.codex`。
- 未修改 README。
- 未读取敏感文件或完整 transcript。

## 必须回传

回传必须包含：

1. 薄弱点。
2. 是否获得用户明确批准。
3. 是否写真实 workflow state。
4. 是否写 `/Users/yoyi/.codex`。
5. 是否执行 `codex exec` 或 `codex exec resume`。
6. 是否修改 README。
7. 目标 project id / workflow id / work item id / node id。
8. 绑定 thread id 和 rollout 状态。
9. 备份路径。
10. audit event id。
11. 新增 evidence / handoff。
12. 只读复核结果。

## 总指导回收重点

总指导回收时必须判断：

- 是否只准备 workflow state，没有执行 README smoke。
- 是否绑定到了正确项目路径的测试会话。
- 是否没有改动旧的 `agent world` workflow。
- 是否可以进入下一轮真实 README smoke 派发任务。

# 任务包：准备 README smoke 测试 workflow state v1

## 任务名

准备 README smoke 测试 workflow state v1。

## 所属开发线

桌面应用线 / Codex 会话线 / 总指导线。

验证线按需复核。

## 当前判断

真实 README smoke 不能直接执行。

依据：

- 当前真实 workflow state 只登记了 `/Users/yoyi/gameai/agent world` 的 workflow。
- 当前唯一 work item 状态是 `ready_for_review`，不是 `ready_to_dispatch`。
- 真实 workflow state 里没有 `/Users/yoyi/codex-workflow-mario-test` 的 workflow / work item / active binding。
- README 目标行 `Workflow dispatch smoke passed.` 当前还不存在。

大白话：

要改的是测试项目 README，但工作流账本还没有这个测试项目的工作流节点和待派发工作项。不能拿旧的 `agent world` workflow 硬派发到新项目。

## 薄弱点

- 这一步会写真实 workflow state。
- 如果要绑定真实 Codex 会话，必须确认绑定哪个 thread。
- 这一步不执行 README 修改，只准备派发前置条件。
- 不能把准备 workflow state 说成 README smoke 已执行。

## 目标

把真实 workflow state 准备成可以派发 README smoke 的状态：

1. 为 `/Users/yoyi/codex-workflow-mario-test` 创建或登记项目。
2. 创建默认 workflow。
3. 创建用于 README smoke 的 work item。
4. work item 状态设为 `ready_to_dispatch`。
5. 创建或绑定 active Codex 会话。
6. 节点绑定必须指向 cwd / project_root 匹配 `/Users/yoyi/codex-workflow-mario-test` 的测试会话，或明确记录 warning。
7. 写入备份和 audit events。
8. 只读复核：
   - workflow 存在。
   - work item 是 `ready_to_dispatch`。
   - active binding 存在。
   - 绑定 thread 在 `codex-index.json` 中存在。
   - rollout 存在。

## 非目标

- 不执行真实 `codex exec resume`。
- 不发送 README smoke 指令。
- 不写 `/Users/yoyi/.codex`，除非用户另行批准创建新 Codex 会话。
- 不修改 `/Users/yoyi/codex-workflow-mario-test/README.md`。
- 不读取完整 transcript。
- 不读取 `auth.json`、`.env`、密钥、token、授权文件。
- 不运行 harness。
- 不把准备状态说成真实业务派发完成。

## 允许读取

允许读取：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/tasks/2026-05-30-workflow-user-reviewed-business-dispatch-real-readme-smoke-v1.md`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json`
- `/Users/yoyi/codex-workflow-mario-test/README.md`
- 真实 workflow state 必要结构：
  - `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

允许读取 Codex 索引中的线程元数据，但禁止读取完整 transcript。

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件内容
- 完整 transcript 正文

## 允许写入

用户明确确认后，允许写真实 workflow state：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`

允许写 evidence / handoff：

- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-prepare-workflow-state-for-readme-smoke-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-prepare-workflow-state-for-readme-smoke-v1-result.md`

允许更新：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`

如必须创建一个 cwd 为 `/Users/yoyi/codex-workflow-mario-test` 的新 Codex 会话，必须单独获得用户确认，因为会执行 `codex exec` 并写 `/Users/yoyi/.codex`。

## 禁止事项

- 禁止未获用户确认就写真实 workflow state。
- 禁止执行 `codex exec resume`。
- 禁止发送 README smoke 指令。
- 禁止未获单独确认就执行任何 `codex exec`。
- 禁止写 `/Users/yoyi/.codex`，除非用户明确批准创建测试会话。
- 禁止修改 `/Users/yoyi/codex-workflow-mario-test/README.md`。
- 禁止读取完整 transcript。
- 禁止读取敏感文件。
- 禁止运行 harness。
- 禁止在 shell 双引号里写未转义反引号模式；搜索包含反引号文本时必须使用单引号或 `rg -F`。

## 建议执行顺序

1. 只读检查 README 目标行是否仍不存在。
2. 只读检查 `codex-index.json` 中是否已有 project_root 为 `/Users/yoyi/codex-workflow-mario-test` 的 Codex thread。
3. 如果没有合适 thread，停止并回传需要创建测试会话，不要擅自创建。
4. 用户确认写 state 后：
   - 备份 workflow state。
   - 写项目 / workflow / node / work item。
   - work item 设为 `ready_to_dispatch`。
   - 写 active binding。
   - 写 audit events。
5. 只读复核。

## 验收标准

必须满足：

- 真实 workflow state 有 `/Users/yoyi/codex-workflow-mario-test` 对应 project / workflow。
- 有 README smoke work item。
- work item 状态是 `ready_to_dispatch`。
- 有 active binding。
- 绑定 thread 在索引中存在且 rollout 存在。
- 写入前有备份。
- 有 audit events。
- 未执行 `codex exec resume`。
- 未修改 README。
- 未读取敏感文件或完整 transcript。

## 必须回传

回传必须包含：

1. 薄弱点。
2. 是否写真实 workflow state。
3. 是否写 `/Users/yoyi/.codex`。
4. 是否执行 `codex exec` 或 `codex exec resume`。
5. 是否修改 README。
6. 目标 workflow id / work item id / node id。
7. 绑定 thread id 和 rollout 状态。
8. 备份路径。
9. audit event id。
10. 新增 evidence / handoff。
11. 只读复核结果。

## 总指导回收重点

总指导回收时必须判断：

- 是否把 README smoke 的前置状态准备好。
- 是否没有执行真实 README 修改。
- 是否绑定到了正确项目路径的测试会话。
- 如果绑定路径不一致，warning 是否清楚。

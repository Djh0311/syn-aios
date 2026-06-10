# 任务包：工作流节点状态收口修正 v1

## 任务名

工作流节点状态收口修正 v1。

## 所属开发线

桌面应用线 / 工作流状态线。

总指导线回收。

## 当前判断

真实 README smoke 已回收通过，但暴露出状态收口缺口。

依据：

- `handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-real-readme-smoke-v1-review.md`
- 真实 workflow state 只读复核显示：
  - work item：`workflow:users-yoyi-codex-workflow-mario-test:default:readme-smoke`
  - work item state：`ready_for_review`
  - current node：`...:node:review`
  - 实际派发节点：`...:node:codex-dev`
  - codex-dev node state：`running`
- 后端 `write_started_dispatch` 会把 running 节点设为 `running`。
- 后端 `write_completed_dispatch` 会把 review 节点设为 `ready_for_review`，但没有明确收回之前的 codex-dev running 状态。

大白话：

任务已经派完并进入“待回收”，但执行节点还挂着“执行中”。这会让工作台看起来像任务还没结束。

## 薄弱点

- 这不是新一轮业务派发，不应再执行 README smoke。
- 不能靠手工改真实 workflow state 来掩盖代码问题。
- 当前只发现 completed 路径的真实残留；失败、超时、取消等路径也可能留下旧节点状态，需要一起复核。
- 真实 workflow state 里已有一条存量脏状态；本任务默认先修代码和离线测试，不直接修存量真实 state。

## 目标

修正 workflow node 状态收口规则：

1. 当 dispatch started 时，实际派发节点可以进入 `running`。
2. 当 dispatch completed 且 work item 进入 `ready_for_review` 时：
   - work item current node 应指向 review 节点。
   - review 节点可显示 `ready_for_review`。
   - 原实际派发节点不应继续显示 `running`。
3. 对失败路径做同类复核：
   - `failed`
   - `timed_out`
   - 后续若有 `cancelled`
   - 不应让已经结束的派发节点永久停在 `running`。
4. 增加离线测试覆盖：
   - started 后 codex-dev 为 `running`。
   - completed 后 codex-dev 不再是 `running`。
   - completed 后 work item 为 `ready_for_review`，current node 为 review。
   - 用户审核业务派发 completed 路径也覆盖同样状态收口。
5. 如需要新增 helper，应保持小范围：
   - 例如按 dispatch 里的 `node_id` 清理旧执行节点状态。
   - 不要重构整个 workflow state 模型。

## 非目标

- 不执行真实 `codex exec resume`。
- 不执行任何新的 `codex exec`。
- 不发送 README smoke 指令。
- 不写 `/Users/yoyi/.codex`。
- 不修改 `/Users/yoyi/codex-workflow-mario-test/README.md`。
- 不写真实 workflow state。
- 不手工修复真实 workflow state 里的存量 `running`。
- 不读取完整 transcript。
- 不读取 `auth.json`、`.env`、密钥、token、授权文件。
- 不运行 harness。
- 不做复杂业务自动编排。
- 不做项目团队工作区 v1。

## 允许读取

允许读取：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-real-readme-smoke-v1-review.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-user-reviewed-business-dispatch-real-readme-smoke-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-real-readme-smoke-v1-result.md`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json`

允许只读复核真实 workflow state 必要摘要：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件内容
- 完整 transcript 正文
- rollout JSONL 正文

## 允许写入

允许修改代码和测试：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`，仅当类型确实需要
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`，仅当 UI 展示依赖状态字段确实需要
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/`

允许写构建产物：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/dist/`

说明：

- `dist/` 只有在执行 `npm run build` 时允许变化。
- 如果 `dist/` 变化，必须在 evidence / handoff 明确列出。

允许写 evidence / handoff：

- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-node-state-closure-fix-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-node-state-closure-fix-v1-result.md`

允许更新：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`

## 禁止事项

- 禁止执行 `codex exec resume`。
- 禁止执行新的 `codex exec`。
- 禁止写 `/Users/yoyi/.codex`。
- 禁止发送任何业务指令或 safe probe。
- 禁止写真实 workflow state。
- 禁止手工把真实 codex-dev node 从 `running` 改掉。
- 禁止修改 `/Users/yoyi/codex-workflow-mario-test/README.md`。
- 禁止读取完整 transcript。
- 禁止读取敏感文件。
- 禁止运行 harness。
- 禁止联网安装依赖。
- 禁止使用 `--dangerously-bypass-approvals-and-sandbox`。
- 禁止把本轮代码修正说成复杂业务自动编排完成。
- 禁止在 shell 双引号里写未转义反引号模式；搜索包含反引号文本时必须使用单引号或 `rg -F`。

## 建议实现口径

优先在后端状态写入路径修正：

- `write_started_dispatch`
- `write_completed_dispatch`
- `write_failed_dispatch`

建议重点：

1. completed 路径从 dispatch 记录读取实际派发 `node_id`。
2. work item 转入 `ready_for_review` 后，更新 review 节点状态。
3. 同时把实际派发节点从 `running` 收口到一个非运行状态。
4. 如果现有状态枚举没有合适节点状态，优先复用已存在、可解释的状态，例如：
   - completed 成功后：`ready_for_review`
   - failed 后：`failed`
   - timed out 后：`timed_out`
5. 不要把 work item current node 改回 codex-dev；current node 仍应按流程进入 review。
6. 不要通过修改真实 workflow state 验证；用 Rust 离线 fixture 证明。

需要复核的现有代码点：

- `workflow_node_for_work_item_state` 当前把 `running` 映射到 codex-dev，把 `ready_for_review` 映射到 review。
- `write_started_dispatch` 会设置 running 节点为 `running`。
- `write_completed_dispatch` 当前只设置 review 节点为 `ready_for_review`。
- `write_failed_dispatch` 当前写 dispatch/control/attempt，但需要复核 work item 和 node 状态是否也应明确收口。

## 验收标准

必须满足：

- completed 派发后，work item state 为 `ready_for_review`。
- completed 派发后，work item current node 为 review。
- completed 派发后，实际派发 node 不再是 `running`。
- 用户审核业务派发 completed 后也满足上述状态。
- failed / timed_out 路径不应留下实际派发 node 永久 `running`；如果本轮不修，必须明确写入 evidence 并从完成口径移出。
- safe probe 路径不回退。
- 不保存完整 transcript。
- 不读取敏感文件。
- 不写真实 workflow state。
- 不写 `/Users/yoyi/.codex`。

## 建议验证

代码验证：

```bash
cargo fmt
cargo test --offline
npm run typecheck
npm run test:offline-interaction
npm run build
/Users/yoyi/miniconda3/bin/python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json
rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md
```

如果 Cargo 需要指定缓存路径，沿用项目既有路径：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline
```

测试建议：

- 增加 Rust 离线测试：safe probe completed 后实际派发节点不再 `running`。
- 增加 Rust 离线测试：user reviewed instruction completed 后实际派发节点不再 `running`。
- 增加 Rust 离线测试：failed / timed_out 写入 attempt 后实际派发节点状态不会残留 `running`。
- 如前端依赖节点状态展示，补前端离线测试；否则说明前端无需改。

## 必须回传

回传必须包含：

1. 薄弱点。
2. 做了什么。
3. 改了哪些文件。
4. 是否执行 `codex exec` 或 `codex exec resume`。
5. 是否写 `/Users/yoyi/.codex`。
6. 是否写真实 workflow state。
7. 是否读取敏感文件或完整 transcript。
8. 是否修改 `/Users/yoyi/codex-workflow-mario-test/README.md`。
9. completed / failed / timed_out 的节点状态收口规则。
10. 离线测试覆盖点。
11. `dist/` 是否变化，以及如何处理。
12. 验证命令和结果。
13. 新增 evidence / handoff。
14. 是否仍需要单独任务修复存量真实 workflow state。

## 总指导回收重点

总指导回收时必须判断：

- 是否真正修掉 “work item 已待回收，但 codex-dev node 仍 running”。
- 是否没有通过手工修改真实 workflow state 绕过代码问题。
- failed / timed_out 是否有明确状态收口。
- 是否仍然没有执行新的 `codex exec resume`。
- 是否需要单独批准一次真实 workflow state 存量修复。

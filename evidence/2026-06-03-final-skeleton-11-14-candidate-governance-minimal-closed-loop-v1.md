# final-skeleton-11 + 14 候选治理最小闭环 evidence v1

日期：2026-06-03

## 先说薄弱点

- 本轮 UI 没有真实浏览器 / Tauri 窗口截图证据：当前可用工具没有浏览器控制能力，本项目也没有 Playwright 依赖；已用离线 SSR 测试、typecheck 和 build 覆盖文案与渲染路径。
- `MemoryRecord` 仍只是目标形状，不是正式记忆实现。
- 两个 sidecar 目前是 JSON 第一版，不是 SQLite；并发控制是文件 lock + revision，不是跨进程事务数据库。
- 项目页候选治理条是最小入口，不是完整候选工作台。

## 本轮目标

按 `tasks/2026-06-03-final-skeleton-11-14-candidate-governance-minimal-closed-loop-v1.md` 完成：

1. `final-skeleton-11` 黑板候选持久确认状态。
2. `final-skeleton-14` 记忆候选生命周期。
3. 交叉边界测试，证明两类候选不串线。

## 已实现

### 黑板候选

- 新增后端 sidecar store：`prototypes/productized-desktop-shell/src-tauri/src/blackboard_candidate_store.rs`
- sidecar 路径：`<workflow_state_dir>/blackboard-candidates.v1.json`
- 支持读取不存在时初始化空 store。
- 支持 `candidate_pending_control_core`、`candidate_confirmed_for_followup`、`candidate_rejected`、`candidate_deferred`、`candidate_discarded`。
- 写入使用 lock、backup、tmp + rename、revision。
- 损坏 JSON 读取时拒绝覆盖。
- `candidate_key` 默认按 `project_id + workflow_id + entry_kind + target_kind + source_refs` 生成 `bbcand:v1:<sha256>`。
- Tauri 命令：
  - `load_blackboard_candidate_store`
  - `record_blackboard_candidate_decision`

### 记忆候选

- 新增后端 sidecar store：`prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- sidecar 路径：`<workflow_state_dir>/memory-candidates.v1.json`
- 支持创建候选、确认保留、拒绝、隔离、废弃、替代相关候选状态。
- `candidate_confirmed` 只表示候选确认保留，不写正式 `MemoryRecord`。
- 禁止 `candidate_confirmed -> memory_active`。
- 无来源候选拒绝。
- Tauri 命令：
  - `load_memory_candidate_store`
  - `create_memory_candidate`
  - `record_memory_candidate_decision`

### 前端

- 新增前端类型：`prototypes/productized-desktop-shell/src/lib/types.ts`
- 新增纯读模型：`prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`
- 新增 Tauri 调用包装：`prototypes/productized-desktop-shell/src/lib/tauri.ts`
- 项目工作流页新增最小候选治理条：
  - 黑板 sidecar / 记忆 sidecar 摘要。
  - 黑板候选确认后续处理、拒绝、暂缓、废弃。
  - 记忆候选确认保留、隔离、废弃。
  - 文案明确“不写正式事实、不写正式长期记忆、不推进 workflow state”。

## 红灯测试

先写测试后实现：

- Rust 聚焦测试最初失败：`BlackboardCandidateDecisionOutcome::ConfirmedForFollowup / Deferred / Discarded` 不存在。
- 前端离线测试最初失败：`../src/lib/candidateGovernance` 不存在。

## 验证结果

通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib
rustfmt --check src/blackboard_candidate_store.rs src/memory_candidate_store.rs
```

最新结果摘要：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 6`。
- `npm run build`：通过；Vite 仍提示 chunk > 500 kB，这是既有构建体积提醒，不是本轮失败。
- `cargo test --lib`：通过，93 passed，1 ignored。
- `rustfmt --check src/blackboard_candidate_store.rs src/memory_candidate_store.rs`：通过。

## 边界自检

已做固定文本检查：

- 新增候选代码没有 `codex exec`。
- 新增候选代码没有 `/Users/yoyi/.codex`。
- 新增候选代码没有 “已记住 / 正式记忆已写入 / 长期记忆已生效 / 已学习” 禁用 UI 文案。
- 新增候选类型没有把 `blackboard_candidates` / `memory_records` 写入 `workflow-state.v0.json`。

说明：

- `ProjectsView.tsx` 里仍存在既有真实 Codex 派发文案，属于历史派发入口，不是本轮候选治理新增路径。

## 没有做

- 没有写正式事实。
- 没有写正式 `MemoryRecord`。
- 没有写长期正式记忆。
- 没有改 `workflow-state.v0.json` 结构。
- 没有迁移数据库。
- 没有接 SQLite、向量库、图数据库或 Obsidian。
- 没有执行真实 Codex。
- 没有读写 `/Users/yoyi/.codex`。
- 没有启动 MCP canvas run。
- 没有写真实业务项目目录。

## 下一步判断

可以进入 `final-skeleton-15-secretary-core-readonly-model-v1`，但只建议做秘书只读模型。

仍不能做：

- 秘书直接改事实。
- 秘书直接派发任务。
- 秘书写正式记忆。
- 候选注入任务包成为正式依据。

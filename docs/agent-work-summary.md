# Syn 站 3a 工作总结

> 日期：2026-07-12
> 范围：主管动作意图 → Syn 控制核心 → worker 受控适配器桥
> 结论：`PASS__RISK_CLEANED__READY_FOR_3B`
> 边界：站 3b 未启动；未执行 `git add`、commit 或 push。

本文是站 3a 的历史工作总结，不是当前状态正本。当前状态看 `docs/current-state.md`，文件效力看 `docs/product/authority-register-v1.md`，当时完成事实与原始证据以 `evidence/2026-07-12-orchestrator-station3a-control-core-bridge-v1.md` 为准。

## 1. 这轮做了什么

### 1.1 把职责重新分清

- 主管模型只输出一个严格结构化的动作提议，不直接获得项目写权限，也不直接启动 worker。
- Syn 控制核心重新检查当前 run、授权、任务包、项目、工作项、配额、幂等键和账本修订，检查通过后才调用受控适配器。
- worker 只接收用户目标、任务包和 worker 验收条件；Syn 的控制职责和主管的终审职责不进入 worker prompt。
- 公共主管 MCP 工具面保持只读；派发、检查、终标和报告等副作用只能经过宿主控制核心。
- worker 的结构化回程先由 Syn 检查。`blocked`、格式错误、证据为空或占位证据都会安全停在 `waiting_user`，不能被进程退出码冒充业务完成。

### 1.2 完成固定测试项目真实验收

站 3a 在 `/Users/yoyi/codex-workflow-mario-test` 进行了多轮真实 UI 发射：

- v3、v4：worker 回程格式无效，Syn 拒绝验收并安全停下。
- v5：发射前发现会复用历史 native thread，零派发停止并补上全新任务会话守卫。
- v6：worker 已完成，但主管重复 `inspect_worker` 直到配额停止；据此补上动作推进契约。
- v7：唯一业务完成运行。使用全新 authorization、work item、supervisor run 和 native worker thread，只启动一个 worker、一次 execution attempt、零 follow-up。

v7 的动作顺序严格为：

```text
dispatch_worker
→ inspect_worker
→ finalize(pass)
→ report_user
```

目标文件为 `/Users/yoyi/codex-workflow-mario-test/station3a-control-core-proof-v7.txt`，独立核验结果：

- 39 bytes；
- 末字节为 `33`（`!`）；
- 无末尾换行；
- SHA-256：`7777cfb8a53af75923f665191c80e5acf83c81436658c0b4cc61a25a420c18f3`。

### 1.3 做了三轮独立工程风险复核

真实业务链跑通后没有直接进入 3b，而是继续清理了三个风险：

1. **binding 身份碰撞**：旧 `binding_id` 使用截断 slug，不同任务可能得到相同 ID。现已改为基于完整 workflow、node、work item 和 native thread 身份的 SHA-256。
2. **迁移漏同步引用**：首轮迁移只更新了 binding，历史 dispatch 仍引用旧 ID。现已同步修复 binding 与 dispatch，并保留两次写前备份和中间不完整状态。
3. **历史记录误改绑**：宽松回退可能把保留的旧 SHA 历史引用绑定到错误对象。现只迁移可证明的真实 legacy 候选；候选仍歧义时整次拒写，未知或旧 SHA 历史引用保持原样。

最终线上账本核验：

- 71 条 session binding；
- 71 个唯一 `binding:sha256:<64 hex>` 身份；
- 352 条 dispatch 全部具有 binding 引用；
- 352/352 条引用可解析；
- 0 条孤儿引用；
- 线上 `workflow-state.v0.json` 与冻结的最终迁移快照 SHA-256 相同。

同时完成：

- `authorization_snapshot_hash` 和 `task_package_fingerprint` 改为真实 SHA-256，并在同一 run 的后续动作中持续核对；
- 授权快照发生漂移时以 `authorization_stale` 停止；
- fresh task session 写入真实创建者、绑定来源和控制核心权限 provenance；
- 合法 worker report 回写主管 worker 记录，不再只留在临时文件。

### 1.4 验证结果

- `cargo test --lib --quiet`：867 passed，0 failed，43 ignored，共 910。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：15 项通过。
- `cargo check --offline`：通过，保留既有 570 warnings。
- `cargo fmt --check`：仅报告三个历史漂移文件，没有本轮新增漂移。
- `git diff --check`：通过。
- v7 原始证据和 binding 迁移证据的 SHA-256 manifest：全部通过。

## 2. 改了哪些文件

以下列出站 3a 收口涉及的核心文件；当前工作树还包含其它执行线的既有改动，不能把 `git status` 中所有文件都归到站 3a。

### 2.1 主管协议、发射与控制核心

- `prototypes/productized-desktop-shell/src-tauri/src/supervisor_action_protocol.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/supervisor_action_controller.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/supervisor_session_launcher.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_orchestrator.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/director_agent.rs`

### 2.2 用户原文、任务包和 worker 回程

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/consultant_agent.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_consultation_proposal_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/worker_report.rs`

### 2.3 会话绑定、迁移与持久化

- `prototypes/productized-desktop-shell/src-tauri/src/workflow_run_dispatch_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib_task_package_preview_binding_read_model_tests.rs`

### 2.4 UI 与共享类型

- `prototypes/productized-desktop-shell/src/views/projects/ProjectJiaobanPanel.tsx`
- `prototypes/productized-desktop-shell/src/lib/types/workflow.ts`

### 2.5 计划、契约与状态文档

- `CURRENT.md`
- `AUTHORITY.md`
- `docs/plans/2026-07-11-orchestrator-fast-path-five-stations-plan-v1.md`
- `docs/plans/2026-07-11-supervisor-contract-v1-draft.md`
- `docs/plans/2026-07-11-supervisor-orchestrator-mode-proposal-v1.md`
- `tasks/2026-07-12-orchestrator-station3a-supervisor-action-control-core-bridge-v1.md`
- `evidence/2026-07-12-orchestrator-station3a-control-core-bridge-v1.md`

## 3. 新增了哪些 evidence / handoff

### 当前完成证据

- `evidence/2026-07-12-orchestrator-station3a-control-core-bridge-v1.md`：站 3a 总证据，包含 v3–v7 运行事实、职责边界、三轮风险清理和最终验证。
- `evidence/raw/2026-07-12-station3a-v7/`：v7 前后 sidecar、UI 截图、主管分步输出、worker 原始回程、目标文件快照、独立字节核验和哈希清单。
- `evidence/raw/2026-07-12-station3a-binding-id-migration/`：迁移前状态、两次写前备份、中间不完整状态、最终状态、引用核验和哈希清单。

### 交接方式

- 本轮没有再创建一份重复 handoff；`CURRENT.md` 负责当前入口，上述主 evidence 负责可追溯交接。
- `tasks/2026-07-12-orchestrator-station3a-supervisor-action-control-core-bridge-v1.md` 保留实现边界和验收合同。

## 4. 当前权威或当前入口

按读取顺序：

1. `CURRENT.md`：当前事实；站 3a 已完成，停在 3b 单独拍板门。
2. `AUTHORITY.md`：当前权威文档索引。
3. `docs/plans/2026-07-11-orchestrator-fast-path-five-stations-plan-v1.md`：五站排布和 3b 的独立授权边界。
4. `docs/plans/2026-07-11-supervisor-contract-v1-draft.md`：主管运行契约正本；`draft` 只是历史路径兼容，不表示待定。
5. `evidence/2026-07-12-orchestrator-station3a-control-core-bridge-v1.md`：站 3a 完成事实和风险清理正本。

## 5. Superseded / paused / historical

### Historical

- v3、v4、v5、v6 都只作为失败路径或安全拦截证据，不能作为站 3a 完成证据。
- `tasks/2026-07-12-orchestrator-station3a-unattended-closure-v1.md` 已被当前控制核心桥任务包取代。
- `evidence/2026-07-12-orchestrator-station3a-unattended-closure-v1.md` 已明确标记 `HISTORICAL__SUPERSEDED`。
- 旧的主管 MCP 副作用工具方案已由宿主控制核心桥取代；公共 MCP 现为只读。
- 旧 `--ignore-user-config` 与临时配置中的 `approval_policy="never"` 不再是当前发射方案。

### Paused

- 站 3b 未启动。它会进入非测试真实项目，即使只读也属于重档，不能继承站 3a 的固定测试授权。
- 未经用户明确批准，不执行站 3b，也不执行 commit 或 push。

## 6. 清理建议与剩余风险

### 已清理的阻断风险

- binding ID 截断碰撞；
- dispatch 迁移后悬空；
- 历史 SHA 引用被宽松回退错误改绑；
- 假 hash/fingerprint、fresh session provenance 不实、合法 worker report 未回写；
- worker 回程错误被进程退出码误判为业务完成；
- 主管重复已完成的 `inspect_worker`。

第三轮独立复核没有发现剩余 P0/P1，站 3a 可以停在 `READY_FOR_3B`。

### 非阻断风险

- `finalize` 仍是 advisory，`workflow_chain_state_written=false`；3b 不能把主管意见自动当成用户决定。
- v7 有一步记录了 model-list refresh timeout，但动作账本闭合且业务链完成；建议作为性能/供应端诊断保留。
- `cargo check --offline` 仍有既有 570 warnings。
- `cargo fmt --check` 仍有三个历史漂移：`src/codex_db.rs`、`src/codex_local_runner.rs`、`src/mcp/storage.rs`。
- `docs/memory-layer-design-v1.md` 已超过 3,000 行，后续单独做文档拆分；本轮不把无关文档重构混入站 3a 收口。
- 工作树存在其它执行线和历史未跟踪文件；后续提交前必须按来源拆分，不能整树打包。

### 进入 3b 前必须重新确认

- 用户明确批准具体真实项目、只读目标和可读根；
- sandbox 保持 `read-only`，写根为空；
- 新建 authorization、work item、supervisor run 和 native worker thread；
- 单独保存 3b 原始证据，不复用 v7 身份或把站 3a PASS 外推到真实项目。

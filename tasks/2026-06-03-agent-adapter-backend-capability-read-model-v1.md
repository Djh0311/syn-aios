# Task Package：Agent adapter 后端能力声明读模型 v1

状态：已完成。  
用途：把当前前端只读 `adapterCapabilities.ts` 能力声明收敛为后端 typed read model，为 Claude Code / OpenClaw / OpenCode 后续接入预留同一套 adapter 入口。  
执行方式：一个小批次完成，不拆成十几个微任务；最终统一验收。

完成记录：

- evidence：`../evidence/2026-06-03-agent-adapter-backend-capability-read-model-v1.md`
- handoff：`../handoffs/2026-06-03-agent-adapter-backend-capability-read-model-v1-result.md`
- 结论：后端 `WorkbenchSnapshot.agent_adapters[]` 已输出结构化 `codex-local` 能力声明；Agent 页和秘书只读模型优先使用后端 descriptor；前端 `adapterCapabilities.ts` 只保留 fallback。

## 1. 先说薄弱点

当前 `final-skeleton-12` 已经做了 adapter 能力声明骨架，但薄弱点很明确：

- `AgentAdapterDescriptor` / `AdapterCapability` 现在主要在前端 `src/lib/adapterCapabilities.ts` 派生。
- 后端 `WorkflowStateSnapshot` 只有 `counts.agent_adapters`，没有结构化 `agent_adapters[]` read model。
- `workflow-state.v0.json` 里虽然已有 `agent_adapters` 原始数组，但产品 UI 不能稳定依赖前端临时拼装。
- Claude Code / OpenClaw / OpenCode 还没有接入，不能被 UI 暗示为可用。
- 高风险能力只能声明边界，不能在本轮重新执行真实 Codex。

一句话目标：

```text
后端提供结构化 agent_adapters[] 能力声明；
前端优先展示后端 descriptor；
Codex 仍是唯一已声明可用 adapter；
本轮不接新 agent、不改真实执行语义。
```

## 2. 必须先读

当前入口：

- `CURRENT.md`
- `AUTHORITY.md`
- `tasks/README.md`
- `docs/workbench-system-architecture-v1.md`

前置记录：

- `evidence/2026-06-03-final-skeleton-12-adapter-capability-registry-v1.md`
- `handoffs/2026-06-03-final-skeleton-12-adapter-capability-registry-v1-result.md`
- `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md` 的 Skeleton-12
- `tasks/2026-06-03-session-center-foundation-hardening-v1.md`
- `tasks/2026-06-03-workflow-dispatch-readback-native-parser-v1.md`

主要代码入口：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/adapterCapabilities.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

重点搜索：

- `AgentAdapterDescriptor`
- `AdapterCapability`
- `deriveAgentAdapterDescriptors`
- `agent_adapters`
- `codex-local`
- `hidden_unimplemented_adapters`
- `adapter_descriptor_is_read_model_only`

搜索固定文本必须用 `rg -F '...'` 或单引号，避免 shell 反引号命令替换。

## 3. 已知事实 / 未知 / 假设

已知事实：

- 前端已有 `AdapterCapabilityKind`、`AdapterCapabilityStatus`、`AdapterCapability`、`AgentAdapterDescriptor`。
- 前端已能从 sessions / projects / workflowState 派生 Codex `codex-local` 能力声明。
- `WorkflowStateCounts` 已有 `agent_adapters` 数量。
- `workflow-state.v0.json` 初始化时已有 `agent_adapters` 原始数组，包含 `codex-local`。
- 当前 Agent 页已展示只读能力面板。
- 秘书只读模型会读取 adapter descriptor warnings 生成风险信号。

未知：

- 后端 read model 最终放在 `WorkbenchSnapshot.agent_adapters`、`WorkflowStateSnapshot.agent_adapters`，还是两者都放。
- 是否需要完全删除前端派生 helper，还是先保留为离线 fallback。
- 现有离线测试 fixtures 是否需要补完整 backend descriptor。

本任务采用的假设：

- 第一版以后端 `WorkbenchSnapshot.agent_adapters` 为主，因为它能同时看到 sessions、projects 和 workflow state 读模型所需的上下文。
- 如实现者判断 `WorkflowStateSnapshot.agent_adapters` 更适合，也可以同时加，但必须避免重复 UI 真相。
- 前端 `deriveAgentAdapterDescriptors` 可以暂时保留为纯函数 fallback 或测试 helper，但 UI 应优先使用后端 descriptor。

## 4. 范围

允许：

- 在 Rust 后端新增 typed adapter descriptor 类型。
- 给 `WorkbenchSnapshot` 增加 `agent_adapters: Vec<AgentAdapterDescriptor>`。
- 可选：给 `WorkflowStateSnapshot` 增加 `agent_adapters: Vec<AgentAdapterDescriptor>`，但要说明为何需要。
- 新增后端 helper，从 sessions、projects、workflow state summary / raw state value 派生 `codex-local` descriptor。
- 更新 TypeScript 类型。
- 更新 Agent 页优先展示后端 descriptor。
- 更新秘书只读模型使用后端 descriptor，前端 helper 仅作 fallback。
- 更新离线测试和 Rust 单元测试。
- 更新 evidence / handoff / 当前入口文档。

禁止：

- 不接 Claude Code / OpenClaw / OpenCode 真实实现。
- 不显示未实现 adapter 的可点击能力按钮。
- 不执行 `codex exec`。
- 不执行 `codex exec resume`。
- 不启动真实 workflow machine。
- 不改真实 Codex 执行语义。
- 不读取或写入 `/Users/yoyi/.codex`。
- 不读取真实完整 transcript。
- 不改 `workflow-state.v0.json` 结构。
- 不迁移数据库。
- 不写正式事实。
- 不写正式记忆。
- 不运行 harness。
- 不启动 MCP canvas run。
- 不写真实业务项目目录。
- 不把本轮说成 Claude / OpenClaw / OpenCode 已接入。
- 不把能力声明说成能力已经重新真实验证。

## 5. 数据模型要求

建议 Rust 类型：

```text
AgentAdapterDescriptor {
  adapter_id,
  agent_type,
  agent_id,
  display_name,
  provider,
  status,
  permission_level,
  source_kind,
  capabilities,
  implemented_action_kinds,
  hidden_unimplemented_adapters,
  warnings
}

AdapterCapability {
  capability_id,
  kind,
  label,
  status,
  description,
  boundary,
  evidence_refs,
  warnings
}
```

字段要求：

- `adapter_id = "codex-local"`。
- `agent_type = "codex"`。
- `source_kind` 第一版应为 `"backend_read_model"`，不再是 `"frontend_read_model"`。
- `hidden_unimplemented_adapters` 仍包含 `claude-code`、`openclaw`、`opencode`。
- `warnings` 必须包含本轮边界，例如：
  - `adapter_descriptor_is_backend_read_model_only`
  - `does_not_change_codex_execution_semantics`
  - `unimplemented_adapters_hidden`

能力列表第一版至少覆盖现有前端能力：

- `session_index_read`
- `session_transcript_read`
- `workflow_node_binding`
- `safe_probe_dispatch`
- `user_reviewed_dispatch`
- `workflow_machine_run`
- `permission_decision_record`
- `harness_resource_index`

高风险能力状态必须是 `requires_confirmation`，并写清 boundary。

## 6. 执行段 A：后端 read model

目标：

- 后端 snapshot 直接输出结构化 `agent_adapters[]`。
- Codex adapter 的能力声明不再只能由前端拼出来。

建议实现：

1. 在 `src-tauri/src/types.rs` 增加 adapter descriptor 类型。
2. 在 `WorkbenchSnapshot` 增加 `agent_adapters` 字段。
3. 在 `build_snapshot` 或相邻 helper 中派生 `codex-local` descriptor。
4. 派生依据可以包括：
   - Codex sessions 数量。
   - 带 rollout 的 sessions 数量。
   - workflow node active bindings。
   - dispatch history。
   - permission requests。
   - harness resources。
   - workflow state 原始 `agent_adapters` 数组是否存在。
5. 如果 workflow state 不存在，也应该返回一个 `codex-local` descriptor，状态可为 `not_connected` 或 `degraded`，并带 warnings。

验收：

- `load_workbench_snapshot` 返回 `agent_adapters[]`。
- 有 Codex sessions 时，`codex-local.status = available`。
- 无 Codex signals 时，descriptor 不崩溃，状态和 warning 合理。
- 未实现 adapter 不显示为可用能力。

## 7. 执行段 B：前端收敛

目标：

- Agent 页展示后端 descriptor。
- 前端 helper 不再是 UI 主真相。

建议实现：

1. `src/lib/types.ts` 增加和 Rust 匹配的类型。
2. `WorkbenchSnapshot` 增加 `agent_adapters`。
3. `AgentView.tsx` 优先使用 `snapshot.agent_adapters` 或从 App 传入的后端 descriptors。
4. `adapterCapabilities.ts` 可保留为 fallback，但：
   - UI 主路径不再调用它生成主 descriptor。
   - 如果保留 fallback，warning 必须写 `adapter_descriptor_frontend_fallback_used`。
5. `secretaryReadModel.ts` 使用后端 descriptor；只有测试或无后端字段时才 fallback。
6. UI 文案仍然只显示只读能力边界，不显示 Claude / OpenClaw / OpenCode 的操作按钮。

验收：

- Agent 页仍显示 Codex 能力声明。
- 面板能标出 source 为 backend read model。
- 秘书风险仍能读到 adapter warnings。
- 前端无类型错误。

## 8. 执行段 C：测试

必须补：

- Rust 单元测试：`WorkbenchSnapshot` 含 `agent_adapters[]`。
- Rust 单元测试：`codex-local` 能力状态和 warnings 稳定。
- 前端离线测试：Agent 页展示后端 descriptor。
- 前端离线测试：没有后端 descriptor 时 fallback 不崩溃。
- 前端离线测试：未实现 adapter 不出现可执行按钮。

验证命令：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib
rustfmt --check src/types.rs
```

如果新增 Rust 文件，也要对新增文件跑 `rustfmt --check`。

## 9. Evidence / Handoff

完成后新增：

- `evidence/2026-06-03-agent-adapter-backend-capability-read-model-v1.md`
- `handoffs/2026-06-03-agent-adapter-backend-capability-read-model-v1-result.md`

并更新：

- `CURRENT.md`
- `tasks/README.md`
- 本任务包状态改为已完成。

evidence 必须写清：

- 后端新增了哪些类型和字段。
- 前端是否仍保留 fallback。
- 哪些能力只是声明，未重新真实验证。
- Claude / OpenClaw / OpenCode 没有接入。
- 没有执行真实 Codex。
- 没有读写 `/Users/yoyi/.codex`。

## 10. 完成口径

接受为：

- Agent adapter 能力声明从前端临时读模型升级为后端 typed read model。
- Codex `codex-local` 是第一版唯一可见 adapter descriptor。
- UI 能显示后端 descriptor 和边界。

不接受为：

- Claude Code / OpenClaw / OpenCode 已接入。
- Codex 真实执行路径已重新验证。
- 工作台已经有完整多 agent adapter 系统。
- 发消息 / stop / restart / resume 能力完成。
- 权限、凭据、模型池或成本统计完成。

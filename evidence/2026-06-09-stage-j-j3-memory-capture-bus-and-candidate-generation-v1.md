# Stage J / J3 Memory Capture Bus And Candidate Generation v1 Evidence

日期：2026-06-09

状态：已完成，结论为 `accepted_with_deferred_items`。复核线初审发现 P1，主管线已修补；复审确认无 P0/P1，允许收口。当前不能声明 Stage J 完成。

## 1. 结论

J3 已新增工作台自有 `memory-capture-events.v1.json` 捕获总线，并把 capture event 接入现有 ObservationStore / MemoryCandidate 链路。`candidate_allowed` 可生成 observation 和 MemoryCandidate；`audit_only` / `blocked_sensitive` 不生成 observation / candidate；任何路径都不会自动写 FormalMemory。

本轮没有执行新的真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有启动 Tauri / Browser / Chrome / 截图工具。产品实现与测试没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout 内容。过程说明：主管线为处理用户 UI 设计要求按 Product Design skill 读取过 `.codex` 下的 skill / user-context 元数据，未读取会话、secret 或业务 transcript；因此不能把本轮过程表述为“完全没有访问 `.codex` 路径”。

## 2. 改动范围

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`：新增 `MemoryCaptureSourceRef`、`MemoryCaptureCandidateDraft`、`CaptureMemoryEventInput`、`MemoryCaptureEventRecord`、`MemoryCaptureStoreV1`、`CaptureMemoryEventOutput`。
- `prototypes/productized-desktop-shell/src-tauri/src/memory_capture_bus.rs`：新增 capture store 读写、校验、敏感内容阻断、observation / candidate 生成桥接和单测。
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`：注册 `memory_capture_bus` 模块和 command handler。
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`：新增 `load_memory_capture_store`、`capture_memory_event`。
- `prototypes/productized-desktop-shell/src/lib/types.ts`、`src/lib/tauri.ts`：同步 TS 类型和 Tauri wrapper。
- `prototypes/productized-desktop-shell/src/App.tsx`：加载 `memoryCaptureStore` 并传给记忆 / 知识库读模型。
- `prototypes/productized-desktop-shell/src/lib/memoryCenter.ts`、`src/views/MemoryCenterView.tsx`：记忆中心新增 capture events 摘要。
- `prototypes/productized-desktop-shell/src/lib/knowledgeBase.ts`、`src/views/KnowledgeBaseView.tsx`：知识库新增“记忆捕获来源”只读摘要。
- `prototypes/productized-desktop-shell/src/lib/workbenchNavigation.ts` 已核对：左侧栏已有 `想法箱 / 知识库 / 记忆层` 主入口，图标使用 inkwash 原型同组 `✎ / ▢ / ◐`；`运行中工作流` 使用 `≋`。

## 3. 数据契约

`MemoryCaptureEventRecord` 记录：

- 来源：`source_type`、`source_ref_id`、`audit_refs`、`readback_ref`、`task_package_ref`、`memory_packet_ref`。
- 绑定：`project_id`、`workflow_id`、`workflow_node_id`、`run_unit_id`、`product_command_id`、`product_attempt_id`。
- 摘要：`summary`、`evidence_summary`。
- 策略：`sensitivity`、`candidate_policy`、`blocked_reason`。
- 回链：`observation_id`、`candidate_key`。
- 审计时间：`created_by`、`created_at`、`updated_at`。

`candidate_policy` 当前为：

- `observation_only`
- `candidate_allowed`
- `audit_only`
- `blocked_sensitive`

`sensitivity` 当前为：

- `public`
- `internal`
- `project_confidential`
- `secret`

## 4. 生成链路

- `candidate_allowed`：先写 observation，再通过现有 observation -> MemoryCandidate 链路生成候选，候选仍不是正式记忆。
- `observation_only`：只写 capture event 和 observation，不生成 candidate。
- `audit_only`：只写 capture event，不生成 observation / candidate。
- `blocked_sensitive`：记录 blocked summary，不生成 observation / candidate。
- secret sensitivity 或 secret source 必须使用 `blocked_sensitive`。
- 损坏 JSON、revision conflict、重复 event_key 不覆盖既有 store。

## 5. UI 和信息层级

普通 UI 只显示用户可理解摘要：

- 记忆中心：显示捕获事件来源、摘要、策略和“候选不是正式记忆”边界。
- 知识库：显示 capture event 数量和最近捕获来源，帮助用户理解资料 / 执行事件如何进入候选链路。
- 左侧栏：已核对存在 `想法箱 / 知识库 / 记忆层` 主入口，图标与 `inkwash-full.html` 的 `✎ / ▢ / ◐` 一致；没有新增开发者折叠入口。

普通 UI 不显示 raw capture JSON、sidecar 绝对路径、完整 prompt、完整 transcript、raw stdout/stderr、secret 或 credential。

## 6. 复核线初审和修补

复核线初审结论：无 P0；发现 1 个 P1，因此不允许主管线直接收口 J3。

P1：

- `App.tsx` 已加载 `memoryCaptureStore`，但未传入 `MemoryCenterView`，导致记忆中心 capture 区实际为空，不满足 J3 “记忆中心可见 capture / observation / candidate source 摘要”的验收点。

修补：

- 已在 `App.tsx` 的 `MemoryCenterView` 调用中传入 `memoryCaptureStore={memoryCaptureStore}`，见 `prototypes/productized-desktop-shell/src/App.tsx`。

P2 已处理 / 分类：

- 已新增 `memory_capture_duplicate_event_is_rejected_without_append`，覆盖重复 event_key 不追加、不生成 observation/candidate。
- 已新增 `memory_capture_revision_conflict_does_not_overwrite_store`，覆盖 expected revision stale 时不覆盖 store。
- 已新增 `memory_capture_corrupt_json_is_rejected_without_overwrite`，覆盖损坏 JSON 拒绝且保留原文件内容。
- “UI 只读”口径调整为：J3 capture 摘要区只读；记忆中心和知识库仍保留既有正式记忆治理 / 候选创建动作，不能整体声称只读。

复核线复审结论：

- P1 已关闭：`App.tsx` 已向 `MemoryCenterView` 传入 `memoryCaptureStore`，`MemoryCenterView` 已接收并用于 `deriveMemoryManagementSummary`。
- 未发现新的 P0/P1。
- 允许主管线把 J3 收口为 `accepted_with_deferred_items`。

保留 P2：

- `candidate_allowed` 当前仍是先写 observation/candidate，再 append capture event；若晚期 capture append 因锁或写入失败中断，理论上仍可能出现下游记录已写但 capture event 未写的半完成状态。该项留给 J4/J5 做事务化或补偿记录设计。
- 只能声明 J3 capture 摘要区只读，不能声明整个记忆中心 / 知识库 UI 只读。

## 7. 验证

已通过：

- `npm run typecheck`
- `cargo test --lib memory_capture`：7 passed
- `cargo test --lib observation`：15 passed
- `cargo test --lib memory_candidate`：9 passed
- `cargo test --lib task_memory_packet`：10 passed
- `cargo test --lib project_workflow_automation`：11 passed / 2 ignored
- `cargo fmt -- --check`
- `cargo test --lib`：320 passed / 10 ignored
- `npm run test:offline-interaction`：13 passed
- `npm run build`：通过，仅既有 Vite chunk-size warning

## 8. 边界扫描

J3 限定文件敏感词扫描命中分类：

- `memory_capture_bus.rs` 命中 `full transcript`、`raw stdout`、`raw stderr`、`prompt body`、`auth token`、`oauth`、`keychain`、`.env`、`rollout`、`provider credential`：均为敏感内容 guard 黑名单或拒绝测试。
- `src-tauri/src/lib.rs`、`commands.rs`、`types.rs`、`types.ts`、`tauri.ts` 命中 `rollout` / `.codex`：属于既有 transcript / diagnostics / guard / fixture 代码，不是 J3 捕获总线新增真实读取。
- 未发现 J3 新增产品路径保存 prompt body、full transcript、raw stdout/stderr、secret 或 provider credential。

## 9. 不能声明

- 不能声明 Stage J 完成。
- 不能声明任意项目无限制自由执行完成。
- 不能声明自动 retry / stop / restart 完成。
- 不能声明 planned adapters 真实接入。
- 不能声明 provider credential / model verification 完成。
- 不能声明 FormalMemory 自动写入完成。
- 不能声明真实 Tauri J5 验收完成。

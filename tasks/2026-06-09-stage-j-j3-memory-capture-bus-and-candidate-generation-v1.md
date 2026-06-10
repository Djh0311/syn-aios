# Stage J / J3 Memory Capture Bus And Candidate Generation v1

日期：2026-06-09

状态：已完成，结论为 `accepted_with_deferred_items`。复核线初审发现 P1，主管线已修补，复审确认无 P0/P1，允许收口。本任务包用于接在 J2-B B1/B2 `accepted_with_deferred_items` 之后，推进 Stage J 的“记忆层记录 / 分析 / 候选化”产品链路。J3 不授权新的真实 `codex exec` / `codex exec resume`，不发送 prompt，不直接写 FormalMemory。产品实现与测试未读写 `/Users/yoyi/.codex`；本轮主管线为处理 UI 设计要求按 Product Design skill 读取过 `.codex` 下的 skill / user-context 元数据，未读取会话、secret 或业务 transcript，作为过程边界说明单独记录。

全局主管任务。J3 的目标是把 J1 / J2 / J2-B 已经形成的用户操作、Product Command、runtime log、audit、readback、worker report candidate、process fact decision 和 final review 安全接入记忆层：先形成 `MemoryCaptureEvent`，再按规则进入 `ObservationStore` 和 `MemoryCandidate`，最终仍由既有 M2 / M9 / M12 确认链路决定是否进入 FormalMemory。

## 0. 先说薄弱点

- J2-A 已经能从用户目标生成 run units，并有 process fact observation 的最小回收能力，但不是统一 memory capture bus。
- J2-B B1/B2 已经证明 run unit 可以走统一 Product Command Phase B 真实执行并 readback 成功，但 worker report candidate / C5 / observation / candidate 完整回收仍未完成。
- 现有 M3 `ObservationStore` 和 `create_memory_candidate_from_observation` 可复用，但缺少“哪些运行事件可以进入观察 / 候选、哪些只能留在 audit / runtime”的统一入口。
- 如果 J3 直接从 readback 或 last message 生成正式记忆，会绕过候选、确认权、lint、冲突和来源治理。
- 如果 J3 保存完整 prompt、完整 transcript、runner stderr 或 secret-like 内容，记忆层会变成敏感数据泄漏面。

## 1. 权威依据

必须服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-09-stage-j-codex-control-plane-workflow-memory-productization-plan-v1.md`
- `tasks/2026-06-09-stage-j-j0-permission-product-scope-and-acceptance-matrix-freeze-v1.md`
- `tasks/2026-06-09-stage-j-j1-codex-control-plane-free-control-entry-v1.md`
- `tasks/2026-06-09-stage-j-j1-b-mario-test-codex-control-real-resume-execution-point-v1.md`
- `tasks/2026-06-09-stage-j-j2-project-workflow-automation-run-units-and-controlled-closed-loop-v1.md`
- `tasks/2026-06-09-stage-j-j2-b-controlled-real-workflow-automation-execution-point-freeze-v1.md`
- `evidence/2026-06-09-stage-j-j2-b-b1-supervisor-acceptance-review-v1.md`
- `evidence/2026-06-09-stage-j-j2-b-b2-supervisor-acceptance-review-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

必须优先复用：

- M3：`ObservationStore`、`create_observation`、`create_memory_candidate_from_observation`。
- M2：候选到正式记忆的受控采纳链路。
- M5：memory lint / conflict 最小阻断。
- M6：TaskMemoryPacket included / excluded / review materials 规则。
- M9：FormalMemory lifecycle 版本、审计和用户确认。
- J2：`ProjectWorkflowAutomationPlan` / `ProjectWorkflowRunUnit` / run unit refs。
- G1/G2：runtime log / diagnostics 摘要边界。
- PCR10：真实执行 evidence 只通过统一 Product Command 链路解释。

## 2. J3 目标

J3 要交付：

1. 新增或等价实现 `MemoryCaptureEvent` 数据契约和受控捕获入口。
2. 支持从以下来源生成 capture event：
   - `user_action`
   - `product_command`
   - `runtime_log`
   - `readback`
   - `worker_report`
   - `process_fact_decision`
   - `final_review`
3. Capture event 必须携带 project / workflow / node / run unit / product command / runtime / audit / readback / task package / memory packet refs。
4. 明确 `candidate_policy`：
   - `observation_only`
   - `candidate_allowed`
   - `audit_only`
   - `blocked_sensitive`
5. 明确 `sensitivity`：
   - `public`
   - `internal`
   - `project_confidential`
   - `secret`
6. `audit_only` 和 `blocked_sensitive` 不得生成 observation / candidate。
7. `candidate_allowed` 可先写 observation，再按用户 / 项目主管授权生成 MemoryCandidate。
8. 生成候选时必须保留 source refs，并声明“候选不是正式记忆”。
9. 记忆中心能看到 J3 capture 产生的 observation / candidate / source refs。
10. 任务记忆包预览能继续把 candidate / observation 作为 review materials，而不是 included formal memory。

## 3. J3 非目标

J3 不做：

- 不执行新的真实 Codex。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 auth/token/secret/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 不保存完整 prompt body、完整 transcript、raw stdout、raw stderr 或 runner noise。
- 不自动写 FormalMemory。
- 不绕过 M2 / M9 / M12 的确认权。
- 不开放 planned adapters 真实接入。
- 不做自动 retry / stop / restart。
- 不把 J3 完成说成 J4 / J5 / J6 或 Stage J 完成。

## 4. 数据契约

### 4.1 MemoryCaptureEvent

建议字段：

```text
capture_event_id
event_key
source_type
source_ref
project_id
project_root
workflow_id
workflow_node_id
run_unit_id
product_command_id
product_attempt_id
runtime_log_ref
audit_refs[]
readback_ref
task_package_ref
memory_packet_ref
summary
evidence_summary
sensitivity
candidate_policy
blocked_reason
observation_id
candidate_key
created_by
created_at
updated_at
```

实现可以独立新增 `memory-capture-events.v1.json`，也可以在不破坏现有 schema 的前提下复用既有 observation / candidate store 并新增前端 read model；但必须能证明 capture event 这一层的规则存在，不能只散落在 UI 文案里。

### 4.2 Source Ref 规则

允许进入 source refs：

- Product Command id / attempt id。
- runtime log ref。
- audit event id。
- readback summary ref。
- workflow id / node id / run unit id。
- task package ref / memory packet ref。
- evidence / handoff ref。

禁止进入 source refs 或正文：

- prompt body。
- full transcript。
- raw stdout / stderr。
- `/Users/yoyi/.codex` 内部路径内容。
- secret、token、`.env`、keychain、OAuth、provider credential、rollout。

## 5. UI 显示边界确认

本任务会改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [ ] 新增普通用户主导航入口。

普通 UI 应显示：

- 本轮操作是否形成 observation。
- 本轮操作是否形成 candidate。
- 来源类型和可读摘要。
- “候选不是正式记忆”。
- “正式化仍需确认”。
- 与 workflow / run unit / Product Command / runtime log / readback 的可追溯摘要。

普通 UI 禁止显示：

- raw capture JSON。
- sidecar 绝对路径。
- internal id 长列表。
- prompt body。
- full transcript。
- raw stdout / stderr。
- runner stderr 噪声作为主线证据。
- secret / credential。
- “系统已学习”“自动记住”“已写正式记忆”等越界文案。

显示位置：

- `记忆`：新增或加强 capture / observation / candidate source 摘要。
- `项目`：节点详情或自动编排摘要显示 capture 状态。
- `运行中工作流`：显示是否已生成 observation / candidate。
- `设置 / 开发者`：raw refs、diagnostics、sidecar 名称和内部边界放开发者区。

本任务不做手机端 UI，不新增 mobile responsive 规则。

## 6. 后端改动范围

允许改：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/task_memory_injection.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- 必要时新增小型模块，例如 `memory_capture_bus.rs`。

默认不改：

- FormalMemory schema。
- provider / credential store。
- planned adapter 真实执行逻辑。
- legacy / H5 真实 runner 产品入口。
- workflow state 顶层结构，除非只向既有数组追加 refs 且有测试覆盖。

## 7. 前端改动范围

允许改：

- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/memoryCenter.ts`
- `prototypes/productized-desktop-shell/src/lib/projectCanvas.ts`
- `prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

前端原则：

- 先让用户理解“这条操作变成了什么记忆候选”，不要把内部 refs 铺满普通界面。
- Capture / observation / candidate 要分层显示。
- 候选正式化按钮如果出现，必须走既有 M2 / M9 / M12 确认链路。

## 8. 验收标准

必须通过：

- Rust 单测覆盖 capture event 创建、audit-only 阻断、sensitive 阻断、observation 写入、candidate 生成、FormalMemory 不自动写入、corrupt JSON / revision conflict 不覆盖。
- 至少用 J2-B B2 的真实 run artifacts 或等价 fixture 生成一条 observation 和一条 MemoryCandidate。
- 记忆中心可见 capture / observation / candidate source 摘要。
- TaskMemoryPacket 继续把 candidate / observation 放入 review materials，不进入 included formal memory。
- `result_count=null` 的 readback unavailable / failed / timed out 不能显示成 0。
- prompt body / full transcript / secret / rollout 扫描无命中，或命中被分类为任务包中的禁止项 / 历史文案。

推荐命令：

```text
cargo test --lib memory_capture
cargo test --lib observation
cargo test --lib memory_candidate
cargo test --lib task_memory_packet
cargo test --lib project_workflow_automation
cargo test --lib
cargo fmt -- --check
npm run typecheck
npm run test:offline-interaction
npm run build
```

如果没有新增 Rust 模块名 `memory_capture`，对应命令应替换为实际模块测试名。

## 9. Evidence / Handoff

完成后新增：

- `evidence/2026-06-09-stage-j-j3-memory-capture-bus-and-candidate-generation-v1.md`
- `handoffs/2026-06-09-stage-j-j3-memory-capture-bus-and-candidate-generation-v1-result.md`

Evidence 必须记录：

- 改动范围。
- capture event 数据契约。
- observation / candidate 生成链路。
- J2-B B2 输入来源或 fixture 来源。
- FormalMemory 未自动写入证据。
- 敏感信息排除扫描。
- 测试结果。
- 真实执行边界：本任务未执行新的真实 Codex。

## 10. 复核线要求

完成实现和自测后，交给长期只读复核线审查：

- 是否存在 P0/P1 越界。
- 是否把 candidate / observation 冒充 FormalMemory。
- 是否保存了 prompt body / full transcript / secret-like 内容。
- 是否绕过统一 Product Command 或 M2 / M9 / M12 确认链路。
- 是否把 J2-B 或 J3 冒领为 Stage J 完成。

主管线只在复核线无 P0/P1 且 fresh verify 通过后，才能把 J3 收口为 `accepted_with_deferred_items` 或 `accepted`。

## 11. 完成后不得声明

- 不得声明 Stage J 完成。
- 不得声明任意项目无限制自由执行完成。
- 不得声明自动 retry / stop / restart 完成。
- 不得声明 planned adapters 真实接入。
- 不得声明 provider credential / model verification 完成。
- 不得声明 FormalMemory 自动写入完成。
- 不得声明真实 Tauri J5 验收完成。

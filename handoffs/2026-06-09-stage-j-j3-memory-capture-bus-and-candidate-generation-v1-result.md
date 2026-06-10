# Stage J / J3 Memory Capture Bus And Candidate Generation v1 Handoff

日期：2026-06-09

状态：已完成，结论为 `accepted_with_deferred_items`。复核线初审发现 P1，主管线已修补；复审确认无 P0/P1，允许收口。

## 1. 本轮做了什么

J3 新增 `MemoryCaptureEvent` / `memory-capture-events.v1.json`，把用户操作、Product Command、runtime log、readback、worker report、process fact decision、final review 等来源统一收敛为可审计 capture event，并按 `candidate_policy` 接入现有 observation / MemoryCandidate 链路。

当前链路保持边界：

- capture event 不是 FormalMemory。
- observation 不是 FormalMemory。
- MemoryCandidate 不是 FormalMemory。
- 正式化仍必须走既有 M2 / M9 / M12 用户确认、版本、审计和 lint / conflict 链路。

## 2. 代码落点

- `src-tauri/src/memory_capture_bus.rs`
- `src-tauri/src/types.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src/lib/types.ts`
- `src/lib/tauri.ts`
- `src/App.tsx`
- `src/lib/memoryCenter.ts`
- `src/views/MemoryCenterView.tsx`
- `src/lib/knowledgeBase.ts`
- `src/views/KnowledgeBaseView.tsx`

左侧栏核对结果：`src/lib/workbenchNavigation.ts` 已包含 `想法箱 / 知识库 / 记忆层` 主入口，图标为 inkwash 原型同组 `✎ / ▢ / ◐`；`运行中工作流` 图标为三条横向波浪线 `≋`。

## 3. 验证结果

已通过：

- `npm run typecheck`
- `cargo test --lib memory_capture`：7 passed
- `cargo test --lib observation`
- `cargo test --lib memory_candidate`
- `cargo test --lib task_memory_packet`
- `cargo test --lib project_workflow_automation`
- `cargo fmt -- --check`
- `cargo test --lib`：320 passed / 10 ignored
- `npm run test:offline-interaction`：13 passed
- `npm run build`：通过，仅既有 Vite chunk-size warning

## 4. 复核线初审修补

复核线初审结论：无 P0；发现 1 个 P1，不允许主管线直接收口 J3。

已修补：

- P1：`MemoryCenterView` 未接入 `memoryCaptureStore`，导致记忆中心 capture 区为空。现已在 `App.tsx` 中传入 `memoryCaptureStore={memoryCaptureStore}`。
- P2：补充 duplicate / revision conflict / corrupt JSON 三个测试，支撑“不覆盖 store”的证据。
- P2：明确 J3 capture 摘要区只读；记忆中心 / 知识库整体不能声称只读，因为仍保留既有治理动作。

复核线复审结论：

- P1 已关闭。
- 未发现新的 P0/P1。
- 允许主管线把 J3 收口为 `accepted_with_deferred_items`。

保留 P2：

- 跨 store 原子性仍是后续增强项：`candidate_allowed` 仍先写 observation/candidate，再 append capture event；J4/J5 可考虑事务化或补偿记录。
- 文案边界继续保持：只能说 J3 capture 摘要区只读，不能说记忆中心 / 知识库整体只读。

## 5. 边界和偏差

本轮没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有启动 Tauri / Browser / Chrome / 截图工具。产品实现和测试没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout 内容。

过程说明：主管线为处理用户 UI 设计要求按 Product Design skill 读取过 `.codex` 下的 skill / user-context 元数据，未读取会话、secret 或业务 transcript；因此复核时不要把本轮说成“过程层面完全没有访问 `.codex` 路径”。

## 6. 给复核线

请只读复核：

- 是否存在 P0/P1 越界。
- 是否把 capture / observation / candidate 冒充 FormalMemory。
- 是否保存 prompt body、full transcript、raw stdout/stderr、secret-like 内容。
- 是否绕过统一 Product Command 或 M2 / M9 / M12 正式记忆确认链路。
- 是否把 J3 冒领为 Stage J 完成。
- P1 是否已关闭：记忆中心能否实际接收 `memoryCaptureStore` 并显示 capture events。
- P2 测试补强是否足够支撑 corrupt JSON / revision conflict / duplicate 不覆盖 store。
- 是否需要把知识库中的 capture 摘要进一步收纳到详情，避免普通 UI 信息过载。

J3 已允许收口为 `accepted_with_deferred_items`。下一步进入 J4 运行队列、失败控制和用户确认队列；不得把 J3 说成 Stage J 完成。

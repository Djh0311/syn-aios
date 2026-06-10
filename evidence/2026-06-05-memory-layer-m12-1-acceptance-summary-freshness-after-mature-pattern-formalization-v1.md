# Evidence：Memory Layer M12.1 Acceptance Summary Freshness After Mature Pattern Formalization v1

日期：2026-06-05

## 结论

M12.1 已完成。`record_mature_pattern_decision` 在用户确认 mature pattern candidate 正式化并写入 formal mature pattern memory 后，当次返回的 `acceptance_summary` 已改为基于写入后的 fresh formal memory store。

接受为：

- 用户确认正式化后，`formal_memory` gate 能看到新写入的 record / version / audit。
- 用户确认正式化后，`task_packet` gate 能看到 active formal memory，不再因为旧空 store 报 `缺少 active formal memory`。
- reject / quarantine / request changes 等非正式化决定仍不写 formal store，summary 不误报新正式记忆。
- 用户确认 guard、mature pattern candidate 派生规则和 task packet recall 选择逻辑保持不变。

不接受为：

- M13 最终权威验收完成。
- M12 真实窗口 / 截图验收完成。
- 新增成熟模式、跨项目主题、UI、Tauri command、sidecar、worker 或 Codex 执行能力。

## 修补前问题

M12 的 `record_mature_pattern_decision` 会先加载 `formal_store`，再在 `ConfirmAsFormalMemory` 路径调用 `create_formal_mature_pattern_memory(...)` 写入正式记忆。

风险点是：写入完成后返回 `acceptance_summary` 时，仍可能把写入前加载的 `formal_store` 传给 `build_acceptance_summary(...)`。这样第一条 formal mature pattern memory 写入成功后，当次返回的 gate evidence 可能仍显示 `record 0 / version 0 / audit 0`，`task_packet` gate 也可能继续 blocked。

## 实际修补

后端：

- `prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_governance.rs`
  - 在 `formal_memory_output.is_some()` 时重新调用 `crate::formal_memory_store::load_store(workflow_state_path, timestamp)?`。
  - `acceptance_summary` 改为使用 `acceptance_formal_store`。
  - 非正式化决定继续使用原 `formal_store`，避免 reject / quarantine 等路径误报 fresh formal memory。

测试：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
  - `mature_pattern_user_confirmation_writes_formal_memory_and_task_packet_can_recall`
    - 断言 `formal_memory` gate status 为 `passed`。
    - 断言 gate evidence 包含 `record 1 / version 1 / audit 1`。
    - 断言 `task_packet` gate status 为 `passed`。
    - 断言 `task_packet` gate 没有 `blocking_reason`。
  - `mature_pattern_reject_quarantine_revision_and_damaged_json_do_not_mutate_formal_memory`
    - 断言 reject 的 `formal_memory_output` 为 `None`。
    - 断言 reject summary 的 `formal_memory` gate 为 `blocked`。
    - 断言 reject evidence 包含 `record 0 / version 0 / audit 0`。
    - 继续断言 quarantine 不写 formal store。

文档 / 入口：

- `tasks/2026-06-05-memory-layer-m12-1-acceptance-summary-freshness-after-mature-pattern-formalization-v1.md`
  - 状态已标记为 `已完成`。
- `tasks/README.md`
  - 最小同步 M12.1 已完成，下一步仍指向 M13。

## 验证

通过：

```text
cargo test --lib mature_pattern
5 passed; 0 failed
```

```text
cargo test --lib memory_cluster
2 passed; 0 failed
```

```text
cargo test --lib formal_memory
29 passed; 0 failed
```

```text
cargo test --lib task_memory_packet
10 passed; 0 failed
```

```text
cargo test --lib
221 passed; 0 failed; 1 ignored
```

说明：Rust 测试仍保留既有 `JsonRpcError::invalid_params` dead_code warning。

```text
rustfmt --check src/mature_pattern_governance.rs src/lib.rs
```

结果：通过。

```text
npm run typecheck
```

结果：通过。

未运行：

- `npm run test:offline-interaction`
- `npm run build`
- 真实窗口 / 截图验收

原因：M12.1 未改前端、UI 文案、读模型或可见交互，任务包默认不要求前端离线交互、build 或截图验收。

## 边界

- 未改 UI。
- 未改前端类型、Tauri wrapper、`MemoryCenterView.tsx`、`PermissionDialog.tsx` 或 `App.tsx`。
- 未新增 sidecar。
- 未新增 Tauri command。
- 未改变 mature pattern candidate 派生规则。
- 未改变用户确认 guard。
- 未改变 task packet recall 选择逻辑。
- 未改变 formal memory schema。
- 未接 GraphRAG、向量库、图数据库或自动索引重建。
- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec` 或 `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未推进 M13。

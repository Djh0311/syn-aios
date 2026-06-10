# Task Package：Memory Layer M12.1 Acceptance Summary Freshness After Mature Pattern Formalization v1

状态：已完成。  
用途：修补 M12 中 `record_mature_pattern_decision` 用户确认正式化后返回的 `acceptance_summary` 新鲜度问题。  
执行方式：小修补任务；只修 M12 acceptance summary 使用写入前 `formal_store` 的风险，不新增成熟模式功能，不重开 M12，不推进 M13。

预期回收记录：

- `evidence/2026-06-05-memory-layer-m12-1-acceptance-summary-freshness-after-mature-pattern-formalization-v1.md`
- `handoffs/2026-06-05-memory-layer-m12-1-acceptance-summary-freshness-after-mature-pattern-formalization-v1-result.md`

## 1. 先说薄弱点

- M12 已完成 mature pattern candidate、cluster report、用户确认正式化、task packet recall 和 M1-M12 gate 摘要。
- 全局主管复核发现一个窄风险：`record_mature_pattern_decision` 在写 formal mature pattern memory 前先加载 `formal_store`，但返回的 `acceptance_summary` 使用的仍可能是写入前的 `formal_store`。
- 相关代码位置：
  - `prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_governance.rs` 约第 75 行：写正式记忆前加载 `formal_store`。
  - 同文件约第 101-111 行：用户确认后调用 `create_formal_mature_pattern_memory`。
  - 同文件约第 172-180 行：返回 `acceptance_summary` 时仍传入旧 `formal_store`。
- 这不会推翻 M12 主链路，但会影响 M13 最终验收对 gate 摘要的可信度。
- M12.1 必须保持窄范围，不能借机新增功能、改 UI 或重构记忆系统。

## 2. 任务目标

修补后必须满足：

```text
user confirm mature pattern candidate
-> create formal mature pattern memory
-> reload or derive fresh formal store
-> build acceptance_summary from fresh formal store
-> returned gate evidence reflects newly written record / version / audit
```

M12.1 完成后可以说：

- `record_mature_pattern_decision` 用户确认正式化后返回的 `acceptance_summary` 使用写入后的 formal memory store。
- 当用户确认写入第一条 formal mature pattern memory 时，返回的 formal memory gate / task packet gate 不再基于旧空 store。
- 非正式化决定 reject / quarantine / request changes 仍不写 formal store，summary 仍可基于原 formal store。

M12.1 完成后仍不能说：

- M13 最终权威验收完成。
- M12 真实窗口 / 截图验收完成。
- 成熟模式、cluster report 或 maintenance report 可以绕过正式记忆状态机。
- 新增了成熟模式、跨项目主题或 UI 能力。
- 真实 worker / Codex 已执行。

## 3. 前置条件

必须已完成：

- `tasks/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1.md`

必须读取：

- `tasks/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1.md`
- `evidence/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1.md`
- `handoffs/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1-result.md`
- `prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_governance.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

## 4. 范围

允许：

- 修改 `prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_governance.rs`。
- 修改 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 中 M12 相关 Rust 测试。
- 如必须，可以极小幅修改 `types.rs`，但只有在现有类型无法表达新鲜度断言时才允许。
- 在 `record_mature_pattern_decision` 成功写 formal mature pattern memory 后，重新加载 `formal_memory_store::load_store`，或用 `formal_memory_output` 构造等价的新鲜 formal store 输入。
- 新增 / 修改测试，断言返回的 `acceptance_summary` 已反映新写入的 formal mature pattern memory。
- 更新 M12.1 evidence / handoff。
- 如当前入口文档需要记录 M12.1 已完成，可在执行完成后由回收者最小同步；但本任务包不要求大规模阶段计划改写。

本任务显式授权的数据变更：

- 允许测试环境继续写临时 `formal-memories.v1.json` 和 `memory-patterns.v1.json`。
- 允许生产逻辑在用户确认正式化时按 M12 既有路径写 `formal-memories.v1.json` 和 `memory-patterns.v1.json`。

禁止：

- 不新增 sidecar。
- 不新增 Tauri command。
- 不新增前端类型、Tauri wrapper、读模型、按钮或 UI 文案。
- 不改 `MemoryCenterView.tsx`、`PermissionDialog.tsx`、`App.tsx` 或前端测试，除非发现现有前端类型被后端类型破坏；正常情况下不应改前端。
- 不改变 mature pattern candidate 派生规则。
- 不改变用户确认 guard。
- 不改变 task packet recall 选择逻辑。
- 不接 GraphRAG、向量库、图数据库或自动索引重建。
- 不执行真实 worker。
- 不执行真实 Codex。
- 不执行 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不把 M12.1 说成 M13 最终验收完成。

如果执行者认为必须改 UI、改 task packet 选择器、改正式记忆 schema、改 mature pattern 派生规则或推进 M13，必须停下回传。

## 5. UI 显示边界

UI 显示边界：本任务不改前端、不改读模型、不改 UI 文案；因此不需要 UI 验收。

如果执行中发现必须改 UI，必须停下回传，不得自行扩大范围。

## 6. 实施建议

建议按以下顺序修：

1. 在 `record_mature_pattern_decision` 中区分两种路径：
   - `ConfirmAsFormalMemory` 成功写 formal store 后，重新加载 fresh formal store。
   - `Reject` / `Quarantine` / `RequestChanges` 不写 formal store，可继续使用原 formal store。
2. 构建 `acceptance_summary` 时传入 fresh formal store。
3. 保持 `formal_memory_output` 原样返回。
4. 增加测试覆盖：
   - 项目主管确认仍被拒绝。
   - 用户确认成功后返回的 `acceptance_summary.gates` 中 `formal_memory` gate evidence 包含 `record 1 / version 1 / audit 1` 或等价新鲜计数。
   - 用户确认成功后返回的 `acceptance_summary.gates` 中 `task_packet` gate 不再因为“缺少 active formal memory”而 blocked。
   - reject / quarantine 仍不改 formal store。
5. 不改前端，不跑浏览器验收。

实现方式建议：

```text
let acceptance_formal_store = if formal_memory_output.is_some() {
    crate::formal_memory_store::load_store(workflow_state_path, timestamp)?
} else {
    formal_store
};
```

如果因为 borrow / ownership 需要调整变量名，应保持最小改动，不重构整段逻辑。

## 7. 验收

必须通过：

```text
cargo test --lib mature_pattern
cargo test --lib memory_cluster
cargo test --lib formal_memory
cargo test --lib task_memory_packet
cargo test --lib
rustfmt --check src/mature_pattern_governance.rs src/lib.rs
```

如果实际修改了其他 Rust 文件，必须把对应文件加入 `rustfmt --check`。

建议但非必须：

```text
npm run typecheck
```

说明：本任务默认不改前端，因此不要求 `npm run test:offline-interaction`、`npm run build` 或截图验收；如果执行者实际改了前端，则必须补完整前端验证和 UI evidence。

必须覆盖的断言：

- 用户确认正式化后，返回的 `acceptance_summary` 使用 fresh formal store。
- 用户确认正式化后，formal memory gate 能看到新写入的 record / version / audit。
- 用户确认正式化后，task packet gate 能看到 active formal memory。
- 非正式化决定不写 formal store，不误报 fresh formal memory。
- 用户确认 guard 不被削弱。

## 8. evidence / handoff 要求

M12.1 完成后必须新增：

- `evidence/2026-06-05-memory-layer-m12-1-acceptance-summary-freshness-after-mature-pattern-formalization-v1.md`
- `handoffs/2026-06-05-memory-layer-m12-1-acceptance-summary-freshness-after-mature-pattern-formalization-v1-result.md`

evidence 必须记录：

- 修补前问题：`acceptance_summary` 可能使用写入前 formal store。
- 实际修补方式：重新加载 formal store 或等价 fresh summary 输入。
- 新增 / 修改的测试和断言。
- 验证命令和结果。
- 边界：未改 UI、未新增能力、未执行真实 worker / Codex、未读写 `/Users/yoyi/.codex`、未推进 M13。

handoff 必须写清：

- M12.1 接受为什么。
- M12.1 不接受为什么。
- M13 是否可以继续拆 / 执行。
- 是否仍有 M12 真实窗口 / 截图验收缺口。

## 9. Stop 条件

遇到以下情况必须停下回传：

- 需要改前端或 UI 文案。
- 需要新增 sidecar、command 或用户可见功能。
- 需要改变 mature pattern candidate 派生规则。
- 需要改变正式记忆 schema 或 task packet 召回选择器。
- 需要读写 `/Users/yoyi/.codex`。
- 需要执行真实 worker / Codex。
- 需要把 M12.1 扩大成 M13 最终验收。

## 10. 回收口径

完成后接受为：

- M12.1 acceptance summary 新鲜度修补完成。
- 用户确认 mature pattern 正式化后，当次返回的 M1-M12 gate 摘要基于写入后的 formal store。
- M13 可以继续进入最终权威验收任务包。

完成后不接受为：

- M12 新功能新增。
- M13 最终权威验收完成。
- M12 真实窗口 / 截图验收完成。
- 成熟模式自动技能化、自动全局规则或跨项目摘要直接影响 worker 完成。
- 真实 worker / Codex 已执行。

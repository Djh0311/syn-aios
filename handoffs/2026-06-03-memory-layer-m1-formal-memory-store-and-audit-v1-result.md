# memory-layer M1 formal memory store and audit handoff v1

日期：2026-06-03

## 结论

本轮完成 M1：正式记忆受控存储和审计骨架。

接受为：

- 新增正式记忆独立 sidecar：`<workflow_state_dir>/formal-memories.v1.json`。
- 显式 `create_formal_memory_record` 能同步写入 `MemoryRecord`、第一版 `MemoryVersion` 和 `MemoryAuditEvent(memory_record_created)`。
- `load_formal_memory_store` 能只读加载正式记忆 store。
- 写入使用 lock、revision、backup、tmp + rename。
- 损坏 JSON 读取时拒绝覆盖。
- 无来源正式记忆会被拒绝。
- 候选状态不能作为正式记忆初始状态。
- `candidate_confirmed` 不会自动创建正式记忆。
- 项目页和记忆入口能只读显示正式记忆骨架摘要。

不接受为：

- 候选采纳流程完成。
- 任务包召回完成。
- 任务包注入完成。
- 完整记忆管理页面完成。
- 正式记忆生命周期操作完成。
- 中间版本记忆层完成。
- Obsidian / 知识库集成完成。
- 向量库 / 图数据库完成。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `CURRENT.md`
- `tasks/README.md`
- `evidence/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- `handoffs/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1-result.md`

## 可操作状态

后端命令：

- `load_formal_memory_store`
- `create_formal_memory_record`

前端：

- `loadFormalMemoryStore()`
- `createFormalMemoryRecord()`
- `summarizeFormalMemoryStore()`
- 项目工作流侧栏“候选治理”卡显示正式记忆 sidecar、revision、record/version/audit 计数和最近 audit 类型。
- “记忆”入口显示正式记忆 sidecar、数量和 revision。

## 手动检查清单

1. 打开应用，进入“项目”。
2. 选择已有项目。
3. 进入“项目工作流”。
4. 在项目画布侧栏的“候选治理”卡中检查：
   - 显示 `formal-memories.v1.json`。
   - 显示“正式记忆骨架”。
   - 显示“创建时写入 version 和 audit”。
   - 显示“M1 不包含候选采纳和任务包注入”。
5. 进入“记忆”入口，检查显示正式记忆 sidecar、数量和 revision。
6. 不应出现“候选已记住”“系统已学习”“正式记忆完整完成”“任务包注入已完成”。

文件层检查：

1. 找到当前 `workflow-state.v0.json` 所在目录。
2. 显式创建正式记忆后，应出现 `formal-memories.v1.json`。
3. `workflow-state.v0.json` 不应新增正式记忆字段。
4. `memory-candidates.v1.json` 仍是候选 store，不应被自动写入正式记忆。
5. 修改已有正式记忆文件后再次创建，会在 `backups/` 下出现正式记忆 sidecar 备份。

## 验证

已通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib
rustfmt --check src/formal_memory_store.rs
curl -sS -I http://127.0.0.1:5174/
```

结果摘要：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 9`。
- `npm run build`：通过；Vite 仍提示 chunk > 500 kB。
- `cargo test --lib`：通过，122 passed，1 ignored；仍有既有 `JsonRpcError::invalid_params` unused warning。
- `rustfmt --check src/formal_memory_store.rs`：通过。
- `curl -sS -I http://127.0.0.1:5174/`：通过，HTTP 200；Vite dev server 当前运行在 `http://127.0.0.1:5174/`。

未验证：

- 未做真实浏览器或 Tauri 窗口截图验收。当前工具发现没有 in-app browser 控制工具，项目也没有 Playwright 依赖；本轮未安装新依赖。

## 边界

- 未执行真实 Codex。
- 未执行 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未改 `workflow-state.v0.json` 结构。
- 未迁移数据库。
- 未接 Obsidian / 知识库。
- 未接向量库 / 图数据库。
- 未做 M2 / M4 / M6 / M9。

## 下一步

下一步如果继续记忆层，建议进入 M2：候选到正式记忆的受控采纳。

M2 开始前需要单独任务包确认：

- 谁能采纳候选。
- 哪些记忆必须用户确认。
- 采纳后候选状态如何保留历史。
- 如何把候选来源复制到正式记忆。
- 不能让秘书、worker 或黑板候选直接写正式记忆。

# Task Package：工作流派发 readback 迁到 Rust parser v1

状态：已完成。  
用途：清理会话中心硬化后留下的最后一条旧 Python transcript reader 依赖：工作流派发结果 readback。  
执行方式：一个小批次完成；不拆微任务；最终统一验收。

完成记录：

- evidence：`../evidence/2026-06-03-workflow-dispatch-readback-native-parser-v1.md`
- handoff：`../handoffs/2026-06-03-workflow-dispatch-readback-native-parser-v1-result.md`
- 结论：工作流派发 readback stats 主路径已迁到 Rust 原生 transcript parser；旧 `transcript_reader.py` / `Command::new("python3")` 只保留在 deprecated 兼容函数中，不再被 dispatch readback stats 主路径调用。

## 1. 先说薄弱点

`session-center-foundation-hardening-v1` 已经把会话中心 transcript 主读取路径迁到 Rust 原生 JSONL parser。

但工作流派发后读回执行结果的 `dispatch_readback_stats` 仍调用：

```text
load_codex_session_transcript_with_reader(index_path, transcript_reader, thread_id)
```

这条旧函数内部仍有：

```text
Command::new("python3")
```

风险：

- 工作流派发 readback 仍依赖 Python 子进程。
- readback 仍依赖静态 index / Python reader 的旧契约。
- 出错时容易退化为 `0 events / 0 hits`，问题被吞掉。
- 如果以后彻底删除 Python reader，会打断工作流派发结果读回。

一句话目标：

```text
工作流派发 readback stats 使用和会话中心相同的 Rust 原生 transcript parser；
Python reader 不再参与工作流派发 readback 主路径。
```

## 2. 必须先读

当前入口：

- `CURRENT.md`
- `AUTHORITY.md`
- `tasks/README.md`
- `docs/workbench-system-architecture-v1.md`

前置依据：

- `tasks/2026-06-03-session-center-foundation-hardening-v1.md`
- `evidence/2026-06-03-session-center-foundation-hardening-v1.md`
- `handoffs/2026-06-03-session-center-foundation-hardening-v1-result.md`

主要代码入口：

- `prototypes/productized-desktop-shell/src-tauri/src/codex_transcript.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/codex_db.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

重点搜索：

- `dispatch_readback_stats`
- `load_codex_session_transcript_with_reader`
- `Command::new("python3")`
- `transcript_reader.py`
- `read_workflow_node_dispatch_result_at`
- `execute_workflow_node_dispatch_at`

搜索固定文本必须用 `rg -F '...'` 或单引号，禁止把含反引号文本放进 shell 双引号。

## 3. 已知事实 / 未知 / 假设

已知事实：

- `codex_transcript.rs` 已能用 Rust 原生 parser 读取 rollout JSONL。
- `load_codex_session_transcript_for_index` 已支持 sqlite 优先、index 可选 fallback。
- 会话中心主读取路径不再使用 Python reader。
- `dispatch_readback_stats` 当前仍通过旧 reader 读取 transcript，再统计：
  - `transcript_event_count`
  - `transcript_target_hits`
- 工作流派发 readback 失败目前会回落到 `0 / 0` stats。
- `commands.rs` 里仍为 execute/readback/workflow-machine 构造 `transcript_reader.py` 路径。

未知：

- 工作流机器和手动 readback 是否都必须保留完全相同的失败降级语义。
- 当前 fixture 是否覆盖了 safe probe target、业务派发 readback、workflow machine readback 三类路径。
- 旧 Python reader 是否还有除 readback stats 以外的实际产品路径依赖。

本任务包假设：

- 本轮只迁工作流派发 readback stats 的 transcript 读取依赖。
- 不执行真实 Codex。
- 不写真实 `/Users/yoyi/.codex`。
- 不改 workflow state JSON 结构。
- 不改变工作流状态机和总指导决策。

## 4. 范围

允许：

- 新增或复用后端 helper，让 readback stats 走 Rust native parser。
- 调整 `dispatch_readback_stats` 参数，让它不再需要 `transcript_reader: &Path`。
- 调整 `execute_workflow_node_dispatch_at` / `read_workflow_node_dispatch_result_at` / workflow machine 相关调用点里 readback stats 的参数传递。
- 保留旧 `load_codex_session_transcript_with_reader` 作为临时兼容，但本轮必须证明 readback stats 主路径不调用它。
- 给旧 Python reader 路径加 `deprecated` 注释或缩小可见性。
- 补 Rust 单元测试和离线测试。
- 更新 evidence / handoff / 当前入口文档。

禁止：

- 不执行 `codex exec`。
- 不执行 `codex exec resume`。
- 不启动真实 workflow machine。
- 不改真实 workflow state JSON。
- 不迁移数据库。
- 不写正式事实。
- 不写正式记忆。
- 不改变工作流状态机。
- 不改变派发成功 / 失败 / 超时的状态推进规则。
- 不改变总指导回收策略。
- 不删除 `prototypes/index-kernel/transcript_reader.py`。
- 不顺手把 workflow machine 全量拆模块。
- 不读写 `/Users/yoyi/.codex`。
- 不读取真实完整 transcript，除非执行线程另行获得用户明确批准。
- 不接 Claude / OpenClaw / OpenCode。
- 不做真实 Tauri 窗口验收，除非本轮明确需要 UI 证据。

## 5. 执行段 A：设计 native readback helper

目标：

- readback stats 使用 Rust 原生 parser 读取目标 thread。
- 复用会话中心已经建立的 sqlite / index 权威关系。

建议实现：

1. 新增 helper，例如：

```text
dispatch_readback_stats_native(index: Option<&Value>, thread_id, target, db_path)
```

或直接让现有 `dispatch_readback_stats` 调用：

```text
load_codex_session_transcript_with_optional_catalog(...)
```

2. `safe_probe_target()` 仍作为默认命中目标。
3. stats 统计逻辑保持不变：
   - `transcript_event_count = transcript.summary.total_events`
   - `transcript_target_hits` 统计 event text / stdout 中包含 target 的次数
4. readback 失败是否继续降级为 `0 / 0` 可以保留，但必须记录 warning 或 evidence 说明该行为是兼容旧语义。

验收：

- `dispatch_readback_stats` 主路径不再调用 `load_codex_session_transcript_with_reader`。
- readback 可以读取 sqlite-only thread。
- readback 不受静态 index 缺 thread 或损坏影响。
- readback 仍能用 index fallback 读取旧 fixture。
- target hit 统计和旧逻辑一致。

## 6. 执行段 B：收敛调用点

目标：

- 工作流派发 execute 成功后的 readback。
- 手动读取节点派发结果 readback。
- workflow machine 中依赖派发 readback stats 的路径。

必须检查并处理：

- `execute_workflow_node_dispatch_at`
- `read_workflow_node_dispatch_result_at`
- `run_workflow_machine_at`
- `run_workflow_machine_for_index_at`
- `RealCodexResumeRunner` / stub runner 的 `readback_stats` 注入语义
- `commands.rs` 中传入 `transcript_reader.py` 的包装层

边界：

- 如果某些函数保留 `transcript_reader: &Path` 参数只是为了其他旧路径，本轮可以不删除，但 evidence 必须说明哪些参数已无 readback stats 主路径用途。
- 如果删除参数会引发大面积签名变更，优先小步：先让 readback stats 不使用该参数，再在后续清理任务删签名。

验收：

- execute 成功路径仍会写 completed dispatch。
- readback button / command 仍会更新 dispatch readback stats。
- workflow machine stub 测试仍通过。
- 没有真实执行 Codex。

## 7. 执行段 C：保留或废弃旧 Python reader 的边界

目标：

- 明确 Python reader 是否仍有产品路径。

要求：

1. 搜索 `Command::new("python3")`：
   - 如果只剩旧兼容函数且无 readback 主路径调用，记录为“可后续删除”。
   - 如果还有实际路径调用，必须列出调用路径和未迁移原因。
2. 搜索 `transcript_reader.py`：
   - 区分测试 fixture、旧兼容函数、产品路径。
3. 不删除 `prototypes/index-kernel/transcript_reader.py`。
4. 不把“Python reader 文件还存在”误写成“产品仍依赖 Python reader”；要区分文件存在和主路径调用。

验收：

- evidence 明确写出 Python reader 当前是否仍被产品路径调用。
- 如果仍有调用，必须说明不是本轮 readback stats 主路径，或判定任务未完成。

## 8. 测试要求

必须补 Rust 测试，至少覆盖：

1. readback stats 用 sqlite-only thread 读取 native rollout。
2. readback stats 在 index unavailable 时仍能读取 sqlite thread。
3. readback stats 在 sqlite unavailable 时可走 index fallback。
4. readback stats 命中 `safe_probe_target()`。
5. readback stats 不命中时返回 0 hits。
6. readback 读取失败时保留旧兼容降级行为，或如果改为 warning/error，必须覆盖新语义。
7. execute dispatch 成功路径使用 stub readback stats 时仍不触发 native 读取。
8. execute dispatch 成功路径没有 stub stats 时走 native readback。
9. `Command::new("python3")` 不在 readback stats 主路径测试中被需要。

建议补一个静态断言或普通测试：

- 没有 `transcript_reader.py` 文件也能完成 native readback stats fixture。

如果改前端，才补 `npm run test:offline-interaction` 中的前端断言；本轮理论上可以只改 Rust 后端和文档。

## 9. 验证命令

在：

```text
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell
```

必须跑：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

在：

```text
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri
```

必须跑：

```text
cargo test --lib
rustfmt --check src/codex_transcript.rs src/codex_db.rs
```

如果改了其他 Rust 新模块，也要加入 `rustfmt --check`。

不要默认跑全仓库 `cargo fmt --check` 修历史格式债；如果仍因既有 `src/lib.rs` 或 `src/mcp/**` 失败，记录即可。

## 10. 真实验证

本轮默认不做真实 Codex 验证。

如果执行线程想做真实 workflow readback，必须先停下来获得用户明确确认，因为这可能涉及：

- 读取真实 `.codex` rollout。
- 修改真实 workflow state。
- 触发真实 Codex 相关流程。

没有明确确认时，只能用临时 fixture / stub runner / 单元测试验证。

## 11. 验收标准

接受为：

- 工作流派发 readback stats 主路径使用 Rust 原生 parser。
- readback stats 不再依赖 Python 子进程。
- readback stats 不再把静态 index 当 sqlite 会话准入名单。
- execute/readback/workflow-machine 相关测试通过。
- 旧 Python reader 的残留路径被清楚标注。
- 未执行真实 Codex。
- 未读写 `/Users/yoyi/.codex`。
- 未改 workflow state JSON 结构。

不接受为：

- Python reader 文件已删除。
- 整个 app 完全没有 Python 代码。
- 工作流机器重构完成。
- 真实业务派发验证完成。
- 总指导回收策略完成。
- 会话中心真实 Tauri 窗口验收完成。
- Claude / OpenClaw / OpenCode 接入完成。

## 12. 必须输出

执行完成后必须新增：

- `evidence/2026-06-03-workflow-dispatch-readback-native-parser-v1.md`
- `handoffs/2026-06-03-workflow-dispatch-readback-native-parser-v1-result.md`

并更新：

- `CURRENT.md`
- `tasks/README.md`

handoff 必须包含：

- 实际改动文件。
- 仍包含 `Command::new("python3")` 的位置和原因。
- readback stats 是否仍调用旧 reader。
- 验证命令和结果。
- 是否读取过真实 `/Users/yoyi/.codex`。
- 是否执行过真实 Codex。

## 13. 下一步建议

本任务完成后，最近可以继续二选一：

1. 单开“彻底删除旧 Python reader 产品路径 / 签名清理”任务，如果确认没有任何产品路径需要它。
2. 单开“Agent adapter 后端能力声明”任务，把前端只读 adapter descriptor 收敛到后端 `agent_adapters[]` 读模型。

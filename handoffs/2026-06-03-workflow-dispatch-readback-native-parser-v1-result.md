# Handoff: workflow dispatch readback native parser v1

日期：2026-06-03

## 结论

`tasks/2026-06-03-workflow-dispatch-readback-native-parser-v1.md` 已完成。

接受为：

- 工作流派发 readback stats 主路径已使用 Rust 原生 transcript parser。
- `dispatch_readback_stats` 不再调用旧 `load_codex_session_transcript_with_reader(...)`。
- execute 成功 readback、手动 readback、workflow machine 派发 readback 都通过 sqlite / index catalog helper 读取 transcript。
- runner 已注入 `readback_stats` 时不会触发 native transcript read。
- 读取失败仍保持旧兼容降级：`transcript_event_count = 0`、`transcript_target_hits = 0`。

不接受为：

- 真实 Codex 验证完成。
- 真实 workflow machine 执行完成。
- 工作流状态机重构完成。
- workflow state JSON 结构变更完成。
- 旧 Python reader 文件已删除。
- 总指导回收策略已改。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `tasks/2026-06-03-workflow-dispatch-readback-native-parser-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `evidence/2026-06-03-workflow-dispatch-readback-native-parser-v1.md`
- `handoffs/2026-06-03-workflow-dispatch-readback-native-parser-v1-result.md`

## 关键实现点

- `dispatch_readback_stats_native(...)` 复用 `load_codex_session_transcript_with_optional_catalog(...)`，因此读取权威关系和会话中心一致：sqlite 优先，index 兼容 fallback。
- `execute_workflow_node_dispatch_at(...)` 从 `unwrap_or(...)` 改成 `match`，避免 runner 已有 stub stats 时仍急切读取 transcript。
- `commands.rs` 中 execute / readback / workflow machine wrapper 不再构造 `transcript_reader.py`，改为传 `codex_db::default_state_db_path()`。
- 新写入的 readback audit 元数据改为 native parser 命名；未迁移历史 workflow state。

## 剩余 Python reader 位置

桌面壳产品代码里不再包含 `Command::new("python3")`：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`：无。
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`：无。

原因：

- 复核后已删除桌面壳里的 deprecated 兼容函数 `load_codex_session_transcript_with_reader(...)`。
- 本轮仍未删除 `prototypes/index-kernel/transcript_reader.py` 文件本身。
- 当前 readback stats 主路径已不调用旧 reader。
- `transcript_reader.py` 文本仍会出现在测试 fixture / 断言中；这不是产品路径依赖。

## 验证结果

在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell`：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，9 个 offline interaction scenarios。
- `npm run build`：通过；Vite 有 chunk size warning。

在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri`：

- `cargo test --lib`：通过，114 passed / 0 failed / 1 ignored；有既有 `JsonRpcError::invalid_params` unused warning。
- `rustfmt --check src/codex_transcript.rs src/codex_db.rs`：通过。

额外检查：

- `rustfmt --check src/codex_transcript.rs src/codex_db.rs src/lib.rs src/commands.rs`：失败，暴露 `src/lib.rs` 和 `src/mcp/**` 既有格式债；未批量格式化，避免无关 diff。

## 手动测试清单

不执行真实 Codex 的手动检查：

1. 打开工作台项目页，选择一个有本地 workflow 的项目。
2. 找到已有派发记录，点击“读取结果 / readback”类入口。
3. 预期：UI 仍能刷新派发统计；如果临时 fixture 或本地状态没有对应 transcript，统计兼容显示 `0 / 0`，不报 Python reader 缺失。
4. 查看日志或后端错误：不应出现找不到 `transcript_reader.py` 的错误。
5. 运行离线测试确认项目页交互不退化：`npm run test:offline-interaction`。

需要真实 Codex 的手动检查暂不执行：

1. 真实 `codex exec resume` 派发。
2. 真实 workflow machine run。
3. 读取真实 `/Users/yoyi/.codex` rollout。

以上三类必须另行获得用户明确批准。

## 边界声明

- 本轮没有读取或写入 `/Users/yoyi/.codex`。
- 本轮没有执行真实 `codex exec` / `codex exec resume`。
- 本轮没有读取真实完整 transcript。
- 本轮没有启动真实 workflow machine。
- 本轮没有修改 workflow state JSON 结构。
- 本轮没有迁移数据库。
- 本轮没有删除 `prototypes/index-kernel/transcript_reader.py`。

## 后续建议

最近可选下一步：

1. 单开“index-kernel Python reader 是否归档 / 删除”任务，前提是确认 `prototypes/index-kernel/transcript_reader.py` 不再被任何索引工具或历史流程需要。
2. 单开“Agent adapter 后端能力声明”任务，把前端只读 adapter descriptor 收敛到后端 `agent_adapters[]` 读模型。
3. 单开真实 Tauri 窗口验收或项目工作流画布深化，不和 readback 迁移混在一起。

## 复核后补充

用户要求“旧的要删掉”后，已删除桌面壳 deprecated 兼容函数，并重新验证：

- `Command::new("python3")`：桌面壳 `src-tauri/src/lib.rs` / `commands.rs` 无残留。
- `load_codex_session_transcript_with_reader(...)`：桌面壳 `src-tauri/src/lib.rs` / `commands.rs` 无残留。
- `cargo test --lib`：通过，`114 passed; 0 failed; 1 ignored`。
- `rustfmt --check src/codex_transcript.rs src/codex_db.rs`：通过。
- `npm run test:offline-interaction`：通过，9 个 offline interaction scenarios。

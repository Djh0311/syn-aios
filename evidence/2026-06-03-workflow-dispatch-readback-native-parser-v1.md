# Evidence: workflow dispatch readback native parser v1

日期：2026-06-03

## 范围

执行任务包：`tasks/2026-06-03-workflow-dispatch-readback-native-parser-v1.md`

本轮只迁移工作流派发 readback stats 主路径：

- `dispatch_readback_stats`
- execute 成功后的 readback stats
- 手动 `read_workflow_node_dispatch_result` readback
- workflow machine 相关派发 readback
- Tauri command wrapper 中的 readback 参数传递

未做：

- 未执行真实 `codex exec` / `codex exec resume`
- 未启动真实 workflow machine
- 未读写 `/Users/yoyi/.codex`
- 未读真实完整 transcript
- 未改 workflow state JSON 结构
- 未改工作流状态机
- 未改变派发成功 / 失败 / 超时推进规则
- 未改变总指导回收策略
- 未迁移数据库
- 未删除 `prototypes/index-kernel/transcript_reader.py`

## 实现证据

### readback 主路径已迁 Rust native

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs:6954`：`dispatch_readback_stats` 改为调用 `dispatch_readback_stats_native(...)`。
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs:6967`：`dispatch_readback_stats_native(...)` 复用 `load_codex_session_transcript_with_optional_catalog(...)`，读取 sqlite / index catalog 后交给 Rust 原生 JSONL parser。
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs:6991`：统计逻辑保持不变：
  - `transcript_event_count = transcript.summary.total_events`
  - `transcript_target_hits` 统计 event `text` / `stdout` 包含 target 的次数
- readback 失败仍兼容旧语义，降级为 `0 / 0`，见 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs:6984`。

### dispatch / workflow machine 调用点已切换

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs:2115`：execute 成功后优先使用 runner 注入的 `readback_stats`；没有注入时才调用 native readback。
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs:2181`：手动 readback 调用 native readback。
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs:368`：execute command wrapper 使用 `codex_db::default_state_db_path()`，不再构造 `transcript_reader.py`。
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs:397`：manual readback command wrapper 使用 `codex_db::default_state_db_path()`。
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs:519`：workflow machine command wrapper 使用 `codex_db::default_state_db_path()`。
- 新写入的 readback audit 元数据已从旧 `desktop_shell_transcript_reader` / `transcript_reader_stats` 改为 `desktop_shell_native_transcript_parser` / `native_transcript_readback_stats`；不迁移历史 workflow state。

### stub readback stats 不触发 native read

旧代码使用 `unwrap_or(dispatch_readback_stats(...)?)`，会急切求值。本轮改为 `match` 懒分支：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs:2115`

测试证据：

- `workflow_node_dispatch_execute_uses_stub_and_advances_to_review` 用测试计数器确认 stub `readback_stats` 分支没有触发 native read。
- `workflow_node_dispatch_execute_without_stub_stats_uses_native_readback` 用无 stats 的 stub runner 确认 execute 成功后会触发 native readback。

## Python reader 残留边界

搜索结果：

```text
rg -n -F -e 'load_codex_session_transcript_with_reader(' -e 'Command::new("python3")' -e 'transcript_reader.py' prototypes/productized-desktop-shell/src-tauri/src/lib.rs prototypes/productized-desktop-shell/src-tauri/src/commands.rs
```

结论：

- 复核后已删除桌面壳里的 deprecated 兼容函数 `load_codex_session_transcript_with_reader(...)`。
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 和 `src-tauri/src/commands.rs` 中不再包含 `Command::new("python3")`。
- 当前 dispatch readback stats 主路径不再调用旧 reader。
- `commands.rs` 中 execute / readback / workflow machine command wrapper 不再构造 `transcript_reader.py` 路径。
- `transcript_reader.py` 字符串仍会出现在测试 fixture / 断言中；这不是产品路径依赖。

保留边界：

- 任务包明确禁止删除 `prototypes/index-kernel/transcript_reader.py`。
- 本轮已删除桌面壳旧兼容函数，但没有删除 index-kernel 下的 Python reader 文件。

## 新增 / 调整测试

新增或调整的 Rust 测试覆盖：

- sqlite-only thread + native rollout readback
- index unavailable 时仍能读 sqlite thread
- sqlite unavailable 时可走 index fallback
- `safe_probe_target()` 在 text / stdout 中命中
- target missing 时 hits 为 0
- readback 失败保持旧 `0 / 0` 降级
- execute 成功且 runner 注入 stub stats 时不触发 native read
- execute 成功且 runner 不注入 stats 时触发 native readback
- 无 `transcript_reader.py` 文件也能完成 native readback fixture

## 验证命令

工作目录：`/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell`

```text
npm run typecheck
```

结果：通过。

```text
npm run test:offline-interaction
```

结果：通过，`offline interaction tests passed: 9`。

```text
npm run build
```

结果：通过；Vite 仍提示 chunk size warning，这是构建警告，不是失败。

工作目录：`/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri`

```text
cargo test --lib
```

结果：通过，`114 passed; 0 failed; 1 ignored`。存在既有 warning：`JsonRpcError::invalid_params` 未使用。

```text
rustfmt --check src/codex_transcript.rs src/codex_db.rs
```

结果：通过。

额外执行：

```text
rustfmt --check src/codex_transcript.rs src/codex_db.rs src/lib.rs src/commands.rs
```

结果：失败。原因是 `src/lib.rs` 和 `src/mcp/**` 既有大量格式债会被 rustfmt 重排；本轮未批量格式化，避免无关 diff。任务包要求的 `src/codex_transcript.rs src/codex_db.rs` 格式检查已通过。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `tasks/2026-06-03-workflow-dispatch-readback-native-parser-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `evidence/2026-06-03-workflow-dispatch-readback-native-parser-v1.md`
- `handoffs/2026-06-03-workflow-dispatch-readback-native-parser-v1-result.md`

## 边界声明

- 本轮没有执行真实 Codex。
- 本轮没有执行 `codex exec` / `codex exec resume`。
- 本轮没有读取或写入 `/Users/yoyi/.codex`。
- 本轮没有读取真实完整 transcript。
- 本轮没有改 workflow state JSON 结构。
- 本轮没有迁移数据库。
- 本轮没有删除旧 Python reader 文件。

## 复核后补充

用户要求“旧的要删掉”后，已删除桌面壳未调用的 deprecated 兼容函数 `load_codex_session_transcript_with_reader(...)`，同时确认：

- `rg -n -F 'Command::new("python3")' prototypes/productized-desktop-shell/src-tauri/src/lib.rs prototypes/productized-desktop-shell/src-tauri/src/commands.rs`：无输出。
- `rg -n -F 'load_codex_session_transcript_with_reader(' prototypes/productized-desktop-shell/src-tauri/src/lib.rs prototypes/productized-desktop-shell/src-tauri/src/commands.rs`：无输出。
- `cargo test --lib`：通过，`114 passed; 0 failed; 1 ignored`。
- `rustfmt --check src/codex_transcript.rs src/codex_db.rs`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 9`。

仍未删除 `prototypes/index-kernel/transcript_reader.py` 文件本身；它属于 index-kernel 历史工具，不是桌面壳 readback 产品路径。

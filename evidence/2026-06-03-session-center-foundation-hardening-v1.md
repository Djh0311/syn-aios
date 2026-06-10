# Evidence：session-center-foundation-hardening-v1

日期：2026-06-03

## 范围

执行任务包：`tasks/2026-06-03-session-center-foundation-hardening-v1.md`

本轮接受目标：

- sqlite 作为会话中心目录主权威。
- `index.json` 降为缓存 / 兼容 / 辅助，不再作为 sqlite 会话的 transcript 准入名单。
- 会话中心 transcript 主读取路径迁到 Rust 原生 JSONL parser。
- 前端补对话清洗、搜索过滤、用户控制收纳、固定框架、错误分类和孤儿 CSS 清理。

本轮没有做：

- 未执行 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未读取真实完整 transcript。
- 未迁移数据库。
- 未改 workflow state JSON。
- 未改工作流状态机。
- 未写正式事实或正式记忆。
- 未启动 MCP canvas run。
- 未做真实 Tauri 窗口验收。

## 后端证据

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/codex_transcript.rs`

改动：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`

事实：

- `load_codex_session_transcript_for_index` 现在允许 index 读取失败后继续走 sqlite 会话目录；sqlite 有目标 thread 时，不再被静态 index 缺失、损坏或旧状态挡住。
- `load_codex_session_transcript_with_catalog` 先读 `codex_db::read_threads(db_path)`；sqlite 有目标 thread 时直接以 sqlite row 组装 `TranscriptThreadMetadata` 并调用 Rust parser。
- sqlite 不可用时才走 index fallback；sqlite 可用但 thread 缺失时，index 只作为兼容回退。
- 验收复核补测：`load_codex_session_transcript_with_optional_catalog(None, ...)` 可在 index 不可用时直接读取 sqlite thread。
- `codex_transcript::read_transcript_from_rollout` 使用 `BufRead` 逐行解析 JSONL，不通过 Python 子进程。
- Rust parser 覆盖：`event_msg` / `response_item` 用户和 assistant 消息、tool call、tool result、command output、session meta、turn context、compacted、reasoning/system context、bad JSON line warning、unknown event、encrypted content omit、sensitive-like warning。
- rollout 路径仍限制在 Codex home 的 `sessions` 或 `archived_sessions` 下；缺失文件返回 `rollout_missing:*`，越界返回 `rollout_outside_allowed_dirs:*`。
- `reveal_indexed_rollout` 的允许集合新增 sqlite 中合法 rollout 路径；复制路径仍保持旧 index 白名单。

未迁移项：

- `load_codex_session_transcript_with_reader` 仍存在，并仍包含 `Command::new("python3")`。
- 该旧 reader 当前用于 `dispatch_readback_stats` 的工作流派发读回，不是会话中心 Tauri transcript command 的主路径。

## 前端证据

新增：

- `prototypes/productized-desktop-shell/src/lib/conversationTurns.ts`

改动：

- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

事实：

- `conversationTurns` 从组件文件抽为纯函数，并过滤 system/environment 注入、thinking/reasoning、tool event、空消息和重复双流；验收复核补测了 `event_msg` 只有用户消息、`response_item` 含 Agent 回复的混合流，避免真实会话只显示半边对话。
- `AgentSessionCenter` 新增搜索框，匹配标题、thread id、项目路径及末段、rollout 路径及末段、模型、reasoning、软件名、状态和 warning。
- `AgentSessionCenter` 新增状态过滤：可读取 / 全部 / 缺 rollout / 已归档。
- 分组折叠仍由用户点击控制；选中会话在已收纳分组内时显示提示，不替用户强制展开。
- `SessionReader` 对后端错误 code 做前端分类：data missing、filesystem、parse、safety、system。
- `ChatTranscript` 保留早期消息默认收纳和长消息折叠；新增 fenced code block 容器和复制按钮。
- 清理确认未引用的旧会话中心 CSS：`.agent-session-item*`、`.agent-session-group*`、`.agent-session-items`、`.session-summary-row`、`.session-show-hidden`。

## 验证命令

在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell`：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，输出 `offline interaction tests passed: 9`。
- `npm run build`：通过；Vite 仍提示 chunk 大于 500 kB，这是既有构建提示，不阻塞本轮。

在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri`：

- `cargo test --lib`：通过，验收复核后为 `107 passed; 0 failed; 1 ignored`；保留既有 `JsonRpcError::invalid_params` dead_code warning。
- `rustfmt --check src/codex_transcript.rs src/codex_db.rs`：通过。

额外确认：

- `rg -n "agent-session-item|agent-session-group|agent-session-items|session-summary-row|session-show-hidden" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests`：无输出，旧选择器无残留。
- `rg -n "Command::new\\(\"python3\"\\)|load_codex_session_transcript_with_reader|load_codex_session_transcript_for_index|read_transcript_from_rollout" ...`：确认 Python reader 仍只在旧 `load_codex_session_transcript_with_reader` 和工作流派发 readback 路径中出现；会话中心 command 入口走 Rust parser。

## 风险

- 未读取真实 `/Users/yoyi/.codex`，因此没有用真实历史 rollout 做抽样泛化验证。
- Rust parser 对未知未来事件采取 `unknown` + warning + stripped metadata 降级，不保证已对所有真实历史事件做语义级归类。
- 真实 Tauri 窗口未启动，没有截图证据；本轮 UI 结论来自离线静态渲染测试和类型 / 构建验证。
- 工作流派发 readback 仍依赖 Python reader；如后续要彻底去 Python，需要单开任务，避免顺手改工作流机器。

## 自检事故记录

收尾文本自检时，有一次 `rg` 命令把包含反引号的搜索模式放进了 shell 双引号，触发 zsh command substitution，导致 shell 尝试执行 `/Users/yoyi/.codex` 并返回 `permission denied`。该命令没有读取或写入 `/Users/yoyi/.codex`，但违反了任务包里“搜索固定文本时必须用 `rg -F` 或单引号”的操作要求。

随后已使用单引号重跑同类搜索，未再触发该问题。后续搜索含反引号文本必须继续使用单引号或 `rg -F`。

## 验收复核补丁

复核时发现两个缺口并已修复：

- 后端：原实现仍先 `read_index(state)?`，如果静态 index 缺失或损坏，即使 sqlite 有会话也会失败；已改为 index 可选，sqlite 有目标 thread 时不受 index 读取失败影响。
- 前端：原 `conversationTurns` 只要存在任意 `event_msg` 就丢弃所有 `response_item`；若真实 rollout 是 `event_msg` 用户消息 + `response_item` Agent 回复，会只显示用户消息。已改为当 `event_msg` 缺 user 或 assistant 其中一侧时，从干净 `response_item` 中补齐缺失轮次。

复核后重新验证：

- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 9`。
- `cargo test --lib`：通过，`107 passed; 0 failed; 1 ignored`。
- `npm run typecheck`：通过。
- `npm run build`：通过，仍有既有 Vite chunk size warning。
- `rustfmt --check src/codex_transcript.rs src/codex_db.rs`：通过。

# Handoff：session-center-foundation-hardening-v1

日期：2026-06-03

## 结论

`tasks/2026-06-03-session-center-foundation-hardening-v1.md` 已完成。

接受为：

- 会话中心目录权威从 `index.json` 收敛到 sqlite 主权威。
- `index.json` 只保留缓存 / 兼容 / 辅助角色。
- 会话中心 transcript 主读取路径迁到 Rust 原生 JSONL parser。
- Python reader 不再参与会话中心 transcript command 主路径。
- 主对话默认只显示用户消息和 Agent 回复；过程事件默认收纳。
- 搜索、状态过滤、用户控制收纳、固定布局、错误分类、代码块复制和会话中心孤儿 CSS 清理已完成。

不接受为：

- 完整 Codex 控制器完成。
- 发消息 / stop / restart / resume 完成。
- 删除 / 导出 / 收藏 / 分享完成。
- 实时运行进度、多会话对比或会话 lineage 完成。
- Claude Code / OpenClaw / OpenCode 接入完成。
- 多智能体会话底座完成。
- 真实 Tauri 窗口验收完成。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/codex_transcript.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src/lib/conversationTurns.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `CURRENT.md`
- `tasks/README.md`
- `evidence/2026-06-03-session-center-foundation-hardening-v1.md`
- `handoffs/2026-06-03-session-center-foundation-hardening-v1-result.md`

## 权威关系

sqlite / index：

- sqlite 现在是会话中心 transcript 读取的主目录权威。
- sqlite 有目标 thread 时，直接读取 sqlite row 的 rollout path，不受静态 index 缺 thread 或旧 `rollout_exists=false` 影响。
- 验收复核后，静态 index 读取失败也不再阻止 sqlite 会话读取。
- sqlite 不可用或 sqlite 缺 thread 时，才允许 index compatibility fallback。
- `index.json` 不再是 sqlite 会话正文读取的准入名单。

Python reader：

- 会话中心 Tauri command `load_codex_session_transcript` 不再走 Python reader。
- `load_codex_session_transcript_with_reader` 仍保留给工作流派发 readback；这是本轮明确未迁移项。

未覆盖 rollout 事件：

- 未对真实历史 rollout 做抽样读取。
- Rust parser 对未识别事件统一保留为 `unknown`，写入 `unknown_event_type` warning，并保留脱敏后的 diagnostic metadata。

## 验证结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，输出 `offline interaction tests passed: 9`
- `npm run build`
- `cargo test --lib`，验收复核后为 `107 passed; 0 failed; 1 ignored`
- `rustfmt --check src/codex_transcript.rs src/codex_db.rs`

备注：

- `npm run build` 保留 Vite chunk size warning。
- `cargo test --lib` 保留既有 `JsonRpcError::invalid_params` dead_code warning。
- 未启动真实 Tauri 窗口，因此没有截图。

## 验收复核补丁

复核时发现并已修复两个缺口：

- `load_codex_session_transcript_for_index` 原来仍先读静态 index，index 坏了会挡住 sqlite 会话；现已允许 index 不可用时继续用 sqlite 主目录读取。
- `conversationTurns` 原来在存在任意 `event_msg` 时会丢弃全部 `response_item`，混合流可能只显示用户消息；现已在 `event_msg` 缺 user 或 assistant 一侧时，从干净 `response_item` 补齐缺失轮次。

## 边界记录

本轮没有：

- 执行真实 `codex exec` 或 `codex exec resume`。
- 读写 `/Users/yoyi/.codex`。
- 读取真实完整 transcript。
- 写 workflow state JSON。
- 改工作流状态机。
- 迁移数据库。
- 写正式事实或正式记忆。
- 启动 MCP canvas run。

收尾自检事故：

- 有一次 `rg` 文本搜索把含反引号的模式放在 shell 双引号里，zsh 尝试执行 `/Users/yoyi/.codex` 并返回 `permission denied`。
- 该事故没有读取或写入 `/Users/yoyi/.codex`，但违反了任务包的固定文本搜索要求。
- 已用单引号重跑安全搜索；后续继续用单引号或 `rg -F` 搜索含反引号文本。

## 手动测试清单

在应用里测试会话中心：

1. 打开桌面壳，进入“智能体 / Agent”页面。
2. 确认左侧会话列表有搜索框，以及“可读取 / 全部 / 缺 rollout / 已归档”状态过滤。
3. 在搜索框输入一个会话标题片段，确认列表缩小；清空后恢复。
4. 输入项目目录末段、thread id 片段或模型名，确认也能匹配。
5. 切换到“缺 rollout”，确认只显示缺失 rollout 的会话；切换到“已归档”，确认只显示归档会话；切回“可读取”。
6. 点击项目分组标题收纳 / 展开，确认系统不因选中会话自动替你展开分组；如果选中会话在收纳分组内，左侧应显示提示。
7. 点击一个可读取会话，确认右侧自动读取正文，主流只显示用户消息和 Codex 回复。
8. 找一条较长会话，确认较早消息默认收纳，有“展开全部”；长消息气泡有“展开 / 收起”。
9. 找包含 fenced code block 的回复，确认代码块有独立容器和“复制”按钮。
10. 勾选“显示过程事件”，确认工具调用、系统上下文、reasoning 等只在过程事件区展示。
11. 对缺 rollout 或安全拒绝场景，确认错误显示为可读分类提示，不展示 Python traceback。
12. 滚动左侧列表和右侧消息区，确认页面外层不被消息撑高，左右区域各自内滚。

真实窗口验收尚未完成；上述清单需要下一轮手动或 Tauri 截图验收切片补证据。

## 下一步建议

最近可单开：

- Agent adapter 后端能力声明：把前端只读能力声明收敛到后端 `agent_adapters[]` 读模型。
- 会话中心真实 Tauri 窗口验收：补截图和真实历史 rollout 抽样。
- 项目工作流画布产品化深化：继续做节点详情、局部编辑、运行反馈和安全确认。

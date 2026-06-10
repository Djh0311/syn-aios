# Handoff：会话中心可读性重做 v3 结果

更新时间：2026-06-02

## 结论

用户在 v2 后提的 6 条已处理：5 条做完（#1 收纳、#2 自动跳最新、#3 缩页不溢出、#5 弹窗 bug、#6 索引拒读），#4 发消息走用户决策后跳过。通过 typecheck / 离线测试（3）/ build / cargo build --lib / cargo test --lib（90 passed）。

不接受为真实 Tauri 截图验收，#6 在真实数据上的效果需真机抽查。

## 逐条

1. 项目分组可折叠收纳（含选中会话的组强制展开）。
2. 会话切换/加载后自动滚到最新消息（向上找可滚动祖先，兼容两种布局）。
3. agent 页固定视口高度，左列表 + 右正文各自内滚，外层不再整体滚动。
4. 发消息 = `codex exec resume` + 写 `~/.codex`，核心硬边界；AskUserQuestion 让用户选，用户选跳过，本轮不做。
5. 弹窗：补 z-index（被状态轨盖住）、点背板关闭、失败也关闭（catch 补 setPendingAction(null)）；「无真实功能」是误解，reveal 是真实 Tauri 命令，仅离线壳不生效。
6. 列表走实时 sqlite（368）、读 transcript 却校验静态 index（356）→ 新会话被拒读。改 `load_codex_session_transcript_for_index`：索引找不到就回退 sqlite 查 rollout_path、合成最小临时索引交同一个 reader 读、读完删；reader 的 `~/.codex/sessions` 路径校验保持不变。

## 改动文件

- 前端：`src/components/PermissionDialog.tsx`、`src/App.tsx`、`src/views/AgentView.tsx`、`src/styles.css`。
- 后端：`src-tauri/src/lib.rs`（新增 `load_codex_session_transcript_from_sqlite`，改 `load_codex_session_transcript_for_index`）。

## 验证

- `npm run typecheck` / `npm run test:offline-interaction`（3）/ `npm run build`：通过。
- `cargo build --lib` / `cargo test --lib`（90 passed）：通过。

## 不接受为

- 真实 Tauri 窗口截图验收（沙箱无法起 Tauri）。
- #6 在真实数据上已验证（逻辑+编译通过，但「之前被拒读的 12 条现在能读」未真机跑过）。
- 实现了发消息（#4 用户选跳过）。
- 多智能体会话底座完成。

## 残留

- 真机抽查全部 6 点，重点 #6 拒读会话现可读、#3 不溢出。
- #4 若未来要做 = 正式推翻硬边界，需单独授权与设计。
- 孤儿样式待清理切片。

依据见 `evidence/2026-06-02-session-center-legibility-v3.md`。

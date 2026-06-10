# Evidence：会话中心可读性重做 v3（收纳 / 自动跳转 / 缩页 / 弹窗 bug / 索引拒读 / 发消息边界）

更新时间：2026-06-02

## 任务

用户在 v2 后提出 6 条（用户原话）：

1. 有项目分类但没有收纳——会话按项目分类后仍是平铺，需要可折叠收纳。
2. 会话页面不要从第一条消息开始，自动跳到最新消息。
3. 整个 agent 页面仍太长，缩短到不用上下滑也能全显示。
4. 加入对话功能，可以直接在会话页面给 codex 会话实例发消息。
5. 右上角「定位 rollout」弹窗会和其他 UI 叠在一起、不能点选项、没有真实功能、且一直停留在界面上。
6. 很多会话显示「不在索引内」，被拒绝读取 transcript。

风险路径：#1#2#3#5 = Standard Path 前端；#6 = 后端 Rust transcript 读取回退（read-only sqlite + 复用既有 reader，保留路径安全校验）；#4 = 触碰核心硬边界，已走用户决策。

## #4 边界决策

给 codex 会话真发消息 = `codex exec resume` + 写 `~/.codex`，是所有治理文档明文禁止的核心硬边界。用 AskUserQuestion 让用户三选一，用户选「暂不做发消息」。本轮不实现发送/输入框，按用户选择跳过。

## 根因（#5 / #6 是真 bug）

- #5：`.dialog-backdrop` 用了 `position:fixed; inset:0` 但**没有 z-index**，被层级更高的状态轨（z-index 50）盖住，所以「叠在一起、点不到按钮」；同时 `confirmAction` 在 catch 分支只 setNotice/ setError，**没有 setPendingAction(null)**，失败时弹窗永远不关 →「一直停留」。「没有真实功能」是误解：`reveal_indexed_rollout` 是真实 Tauri 命令（在 Finder 里定位文件），只是浏览器/离线壳里不生效。
- #6：会话**列表**来自实时 `~/.codex/state_5.sqlite`（368 条），transcript **读取**却用冻结的静态 `index-kernel/codex-index.json`（356 条，5/31 构建）校验 thread 是否在索引内，不在就拒读。新增的 12 条会话能列出但读不了。

## 改动

1. #1 收纳：`AgentSessionCenter` 加 `collapsedKeys` 状态，分组头从 div 改 button（加 ▸/▾ caret），点击折叠/展开；含当前选中会话的分组强制保持展开，避免把正在读的会话折没。
2. #2 自动跳转：`ChatTranscript` 加 `streamRef` + effect，会话切换或对话条数变化时滚到底；滚动目标向上找最近的可滚动祖先，兼容「stream 自己滚」和「外层面板滚」两种布局。
3. #3 缩页：`.agent-session-center.embedded` 固定高度 `calc(100vh - topbar - dock - 72px)`，左列表和右正文各自内部滚动，外层 stage 不再因 agent 页变长而整体滚动；embedded 下 chat-stream 取消自身 max-height，避免双滚动条。
4. #5 弹窗：`.dialog-backdrop` 加 `z-index:1000`、`.dialog` `z-index:1001` + `max-height` + 自身滚动；点暗色背板（非面板）可关闭（busy 时不关）；`confirmAction` catch 分支补 `setPendingAction(null)`，失败也关闭，错误改由 notice 面板独立显示。
5. #6 索引拒读：`load_codex_session_transcript_for_index` 在静态索引找不到该 thread 时，回退到 `load_codex_session_transcript_from_sqlite`——从 sqlite 查出该 thread 的 rollout_path，合成一份最小单线程索引写临时文件，交给**同一个** `transcript_reader.py` 读，读完删临时文件。python reader 仍校验 rollout 必须在 `~/.codex/sessions` 下，安全边界不变。

## 验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过（3 scenarios）。
- `npm run build`：通过。
- `cargo build --lib`：通过（仅既有 dead-code 警告）。
- `cargo test --lib`：90 passed。

## 不接受为

- 不接受为真实 Tauri 窗口截图级验收。沙箱起不了 Tauri：#1 折叠手感、#2 滚到底、#3 是否真的不溢出、#5 弹窗在真实层级下的表现，都要真机 `npm run tauri:dev` 确认。
- 不接受为 #6 在真实数据上已验证。回退逻辑 Rust 编译通过、复用既有 reader，但「sqlite 里有、静态索引里没有的那 12 条会话现在能读」这件事我没在真实环境跑过——需要真机点开一条「之前被拒读」的会话确认。
- 不接受为实现了发消息（#4 用户选择跳过）。
- 不接受为多智能体会话底座完成。

## 边界遵守

- 未执行真实 `codex exec` / resume。
- 未写 `/Users/yoyi/.codex`（sqlite 只读打开 `mode=ro`；临时索引写在系统 temp 目录并即时删除）。
- 未改 workflow state JSON / 状态机。
- 未改首页内容。
- transcript 读取的路径安全校验（rollout 必须在 `~/.codex/sessions` 下）保持不变。

## 残留 / 下一步

- 真机 `npm run tauri:dev` 抽查全部 6 点，重点是 #6 的「之前被拒读的会话现在能读」和 #3 的「不溢出」。
- #4 发消息：用户当前选跳过；若未来要做，等于正式推翻核心硬边界，需要单独的授权与设计（每次发送加确认弹窗、复用 workflow 的 resume 路径）。
- 孤儿样式（`.agent-session-item*` 等）仍未清，留待清理切片。

# Handoff：会话中心可读性重做 v2（五点打磨）结果

更新时间：2026-06-02

## 结论

用户在 v1 后提的 5 条会话页打磨已全部完成代码切片，通过 typecheck / 离线交互测试（3 scenarios）/ build。最关键的是 #3：定位到 codex rollout 的 `event_msg` / `response_item` 双流问题，前端按 `raw_type` 只保留 `event_msg` 干净轮次，去掉系统提示词注入和重复。

不接受为真实 Tauri 截图验收，也不接受为多智能体会话底座完成。

## 对应 5 点

1. 去掉「已读取索引…需点击确认」常驻提示和同类说明文本（notice 仅在有内容/错误时显示；footer 改「秘书」；reader 去冗余说明和 detail grid）。
2. 全局会话保持按项目分组；标题用 sqlite 真实 codex 标题（数据层本就如此）。
3. 选中即自动加载（不再先点「读取正文」）；新增 `conversationTurns` 只显示 event_msg 流的人/Agent 消息，过程事件默认折叠可展开。
4. 列表列宽 360px → 248px。
5. 正文区从界面顶部开始（页头移进左列、工具条移到对话流底部、sticky/max-height 改视口）。

## 改动文件

- `src/App.tsx`、`src/views/AgentView.tsx`、`src/styles.css`、`tests/offline-permission-dialog.test.tsx`。
- 未改后端 Rust 和 `transcript_reader.py`（reader 已透传 `raw_type`，清洗在前端做）。

## 验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过（3）；新增双流清洗 scenario。
- `npm run build`：通过。

## 不接受为

- 真实 Tauri 窗口截图验收（沙箱无法起 Tauri）。
- 读取过 `~/.codex` 真实正文（离线 fixture 验证）。
- 多智能体会话底座完成。
- schema / 状态机 / workflow state 变更。

## 残留

- 真机 `npm run tauri:dev` 抽查：自动加载、干净对话、收窄列表、顶起正文，以及 `conversationTurns` 在真实历史 rollout 上的表现。
- 孤儿样式（`.agent-session-item*` 等）待清理切片。

依据见 `evidence/2026-06-02-session-center-legibility-v2.md`。

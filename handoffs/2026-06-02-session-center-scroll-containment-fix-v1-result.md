# Handoff: session center scroll containment fix v1

日期：2026-06-02

## 结果

已修会话中心滚动和消息收纳的第一版实现：

- 页面根增加 `agent-view-root`，会话中心高度链改为固定在页面内。
- 会话列表在左侧框内滚动。
- 消息面板在右侧框内滚动。
- 默认只显示最近 12 条对话，较早消息收纳到提示条。
- 切换会话后恢复默认收纳。
- 单条长消息默认折叠，可手动展开。

## 改动文件

- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `evidence/2026-06-02-session-center-scroll-containment-fix-v1.md`
- `handoffs/2026-06-02-session-center-scroll-containment-fix-v1-result.md`

## 验证

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`

未做：

- 未做真实浏览器截图。
- 未启动 Tauri。

## 手动验收建议

在当前前端页面刷新后检查：

1. 整个工作台页面不应跟随会话数量或消息数量滚动。
2. 会话列表长内容只能在左侧框里滚动。
3. 消息长内容只能在右侧消息框里滚动。
4. 默认只显示最近 12 条消息；顶部出现“已收纳较早 N 条消息”。
5. 长消息默认折叠，点击“展开”后只展开当前消息。

## 边界

本轮未改数据模型、未改 transcript 读取、未改 workflow state、未执行真实 Codex、未启动 Tauri。

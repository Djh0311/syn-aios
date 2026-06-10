# Handoff：会话中心可读性重做 v1 结果

更新时间：2026-06-02

## 结论

会话中心可读性重做已完成代码切片并通过 typecheck / 离线交互测试 / build。本轮把「混乱、看不清」的根因从表现层修掉：去掉强制选软件层的占位步骤，会话列表从窄栏 4 列表格换成可读会话卡，补相对时间和有意义的身份信息，提高对比。

不接受为真实 Tauri 窗口截图级验收，也不接受为多智能体会话底座完成。

## 用户指令依据

用户原话要点：会话看不清、Codex 会话管理和会话信息混乱、不像 codex 原生体验；随后明确「可以折叠冗余层 + 重做会话卡，但不是非要 codex 原生，要一个良好优秀的对话体验，codex 只是参考，可以自由发挥」。

## 改动文件

- `prototypes/productized-desktop-shell/src/lib/format.ts`：新增 `relativeTime`、`pathTail`。
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`：去强制软件层步骤，加筛选 chip 插槽，会话卡重做，reader 头部副标题。
- `prototypes/productized-desktop-shell/src/styles.css`：新增筛选 chip、会话卡、reader 副标题样式，提高对比。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`：更新 AgentView 断言到新 IA。

## 验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过（2）。
- `npm run build`：通过。

## 不接受为

- 真实 Tauri 窗口截图验收（沙箱无法启动 Tauri，未截图）。
- 读取过 `~/.codex` 真实正文（全程离线 fixture）。
- 多智能体会话底座完成（Claude Code / OpenClaw 仍只是筛选位）。
- schema / 状态机 / workflow state 变更（未动事实结构）。

## 残留

- 真实 Tauri 截图验收待补。
- `softwareGroupsForSessions` 既有死导出未删，留待后续清理切片。

依据见 `evidence/2026-06-02-session-center-legibility-v1.md`。

# 桌面壳 Codex 会话中心只读 UI v1 结果

## 薄弱点先说

- 没有真实 Tauri 窗口验证。
- 没有做发送消息、resume、多轮聊天。
- 没有做项目页内 Agent 会话入口。
- 后端读取 transcript 依赖现有 Python reader。

## 做了什么

- Agent 页改成 Codex 会话中心。
- 显示 Codex 会话列表。
- 点选单个会话后可读取 transcript。
- 展示 transcript 时间线、工具调用、工具结果、命令输出和 warning。
- 保留未接入 agent 空白位。
- 后端新增只读命令 `load_codex_session_transcript`。

## 改了哪些文件

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/styles.css`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

新增：

- `/Users/yoyi/workspace/product-line/evidence/2026-05-29-desktop-shell-codex-session-center-readonly-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-29-desktop-shell-codex-session-center-readonly-v1-result.md`

## 验证结果

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 3`。
- `npm run build`：通过。
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`：通过，48 passed，1 ignored。

## 边界

- 没有启动真实 Tauri 窗口。
- 没有读取新的真实会话正文做 smoke。
- 没有执行 Codex CLI。
- 没有写 `/Users/yoyi/.codex`。
- 没有读授权、密钥、`.env`。
- 没有运行 harness。

## 下一步建议

建议下一步二选一：

1. 验证线做真实 Tauri 窗口 smoke：打开 Agent 页，读取一个真实会话 transcript，只记录统计和 UI 是否出现时间线，不贴全文。
2. 桌面应用线继续做项目内 Agent 会话入口 v1：项目页复用同一套会话组件，按项目过滤和打开会话。

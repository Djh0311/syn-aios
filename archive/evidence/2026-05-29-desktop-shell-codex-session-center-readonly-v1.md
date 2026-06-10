# 桌面壳 Codex 会话中心只读 UI v1 证据

## 薄弱点

- 本轮没有真实 Tauri 窗口 smoke。依据：只跑了类型检查、离线前端测试、构建和 Rust 单测。
- 本轮没有做发送消息、resume、多轮聊天、新建会话 UI。依据：前端只新增只读会话中心和“读取正文”按钮，后端只新增读取 transcript 命令。
- transcript 后端命令通过 `python3 transcript_reader.py` 读取单个索引内会话，依赖 Python 和现有 reader 文件。依据：Rust 命令会检查 reader 路径并调用该脚本。
- Agent 页读取正文是用户点选触发，不是全量预加载。依据：`AgentView` 只有点击会话或“读取正文”时调用 `onLoadTranscript`。
- 项目页内 Agent 会话入口还没做。依据：本轮只改 Agent 页和后端 transcript 命令。

## 做了什么

- 把 Agent 页从旧的“只展示 Codex 可用卡片”改成 Codex 会话中心。
- Agent 页现在显示 Codex 会话列表。
- 用户选择单个会话后，可以只读加载 transcript。
- transcript 时间线展示用户消息、Codex 消息、工具调用、工具结果、命令输出、系统上下文和 warning。
- 工具和原始字段用折叠区展示。
- 后端新增 `load_codex_session_transcript`，按 `thread_id` 读取单个索引内会话。
- 后端先校验 `thread_id` 存在于当前静态索引且 rollout 存在，再调用 transcript reader。
- 没有把 transcript 写入默认 `codex-index.json`，也没有复制完整 transcript 到工作台状态。

## 改动文件

- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/styles.css`
- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

新增证据和交接：

- `product-line/evidence/2026-05-29-desktop-shell-codex-session-center-readonly-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-codex-session-center-readonly-v1-result.md`

## 后端实现

新增命令：

```text
load_codex_session_transcript(thread_id)
```

行为：

- 读取当前静态 `codex-index.json`。
- 查找索引内 `thread_id`。
- 如果会话不在索引内，拒绝。
- 如果 rollout 不存在，拒绝。
- 调用 `product-line/prototypes/index-kernel/transcript_reader.py`。
- transcript 临时输出写到系统临时目录。
- Rust 读回并解析为结构化对象后，删除临时输出文件。

新增 Rust 测试：

- 非索引线程拒绝。
- transcript reader 输出能映射成前端结构。

## 前端实现

Agent 页现在包含：

- Codex 能力卡。
- 未接入 Agent 空白位。
- Codex 会话列表。
- 当前会话详情。
- “读取正文”按钮。
- “定位 rollout”按钮。
- transcript 概览：事件数、未知事件、加密省略、疑似敏感。
- transcript 时间线。
- warning 展示。

没有新增：

- 发送框。
- 新建会话按钮。
- resume 按钮。
- 删除 / 移动 / 归档会话按钮。
- provider 配置。

## 验证

已运行：

```bash
npm run typecheck
```

结果：通过。

已运行：

```bash
npm run test:offline-interaction
```

结果：`offline interaction tests passed: 3`。

已运行：

```bash
npm run build
```

结果：通过，生成 `dist`。

已运行：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline
```

结果：48 个通过，1 个真实任务文件确认测试 ignored。

说明：

- 直接 `cargo test --offline` 曾因默认 cargo 缓存缺 `serde_json 1.0.150` 失败。
- 复用项目既有共享 cargo 缓存后通过。

## 安全边界

- 没有执行 Codex CLI。
- 没有创建新会话。
- 没有发送 prompt。
- 没有运行 resume 或 fork。
- 没有写 `/Users/yoyi/.codex`。
- 没有改 Codex 状态库。
- 没有读取 `auth.json`、`.env`、授权文件或密钥文件。
- 没有把完整 transcript 写入 evidence / handoff。
- 没有运行 harness。

## 结论

本轮可以接受为“桌面壳 Agent 页 Codex 会话中心只读 UI v1”。

不能接受为“工作台内 Codex 聊天已完成”，也不能接受为“项目页 Agent 会话入口已完成”。

下一步建议：

- 做真实 Tauri 窗口 smoke，确认 Agent 页能读取并展示一个真实会话 transcript。
- 或继续做“项目内 Agent 会话入口 v1”，把同一套会话组件按项目过滤后接入项目页。

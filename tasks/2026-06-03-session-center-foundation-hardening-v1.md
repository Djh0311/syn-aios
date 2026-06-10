# Task Package：会话中心底座硬化 v1

状态：已完成。  
用途：把 Opus 复盘指出的会话中心底座问题一次性收敛到可继续开发的形态。  
执行方式：一个批次内完成，不拆成十几个微任务；最终统一验收。  

完成记录：见 `../evidence/2026-06-03-session-center-foundation-hardening-v1.md` 与 `../handoffs/2026-06-03-session-center-foundation-hardening-v1-result.md`。

## 1. 先说薄弱点

当前会话中心不是“底座完成”，只是做过三轮可读性修补。

主要风险：

- 会话列表已经倾向实时 sqlite，但 transcript 读取仍会绕回静态 `index.json` 和 Python reader，数据源双轨没有根治。
- `index.json` 仍在部分路径里扮演 transcript 准入名单，这会继续制造“列表看得到、正文读不了”的问题。
- 后端通过 Python 脚本读 rollout，错误不可控，进程开销也不适合作为长期底座。
- 会话中心的产品边界需要写清：本轮是只读历史会话浏览器，不是完整 Codex 控制器。
- 368 条会话没有足够搜索、过滤和稳定收纳。
- 对话清洗和展示仍需要后端 / 前端共同兜底，不能只靠一个 `raw_type === event_msg` 假设。
- 错误处理仍是粗字符串，用户分不清数据缺失、文件系统错误、解析错误和安全拒绝。
- 真机验收、样式清理、可访问性仍是残留风险。

一句话目标：

```text
会话中心以 sqlite 为会话目录权威，以 Rust 原生 JSONL parser 读取 rollout，
index.json 只做缓存 / 兼容 / 辅助信息，不再当 transcript 准入名单。
UI 做成固定框架内的可搜索、可收纳、可阅读历史会话浏览器。
```

## 2. 必须先读

当前入口：

- `CURRENT.md`
- `AUTHORITY.md`
- `tasks/README.md`
- `docs/workbench-system-architecture-v1.md`

会话中心前置记录：

- `evidence/2026-06-02-session-center-legibility-v1.md`
- `handoffs/2026-06-02-session-center-legibility-v1-result.md`
- `evidence/2026-06-02-session-center-legibility-v2.md`
- `handoffs/2026-06-02-session-center-legibility-v2-result.md`
- `evidence/2026-06-02-session-center-legibility-v3.md`
- `handoffs/2026-06-02-session-center-legibility-v3-result.md`

主要代码入口：

- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/src-tauri/src/codex_db.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/index-kernel/transcript_reader.py`
- `prototypes/index-kernel/tests/test_transcript_reader.py`

## 3. 已知事实 / 未知 / 假设

已知事实：

- 当前 `codex_db.rs` 可只读读取 Codex sqlite，并从 `session_index.jsonl` 补标题。
- 当前 `load_sessions` 已有 `SessionSourceMode::RealWithSqliteFallback`。
- 当前 `load_codex_session_transcript_for_index` 仍先读静态索引；索引找不到时才回退 sqlite 并合成临时 index 交给 Python reader。
- 当前 Python reader 已处理 encrypted content 脱敏、敏感内容 warning、bad jsonl warning、unknown event、工具调用和命令输出。
- 当前前端 `conversationTurns` 优先展示 `metadata.raw_type === "event_msg"` 的 user/assistant 消息。
- 用户已经明确跳过会话页发消息功能，因为这会触发 `codex exec resume` 和写 `/Users/yoyi/.codex`。

未知：

- 真实历史 rollout 里是否还有 `event_msg` / `response_item` 之外的新事件形态。
- 当前真实 Tauri 窗口里固定高度、内滚、收纳和消息折叠是否完全符合用户观感。
- `index.json` 目前是否还有其他调用点被当成 rollout 安全白名单。

本任务包的假设：

- 本轮可以改会话中心相关前后端代码和测试。
- 本轮测试使用临时目录 / fixture，不读取真实 `/Users/yoyi/.codex` 正文。
- 如执行者要做真实窗口验收，必须先确认当前对话允许启动 Tauri 并读取本机 Codex 元数据；否则只能输出未完成真机验收。

## 4. 产品边界

会话中心 v1 是：

- 历史会话浏览器。
- 项目分组 / 软件分组 / 搜索过滤入口。
- 只读 transcript 阅读器。
- Agent adapter 能力声明的只读展示面。
- 允许用户确认后的本机辅助动作，例如定位 rollout 文件。

会话中心 v1 不是：

- Codex 实例控制器。
- 可直接发消息的 chat client。
- 会话停止 / 重启 / resume 控制台。
- 会话删除 / 导出 / 收藏 / 分享系统。
- 实时运行进度监控器。
- 多会话对比器或会话 lineage 图。

## 5. 全局禁止

- 不实现发送消息。
- 不执行 `codex exec`。
- 不执行 `codex exec resume`。
- 不停止、重启或重开真实 Codex 会话。
- 不删除、移动、归档或改写真实 Codex 会话。
- 不写 `/Users/yoyi/.codex`。
- 不读取真实完整 transcript 作为开发证据，除非用户在执行线程另行明确批准。
- 不读取 auth、token、`.env` 或完整敏感日志。
- 不改 workflow state JSON。
- 不改工作流状态机。
- 不写正式事实。
- 不写正式记忆。
- 不接 Claude / OpenClaw / OpenCode。
- 不迁移数据库。
- 不启动 MCP canvas run。
- 不运行 harness。
- 不把本轮说成多智能体会话底座完成。
- 不把本轮说成真实 Codex 控制器完成。
- 不把 `index.json` 继续设计成 transcript 准入名单。

搜索固定文本时必须用 `rg -F '...'` 或单引号，避免 shell 反引号命令替换。

## 6. 执行段 A：会话目录权威收敛

目标：

- sqlite 成为 Codex 会话目录权威。
- `index.json` 只保留缓存、兼容和辅助元数据角色。
- sqlite 中存在且有合法 rollout 路径的会话，应该可以进入 transcript 读取流程。

建议改动：

1. 在后端增加清晰的会话目录读取 helper，例如 `codex_session_catalog` 或扩展 `codex_db.rs`。
2. `load_codex_session_transcript_for_index` 改为先按 thread id 查询 sqlite 权威目录。
3. 只有 sqlite 不可用时，才允许回落到静态 index。
4. 如果 sqlite 有该 thread 且 rollout 存在，不能因为静态 index 没有该 thread 或静态 index 标记旧状态而拒读。
5. `allowed_rollout_path_count` 和 `reveal_indexed_rollout` 的安全模型要同步修正：允许来源应包括 sqlite 权威目录里合法的 rollout 路径，而不是只看冻结 index。
6. 继续保留路径安全校验：rollout 必须在 Codex home 的 `sessions` 或 `archived_sessions` 下，路径必须规范化后再判断。

验收：

- sqlite 有 thread、index 没有 thread：可以读取 transcript。
- sqlite 有 thread、index 有旧 thread 但 rollout 标记缺失：以 sqlite 权威为准。
- sqlite 没有 thread、index 有 thread：允许兼容回退，但 evidence 必须说明这是 fallback。
- rollout 路径不在允许目录：拒绝读取，错误类别是 safety。
- rollout 文件不存在：拒绝读取，错误类别是 data_missing 或 filesystem，不是“不在索引内”。

## 7. 执行段 B：Rust 原生 transcript JSONL parser

目标：

- 移除会话中心读取 transcript 对 Python 子进程的依赖。
- Rust 直接解析 rollout `.jsonl`，输出现有 `CodexTranscript` / `CodexTranscriptEvent` 契约。
- Python reader 可暂时保留给 index-kernel 历史工具，但桌面壳会话中心不再调用它。

必须对照：

- `prototypes/index-kernel/transcript_reader.py`
- `prototypes/index-kernel/tests/test_transcript_reader.py`
- `src-tauri/src/lib.rs` 中 `parse_codex_transcript` 期望的字段。

实现建议：

1. 新增后端模块，例如 `src-tauri/src/codex_transcript.rs`。
2. 使用 `BufRead` 逐行解析 JSONL，不一次性把大文件全读进内存。
3. 复制 Python reader 的核心语义：
   - bad JSON line 不让整个 transcript 失败，写入 warning。
   - `encrypted_content` 不输出原文，只输出 omitted 标记和 warning。
   - sensitive-like 内容只标 warning，不直接泄露更多上下文。
   - unknown event 保留诊断 metadata。
   - `event_msg` / `response_item` 都保留 `metadata.raw_type`。
   - 解析 user message、assistant message、tool call、command output、reasoning / thinking、session meta、turn context、compacted。
4. 不把 Python traceback 或内部 panic 传给前端。
5. 如果第一轮无法完全覆盖 Python reader 的所有历史分支，必须列出未覆盖事件类型，并用 `unknown` + warning 安全降级。

验收：

- 原 Python reader 测试中的基础 fixture 在 Rust parser 中有等价覆盖。
- bad jsonl line、unknown event、encrypted content、sensitive-like content 都有 Rust 测试。
- `metadata.raw_type`、`payload_type`、`payload_keys` 仍可供前端清洗使用。
- 桌面壳 transcript 读取路径不再调用 `Command::new("python3")`。
- 工作流派发 readback 如果仍依赖 Python reader，必须在 evidence 里列为未迁移；本轮至少要求会话中心读取路径迁移。

## 8. 执行段 C：对话清洗和消息展示硬化

目标：

- 默认只显示“用户发的消息”和“Agent 回复”。
- thinking、system reminder、execution log、tool use、工具输出默认收纳到过程事件，不混入主对话流。
- 消息框架固定，消息在框架内滚动，默认展示最近消息，早期消息收纳。

建议改动：

1. 把 `conversationTurns` 变成可测试的纯函数模块，例如 `src/lib/conversationTurns.ts`。
2. 清洗规则不要只依赖 `raw_type === "event_msg"`：
   - 优先使用 `event_msg` 的 user/assistant。
   - 过滤 system reminder、turn_context、session_meta、reasoning/thinking、tool_call、command_output。
   - 没有 `event_msg` 时，才回退到可确认的 `response_item` user/assistant。
   - 明显是系统注入的环境上下文不能当用户消息展示。
3. 对长消息保留折叠。
4. 对早期消息保留“已收纳 N 条”的明确入口，不自动展开全部。
5. 代码块至少支持 fenced code block 分段展示和复制按钮；没有现成高亮库时，不新增重依赖。
6. 过程事件保留单独开关或折叠区，不能默认铺满主对话。

验收：

- 双流 rollout 不重复展示。
- 系统提示词 / 环境上下文不进入主对话。
- thinking / reasoning / tool call / command output 默认不进入主对话。
- 只有 response_item 的旧会话仍能展示人和 Agent 的真实轮次。
- 长消息有收起 / 展开。
- 早期消息默认收纳，且不会加载后直接把全部消息铺出来。
- 代码块有稳定容器和复制按钮。

## 9. 执行段 D：搜索、过滤、收纳和固定布局

目标：

- 368 条会话可扫描、可搜索、可过滤。
- 收纳语义尊重用户操作，不做“智能强制展开”。
- 整体页面固定；会话列表和消息区各自滚动。

建议改动：

1. 增加搜索输入，至少匹配：
   - 会话标题。
   - thread id。
   - 项目路径末段和完整路径。
   - 模型。
   - 状态 warning。
2. 增加轻量过滤：
   - 全部 / 可读取 / 缺 rollout / 已归档。
   - 软件来源：Codex / Claude Code / OpenClaw，仅展示已有或声明可用的来源。
   - 项目分组仍保留。
3. 收纳规则：
   - 分组折叠 / 展开由用户点击决定。
   - 选中会话不能强制展开分组。
   - 如果选中会话所在组被折叠，可以在列表头提示“当前选中会话在已收纳分组内”，但不能替用户展开。
4. 固定布局：
   - agent 页面外层不产生整体纵向滚动。
   - 左侧会话列表内部滚动。
   - 右侧消息框内部滚动。
   - 列表和消息容器有稳定高度，不被消息内容撑开。
5. 键盘可用：
   - 搜索框可聚焦。
   - 会话卡可 Tab 到达，可 Enter 打开。
   - Escape 可关闭弹窗 / 清除搜索或只作用于当前上下文。
   - focus-visible 样式可见。

验收：

- 搜索标题能缩小列表。
- 搜索项目名能缩小列表。
- 过滤“缺 rollout”只显示缺失项。
- 折叠某分组后，选中该分组内会话不会自动展开。
- 页面外层不滚动，列表和消息区各自滚动。
- Tab / Enter / Escape 的最小交互可用。

## 10. 执行段 E：错误分类和 UI 呈现

目标：

- 不再把所有错误都变成一段不可理解的字符串。
- 前端能区分数据问题、文件系统问题、解析问题、安全拒绝和系统问题。

建议实现：

1. 后端新增稳定错误码，至少覆盖：
   - `session_not_found`
   - `rollout_missing`
   - `rollout_outside_allowed_dirs`
   - `sqlite_unavailable`
   - `jsonl_parse_warning`
   - `jsonl_parse_failed`
   - `transcript_reader_unavailable`（只用于历史 fallback，不应出现在会话中心主路径）
   - `filesystem_read_failed`
   - `unexpected_internal_error`
2. 如果 Tauri command 暂时仍只能返回 string，错误字符串必须带稳定 code 前缀，前端再 normalize。
3. 前端增加错误展示映射：
   - 数据缺失：说明该会话没有可读 rollout。
   - 安全拒绝：说明路径不在允许目录。
   - 文件系统：说明文件读取失败。
   - 解析失败：说明 rollout 格式无法解析，并保留重试入口。
   - 系统错误：显示短提示，不展示 traceback。
4. 错误 UI 只出现在会话阅读区域或 notice，不要全局大面积遮挡。

验收：

- sqlite 缺失不和 rollout 缺失混淆。
- 路径安全拒绝单独显示。
- JSONL 单行坏数据不导致全量失败。
- 前端不显示 Python traceback。
- 错误分类有测试。

## 11. 执行段 F：孤儿样式和旧 UI 清理

目标：

- 删除本轮会话中心重构后确认无用的旧 CSS。
- 只清理会话中心相关孤儿样式，不做全站美化。

候选清理范围：

- `.agent-session-item*`
- `.session-summary-row*`
- `.agent-session-group*`
- 其他 v1/v2 重构后不再被 JSX 使用的 session 旧选择器。

要求：

- 删除前用 `rg -F` 确认没有 JSX / TSX / 测试引用。
- 不清理不确定的全局样式。
- 不顺手重做 UI 主题。

验收：

- 删除的 CSS 选择器均有未引用证据。
- 会话中心样式仍通过离线渲染测试。

## 12. 测试要求

必须补测试，至少覆盖：

后端 Rust：

1. sqlite 有、index 没有的 thread 可读。
2. sqlite 权威覆盖 index 旧状态。
3. rollout outside allowed dirs 被拒绝。
4. rollout missing 被分类。
5. Rust JSONL parser 解析 user / assistant / tool call / command output。
6. Rust JSONL parser 对 bad line 记录 warning 并保留其他事件。
7. Rust JSONL parser 不泄露 encrypted content。
8. Rust JSONL parser 标记 sensitive-like content。
9. 会话中心 transcript 主路径不调用 Python。

前端 / 离线测试：

1. 搜索标题。
2. 搜索项目。
3. 按可读 / 缺 rollout / 已归档过滤。
4. 折叠分组后不会因选中会话强制展开。
5. `conversationTurns` 去重双流。
6. `conversationTurns` 过滤 thinking / system / tool events。
7. 早期消息默认收纳。
8. 长消息可展开 / 收起。
9. 错误分类展示。
10. 会话列表和消息区固定框架内滚动的结构类名存在。
11. 基础键盘可访问性和 focus-visible 样式存在。

## 13. 验证命令

在：

```text
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell
```

必须跑：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib
```

如果新增 Rust 模块，额外跑：

```text
rustfmt --check src/codex_transcript.rs src/codex_db.rs
```

如果修改了其他 Rust 新模块，也要把对应文件加入 `rustfmt --check`。

不要默认跑全仓库 `cargo fmt --check` 修历史格式债；如果它仍因既有 `src/lib.rs` 或 `src/mcp/**` 格式差异失败，记录在 evidence / handoff，不要批量格式化无关文件。

## 14. 真实窗口验收

目标是真实验证，不是假装截图完成。

如果执行线程有可用 Browser / Tauri 截图工具，并且用户允许读取本机 Codex 元数据：

- 启动真实应用。
- 打开智能体 / 会话中心。
- 截图或记录：
  - 搜索前后。
  - 分组折叠后仍不被强制展开。
  - 左列表内部滚动。
  - 右消息区内部滚动。
  - 早期消息收纳。
  - 长消息折叠。
  - 缺 rollout / 安全拒绝 / 解析 warning 的错误展示。
- 把截图路径写入 evidence。

如果无法做真实窗口验收：

- 不能声称真实窗口验收完成。
- 必须在 handoff 里列出用户手动验收步骤。
- 必须明确哪些结论只来自离线测试。

## 15. 验收标准

接受为：

- 会话目录权威从 `index.json` 收敛到 sqlite。
- `index.json` 不再是 transcript 准入名单，只做缓存 / 兼容 / 辅助。
- 会话中心 transcript 读取主路径迁到 Rust 原生 JSONL parser。
- Python reader 不再参与会话中心主读取路径。
- 主对话默认只显示用户消息和 Agent 回复。
- 过程事件默认收纳。
- 搜索、过滤、用户控制收纳可用。
- 页面外层固定，列表和消息各自滚动。
- 错误分类可读。
- 会话中心相关孤儿 CSS 已清理。
- 前后端测试覆盖关键路径。

不接受为：

- 完整 Codex 控制器完成。
- 发消息 / stop / restart / resume 完成。
- 会话删除 / 导出 / 收藏 / 分享完成。
- 实时运行进度完成。
- 多会话对比完成。
- 会话 lineage 图完成。
- Claude / OpenClaw / OpenCode 会话接入完成。
- 多智能体会话底座完成。
- 真实窗口验收完成，除非 evidence 有截图或明确记录。

## 16. 必须输出

执行完成后必须新增：

- `evidence/2026-06-03-session-center-foundation-hardening-v1.md`
- `handoffs/2026-06-03-session-center-foundation-hardening-v1-result.md`

并更新：

- `CURRENT.md`
- `tasks/README.md`

handoff 必须包含：

- 实际改动文件。
- 跑过的验证命令和结果。
- sqlite / index 权威关系是否已根治。
- Python reader 是否仍在任何会话中心主路径中。
- 未覆盖的 rollout 事件类型。
- 真实窗口验收是否完成。
- 是否读取过真实 `/Users/yoyi/.codex`，如果有，必须写明用户授权和读取范围。

## 17. 下一步建议

本任务完成后，最近两项才适合继续：

1. Agent adapter 后端能力声明：把当前前端只读 `adapterCapabilities.ts` 收敛到后端 `agent_adapters[]` 读模型，为 Claude / OpenClaw / OpenCode 接入做准备。
2. 项目画布交互专项：在会话中心底座稳定后，再按画布参考研究推进节点详情、局部编辑、运行反馈和安全确认，不要把会话中心数据债带进画布。

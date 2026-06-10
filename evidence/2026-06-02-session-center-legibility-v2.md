# Evidence：会话中心可读性重做 v2（五点打磨）

更新时间：2026-06-02

## 任务

用户在 v1 基础上提出 5 条具体打磨要求（用户原话）：

1. 智能体页最上方「已读取索引。本机所有动作仍需用户点击并确认」这类说明文本去掉，并检查所有页面里这类对实际体验没帮助的说明性文本。
2. 会话列表现在全平铺，需要按项目分类显示；对话标题要显示 codex 的实际会话标题。
3. 不要点「读取正文」后才显示对话；并且对话内容很乱，很多过程文本、系统提示词、上下文都被带出来了，只需要显示「我发的消息」和「Agent 回复的消息」。
4. 会话列表太宽，收窄。
5. 正式会话界面可以不和列表对齐，直接从整体界面顶部开始。

风险路径：Standard Path（前端 UI + 一个纯函数行为变更）。

## 读 / 写范围

读：会话相关前后端代码（`AgentView.tsx`、`App.tsx`、`codex_db.rs`、`lib.rs` 会话/transcript 部分、`transcript_reader.py`）、`styles.css`、离线测试。

写：
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 关键根因（#3 最重要）

读了 `transcript_reader.py` 后确认：codex rollout 同一轮对话存在两条并行流——

- `event_msg`：codex 自己 UI 展示的干净轮次（用户消息 / agent 回复）。
- `response_item`：原始 API 视图，会把系统提示词 / 环境上下文当成第一条「user」轮注入，并且把后续每条消息再复制一遍。

reader 把两条流都映射成了 `user_message` / `assistant_message`，前端又全量渲染，于是「又乱又重复、带出系统提示词」。reader 已经把 `metadata.raw_type` 透传到前端，所以可以在前端按 `raw_type` 区分。

会话来源确认：当前后端走 `SessionSourceMode::RealWithSqliteFallback`，会话标题直接来自 `~/.codex/state_5.sqlite` 的 `title` 字段（codex 实际会话标题），空标题回落「未命名会话」。所以 #2 的「显示 codex 实际标题」本就成立，无需改数据层。

## 改动

1. #1 说明文本：
   - `App.tsx`：去掉「已读取索引。所有本机动作仍需用户点击并确认」常驻提示；notice 初值改空；notice 面板只在有 notice 或 error 时渲染（错误提示仍保留）。footer「本机动作需确认」改为角色名「秘书」。
   - `AgentView`：智能体页 description 置空；reader 头部下沉冗余 detail grid（项目归属 / 来源 / 会话编号四格）和「正在读取单个会话正文，只读取当前选中的 rollout…」「还没有读取正文」等说明文本。
2. #2 项目分类 + 真实标题：全局会话保持按 `project_root` 分组（已有），分组头只显示项目末段；卡片标题用 sqlite 真实 `title`。
3. #3 自动加载 + 干净对话：
   - 选中会话即自动读取 transcript（`useEffect` 监听 selectedSession + `useCallback` loadTranscript + selectedThreadId ref 防竞态），无需先点按钮；「读取正文」按钮改为「重新读取」。
   - 新增 `conversationTurns`：优先取 `raw_type === "event_msg"` 的 user/assistant 轮，无 event_msg 流时回退 response_item；过滤空文本轮。过程事件（工具调用 / 上下文 / 系统）默认折叠，可勾选「显示过程事件」查看。
4. #4 收窄列表：`.agent-session-shell` 左列 360px → 248px，gap 14→18。
5. #5 正文顶起：列表 sticky `top:0`、`max-height` 用视口表达式；正文区不再被页头下推（页头移进左列 `agent-list-head`）；chat 工具条从顶部移到对话流底部；chat-stream max-height 改视口表达式。

## 验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过（offline interaction tests passed: 3）。新增 `runTranscriptCleaningScenario`：双流去重、去系统提示词注入、无 event_msg 时回退 response_item，均断言通过。
- `npm run build`：通过（chunk>500kB 警告为既有，无关本次）。

## 不接受为

- 不接受为真实 Tauri 窗口截图级验收。沙箱内无法启动 Tauri，未采集真实窗口截图；#1/#4/#5 的实际观感、#3 在真实 rollout 上的清洗效果，都需要在真机 `npm run tauri:dev` 里确认。
- 不接受为读取过 `~/.codex` 真实会话正文。`conversationTurns` 的双流假设基于 `transcript_reader.py` 的 `raw_type` 透传和对 codex rollout 结构的理解，用离线 fixture 验证；真实 rollout 里若存在 reader 未覆盖的事件形态，可能仍需微调。
- 不接受为多智能体会话底座完成。仍只重做 Codex 会话可读性。

## 边界遵守

- 未执行真实 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未改 workflow state JSON 结构 / 状态机。
- 未改首页内容。
- 未改后端 Rust 代码和 `transcript_reader.py`（清洗放在前端，reader 透传字段已够用）。

## 残留 / 下一步

- 真实 Tauri 窗口截图验收（重点：智能体页自动加载 + 干净对话 + 收窄列表 + 顶起正文）。
- 真实 rollout 上验证 `conversationTurns` 的双流判断是否对所有历史会话都成立；若某些老会话只有 response_item，回退路径已覆盖，但仍建议真机抽查。
- `styles.css` 中 `.agent-session-item*` / `.session-summary-row` / `.agent-session-group*` 等为本轮重写后遗留的孤儿样式，未顺手删，留待清理切片。

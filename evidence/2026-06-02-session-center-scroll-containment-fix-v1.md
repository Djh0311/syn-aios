# Evidence: session center scroll containment fix v1

日期：2026-06-02

## 触发原因

用户反馈 Claude 已改过会话中心 UI，但会话列表和对话框仍不能滚动。

## 依据

- 用户提供的粘贴文本记录了 V1 / V2 / V3 三轮会话中心修复内容。
- 当前代码里会话中心主要落在：
  - `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
  - `prototypes/productized-desktop-shell/src/styles.css`

## 判断

滚动未生效的主要风险是高度链不完整：

- 外层页面已禁止整体滚动，但内部滚动区需要从页面根到左右两栏逐层有 `height` / `min-height: 0` / `overflow`。
- CSS 中会话相关规则存在多段定义，单纯给 `.chat-stream` 加 `overflow: auto` 不一定足够。
- 当前消息仍会一次性渲染全部 conversation，不符合“消息要收纳起来”的要求。

## 改动

### `src/views/AgentView.tsx`

- 给 Agent 页面根节点增加 `agent-view-root`，让样式能只作用到全局会话页，不误伤项目内嵌会话面板。
- 移除未使用的 `streamRef`。
- 消息默认只显示最近 12 条，较早消息进入“已收纳较早 N 条消息”，用户点击后才展开全部。
- 切换会话时自动恢复默认收纳状态，避免上一条会话的“展开全部”状态串到下一条。
- 长消息默认折叠，单条消息可展开 / 收起。

### `src/styles.css`

- 给 `.agent-view-root`、`.agent-session-center.embedded`、`.agent-session-shell`、`.agent-session-list`、`.agent-transcript-panel`、`.session-reader`、`.transcript-shell`、`.chat-stream` 补高度链和 `min-height: 0`。
- 会话列表固定在自身框内 `overflow-y: auto`。
- 消息面板固定在自身框内，消息流 `overflow-y: auto`。
- 新增 `.chat-fold-notice`、`.chat-bubble .body.is-collapsed`、`.chat-expand` 样式。

## 验证

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`

`npm run build` 仍有既有 Vite chunk size warning，不是本轮新增错误。

## 未验证

- 本轮没有真实浏览器截图。原因：当前环境没有可用 Playwright 包，也没有暴露可截图的 in-app Browser 工具；本地已有 `127.0.0.1:5173` 服务，但未进行截图级验收。
- 本轮没有启动 Tauri。

## 边界

- 未改会话数据模型。
- 未改 transcript 读取逻辑。
- 未改 workflow state。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未写 `/Users/yoyi/.codex`。
- 读取了用户明确提供的附件 `/Users/yoyi/.codex/attachments/fd9911dd-d991-4656-94fd-69ed35b0c638/pasted-text.txt`。

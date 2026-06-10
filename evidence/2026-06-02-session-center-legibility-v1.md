# Evidence：会话中心可读性重做 v1

更新时间：2026-06-02

## 任务

用户反馈：工作台会话「看不清」，Codex 会话管理和会话信息很混乱，不像原生体验。用户授权把这条提前到当前主线第一条（Codex 原生感会话中心 / 多智能体会话底座）做，不强求复刻 codex 原生，目标是「良好优秀的对话体验」。

风险路径：Standard Path（UI / 前端，单视图重做，无 schema、无状态机、无真实 Codex）。

## 读 / 写范围

读：`CURRENT.md`、`AUTHORITY.md`、`README.md`、`STAGE_PLAN.md`、`AGENTS.md`、`tasks/README.md`、总执行包、`skills/using-superpowers/SKILL.md`、`prototypes/productized-desktop-shell` 下会话相关前后端代码与 CSS、离线测试、Tauri 截图清单。

写：
- `prototypes/productized-desktop-shell/src/lib/format.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 根因诊断（改前）

会话数据链路：`~/.codex/state_5.sqlite`（只读）→ `codex_db::read_threads` → `SessionRecord` → `AgentView` 按 `project_root` 分组渲染。问题在表现层，不在数据层：

1. 强制两步下钻：进智能体页先选「软件层」卡片，Claude Code / OpenClaw 未接入却长期占位显示「未接入」，第一屏是占位噪声，真实会话被压到下方。
2. 会话列表是塞在 360px 窄栏里的 4 列表格（`.ptable`），中文标题换行挤压；为卡片准备的 `.agent-session-item` 样式是死代码，从未被使用。
3. 会话身份弱：空标题统一「未命名会话」造成大量重名；分组头打印完整绝对路径；时间只有绝对时间戳，没有相对时间，没有「最近活跃」感。
4. 全局低对比：`ink-light`/`ink-mid` + 10–12px，层级弱，扫读困难。

## 改动

1. `format.ts` 新增 `relativeTime`（刚刚 / N 分钟前 / N 小时前 / N 天前，超过 7 天回落绝对日期）和 `pathTail`（取路径末段）。
2. `AgentView`：
   - 删除强制「选择智能体」软件卡步骤和 `buildSoftwareSummary` 调用路径；改为只有在确实存在多软件来源时，才显示一行轻量筛选 chip（全部 / 各软件 + 计数），单软件时不显示。
   - 复用共享 `AgentSessionCenter`，新增 `filterBar` 插槽。
3. `AgentSessionCenter` 会话层：
   - 窄栏 4 列表格换成可读会话卡：第一行状态点 + 会话标题（serif、14px、单行省略），第二行相对时间 · 模型 · 状态标签。
   - 分组头项目名只显示路径末段（hover 显示全路径），不再整条绝对路径占行。
   - 选中态用朱砂左边框 + 浅底高亮。
4. `SessionReader` 头部：标题下加一行「项目末段 · 相对时间 · 模型」副标题；原始 thread_id 从主标题位下沉到详情网格的「会话编号」格。
5. CSS 新增 `.session-filter-bar`/`.filter-chip`、`.session-group`/`.session-card`、`.session-reader-sub`，提高标题字号与对比；保留墨色 ink-wash 语言。
6. 离线测试更新：`AgentView` 断言改为新 IA（含 `会 话 层`、`选会话即读对话正文`、`offline-model`、`codex-workbench` 项目末段），并新增反向断言「不应再强制先选软件层」。默认 scope 的 `AgentSessionCenter` 断言（`Codex 会话中心` / `会 话 层` / 软件层 / `OpenClaw`）保持不变并继续通过。

## 验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过（offline interaction tests passed: 2）。
- `npm run build`：通过（202 modules，dist 产出；chunk>500kB 警告为既有，无关本次）。

## 不接受为

- 不接受为真实 Tauri 窗口截图级验收。本轮在沙箱内无法启动 Tauri，未采集真实窗口截图；视觉证据待真实 Tauri 验收线（Skeleton-04 同款流程）单独补。
- 不接受为读取了 `~/.codex` 真实会话正文。全程未读真实 sqlite / rollout 正文，验证基于离线 fixture。
- 不接受为多智能体会话底座完成。本轮只重做 Codex 会话的可读性与信息架构；Claude Code / OpenClaw 仍只是筛选位，未真实接入。
- 不接受为 schema / 状态机 / workflow state 变更。未改任何事实结构。

## 边界遵守

- 未执行真实 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未改 workflow state JSON 结构或状态机。
- 未改首页内容。
- 未把任务包管理器搬回主界面。

## 残留 / 下一步

- 真实 Tauri 窗口截图验收（首页不变，重点截智能体页会话层）。
- `softwareGroupsForSessions` 为改前既有死导出，本轮未顺手删，避免扩大范围；可在后续清理切片移除。
- 后续若推进「多智能体会话底座」，筛选 chip 行可自然承载真实接入的软件来源。

# L3 R4 pre-Gate-0 既有 store 面停点指导复核 v1

- 日期：2026-07-24
- 结论：**ACCEPTED BLOCKER / 不是 relay、binding 或五工具面失败**
- 复核方式：只读检查执行线回报、当前任务包/真实 App evidence、目标启动路径与 Git 校验；未启动 App、Codex CLI/MCP server，未读取真实 vault/store 内容。

## 1. 裁决

执行线在发主管首句前发现 Syn 首屏已经呈现既有非验收项目/待办等状态，并立即退出，符合当前任务包“触及既有非验收条目即停止”的合同。由于 Gate 0 尚未开始，本轮不能归因为：

- conversation binding 建立失败；
- `tools/list` 不是精确五项；
- `knowledge_open` relay 失败；
- 十二项知识工作区验收失败。

正确状态是：

`BLOCKED_REAL_APP_PRE_GATE0_EXISTING_STORE_SURFACE`

十二项仍为 `0/12 executed`，R3 离线验收事实不变，shape `17/5/5` 仍是单列历史债。

## 2. 独立复核到的实现事实

仅把 Tauri app-data 指向临时目录不足以解除 blocker。当前启动/运行路径至少同时存在以下默认来源：

1. `AppState::new()` 从仓库 `index-kernel/codex-index.json` 与 `tasks/README.md` 取首页项目/任务；
2. 默认 snapshot 使用 `SessionSourceMode::RealWithSqliteFallback`，会解析 `$HOME/.codex/state_*.sqlite`；
3. workflow state、知识 vault、recovery 与 Canvas 分别从默认 `CodexGovernanceWorkbench` app-data 根派生；
4. supervisor 启动需要保留真实 `HOME` 供既有受控认证复制流程使用，不能用整体改 `HOME` 的办法隔离应用状态。

因此下一步必须是一个进程级、验收专用、默认行为不变的 runtime profile：在任何窗口、migration、reaper、snapshot 或 command 读取之前完成校验，并把 index/tasks/session source/workflow/vault/recovery/canvas 全部路由到唯一临时验收根。profile 无效时必须在窗口出现前失败关闭。

## 3. Git 与非动作

- `git diff --check`：通过；
- 暂存区：空；
- 本指导复核没有修改代码、真实 store、vault、binding、DB/JSON schema、能力 allowlist 或权限；
- 没有启动 Syn、Codex CLI/MCP server、Obsidian；
- 没有 stage、commit、push、reset、clean 或 stash；
- 本轮没有新增 harness catch；上述路径面是下一包的设计输入，不写入 `docs/harness-catch-log.md`。

## 4. 下一步边界

先执行：

`tasks/2026-07-24-l3-syn-r4-isolated-runtime-profile-preflight-package-v1.md`

该包只做到“离线证明 + 隔离 Syn 首屏 smoke”，然后停在 Gate 0 前回交指导验收。不得在同包发送主管首句、启动 Codex CLI/MCP server、调用 `tools/list` 或进入十二项。


# Syn M5 → M6 与新壳后续欠账交接 v1

日期：2026-08-18

状态：`DEFERRED_INPUT_ONLY / M6_NOT_ACTIVE / SYN_SHELL_NOT_ACTIVE / NO_PIXEL_EVIDENCE`

来源：M5R07 独立结论 `M5R07-20260818-1344.verdict.md` 的欠账 4、8，以及当前 M5R08 leaf 的完成标准 8。本文件只记录依赖和禁止继承边界，不启动 M6、F3、F5 或任何 `syn-shell` 实施。

## 1. M6 入口前置

- M5R08 已在本仓候选中处理六个 memory/mature governance command 的 M1 canonical ProjectId 消费、attempt/grant-scoped runtime carrier 与 ordinary identity source 同句柄读取；这些事实仍须 M5R08 独立验收，不能从 working copy 或本交接推断为已接受。
- M6 只有在 M5R08 及后续 M5 closeout 获得适用的独立验收/用户授权后才可激活。当前 `stage-15` 不存在 active stage，本文件不是授权票。
- 当前 6 个未跟踪 `m6_*.rs`（含 `m6_member_directory.rs.bak`）只是受保护的未归属候选字节；不得把它们视为 M6 基线、自动暂存、clean、恢复或实现输入。逐项 hash 和 disposition 见 `docs/harness/reports/M5R08-protected-wip-attribution-v1.md`。

## 2. 新壳 F3 的 acceptance-driver 继承禁令

- 普通旧 Tauri 前端的 M5R07 acceptance driver 已改为仅当构建变量 `VITE_SYN_M5R07_ACCEPTANCE_DRIVER` 精确等于 `1` 时进入 bundle；默认 production build 不携带这些 M5R07 driver 标记，服务端 `status.active` / `status.isolated` runtime gate 保留。
- 新壳 F3 不得复制、移植、默认启用或从旧 `main.tsx` 继承 M5R07 acceptance driver、DOM 自动点击、验收 receipt 写入或验收专用选择器流程。若 F3 需要自己的验收接线，必须另有明确任务和 acceptance-only build/runtime gate，且不得进入正常发行载荷。
- M5R08 没有进入 `syn-shell`，没有形成 F3 implementation、distribution 或 adoption 事实。
- 截至 M5R09，本交接尚未被 F3 接收；“不得继承 M5R07 acceptance driver”仍是待接收边界，不是已完成的壳侧实施事实。

## 3. 新壳 F5 的真实窗口像素责任

- M5R07 的 `NO_WINDOW_CAPTURE` 与本 M5R08 的本地构建/离线测试都不构成真实桌面窗口像素证据。
- 真桌面窗口像素证据保留给新壳 F5 一次性完成：必须绑定新壳自己的候选 SHA/构建、真实可见窗口与明确截图/观察载体，并区分页面像素、窗口 chrome、运行进程和后端状态。旧壳截图、DOM/SQLite 推断、Xvfb 载体或静态源码均不得替代。
- F5 的启动、允许工具、截图范围、隐私边界和验收标准须由后续适用授权决定；本文件不授权浏览器、Computer Use、真实资料或真实账号动作。
- 截至 M5R09，F5 尚未启动或接收该责任，真实窗口像素证据仍不存在。

## 4. `syn-shell` F2 的首项接收责任（仍未完成）

- `syn-shell` F2 后续获得适用授权并启动时，第一件事必须把本文件第 2、3 节登记为明确下游责任：F3 acceptance-driver 继承禁令与 F5 真实窗口像素责任。
- 当前未建立、未启动或未验收 F2/F3/F5 leaf；本条只保留可追踪的接收责任，不声称 `syn-shell` 已接收、实现或验证。

## 5. 当前停止边界

- M5R08 到节点后只请求本 leaf 的独立验收；不自行归档 M5R08，不关闭 stage-14，不宣布 M5 完成。
- 不 push、merge、rebase、deploy、release；不进入 M6、F3、F5；不接真实 provider、账号、凭据、个人资料或外部网络业务写。

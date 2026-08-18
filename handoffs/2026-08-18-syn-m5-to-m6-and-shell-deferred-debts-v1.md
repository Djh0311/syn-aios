# Syn M5 → M6 与新壳后续欠账交接 v1

日期：2026-08-18

状态：`M5_CLOSEOUT_INPUT / M6_NOT_ACTIVE / SYN_SHELL_NOT_ACTIVE / NO_PIXEL_EVIDENCE`

来源：M5R07 独立结论 `M5R07-20260818-1344.verdict.md` 的欠账 4、8，M5R08/M5R09 后续收敛，以及 `M5R09-20260818-1836.verdict.md`。本文件只记录已接受 M5 输入、后续依赖和禁止继承边界，不启动 M6、F2、F3、F5 或任何 `syn-shell` 实施。

## 1. M6 可接收输入与入口前置

- M5 产品内容锚为已独立接受的 `c91d8fc72bcbf80186736caff841cb7a9b0660d1` / tree `fe2d982267d474631ca4ea7b3f90ed846f72a89d`；M5R09 接受记账为 `8e6f59f48d2d90891d3c02396378921e4a2f5d6e` / tree `2043660c9547c6c102ae24414674918ca8215eb0`。后续 M5C01 只绑定 closeout 文档和生命周期，不改该产品锚。
- ProjectSummary 输入固定为 `docs/contracts/m5-project-summary-projection-v1.md` 与 `m5_project_summary.rs` 的 `ProjectSummaryQueryPort`：version、watermark、source refs、summary hash、consumer RoleSession/scope/expiry/policy gate、stale/foreign 拒绝和只读不可反写必须保留。
- TemporaryAgent / Advisory 的执行引用固定为 M5 完整 envelope：`project_id + orchestration_id + workflow_run_id + work_item_id + node_id + dispatch_id + attempt_id + grant_id + worker_role_session_id + authoritative receipt + trusted actor + hashes`。不得从 report 自报、缺字段兼容或 runtime trace 推导正式执行身份。
- 旧入口继续按 `m5_runner_entry_registry` 的 `new-grant / guarded-legacy / blocked` 分类；M5 未物理删除旧路。M6 不得把 guarded legacy、当前未跟踪 `m6_*.rs` 或 `.bak` 自动升格为基线，也不得在回切时放宽 Grant/receipt/audit/quarantine。
- M6 域层施工前还须处理 `docs/harness/unfinished/M6P00-canonical-project-id-consumption-and-relation-owner-typing.md`：把 canonical ProjectId 扩到 workflow/执行链消费面并类型化 relation project owner。该 unfinished 是后续前置记录，不反向否定 M5 scoped PASS。
- M6 只有在 M5C01 closeout 完成、stage-14 关闭且用户另行明确开始后才可激活。当前 `stage-15` 不存在 active stage，本文件不是授权票。
- 当前 6 个未跟踪 `m6_*.rs`（含 `m6_member_directory.rs.bak`）只是受保护的未归属候选字节；不得把它们视为 M6 基线、自动暂存、clean、恢复或实现输入。逐项 hash 和 disposition 见 `docs/harness/reports/M5R08-protected-wip-attribution-v1.md`。

## 2. 新壳 F3 的 acceptance-driver 继承禁令

- 普通旧 Tauri 前端的 M5R07 acceptance driver 已改为仅当构建变量 `VITE_SYN_M5R07_ACCEPTANCE_DRIVER` 精确等于 `1` 时进入 bundle；默认 production build 不携带这些 M5R07 driver 标记，服务端 `status.active` / `status.isolated` runtime gate 保留。
- 新壳 F3 不得复制、移植、默认启用或从旧 `main.tsx` 继承 M5R07 acceptance driver、DOM 自动点击、验收 receipt 写入或验收专用选择器流程。若 F3 需要自己的验收接线，必须另有明确任务和 acceptance-only build/runtime gate，且不得进入正常发行载荷。
- M5R08 没有进入 `syn-shell`，没有形成 F3 implementation、distribution 或 adoption 事实。
- 截至 M5C01 closeout，本交接尚未被 F3 接收；“不得继承 M5R07 acceptance driver”仍是待接收边界，不是已完成的壳侧实施事实。
- M1 `UNENROLLED` 主动引导与服务端状态投影由 `docs/harness/unfinished/F3-m1-unenrolled-guidance-and-status-projection.md` 跟踪；新壳不得用前端 path 派生或自动登记替代权威状态。

## 3. 新壳 F5 的真实窗口像素责任

- M5R07 的 `NO_WINDOW_CAPTURE` 与本 M5R08 的本地构建/离线测试都不构成真实桌面窗口像素证据。
- 真桌面窗口像素证据保留给新壳 F5 一次性完成：必须绑定新壳自己的候选 SHA/构建、真实可见窗口与明确截图/观察载体，并区分页面像素、窗口 chrome、运行进程和后端状态。旧壳截图、DOM/SQLite 推断、Xvfb 载体或静态源码均不得替代。
- F5 的启动、允许工具、截图范围、隐私边界和验收标准须由后续适用授权决定；本文件不授权浏览器、Computer Use、真实资料或真实账号动作。
- 截至 M5C01 closeout，F5 尚未启动或接收该责任，真实窗口像素证据仍不存在。

## 4. `syn-shell` F2 的首项接收责任（仍未完成）

- `syn-shell` F2 后续获得适用授权并启动时，第一件事必须把本文件第 2、3 节登记为明确下游责任：F3 acceptance-driver 继承禁令与 F5 真实窗口像素责任。
- 当前未建立、未启动或未验收 F2/F3/F5 leaf；本条只保留可追踪的接收责任，不声称 `syn-shell` 已接收、实现或验证。

## 5. 当前停止边界

- M5C01 只在精确 closeout 内容与 lifecycle 载体形成后关闭 stage-14，并写仓外节点请求等待独立复核；不自行建立 stage-15 或 M6 leaf。
- 不 push、merge、rebase、deploy、release；不进入 M6、F2、F3、F5；不接真实 provider、账号、凭据、个人资料或外部网络业务写。

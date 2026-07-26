# 证据：S1B-H2-R4F 一次真实工具归因与 Pending 卡收口 v1

- 日期：2026-07-22
- 状态：现场已止损；H2 live 未通过
- 执行合同：`tasks/2026-07-22-s1b-h2-r4f-one-pass-real-app-tool-attribution-and-pending-card-package-v1.md`
- 后续唯一入口：`tasks/2026-07-22-s1b-h2-r4f-preflight-home-repair-package-v1.md`

## 0. 结论

本轮有效 Gate 0/1 后，首句以新的 message identity 仅完成 canonical `recorded`，在 resident runner 启动前止于安全事实 `stage=preflight`、`stable_error_family=preflight_home`。同一 message 没有 injected、自然 reply、resident binding 或 R4E 工具诊断。

因此第二句合规未发送；R4E 五阶段和 A/B/C/D/LIVE-DIAG 矩阵均未到达，不能伪造工具路径裁决。没有 Pending 卡、chain、worker 或固定测试项目变更。本轮唯一后续是受控 private-home 的 fail-closed 最小修复包；未在本轮修码。

## 1. 有效 Gate 0 与 Gate 1

- 新鲜 Gate 0：Workbench/dev/Codex/MCP scoped process、registry、lock、workflow-state、DB/WAL/SHM holder 全空；active registry entries=`0`。相关六源码 hash 与已归属工作线一致，未见无法归属的重叠；HEAD=`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`，staged=`0`，porcelain=`50`。
- 固定测试项目：holder=`0`，HEAD=`caa02ded684d9e1d92d00c367949fab6f83430d1`，staged=`0`，porcelain=`14`，全文件 manifest=`f9c8867116851f688ee1311869c8703fd1f7f4f833cecd482eb42bb9115ad9a4`。
- Gate 0 安全基线：目标工作流 canonical `R/I/S/D=13/5/5/0`；全局 proposal/Pending/chain=`74/17/40`、目标工作流=`59/11/29`；R4E tool diagnostic=`0`；storage initialized/degraded=`41/11`。
- Gate 1：既有 debug build 命令 exit `0`（仅既有 warnings）；六源码 hash build 前后不变。冻结的裸 binary SHA-256=`d761786dd0d878d5acf371ef45be7e02f9c3b876ba214b873999f283a9bec425`，大小=`66570296` bytes，mtime epoch=`1784721198`。

发送前曾发生一次无效的 App bundle 启动尝试；在任何消息、工具或卡片写入前即正常退出，随后重新执行本节的有效 Gate 0/1。该无效尝试仅使 storage initialized 历史计数进入本轮有效基线，不得作为 R4F 验收事实。

## 2. 唯一首句的安全增量

新 message short digest=`0c7ca0e1fb8fce19`。从有效 Gate 0 到退出后：

| 面 | 基线 | 退出后 | 增量 |
| --- | ---: | ---: | ---: |
| recorded | 13 | 14 | +1 |
| injected | 5 | 5 | +0 |
| natural reply | 5 | 5 | +0 |
| delivery diagnostic | 0 | 1 | +1 |
| R4E tool diagnostic | 0 | 0 | +0 |

该 message 的 delivery diagnostic 恰一条：`stage=preflight`、`stable_error_family=preflight_home`、generation=`6`、thread short digest=`c4cdd7e81ff8e498`。supervisor sidecar 中同 message 的 session active-message match 与 lifecycle-audit match 均为 `0`，active host PID=`0`。

源码交叉证据表明 `preflight_home` 在 controlled resident-home `ensure_active` 边界映射，且发生在 watchdog runner 前。因此本轮没有 runner、`thread.started`、injected 或 MCP tools/list/call 的可能。该 family 覆盖受控 home 的创建和既有 home 校验多个 fail-closed 分支，现有脱敏事实不能诚实缩小为 auth、配置、权限或元数据中的单一根因。

## 3. 第二句、工具与业务不变量

- 第二句：未发送；无重发、无技术性 retry。
- R4E 五阶段：不适用，全部未到达；不存在可绑定的 tools/list、tools/call、handler 或 audit/outcome 事实。
- proposal/Pending/chain：全局保持 `74/17/40`，目标工作流保持 `59/11/29`；无新增目标 Pending 卡、无 refresh。
- 未出现工具批准弹窗、第二工具、卡片点击、方案批准、chain/worker 启动或固定测试项目写入。

## 4. 退出与最终对账

用户正常退出本轮冻结的裸 binary。随后：精确 binary process=`0`、registry entries=`0`、store holder=`0`、实际 DB/WAL/SHM holder=`0`、lock=`0`；没有 residual，故未使用 TERM。

普通 readonly SQLite 打开仍复现既有不可用；在 WAL/SHM 均缺席且 holder=`0` 的前提下，采用 immutable readonly 口径，`integrity_check=ok`。DB/JSON count-level 投影一致：canonical=`14/5/5/1`、R4E=`0`、全局 proposal/Pending/chain=`74/17/40`、目标工作流=`59/11/29`。这只是计数级对账，不替代全语义 reconcile 结论。

退出后 storage initialized/degraded=`42/11`：本轮有效裸 App 启动产生一条 initialized audit，没有新增 degraded-json-only。固定测试项目 manifest 与 Gate 0 完全相同；六个冻结源码 hash 亦完全相同。

最终受控 store SHA-256：workflow state=`c45420c1acc9607853ec6ea03dc5bb464b2b4ef83e903b75a05a059b4802cc38`、proposal store=`3d7d965e02fb12761d5f7e9d85218fd154050131edf77e92951f90540238f631`、supervisor store=`e63079fcaad521a823e33a2c4cc1bce9ecb2536f4e24e9f2407646a914f7140b`、registry=`cac021ebb443ba3c1df32221aa21ee488f439d6f3da812a8fbcf8cad54a28310`、DB=`42b4e80c2dd94c96a27811aff51aa98be73786cff49d79c40f1558bf0ac10d79`；WAL/SHM absent。

## 5. 裁决与停止

裁决是**首句 Gate 2 preflight blocker**，不是 A/B/C/D/LIVE-DIAG：后五项只适用于首句交付完整后发送的第二句。本轮已以 canonical 增量和同 message 安全 diagnostic 两类证据交叉证明最早边界。

唯一后续包必须只修受控 private-home 预检的可证明分支，并保持 fail-closed。它不得改 H2 单工具预批准、MCP transport、watchdog、invalid-resume、进程清理、M5 或真实 store；代码/离线完成即停，新的现场验收必须另包、另授权、重新 Gate 0 与新 identity。

## 6. 审计与写面

本轮没有源码、配置、审批或真实 store 的直接写入；没有 stage、commit、push、reset、clean 或 stash。永久仓内写面仅为本 evidence、`CURRENT.md`、catch log 新拦截条目，以及一份修复任务包和 kickoff。本文未记录用户/主管正文、完整 identity、原始错误/stderr、argv、环境、token/auth 或私有路径。

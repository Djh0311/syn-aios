# M6S01 双项目隔离 App 验收与顶层入口 UI（ORG-007，载体在新壳）

阶段：M6，但**不属 stage-15**。载体是新壳（`syn-shell`），须在 F2/F3 就绪并由用户明确开始后另排。

状态：`PLANNED` / `NOT_STARTED` / `BLOCKED_ON_SHELL`。stage-15 只做 M6 域层，本叶是被明确排除在 stage-15 之外的那一段。

来源收据：stage-6 计划第 4 节 SYN-ORG-007 与第 7 节验证矩阵 "Isolated Tauri" 行；载体改写依 `decisions/2026-08-17-syn-lightcode-fork-desktop-shell-direction-v1.md` 与 `docs/plans/2026-08-17-syn-lightcode-fork-shell-adoption-plan-v1.md`；真实窗口像素责任依 `handoffs/2026-08-18-syn-m5-to-m6-and-shell-deferred-debts-v1.md` 第 3 节（欠在 F5）。

欠什么：

1. 顶层 Global Supervisor 入口 UI 与成员目录 UI（导航、Active View、入口与项目主管 / 秘书清楚分离），载体是新壳；
2. 两个 scratch projects + fake roles 的隔离 App 验收，覆盖冲突发现、stale summary、ACL denied、consult、稳定 / 临时成员查找与直接联系；
3. source deep link 点击回源、跨重启、意见不反写的真实 App 证据；
4. 真实桌面窗口像素证据（窗口可见、截图 / 交互载体），按 F5 的一次性责任结算，旧壳截图、DOM / SQLite 推断、Xvfb 载体或静态源码都不算；
5. UI 明确区分"临时"与"稳定"成员；stale availability 在界面上不被当能力。
6. M6P00 PASS verdict 点名的隔离档前置：`SharedProductAuthorityProfile::IsolatedUninstalled` 当前不安装 M1 project-index authority，13 个 canonical command 会正确以 `m1_project_index_unavailable` fail-closed。新壳双项目隔离验收前必须明确裁定并实现隔离 profile 是否/如何安装独立 M1 authority；不得靠 path-derived fallback 或复用普通 profile 数据绕过。
7. CP1/CP2 PASS verdict 点名的 renderer 消费：新壳必须实际调用（或经正式桥接保持等价 typed contract）以下 8 个已注册 command：`load_global_supervisor_role_session_status`、`run_global_supervisor_cross_project_advisory`、`adopt_global_supervisor_cross_project_advisory`、`observe_global_supervisor_advisory_application_receipt`、`attempt_global_supervisor_project_write`、`start_secretary_global_supervisor_consult`、`decide_global_supervisor_consult_handoff`、`read_secretary_global_supervisor_consult_receipt`；展示 ready/unavailable、只读/恒拒写边界、advisory/采纳/receipt 与 consult lifecycle。源码注册、静态 DOM、旧壳调用或离线命令测试不能冒充真实用户可见消费。

为什么不在 stage-15：syn 仓库源码写面同一时间只允许一个施工者，F2 起的壳侧实施与 stage-15 争用该写面；且 UI 载体已从旧壳改到新壳，旧壳 UI 不再是 M6 的验收载体。

前置：stage-15 M6 域层通过总指导阶段验收；`syn-shell` F2（壳 ↔ Syn 核心桥）与 F3 就绪；F3 必须先接收"不得继承 M5R07 acceptance driver"的禁令；上方 M1 authority 隔离方案已明确；用户明确开始。

不许动（在本叶被正式排入之前）：不得由 stage-15 任一叶提前实现、不得用旧壳 UI 顶替、不得用域层证据声称本叶已完成、不得用离线 fixture 或协议推断冒充真实窗口证据。

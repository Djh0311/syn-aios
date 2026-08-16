# M5R04 普通项目的持久 Project Supervisor

阶段：stage-14 M5 项目主管与执行闭环（事实重整与产品闭环）

目标：用户进入普通项目后面对可恢复的项目主管会话；默认理解和只读，只有明确要求动作时才进入 Proposal 和授权步骤。复用 M3 RoleSession 身份，不自建字符串 session 真源；不得直接 start/dispatch 绕过 Grant。

来源收据：用户 2026-08-16 明确按实际完成剩余 M5；M5R03 PASS（`6b252a3`）。

产品：m5_project_supervisor.rs、M3 RoleSession port、supervisor binding/turns/proposals

证据：docs/harness/reports/M5R04-persistent-project-supervisor.md [新增]

载体：working-copy + 独立内容 commit（opening HEAD=6b252a3）

允许动：

- docs/contracts/（仅新增 M5 supervisor 补充合同；不改 M1–M4 正文与 hash）
- prototypes/productized-desktop-shell/src-tauri/src/m5_project_supervisor.rs
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs（仅本包最小声明）
- tasks/2026-08-16-syn-m5r04-persistent-project-supervisor-v1.md [新增]
- docs/harness/plan.md、docs/current-state.md、docs/harness/audit/2026-08.jsonl、docs/harness/stages/stage-14.md
- docs/harness/reports/M5R04-persistent-project-supervisor.md [新增]
- docs/harness/leaves/M5R04-persistent-project-supervisor.md
- docs/harness/done/2026-08/M5R04-persistent-project-supervisor.md [退场时新增]

不许动：

- M1–M4 冻结合同；m6_*.rs；stage-12 / D0C04 / D0C05
- 绕过 Grant 的 start/dispatch；真实资料/provider/push/reset

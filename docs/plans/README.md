# 产品计划历史索引

> 本目录保存设计、实施、迁移、发布与复核计划。阶段状态已于 2026-08-08 按 main 的 M2 收口事实校正；当前开发工作仍只看 Harness Lite 的 plan、当前 stage、唯一 current leaf 与当前用户授权。

## 当前阶段路由（不自行激活工作）

- 唯一当前总开发计划：`2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md`。
- M1 与 M2 已完成主线收口；M3-M10 仍是 planned，不是 active。
- 当前没有活动工程任务。下一入口是只读复核 M3；实现仍需新的用户指令和 Harness Lite 激活。
- 已确认运行模型：`../../decisions/2026-08-01-whole-workbench-event-driven-operating-model-amendment-v1.md`。

## M1-M10 独立阶段计划

| 阶段 | 文件 | 状态 / 入口 |
|---|---|---|
| M1 | `2026-08-01-syn-stage-1-contracts-and-security-foundation-plan-v1.md` | `COMPLETED / MAINLINE` |
| M2 | `2026-08-01-syn-stage-2-fact-event-audit-transaction-foundation-plan-v1.md` | `COMPLETED / MAINLINE / BOUNDED_REFERENCE_SLICE`；live cutover 未进入本阶段 |
| M3 | `2026-08-01-syn-stage-3-role-session-and-explicit-handoff-plan-v1.md` | `PLANNED / NOT_ACTIVE`；下一入口仅只读复核 |
| M4 | `2026-08-01-syn-stage-4-secretary-attention-and-daily-rhythm-plan-v1.md` | `PLANNED / NOT_ACTIVE`；依赖 M1-M3，可与 M5 条件并行 |
| M5 | `2026-08-01-syn-stage-5-project-supervisor-and-execution-loop-plan-v1.md` | `PLANNED / NOT_ACTIVE`；依赖 M1-M3，可与 M4 条件并行 |
| M6 | `2026-08-01-syn-stage-6-global-supervisor-and-internal-organization-plan-v1.md` | `PLANNED / NOT_ACTIVE`；依赖 M3-M5 |
| M7 | `2026-08-01-syn-stage-7-memory-personal-model-and-skill-governance-plan-v1.md` | `PLANNED / NOT_ACTIVE`；M7-A 等 M1/M2，M7-B live capture 等 M4/M5；A 完成不等于阶段完成 |
| M8 | `2026-08-01-syn-stage-8-connector-and-credential-reference-plan-v1.md` | `PLANNED / NOT_ACTIVE`；CON-000 仅 design-only，framework 与真实 provider 分层授权 |
| M9 | `2026-08-01-syn-stage-9-read-model-migration-and-legacy-retirement-plan-v1.md` | `PLANNED / NOT_ACTIVE`；依赖 M2-M8 replacement evidence |
| M10 | `2026-08-01-syn-stage-10-full-day-pilot-and-release-hardening-plan-v1.md` | `PLANNED / NOT_ACTIVE`；发布动作另获用户指令 |

## 历史计划

- `2026-07-16-master-execution-plan-conversation-first-v1.md`：原 conversation-first 当时的唯一计划；已被当前 master supersede。之后出现过窄范围重建候选，但也已在激活前停止。
- `2026-07-16-conversation-first-direction-and-execution-plan-v1.md`：保留自然对话、结构化实物和明确人闸等产品原则，不再承担当前排期。
- `2026-08-01-jiaoban-bounded-rebuild-execution-plan-v1.md`：曾短暂作为候选当前路线；在 Phase 1 激活前因范围只覆盖项目助手 / 工作流拆分、不能覆盖全工作台后端协作而停止。
- `2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md`：知识线已停放，不能借此恢复施工。
- `2026-07-23-development-harness-routing-code-map-and-authority-governance-remediation-plan-v1.md`：历史治理计划；旧 AUTHORITY/CURRENT 已退出当前入口。
- 其他计划均按历史资料查阅，不能反向授予写入或运行权限。

计划状态只使用以下含义：`CURRENT_MASTER`、`CURRENT_STAGE`、`PLANNED / NOT_ACTIVE`、`HISTORICAL / SUPERSEDED`、`STOPPED_BEFORE_ACTIVATION`、`PARKED / HOLD`。正文里的“当前”“下一步”“唯一”只在其状态栏允许时才有当前效力。

## 编写与启用规则

- 计划必须写清目标、不做什么、边界、阶段、验收层级、停止条件和退役条件。
- 只有 Harness Lite 当前 leaf 与当前用户授权明确覆盖时，计划中的产品切片才可进入代码。
- 离线、synthetic、build、artifact、真实 App、发布分别结算，不得互相替代。
- 当前路线变化时更新 Lite plan/stage/leaf；不要在 README 或长期计划里复制动态状态。

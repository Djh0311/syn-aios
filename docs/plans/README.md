# 产品计划索引

> 本目录同时保存现行总计划、独立阶段计划和历史实施资料。文件名、正文中的“当前”“下一步”“唯一”或过去授权，均不能单独证明它现在有效。

## 先看哪里

- [Syn 产品正本](../product/syn-product-canon-v1.md)：回答 Syn 是什么、服务谁、长期要具备什么。
- [权威登记](../product/authority-register-v1.md)：判断一份材料是现行、专项、被修订、历史、停放还是记录。
- [候选登记](../product/candidate-register-v1.md)：唯一开放候选清单；散落提案和草稿只作来源，不自行保持候选状态。
- [知识基础设施专题正本](../product/knowledge-infrastructure-canon-v1.md)：定义知识库与所有智能体、资料和技能之间的基础设施关系。
- [当前事实](../current-state.md)：记录当前主线、已完成范围、未知和证据上限。

工程执行另按工作区与项目 `AGENTS.md`、`docs/harness/plan.md`、活动阶段、唯一活动叶和 `docs/harness/authorization.json` 判断。计划只定义方向和边界，不自行产生写入、运行、真实消息、外部连接、Git 或发布权限。

## 当前计划关系

- 唯一当前总开发计划：`2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md`。
- 该计划完整覆盖公共底座重构到首个全日试点和发布候选；M10 之后五类长期业务、个人服务器备份、开源成本工具和高级检索按真实需求另建路线，不伪装成已经排完的终身计划。
- M0 产品正本、权威分级、候选收口和文档入口整理已经完成；它是文档收口，不是产品代码阶段。
- M1、M2 已完成主线收口。
- M3 已完成：M3C01–M3C08 已进入主线并归档，M3C08 内容提交为 `fa8e392`，阶段状态为 `COMPLETED / MAINLINE / STAGE-05 CLOSED`。
- M4 已完成：M4C01–M4C10 已进入主线并归档，C09 隔离验收提交为 `c823986c`，C10 launcher 回归修复为 `9e97120`，阶段状态为 `COMPLETED / MAINLINE / STAGE-06 CLOSED`。当前没有活动 stage 或 leaf；M5–M10 仍是计划中、未激活。
- 当前产品运行模型见 `../../decisions/2026-08-01-whole-workbench-event-driven-operating-model-amendment-v1.md`。

## M1-M10 独立阶段计划

| 阶段 | 文件 | 状态 |
|---|---|---|
| M1 | `2026-08-01-syn-stage-1-contracts-and-security-foundation-plan-v1.md` | `COMPLETED / MAINLINE`；关闭验收只证明 M1 |
| M2 | `2026-08-01-syn-stage-2-fact-event-audit-transaction-foundation-plan-v1.md` | `COMPLETED / MAINLINE / BOUNDED_REFERENCE_SLICE`；真实工作台切换未进入 |
| M3 | `2026-08-01-syn-stage-3-role-session-and-explicit-handoff-plan-v1.md` | `COMPLETED / MAINLINE / STAGE-05 CLOSED`；内容提交 `fa8e392` |
| M4 | `2026-08-01-syn-stage-4-secretary-attention-and-daily-rhythm-plan-v1.md` | `COMPLETED / MAINLINE / STAGE-06 CLOSED`；只证明 M4 具名范围与 synthetic isolated App |
| M5 | `2026-08-01-syn-stage-5-project-supervisor-and-execution-loop-plan-v1.md` | `PLANNED / NOT_ACTIVE`；依赖 M1-M3 |
| M6 | `2026-08-01-syn-stage-6-global-supervisor-and-internal-organization-plan-v1.md` | `PLANNED / NOT_ACTIVE`；依赖 M3-M5 |
| M7 | `2026-08-01-syn-stage-7-memory-personal-model-and-skill-governance-plan-v1.md` | `PLANNED / NOT_ACTIVE`；存储策略片与真实日常接入分层依赖 |
| M8 | `2026-08-01-syn-stage-8-connector-and-credential-reference-plan-v1.md` | `PLANNED / NOT_ACTIVE`；设计、框架和真实服务提供方分层授权 |
| M9 | `2026-08-01-syn-stage-9-read-model-migration-and-legacy-retirement-plan-v1.md` | `PLANNED / NOT_ACTIVE`；依赖替代链证据 |
| M10 | `2026-08-01-syn-stage-10-full-day-pilot-and-release-hardening-plan-v1.md` | `PLANNED / NOT_ACTIVE`；发布动作另获用户指令 |

## 停放计划

- `2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md`：知识线施工计划已停放，不因知识基础设施仍属产品核心而自动恢复施工。

停放表示产品方向可能继续有效，但当前没有活动任务、持续授权或下一包入口。

## 历史与被取代计划

- `2026-07-16-master-execution-plan-conversation-first-v1.md`：曾是当时的唯一执行计划，已被 2026-08-01 总计划取代。
- `2026-07-16-conversation-first-direction-and-execution-plan-v1.md`：自然对话、结构化实物和明确人闸等净原则已由产品正本吸收；本文不再承担排期。
- `2026-07-23-development-harness-routing-code-map-and-authority-governance-remediation-plan-v1.md`：旧开发治理计划；旧 `AUTHORITY.md` / `CURRENT.md` 路由已经退出。
- `2026-07-23-l3-obsidian-full-interface-and-maximal-integration-small-stage-plan-v1.md`：已被 Syn 原生知识工作区路线取代。
- 2026-08-01 曾出现的窄范围交办旁路重建只保留为历史来源；对应计划不在当前主线仓库，不再保留一个失效的本地文件指针。
- 除上面列出的当前总计划、M1-M10 阶段计划和停放计划外，本目录其他文件默认按 `HISTORICAL / SUPERSEDED` 阅读，除非权威登记明确给出更窄状态。

历史计划可以提供当时的需求、实现选择、风险、迁移素材和证据指针，但不能反向授予当前写入、运行或排期效力。

## 候选规则

- 开放候选只登记在 `../product/candidate-register-v1.md`。
- 带 `draft`、`proposal`、`recommendation`、`talk-materials` 或“草案 / 提案 / 建议”字样的旧文件，不因名称自动成为开放候选。
- 已被正式决定吸收、明确拒绝、实际执行或后稿取代的材料，标为历史来源，不重复放进候选池。
- 候选必须说明问题、来源、当前判断、所属领域、冲突、重新评估条件和升格条件；未升格前不进入产品正本或执行计划。

## 状态词

- `CURRENT_MASTER`：唯一当前总计划。
- `COMPLETED / MAINLINE`：已完成并进入当前主线，只证明具名范围。
- `PLANNED / NOT_ACTIVE`：已规划，当前没有执行效力。
- `PARKED / HOLD`：方向或素材保留，施工停放。
- `HISTORICAL / SUPERSEDED`：历史或已被后稿取代。
- `STOPPED_BEFORE_ACTIVATION`：激活前停止，没有产生施工授权。
- `CANDIDATE_SOURCE`：候选来源材料；开放状态只看候选登记。
- `COMPLETED / MAINLINE / STAGE-05 CLOSED`：M3 内容提交已进入主线；与本次状态回写同批的终态控制提交执行并归档 M3C08 `done` 与 stage-05 `close-stage`，不据此激活下游实现。
- `COMPLETED / MAINLINE / STAGE-06 CLOSED`：M4C01–M4C10 已完成并归档；只结算 M4 typed refs、协调状态、日报、兼容读面与隔离验收，不据此激活 M5–M10 或升级为真实日常使用。

正文内旧的“当前”“下一步”“唯一”只在文件状态与本索引一致时有效。

## 编写与启用规则

- 计划写清目标、不做什么、边界、依赖、验收层级、退出和退役条件。
- 只有当前用户指令及匹配的活动阶段、活动叶和授权才能进入工程执行。
- 离线、合成夹具、构建、产物、真实桌面应用、真实智能体和发布分别结算，不能互相替代。
- 当前路线变化时更新轻量开发护栏入口、权威登记和本索引；长期计划不复制逐任务动态状态。
- 历史文件尽量保留原路径供证据回链。需要合并或降级时，优先加状态头和替代指针；批量移动前先完成引用检查。

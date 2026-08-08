# 决策：开发 Harness 轻量导航运行模型 v1

> **状态校正（2026-08-09）：HISTORICAL / SUPERSEDED BY HARNESS LITE。** 本文记录已经退出的旧开发护栏；正文中的 `AUTHORITY.md`、`CURRENT.md`、第七阶段等待派发和旧命令路由均不再是当前入口。当前只看项目 `AGENTS.md`、`docs/harness/plan.md`、活动阶段、唯一活动叶和 `docs/harness/authorization.json`。

日期：2026-07-23
状态：**已定并落地（Phase 0～6、Phase 5-R1 与 Phase 6-R1 已完成并复核；短路由、权威索引、CURRENT 历史分离、结构化 Code Map、重要任务计划对齐、CLI / config / legacy consumer 收缩及只读 maintenance audit 已落地，自动 / Hook 语义未改变；Phase 7 等待用户单独派发）**
依据：[开发 Harness 短路由、代码图谱与权威治理整改执行计划 v1](../docs/plans/2026-07-23-development-harness-routing-code-map-and-authority-governance-remediation-plan-v1.md)；[Phase 0 基线与 consumer 审计](../docs/2026-07-23-development-harness-phase0-baseline-and-consumer-audit-v1.md)

## 决定

### 1. 开发 Harness 与产品 Harness 永远分开

- **开发 Harness / dev-gate** 仅指仓库内的 `scripts/harness/**`、其配置与开发治理文档；它帮助开发者导航、检查和留证。
- **产品 Harness** 指产品任务中的验收协议、`harness_requirements` 或产品能力。本整改不修改、不替代，也不能由开发 Harness 状态证明其已验收。

这一定义不改变当前业务唯一执行入口：仍以 `AUTHORITY.md` 指向的 07-16 业务总计划及 07-22 shared Conversation Transport 实施包为准。本整改计划是并行开发治理，不抢业务排期。

### 2. 三种权威分别处理

1. 授权与安全：当次用户明确指令 > `AGENTS.md` 高危边界 > 任务包中的更窄限制。
2. 执行路由：`AUTHORITY.md` 当前指针 > 被指向的当前决策或任务包 > 当前业务总计划 > 历史计划、handoff、evidence。
3. 事实判断：真实源码与新鲜直接验证 > `CURRENT.md` 摘要 > Code Map / 现状说明 > 计划状态与历史记录。

未来的短路由和 Code Map 只提供导航、声明的下一步和结构线索；它们不替代源码检查、任务相关测试、build、UI/真实 App 验证或用户确认。

### 3. 默认路径是“人工先跑的短导航”，不是自动总闸

Phase 1 的 `project-context` 已落地为新会话**人工显式执行的第一条导航命令**，而非 Hook、CI、pre-work 或完成门。它保持只读、fail-open，且默认不运行 Git、Hook、Code Map、源码扫描或历史全文读取。

这与 `AGENTS.md` 的“`scripts/harness/**` 默认关、按需手动跑”并不冲突：这里的“默认”是人类/agent 的最短导航顺序，不是自动执行路径。该语义已通过一个可与既有 hunk 分离的最小指针写入 `AGENTS.md`，没有把新决策扩成自动执行规则。

### 4. CURRENT 的每次收口义务是“必复核”，不是“必制造 diff”

每次可提交收口都必须复核 `CURRENT.md`；只有当前事实、唯一下一步、blocker 或权威入口实际变化时才更新文本。这样同时满足 `AGENTS.md` 的回写义务和整改计划的“无事实变化不制造文档噪音”规则。

### 5. Code Map、旧 capability-map 与 config 的边界

- `docs/2026-07-09-codebase-capability-map-v2.md` 已完成调查输入职责，并以历史指针标为 historical/superseded；当前结构化 seed 位于 `docs/code-map/**`，其 public symbol 由 `codebase-map check` 对 `verifiedAtCommit` 源码声明做显式核验。
- 现有 `scripts/harness/capability-map.js` 是本机 Agent/工具能力扫描，不能冒充仓库代码能力图；新的仓库能力工具名称固定为 `codebase-map`。
- Phase 5 已新增 `harness.config.json` 的 `activeBoundary`：四个键固定为 `mechanical`、`reportingOnly`、`explicitTool`、`legacyIgnored`，由 `config-schema.js`、`config-check.js` 与 `config-policy.js` 共同核验；`reportingOnly` / `explicitTool` 不得被声明式 `gates.hard` 升格。默认 help 现为九项，35 条原兼容路由仍可直调、其中 34 条隐藏于 `--legacy`。Phase 5-R1 已把 `context diagnostic` 纳入 `explicitTool`，并锁定默认九项各且仅声明一次；这些开发 Harness 事实不替代业务或产品验收。

### 6. Phase 6 维护审计

- `PHASE6_IMPLEMENTATION_PASS / PHASE6_R1_PASS / PHASE6_STATUS_ALIGNED` 只确认显式、只读 maintenance audit 及其治理回写通过；它不接 Hook、CI、cron 或默认 CLI，也不自动修改文档。
- 首次复核抓到两处假绿：staged canonical rename/delete 未被映射到受影响 capability；默认 CLI help 为空或不可解析时被当成零项而非漂移。R1 补齐 capability impact 报告，以及 `DEFAULT_CLI_BOUNDARY_DRIFT` / exit 1。
- R1 定向回归为 8/8；当前审计六项均 PASS。shape baseline/check 仍为 exit 0 / 1、`17 error / 5 warning / 5 info`；Stage K 仍 exit 0、`0 error / 15 warning / 36 info`，均单列为既有未绿事实，不等于业务或产品验收。

### 7. Hook 与 legacy 的处置顺序

`policy.hooks.enabled=false` 只表示托管的 pre-commit/pre-push 未启用，不等于所有 Git Hook 都关闭。当前 `.githooks/commit-msg` 的 `catch:` 机械检查保持原样；本整改不新增 Code Map 或文档 Hook，也不修改 sandbox、approval 或安全检查退出语义。

旧 CLI、直接脚本、模板和历史命令先保留。任何隐藏、更名或删除都必须先完成 consumer-first 审计、保留兼容路径，并在独立 Phase 中验证；Phase 0 不执行此类行为改变。

## Phase 0 冻结的非目标（历史执行边界）

- 不改 `scripts/harness/**`、`harness.config.json`、`.githooks/**`、产品源码、真实 store、sandbox 或 approval。
- 不新增 task/evidence lifecycle，不把 checkpoint、Code Map 或文档校验接入默认 Hook。
- 不 stage、commit、push，也不覆盖已有 `AGENTS.md`、`AUTHORITY.md`、`CURRENT.md` 的并行业务改动。

## Phase 0 事实快照

基线以 consumer 审计中的命令、清单摘要和 SHA-256 为准。结论是：Harness 本体当前干净，根 CLI 仍显示 35 条混合命令；实际接线仅有 `commit-msg` 的 `catch:` 检查；managed Hook 与 CI 关闭；严格 schema 已存在 legacy 字段漂移。该快照是后续阶段的比较基准，不是对业务代码或历史债的完成声明。

## 下一门

Phase 0～6 已经复核通过：短路由保持只读、fail-open，`CURRENT.md` 已收为四块短视图，完整稳定基线已冻结到 archive，`AUTHORITY.md` 与 plans README 已明确当前、长期、历史和 superseded；结构化 Code Map 已覆盖六域、17 条能力，并提供显式 `query / overlay / check`；重要任务计划对齐只读取短路由显式绑定的 `checkpoint.currentImportantTask`，不扫描历史任务包，字段齐全也不代表语义正确、实现完成或产品验收；Phase 5 以 `PHASE5_IMPLEMENTATION_PASS / PHASE5_R1_PASS / PHASE5_STATUS_ALIGNED` 收口，Phase 6 以 `PHASE6_IMPLEMENTATION_PASS / PHASE6_R1_PASS / PHASE6_STATUS_ALIGNED` 收口。Phase 6 的首次复核两处假绿已由 R1 补齐：staged canonical impact 与空白/不可解析 default CLI help 不再静默通过。当前没有绑定重要任务，`checkpoint-audit --current` 因此诚实返回 `NOT_APPLICABLE`。下一门仅为 Phase 7 回放、观察期与收口，**等待用户单独派发**；任何 Harness 阶段状态仍不等于业务完成或产品验收。

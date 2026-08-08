# M0C01 文档正本权威链与干净本地基线

阶段：stage-04 阶段4 M0 产品与文档正本干净基线收口
目标：只收口接管时冻结的 77 个 M0 Markdown 文件，精确排除产品源码与 M3 实现，并建立可复现的本地干净基线。
干完的标准：77 个允许路径通过权威链、链接、格式和 Git 核验；Harness 查询副作用得到精确恢复；M0 内容提交只包含 77 个允许路径；阶段收口提交后工作树干净。

允许动：

- DEV_LINES.md
- PROTOTYPE_WORK_LINES.md
- README.md
- RESULT_REVIEW.md
- archive/README.md
- backlog.md
- codex-multi-agent-safe-collaboration.md
- decisions/2026-07-23-development-harness-operating-model-v1.md
- decisions/2026-07-23-knowledge-and-conversation-parallel-workstreams-v1.md
- decisions/2026-08-03-syn-m1-closure-acceptance-v1.md
- decisions/2026-08-03-syn-m2-blanket-authorization-v1.md
- decisions/2026-08-09-syn-product-canon-authority-and-knowledge-infrastructure-v1.md
- decisions/README.md
- docs/2026-07-08-workbench-current-feature-inventory-for-prototype-v1.md
- docs/2026-07-09-codebase-capability-map-v1.md
- docs/2026-07-09-codebase-capability-map-v2.md
- docs/2026-07-11-state-machine-failure-path-audit-report-v1.md
- docs/2026-07-13-architecture-review-v1.md
- docs/2026-07-13-sqlite-migration-completeness-contract-m0-v1.md
- docs/2026-07-13-sqlite-migration-m2-txt-migration-plan-v1.md
- docs/2026-07-23-development-harness-phase0-baseline-and-consumer-audit-v1.md
- docs/agent-memory-governance.md
- docs/agent-mistake-ledger.md
- docs/agent-work-summary.md
- docs/context-checkpoints.md
- docs/contracts/README.md
- docs/conversation-model-adaptive-routing-governance-design-v1.md
- docs/current-state.md
- docs/decisions.md
- docs/design/README.md
- docs/evidence/README.md
- docs/execution-entry-inventory.md
- docs/harness-catalog.md
- docs/harness-catch-log.md
- docs/harness-script-audit-2026-06-14.md
- docs/harness-source-package-audit-2026-06-14.md
- docs/memory-layer-consolidated-canon-v1.md
- docs/memory-layer-design-v1.md
- docs/middleware-version-development-plan-v1.md
- docs/open-questions.md
- docs/own-agent-and-company-vision-v1.md
- docs/plans/2026-07-16-master-execution-plan-conversation-first-v1.md
- docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md
- docs/plans/2026-08-01-syn-stage-1-contracts-and-security-foundation-plan-v1.md
- docs/plans/2026-08-01-syn-stage-10-full-day-pilot-and-release-hardening-plan-v1.md
- docs/plans/2026-08-01-syn-stage-3-role-session-and-explicit-handoff-plan-v1.md
- docs/plans/2026-08-01-syn-stage-4-secretary-attention-and-daily-rhythm-plan-v1.md
- docs/plans/2026-08-01-syn-stage-5-project-supervisor-and-execution-loop-plan-v1.md
- docs/plans/2026-08-01-syn-stage-6-global-supervisor-and-internal-organization-plan-v1.md
- docs/plans/2026-08-01-syn-stage-7-memory-personal-model-and-skill-governance-plan-v1.md
- docs/plans/2026-08-01-syn-stage-8-connector-and-credential-reference-plan-v1.md
- docs/plans/2026-08-01-syn-stage-9-read-model-migration-and-legacy-retirement-plan-v1.md
- docs/plans/README.md
- docs/plans/middleware-version-stage-plan-v1.md
- docs/plans/task-package-ui-display-boundary-rule-v1.md
- docs/product/README.md
- docs/product/authority-register-v1.md
- docs/product/candidate-register-v1.md
- docs/product/knowledge-infrastructure-canon-v1.md
- docs/product/syn-product-canon-v1.md
- docs/requirements-matrix.md
- docs/research/README.md
- docs/sprint-contract.md
- docs/syn-global-jarvis-capability-and-explicit-project-entry-recommendation-v1.md
- docs/task-queue.md
- docs/tooling-and-mcp-registry.md
- docs/ux-friction-log.md
- docs/workbench-frontend-display-boundary-v1.md
- docs/workbench-system-architecture-v1.md
- docs/workflow-task-package-design-v1.md
- evidence/README.md
- handoffs/2026-08-01-syn-m0-document-alignment-to-fnd-001-guidance-handoff-v1.md
- handoffs/2026-08-03-syn-fnd-002-r1-m1-wip-committed-handoff-v1.md
- handoffs/2026-08-08-syn-m2-mainline-closeout-to-m3-guidance-handoff-v1.md
- handoffs/README.md
- principles.md
- tasks/README.md

## 步骤

1. 冻结 HEAD/tree、77 个文档路径、staged/conflict 状态与 `.turn` 四行副作用。
2. 核对产品正本入口、权威登记、候选登记、当前状态、阶段计划和交接的一致性。
3. 检查 77 个 Markdown 的相对链接与格式；证明 M1 冻结合同正文没有被本批改动触碰。
4. 只恢复 `.turn` 最后四行，并复核其他工作树与产品源码零写入。
5. 精确暂存 77 个允许路径，核对 staged 路径后形成独立的 M0 内容提交。
6. 归档叶子和阶段，提交必要控制记录，确认提交内容、HEAD 和干净工作树，再建立 M3 阶段。

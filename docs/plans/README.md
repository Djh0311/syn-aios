# Plans Directory

> 本目录保存设计、实施、迁移、发布与复核计划；计划不是状态真相。当前事实以 CURRENT.md 为准，唯一人工索引以 AUTHORITY.md 为准。

## 当前路由

- 唯一业务执行计划：2026-07-16-master-execution-plan-conversation-first-v1.md。它承接 conversation-first 主线与共享 Conversation Transport 的后续顺序。
- 方向与原则：2026-07-16-conversation-first-direction-and-execution-plan-v1.md。它不是第二个业务排期或执行入口。
- 当前知识库子计划：2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md。它只细化总计划中的 L3/N2R，不产生第二条总排期。
- 2026-07-23-development-harness-routing-code-map-and-authority-governance-remediation-plan-v1.md 是并行的开发治理计划，不抢业务排期，也不改变上述唯一业务计划。

## 历史 / 已收官计划

- 2026-06-10-stage-k-daily-use-codex-workbench-productization-plan-v1.md：Stage K 历史计划，不是当前执行入口。
- 2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md：Stage H/I 历史计划。
- 2026-06-01-workbench-architecture-implementation-plan-v1.md 与 2026-06-06-stage-e-f-g-refinement-plan-v1.md：早期架构和阶段细化计划。
- 2026-07-11-orchestrator-fast-path-five-stations-plan-v1.md 及其他旧 resident/private-home 相关计划：仅历史参照，不能借此重新派发。

## 编写计划

- 先标明当前权威、目标、明确不做什么、复用边界、停止条件与可直接复跑的验证。
- 只有确实改变行为、范围或安全边界时，才链接或新增相应 decision。
- 计划依赖的事实必须指向当前 authority；不要把计划内的状态标记当作已完成事实。

## 规则

- 当前业务事实或唯一下一步变化时，更新 CURRENT.md；否则不因计划文本而制造状态 diff。
- 暂停、已收官或历史计划永远不能成为默认派发入口。
- requirements-matrix、task-queue、open-questions、current-state 与旧 docs/decisions.md 等停用文档不再是计划的强制前置或更新目标。

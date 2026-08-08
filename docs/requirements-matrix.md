# Requirements Matrix

> 资料状态（2026-08-09）：工程验收模板，不是产品需求正本。只有当前任务明确采用时才填写；产品长期要求看 `docs/product/syn-product-canon-v1.md`，施工权限仍看当前用户指令和轻量开发护栏。

Purpose: Strict Path owner file for numbered requirements, requirement status, acceptance evidence links, tests/checks, and risks. This prevents completion by vibes without duplicating task packages or sprint-contract scope. Standard Path uses this only when the task already has requirements or the user asks for requirements tracking.

Status values: `Planned`, `Ready`, `In Progress`, `Done`, `Done With Concerns`, `Blocked`, `Deferred`, `Discarded`.

| ID | Requirement | Source | Acceptance Criteria | Owner | Status | Evidence | Tests / Checks | Risk |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| R-001 | TBD | PRD / plan link | TBD | Main / Agent | Planned | TBD | TBD | TBD |

## Update Rules

- Add a row before implementing a new Strict Path requirement.
- Mark `Done` only after verification evidence exists.
- If tests pass but acceptance is incomplete, use `Done With Concerns` or `In Progress`, not `Done`.
- Link evidence to `docs/evidence/**`, `docs/agent-work-summary.md`, test output summaries, PRs, or concrete file paths.
- If a requirement changes, update the row and add or update a decision in `docs/decisions.md`.
- Do not duplicate full task steps; link to `docs/task-queue.md` task IDs.

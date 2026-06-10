# Plans Directory

Purpose: store PRDs, design documents, implementation plans, migration plans, rollout plans, and review plans.

This template describes an installed project's `docs/plans/**` directory. It does not define where the standard rule source package stores its own development plans; source-package plans belong under repo-root `plans/**`.

## Required Plan Links

Every implementation plan should link to:

- `docs/requirements-matrix.md` requirement IDs.
- `docs/task-queue.md` task IDs.
- `docs/decisions.md` decision IDs that constrain implementation.
- `docs/open-questions.md` unresolved questions and conservative defaults.

## Recommended Plan Structure

```markdown
# <Feature Or Phase> Implementation Plan

Goal:

Architecture:

Relevant Requirements:
- R-001

Relevant Decisions:
- D-001

Open Questions:
- Q-001, or None

Tasks:
- TASK-001

Verification:

Risks:
```

## Rules

- Do not use plans as the only source of current truth. Update `docs/current-state.md` and `docs/requirements-matrix.md` as work progresses.
- If a plan changes behavior or scope, add or update a decision in `docs/decisions.md`.
- If a plan depends on unresolved information, add the question to `docs/open-questions.md`.

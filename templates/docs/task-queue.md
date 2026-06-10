# Task Queue

Purpose: owner file for dispatch-ready task packages, task read/write scope, task state, and verification commands. Every Standard/Strict task and every subagent task must have explicit read/write scope, even when no persistent queue entry is needed. Do not duplicate full sprint goals or requirement evidence here; link to their owner files.

Task status values: `Backlog`, `Ready`, `In Progress`, `Review`, `Done`, `Done With Concerns`, `Blocked`, `Discarded`.

---

## TASK-000 Template

Status: Backlog

Requirement IDs:

- R-000

Role:

- Main Agent | Agent A | Agent B | Agent C | Agent D | Agent E | Agent T | Temporary Expert

Mission:

- One clear task objective.

Model Routing:

- Assigned Model: Runtime default | smaller/faster | default strong coding | strongest available | exact model if supported
- Reasoning Effort: low | medium | high | xhigh | runtime default
- Reason For Choice: TBD
- Escalation Trigger: TBD

Allowed Read Paths:

- `AGENTS.md`
- `codex-multi-agent-safe-collaboration.md`
- `docs/current-state.md`
- `docs/requirements-matrix.md`
- TBD

Allowed Write Paths:

- TBD or None

Required Skills:

- `skills/<name>/SKILL.md`

Path:

- Fast | Standard | Strict

Forbidden:

- Do not read unauthorized paths.
- Do not modify unauthorized paths.
- Do not execute `git add` / `git commit`.
- Do not perform unrelated refactors or formatting.

Inputs:

- Relevant decisions:
- Relevant open questions:
- Relevant acceptance criteria:

Steps:

1. TBD
2. TBD
3. TBD

Acceptance Criteria:

- TBD

Verification:

- Command/check: TBD
- Expected result: TBD

Risks:

- TBD

Requests:

- None

---

## Ready Queue

- None

## Blocked Queue

- None

## Done Queue

- None

# Context Checkpoints

Purpose: periodic anti-context-rot snapshots for long-running work. Add a checkpoint every 60-90 minutes, every 2-3 completed tasks, after interruption recovery, and before cross-session continuation.

---

## YYYY-MM-DD HH:mm Checkpoint Template

### Trigger

- Scheduled | Completed task batch | New session | Interrupted recovery | Before dispatch | Before completion

### Re-Read

- `AGENTS.md`
- `codex-multi-agent-safe-collaboration.md`
- Relevant `skills/<name>/SKILL.md`
- `docs/current-state.md`
- `docs/requirements-matrix.md`
- `docs/task-queue.md`
- `docs/decisions.md`
- `docs/open-questions.md`
- `docs/context-checkpoints.md`
- `docs/agent-work-summary.md` when needed

### Control Files Updated

- `docs/current-state.md`: yes/no, reason
- `docs/requirements-matrix.md`: yes/no, reason
- `docs/task-queue.md`: yes/no, reason
- `docs/decisions.md`: yes/no, reason
- `docs/open-questions.md`: yes/no, reason
- `docs/context-checkpoints.md`: yes/no, reason

### Still True

- TBD

### Drift Detected

- TBD or None

### Requirement Status Changes

- TBD or None

### Task Queue Changes

- TBD or None

### Verification

- Command/check: TBD
- Result: TBD

### Risks / Residual Concerns

- TBD or None

### Next 1-3 Tasks

1. TBD
2. TBD
3. TBD

---

## Checkpoint Log

No checkpoints yet.

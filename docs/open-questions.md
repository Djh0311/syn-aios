# Open Questions

Purpose: unresolved questions that agents must not answer by guessing. Each question needs a conservative default behavior until resolved.

Status values: `Open`, `Answered`, `Deferred`, `No Longer Relevant`.

| ID | Question | Blocks | Owner | Needed By | Conservative Default Until Answered | Status | Answer / Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Q-001 | TBD | TBD | User / External / Main Agent | TBD | TBD | Open | TBD |

## Update Rules

- Add questions as soon as they are discovered.
- If implementation can continue safely with a conservative default, write that default explicitly.
- If no safe default exists, mark dependent tasks `Blocked` in `docs/task-queue.md`.
- Move answered questions into `docs/decisions.md` when they become binding decisions.

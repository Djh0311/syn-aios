# Open Questions

> 资料状态（2026-08-09）：工程任务中的临时问题模板，不是产品候选池。产品和架构未决项只登记在 `docs/product/candidate-register-v1.md`；本文件只用于某个当前施工包发现且需要短期阻塞处理的问题。

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

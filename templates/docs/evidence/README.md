# Evidence Archive

Purpose: store durable proof for important claims, failures, UI states, diagnostics, and verification runs. Control files summarize conclusions; this archive keeps the evidence behind them.

Use this archive for Strict Path tasks, long-running work, important UI verification, bug root-cause evidence, multi-agent integration, failed fixes, production incidents, recovery, and any claim that may need cross-session recovery.

---

## Directory Pattern

```text
docs/evidence/
  YYYY-MM-DD-short-task-name/
    summary.md
    test-output.md
    browser-check.md
    console-network.md
    before-screenshot.png
    after-screenshot.png
    trace.zip
```

Use only the files that are relevant. Prefer summaries over dumping huge logs. Link to external CI, PR, trace, or artifact URLs when they are the source of truth.

---

## Summary Template

```markdown
# Evidence: Short Task Name

Date:
Task / Requirement:
Agent:
Path: Fast | Standard | Strict

## Claims Verified

- Claim:
  - Evidence:
  - Result:

## Commands Run

| Command | Result | Notes |
| --- | --- | --- |
| TBD | TBD | TBD |

## Browser / UI Checks

| Route | Viewport | Interaction | Console | Network | Evidence |
| --- | --- | --- | --- | --- | --- |
| TBD | TBD | TBD | TBD | TBD | TBD |

## Failures Or Gaps

- TBD

## Links

- Requirements:
- Mistake ledger:
- PR / CI:
```

---

## Recording Rules

- Store evidence for Strict Path tasks, important UI tasks, interrupted/recovered work, bug root cause, failed hypotheses, evaluator rejection, multi-agent integration, production incidents, and any non-trivial verification claim.
- Do not use evidence archive as a substitute for fresh verification. Fresh verification is still required before completion claims.
- If output is too large, summarize the key result and save or link only the diagnostic slice needed for recovery.
- When a mistake is recorded in `docs/agent-mistake-ledger.md`, link the relevant evidence here.
- When a requirements row is marked `Done`, its evidence column should point to fresh verification output, this archive, CI, or a reviewed summary.

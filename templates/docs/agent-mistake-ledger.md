# Agent Mistake Ledger

Purpose: durable memory for mandatory-record agent mistakes and other mistakes likely to recur or cause real damage. This file prevents the same important mistake from being made twice without turning small corrections into process noise.

Status values: `Open`, `Encoded In Test`, `Encoded In Skill`, `Encoded In Rule`, `Accepted Risk`, `Obsolete`.

Read this file before debugging retries, failed-fix recovery, Strict Path work where known mistakes may apply, fixing review feedback caused by agent error, and before finishing any task that involved a wrong turn likely to require prevention.

---

## Entry Template

```markdown
## M-0001: Short Mistake Title

Date:
Task / Requirement:
Affected Area:
Detected By:

### Symptom

- What was observed?

### Wrong Assumption

- What did the agent believe that was false?

### Wrong Action

- What incorrect change, claim, or diagnosis happened?

### Actual Root Cause

- What was actually true?

### Detection Evidence

- Command, screenshot, log, review finding, or user correction:

### Correct Fix

- What fixed the real issue?

### Regression Protection

- Test/check added:
- Evidence location:

### Prevention

- Skill/rule/checklist to update:
- New guardrail:

Status: Open
```

---

## Recording Rules

- Add an entry when an agent fixes the wrong bug, misidentifies root cause, changes a symptom instead of the source, introduces a regression, claims success without required verification, violates read/write scope, or repeats a known mistake.
- Add an entry when the user has to correct a factual assumption that materially affected implementation, debugging, scope, or completion claims.
- Add an entry for other mistakes when they are likely to recur or cause real damage.
- Do not add an entry for wording/style corrections, harmless formatting issues, immediately corrected minor misunderstandings, or exploratory dead ends that produced no wrong change and no wrong success claim unless the same pattern repeats.
- If an error is testable, add or update a regression test and set status to `Encoded In Test` after verification.
- If an error is caused by weak process, update the relevant `SKILL.md`, `AGENTS.md`, or task template and set status to `Encoded In Skill` or `Encoded In Rule` after the rule is changed.
- If the same mistake class appears twice, treat it as a process failure and update the relevant skill before continuing similar work.
- Link evidence to `docs/evidence/` when the mistake involved logs, screenshots, browser traces, command output, or before/after behavior.

---

## Active Mistakes

- None yet.

---

## Closed Mistakes

- None yet.

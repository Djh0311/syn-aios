---
name: learning-from-mistakes
description: Use after a wrong root cause, wrong fix, repeated failure, missed verification with a success claim, material user correction, regression, evaluator rejection, scope violation, or other agent mistake likely to recur or cause real damage
---

# Learning From Mistakes

## Rule

Mandatory-record agent mistakes must produce a durable prevention mechanism. A mistake that is only apologized for is not fixed.

Do not record every small correction. The ledger is for mandatory-record failures and other mistakes likely to recur or cause real damage.

## Required Workflow

1. Identify whether a mandatory-record mistake happened:
   - wrong root cause
   - wrong fix
   - repeated failed fix
   - regression introduced
   - missed material requirement
   - false completion claim
   - skipped verification while implying success
   - user correction of a material assumption
   - subagent scope violation
2. Record other mistakes when they are likely to recur or cause real damage.
3. Do not record minor issues unless they repeat:
   - wording/style correction
   - harmless formatting issue
   - immediately corrected small misunderstanding with no implementation impact
   - exploratory dead end that did not produce a wrong claim or wrong change
4. Add or update an entry in `docs/agent-mistake-ledger.md`.
5. Link evidence from `docs/evidence/`, command output, review comment, or user correction when available.
6. Decide the prevention mechanism:
   - regression test
   - stronger verification command
   - skill update
   - `AGENTS.md` rule update
   - task template update
   - explicit accepted risk
7. Implement the prevention mechanism when feasible before continuing similar work.
8. If the same mistake class appears twice, stop and update the relevant skill/rule before more implementation.
9. Report the ledger entry ID and prevention status.

## Ledger Quality Bar

Each entry must include:

- symptom
- wrong assumption
- wrong action
- actual root cause
- detection evidence
- correct fix
- regression protection
- prevention rule
- status

## Status Rules

- `Open`: mistake recorded, prevention not yet encoded.
- `Encoded In Test`: regression or guard test exists and has been verified.
- `Encoded In Skill`: relevant skill changed to prevent recurrence.
- `Encoded In Rule`: `AGENTS.md`, task package, or protocol changed.
- `Accepted Risk`: user accepted no further prevention.
- `Obsolete`: no longer relevant because the affected system/rule was removed.

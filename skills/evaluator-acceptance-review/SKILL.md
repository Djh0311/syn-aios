---
name: evaluator-acceptance-review
description: Use before declaring Strict Path, long-running, multi-agent, cross-module, production-facing, risky, or high-impact work complete; performs independent acceptance review against the sprint contract, requirements matrix, user goal, evidence, tests, browser checks, and known risks
---

# Evaluator Acceptance Review

## Rule

Implementation completion and acceptance are different claims. Do not accept work just because code changed or tests passed.

The evaluator checks whether the delivered behavior satisfies the user goal and acceptance criteria. It may reject work that passes tests.

## Required Inputs

- user goal or task summary
- `docs/sprint-contract.md` when active
- relevant rows from `docs/requirements-matrix.md`
- relevant decisions and open questions
- changed files summary
- verification evidence
- UI/browser evidence when user-facing UI changed
- known risks and unverified paths
- relevant mistake ledger entries

## Review Workflow

1. Restate the acceptance target in one paragraph.
2. Check each relevant requirement independently.
3. Compare delivered behavior against non-goals and forbidden scope.
4. Verify that evidence is fresh and matches the claim.
5. For UI work, require browser evidence from Chrome DevTools MCP or an equivalent browser harness.
6. Check whether any known mistake pattern applies.
7. Decide one status:
   - `ACCEPTED`
   - `ACCEPTED_WITH_CONCERNS`
   - `NEEDS_FIXES`
   - `REJECTED`
   - `NEEDS_MORE_EVIDENCE`
8. List blocking issues before summaries.

## Output Format

```markdown
STATUS: ACCEPTED | ACCEPTED_WITH_CONCERNS | NEEDS_FIXES | REJECTED | NEEDS_MORE_EVIDENCE

ACCEPTANCE:
- R-001:

EVIDENCE CHECKED:
- Tests:
- Build/type/lint:
- UI/browser:
- API/DB:

BLOCKERS:
- None, or specific requirement/evidence gaps.

CONCERNS:
- Residual risks or assumptions.

REQUIRED FOLLOW-UP:
- Tests, fixes, evidence, decisions, or mistake ledger updates.
```

## Rejection Conditions

Return `NEEDS_FIXES`, `REJECTED`, or `NEEDS_MORE_EVIDENCE` if:

- a requirement lacks evidence
- UI changed without browser verification
- tests do not cover the changed behavior and no reason is recorded
- implementation expanded into forbidden scope
- a user-visible path is broken
- root cause remains unproven for a bug fix
- a known mistake pattern was repeated without prevention

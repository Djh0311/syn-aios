---
name: requesting-code-review
description: Use when a fresh independent review materially reduces risk, especially for major implementation, complex fixes, Strict integration, multi-agent outputs, or pre-merge readiness
---

# Requesting Code Review

## Purpose

Use independent review to catch requirement, scope, correctness, and maintainability issues before integration or completion claims. Do not turn every small task into review ceremony.

## When To Use

Request review when any apply:

- major feature, complex bugfix, risky refactor, or shared abstraction
- Strict Path integration or multi-agent output
- public API, DB/schema, auth, permissions, security, payment, deployment, or data-integrity surface
- pre-merge or PR readiness where review is part of the project workflow
- repeated failures, uncertain root cause, or reviewer/evaluator rejection
- the main agent has low confidence that tests cover the changed behavior

Skip review for small, low-risk, well-verified Fast/Standard changes unless the user asks or a review would materially reduce risk.

| Skip Review | Require Review |
| --- | --- |
| Typo, copy, or documentation-only edit. | Major feature, complex bugfix, or risky refactor. |
| Non-behavioral config or formatting change. | Shared abstraction, public API, DB/schema, auth, security, payment, deployment, or data-integrity surface. |
| Small style/layout fix already browser-verified. | Strict Path integration, multi-agent output, or pre-merge readiness where review is required. |
| Single-file non-behavioral change with direct verification. | Repeated failure, prior wrong fix, uncertain coverage, or low confidence in tests. |

## Required Inputs

- what changed
- user goal, plan, sprint contract, or relevant requirements
- changed files or diff range
- verification already run and gaps
- known risks, assumptions, and scope boundaries
- specific review focus, if any

## Workflow

1. **Decide review scope**
   - State why review is needed.
   - Choose full change review, focused file review, architecture review, security review, UI review, or evaluator review.
   - Keep the reviewer read-only unless explicit write scope is authorized.

2. **Prepare review package**
   - Include task goal, accepted requirements, changed files, relevant evidence, and known concerns.
   - Include base/head commits when available, but do not require Git if the project is not a repo.
   - Include forbidden scope and decisions the reviewer must not re-open.

3. **Request review**
   - Use the available subagent/review mechanism or perform an equivalent independent review.
   - For subagents, provide an explicit task package with allowed read paths, allowed write paths `None`, required skills, and output format.
   - Use `requesting-code-review/code-reviewer.md` as a template when helpful.

4. **Process feedback**
   - Use `receiving-code-review` before acting on feedback.
   - Fix Critical issues before proceeding.
   - Fix Important issues unless explicitly deferred with reason.
   - Track Minor issues only when useful; do not expand scope automatically.

5. **Verify after changes**
   - Re-run relevant checks after review-driven fixes.
   - Use `verification-before-completion` before claiming review is resolved or work is ready.

## Output Expected From Reviewer

```markdown
STATUS: APPROVED | APPROVED_WITH_CONCERNS | NEEDS_FIXES | NEEDS_MORE_EVIDENCE

SCOPE CHECK:
- In scope:
- Out of scope:

FINDINGS:
- Critical:
- Important:
- Minor:

EVIDENCE REVIEWED:
- Tests/checks:
- Files:

RISKS:
- Remaining risks or assumptions.
```

## Stop Conditions

Stop and ask or re-route when:

- review scope is unclear
- reviewer needs files outside allowed read scope
- reviewer wants to modify files without write authorization
- feedback conflicts with user decisions or requirements
- review reveals a Strict Path risk not previously routed
- review identifies a meaningful agent mistake

## Related Skills

- `receiving-code-review` before implementing review feedback.
- `verification-before-completion` before claiming review is resolved.
- `evaluator-acceptance-review` for Strict Path acceptance, which is stronger than ordinary code review.

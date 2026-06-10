---
name: receiving-code-review
description: Use before acting on user or reviewer feedback that changes code, tests, architecture, scope, or completion status; requires technical verification before implementation
---

# Receiving Code Review

## Purpose

Prevent blind implementation, performative agreement, and defensive pushback. Review feedback is input to evaluate against the codebase, requirements, user decisions, and verification evidence.

## When To Use

Use this skill when feedback asks for or implies changes to:

- code, tests, docs that affect behavior, architecture, or scope
- requirements, acceptance criteria, or completion status
- review comments from the user, a teammate, an external reviewer, CI, evaluator, or subagent

Do not use it for harmless typo/style feedback that can be applied directly inside the current write scope.

## Core Rule

Understand and verify feedback before implementing it.

The user is authoritative on intent and priority. External reviewers are not authoritative by default; their feedback must be checked against this codebase and the user's decisions.

## Workflow

1. **Read the full feedback**
   - Identify each requested change.
   - Separate blockers from suggestions, questions, and optional improvements.
   - Note whether the feedback changes scope or acceptance.

2. **Clarify before partial implementation**
   - If any blocking item is unclear, ask before editing.
   - If only non-blocking items are unclear, state the safe subset and the deferred questions.
   - Do not guess requirements, architecture, or data contracts.

3. **Verify against project reality**
   - Read the affected code/tests/docs before changing them.
   - Check whether the suggestion is technically correct for this stack, supported platforms, existing behavior, and current decisions.
   - For "make it proper" or expanded-feature feedback, check actual usage and YAGNI risk.

4. **Decide response**
   - If feedback is correct and in scope, implement the smallest appropriate change.
   - If feedback is correct but expands scope, stop and state the new scope before proceeding.
   - If feedback conflicts with user decisions or requirements, ask the user.
   - If feedback appears wrong, push back with concise technical evidence.

5. **Implement carefully**
   - Keep changes inside declared write scope.
   - For multi-item feedback, handle blocking issues first, then simple fixes, then complex refactors.
   - Test each meaningful change or run the relevant grouped verification.
   - Avoid unrelated cleanup.

6. **Verify and report**
   - Use `verification-before-completion` before claiming feedback is resolved.
   - For UI-visible feedback, use `ui-browser-verification`.
   - If the feedback exposed an agent mistake, use `learning-from-mistakes`.

## Pushback Rules

Push back when the suggestion:

- breaks existing behavior
- conflicts with confirmed user or architecture decisions
- adds unused or speculative functionality
- assumes a platform/version/tooling constraint that is false
- lacks enough information to implement safely
- would require scope outside the current write boundary

Pushback should cite code, tests, logs, docs, requirements, or concrete uncertainty. Keep it technical and brief.

## Response Style

Prefer factual responses:

- "Implemented X; verified with Y."
- "I checked X. This conflicts with Y, so I need a decision before changing it."
- "This appears out of scope because X is unused; should I remove it or keep it?"

Avoid performative agreement before verification. Do not use agreement as a substitute for checking the code.

## Stop Conditions

Stop and ask or re-route when:

- feedback is unclear and could affect implementation direction
- feedback expands read/write scope
- feedback conflicts with a user decision, sprint contract, or requirement
- suggested changes touch Strict Path risk such as security, payments, migrations, deployment, or data integrity
- verification cannot be run but resolution depends on it
- applying one item may invalidate another item
- the feedback reveals a meaningful agent mistake

## Checklist

- [ ] Feedback items are understood or clarification was requested.
- [ ] Read/write scope is still valid.
- [ ] Affected code/tests/docs were checked before edits.
- [ ] Incorrect or out-of-scope feedback was challenged with evidence.
- [ ] Correct feedback was implemented with minimal scoped changes.
- [ ] Relevant tests, checks, browser verification, or evidence were run.
- [ ] Mistake ledger was updated when the feedback exposed a meaningful agent error.

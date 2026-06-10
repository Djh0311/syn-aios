---
name: test-driven-development
description: Use when implementing changed business behavior, bugfix regression protection, public/API behavior, state flow, data transformation, permission/security/payment logic, or risky refactor before writing implementation code
---

# Test-Driven Development

## Purpose

Use TDD when the risk router says behavior must be protected by an executable expectation before implementation.

Core rule for TDD-required work: write a failing test first, verify it fails for the expected reason, implement the smallest fix, then verify it passes.

If a task is in a TDD-required category, verify-after is not a substitute for test-first unless it matches an allowed exception below or the user explicitly approves the skip.

## When To Use

Required for:

- New or changed business behavior.
- Bug fixes that need regression protection.
- Public/API behavior.
- State flow or data transformation.
- Permission, security, payment, or data-integrity logic.
- Risky refactors where behavior must be preserved.

Verify-after is allowed instead of test-first only for:

- Copy/text-only changes.
- Styling or small layout-only UI changes.
- Low-risk configuration edits that do not change runtime behavior, external contracts, security/data/deployment behavior, or cross-file assumptions.
- Mechanical renames or formatting.
- Throwaway prototypes or generated code.
- Cases where no automated test is feasible, with the reason recorded before implementation.

If unsure, treat the work as TDD-required and start with a failing test.

## Workflow

1. State the behavior to protect in one sentence.
2. Write the smallest test that expresses that behavior.
3. Run the focused test and confirm it fails for the expected reason.
4. If the test passes immediately, rewrite the test or re-check whether TDD applies.
5. If the test errors for setup/typo reasons, fix the test until it fails for the behavior reason.
6. Implement the smallest production change that can pass the test.
7. Run the focused test and confirm it passes.
8. Run the relevant surrounding tests/checks.
9. Refactor only while tests stay green.

## Test Quality

Good TDD tests:

- test one behavior
- have a clear name
- exercise real code where feasible
- assert observable behavior, not implementation details
- avoid mocks unless the boundary genuinely requires one

When adding mocks or test utilities, read `testing-anti-patterns.md` if mock behavior could replace real behavior.

## Stop Conditions

Stop and ask or re-route when:

- You cannot state the behavior being protected.
- You do not know how to observe the behavior.
- The test would require broad architecture changes before it can run.
- The only available test would assert mock behavior instead of real behavior.
- No automated test is feasible; record why and use the strongest available verification.
- The needed change touches Strict Path risks not covered by the current scope.

## If Code Was Written First

For TDD-required work, code written before the failing test cannot be treated as TDD-compliant.

Safe recovery options:

- discard or set aside the implementation and restart from the failing test
- keep the code only as temporary investigation notes, then implement fresh from the test
- ask the user to explicitly approve verify-after instead of TDD only when an allowed exception does not apply

Do not quietly convert tests-after into a TDD claim.

## Debugging Integration

For a bug fix:

1. Use `systematic-debugging` to find the root cause.
2. Add the smallest failing regression test when feasible.
3. Implement the fix.
4. Verify the regression test and relevant surrounding checks.

## Verification Checklist

Before marking TDD-required work complete:

- [ ] Behavior was stated before implementation.
- [ ] Test was written before production code.
- [ ] Test failed for the expected behavior reason.
- [ ] Production change was minimal.
- [ ] Focused test passed after the change.
- [ ] Relevant surrounding tests/checks ran or blockers were reported.
- [ ] Any verify-after exception was explicitly recorded.

## Completion Wording

Use precise wording:

- "TDD complete for [behavior]; focused test failed then passed."
- "Implemented with verify-after because [reason]; verification run: [check]."
- "Modified but not verified; remaining check: [check]."

Do not claim TDD if the test did not fail first.

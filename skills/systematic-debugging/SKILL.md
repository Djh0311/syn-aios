---
name: systematic-debugging
description: Use when encountering bugs, failing tests/builds, runtime errors, unexpected behavior, production issues, or failed prior fixes before proposing fixes
---

# Systematic Debugging

## Purpose

Prevent guess-and-patch behavior by proving what is failing, where it fails, and why it fails before changing production code.

## When To Use

Use this skill for:

- bug reports
- failing tests, builds, type checks, lint, CI, or runtime errors
- crashes, warnings, regressions, or unexpected UI/API/data behavior
- production or customer-observed issues
- any prior fix attempt that did not resolve the issue

## Core Rule

No fix before root-cause investigation.

Before editing production code, you must have a stated symptom, reproduction or evidence, relevant errors reviewed, likely boundary checked, one explicit hypothesis, and a minimal way to test it.

## Workflow

1. **Define symptom**
   - State the observed failure and the expected behavior.
   - Identify the affected command, route, screen, API, job, test, or environment.
   - Separate confirmed facts from assumptions.

2. **Capture evidence**
   - Reproduce locally when possible.
   - If reproduction is not possible, collect logs, screenshots, traces, failing command output, request/response data, or user-provided evidence.
   - For intermittent failures, capture timing, environment, inputs, and frequency instead of guessing.

3. **Read errors**
   - Read stack traces, assertion diffs, warnings, console errors, network failures, and build output closely.
   - Note file paths, line numbers, error codes, failing inputs, and first failure point.
   - Do not rely on the last or loudest error if an earlier error explains the cause.

4. **Check context**
   - Inspect recent edits, diffs, dependency/config changes, data changes, and environment differences.
   - Read nearby code before changing it.
   - Check `docs/agent-mistake-ledger.md` for prior failed fixes, repeated failures, material corrections, or similar mistakes.
   - Check `docs/tooling-and-mcp-registry.md` when the right verification/debugging tool is not obvious.

5. **Trace boundary**
   - For multi-component issues, identify the boundary where correct input becomes incorrect output.
   - Inspect data entering and leaving each relevant layer.
   - Use temporary diagnostics only when they are the smallest way to locate the failing boundary.
   - For deep call stacks or bad values, use `root-cause-tracing.md`.

6. **Compare pattern**
   - Find similar working code, tests, config, or prior evidence in the same codebase.
   - Compare the broken path with the working path.
   - List meaningful differences before deciding which one matters.

7. **State hypothesis**
   - Write: "I think the root cause is X because Y."
   - Keep one hypothesis active at a time.
   - If evidence does not support it, discard or revise it before editing.

8. **Test minimally**
   - Use the smallest diagnostic, failing test, isolated command, or browser check that can confirm or reject the hypothesis.
   - Change one variable at a time.
   - Do not stack fixes on top of an unproven theory.

9. **Fix cause**
   - Add a failing regression test first when feasible or required by `test-driven-development`.
   - Make the smallest production change that addresses the proven cause.
   - Avoid unrelated refactors, opportunistic cleanup, or broad rewrites.
   - If defense at multiple layers is needed after root cause is known, use `defense-in-depth.md`.

10. **Verify**
    - Re-run the failing test, command, scenario, or browser path.
    - Run adjacent regression checks appropriate to the blast radius.
    - For timing or async issues, prefer `condition-based-waiting.md` over fixed sleeps.
    - Store durable evidence when required by the selected path, the task contract, or repeated-failure risk.

## UI And Browser Bugs

For frontend, layout, responsive, browser interaction, console, network, or visual failures, reproduce in a real browser harness when available. Record route, viewport, steps, console/network status, and screenshots when useful. Use `ui-browser-verification` before claiming the fix works; passing unit tests do not prove visual or interaction correctness.

## Failed Fixes

If a fix attempt fails, stop editing, record what was tried and what evidence changed, then return to root-cause investigation.

If two attempts have failed, tighten scope and re-check assumptions before a third. If three attempts have failed, stop and discuss fundamentals with the user before continuing; the boundary, architecture, requirement, or diagnosis is probably wrong.

## Mistake Ledger

Use `learning-from-mistakes` and update `docs/agent-mistake-ledger.md` when the agent claimed or acted on a wrong root cause, made a wrong fix, repeated a failed attempt pattern, skipped or misrepresented verification, introduced a regression, violated read/write scope, or was corrected by the user on a material assumption.

Do not log minor wording corrections or harmless clarifications unless they reveal a repeatable failure mode.

## Stop Conditions

Stop and ask, re-route, or gather more evidence when:

- the symptom cannot be stated clearly
- reproduction and evidence are missing
- the fix would require guessing about requirements or data contracts
- the issue touches Strict Path risk such as security, payments, migrations, deployment, or data integrity
- needed verification tools are unavailable and the completion claim depends on them
- current changes are mixed with unrelated user edits
- repeated attempts suggest the architecture or requirement may be wrong

## Verification Checklist

Before reporting a bug fixed:

- [ ] Symptom, expected behavior, and root cause are stated with evidence.
- [ ] Fix addresses the cause, not only the symptom.
- [ ] Regression test exists when feasible or required.
- [ ] Original failing scenario and adjacent checks were re-run fresh.
- [ ] UI/browser fixes were verified in a browser harness when relevant.
- [ ] Durable evidence and mistake-ledger updates were made when required.

## Related Skills And References

Read only when relevant:

- `test-driven-development` for writing failing regression tests before code changes.
- `verification-before-completion` before claiming fixed, passing, done, or ready.
- `ui-browser-verification` for browser-visible behavior.
- `learning-from-mistakes` after meaningful agent errors.
- `root-cause-tracing.md` for backward tracing through call stacks and data flow.
- `defense-in-depth.md` for layered validation after root cause is known.
- `condition-based-waiting.md` for replacing sleeps with condition-based waits.

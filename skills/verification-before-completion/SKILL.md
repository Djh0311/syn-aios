---
name: verification-before-completion
description: Use before claiming work is complete, fixed, passing, ready, accepted, or before closing implementation work; requires fresh verification evidence before success claims
---

# Verification Before Completion

## Purpose

Prevent false completion claims. A status claim is allowed only when fresh evidence directly supports that claim.

## When To Use

Use this skill before saying or implying:

- complete, done, fixed, passing, ready, accepted, verified, works
- tests pass, build passes, lint passes, UI works, bug is fixed
- a task, branch, PR, phase, agent task, or review is finished

Use it before commit/PR/merge readiness statements and before closing Standard or Strict work.

## Core Rule

No completion claim without fresh verification evidence.

If verification was not run in the current work phase, say what changed and what remains unverified. Do not convert confidence, reasoning, prior runs, or subagent reports into success claims.

## Workflow

1. **Identify the claim**
   - What exact status are you about to report?
   - Is it about tests, build, lint, behavior, UI, requirements, integration, or acceptance?

2. **Identify required evidence**
   - Choose the command, browser harness, MCP/tool check, log, API/DB query, review result, or evaluator result that directly proves the claim.
   - Use `docs/tooling-and-mcp-registry.md` when tool choice is not obvious.

3. **Run or inspect fresh evidence**
   - Run the full relevant command or tool check when available.
   - Read the output, exit code, failure count, console/network status, screenshot/DOM/trace, or review finding.
   - For delegated work, inspect changed files and verify independently before relying on the report.

4. **Compare evidence to claim**
   - If evidence supports the claim, report the claim with the evidence.
   - If evidence is partial, report the partial status and the unverified gaps.
   - If evidence contradicts the claim, report the actual failing state and continue the appropriate workflow.

5. **Record durable evidence when required**
   - Store or summarize evidence for Strict Path, long-running work, interrupted/recovered work, important UI, non-trivial bug fixes, production-observed issues, failed hypotheses, multi-agent integration, evaluator review, or anything likely to need cross-session recovery.

## Evidence By Claim

| Claim | Required Evidence |
| --- | --- |
| Tests pass | Fresh test command output with exit status and failure count. |
| Build/type/lint passes | Fresh command output from the relevant check. |
| Bug fixed | Original failing scenario re-run, plus regression check when feasible. |
| Regression test protects fix | Red-green verification when TDD requires it, or documented allowed verify-after exception. |
| Requirement met | Relevant acceptance criteria checked against evidence. |
| UI/browser behavior works | Real browser harness evidence for route, viewport, interaction, console, and network when relevant. |
| Agent task complete | Changed files inspected, scope checked, and verification run by main agent or trusted harness. |
| Strict work accepted | `evaluator-acceptance-review` result with evidence. |

## Path Handling

- **Fast Path:** inspect the changed line or diff, confirm the write scope stayed narrow, run the smallest direct check when one exists, or state that file/diff inspection is the available evidence. Do not imply tests passed when no test ran.
- **Standard Path:** run relevant tests/tool checks; UI changes require `ui-browser-verification`; update only affected docs/control files.
- **Strict Path:** require durable evidence, affected control-file updates, and evaluator acceptance before final completion.

## Required Cross-Checks

- Use `ui-browser-verification` before claiming frontend, visual, layout, responsive, browser interaction, or web app behavior is complete or fixed.
- Use `evaluator-acceptance-review` before completing Strict Path, long-running, multi-agent, cross-module, risky, high-impact, or production-facing work.
- Use `learning-from-mistakes` when a false success claim, missed verification, wrong fix, regression, material correction, repeated failure, or scope violation occurred.
- Ask before `git add` or `git commit`; verification does not grant commit permission.

## Stop Conditions

Stop and report the real state when:

- no command or tool can prove the claim
- verification cannot run and the claim depends on it
- output was not read or is ambiguous
- tests/build/lint/browser checks fail
- UI was not opened in a real browser harness when UI behavior is claimed
- a subagent report is the only evidence
- affected requirements or control files are stale for Standard/Strict work
- Strict Path acceptance has not been independently reviewed

## Reporting

Use precise wording:

```markdown
Verification:
- Claim:
- Evidence:
- Result:
- Unverified:
- Residual risk:
```

If verification did not run:

```markdown
Changed:
- ...

Not verified:
- Reason:
- Suggested check:
```

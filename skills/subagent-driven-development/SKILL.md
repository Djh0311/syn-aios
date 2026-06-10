---
name: subagent-driven-development
description: Use for main-agent-approved implementation dispatch inside the multi-agent protocol when tasks have explicit read/write scope and can be executed by fresh subagents in the current session
---

# Subagent-Driven Development

## Purpose

Execute approved multi-agent implementation work without losing scope control. The main agent remains responsible for task packaging, dispatch, review, integration, verification, recovery, and final completion.

This skill is an execution wrapper for `codex-multi-agent-safe-collaboration.md`; the protocol is the source of truth for roles, task package fields, recovery, evidence, and final verification.

## When To Use

Use this skill when all are true:

- the risk router has selected Strict Path or the user explicitly approved multi-agent work
- there is a written plan, sprint contract, or dispatch-ready task package
- work can be split into bounded tasks with disjoint or carefully sequenced write scopes
- the main agent can review and integrate every result in the current session

Do not use it for tightly coupled work, vague exploration, simple single-agent tasks, or any task without explicit read/write scope.

## Core Rules

- Every subagent task must follow the required task package from `codex-multi-agent-safe-collaboration.md`.
- Subagents may read/write only authorized paths plus read-only protocol files.
- Subagents must return `NEEDS_DECISION` before expanding scope.
- Subagents must not run `git add` or `git commit`.
- Subagent `DONE` means local completion only; the main agent owns final completion claims.
- Main agent must independently inspect changed files, scope compliance, evidence, risks, and conflicts.

## Workflow

1. **Confirm activation**
   - Re-read `AGENTS.md`, `using-superpowers`, and `codex-multi-agent-safe-collaboration.md`.
   - Confirm Strict Path or explicit multi-agent authorization.
   - Confirm recovery is complete if the work was interrupted.

2. **Prepare task packages**
   - Split work into bounded tasks with one clear mission each.
   - Assign role, model routing, allowed read paths, allowed write paths, required skills, forbidden actions, inputs, verification, and output format.
   - Make write scopes disjoint when tasks run concurrently; otherwise run tasks serially.

3. **Dispatch fresh subagents**
   - Give each subagent the complete task package.
   - Prefer read-only reconnaissance before implementation when contracts or boundaries are uncertain.
   - Do not ask subagents to infer missing scope from chat history.

4. **Handle questions and blockers**
   - If a subagent returns `NEEDS_CONTEXT`, provide only authorized context.
   - If it returns `NEEDS_DECISION` or `BLOCKED`, main agent decides whether to ask the user, redraw scope, or stop.
   - Do not let subagents self-authorize broader reads/writes.

5. **Review each result**
   - Check `CHANGED_FILES` against allowed write paths.
   - Inspect diffs or changed files directly.
   - Verify evidence and rerun important checks when needed.
   - Reject or rework out-of-scope changes instead of silently integrating them.

6. **Integrate serially**
   - Resolve conflicts under main-agent ownership.
   - Run target and regression verification across integrated changes.
   - Use `requesting-code-review` when a fresh review materially reduces risk.
   - Use `evaluator-acceptance-review` before final Strict Path completion.

7. **Record state**
   - Update affected owner files only: task queue, requirements, current state, decisions, open questions, checkpoints, evidence, mistake ledger, and agent summary as applicable.
   - Do not claim final completion until `verification-before-completion` passes.

## Stop Conditions

Stop dispatch or integration when:

- a task package lacks read/write scope
- two tasks need the same write path without sequencing
- a subagent reads or modifies unauthorized paths
- a subagent output lacks changed files, evidence, or risks
- requirements, contracts, or ownership boundaries are unclear
- verification tools are unavailable for a claim that depends on them
- recovery is required but incomplete
- a mandatory-record mistake or scope violation needs ledger handling

## Related Files

- `codex-multi-agent-safe-collaboration.md` for the required task package, roles, recovery, and final verification checklist.
- `skills/dispatching-parallel-agents/SKILL.md` for independent parallel investigations inside the protocol.
- `skills/requesting-code-review/SKILL.md` for review dispatch when needed.
- `skills/verification-before-completion/SKILL.md` before any completion claim.

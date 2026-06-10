---
name: dispatching-parallel-agents
description: Use for main-agent-approved parallel subagent dispatch inside the multi-agent protocol when two or more tasks are independent, bounded, and have non-overlapping or read-only scopes
---

# Dispatching Parallel Agents

## Purpose

Use parallelism only when it reduces wall-clock time without weakening scope control, root-cause quality, or integration safety. Parallel dispatch is a multi-agent protocol pattern, not a shortcut around planning.

## When To Use

Use this skill when all are true:

- `codex-multi-agent-safe-collaboration.md` is active
- two or more tasks can proceed without waiting on each other
- each task has a complete task package
- allowed write paths are disjoint, or tasks are read-only
- each task has its own verification/evidence target
- the main agent can review and integrate all results

Common fit:

- independent read-only investigations
- separate test failures with different likely domains
- frontend/backend/data impact surveys before a contract freeze
- independent implementation tasks with disjoint files and stable contracts

Do not use parallel dispatch just because a task is large.

## Independence Check

Before dispatching in parallel, confirm:

- one task's result is not needed to define another task
- tasks do not edit the same file, generated artifact, lockfile, migration, shared type, or public contract
- tasks do not require exclusive access to the same database, server, port, queue, fixture, or external account
- failure in one task will not make another task's work invalid
- integration order is known

If any answer is uncertain, run read-only reconnaissance first or sequence the tasks.

## Workflow

1. **Group work by domain**
   - Separate independent problem areas or ownership slices.
   - Keep related failures together when one fix may affect all of them.

2. **Prepare one task package per agent**
   - Include mission, role, model routing, allowed read paths, allowed write paths, required skills, forbidden actions, inputs, verification, output format, and escalation trigger.
   - Mark read-only tasks with `Allowed Write Paths: None`.
   - State that scope expansion requires `NEEDS_DECISION`.

3. **Dispatch together**
   - Start only the tasks that passed the independence check.
   - Do not dispatch parallel implementation agents to overlapping write scopes.
   - Do not dispatch if recovery is required but incomplete.

4. **Collect results before integration**
   - Wait for enough results to detect conflicts.
   - Review `STATUS`, `CHANGED_FILES`, evidence, risks, and requests.
   - Treat missing evidence or unauthorized scope as a failed task, not a successful result.

5. **Integrate under main-agent ownership**
   - Inspect changed files directly.
   - Resolve conflicts serially.
   - Run combined verification across integrated changes.
   - Update affected owner files and evidence when required.

## Stop Conditions

Stop and switch to serial work when:

- task independence is uncertain
- write scopes overlap
- public contracts, migrations, lockfiles, generated artifacts, or shared types would be edited by more than one task
- tasks share mutable runtime state that cannot be isolated
- a subagent returns `NEEDS_DECISION`, `BLOCKED`, unclear root cause, or conflicting evidence
- the main agent cannot review outputs before they are integrated
- verification cannot prove the combined result

## Checklist

- [ ] Multi-agent protocol is active.
- [ ] Each task has a complete task package.
- [ ] Read/write scopes are explicit.
- [ ] Parallel tasks are independent.
- [ ] Write scopes are disjoint or read-only.
- [ ] Verification target is defined per task and for the integrated result.
- [ ] Main agent will inspect changes and evidence before completion.

## Related Files

- `codex-multi-agent-safe-collaboration.md` for required task package and integration rules.
- `skills/subagent-driven-development/SKILL.md` for current-session implementation dispatch.
- `skills/systematic-debugging/SKILL.md` when parallel work investigates failures.
- `skills/verification-before-completion/SKILL.md` before any completion claim.

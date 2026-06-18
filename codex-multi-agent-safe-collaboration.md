# Codex Multi-Agent Safe Collaboration Protocol

> ⚠️ **已废弃（2026-06-19）**：本协议（科学家代号复核线 / 咨询审实物 / Strict Path 重型流程）是旧重型治理的源头，已被 `AGENTS.md` 精简版 v2 取代。轻档不用、**勿据此重启重流程**；保留仅作历史参考。乙·自动连环阶段若要重型协作护栏，再另议。

Use this protocol for any multi-agent work. Also use it for Strict Path work that is large, risky, cross-module, parallelizable, or long-running.

Default to a single main agent plus tools. Multi-agent work is optional and should be used only when the parallel benefit is clearly larger than the coordination cost. Most tasks are safer as one agent with focused tool calls; the usual multi-agent exception is bounded read-only reconnaissance, independent evaluator review, or truly disjoint write scopes that the main agent can integrate and verify.

The goal is safe, stable, bounded, auditable progress. The main agent owns planning, dispatch, review, integration, verification, recovery, and final completion. Subagents operate only within explicit task-package boundaries.

In the standard rule source package, runtime document shapes live under `templates/docs/`. In an installed project, the active runtime files live under project-owned `docs/**`. Do not treat `templates/docs/**` as current project state, and do not overwrite active `docs/**` files during rule updates unless explicitly authorized.

For work on this standard rule source package itself, source-package plans and migration notes belong under repo-root `plans/**`, not `docs/**`.

---

## Activation And Precedence

- `AGENTS.md` and `skills/using-superpowers/SKILL.md` decide whether Fast, Standard, or Strict Path is active and whether the multi-agent protocol is required.
- Once multi-agent is active, this document governs roles, scope boundaries, dispatch, review, integration, summaries, recovery, and long-running context protection.
- The Agent A-E roles below are available role labels, not a recommended default workflow. Do not dispatch them reflexively; dispatch only the smallest number of subagents needed for a bounded, parallelizable job.
- Relevant `skills/<name>/SKILL.md` still governs how each role performs its work.
- Protocol read files (`AGENTS.md`, this file, `./skills/**`, and the explicit control/audit files) are always readable as read-only protocol context.
- Explicit control files are `docs/current-state.md`, `docs/requirements-matrix.md`, `docs/task-queue.md`, `docs/decisions.md`, `docs/open-questions.md`, `docs/context-checkpoints.md`, and `docs/sprint-contract.md`.
- Explicit audit/harness files are `docs/agent-mistake-ledger.md`, `docs/tooling-and-mcp-registry.md`, and `docs/evidence/**`.
- `docs/plans/**`, `docs/platform/**`, `docs/agent-work-summary.md`, and any other `docs/**` files are not default control files. Read them only when the task package, user request, or recovery need explicitly authorizes them. `docs/agent-work-summary.md` may be read when historical evidence is needed.
- Any `git add` / `git commit` must be done only by the main agent, only at the end of a turn or execution phase, and only after explicit user confirmation.

---

## Multi-Agent Decision Gate

Before dispatching any subagent, the main agent must write down the reason multi-agent work is justified. If the reason is weak, keep the task single-agent and use normal tools.

Use a subagent only when at least one of these is true:

- Bounded read-only reconnaissance can run in parallel while the main agent continues non-overlapping work.
- Independent evaluator review materially reduces risk before a Strict Path completion claim.
- Implementation can be split into truly disjoint write scopes with low integration ambiguity.
- A specialized temporary expert can answer a narrow question faster than expanding the main agent's read scope.

Do not use a subagent when:

- The next main-agent step is blocked on that exact answer.
- The work is mostly sequential debugging, integration, or product judgment.
- The split would create shared-file coordination, contract churn, or merge ambiguity.
- The only reason is that the role labels below exist.

Record the chosen mode in the task package or status note:

```markdown
Execution Mode: Single agent | Multi-agent exception
Multi-Agent Justification:
Coordination Cost:
Fallback If Coordination Fails:
```

---

## Exception Path Role Labels

- Main agent: product judgment, task decomposition, dispatch, context distribution, conflict management, integration, final verification, and user communication.
- Agent A - Architecture and boundaries: architecture investigation, module boundaries, dependency risks, and architecture review. No business implementation by default.
- Agent B - Data / API / DB: schema, migrations, API contracts, services, adapter contracts, persistence boundaries.
- Agent C - Frontend / UX: UI, state, interaction, frontend tests, and frontend DTO consumption.
- Agent D - Testing and verification: test strategy, acceptance criteria, regression scope, verification evidence. Does not write test code unless explicitly authorized.
- Agent E - Evaluator: independent acceptance review against user goal, sprint contract, requirements matrix, evidence, UI/browser checks, risks, and mistake ledger. Read-only by default.
- Agent T - Test implementation: optional role for writing failing tests, regression tests, and test helpers under explicit write scope.
- Temporary experts: security, performance, dependency, documentation, debugging, migration, or domain experts. Default read-only unless explicitly authorized.

These labels are only vocabulary for exception-path dispatch. They are not a phase checklist and not evidence that parallel work is warranted.

---

## Core Rules

- Subagents may read and write only paths explicitly listed in their task package, except read-only protocol files.
- If a subagent needs scope outside its task package, it must return `NEEDS_DECISION`; it must not expand scope on its own.
- If a subagent actually reads or modifies unauthorized paths, the main agent must mark that task as failed and reject direct integration.
- Subagents may not execute `git add` or `git commit`.
- Public config, dependency manifests, lockfiles, migrations, shared types, generated artifacts, and cross-cutting contracts are high-risk and require explicit authorization.
- Agent C may not modify backend contracts unless explicitly authorized. Agent B may not modify frontend interaction unless explicitly authorized.
- Agent E may not implement fixes for the same acceptance decision it evaluates unless explicitly re-dispatched after its review is complete.
- Architecture recommendation and business implementation should not be owned by the same subagent for the same decision.
- Final completion can only be announced by the main agent. Subagent `DONE` means local completion only.

---

## Model Routing Policy

Use model routing only when the agent runtime supports subagent model or reasoning-effort overrides. If model overrides are unavailable, keep the default model and record the intended routing in the task package.

The main agent owns model selection and remains responsible for integration, verification, and final correctness. A stronger subagent model does not reduce main-agent review obligations. A cheaper subagent model may execute bounded work, but high-risk judgment must be reviewed by the main agent or a strongest-available evaluator.

### Routing Defaults

| Task Type | Suggested Model Class | Suggested Reasoning | Notes |
| --- | --- | --- | --- |
| File search, inventory, simple summarization, log slicing | Smaller/faster model | Low/medium | Read-only, low ambiguity, bounded output. |
| Mechanical edits, formatting, copy updates, simple tests | Smaller or default coding model | Medium | Use only with tight write scope and clear verification. |
| Normal feature implementation or bugfix | Default strong coding model | Medium/high | Prefer existing project default unless risk is clearly low or high. |
| Cross-module implementation, API/DB contracts, migrations, auth, permissions, payments, data integrity | Strongest available model | High/xhigh | High blast radius and contract risk. |
| Systematic debugging after failed fixes, unclear root cause, flaky tests, production incidents | Strongest available model | High/xhigh | Root-cause reasoning matters more than speed. |
| Architecture review, security review, evaluator acceptance, release readiness | Strongest available model | High/xhigh | Independent judgment quality is the priority. |
| UI/browser verification with Chrome DevTools MCP or equivalent | Default or strong model plus browser harness | Medium/high | Tool evidence is mandatory; model choice does not replace browser checks. |

### Escalation Triggers

Escalate a subagent to a stronger model, or re-dispatch/review with a stronger model, when:

- the subagent reports `NEEDS_DECISION`, `BLOCKED`, unclear root cause, or conflicting evidence
- the task touches public contracts, database state, auth, permissions, security, payment, migration, or generated artifacts
- the task affects multiple modules or shared abstractions
- two or more fix attempts failed
- the user corrected a material assumption
- evaluator review rejects or requests more evidence
- the lower-cost model produced broad, speculative, or out-of-scope changes

### Routing Fields

Every subagent task package should include model routing fields. If the runtime cannot enforce them, write `Runtime default` and continue with the normal task boundaries.

```markdown
## Model Routing
Assigned Model: Runtime default | smaller/faster | default strong coding | strongest available | exact model if supported
Reasoning Effort: low | medium | high | xhigh | runtime default
Reason For Choice:
Escalation Trigger:
```

---

## Required Task Package

Every subagent dispatch must include:

```markdown
## Mission
One clear objective for this task only.

## Execution Mode
Multi-agent exception.

## Multi-Agent Justification
Why this subagent is worth the coordination cost, and why the task is not better handled by the main agent plus tools.

## Role
Agent A | Agent B | Agent C | Agent D | Agent E | Agent T | Temporary Expert

## Model Routing
Assigned Model: Runtime default | smaller/faster | default strong coding | strongest available | exact model if supported
Reasoning Effort: low | medium | high | xhigh | runtime default
Reason For Choice:
Escalation Trigger:

## Allowed Read Paths
- Exact files or directories the agent may read.

## Allowed Write Paths
- Exact files or directories the agent may modify, or None.

## Required Skills
- `skills/<name>/SKILL.md`, or None.
- If a required skill needs files outside allowed scope, return NEEDS_DECISION.

## Forbidden
- Do not read unauthorized paths.
- Do not modify unauthorized paths.
- Do not modify lockfiles, global config, build config, migrations, shared types, or generated files unless explicitly authorized.
- Do not execute `git add` / `git commit`.
- Do not perform unrelated refactors, formatting, dependency upgrades, or architecture changes.

## Inputs
- User goal summary.
- Current phase plan.
- Approved contracts or decisions.
- Relevant acceptance criteria.
- Required verification commands.
- Required MCP/tool evidence from `docs/tooling-and-mcp-registry.md` when relevant.
- Mistake ledger entries or known failure patterns when relevant.

## Required Output
STATUS: DONE | DONE_WITH_CONCERNS | NEEDS_CONTEXT | NEEDS_DECISION | BLOCKED

CHANGED_FILES:
- List actual changed files, or None.

SUMMARY:
- Short factual summary.

EVIDENCE:
- Commands/checks run and result summary; if not verified, explain why.
- Browser/tool evidence summary when UI, integration, DB, API, logs, or production-observed behavior is involved.

RISKS:
- Remaining risks, assumptions, conflicts.

REQUESTS:
- Decisions or authorizations needed, or None.
```

---

## Exception Multi-Agent Workflow

This workflow is for the minority of tasks that passed the decision gate. If a phase below would add ceremony without reducing risk, skip the subagent and keep the work with the main agent.

### Phase 1: Read-Only Reconnaissance

- Main agent dispatches only the read-only investigations that passed the decision gate.
- A reports architecture boundaries, dependencies, forbidden zones.
- B reports data/API/DB state, contract drafts, migration risks.
- C reports frontend impact, component boundaries, interaction risks.
- D reports current tests, gaps, acceptance strategy, regression scope.
- Main agent summarizes implementation plan, ownership, serial/parallel tasks, and acceptance criteria.

### Phase 2: Contract Freeze

- B proposes data/API contracts.
- C designs only against approved contracts.
- D defines acceptance and test strategy against approved contracts.
- E may review the proposed acceptance target for Strict Path or high-impact work before implementation starts.
- A reviews architecture boundaries.
- Main agent approves contract before implementation.

### Phase 3: Controlled Implementation

- Main agent or Agent T writes failing tests first when implementation requires tests.
- B implements data/API/DB-owned tasks.
- C implements frontend-owned tasks.
- A reviews architecture consistency.
- Extra agents may be dispatched only with full task packages and the same boundary rules.

### Phase 4: Review And Integration

- Main agent checks each subagent report, changed files, scope boundaries, and evidence.
- Each implementation task must pass spec review and code quality review before integration.
- Out-of-scope changes are rejected unless explicitly reauthorized and reworked.
- UI/frontend implementation must include browser evidence from Chrome DevTools MCP or an equivalent browser harness before acceptance.
- Main agent runs target tests, relevant regression tests, type checks, lint/build/format, or project-required verification.
- Strict Path work must pass Agent E evaluator acceptance review before final reporting.

### Phase 5: Summary

- Subagents do not edit the summary document directly.
- Main agent writes reviewed summaries to `docs/agent-work-summary.md` at phase end or task-batch end.
- Main agent records final integration conclusion, verification evidence, and remaining risks.

---

## Long-Running Context Rot Protection

Long-running work must use external memory files, not chat context.

Before any new session continues work, before each long-running batch, and before dispatching new agents after a pause, the main agent must read:

1. `docs/current-state.md`
2. `docs/requirements-matrix.md`
3. `docs/task-queue.md`
4. `docs/decisions.md`
5. `docs/open-questions.md`
6. `docs/context-checkpoints.md`
7. `docs/sprint-contract.md`
8. `docs/tooling-and-mcp-registry.md` when selecting tools or evidence
9. `docs/agent-mistake-ledger.md` before debugging retries or when mistakes may apply
10. `docs/agent-work-summary.md` when historical evidence is needed

Main agent must update:

- `docs/current-state.md` after each phase, batch, or meaningful state change.
- `docs/requirements-matrix.md` when requirement status or evidence changes.
- `docs/task-queue.md` when tasks become ready, blocked, in progress, done, or discarded.
- `docs/decisions.md` when a product/architecture/technical decision is confirmed.
- `docs/open-questions.md` when a question is discovered, answered, or no longer relevant.
- `docs/context-checkpoints.md` every 60-90 minutes, every 2-3 completed tasks, after interruption recovery, and before cross-session continuation.
- `docs/sprint-contract.md` when Strict Path goal, non-goals, scope, acceptance, or required evidence changes.
- `docs/agent-mistake-ledger.md` when a mandatory-record mistake occurs: wrong root cause, wrong fix, repeated failure, missed verification with success claim, material user correction, regression, or scope violation.
- `docs/evidence/**` when durable evidence is needed for Strict Path, important UI verification, bug root cause, failed hypotheses, or multi-agent integration.
- `docs/agent-work-summary.md` after reviewed multi-agent batches.

A checkpoint must record what was re-read, which control files were updated, what is still true, what drift was detected, next tasks, verification evidence, and residual risks.

Completion gate: before final reporting for any long-running, cross-session, interrupted, Strict Path, or multi-agent task, the main agent must update every affected owner file. Do not update unrelated control files just to satisfy a checklist. If current state, requirements matrix, task queue, decisions, open questions, checkpoint, sprint contract, mistake ledger, or evidence files are stale relative to the work just performed, the main agent must not announce phase completion until those files are synchronized or the reason no update was needed is recorded.

---

## Interrupted Multi-Agent Recovery

When a previous multi-agent run stopped because of network failure, abort, crash, timeout, context loss, tool-session loss, or unknown interruption, the new run must not directly continue implementation. It must first enter read-only recovery.

### Recovery Principles

- Recoverable evidence is file state, Git status, summary documents, test results, logs, task packages, and explicit saved outputs.
- Unpersisted subagent thoughts, plans, and claims are treated as lost.
- Old subagents are not trusted as long-running workers. Only persisted, reviewed, non-conflicting outputs may be adopted.
- Before closing, discarding, or replacing old subagents, main agent must state the basis: recovered, unknown, lost, conflicting, or untrusted.
- Recovery is read-only until the audit is complete. Do not continue writing code, dispatch implementation agents, commit, merge, or claim completion before recovery finishes.

### Recovery Steps

1. Re-read protocol files, control files, harness registry, mistake ledger, and relevant evidence.
2. Inspect Git/file state, changed files, untracked files, recent modification times, diffs, logs, temp outputs, test artifacts, and task packages.
3. Assign each known previous agent one status: `RECOVERED`, `PARTIAL`, `UNKNOWN`, `CONFLICTING`, or `DISCARDED`.
4. Check every previous `CHANGED_FILES` entry against authorized scope; unauthorized changes are failed tasks and cannot be directly integrated.
5. Run minimal verification needed to assess recoverable state; record commands that cannot run and why.
6. Update `docs/agent-work-summary.md` with a recovery section.
7. Replan current dispatch from recovered file/document state. New subagents receive fresh complete task packages.

### Recovery Template

```markdown
## Interrupted Multi-Agent Recovery

### Recovery Trigger
- network | aborted | crash | timeout | context loss | tool-session loss | unknown

### Recovered Evidence
- Summary docs:
- Git / file diff:
- Logs / task outputs:
- Verification:

### Previous Agent Status
- Agent A: RECOVERED | PARTIAL | UNKNOWN | CONFLICTING | DISCARDED
- Agent B: RECOVERED | PARTIAL | UNKNOWN | CONFLICTING | DISCARDED
- Agent C: RECOVERED | PARTIAL | UNKNOWN | CONFLICTING | DISCARDED
- Agent D: RECOVERED | PARTIAL | UNKNOWN | CONFLICTING | DISCARDED
- Agent E: RECOVERED | PARTIAL | UNKNOWN | CONFLICTING | DISCARDED
- Temporary agents:

### Decisions
- Adopted:
- Rejected:
- Needs user decision:

### Risks
-

### Next Dispatch Plan
-
```

---

## Final Verification Checklist

Before main agent announces completion:

- Task package check: every subagent task had mission, role, read/write scope, required skills, forbidden actions, inputs, and output format.
- Boundary check: no out-of-scope changes were accepted; any actual violation was failed and not directly integrated.
- Context check: current-state, requirements matrix, task queue, decisions, open questions, and checkpoint files are updated where relevant; completion is blocked until affected control files are synchronized or explicitly marked as not changed.
- Sprint contract check: Strict Path scope, non-goals, acceptance, and required evidence are current.
- Mistake ledger check: mandatory-record mistakes and other high-risk mistakes were recorded and prevention was decided.
- Tooling check: required MCP/tool evidence from `docs/tooling-and-mcp-registry.md` was used or the gap was reported.
- UI/browser check: user-facing UI changes have Chrome DevTools MCP or equivalent browser evidence.
- Evaluator check: Strict Path work has Agent E acceptance status before final reporting.
- Recovery check: interrupted runs completed recovery before continuation.
- Summary check: only main agent updated `docs/agent-work-summary.md`.
- Integration check: main agent reviewed all changed files, evidence, risks, and conflicts.
- Verification check: relevant target tests, regression tests, type checks, lint/build/format ran or exact blockers and residual risks were reported.
- Conflict check: any shared-file conflicts were resolved serially or ownership was redrawn.

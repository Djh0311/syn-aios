# Sprint Contract

Purpose: Strict Path owner file for freezing the current implementation slice before substantial work begins. It owns goal, non-goals, allowed/forbidden scope, acceptance contract, and completion conditions. This prevents scope drift, unclear completion, and accidental expansion during long-running or Strict Path work.

Use this file for Strict Path tasks: cross-module changes, DB/API/auth/security/payment/migration/deployment/data-integrity changes, long-running work, multi-agent execution, production bugs, failed-fix retries, and any task where acceptance criteria could drift. Do not use it for ordinary Fast Path or Standard Path work unless the user asks.

Status values: `Draft`, `Active`, `Completed`, `Superseded`, `Paused`.

---

## Current Contract

Status: Draft
Last Updated: TBD
Owner: Main Agent

### Goal

TBD

### Non-Goals

- TBD

### Requirements

| ID | Requirement | Acceptance Criteria | Evidence Needed |
| --- | --- | --- | --- |
| R-001 | TBD | TBD | TBD |

### Allowed Change Scope

- TBD

### Forbidden Change Scope

- TBD

### Required Tools And Harnesses

- TBD

### Required Skills

- TBD

### Verification Plan

- Unit / integration:
- Build / type / lint:
- UI / browser:
- API / DB:
- Evaluator:

### Completion Conditions

- TBD

### Open Questions

- TBD

---

## Update Rules

- Set status to `Active` before implementing Strict Path work.
- Link requirements to `docs/requirements-matrix.md`.
- Record scope changes as decisions in `docs/decisions.md`.
- Do not silently expand allowed change scope during implementation.
- If the user changes goals mid-work, update this contract before continuing.
- Mark `Completed` only after verification evidence exists and all completion conditions are satisfied or explicitly deferred.
- Do not duplicate detailed task packages; link to `docs/task-queue.md`.

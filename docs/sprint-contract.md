# Sprint Contract

Purpose: Strict Path owner file for freezing the current implementation slice before substantial work begins. It owns goal, non-goals, allowed/forbidden scope, acceptance contract, and completion conditions. This prevents scope drift, unclear completion, and accidental expansion during long-running or Strict Path work.

Use this file for Strict Path tasks: cross-module changes, DB/API/auth/security/payment/migration/deployment/data-integrity changes, long-running work, multi-agent execution, production bugs, failed-fix retries, and any task where acceptance criteria could drift. Do not use it for ordinary Fast Path or Standard Path work unless the user asks.

Status values: `Draft`, `Active`, `Completed`, `Superseded`, `Paused`.

---

## Current Contract

Status: Active
Last Updated: 2026-06-18
Owner: Main Agent

### Goal

Execute `docs/plans/2026-06-18-conversation-module-native-execution-plan-v1.md` starting with P0, then proceed only after P0 contract facts are clear: Codex-only conversation module, existing-session send, reply readback, new conversation, and Codex-native rendering through P3.

### Non-Goals

- Do not implement P4 true streaming in this slice.
- Do not loosen Codex sandbox, approval, project-root, or prompt-boundary safeguards.
- Do not unlock non-relay real execution paths.
- Do not read auth files, token files, `.env`, or private Codex session bodies without exact user approval for that probe.
- Do not stage or commit without explicit user confirmation.

### Requirements

| ID | Requirement | Acceptance Criteria | Evidence Needed |
| --- | --- | --- | --- |
| CM-P0-001 | Verify installed Codex CLI contract before implementation | Current `codex` version and supported flags are recorded; any unverified external-service probe is explicitly marked | `docs/evidence/2026-06-18-conversation-module-native-p0-contract-inventory-v1.md` |
| CM-P0-002 | Inventory existing conversation assets | Existing transcript parser, relay runner, Tauri commands, frontend shell, composer, and rendering gaps are recorded with file references | `docs/evidence/2026-06-18-conversation-module-native-p0-contract-inventory-v1.md` |
| CM-P1-001 | Existing-session send loop | Any selected Codex session with a verified project root can send, show user message, receive assistant reply, finish/unlock, stop if running, and allow a second send | `docs/evidence/2026-06-18-conversation-module-native-p1-event-reply-loop-v1.md` for event-reply implementation; real Tauri acceptance still needed |
| CM-P2-001 | New Codex conversation | User can create a new Codex session from a selected project cwd and then continue it | Future P2 implementation evidence |
| CM-P3-001 | Codex-native transcript rendering | Transcript rendering uses flat Codex-style text/tool/reasoning/compaction/diff views instead of chat bubbles | Future P3 implementation evidence |

### Allowed Change Scope

- P0: `docs/sprint-contract.md` and `docs/evidence/**` only.
- P1, after P0 acceptance: `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`, `codex_local_runner.rs` as needed, `commands.rs`, `command_registry.rs`, frontend Tauri bridge, `AgentConversationShell.tsx`, `AgentChatComposer.tsx`, related types/tests.
- P2, after P1 acceptance: same conversation send surface plus session creation entry/list updates.
- P3, after P2 acceptance: `codex_transcript.rs`, `TranscriptViews.tsx`, `conversationEngine.ts`, styles, and focused tests for rendering/view-model mapping.

### Forbidden Change Scope

- No `.codex` writes except via explicitly user-approved Codex execution probes or product relay execution.
- No direct reads of `auth.json`, `.env`, keys, tokens, authorization files, or private transcript bodies without exact user approval.
- No `--dangerously-bypass-approvals-and-sandbox`, `--yolo`, `--full-auto`, or approval-bypass args.
- No unrelated workflow, memory, database, harness, or legacy execution refactors.
- No broad formatting churn.

### Required Tools And Harnesses

- `codex --version` / `codex exec --help` for local CLI surface checks.
- `rg`, `sed`, and targeted source reads for asset inventory.
- `cargo test --lib manual_relay` and targeted transcript tests for P1/P3 code changes.
- Frontend offline interaction tests and real Tauri/manual UI verification for user-facing stages.
- `node scripts/harness/guard-state-files.js --target .` before/after protected doc changes when practical.

### Required Skills

- `using-superpowers`
- `executing-plans`
- `test-driven-development` for behavior-changing P1/P2/P3 code.
- `ui-browser-verification` for UI completion claims.
- `verification-before-completion` before claiming a phase complete.
- `evaluator-acceptance-review` before closing Strict Path work.

### Verification Plan

- Unit / integration: targeted Rust tests for relay and transcript parser; frontend offline tests around send state when product code changes.
- Build / type / lint: relevant `cargo check` / `cargo test`, frontend typecheck/build where touched.
- UI / browser: real Tauri or approved browser verification for P1/P2/P3 UI claims; P0 has no UI change.
- API / DB: no DB/schema changes planned.
- Evaluator: independent review/evaluator acceptance before marking P1/P2/P3 complete.

### Completion Conditions

- P0 is complete only when the CLI surface, event-stream status, rollout/parser status, existing asset inventory, and P1 interface proposal are documented.
- P1 is complete only after real app verification shows: send -> user message -> assistant reply -> terminal unlock -> second send.
- P2 is complete only after a new session can be created, replied to, persisted, rediscovered, and resumed.
- P3 is complete only after side-by-side Codex-style rendering checklist is satisfied.

### Open Questions

- Resolved 2026-06-18: user approved one live `codex exec --json --ephemeral` probe. It was run from `/private/tmp/codex-json-probe-empty` with `--ignore-rules --skip-git-repo-check --sandbox read-only`; observed `thread.started`, `turn.started`, `item.completed.item.type="agent_message"`, and `turn.completed.usage`.
- Current blocker: real `~/.codex/sessions/**.jsonl` body schema was not read in this P0 pass; exact approval is needed before reading private rollout bodies, or use a newly generated approved fixture.

---

## Update Rules

- Set status to `Active` before implementing Strict Path work.
- Link requirements to `docs/requirements-matrix.md`.
- Record scope changes as decisions in `docs/decisions.md`.
- Do not silently expand allowed change scope during implementation.
- If the user changes goals mid-work, update this contract before continuing.
- Mark `Completed` only after verification evidence exists and all completion conditions are satisfied or explicitly deferred.
- Do not duplicate detailed task packages; link to `docs/task-queue.md`.

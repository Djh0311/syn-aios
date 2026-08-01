# product-line Authority

schema: harness-authority/v2
project: product-line
updated-at: 2026-08-01T22:10:00+08:00

## Canonical

- project-rules: AGENTS.md
- current-state: docs/harness/CURRENT.md
- active-authority: docs/harness/CURRENT.md
- master-plan: docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md
- stage-plan: docs/plans/2026-08-01-syn-stage-1-contracts-and-security-foundation-plan-v1.md
- code-map: docs/code-map/index.json
- mistake-ledger: none

## Precedence

1. Current user instruction.
2. Explicitly activated v0.5 task node plus its matching active package.
3. This index, CURRENT, current decisions and current plans.
4. Project rules, source and fresh direct verification.
5. Partial Code Map.
6. Historical decisions, plans, tasks, handoffs and evidence.

## Current execution boundary

- No task is active; current plans grant no execution.
- First activatable slice: contract-only `SYN-FND-001`; product code needs a matching v0.5 node/package.
- No App, real store/message/workflow/connector, product-code or Git write from this state.
- Preserve dirty WIP; no reset, clean, stash, bulk staging or blanket attribution.

## Direct current index

- Whole-workbench model: `decisions/2026-08-01-whole-workbench-event-driven-operating-model-amendment-v1.md`.
- Memory/Skill amendment: `decisions/2026-08-01-memory-self-capture-daily-consolidation-and-skill-governance-amendment-v1.md`.
- Role entries: `decisions/2026-07-10-entry-direction-project-internal-no-second-entry-v1.md`; `decisions/2026-07-10-jarvis-global-capability-and-five-domain-vision-v1.md`.
- Target architecture: `docs/workbench-system-architecture-v1.md`. Current source inventory: `docs/2026-07-08-workbench-current-feature-inventory-for-prototype-v1.md`.
- Stage route: `docs/plans/README.md`; only M1 is current, M2-M10 are not active.
- UI boundary: `docs/workbench-frontend-display-boundary-v1.md`. Organization: `docs/own-agent-and-company-vision-v1.md`.
- Project interaction chain: `decisions/2026-07-14-interaction-model-canon-v1.md` → `decisions/2026-07-16-conversation-first-direction-ratified-v1.md` → `decisions/2026-07-18-conversation-substrate-correction-freeform-supervisor-plus-tools-v1.md` → `decisions/2026-07-19-s1b-supervisor-transport-oneshot-resume-amendment-v1.md` → `decisions/2026-07-22-shared-conversation-transport-and-syn-mcp-capability-plane-v1.md`.
- Orchestration: `decisions/2026-07-13-orchestration-and-governance-two-axis-routing-v1.md`.
- Memory/knowledge topic authorities: `docs/memory-layer-consolidated-canon-v1.md`; `decisions/2026-06-25-memory-source-type-canon-vs-code-drift-v1.md`; `decisions/2026-07-20-knowledge-vault-first-slice-v1.md`; `decisions/2026-07-21-knowledge-vault-audit-production-write-path-v1.md`; `decisions/2026-07-23-l3-syn-native-knowledge-workspace-route-v2.md`.
- External design inputs only: `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-blueprint-v1.md`; `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-ui-blueprint-v1.md`.

## Scope notes

- Git baseline HOLD: current plan/amendment set is untracked worktree state; recheck before activation/handoff.
- Secretary and Global Supervisor are top-level entries; Project Supervisors live inside explicit projects. “Jarvis” is only a historical Secretary alias.
- Plans describe targets; inventory and direct verification describe current capability. Offline proof is not desktop or release acceptance.
- Stopped route: `decisions/2026-08-01-jiaoban-stop-patching-and-bounded-rebuild-v1.md`; R7 and older schedules stay inactive.
- The interaction chain is project-internal; later amendments narrow earlier details.
- Current decisions override conflicting details in external blueprints.
- Code Map is partial and currently `UNKNOWN`; it cannot prove capability or acceptance.
- Git writes and `.git/adaptive-harness/` changes remain unauthorized.

# Context Checkpoints

Purpose: periodic anti-context-rot snapshots for long-running work. Add a checkpoint every 60-90 minutes, every 2-3 completed tasks, after interruption recovery, and before cross-session continuation.

---

## YYYY-MM-DD HH:mm Checkpoint Template

### Trigger

- Scheduled | Completed task batch | New session | Interrupted recovery | Before dispatch | Before completion

### Re-Read

- `AGENTS.md`
- `codex-multi-agent-safe-collaboration.md`
- Relevant `skills/<name>/SKILL.md`
- `docs/current-state.md`
- `docs/requirements-matrix.md`
- `docs/task-queue.md`
- `docs/decisions.md`
- `docs/open-questions.md`
- `docs/context-checkpoints.md`
- `docs/agent-work-summary.md` when needed

### Control Files Updated

- `docs/current-state.md`: yes/no, reason
- `docs/requirements-matrix.md`: yes/no, reason
- `docs/task-queue.md`: yes/no, reason
- `docs/decisions.md`: yes/no, reason
- `docs/open-questions.md`: yes/no, reason
- `docs/context-checkpoints.md`: yes/no, reason

### Still True

- TBD

### Drift Detected

- TBD or None

### Requirement Status Changes

- TBD or None

### Task Queue Changes

- TBD or None

### Verification

- Command/check: TBD
- Result: TBD

### Risks / Residual Concerns

- TBD or None

### Next 1-3 Tasks

1. TBD
2. TBD
3. TBD

---

## Checkpoint Log

## 2026-06-14 Harness HG-1/HG-3/HG-2 batch (worktree `harness-hg`)

### Trigger

- Completed task batch (3/3) + Before completion (awaiting review line + merge).

### Scope / isolation

- Branch `harness-hg`, worktree `/Users/yoyi/workspace/product-line-harness-hg` off `main`@70f5557. All work in `scripts/harness/` + `docs/` + `AGENTS.md` + `tasks/`. Product code (`prototypes/`, `src-tauri/`) untouched — isolated from the product main line.

### Commits (this branch only; NOT merged)

1. `dd0e372` HG-1: `docs/harness-catalog.md` (79-script index) + AGENTS.md pointer; brought audit reports into branch as data source.
2. `b72c36f` HG-3: warning-only U-Gate dedup check in `workbench-shape-gate.js` + `workbench-shape-gate.dedup.selftest.js`.
3. `8fe77ff` HG-2: wired 5 tool groups (one AGENTS.md call-point each) + `capability-map` retired in catalog.

### Verification (fresh, raw output in each task package)

- HG-1: catalog rows = `ls` (66+13=79), 0 missing/0 stale, pointer at AGENTS.md, `git diff --check` clean.
- HG-3: gate 485 lines (<500); current code `pass`, dedup warnings 0, deferred-whitelisted 12; existing findings byte-identical to pre-change (summary pass/0/0/9); self-test 8/8.
- HG-2: 14/14 scripts ran clean (exit 0, dry-run default, no writes), 0 deferred; catalog 已接 14 / 未接 42 / 退役 1 (=79); shape-gate still `pass`.

### Boundaries held

- hooks.enabled / ci.required still `false` (not opened). No script deleted (`capability-map` file kept). No existing AGENTS.md rule rewritten (only call-point references added). No existing ratchet/waterline check in shape-gate changed. No product code / `~/.codex` / real execution / backlog thaw.

### Risks / Residual

- HG-2 call-points are manual (hooks off) → enforcement depends on humans following AGENTS.md. HG-2 only exercised default read-only/dry-run paths (`--write` paths unverified). agentmemory 9 left `休眠·待定` (user decides permanent retirement). HG-1 invocation flags partly inferred (not every script `--help`-checked).

### Next

1. Review line verifies artifacts + merges `harness-hg` into `product-line` (executor must NOT self-merge).
2. (Optional, post-merge) decide agentmemory permanent retirement; verify `--write` paths if desired.

---

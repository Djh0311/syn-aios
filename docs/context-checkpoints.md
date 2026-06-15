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

## 2026-06-15 Harness C4 checkpoint-audit (worktree `c4-checkpoint-audit`)

### Trigger / scope

- C1 gate satisfied (closed at `1356378`); A merged; C4 doesn't depend on B/R3. Worktree off `main`@f50848b. Codex live & dirty in `src-tauri` (B1) — isolated by worktree; not touched.

### Commits (branch `c4-checkpoint-audit`, NOT merged)

- `3acb95b` C4: `scripts/harness/checkpoint-audit.js` + `checkpoint-audit.selftest.js` + AGENTS.md call-point + catalog (2 rows, 80→82). (Rebased onto main `370acd3` after B1 landed; branch diff vs main shows only the 6 C4 files.)

### Verification (fresh, raw)

- selftest: **16/16** (good→PASS; forged absent-commit/dirty-tree/out-of-bounds/no-STATUS → FAIL).
- DEMO A — real product pkg R-U5, `--package` auto, gates ON → **VERDICT PASS** (commits_reachable/tree_clean/review_status[CLEAR]/current_md_refs/gates_green pass; files_within_allow NA-honest).
- DEMO B — R-U5 `--allow 'prototypes/**,evidence/**,handoffs/**'` → files_within_allow **PASS** (13 files in-bounds).
- DEMO C — forged: R-U5 `--commit deadbeefdeadbeef` → commits_reachable **FAIL** (MISSING) → **VERDICT FAIL** (exit 1). [一句话判据 satisfied]
- DEMO D — harness HG-3 `--commit b72c36f --review <hg-batch-review> --allow 'scripts/harness/**,tasks/**'` → **VERDICT PASS** (review STATUS FINDINGS parsed; 3 files in-bounds).
- shape-gate still `pass`/0/0; catalog count = `ls scripts/harness` (82); `git diff --check` clean.

### Boundaries held

- Mechanical-only by design (printed banner + catalog row); does not judge behavior-change/pitfalls. ⑥ delegates to existing `workbench-shape-gate.js` (no verification rewrite). No product code, no shape-gate existing-check change, no hooks/CI, no script deleted, **agentmemory status untouched** (already retired on main by `f69922e`/C3 — out of C4 scope).

### Risks / Residual

- allow-list auto-parse is best-effort (prose / "本任务包" → ⑤ NA; pass `--allow` for a real boundary check). check ② verifies the *current* working tree, so audit historical packages from a clean tree. Tool is advisory (not CI-enforced) per stop-line.

### Next

1. Consultation line's pre-merge scan → user approves → merge `c4-checkpoint-audit`. Executor must NOT self-merge.

---

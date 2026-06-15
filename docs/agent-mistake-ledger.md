# Agent Mistake Ledger

Purpose: durable memory for mandatory-record agent mistakes and other mistakes likely to recur or cause real damage. This file prevents the same important mistake from being made twice without turning small corrections into process noise.

Status values: `Open`, `Encoded In Test`, `Encoded In Skill`, `Encoded In Rule`, `Accepted Risk`, `Obsolete`.

Read this file before debugging retries, failed-fix recovery, Strict Path work where known mistakes may apply, fixing review feedback caused by agent error, and before finishing any task that involved a wrong turn likely to require prevention.

---

## Entry Template

```markdown
## M-0001: Short Mistake Title

Date:
Task / Requirement:
Affected Area:
Detected By:

### Symptom

- What was observed?

### Wrong Assumption

- What did the agent believe that was false?

### Wrong Action

- What incorrect change, claim, or diagnosis happened?

### Actual Root Cause

- What was actually true?

### Detection Evidence

- Command, screenshot, log, review finding, or user correction:

### Correct Fix

- What fixed the real issue?

### Regression Protection

- Test/check added:
- Evidence location:

### Prevention

- Skill/rule/checklist to update:
- New guardrail:

Status: Open
```

---

## Recording Rules

- Add an entry when an agent fixes the wrong bug, misidentifies root cause, changes a symptom instead of the source, introduces a regression, claims success without required verification, violates read/write scope, or repeats a known mistake.
- Add an entry when the user has to correct a factual assumption that materially affected implementation, debugging, scope, or completion claims.
- Add an entry for other mistakes when they are likely to recur or cause real damage.
- Do not add an entry for wording/style corrections, harmless formatting issues, immediately corrected minor misunderstandings, or exploratory dead ends that produced no wrong change and no wrong success claim unless the same pattern repeats.
- If an error is testable, add or update a regression test and set status to `Encoded In Test` after verification.
- If an error is caused by weak process, update the relevant `SKILL.md`, `AGENTS.md`, or task template and set status to `Encoded In Skill` or `Encoded In Rule` after the rule is changed.
- If the same mistake class appears twice, treat it as a process failure and update the relevant skill before continuing similar work.
- Link evidence to `docs/evidence/` when the mistake involved logs, screenshots, browser traces, command output, or before/after behavior.

---

## Active Mistakes

## M-0003: Copied Previous Window Result Into B3b Execution Record

Date: 2026-06-16
Task / Requirement: R3 Level B / B3b controlled observation ledger closure
Affected Area: Evidence ledger / execution-record accuracy
Detected By: User correction before commit

### Symptom

- `execution-record.json` for B3b contained a `read_cut_results[0]` flag-off entry claiming `feature_flag_disabled_fallback`.
- B3b runner actually ran only Pass A hash discovery and Pass B flag-on DB limited observation.
- No flag-off B3b report or artifact existed.

### Wrong Assumption

- The agent reused the B2b execution-record shape and assumed the flag-off / flag-on pair applied to B3b as well.

### Wrong Action

- Wrote a result entry for an execution that did not happen.
- Used the copied field name `read_cut_results` for an observation window.

### Actual Root Cause

- The B3b observation window was not the same execution shape as B2b read-cut.
- The evidence ledger was templated from the previous window without proving each result row against a real runner pass and artifact.

### Detection Evidence

- User correction on 2026-06-16: B3b only had one report `9cd28f...`, and no flag-off run or product existed.

### Correct Fix

- Replace `read_cut_results` with `observation_results`.
- Remove the fake flag-off result.
- Keep only the true flag-on `stable_verified` observation result and its two stable samples.

### Regression Protection

- Test/check added: process check only; B3b review was rerun with explicit requirement to match every result row to a real run and artifact.
- Evidence location: `evidence/r3-level-b/b3-observation-20260615-225700/review-parfit-v1.md`.

### Prevention

- Before closing any execution record, verify every result entry answers: "Was this actually run?" and "Which artifact proves it?"
- Do not copy result array names across windows unless the execution shape is identical.
- If an execution record is template-derived, list every copied result field and delete the ones that do not have a matching runner pass and artifact.

Status: Open

---

## M-0002: Markdown Backticks Triggered Shell Command Substitution During H3-B Scan

Date: 2026-06-07
Task / Requirement: Stage H / H3-B authority entry sync and boundary scan
Affected Area: Verification command safety / sensitive `.codex` boundary
Detected By: Self-review after scan command output

### Symptom

- A verification scan intended to search for misleading H3-B completion wording unexpectedly attempted to run `codex exec` because Markdown backticks inside a double-quoted shell argument were interpreted by the shell.
- The command had no stdin prompt and failed to initialize state at `/Users/yoyi/.codex` because the database was readonly.

### Wrong Assumption

- The agent assumed Markdown backticks inside a double-quoted `rg` pattern would remain literal search text.

### Wrong Action

- Ran a shell command with unescaped backticks in a double-quoted search pattern.

### Actual Root Cause

- In shell, backticks perform command substitution inside double quotes. Search patterns containing Markdown code spans must be single-quoted, escaped, or passed as fixed strings safely.

### Detection Evidence

- Command output included `Reading prompt from stdin...`, `No prompt provided via stdin.`, and readonly database warnings for `/Users/yoyi/.codex/state_5.sqlite`.

### Correct Fix

- Stop using double-quoted shell search patterns when the pattern contains Markdown backticks.
- Re-run boundary scans with single-quoted patterns or `rg -F` in separate commands.

### Regression Protection

- Test/check added: process check only; subsequent H3-B scans must use single-quoted or fixed-string patterns.
- Evidence location: current H3-B authority sync turn output.

### Prevention

- For verification scans over Markdown text, use single quotes around patterns by default.
- If searching for text containing backticks, prefer `rg -F 'literal text' ...` or split into simpler literal scans.

Status: Open

---

## M-0001: Read Codex Skill File During G1 Boundary-Limited Work

Date: 2026-06-07
Task / Requirement: Stage G / G1 Runtime Log Boundary And Minimal Store
Affected Area: Scope / sensitive path boundary
Detected By: Self-review during G1 browser verification setup

### Symptom

- The work was instructed not to read or write `/Users/yoyi/.codex`.
- During browser verification setup, `/Users/yoyi/.codex/skills/playwright/SKILL.md` was read.

### Wrong Assumption

- The agent treated the local skill file as a harmless tool instruction source despite the explicit `.codex` path ban in this task.

### Wrong Action

- Read `/Users/yoyi/.codex/skills/playwright/SKILL.md`.

### Actual Root Cause

- Task-specific path restrictions override normal skill lookup habits.

### Detection Evidence

- G1 evidence and handoff record the deviation.

### Correct Fix

- Stop reading `.codex` paths for this task.
- Use bundled workspace dependencies instead.
- Report the deviation explicitly.

### Regression Protection

- Test/check added: not testable as product behavior.
- Evidence location: `evidence/2026-06-07-stage-g-g1-runtime-log-boundary-and-minimal-store-v1.md`.

### Prevention

- Before using a skill or plugin file, check whether the task forbids its path.
- If a task bans `.codex`, do not read skill files under `.codex`; use already provided tool metadata or workspace dependencies.

Status: Open

---

## Closed Mistakes

- None yet.

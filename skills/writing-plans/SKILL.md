---
name: writing-plans
description: Use when you have a spec or requirements for a multi-step task, before touching code
---

# Writing Plans

## Overview

Write comprehensive implementation plans for multi-step work. Document the required context, read/write scope, files to touch, tests/checks, and completion evidence. Give the work as bite-sized tasks. DRY. YAGNI. TDD where required by the risk router. Do not include `git add` or `git commit` steps; commits require explicit user confirmation at the end of a turn or execution phase.

**Context:** Use a dedicated worktree only when isolation is required and Git/worktrees are available.

**Save plans to:** `docs/plans/YYYY-MM-DD-<feature-name>.md` in installed projects.

**Standard harness source package exception:** when planning changes to the rule/harness package itself, save plans to `plans/YYYY-MM-DD-<feature-name>.md`. Do not use `docs/plans/**` inside the source package; `docs/**` is reserved for installed project runtime state.

## Bite-Sized Task Granularity

**Each step is one action (2-5 minutes):**
- "Write the failing test" - step
- "Run it to make sure it fails" - step
- "Implement the minimal code to make the test pass" - step
- "Run the tests and make sure they pass" - step
- "Record verification evidence" - step

## Plan Document Header

**Every plan MUST start with this header:**

```markdown
# [Feature Name] Implementation Plan

> **For agents:** Required skill when executing this plan: `executing-plans`.

**Goal:** [One sentence describing what this builds]

**Architecture:** [2-3 sentences about approach]

   **Tech Stack:** [Key technologies/libraries]

   **Path:** Fast | Standard | Strict

   **Read Scope:** [Allowed files/directories]

   **Write Scope:** [Allowed files/directories, or None]

---
```

## Task Structure

```markdown
### Task N: [Component Name]

**Files:**
- Create: `exact/path/to/file.py`
- Modify: `exact/path/to/existing.py:123-145`
- Test: `tests/exact/path/to/test.py`

**Step 1: Write the failing test**

```python
def test_specific_behavior():
    result = function(input)
    assert result == expected
```

**Step 2: Run test to verify it fails**

Run: `pytest tests/path/test.py::test_name -v`
Expected: FAIL with "function not defined"

**Step 3: Write minimal implementation**

```python
def function(input):
    return expected
```

**Step 4: Run test to verify it passes**

Run: `pytest tests/path/test.py::test_name -v`
Expected: PASS

**Step 5: Record verification**

- Evidence: `pytest tests/path/test.py::test_name -v` passed.
- Unverified: None, or list exact gaps.
```

## Remember
- Exact file paths always
- Complete code in plan (not "add validation")
- Exact commands with expected output
- Include read/write scope for every task
- Reference relevant skills by name
- DRY, YAGNI, TDD where required
- Do not include `git add` or `git commit` steps

## Execution Handoff

After saving the plan, offer execution choice:

**"Plan complete and saved to `<plan-path>`. Two execution options:**

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

**Which approach?"**

**If Subagent-Driven chosen:**
- **Required skill:** Use `subagent-driven-development`
- Stay in this session
- Fresh subagent per task + code review

**If Parallel Session chosen:**
- Guide them to open new session in worktree
- **Required skill:** New session uses `executing-plans`

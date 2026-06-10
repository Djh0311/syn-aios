# Code Quality Reviewer Prompt Template

Use this template when dispatching a code quality reviewer subagent.

**Purpose:** Verify implementation is well-built (clean, tested, maintainable)

**Only dispatch after spec compliance review passes.**

```markdown
Use the available subagent/review mechanism with the template at
`skills/requesting-code-review/code-reviewer.md`.

WHAT_WAS_IMPLEMENTED: [from implementer's report]
PLAN_OR_REQUIREMENTS: Task N from [plan-file]
BASE_SHA: [commit before task, or file snapshot if not a Git repository]
HEAD_SHA: [current commit, or file snapshot if not a Git repository]
DESCRIPTION: [task summary]
```

**Code reviewer returns:** Strengths, Issues (Critical/Important/Minor), Assessment

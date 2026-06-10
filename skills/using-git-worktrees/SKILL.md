---
name: using-git-worktrees
description: Use when feature work needs an isolated Git workspace; requires safe directory selection, ignored project-local worktrees, setup/baseline verification, and no automatic commits
---

# Using Git Worktrees

## Purpose

Create an isolated workspace for feature work without polluting the main checkout, losing user changes, or accidentally committing worktree artifacts.

## When To Use

Use a worktree when:

- implementation needs isolation from current uncommitted work
- a plan or Strict Path task requires a separate workspace
- multiple branches or agents need separate checkouts
- risky experimentation should not disturb the current workspace

Do not force worktrees when Git is unavailable, the project does not support them, the task is small enough to handle safely in place, or the user prefers the current workspace.

## Core Rules

- Confirm Git/worktree availability before relying on this workflow.
- Do not overwrite or discard existing user changes.
- Project-local worktree directories must be ignored before creation.
- If no safe directory convention exists, ask the user where to create worktrees.
- Do not run `git add` or `git commit`; commits require separate explicit confirmation.
- Do not claim the worktree is ready until setup and baseline status are verified or the gaps are reported.

## Workflow

1. **Check whether isolation is needed**
   - Identify the task, path, read/write scope, and reason isolation helps.
   - Inspect current workspace status enough to avoid trampling unrelated changes.
   - If isolation is unnecessary, state that and continue in the current workspace.

2. **Choose location**
   - Prefer an existing project convention in this order:
     - `.worktrees/`
     - `worktrees/`
     - project-specific instruction in `AGENTS.md`
   - If none exists, ask the user to choose project-local or external/global storage.
   - Use clear branch/worktree names tied to the task.

3. **Verify project-local safety**
   - For `.worktrees/` or `worktrees/`, confirm the directory is ignored by Git before creation.
   - If it is not ignored, add the ignore rule only if `.gitignore` is inside the declared write scope.
   - Ask before committing any ignore-rule change.
   - External/global worktree directories do not need project `.gitignore` changes.

4. **Create the worktree**
   - Use `git worktree add` with the chosen branch/path.
   - Stop if the branch/path already exists or Git reports conflicts.
   - Report the exact path created.

5. **Run setup**
   - Detect the project package manager/build system from files already in scope.
   - Run only the setup needed for the project.
   - If setup requires network, credentials, secrets, or elevated permissions, ask or report the blocker.

6. **Verify baseline**
   - Run the project's normal baseline check when practical.
   - If baseline fails, report failures and ask whether to investigate or continue with known-bad baseline.
   - If no baseline command is known, report that the worktree exists but is not fully verified.

7. **Report handoff**
   - Path, branch, base branch if known.
   - Setup commands run.
   - Baseline verification result.
   - Unverified gaps and next safe task.

## Stop Conditions

Stop and ask when:

- current directory is not a Git repository
- worktrees are unavailable
- directory location is ambiguous
- project-local worktree directory is not ignored and `.gitignore` is outside write scope
- target path or branch already exists
- current workspace has unrelated changes that could be confused with the task
- setup would install dependencies, use network, or require credentials without approval
- baseline verification fails and the task depends on a clean baseline

## Reporting Template

```markdown
Worktree status:
- Needed because:
- Path:
- Branch:
- Ignored directory check:
- Setup:
- Baseline verification:
- Unverified:
- Next safe action:
```

## Related Skills

- `writing-plans` when creating a plan that will execute in an isolated workspace.
- `executing-plans` or `subagent-driven-development` for implementation inside the worktree.
- `finishing-a-development-branch` for merge/PR/cleanup decisions after work is verified.
